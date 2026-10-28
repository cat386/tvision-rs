//! Файловая панель (`FilePanel`) и пара панелей (`DualPanel`) — по мотивам
//! консольного файлового менеджера [lfm](https://inigo.katxi.org/devel/lfm/)
//! (Last File Manager, Iñigo Serna).
//!
//! Что умеет панель (как в lfm):
//! * **вкладки** — у каждой свой каталог, курсор, выделение, фильтр и
//!   история; заголовки вкладок — в верхней рамке;
//! * **фильтр** вкладки — glob-шаблоны того, что *скрыть*, через запятую;
//!   `!` — исключение: `*.o,*.tmp` прячет объектные и временные файлы,
//!   `*,!*.rs` оставляет только `.rs`. Каталоги фильтр не прячет;
//! * скрытые файлы (точечные; на Windows — и с атрибутом «скрытый»);
//! * **сортировка** по имени, расширению, размеру, времени или без неё,
//!   в обратном порядке; каталоги — сверху;
//! * **выделение**: Ins/пробел, по шаблону ([`FilePanel::mark_glob`]),
//!   инверсия;
//! * **быстрый поиск**: Ctrl+S и набор имени, Alt+буква — к первому файлу на
//!   эту букву;
//! * **виртуальная панель** («panelize» lfm): список найденных файлов из
//!   разных каталогов ([`FilePanel::set_listing`]);
//! * цвета по типам файлов (каталоги, программы, ссылки, архивы,
//!   изображения, звук, видео, документы, исходники, данные, временные);
//! * размеры каталогов по запросу ([`FilePanel::calc_dir_sizes`]).
//!
//! Клавиши панели: ↑ ↓ PgUp PgDn Home End, ← или Backspace — наверх,
//! → — в каталог, Enter — в каталог или «выполнить» (команда
//! [`cm::FILE_PANEL_EXECUTE`]), Ins / пробел — выделить, Ctrl+R — перечитать,
//! Ctrl+S — поиск по имени, Alt+буква — к файлу на букву. Мышь: щелчок —
//! курсор, двойной — Enter, правая кнопка — выделить, колесо — прокрутка,
//! щелчок по вкладке в рамке — переключить.
//!
//! Остальное (копирование, переименование, диалоги) делает программа через
//! методы панели — см. пример `examples/lfm.rs` и [`crate::fileops`].

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::color::{self, attr, attr256, is_bg256, with_fg256, with_fore, xterm_rgb, Attr};
use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Info, Key, KeyEvent, Mods};
use crate::geometry::Rect;
use crate::group::Group;
use crate::locale::tr;
use crate::text::{char_width, str_width};
use crate::view::{default_handle_event, gf, of, sf, Ctx, DrawCtx, View, ViewId, ViewState};

// ----------------------------------------------------------------------
// Записи
// ----------------------------------------------------------------------

/// Вид записи.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    /// `..` — выход наверх.
    Parent,
    Dir,
    File,
    /// Символическая ссылка (`to_dir` — указывает на каталог).
    Link {
        to_dir: bool,
    },
    /// Устройство, канал, сокет…
    Other,
}

/// Класс файла для раскраски (как раздел `[Files]` в `lfm.ini`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileClass {
    Normal = 0,
    Dir,
    Exec,
    Link,
    Archive,
    Image,
    Audio,
    Video,
    Document,
    Source,
    Data,
    Temp,
}

/// Число классов (размер таблицы цветов).
pub const FILE_CLASSES: usize = 12;

/// Расширения по классам.
const EXTENSIONS: &[(FileClass, &[&str])] = &[
    (
        FileClass::Archive,
        &["zip", "rar", "7z", "tar", "gz", "tgz", "bz2", "tbz", "xz", "txz", "lz", "lz4", "zst", "arj", "lzh", "cab", "deb", "rpm", "jar", "iso"],
    ),
    (FileClass::Image, &["png", "jpg", "jpeg", "gif", "bmp", "tif", "tiff", "webp", "svg", "ico", "psd", "xcf", "pcx", "tga"]),
    (FileClass::Audio, &["mp3", "ogg", "oga", "flac", "wav", "aac", "m4a", "opus", "wma", "mid", "mod"]),
    (FileClass::Video, &["mp4", "mkv", "avi", "mov", "webm", "wmv", "mpg", "mpeg", "flv", "m4v"]),
    (
        FileClass::Document,
        &["pdf", "doc", "docx", "odt", "rtf", "txt", "md", "xls", "xlsx", "ods", "ppt", "pptx", "odp", "epub", "fb2", "djvu", "tex"],
    ),
    (
        FileClass::Source,
        &[
            "rs", "c", "h", "cpp", "hpp", "cc", "py", "pas", "pp", "go", "java", "kt", "js", "ts", "sh", "bat", "ps1", "lua", "rb", "pl", "asm",
            "cs", "swift", "zig", "toml", "mk", "cmake",
        ],
    ),
    (FileClass::Data, &["json", "xml", "yaml", "yml", "csv", "ini", "cfg", "conf", "db", "sqlite", "sql", "log", "dat", "html", "htm", "css"]),
    (FileClass::Temp, &["bak", "tmp", "temp", "swp", "old", "orig", "o", "obj", "pyc", "class"]),
];

/// Класс по имени файла (по расширению, без учёта регистра).
pub fn class_by_name(name: &str) -> FileClass {
    if name.ends_with('~') {
        return FileClass::Temp;
    }
    let ext = match name.rsplit_once('.') {
        Some((stem, e)) if !stem.is_empty() => e.to_lowercase(),
        _ => return FileClass::Normal,
    };
    for (c, list) in EXTENSIONS {
        if list.contains(&ext.as_str()) {
            return *c;
        }
    }
    FileClass::Normal
}

/// Файл в панели.
#[derive(Clone, Debug)]
pub struct PanelEntry {
    /// Показываемое имя (в виртуальной панели — путь относительно её корня).
    pub name: String,
    /// Полный путь.
    pub path: PathBuf,
    pub kind: FileKind,
    /// Размер в байтах (у каталогов — 0 или посчитанный).
    pub size: u64,
    pub modified: Option<SystemTime>,
    /// Права Unix (`0o755`); на Windows — 0o444 для «только чтение», иначе 0o666.
    pub mode: u32,
    pub executable: bool,
    pub hidden: bool,
}

