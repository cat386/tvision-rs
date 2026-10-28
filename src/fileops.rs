//! Файловые операции для файловых менеджеров: копирование, перенос и
//! удаление в отдельном потоке — с прогрессом и отменой.
//!
//! В lfm такие операции выполнялись в дочернем процессе (`fork`), а
//! родитель рисовал прогресс и останавливал потомка сигналами. Здесь — поток
//! и общая структура [`JobProgress`]: так работает и на Windows.
//!
//! ```ignore
//! let job = FileJob::start(JobKind::Copy, vec![src], Some(dest_dir), Conflict::Overwrite);
//! // в цикле интерфейса (например, в exec_dialog_idle):
//! let p = job.progress();
//! bar.set_value(p.percent() as i64);
//! if p.finished { /* показать p.errors */ }
//! ```
//!
//! Правила назначения (`dest`) — как у `cp`/`mv`:
//! * `dest` — существующий каталог: источники кладутся **внутрь** него;
//! * иначе (один источник) — `dest` — новое имя.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// Вид операции.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobKind {
    Copy,
    Move,
    Delete,
}

/// Что делать, если файл назначения уже есть.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Conflict {
    /// Заменить.
    Overwrite,
    /// Пропустить файл.
    Skip,
}

/// Состояние операции (копия для интерфейса — [`FileJob::progress`]).
#[derive(Clone, Debug, Default)]
pub struct JobProgress {
    /// Идёт подсчёт объёма (до начала работы).
    pub counting: bool,
    pub total_bytes: u64,
    pub done_bytes: u64,
    pub total_files: u64,
    pub done_files: u64,
    /// Текущий файл.
    pub current: String,
    /// Ошибки: «путь: описание».
    pub errors: Vec<String>,
    /// Пропущенные (уже существующие) файлы.
    pub skipped: u64,
    pub finished: bool,
    pub cancelled: bool,
}

impl JobProgress {
    /// Готовность в процентах (по байтам; для удаления — по файлам).
    pub fn percent(&self) -> u8 {
        let (done, total) = if self.total_bytes > 0 { (self.done_bytes, self.total_bytes) } else { (self.done_files, self.total_files) };
        if total == 0 {
            return if self.finished { 100 } else { 0 };
        }
        ((done.min(total) as u128 * 100 / total as u128) as u8).min(100)
    }
}

/// Операция над файлами в отдельном потоке.
pub struct FileJob {
    pub kind: JobKind,
    progress: Arc<Mutex<JobProgress>>,
    cancel: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl FileJob {
    /// Начать операцию. Для `Delete` `dest` не нужен.
    pub fn start(kind: JobKind, sources: Vec<PathBuf>, dest: Option<PathBuf>, conflict: Conflict) -> FileJob {
        let progress = Arc::new(Mutex::new(JobProgress { counting: true, ..Default::default() }));
        let cancel = Arc::new(AtomicBool::new(false));
        let (p, c) = (progress.clone(), cancel.clone());
        let handle = std::thread::spawn(move || {
            let mut w = Worker { p, cancel: c, conflict };
            w.run(kind, &sources, dest.as_deref());
        });
        FileJob { kind, progress, cancel, handle: Some(handle) }
    }

    /// Выполнить операцию сразу, в текущем потоке (для тестов и мелочей).
    pub fn run_blocking(kind: JobKind, sources: Vec<PathBuf>, dest: Option<PathBuf>, conflict: Conflict) -> JobProgress {
        let mut job = FileJob::start(kind, sources, dest, conflict);
        job.wait()
    }

    /// Снимок состояния.
    pub fn progress(&self) -> JobProgress {
        self.progress.lock().map(|p| p.clone()).unwrap_or_default()
    }

    /// Попросить остановиться (после текущего блока данных).
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    pub fn is_finished(&self) -> bool {
        self.progress.lock().map(|p| p.finished).unwrap_or(true)
    }

    /// Дождаться завершения.
    pub fn wait(&mut self) -> JobProgress {
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        self.progress()
    }
}

impl Drop for FileJob {
    fn drop(&mut self) {
        // Брошенная операция останавливается, а не работает «в никуда».
        if self.handle.is_some() && !self.is_finished() {
            self.cancel();
        }
    }
}

/// Куда попадёт источник `src` при копировании/переносе в `dest`.
pub fn target_path(src: &Path, dest: &Path) -> PathBuf {
    if dest.is_dir() {
        dest.join(src.file_name().unwrap_or_default())
    } else {
        dest.to_path_buf()
    }
}

/// Файлы назначения, которые уже существуют (для вопроса «заменить?»).
/// Проверяется верхний уровень: для каталога — сам каталог.
pub fn conflicts(sources: &[PathBuf], dest: &Path) -> Vec<PathBuf> {
    sources.iter().map(|s| target_path(s, dest)).filter(|t| t.symlink_metadata().is_ok()).collect()
}

/// Размер каталога (или файла): (байт, файлов). Ссылки не разыменовываются.
pub fn dir_size(path: &Path) -> (u64, u64) {
    let mut bytes = 0;
    let mut files = 0;
    let mut stack = vec![path.to_path_buf()];
    while let Some(p) = stack.pop() {
        let md = match p.symlink_metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if md.is_dir() {
            if let Ok(rd) = fs::read_dir(&p) {
                stack.extend(rd.flatten().map(|e| e.path()));
            }
        } else {
            bytes += md.len();
            files += 1;
        }
    }
    (bytes, files)
}

/// Лежит ли `inner` внутри `outer` (или совпадает с ним).
pub fn is_inside(inner: &Path, outer: &Path) -> bool {
    let a = inner.canonicalize().unwrap_or_else(|_| inner.to_path_buf());
    let b = outer.canonicalize().unwrap_or_else(|_| outer.to_path_buf());
    a.starts_with(&b)
}

/// Исполнитель в потоке.
struct Worker {
    p: Arc<Mutex<JobProgress>>,
    cancel: Arc<AtomicBool>,
    conflict: Conflict,
}

/// Размер блока копирования: прогресс и отмена проверяются после каждого.
const BLOCK: usize = 1 << 20;

impl Worker {
    fn update(&self, f: impl FnOnce(&mut JobProgress)) {
        if let Ok(mut p) = self.p.lock() {
            f(&mut p);
        }
    }

