# Справочник: файловые панели и операции

Для файловых менеджеров: панели в стиле lfm / Midnight Commander / DOS
Navigator и копирование в потоке. Модули `widgets::filepanel` и
`fileops`. Пример — `cargo run --example lfm`; подробно —
[глава 17](../17-file-manager.md).

* [`FilePanel`](#filepanel) · [`DualPanel`](#dualpanel)
* [`PanelEntry`](#panelentry) · [`FileKind`](#filekind) · [`FileClass`](#fileclass) · [`FileFilter`](#filefilter) · [`SortMode`](#sortmode) · [`FilePanelColors`](#filepanelcolors) · [функции](#функции-filepanel)
* [`FileJob`](#filejob) · [`JobKind`](#jobkind) · [`Conflict`](#conflict) · [`JobProgress`](#jobprogress) · [функции `fileops`](#функции-fileops)

---

## `FilePanel`

**Файловая панель**: вкладки в рамке, фильтры, сортировка, выделение,
быстрый поиск, история, «виртуальная панель» (результаты поиска), размеры
каталогов, цвета по типам файлов, мышь.

```rust
use std::path::Path;
use tvision::*;

fn panel_window(app: &mut Program, dir: &Path) -> ViewId {
    let mut w = Window::new(Rect::new(0, 0, 50, 20), "Файлы", 1);
    let mut p = FilePanel::new(w.interior(), dir);
    p.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
    p.set_sort(SortMode::Time, true); // новые сверху
    p.set_filter("*.o,*.tmp"); // что скрыть
    let id = w.add(p);
    app.insert_window(w);
    id
}

fn selected(app: &Program, id: ViewId) -> Vec<std::path::PathBuf> {
    app.find::<FilePanel>(id).map(|p| p.targets()).unwrap_or_default() // выделенные или файл под курсором
}
```

| Группа | Методы |
|--------|--------|
| каталог | `FilePanel::new(bounds, path)`, `path()`, `set_path(p)`, `change_dir(p, Some(имя))`, `go_up()`, `enter()`, `refresh()`, `history()`, `error()`, `set_message(s)` |
| записи | `entries()`, `current()`, `cursor()`, `set_cursor(i)`, `select_name(n)`, `select_path(p)`, `quick_find(текст)` |
| выделение | `toggle_mark()`, `mark_glob("*.rs", on)`, `invert_marks()`, `clear_marks()`, `marked()`, `targets()` |
| показ | `set_filter(s)`, `filter()`, `toggle_hidden()`, `set_sort(mode, reverse)`, `resort()` |
| вкладки | `new_tab()`, `close_tab()`, `next_tab()`, `prev_tab()`, `select_tab(i)`, `tab_count()`, `active_tab()`, `tab_paths()` |
| виртуальная панель | `set_listing(заголовок, корень, пути)`, `listing_title()` |
| размеры каталогов | `calc_dir_sizes()`, `dir_size(p)` |

Поля: `sort`, `sort_reverse`, `sort_mix_dirs`, `sort_case_sensitive`,
`show_hidden`, `colors`. Константа `filepanel::HISTORY_LIMIT` — сколько
каталогов помнит вкладка.

Клавиши: стрелки, PgUp/PgDn, Home/End; ← / Backspace — наверх, → / Enter —
внутрь; Ins или пробел — выделить; Ctrl+S и Alt+буква — быстрый поиск;
Ctrl+R — перечитать. Enter на **файле** ставит в очередь команду
`cm::FILE_PANEL_EXECUTE` — что делать с файлом, решает программа; смена
каталога рассылает соседям `cm::FILE_PANEL_DIR_CHANGED`.

## `DualPanel`

**Две панели рядом**: Tab переключает, можно оставить одну, поменять
местами, открыть в другой тот же каталог.

```rust
use std::path::Path;
use tvision::*;

struct App {
    panels: ViewId,
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(cm::FILE_PANEL_EXECUTE) {
            let file = app.find::<DualPanel>(self.panels).and_then(|d| d.active().current().map(|e| e.path.clone()));
            if let Some(f) = file {
                message_box(app, &format!("Открыть {}?", f.display()), mf::CONFIRMATION | mf::YES_NO_CANCEL);
            }
            ev.clear();
        }
    }
}

fn start(app: &mut Program, left: &Path, right: &Path) -> App {
    let r = app.desktop().group.vs.extent();
    App { panels: app.insert_window(DualPanel::new(r, left, right)) }
}
```

| Метод / поле | Что |
|--------------|-----|
| `DualPanel::new(bounds, left, right)` | |
| `active()`, `active_mut()`, `other()`, `other_mut()` | панель в фокусе и другая |
| `left_id()`, `right_id()`, `active_id()`, `other_id()`, `panel_mut(id)` | по id |
| `toggle_single()`, `single` | одна или две панели |
| `swap()`, `sync_other()` | поменять местами; другой — тот же каталог |
| `switch(ctx)`, `layout()` | переключить фокус; разложить по ширине |
| `group` | группа с двумя панелями |

## `PanelEntry`

**Запись панели**: `name` (показываемое имя), `path`, `kind: FileKind`,
`size`, `modified`, `mode` (права), `executable`, `hidden`. Методы
`is_dir()`, `is_parent()`, `extension()`, `class()`, `perms_string()`
(`rwxr-xr-x`); `PanelEntry::from_path(path, name)` — запись без панели.

## `FileKind`

`Parent` (`..`), `Dir`, `File`, `Link { to_dir }`, `Other` (устройство,
канал, сокет).

## `FileClass`

**Класс файла для цвета**: `Normal`, `Dir`, `Exec`, `Link`, `Archive`,
`Image`, `Audio`, `Video`, `Document`, `Source`, `Data`, `Temp`.
`class_by_name(имя)` определяет класс по расширению;
`filepanel::FILE_CLASSES` — их число.

## `FileFilter`

**Фильтр вкладки** — что *скрыть*; `!шаблон` — исключение.

```rust
use tvision::*;

fn filters() {
    let f = FileFilter::parse("*.o,*.tmp,!keep*");
    assert!(f.hides("main.o"));
    assert!(!f.hides("keep.tmp")); // исключение
    assert!(!f.hides("main.rs"));
    assert!(FileFilter::parse("").is_empty());
    assert!(glob_match("[a-c]*.rs", "build.rs", true));
    assert_eq!(class_by_name("photo.JPG"), FileClass::Image);
}
```

Методы `parse(s)`, `hides(name)`, `is_empty()`, `text()`. Шаблоны —
`*`, `?`, `[abc]`, `[a-z]`, `[!x]` (функция `glob_match(pattern, name,
case_sensitive)`).

## `SortMode`

`Name` (по умолчанию), `Ext`, `Size`, `Time`, `Unsorted`; `letter()` —
буква в рамке (`n`, `e`, `s`, `t`, `u`), `SortMode::ALL` — все.

## `FilePanelColors`

**Цвета панели**: `frame`, `frame_active`, `title`, `title_active`,
`header`, `cursor`, `marked`, `marked_cursor`, `info`, `classes`
(по `FileClass`). `FilePanelColors::derive(рамка, рамка_активная, текст,
курсор)` строит набор из четырёх цветов. Без настройки цвета берутся у
владельца; типы файлов в gruvbox раскрашиваются цветами gruvbox.

## Функции `filepanel`

| Функция | Результат |
|---------|-----------|
| `format_size_short(n)` | 7 символов: `1234567`, `12.3M`, `1.20G` |
| `format_size_full(n)` | `1 234 567` |
| `format_time_short(t)`, `format_time_full(t)` | `дд.мм.гг чч:мм`, `дд.мм.гггг чч:мм:сс` (местное время) |
| `available_roots()` | диски на Windows; `/`, домашний, `/mnt/*`, `/media/*` на Unix |
| `class_by_name(name)`, `glob_match(p, name, cs)` | см. выше |

---

## `FileJob`

**Операция над файлами в отдельном потоке**: копирование, перенос,
удаление с прогрессом и остановкой. Работает и на Windows (в lfm был
`fork`).

```rust
use std::path::PathBuf;
use tvision::fileops::{self, Conflict, FileJob, JobKind};
use tvision::*;

/// Скопировать с окном прогресса и кнопкой «Стоп».
fn copy(app: &mut Program, sources: Vec<PathBuf>, dest: PathBuf) -> fileops::JobProgress {
    let conflict = if fileops::conflicts(&sources, &dest).is_empty() { Conflict::Overwrite } else { Conflict::Skip };
    let mut job = FileJob::start(JobKind::Copy, sources, Some(dest), conflict);
    let mut d = Dialog::centered(50, 8, "Копирование");
    let bar = d.add(ProgressBar::new(Rect::new(2, 2, 48, 3), 100));
    d.add(Button::new(Rect::new(19, 4, 31, 6), "~С~топ", cm::CANCEL, bf::DEFAULT));
    let (cmd, _) = app.exec_dialog_idle(d, |app, _| {
        let p = job.progress();
        app.find_mut::<ProgressBar>(bar)?.set_value(p.percent() as i64);
        p.finished.then_some(cm::OK)
    });
    if cmd != cm::OK {
        job.cancel();
    }
    job.wait() // итог: ошибки, пропущенные, отмена
}

fn blocking() {
    let dir = std::env::temp_dir().join(format!("guide-fileops-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("from")).unwrap();
    std::fs::write(dir.join("from/a.txt"), "привет").unwrap();
    let r = FileJob::run_blocking(JobKind::Copy, vec![dir.join("from/a.txt")], Some(dir.clone()), Conflict::Overwrite);
    assert!(r.errors.is_empty() && dir.join("a.txt").exists());
    let _ = fileops::remove_any(&dir);
}
```

| Метод | Что |
|-------|-----|
| `FileJob::start(kind, sources, dest, conflict)` | начать в потоке (`dest` не нужен для `Delete`) |
| `FileJob::run_blocking(…) -> JobProgress` | то же в текущем потоке |
| `progress()` | снимок `JobProgress` |
| `cancel()` | остановить после текущего блока (1 МБ) |
| `is_finished()`, `wait()` | закончилась ли; дождаться и получить итог |
| `kind` | вид операции |

Назначение — как у `cp`/`mv`: существующий каталог — источники кладутся
внутрь, иначе (один источник) — новое имя. Перенос сначала пробует
переименование; недописанный при ошибке или остановке файл удаляется;
права и время сохраняются.

## `JobKind`

`Copy`, `Move`, `Delete`.

## `Conflict`

Что делать с уже существующим файлом назначения: `Overwrite` или `Skip`.
Спросите пользователя заранее с помощью `fileops::conflicts`.

## `JobProgress`

Снимок состояния: `counting` (идёт подсчёт объёма), `total_bytes`,
`done_bytes`, `total_files`, `done_files`, `current` (текущий файл),
`errors` («путь: описание»), `skipped`, `finished`, `cancelled`;
`percent()` — по байтам (у удаления — по файлам).

## Функции `fileops`

| Функция | Что |
|---------|-----|
| `conflicts(sources, dest)` | какие файлы назначения уже есть |
| `target_path(src, dest)` | куда попадёт источник |
| `dir_size(path) -> (байт, файлов)` | размер каталога |
| `is_inside(a, b)` | лежит ли `a` внутри `b` (защита от копирования каталога в себя) |
| `remove_any(path)` | удалить файл, ссылку или каталог целиком |