impl PanelEntry {
    /// Запись по пути (`name` — что показывать).
    pub fn from_path(path: &Path, name: &str) -> PanelEntry {
        let lmd = path.symlink_metadata();
        let mut e = PanelEntry {
            name: name.to_string(),
            path: path.to_path_buf(),
            kind: FileKind::Other,
            size: 0,
            modified: None,
            mode: 0,
            executable: false,
            hidden: name.starts_with('.') && name != "..",
        };
        let lmd = match lmd {
            Ok(m) => m,
            Err(_) => return e,
        };
        let ft = lmd.file_type();
        let md = if ft.is_symlink() { path.metadata().unwrap_or_else(|_| lmd.clone()) } else { lmd.clone() };
        e.kind = if ft.is_symlink() {
            FileKind::Link { to_dir: md.is_dir() }
        } else if ft.is_dir() {
            FileKind::Dir
        } else if ft.is_file() {
            FileKind::File
        } else {
            FileKind::Other
        };
        if !md.is_dir() {
            e.size = md.len();
        }
        e.modified = lmd.modified().ok();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            e.mode = lmd.permissions().mode() & 0o7777;
            e.executable = md.is_file() && md.permissions().mode() & 0o111 != 0;
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            e.mode = if lmd.permissions().readonly() { 0o444 } else { 0o666 };
            e.hidden |= lmd.file_attributes() & 0x2 != 0;
            let ext = name.rsplit_once('.').map(|(_, x)| x.to_lowercase()).unwrap_or_default();
            e.executable = md.is_file() && ["exe", "com", "bat", "cmd", "ps1", "msi"].contains(&ext.as_str());
        }
        #[cfg(not(any(unix, windows)))]
        {
            e.mode = if lmd.permissions().readonly() { 0o444 } else { 0o666 };
        }
        e
    }

    fn parent(path: &Path) -> PanelEntry {
        PanelEntry {
            name: "..".into(),
            path: path.to_path_buf(),
            kind: FileKind::Parent,
            size: 0,
            modified: None,
            mode: 0,
            executable: false,
            hidden: false,
        }
    }

    /// Каталог (в том числе `..` и ссылка на каталог).
    pub fn is_dir(&self) -> bool {
        matches!(self.kind, FileKind::Dir | FileKind::Parent | FileKind::Link { to_dir: true })
    }

    pub fn is_parent(&self) -> bool {
        self.kind == FileKind::Parent
    }

    /// Расширение (без точки, в нижнем регистре).
    pub fn extension(&self) -> String {
        if self.is_dir() {
            return String::new();
        }
        match self.name.rsplit_once('.') {
            Some((stem, e)) if !stem.is_empty() && !stem.ends_with('/') => e.to_lowercase(),
            _ => String::new(),
        }
    }

    /// Класс для раскраски.
    pub fn class(&self) -> FileClass {
        match self.kind {
            FileKind::Parent | FileKind::Dir => FileClass::Dir,
            FileKind::Link { .. } => FileClass::Link,
            _ if self.executable => FileClass::Exec,
            _ => class_by_name(&self.name),
        }
    }

    /// Права строкой: `rwxr-xr-x` (Unix) или `rw`/`r-` (Windows).
    pub fn perms_string(&self) -> String {
        if cfg!(unix) {
            let m = self.mode;
            let bit = |b: u32, c: char| if m & b != 0 { c } else { '-' };
            [
                bit(0o400, 'r'),
                bit(0o200, 'w'),
                bit(0o100, 'x'),
                bit(0o040, 'r'),
                bit(0o020, 'w'),
                bit(0o010, 'x'),
                bit(0o004, 'r'),
                bit(0o002, 'w'),
                bit(0o001, 'x'),
            ]
            .iter()
            .collect()
        } else {
            let mut s = String::from(if self.mode & 0o200 != 0 { "rw" } else { "r-" });
            if self.hidden {
                s.push('h');
            }
            s
        }
    }
}

// ----------------------------------------------------------------------
// Шаблоны и фильтры
// ----------------------------------------------------------------------

/// Совпадает ли имя с glob-шаблоном: `*`, `?`, `[abc]`, `[a-z]`, `[!x]`.
pub fn glob_match(pattern: &str, name: &str, case_sensitive: bool) -> bool {
    let fold = |s: &str| {
        if case_sensitive {
            s.to_string()
        } else {
            s.to_lowercase()
        }
    };
    let p: Vec<char> = fold(pattern).chars().collect();
    let n: Vec<char> = fold(name).chars().collect();
    glob_rec(&p, &n)
}

fn glob_rec(p: &[char], n: &[char]) -> bool {
    let (mut pi, mut ni) = (0, 0);
    // Последняя звёздочка и место в имени, с которого она «съедает» символы.
    let mut star: Option<(usize, usize)> = None;
    while ni < n.len() {
        if pi < p.len() {
            match p[pi] {
                '*' => {
                    star = Some((pi, ni));
                    pi += 1;
                    continue;
                }
                '?' => {
                    pi += 1;
                    ni += 1;
                    continue;
                }
                '[' => {
                    if let Some((ok, len)) = class_match(&p[pi..], n[ni]) {
                        if ok {
                            pi += len;
                            ni += 1;
                            continue;
                        }
                    } else if p[pi] == n[ni] {
                        pi += 1;
                        ni += 1;
                        continue;
                    }
                }
                c if c == n[ni] => {
                    pi += 1;
                    ni += 1;
                    continue;
                }
                _ => {}
            }
        }
        match star {
            Some((sp, sn)) => {
                pi = sp + 1;
                ni = sn + 1;
                star = Some((sp, sn + 1));
            }
            None => return false,
        }
    }
    p[pi..].iter().all(|&c| c == '*')
}

/// `[...]` в начале `p`: (совпало ли с `c`, длина класса). `None` — не класс.
fn class_match(p: &[char], c: char) -> Option<(bool, usize)> {
    let end = p.iter().skip(2).position(|&x| x == ']')? + 2;
    let body = &p[1..end];
    let (neg, body) = match body.first() {
        Some('!') | Some('^') => (true, &body[1..]),
        _ => (false, body),
    };
    let mut ok = false;
    let mut i = 0;
    while i < body.len() {
        if i + 2 < body.len() && body[i + 1] == '-' {
            if body[i] <= c && c <= body[i + 2] {
                ok = true;
            }
            i += 3;
        } else {
            if body[i] == c {
                ok = true;
            }
            i += 1;
        }
    }
    Some((ok != neg, end + 1))
}

/// Фильтр вкладки: что скрыть (как в lfm). Пустой — ничего не скрывает.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileFilter {
    text: String,
    hide: Vec<String>,
    keep: Vec<String>,
}

impl FileFilter {
    /// Разобрать строку `*.o,*.tmp,!keep*`.
    pub fn parse(text: &str) -> FileFilter {
        let mut f = FileFilter { text: text.trim().to_string(), ..Default::default() };
        for part in text.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            match part.strip_prefix('!') {
                Some(k) if !k.is_empty() => f.keep.push(k.to_string()),
                Some(_) => {}
                None => f.hide.push(part.to_string()),
            }
        }
        f
    }

    /// Исходная строка.
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_empty(&self) -> bool {
        self.hide.is_empty()
    }

    /// Скрыть ли файл с этим именем.
    pub fn hides(&self, name: &str) -> bool {
        let cs = !cfg!(windows);
        self.hide.iter().any(|p| glob_match(p, name, cs)) && !self.keep.iter().any(|p| glob_match(p, name, cs))
    }
}

// ----------------------------------------------------------------------
// Форматирование
// ----------------------------------------------------------------------

/// Размер в 7 символов: `1234567`, `12.3M`, `1.20G`…
pub fn format_size_short(n: u64) -> String {
    if n < 10_000_000 {
        return n.to_string();
    }
    let units = ['K', 'M', 'G', 'T', 'P'];
    let mut v = n as f64 / 1024.0;
    let mut u = 0;
    while v >= 1000.0 && u + 1 < units.len() {
        v /= 1024.0;
        u += 1;
    }
    if v < 10.0 {
        format!("{:.2}{}", v, units[u])
    } else if v < 100.0 {
        format!("{:.1}{}", v, units[u])
    } else {
        format!("{:.0}{}", v, units[u])
    }
}