    fn stopped(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    fn error(&self, path: &Path, e: impl std::fmt::Display) {
        let s = format!("{}: {}", path.display(), e);
        self.update(|p| p.errors.push(s));
    }

    fn run(&mut self, kind: JobKind, sources: &[PathBuf], dest: Option<&Path>) {
        // Подсчёт объёма.
        let (mut bytes, mut files) = (0, 0);
        for s in sources {
            let (b, f) = dir_size(s);
            bytes += b;
            files += f.max(1);
        }
        self.update(|p| {
            p.counting = false;
            p.total_bytes = if kind == JobKind::Delete { 0 } else { bytes };
            p.total_files = files;
        });
        for s in sources {
            if self.stopped() {
                break;
            }
            match kind {
                JobKind::Delete => self.delete(s),
                JobKind::Copy | JobKind::Move => {
                    let dest = match dest {
                        Some(d) => d,
                        None => {
                            self.error(s, "не задано назначение");
                            continue;
                        }
                    };
                    let target = target_path(s, dest);
                    if s.is_dir() && is_inside(&target, s) {
                        self.error(s, "нельзя копировать каталог в самого себя");
                        continue;
                    }
                    if kind == JobKind::Move {
                        self.move_one(s, &target);
                    } else {
                        self.copy_tree(s, &target);
                    }
                }
            }
        }
        let cancelled = self.stopped();
        self.update(|p| {
            p.finished = true;
            p.cancelled = cancelled;
            p.current.clear();
        });
    }

    /// Перенос: сначала быстрое переименование, если не вышло (другой
    /// диск) — копирование и удаление.
    fn move_one(&mut self, src: &Path, target: &Path) {
        if target.symlink_metadata().is_ok() {
            if self.conflict == Conflict::Skip {
                let (_, f) = dir_size(src);
                self.update(|p| {
                    p.skipped += 1;
                    p.done_files += f;
                });
                return;
            }
            if target != src {
                let _ = remove_any(target);
            }
        }
        if fs::rename(src, target).is_ok() {
            let (b, f) = dir_size(target);
            self.update(|p| {
                p.done_bytes += b;
                p.done_files += f.max(1);
            });
            return;
        }
        let errors_before = self.progress_errors();
        self.copy_tree(src, target);
        if !self.stopped() && self.progress_errors() == errors_before {
            if let Err(e) = remove_any(src) {
                self.error(src, e);
            }
        }
    }

    fn progress_errors(&self) -> usize {
        self.p.lock().map(|p| p.errors.len()).unwrap_or(0)
    }

    fn copy_tree(&mut self, src: &Path, target: &Path) {
        if self.stopped() {
            return;
        }
        let md = match src.symlink_metadata() {
            Ok(m) => m,
            Err(e) => return self.error(src, e),
        };
        if md.file_type().is_symlink() {
            self.copy_link(src, target);
        } else if md.is_dir() {
            if let Err(e) = fs::create_dir_all(target) {
                return self.error(target, e);
            }
            let entries = match fs::read_dir(src) {
                Ok(rd) => rd,
                Err(e) => return self.error(src, e),
            };
            for e in entries.flatten() {
                if self.stopped() {
                    return;
                }
                self.copy_tree(&e.path(), &target.join(e.file_name()));
            }
            let _ = fs::set_permissions(target, md.permissions());
        } else {
            self.copy_file(src, target, &md);
        }
    }

    fn copy_link(&mut self, src: &Path, target: &Path) {
        #[cfg(unix)]
        {
            if target.symlink_metadata().is_ok() {
                if self.conflict == Conflict::Skip {
                    self.update(|p| {
                        p.skipped += 1;
                        p.done_files += 1;
                    });
                    return;
                }
                let _ = remove_any(target);
            }
            match fs::read_link(src).and_then(|to| std::os::unix::fs::symlink(to, target)) {
                Ok(()) => self.update(|p| p.done_files += 1),
                Err(e) => self.error(src, e),
            }
        }
        #[cfg(not(unix))]
        {
            // На Windows копируется то, на что указывает ссылка.
            match src.metadata() {
                Ok(md) if md.is_dir() => self.copy_tree_follow(src, target),
                Ok(md) => self.copy_file(src, target, &md),
                Err(e) => self.error(src, e),
            }
        }
    }

    #[cfg(not(unix))]
    fn copy_tree_follow(&mut self, src: &Path, target: &Path) {
        if let Err(e) = fs::create_dir_all(target) {
            return self.error(target, e);
        }
        if let Ok(rd) = fs::read_dir(src) {
            for e in rd.flatten() {
                self.copy_tree(&e.path(), &target.join(e.file_name()));
            }
        }
    }

    fn copy_file(&mut self, src: &Path, target: &Path, md: &fs::Metadata) {
        let name = src.display().to_string();
        self.update(|p| p.current = name);
        if target.symlink_metadata().is_ok() {
            if self.conflict == Conflict::Skip {
                let len = md.len();
                self.update(|p| {
                    p.skipped += 1;
                    p.done_files += 1;
                    p.done_bytes += len;
                });
                return;
            }
            if same_file(src, target) {
                return self.error(src, "файл нельзя скопировать сам в себя");
            }
        }
        match self.copy_data(src, target) {
            Ok(()) => {
                let _ = fs::set_permissions(target, md.permissions());
                if let (Ok(t), Ok(f)) = (md.modified(), fs::OpenOptions::new().write(true).open(target)) {
                    let _ = f.set_modified(t);
                }
                self.update(|p| p.done_files += 1);
            }
            Err(e) => {
                // Недописанный файл удаляем.
                let _ = fs::remove_file(target);
                if !self.stopped() {
                    self.error(src, e);
                }
            }
        }
    }

    fn copy_data(&mut self, src: &Path, target: &Path) -> io::Result<()> {
        let mut r = fs::File::open(src)?;
        let mut w = fs::File::create(target)?;
        let mut buf = vec![0u8; BLOCK];
        loop {
            if self.stopped() {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "остановлено"));
            }
            let n = r.read(&mut buf)?;
            if n == 0 {
                break;
            }
            w.write_all(&buf[..n])?;
            self.update(|p| p.done_bytes += n as u64);
        }
        w.flush()
    }

