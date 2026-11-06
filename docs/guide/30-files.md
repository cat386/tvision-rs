# Справочник: файловые панели и операции с файлами

Для программ вроде Norton Commander или Far: панели со списком файлов и
копирование в фоне с полосой выполнения. Большой пример —
`cargo run --example lfm` (файловый менеджер), подробности — [глава
17](../17-file-manager.md).

На этой странице:
[`FilePanel`](#filepanel--файловая-панель) ·
[`DualPanel`](#dualpanel--две-панели) ·
[`PanelEntry`](#panelentry--файл-в-панели) ·
[`FileKind`](#filekind) ·
[`FileClass`](#fileclass) ·
[`FileFilter`](#filefilter--что-скрыть) ·
[`SortMode`](#sortmode) ·
[`FilePanelColors`](#filepanelcolors) ·
[функции панелей](#функции-панелей) ·
[`FileJob`](#filejob--копирование-перенос-удаление) ·
[`JobKind`](#jobkind) ·
[`Conflict`](#conflict) ·
[`JobProgress`](#jobprogress) ·
[функции `fileops`](#функции-fileops)

---

## `FilePanel` — файловая панель

**Что это.** Список файлов каталога: ходить по каталогам, выделять
файлы, сортировать, искать набором букв, открывать несколько вкладок.
Файлы раскрашены по типам (архивы, картинки, исходники).

```rust
use std::path::Path;
use tvision::*;

fn panel_window(app: &mut Program, dir: &Path) -> ViewId {
    let mut w = Window::new(Rect::new(0, 0, 50, 20), "Файлы", 1);
    let mut p = FilePanel::new(w.interior(), dir);
    p.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y; // растягиваться вместе с окном
    p.set_sort(SortMode::Time, true); // новые сверху
    p.set_filter("*.o,*.tmp"); // эти файлы скрыть
    let id = w.add(p);
    app.insert_window(w);
    id
}

/// Выделенные файлы — или файл под курсором, если ничего не выделено.
fn selected(app: &Program, id: ViewId) -> Vec<std::path::PathBuf> {
    app.find::<FilePanel>(id).map(|p| p.targets()).unwrap_or_default()
}
```

**Клавиши:** стрелки, PgUp/PgDn, Home/End; Enter или → — войти в
каталог, Backspace или ← — наверх; Ins или пробел — выделить файл;
Ctrl+S или Alt+буква — поиск по имени; Ctrl+R — перечитать каталог.

**Главные методы:**

| Метод | Что делает |
|-------|-----------|
| `FilePanel::new(прямоугольник, каталог)` | панель |
| `path()`, `set_path(путь)`, `go_up()` | текущий каталог |
| `current()` | файл под курсором ([`PanelEntry`](#panelentry--файл-в-панели)) |
| `targets()` | выделенные файлы или файл под курсором |
| `marked()`, `toggle_mark()`, `mark_glob("*.rs", вкл)`, `clear_marks()` | выделение |
| `set_sort(вид, обратный)` | сортировка |
| `set_filter("*.o,*.tmp")`, `toggle_hidden()` | что скрыть |
| `refresh()` | перечитать каталог |

**Реже:** `change_dir(путь, Some(имя))`, `enter()`, `entries()`,
`cursor()`, `set_cursor(i)`, `select_name(имя)`, `select_path(путь)`,
`quick_find(текст)`, `invert_marks()`, `filter()`, `resort()`, вкладки
(`new_tab`, `close_tab`, `next_tab`, `prev_tab`, `select_tab`,
`tab_count`, `active_tab`, `tab_paths`), `set_listing(…)` и
`listing_title()` (показать произвольный список файлов, например
результаты поиска), `calc_dir_sizes()`, `dir_size(путь)`, `history()`,
`error()`, `set_message(s)`; поля `sort`, `sort_reverse`, `sort_mix_dirs`,
`sort_case_sensitive`, `show_hidden`, `colors`;
`filepanel::HISTORY_LIMIT`.

**Что важно знать.**

* Enter на **файле** ничего не открывает сам — панель посылает команду
  `cm::FILE_PANEL_EXECUTE`, а что делать с файлом, решает ваша программа
  (пример — в [`DualPanel`](#dualpanel--две-панели)).
* Панель сама не растягивается с окном — задайте `grow_mode`.
* Цвета берутся у окна, как у таблицы.

## `DualPanel` — две панели

**Что это.** Две панели рядом, как в Norton Commander: Tab переключает
между ними.

```rust
use std::path::Path;
use tvision::*;

struct App {
    panels: ViewId,
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        // Enter на файле в любой из панелей.
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

| Метод | Что делает |
|-------|-----------|
| `DualPanel::new(прямоугольник, левый_каталог, правый_каталог)` | |
| `active()`, `active_mut()` | панель, в которой курсор |
| `other()`, `other_mut()` | другая панель (например, куда копировать) |
| `swap()` | поменять панели местами |
| `sync_other()` | открыть в другой панели тот же каталог |
| `toggle_single()` | одна панель или две |

**Реже:** `left_id()`, `right_id()`, `active_id()`, `other_id()`,
`panel_mut(id)`, `switch(ctx)`, `layout()`, поля `single`, `group`.

## `PanelEntry` — файл в панели

Поля: `name`, `path`, `kind` ([`FileKind`](#filekind)), `size`,
`modified`, `mode` (права), `executable`, `hidden`. Методы: `is_dir()`,
`is_parent()` (это `..`), `extension()`, `class()`
([`FileClass`](#fileclass)), `perms_string()` (`rwxr-xr-x`);
`PanelEntry::from_path(путь, имя)` — создать без панели.

## `FileKind`

Что это за запись: `Parent` (`..`), `Dir`, `File`, `Link { to_dir }`,
`Other` (устройство, канал).

## `FileClass`

Тип файла — для цвета: `Normal`, `Dir`, `Exec`, `Link`, `Archive`,
`Image`, `Audio`, `Video`, `Document`, `Source`, `Data`, `Temp`.
Определяется по расширению функцией `class_by_name(имя)`.

## `FileFilter` — что скрыть

**Что это.** Шаблоны файлов, которые панель **не показывает**;
`!шаблон` — исключение.

```rust
use tvision::*;

fn filters() {
    let f = FileFilter::parse("*.o,*.tmp,!keep*");
    assert!(f.hides("main.o"));
    assert!(!f.hides("keep.tmp")); // исключение
    assert!(!f.hides("main.rs"));
    assert_eq!(class_by_name("photo.JPG"), FileClass::Image);
}
```

Методы: `parse(s)`, `hides(имя)`, `is_empty()`, `text()`. Шаблоны: `*`,
`?`, `[abc]`, `[a-z]`, `[!x]` (функция `glob_match(шаблон, имя,
учитывать_регистр)`).

## `SortMode`

Как сортировать: `Name` (по имени), `Ext` (по расширению), `Size`,
`Time`, `Unsorted`. `letter()` — буква для рамки, `SortMode::ALL` — все
виды.

## `FilePanelColors`

Свои цвета панели (`frame`, `cursor`, `marked`, `classes` и другие).
Обычно не нужны. `FilePanelColors::derive(рамка, рамка_активная, текст,
курсор)` — собрать набор из четырёх цветов.

## Функции панелей

| Функция | Пример результата |
|---------|------------------|
| `format_size_short(n)` | `12.3M` — размер в 7 символов |
| `format_size_full(n)` | `1 234 567` |
| `format_time_short(t)` | `05.11.26 14:30` |
| `format_time_full(t)` | `05.11.2026 14:30:00` |
| `available_roots()` | диски на Windows; `/`, домашний каталог, `/mnt/*`, `/media/*` на Unix |

---

## `FileJob` — копирование, перенос, удаление

**Что это.** Операция над файлами в **отдельном потоке**: программа не
«замирает», можно показывать полосу выполнения и остановить в любой
момент.

```rust
use std::path::PathBuf;
use tvision::fileops::{self, Conflict, FileJob, JobKind};
use tvision::*;

/// Скопировать с окном прогресса и кнопкой «Стоп».
fn copy(app: &mut Program, sources: Vec<PathBuf>, dest: PathBuf) -> fileops::JobProgress {
    let mut job = FileJob::start(JobKind::Copy, sources, Some(dest), Conflict::Skip);
    let mut d = Dialog::centered(50, 8, "Копирование");
    let bar = d.add(ProgressBar::new(Rect::new(2, 2, 48, 3), 100));
    d.add(Button::new(Rect::new(19, 4, 31, 6), "~С~топ", cm::CANCEL, bf::DEFAULT));
    // Пока диалог открыт, эта функция вызывается снова и снова.
    let (cmd, _) = app.exec_dialog_idle(d, |app, _| {
        let p = job.progress();
        app.find_mut::<ProgressBar>(bar)?.set_value(p.percent() as i64);
        p.finished.then_some(cm::OK) // закончилось — закрыть диалог
    });
    if cmd != cm::OK {
        job.cancel(); // нажали «Стоп»
    }
    job.wait() // итог: ошибки, пропущенные файлы
}

fn blocking() {
    let dir = std::env::temp_dir().join(format!("guide-fileops-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("from")).unwrap();
    std::fs::write(dir.join("from/a.txt"), "привет").unwrap();
    // Без потока — например, в тесте:
    let r = FileJob::run_blocking(JobKind::Copy, vec![dir.join("from/a.txt")], Some(dir.clone()), Conflict::Overwrite);
    assert!(r.errors.is_empty() && dir.join("a.txt").exists());
    let _ = fileops::remove_any(&dir);
}
```

| Метод | Что делает |
|-------|-----------|
| `FileJob::start(вид, файлы, куда, конфликт)` | начать в потоке (`куда` для удаления — `None`) |
| `progress()` | сколько сделано ([`JobProgress`](#jobprogress)) |
| `cancel()` | остановить |
| `wait()` | дождаться и получить итог |
| `is_finished()` | закончилось ли |
| `FileJob::run_blocking(…)` | то же без потока |

**Что важно знать.** «Куда» — как у команд `cp` и `mv`: если это
существующий каталог, файлы кладутся внутрь, иначе (для одного файла) —
это новое имя. Недокопированный файл при ошибке или остановке
удаляется.

## `JobKind`

`Copy` (копировать), `Move` (перенести), `Delete` (удалить).

## `Conflict`

Что делать, если файл в месте назначения уже есть: `Overwrite`
(заменить) или `Skip` (пропустить). Узнать заранее, какие файлы уже есть,
— функция `fileops::conflicts`.

## `JobProgress`

Снимок хода операции: `total_bytes`, `done_bytes`, `total_files`,
`done_files`, `current` (какой файл сейчас), `errors`, `skipped`,
`finished`, `cancelled`, `counting` (ещё считается объём); `percent()` —
сколько процентов сделано.

## Функции `fileops`

| Функция | Что делает |
|---------|-----------|
| `conflicts(файлы, куда)` | какие файлы в месте назначения уже есть |
| `target_path(файл, куда)` | куда попадёт файл |
| `dir_size(путь)` | размер каталога: (байт, файлов) |
| `is_inside(a, b)` | лежит ли `a` внутри `b` (нельзя копировать каталог в себя) |
| `remove_any(путь)` | удалить файл или каталог целиком |