/// Размер с разделением разрядов: `1 234 567`.
pub fn format_size_full(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

/// Дата `дд.мм.гг чч:мм` (14 символов).
pub fn format_time_short(t: Option<SystemTime>) -> String {
    match t {
        Some(t) => {
            let d = crate::platform::time::local(t);
            format!("{:02}.{:02}.{:02} {:02}:{:02}", d.day, d.month, d.year.rem_euclid(100), d.hour, d.minute)
        }
        None => String::new(),
    }
}

/// Дата `дд.мм.гггг чч:мм:сс`.
pub fn format_time_full(t: Option<SystemTime>) -> String {
    match t {
        Some(t) => {
            let d = crate::platform::time::local(t);
            format!("{:02}.{:02}.{:04} {:02}:{:02}:{:02}", d.day, d.month, d.year, d.hour, d.minute, d.second)
        }
        None => String::new(),
    }
}

/// Текст ровно в `w` ячеек (обрезка с «…»).
fn fit(s: &str, w: usize, right: bool) -> String {
    let sw = str_width(s);
    if sw > w {
        if w == 0 {
            return String::new();
        }
        let mut out = String::new();
        let mut used = 0;
        for c in s.chars() {
            let cw = char_width(c);
            if used + cw > w - 1 {
                break;
            }
            out.push(c);
            used += cw;
        }
        out.push('…');
        out.extend(std::iter::repeat_n(' ', w - used - 1));
        return out;
    }
    let pad = " ".repeat(w - sw);
    if right {
        format!("{}{}", pad, s)
    } else {
        format!("{}{}", s, pad)
    }
}

/// Путь, обрезанный слева до `w` ячеек: `…/long/path`.
fn fit_path_left(s: &str, w: usize) -> String {
    if str_width(s) <= w {
        return s.to_string();
    }
    if w == 0 {
        return String::new();
    }
    let chars: Vec<char> = s.chars().collect();
    let mut used = 1;
    let mut start = chars.len();
    while start > 0 && used + char_width(chars[start - 1]) <= w {
        start -= 1;
        used += char_width(chars[start]);
    }
    format!("…{}", chars[start..].iter().collect::<String>())
}

/// Корни для выбора диска/точки монтирования: на Windows — диски `A:\`…`Z:\`,
/// на Unix — `/`, домашний каталог, `/mnt/*`, `/media/*`, `/media/$USER/*`.
pub fn available_roots() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if cfg!(windows) {
        for c in b'A'..=b'Z' {
            let p = PathBuf::from(format!("{}:\\", c as char));
            if p.exists() {
                v.push(p);
            }
        }
    } else {
        v.push(PathBuf::from("/"));
        if let Some(h) = std::env::var_os("HOME") {
            v.push(PathBuf::from(h));
        }
        let user = std::env::var("USER").unwrap_or_default();
        for base in ["/mnt".to_string(), "/media".to_string(), format!("/media/{}", user), "/Volumes".to_string()] {
            if let Ok(rd) = fs::read_dir(&base) {
                let mut items: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
                items.sort();
                for p in items {
                    if !v.contains(&p) && p != Path::new("/media").join(&user) {
                        v.push(p);
                    }
                }
            }
        }
    }
    v
}

// ----------------------------------------------------------------------
// Сортировка
// ----------------------------------------------------------------------

/// Порядок файлов.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SortMode {
    #[default]
    Name,
    Ext,
    Size,
    Time,
    /// Как вернула файловая система.
    Unsorted,
}

impl SortMode {
    /// Буква в рамке панели.
    pub fn letter(self) -> char {
        match self {
            SortMode::Name => 'n',
            SortMode::Ext => 'e',
            SortMode::Size => 's',
            SortMode::Time => 't',
            SortMode::Unsorted => 'u',
        }
    }

    pub const ALL: [SortMode; 5] = [SortMode::Name, SortMode::Ext, SortMode::Size, SortMode::Time, SortMode::Unsorted];
}

// ----------------------------------------------------------------------
// Цвета
// ----------------------------------------------------------------------

/// Цвета панели (готовые атрибуты).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilePanelColors {
    pub frame: Attr,
    pub frame_active: Attr,
    /// Заголовок (путь, вкладка) в рамке неактивной/активной панели.
    pub title: Attr,
    pub title_active: Attr,
    /// Заголовки столбцов.
    pub header: Attr,
    /// Курсор.
    pub cursor: Attr,
    /// Выделенный файл.
    pub marked: Attr,
    /// Курсор на выделенном файле.
    pub marked_cursor: Attr,
    /// Строка сведений внизу.
    pub info: Attr,
    /// Цвета классов файлов ([`FileClass`] как индекс).
    pub classes: [Attr; FILE_CLASSES],
}

impl FilePanelColors {
    /// Цвета из четырёх базовых: рамка (неактивная/активная), обычный текст,
    /// курсор. Для 256-цветного фона типы файлов раскрашиваются цветами
    /// gruvbox, для 16-цветного — цветами BIOS (как в MC и DN).
    pub fn derive(frame: Attr, frame_active: Attr, normal: Attr, cursor: Attr) -> FilePanelColors {
        if is_bg256(normal) {
            let bg = xterm_rgb(color::bg256(normal));
            let light = ((bg >> 16) & 0xFF) + ((bg >> 8) & 0xFF) + (bg & 0xFF) > 3 * 150;
            // Классы: обычный, каталог, программа, ссылка, архив, изображение,
            // звук, видео, документ, исходник, данные, временный.
            let fg: [u8; FILE_CLASSES] = if light {
                [237, 24, 100, 66, 96, 130, 132, 88, 239, 30, 136, 245]
            } else {
                [223, 109, 142, 108, 175, 208, 132, 166, 250, 72, 172, 243]
            };
            let classes = fg.map(|f| with_fg256(normal, f));
            let yellow = if light { 136 } else { 214 };
            FilePanelColors {
                frame,
                frame_active,
                title: frame,
                title_active: with_fg256(frame_active, yellow),
                header: with_fg256(normal, yellow),
                cursor,
                marked: with_fg256(normal, yellow),
                marked_cursor: attr256(if light { 230 } else { 235 }, yellow),
                info: normal,
                classes,
            }
        } else {
            let dark = matches!(color::back(normal), 0 | 1 | 4 | 5 | 8);
            let classes = if dark {
                [7u8, 15, 10, 11, 13, 12, 12, 12, 3, 2, 6, 8].map(|f| with_fore(normal, f))
            } else {
                let mut c = [normal; FILE_CLASSES];
                c[FileClass::Dir as usize] = with_fore(normal, 15);
                c[FileClass::Exec as usize] = with_fore(normal, 2);
                c[FileClass::Archive as usize] = with_fore(normal, 5);
                c
            };
            let (cur, mcur) = if dark { (attr(0, 3), attr(14, 3)) } else { (cursor, with_fore(cursor, 14)) };
            FilePanelColors {
                frame,
                frame_active,
                title: frame,
                title_active: if dark { attr(0, 3) } else { frame_active },
                header: with_fore(normal, 14),
                cursor: cur,
                marked: with_fore(normal, 14),
                marked_cursor: mcur,
                info: normal,
                classes,
            }
        }
    }
}

// ----------------------------------------------------------------------
// Вкладка
// ----------------------------------------------------------------------

/// Виртуальная панель (результаты поиска).
#[derive(Clone, Debug)]
struct Listing {
    title: String,
    base: PathBuf,
    items: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
struct Tab {
    path: PathBuf,
    entries: Vec<PanelEntry>,
    /// Видимые записи (индексы в `entries`) в порядке показа.
    view: Vec<usize>,
    cursor: usize,
    top: usize,
    /// Выделенные (по полному пути).
    marked: BTreeSet<PathBuf>,
    filter: FileFilter,
    history: Vec<PathBuf>,
    listing: Option<Listing>,
    dir_sizes: HashMap<PathBuf, u64>,
    error: Option<String>,
}

impl Tab {
    fn new(path: PathBuf) -> Tab {
        Tab {
            path,
            entries: Vec::new(),
            view: Vec::new(),
            cursor: 0,
            top: 0,
            marked: BTreeSet::new(),
            filter: FileFilter::default(),
            history: Vec::new(),
            listing: None,
            dir_sizes: HashMap::new(),
            error: None,
        }
    }
}

/// Сколько каталогов помнить в истории вкладки.
pub const HISTORY_LIMIT: usize = 50;

// ----------------------------------------------------------------------
// Панель
// ----------------------------------------------------------------------

/// Файловая панель.
pub struct FilePanel {
    pub vs: ViewState,
    tabs: Vec<Tab>,
    active: usize,
    /// Порядок файлов (для всех вкладок).
    pub sort: SortMode,
    pub sort_reverse: bool,
    /// Каталоги вперемешку с файлами (иначе — сверху).
    pub sort_mix_dirs: bool,
    /// Различать регистр при сортировке по имени.
    pub sort_case_sensitive: bool,
    /// Показывать скрытые файлы.
    pub show_hidden: bool,
    /// Свои цвета (иначе — по палитре владельца).
    pub colors: Option<FilePanelColors>,
    /// Быстрый поиск: набранный текст.
    quick: Option<String>,
    /// Путь, о котором последний раз сообщали владельцу.
    last_announced: PathBuf,
}

impl FilePanel {
    /// Панель с каталогом `path` (если его нельзя прочитать — текущий каталог).
    pub fn new(bounds: Rect, path: &Path) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.options |= of::SELECTABLE | of::FIRST_CLICK;
        let mut p = FilePanel {
            vs,
            tabs: vec![Tab::new(PathBuf::new())],
            active: 0,
            sort: SortMode::Name,
            sort_reverse: false,
            sort_mix_dirs: false,
            sort_case_sensitive: false,
            show_hidden: true,
            colors: None,
            quick: None,
            last_announced: PathBuf::new(),
        };
        if p.set_path(path).is_err() {
            let cur = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
            let _ = p.set_path(&cur);
        }
        p.last_announced = p.path().to_path_buf();
        p
    }