    fn delete(&mut self, path: &Path) {
        if self.stopped() {
            return;
        }
        let md = match path.symlink_metadata() {
            Ok(m) => m,
            Err(e) => return self.error(path, e),
        };
        let name = path.display().to_string();
        self.update(|p| p.current = name);
        if md.is_dir() {
            if let Ok(rd) = fs::read_dir(path) {
                for e in rd.flatten() {
                    self.delete(&e.path());
                }
            }
            if self.stopped() {
                return;
            }
            if let Err(e) = fs::remove_dir(path) {
                self.error(path, e);
            }
        } else {
            match fs::remove_file(path) {
                Ok(()) => self.update(|p| p.done_files += 1),
                Err(e) => self.error(path, e),
            }
        }
    }
}

/// Один ли это файл (по каноническому пути).
fn same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(x), Ok(y)) => x == y,
        _ => false,
    }
}

/// Удалить файл, ссылку или каталог целиком.
pub fn remove_any(path: &Path) -> io::Result<()> {
    let md = path.symlink_metadata()?;
    if md.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("tv-fileops-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn tree(root: &Path) {
        fs::create_dir_all(root.join("src/sub")).unwrap();
        fs::write(root.join("src/a.txt"), "привет").unwrap();
        fs::write(root.join("src/sub/b.bin"), vec![7u8; 3 * BLOCK + 5]).unwrap();
    }

    #[test]
    fn copy_dir_into_dir() {
        let d = tmp("copy");
        tree(&d);
        fs::create_dir(d.join("dst")).unwrap();
        let p = FileJob::run_blocking(JobKind::Copy, vec![d.join("src")], Some(d.join("dst")), Conflict::Overwrite);
        assert!(p.errors.is_empty(), "{:?}", p.errors);
        assert_eq!(fs::read_to_string(d.join("dst/src/a.txt")).unwrap(), "привет");
        assert_eq!(fs::metadata(d.join("dst/src/sub/b.bin")).unwrap().len(), (3 * BLOCK + 5) as u64);
        assert_eq!((p.done_files, p.total_files), (2, 2));
        assert_eq!(p.percent(), 100);
        // Повтор со «пропустить» ничего не меняет.
        let p = FileJob::run_blocking(JobKind::Copy, vec![d.join("src/a.txt")], Some(d.join("dst/src")), Conflict::Skip);
        assert_eq!(p.skipped, 1);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn copy_with_new_name_and_conflicts() {
        let d = tmp("rename");
        tree(&d);
        let p = FileJob::run_blocking(JobKind::Copy, vec![d.join("src/a.txt")], Some(d.join("c.txt")), Conflict::Overwrite);
        assert!(p.errors.is_empty());
        assert!(d.join("c.txt").exists());
        assert_eq!(conflicts(&[d.join("src/a.txt")], &d.join("c.txt")), vec![d.join("c.txt")]);
        assert!(conflicts(&[d.join("src/a.txt")], &d.join("nope.txt")).is_empty());
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn no_copy_into_itself() {
        let d = tmp("self");
        tree(&d);
        let p = FileJob::run_blocking(JobKind::Copy, vec![d.join("src")], Some(d.join("src/sub")), Conflict::Overwrite);
        assert_eq!(p.errors.len(), 1);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn move_and_delete() {
        let d = tmp("move");
        tree(&d);
        fs::create_dir(d.join("dst")).unwrap();
        let p = FileJob::run_blocking(JobKind::Move, vec![d.join("src")], Some(d.join("dst")), Conflict::Overwrite);
        assert!(p.errors.is_empty(), "{:?}", p.errors);
        assert!(!d.join("src").exists());
        assert!(d.join("dst/src/sub/b.bin").exists());
        let p = FileJob::run_blocking(JobKind::Delete, vec![d.join("dst")], None, Conflict::Overwrite);
        assert!(p.errors.is_empty(), "{:?}", p.errors);
        assert!(!d.join("dst").exists());
        assert_eq!(p.done_files, 2);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn cancel_stops_and_cleans() {
        let d = tmp("cancel");
        fs::write(d.join("big"), vec![1u8; 64 * BLOCK]).unwrap();
        let mut job = FileJob::start(JobKind::Copy, vec![d.join("big")], Some(d.join("copy")), Conflict::Overwrite);
        job.cancel();
        let p = job.wait();
        assert!(p.finished && p.cancelled);
        assert!(!d.join("copy").exists() || fs::metadata(d.join("copy")).unwrap().len() < 64 * BLOCK as u64);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn sizes() {
        let d = tmp("size");
        tree(&d);
        assert_eq!(dir_size(&d.join("src")), ((3 * BLOCK + 5 + "привет".len()) as u64, 2));
        assert!(is_inside(&d.join("src/sub"), &d.join("src")));
        assert!(!is_inside(&d.join("src"), &d.join("src/sub")));
        let _ = fs::remove_dir_all(&d);
    }
}