    fn tab(&self) -> &Tab {
        &self.tabs[self.active]
    }

    fn tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active]
    }

    // --- каталог ---

    /// Текущий каталог вкладки (у виртуальной панели — её корень).
    pub fn path(&self) -> &Path {
        &self.tab().path
    }

    /// Перейти в каталог. Курсор — на первую запись.
    pub fn set_path(&mut self, path: &Path) -> Result<(), String> {
        self.change_dir(path, None)
    }

    /// Перейти в каталог и поставить курсор на файл `select` (если есть).
    pub fn change_dir(&mut self, path: &Path, select: Option<&str>) -> Result<(), String> {
        let abs = crate::widgets::filedlg::normalize(&crate::widgets::editor::absolute(path));
        let entries = read_entries(&abs).map_err(|e| format!("{}: {}", abs.display(), e))?;
        let t = self.tab_mut();
        if t.path != abs && !t.path.as_os_str().is_empty() && t.listing.is_none() {
            let old = t.path.clone();
            t.history.retain(|p| p != &old);
            t.history.push(old);
            if t.history.len() > HISTORY_LIMIT {
                t.history.remove(0);
            }
        }
        t.path = abs;
        t.entries = entries;
        t.listing = None;
        t.marked.clear();
        t.dir_sizes.clear();
        t.error = None;
        t.cursor = 0;
        t.top = 0;
        self.rebuild_view();
        if let Some(name) = select {
            self.select_name(name);
        }
        Ok(())
    }

    /// Перечитать каталог, сохранив курсор и выделение.
    pub fn refresh(&mut self) {
        let cur = self.current().map(|e| e.path.clone());
        let t = self.tab_mut();
        let path = t.path.clone();
        if let Some(l) = t.listing.clone() {
            t.entries = listing_entries(&l);
        } else {
            // Каталог могли удалить — поднимаемся до существующего.
            let mut p = path.clone();
            while !p.is_dir() {
                match p.parent() {
                    Some(q) => p = q.to_path_buf(),
                    None => break,
                }
            }
            match read_entries(&p) {
                Ok(e) => {
                    t.entries = e;
                    t.path = p;
                    t.error = None;
                }
                Err(e) => t.error = Some(e.to_string()),
            }
        }
        let names: BTreeSet<PathBuf> = t.entries.iter().map(|e| e.path.clone()).collect();
        t.marked.retain(|m| names.contains(m));
        self.rebuild_view();
        if let Some(c) = cur {
            if !self.select_path(&c) {
                self.clamp_cursor();
            }
        }
    }

    /// Наверх: из виртуальной панели — в её корень, иначе — в родительский
    /// каталог (курсор — на каталог, из которого вышли).
    pub fn go_up(&mut self) -> bool {
        if self.tab().listing.is_some() {
            let base = self.path().to_path_buf();
            return self.change_dir(&base, None).is_ok();
        }
        let path = self.path().to_path_buf();
        match path.parent() {
            Some(parent) => {
                let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
                self.change_dir(parent, name.as_deref()).is_ok()
            }
            None => false,
        }
    }

    /// Войти в каталог под курсором. `false` — под курсором не каталог.
    pub fn enter(&mut self) -> bool {
        let e = match self.current() {
            Some(e) => e.clone(),
            None => return false,
        };
        if e.is_parent() {
            return self.go_up();
        }
        if !e.is_dir() {
            return false;
        }
        if let Err(msg) = self.change_dir(&e.path, None) {
            self.tab_mut().error = Some(msg);
        }
        true
    }

    /// Ошибка последнего чтения каталога (показывается в строке сведений).
    pub fn error(&self) -> Option<&str> {
        self.tab().error.as_deref()
    }

    /// Показать сообщение в строке сведений (до следующего перехода).
    pub fn set_message(&mut self, msg: &str) {
        self.tab_mut().error = Some(msg.to_string());
    }

    /// История каталогов вкладки (старые — первыми).
    pub fn history(&self) -> &[PathBuf] {
        &self.tab().history
    }

    // --- записи и курсор ---

    /// Видимые записи в порядке показа.
    pub fn entries(&self) -> Vec<&PanelEntry> {
        let t = self.tab();
        t.view.iter().map(|&i| &t.entries[i]).collect()
    }

    /// Запись под курсором.
    pub fn current(&self) -> Option<&PanelEntry> {
        let t = self.tab();
        t.view.get(t.cursor).map(|&i| &t.entries[i])
    }

    /// Номер курсора среди видимых записей.
    pub fn cursor(&self) -> usize {
        self.tab().cursor
    }

    pub fn set_cursor(&mut self, i: usize) {
        self.tab_mut().cursor = i;
        self.clamp_cursor();
    }

    /// Поставить курсор на запись с таким показываемым именем.
    pub fn select_name(&mut self, name: &str) -> bool {
        let t = self.tab();
        match t.view.iter().position(|&i| t.entries[i].name == name) {
            Some(p) => {
                self.set_cursor(p);
                true
            }
            None => false,
        }
    }

    /// Поставить курсор на запись с таким путём.
    pub fn select_path(&mut self, path: &Path) -> bool {
        let t = self.tab();
        match t.view.iter().position(|&i| t.entries[i].path == path) {
            Some(p) => {
                self.set_cursor(p);
                true
            }
            None => false,
        }
    }

    // --- выделение ---

    /// Выделенные записи (в порядке показа).
    pub fn marked(&self) -> Vec<&PanelEntry> {
        let t = self.tab();
        t.view.iter().map(|&i| &t.entries[i]).filter(|e| t.marked.contains(&e.path)).collect()
    }

    /// Пути для операции: выделенные, а если выделения нет — файл под
    /// курсором (кроме `..`).
    pub fn targets(&self) -> Vec<PathBuf> {
        let m = self.marked();
        if !m.is_empty() {
            return m.iter().map(|e| e.path.clone()).collect();
        }
        match self.current() {
            Some(e) if !e.is_parent() => vec![e.path.clone()],
            _ => Vec::new(),
        }
    }

    /// Переключить выделение файла под курсором.
    pub fn toggle_mark(&mut self) {
        if let Some(e) = self.current().cloned() {
            if e.is_parent() {
                return;
            }
            let t = self.tab_mut();
            if !t.marked.remove(&e.path) {
                t.marked.insert(e.path);
            }
        }
    }

    /// Выделить (`on`) или снять выделение с файлов по шаблону. Каталоги —
    /// только если шаблон явно на них подходит. Возвращает число изменённых.
    pub fn mark_glob(&mut self, pattern: &str, on: bool) -> usize {
        let cs = !cfg!(windows);
        let t = self.tab_mut();
        let mut n = 0;
        for &i in &t.view {
            let e = &t.entries[i];
            if e.is_parent() || !pattern.split(',').map(|p| p.trim()).any(|p| !p.is_empty() && glob_match(p, &e.name, cs)) {
                continue;
            }
            let changed = if on { t.marked.insert(e.path.clone()) } else { t.marked.remove(&e.path) };
            if changed {
                n += 1;
            }
        }
        n
    }

    /// Инвертировать выделение файлов (каталоги не трогаются).
    pub fn invert_marks(&mut self) {
        let t = self.tab_mut();
        for &i in &t.view {
            let e = &t.entries[i];
            if e.is_dir() {
                continue;
            }
            if !t.marked.remove(&e.path) {
                t.marked.insert(e.path.clone());
            }
        }
    }

    pub fn clear_marks(&mut self) {
        self.tab_mut().marked.clear();
    }

    // --- фильтр, скрытые, сортировка ---

    /// Фильтр вкладки (строка).
    pub fn filter(&self) -> &str {
        self.tab().filter.text()
    }

    /// Задать фильтр вкладки (`""` — без фильтра).
    pub fn set_filter(&mut self, text: &str) {
        let cur = self.current().map(|e| e.path.clone());
        self.tab_mut().filter = FileFilter::parse(text);
        self.rebuild_view();
        if let Some(c) = cur {
            self.select_path(&c);
        }
    }

    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.resort();
    }

    /// Задать порядок и перестроить все вкладки.
    pub fn set_sort(&mut self, mode: SortMode, reverse: bool) {
        self.sort = mode;
        self.sort_reverse = reverse;
        self.resort();
    }

    /// Перестроить список после изменения настроек сортировки/показа.
    pub fn resort(&mut self) {
        let keep = self.active;
        for i in 0..self.tabs.len() {
            self.active = i;
            let cur = self.current().map(|e| e.path.clone());
            self.rebuild_view();
            if let Some(c) = cur {
                self.select_path(&c);
            }
        }
        self.active = keep;
    }

    // --- вкладки ---

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn active_tab(&self) -> usize {
        self.active
    }

    /// Новая вкладка с тем же каталогом (становится активной).
    pub fn new_tab(&mut self) {
        let mut t = self.tab().clone();
        t.marked.clear();
        t.history.clear();
        self.tabs.insert(self.active + 1, t);
        self.active += 1;
    }

    /// Закрыть вкладку (последнюю закрыть нельзя).
    pub fn close_tab(&mut self) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }
        self.tabs.remove(self.active);
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        }
        true
    }

    pub fn select_tab(&mut self, i: usize) {
        if i < self.tabs.len() {
            self.active = i;
            self.quick = None;
        }
    }

    pub fn next_tab(&mut self) {
        self.select_tab((self.active + 1) % self.tabs.len());
    }

    pub fn prev_tab(&mut self) {
        self.select_tab((self.active + self.tabs.len() - 1) % self.tabs.len());
    }

    /// Каталоги всех вкладок.
    pub fn tab_paths(&self) -> Vec<PathBuf> {
        self.tabs.iter().map(|t| t.path.clone()).collect()
    }

    // --- виртуальная панель, размеры ---

    /// Показать список путей как «виртуальную панель» (panelize в lfm):
    /// имена — пути относительно `base`, `..` возвращает в `base`.
    pub fn set_listing(&mut self, title: &str, base: &Path, items: Vec<PathBuf>) {
        let l = Listing { title: title.to_string(), base: base.to_path_buf(), items };
        let t = self.tab_mut();
        t.entries = listing_entries(&l);
        t.path = l.base.clone();
        t.listing = Some(l);
        t.marked.clear();
        t.dir_sizes.clear();
        t.error = None;
        t.cursor = 0;
        t.top = 0;
        self.rebuild_view();
    }

    /// Виртуальная панель ли сейчас (и её заголовок).
    pub fn listing_title(&self) -> Option<&str> {
        self.tab().listing.as_ref().map(|l| l.title.as_str())
    }

    /// Посчитать размеры каталогов: выделенных, а если выделения нет —
    /// всех. Размеры показываются в столбце «Размер». Возвращает сумму.
    pub fn calc_dir_sizes(&mut self) -> u64 {
        let marked: Vec<PathBuf> = self.marked().iter().filter(|e| e.is_dir()).map(|e| e.path.clone()).collect();
        let dirs: Vec<PathBuf> =
            if marked.is_empty() { self.entries().iter().filter(|e| e.is_dir() && !e.is_parent()).map(|e| e.path.clone()).collect() } else { marked };
        let mut total = 0;
        for d in dirs {
            let (b, _) = crate::fileops::dir_size(&d);
            total += b;
            self.tab_mut().dir_sizes.insert(d, b);
        }
        total
    }

    /// Посчитанный размер каталога (если считали).
    pub fn dir_size(&self, path: &Path) -> Option<u64> {
        self.tab().dir_sizes.get(path).copied()
    }

    // --- внутреннее ---

    fn rebuild_view(&mut self) {
        let (sort, rev, mix, cs, hidden) = (self.sort, self.sort_reverse, self.sort_mix_dirs, self.sort_case_sensitive, self.show_hidden);
        let t = &mut self.tabs[self.active];
        let mut view: Vec<usize> = (0..t.entries.len())
            .filter(|&i| {
                let e = &t.entries[i];
                e.is_parent() || ((hidden || !e.hidden) && (e.is_dir() || !t.filter.hides(&e.name)))
            })
            .collect();
        let entries = &t.entries;
        let dir_rank = |e: &PanelEntry| {
            if e.is_parent() {
                0
            } else if !mix && e.is_dir() {
                1
            } else {
                2
            }
        };
        let name_key = |e: &PanelEntry| {
            if cs {
                e.name.clone()
            } else {
                e.name.to_lowercase()
            }
        };
        view.sort_by(|&a, &b| {
            let (x, y) = (&entries[a], &entries[b]);
            let r = dir_rank(x).cmp(&dir_rank(y));
            if r != std::cmp::Ordering::Equal || x.is_parent() {
                return r;
            }
            let o = match sort {
                SortMode::Name => name_key(x).cmp(&name_key(y)),
                SortMode::Ext => x.extension().cmp(&y.extension()).then_with(|| name_key(x).cmp(&name_key(y))),
                SortMode::Size => x.size.cmp(&y.size).then_with(|| name_key(x).cmp(&name_key(y))),
                SortMode::Time => x.modified.cmp(&y.modified).then_with(|| name_key(x).cmp(&name_key(y))),
                SortMode::Unsorted => a.cmp(&b),
            };
            if rev {
                o.reverse()
            } else {
                o
            }
        });
        t.view = view;
        self.clamp_cursor();
    }

    fn clamp_cursor(&mut self) {
        let rows = self.rows().max(1);
        let t = self.tab_mut();
        let n = t.view.len();
        if n == 0 {
            t.cursor = 0;
            t.top = 0;
            return;
        }
        t.cursor = t.cursor.min(n - 1);
        if t.cursor < t.top {
            t.top = t.cursor;
        } else if t.cursor >= t.top + rows {
            t.top = t.cursor + 1 - rows;
        }
        t.top = t.top.min(n.saturating_sub(rows));
    }

    /// Строк под список файлов: рамка, заголовки, разделитель, сведения.
    fn rows(&self) -> usize {
        (self.vs.size.y - 5).max(1) as usize
    }

    fn move_cursor(&mut self, delta: i64) {
        let n = self.tab().view.len() as i64;
        if n == 0 {
            return;
        }
        let c = (self.tab().cursor as i64 + delta).clamp(0, n - 1);
        self.set_cursor(c as usize);
    }

    /// Быстрый поиск: к первой записи, начинающейся с текста (без учёта
    /// регистра), иначе — содержащей его. `false` — не нашлось.
    pub fn quick_find(&mut self, text: &str) -> bool {
        let q = text.to_lowercase();
        let t = self.tab();
        let names: Vec<String> = t.view.iter().map(|&i| t.entries[i].name.to_lowercase()).collect();
        let pos = names.iter().position(|n| n.starts_with(&q)).or_else(|| names.iter().position(|n| n.contains(&q)));
        match pos {
            Some(p) => {
                self.set_cursor(p);
                true
            }
            None => false,
        }
    }

    /// Ширины столбцов: (имя, размер, дата) — 0, если столбца нет.
    fn columns(&self) -> (usize, usize, usize) {
        let w = (self.vs.size.x - 2).max(1) as usize;
        if w >= 46 {
            (w - 1 - 8 - 1 - 14, 8, 14)
        } else if w >= 22 {
            (w - 1 - 8, 8, 0)
        } else {
            (w, 0, 0)
        }
    }

    /// Участки вкладок в верхней рамке: (x начала, x конца, текст).
    fn tab_spans(&self) -> Vec<(i32, i32, String)> {
        let w = self.vs.size.x;
        let mut out = Vec::new();
        if self.tabs.len() == 1 {
            return out;
        }
        let mut x = 1;
        let each = ((w - 2 - 8) / self.tabs.len() as i32).max(5) as usize;
        for (i, t) in self.tabs.iter().enumerate() {
            let name = match &t.listing {
                Some(l) => l.title.clone(),
                None => t.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| t.path.display().to_string()),
            };
            let label = format!(" {}:{} ", i + 1, name);
            let label = if str_width(&label) > each { format!("{} ", fit(&label, each - 1, false).trim_end()) } else { label };
            let lw = str_width(&label) as i32;
            if x + lw > w - 1 {
                break;
            }
            out.push((x, x + lw, label));
            x += lw;
        }
        out
    }

    fn colors(&self, dc: &DrawCtx) -> FilePanelColors {
        if let Some(c) = self.colors {
            return c;
        }
        let (fp, fa, n, c) = match dc.owner_palette_len() {
            8 => (1, 2, 6, 7),
            32 => (1, 2, 26, 27),
            l if l >= 100 => (8, 9, 13, 14),
            _ => return FilePanelColors::derive(0x17, 0x1F, 0x1B, 0x30),
        };
        FilePanelColors::derive(dc.color(fp), dc.color(fa), dc.color(n), dc.color(c))
    }

    fn flags(&self) -> String {
        let mut s = String::from("[");
        let l = self.sort.letter();
        s.push(if self.sort_reverse { l.to_ascii_uppercase() } else { l });
        if self.show_hidden {
            s.push('.');
        }
        if !self.tab().filter.is_empty() {
            s.push('f');
        }
        if self.tab().listing.is_some() {
            s.push('p');
        }
        s.push(']');
        s
    }

    fn post_dir_changed(&mut self, ctx: &mut Ctx) {
        if self.last_announced != self.path() {
            self.last_announced = self.path().to_path_buf();
            ctx.message(cm::FILE_PANEL_DIR_CHANGED, Info::Id(self.vs.id));
        }
    }

    /// Enter: в каталог или команда «выполнить файл».
    fn activate(&mut self, ctx: &mut Ctx) {
        if !self.enter() && self.current().is_some() {
            ctx.put_event(Event::command_event(cm::FILE_PANEL_EXECUTE));
        }
    }
}

/// Прочитать каталог: `..` (если есть родитель) и записи.
fn read_entries(path: &Path) -> std::io::Result<Vec<PanelEntry>> {
    let rd = fs::read_dir(path)?;
    let mut v = Vec::new();
    if path.parent().is_some() {
        v.push(PanelEntry::parent(path.parent().unwrap()));
    }
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        v.push(PanelEntry::from_path(&e.path(), &name));
    }
    Ok(v)
}

fn listing_entries(l: &Listing) -> Vec<PanelEntry> {
    let mut v = vec![PanelEntry::parent(&l.base)];
    for p in &l.items {
        let name = p.strip_prefix(&l.base).unwrap_or(p).to_string_lossy().into_owned();
        v.push(PanelEntry::from_path(p, &name));
    }
    v
}

impl View for FilePanel {
    crate::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let c = self.colors(dc);
        let (w, h) = (self.vs.size.x.max(2) as usize, self.vs.size.y.max(1));
        let focused = self.vs.is_focused() || (self.vs.state & sf::SELECTED != 0 && self.vs.state & sf::ACTIVE != 0);
        let fattr = if focused { c.frame_active } else { c.frame };
        let (hl, vl, tl, tr_, bl, br, ml, mr) =
            if focused { ('═', '║', '╔', '╗', '╚', '╝', '╟', '╢') } else { ('─', '│', '┌', '┐', '└', '┘', '├', '┤') };
        let (nw, sw, dw) = self.columns();
        let seps: Vec<usize> = {
            let mut v = Vec::new();
            if sw > 0 {
                v.push(1 + nw);
            }
            if dw > 0 {
                v.push(1 + nw + 1 + sw);
            }
            v
        };
        let t = self.tab();

        // Верхняя рамка: вкладки или путь, флаги справа.
        let mut b = DrawBuffer::new(w);
        b.move_char(0, hl, fattr, w);
        b.move_char(0, tl, fattr, 1);
        b.move_char(w - 1, tr_, fattr, 1);
        let flags = self.flags();
        let fl = str_width(&flags);
        if w > fl + 4 {
            b.move_str(w - 1 - fl - 1, &flags, fattr);
        }
        let spans = self.tab_spans();
        if spans.is_empty() {
            let title = match &t.listing {
                Some(l) => format!("{} [{}]", t.path.display(), l.title),
                None => t.path.display().to_string(),
            };
            let room = w.saturating_sub(fl + 6);
            let title = format!(" {} ", fit_path_left(&title, room));
            b.move_str(2, &title, if focused { c.title_active } else { c.title });
        } else {
            for (i, (x0, _, label)) in spans.iter().enumerate() {
                let a = if i == self.active {
                    if focused {
                        c.title_active
                    } else {
                        c.header
                    }
                } else {
                    c.title
                };
                b.move_str(*x0 as usize, label, a);
            }
        }
        dc.write_line(0, 0, w as i32, 1, &b);

        // Заголовки столбцов.
        if h > 1 {
            let mut b = DrawBuffer::new(w);
            b.move_char(0, ' ', c.info, w);
            b.move_char(0, vl, fattr, 1);
            b.move_char(w - 1, vl, fattr, 1);
            b.move_str(1, &center(tr("Name"), nw), c.header);
            if sw > 0 {
                b.move_str(2 + nw, &center(tr("Size"), sw), c.header);
            }
            if dw > 0 {
                b.move_str(3 + nw + sw, &center(tr("Modified"), dw), c.header);
            }
            for &x in &seps {
                b.move_char(x, '│', fattr, 1);
            }
            dc.write_line(0, 1, w as i32, 1, &b);
        }

        // Файлы.
        let rows = self.rows();
        let list_end = (2 + rows as i32).min(h - 3).max(2);
        for (row, y) in (2..list_end).enumerate() {
            let mut b = DrawBuffer::new(w);
            b.move_char(0, ' ', c.info, w);
            let idx = t.top + row;
            if let Some(&ei) = t.view.get(idx) {
                let e = &t.entries[ei];
                let marked = t.marked.contains(&e.path);
                let is_cur = idx == t.cursor && focused;
                let attr = match (is_cur, marked) {
                    (true, true) => c.marked_cursor,
                    (true, false) => c.cursor,
                    (false, true) => c.marked,
                    (false, false) => c.classes[e.class() as usize],
                };
                let mark = match e.kind {
                    FileKind::Parent => ' ',
                    FileKind::Dir => '/',
                    FileKind::Link { .. } => '@',
                    FileKind::Other => '|',
                    FileKind::File if e.executable => '*',
                    FileKind::File => ' ',
                };
                let mark = if marked { '√' } else { mark };
                let name = format!("{}{}", mark, e.name);
                b.move_str(1, &fit(&name, nw, false), attr);
                if sw > 0 {
                    let size = match e.kind {
                        FileKind::Parent => tr("<UP>").to_string(),
                        _ if e.is_dir() => match t.dir_sizes.get(&e.path) {
                            Some(&s) => format_size_short(s),
                            None if matches!(e.kind, FileKind::Link { .. }) => tr("<LINK>").to_string(),
                            None => tr("<DIR>").to_string(),
                        },
                        _ => format_size_short(e.size),
                    };
                    b.move_char(1 + nw, ' ', attr, 1);
                    b.move_str(2 + nw, &fit(&size, sw, true), attr);
                }
                if dw > 0 {
                    b.move_char(2 + nw + sw, ' ', attr, 1);
                    b.move_str(3 + nw + sw, &fit(&format_time_short(e.modified), dw, true), attr);
                }
                if is_cur {
                    // Курсор — во всю ширину, разделители его цветом.
                    for &x in &seps {
                        b.move_char(x, '│', attr, 1);
                    }
                }
            }
            b.move_char(0, vl, fattr, 1);
            b.move_char(w - 1, vl, fattr, 1);
            let cur_row = t.view.get(t.top + row).is_some() && t.top + row == t.cursor && focused;
            if !cur_row {
                for &x in &seps {
                    b.move_char(x, '│', fattr, 1);
                }
            }
            // Ползунок прокрутки на правой рамке.
            if t.view.len() > rows && rows > 1 {
                let thumb = t.cursor * (rows - 1) / (t.view.len() - 1).max(1);
                if row == thumb {
                    b.move_char(w - 1, '■', fattr, 1);
                }
            }
            dc.write_line(0, y, w as i32, 1, &b);
        }

        // Разделитель и строка сведений.
        if h >= 4 {
            let mut b = DrawBuffer::new(w);
            b.move_char(0, '─', fattr, w);
            b.move_char(0, ml, fattr, 1);
            b.move_char(w - 1, mr, fattr, 1);
            for &x in &seps {
                b.move_char(x, '┴', fattr, 1);
            }
            dc.write_line(0, h - 3, w as i32, 1, &b);
            let mut b = DrawBuffer::new(w);
            b.move_char(0, ' ', c.info, w);
            b.move_char(0, vl, fattr, 1);
            b.move_char(w - 1, vl, fattr, 1);
            b.move_str(1, &fit(&self.info_text(), w - 2, false), c.info);
            if let Some(q) = &self.quick {
                let s = format!("{} {}", tr("Search:"), q);
                b.move_str(1, &fit(&s, w - 2, false), c.marked);
            }
            dc.write_line(0, h - 2, w as i32, 1, &b);
        }

        // Нижняя рамка: итог по каталогу.
        if h >= 2 {
            let mut b = DrawBuffer::new(w);
            b.move_char(0, hl, fattr, w);
            b.move_char(0, bl, fattr, 1);
            b.move_char(w - 1, br, fattr, 1);
            let files: Vec<&PanelEntry> = t.view.iter().map(|&i| &t.entries[i]).filter(|e| !e.is_dir()).collect();
            let bytes: u64 = files.iter().map(|e| e.size).sum();
            let s = format!(" {} {}, {} ", tr("Files:"), files.len(), format_size_full(bytes));
            if str_width(&s) + 4 <= w {
                b.move_str((w - str_width(&s)) / 2, &s, fattr);
            }
            dc.write_line(0, h - 1, w as i32, 1, &b);
        }
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        default_handle_event(&self.vs, ev, ctx);
        let rows = self.rows() as i64;
        match ev {
            Event::KeyDown(k) if self.vs.is_focused() => {
                let k = *k;
                if self.quick.is_some() && self.quick_key(k) {
                    ev.clear();
                    self.post_dir_changed(ctx);
                    return;
                }
                let handled = match (k.key, k.mods.ctrl, k.mods.alt, k.mods.shift) {
                    (Key::Up, false, false, _) => {
                        self.move_cursor(-1);
                        true
                    }
                    (Key::Down, false, false, _) => {
                        self.move_cursor(1);
                        true
                    }
                    (Key::Up, true, false, _) => {
                        self.move_cursor(-10);
                        true
                    }
                    (Key::Down, true, false, _) => {
                        self.move_cursor(10);
                        true
                    }
                    (Key::PgUp, false, false, _) => {
                        self.move_cursor(-(rows - 1).max(1));
                        true
                    }
                    (Key::PgDn, false, false, _) => {
                        self.move_cursor((rows - 1).max(1));
                        true
                    }
                    (Key::Home, _, false, _) => {
                        self.set_cursor(0);
                        true
                    }
                    (Key::End, _, false, _) => {
                        self.set_cursor(usize::MAX);
                        true
                    }
                    (Key::Left, false, false, _) | (Key::Backspace, false, false, _) | (Key::PgUp, true, false, _) => {
                        self.go_up();
                        true
                    }
                    (Key::Right, false, false, _) | (Key::PgDn, true, false, _) => {
                        self.enter();
                        true
                    }
                    (Key::Enter, false, false, false) => {
                        self.activate(ctx);
                        true
                    }
                    (Key::Ins, false, false, false) | (Key::Char(' '), false, false, _) => {
                        self.toggle_mark();
                        self.move_cursor(1);
                        true
                    }
                    (Key::Char(c), true, false, _) if c.eq_ignore_ascii_case(&'r') => {
                        self.refresh();
                        true
                    }
                    (Key::Char(c), true, false, _) if c.eq_ignore_ascii_case(&'s') => {
                        self.quick = Some(String::new());
                        true
                    }
                    (Key::Char(c), false, true, _) if c.is_alphanumeric() => {
                        let s: String = c.to_lowercase().collect();
                        let t = self.tab();
                        let start = t.cursor + 1;
                        let n = t.view.len();
                        let found = (0..n).map(|i| (start + i) % n.max(1)).find(|&i| t.entries[t.view[i]].name.to_lowercase().starts_with(&s));
                        match found {
                            Some(i) => {
                                self.set_cursor(i);
                                true
                            }
                            None => false,
                        }
                    }
                    _ => false,
                };
                if handled {
                    ev.clear();
                }
            }
            Event::MouseDown(m) => {
                self.quick = None;
                let p = self.vs.make_local(m.pos);
                if p.y == 0 {
                    if let Some(i) = self.tab_spans().iter().position(|(a, b, _)| p.x >= *a && p.x < *b) {
                        self.select_tab(i);
                    }
                } else if p.y >= 2 && p.y < 2 + self.rows() as i32 && p.x > 0 && p.x < self.vs.size.x - 1 {
                    let idx = self.tab().top + (p.y - 2) as usize;
                    if idx < self.tab().view.len() {
                        self.set_cursor(idx);
                        if m.buttons & crate::event::MB_RIGHT != 0 {
                            self.toggle_mark();
                        } else if m.double {
                            self.activate(ctx);
                        }
                    }
                }
                ev.clear();
            }
            Event::MouseWheel(m) => {
                self.move_cursor(m.wheel as i64 * 3);
                ev.clear();
            }
            Event::Broadcast(msg) if msg.command == cm::RELEASED_FOCUS => {
                if msg.info == Info::Id(self.vs.id) {
                    self.quick = None;
                }
            }
            _ => {}
        }
        self.post_dir_changed(ctx);
    }

    fn change_bounds(&mut self, bounds: Rect) {
        self.vs.set_bounds(bounds);
        let keep = self.active;
        for i in 0..self.tabs.len() {
            self.active = i;
            self.clamp_cursor();
        }
        self.active = keep;
    }
}

impl FilePanel {
    /// Клавиши в режиме быстрого поиска. `true` — клавиша поглощена.
    fn quick_key(&mut self, k: KeyEvent) -> bool {
        let q = self.quick.clone().unwrap_or_default();
        match k.key {
            Key::Esc => {
                self.quick = None;
                true
            }
            Key::Backspace => {
                let mut q = q;
                q.pop();
                if !q.is_empty() {
                    self.quick_find(&q);
                }
                self.quick = Some(q);
                true
            }
            Key::Char(c) if !k.mods.ctrl && !k.mods.alt => {
                let s = format!("{}{}", q, c);
                if self.quick_find(&s) {
                    self.quick = Some(s);
                }
                true
            }
            Key::Char(c) if k.mods.ctrl && c.eq_ignore_ascii_case(&'s') => {
                // Повторный Ctrl+S — следующее совпадение.
                if !q.is_empty() {
                    let ql = q.to_lowercase();
                    let t = self.tab();
                    let n = t.view.len();
                    let start = t.cursor + 1;
                    if let Some(i) = (0..n).map(|i| (start + i) % n.max(1)).find(|&i| t.entries[t.view[i]].name.to_lowercase().contains(&ql)) {
                        self.set_cursor(i);
                    }
                }
                true
            }
            _ => {
                // Любая другая клавиша заканчивает поиск и работает как обычно.
                self.quick = None;
                false
            }
        }
    }

    /// Текст строки сведений.
    fn info_text(&self) -> String {
        let t = self.tab();
        if let Some(e) = &t.error {
            return e.clone();
        }
        let marked = self.marked();
        if !marked.is_empty() {
            let bytes: u64 = marked.iter().map(|e| self.dir_size(&e.path).unwrap_or(e.size)).sum();
            return format!("{} {}, {}", tr("Selected:"), marked.len(), format_size_full(bytes));
        }
        match self.current() {
            Some(e) if e.is_parent() => tr("Up").to_string(),
            Some(e) => {
                let size = if e.is_dir() { self.dir_size(&e.path).map(format_size_full).unwrap_or_default() } else { format_size_full(e.size) };
                format!("{}  {}  {}  {}", e.name, size, e.perms_string(), format_time_full(e.modified))
            }
            None => String::new(),
        }
    }
}

fn center(s: &str, w: usize) -> String {
    let sw = str_width(s);
    if sw >= w {
        return fit(s, w, false);
    }
    let l = (w - sw) / 2;
    format!("{}{}{}", " ".repeat(l), s, " ".repeat(w - sw - l))
}

// ----------------------------------------------------------------------
// Две панели
// ----------------------------------------------------------------------

/// Две файловые панели рядом (как в lfm, MC и DN): Tab переключает
/// панели, режим одной панели, обмен местами, «тот же каталог».
///
/// Панели — дети группы; к ним можно обращаться через [`DualPanel::active`]
/// и [`DualPanel::other`] (или `app.find_mut::<FilePanel>(id)`).
pub struct DualPanel {
    pub group: Group,
    left: ViewId,
    right: ViewId,
    /// Показана только активная панель.
    pub single: bool,
}

impl DualPanel {
    pub fn new(bounds: Rect, left_path: &Path, right_path: &Path) -> Self {
        let mut group = Group::new(bounds);
        group.vs.options |= of::SELECTABLE;
        group.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
        let ext = group.vs.extent();
        let left = group.add(FilePanel::new(ext, left_path));
        let right = group.add(FilePanel::new(ext, right_path));
        let mut d = DualPanel { group, left, right, single: false };
        d.layout();
        d
    }

    pub fn left_id(&self) -> ViewId {
        self.left
    }

    pub fn right_id(&self) -> ViewId {
        self.right
    }

    /// Id панели в фокусе.
    pub fn active_id(&self) -> ViewId {
        self.group.current_view().map(|v| v.vs().id).unwrap_or(self.left)
    }

    /// Id другой панели.
    pub fn other_id(&self) -> ViewId {
        if self.active_id() == self.left {
            self.right
        } else {
            self.left
        }
    }

    pub fn active(&self) -> &FilePanel {
        self.group.find::<FilePanel>(self.active_id()).expect("панель")
    }

    pub fn active_mut(&mut self) -> &mut FilePanel {
        let id = self.active_id();
        self.group.find_mut::<FilePanel>(id).expect("панель")
    }

    pub fn other(&self) -> &FilePanel {
        self.group.find::<FilePanel>(self.other_id()).expect("панель")
    }

    pub fn other_mut(&mut self) -> &mut FilePanel {
        let id = self.other_id();
        self.group.find_mut::<FilePanel>(id).expect("панель")
    }

    /// Обе панели: (левая, правая).
    pub fn panel_mut(&mut self, id: ViewId) -> Option<&mut FilePanel> {
        self.group.find_mut::<FilePanel>(id)
    }

    /// Одна или две панели.
    pub fn toggle_single(&mut self) {
        self.single = !self.single;
        self.layout();
    }

    /// Поменять панели местами (каталоги остаются за панелями).
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.left, &mut self.right);
        self.layout();
    }

    /// Открыть в другой панели каталог активной (`=` в lfm).
    pub fn sync_other(&mut self) {
        let path = self.active().path().to_path_buf();
        let _ = self.other_mut().set_path(&path);
    }

    /// Разместить панели по ширине группы.
    pub fn layout(&mut self) {
        let ext = self.group.vs.extent();
        let half = ext.b.x / 2;
        let active = self.active_id();
        let (lr, rr) = (Rect::new(0, 0, half, ext.b.y), Rect::new(half, 0, ext.b.x, ext.b.y));
        for (id, r) in [(self.left, lr), (self.right, rr)] {
            let single = self.single;
            if let Some(p) = self.group.find_mut::<FilePanel>(id) {
                p.change_bounds(if single { ext } else { r });
                p.set_state(sf::VISIBLE, !single || id == active);
            }
        }
    }

    /// Переключить фокус на другую панель (в режиме одной — показать её).
    pub fn switch(&mut self, ctx: &mut Ctx) {
        let other = self.other_id();
        self.group.select_id(other, ctx);
        if self.single {
            self.layout();
        }
    }
}

impl View for DualPanel {
    fn vs(&self) -> &ViewState {
        &self.group.vs
    }
    fn vs_mut(&mut self) -> &mut ViewState {
        &mut self.group.vs
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
    fn as_group(&self) -> Option<&Group> {
        Some(&self.group)
    }
    fn as_group_mut(&mut self) -> Option<&mut Group> {
        Some(&mut self.group)
    }
    fn help_ctx(&self) -> u16 {
        self.group.help_ctx()
    }

    fn draw(&self, dc: &mut DrawCtx) {
        self.group.draw_children(dc);
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        self.group.handle_group_event(ev, ctx);
        if let Event::KeyDown(k) = ev {
            if k.key == Key::Tab && k.mods == Mods::NONE && self.group.vs.is_focused() {
                self.switch(ctx);
                ev.clear();
            }
        }
    }

    fn set_state(&mut self, flag: u16, enable: bool) {
        self.group.set_group_state(flag, enable);
    }

    fn change_bounds(&mut self, bounds: Rect) {
        self.group.vs.set_bounds(bounds);
        self.layout();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn globs() {
        assert!(glob_match("*.rs", "main.rs", true));
        assert!(!glob_match("*.rs", "main.rsx", true));
        assert!(glob_match("ma?n.*", "main.c", true));
        assert!(glob_match("[a-c]*", "beta", true));
        assert!(!glob_match("[!a-c]*", "beta", true));
        assert!(glob_match("*shot*", "screenshot.png", true));
        assert!(glob_match("*.PNG", "a.png", false));
        assert!(!glob_match("*.PNG", "a.png", true));
        assert!(glob_match("*", "", true));
        assert!(glob_match("a*b*c", "aXXbYYc", true));
        assert!(!glob_match("a*b*c", "aXXbYY", true));
        assert!(glob_match("отчёт*", "отчёт-2026.txt", true));
    }

    #[test]
    fn filters() {
        let f = FileFilter::parse("*.jpg,!*shot*");
        assert!(f.hides("a.jpg"));
        assert!(!f.hides("screenshot.jpg"));
        assert!(!f.hides("a.png"));
        let f = FileFilter::parse("*,!*.rs");
        assert!(f.hides("a.txt") && !f.hides("main.rs"));
        assert!(FileFilter::parse(" ").is_empty());
    }

    #[test]
    fn sizes_and_classes() {
        assert_eq!(format_size_short(1234567), "1234567");
        assert_eq!(format_size_short(12 * 1024 * 1024), "12.0M");
        assert_eq!(format_size_short(3 * 1024 * 1024 * 1024), "3.00G");
        assert_eq!(format_size_full(1234567), "1 234 567");
        assert_eq!(format_size_full(12), "12");
        assert_eq!(class_by_name("x.tar.gz"), FileClass::Archive);
        assert_eq!(class_by_name("Photo.JPG"), FileClass::Image);
        assert_eq!(class_by_name("main.rs"), FileClass::Source);
        assert_eq!(class_by_name("notes~"), FileClass::Temp);
        assert_eq!(class_by_name(".bashrc"), FileClass::Normal);
        assert_eq!(fit_path_left("/very/long/path/name", 10), "…path/name");
    }
}
