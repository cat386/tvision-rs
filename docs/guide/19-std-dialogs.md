# Справочник: стандартные диалоги — файлы, цвета, справка

Готовые диалоги и окна. Модули `widgets::filedlg`, `widgets::colorsel`,
`widgets::help`.

* Файлы: [`FileDialog`](#filedialog) · [`FileList`](#filelist) · [`FileEntry`](#fileentry) · [`FileInfoPane`](#fileinfopane) · [`ChDirDialog`](#chdirdialog) · [`DirListBox`](#dirlistbox) · [`FileDialogText`](#filedialogtext) · [функции](#функции-filedlg)
* Цвета: [`ColorDialog`](#colordialog) · [`ColorGroup`, `ColorItem`](#colorgroup-и-coloritem) · [`ColorSelector`, `ColorSel`](#colorselector-и-colorsel) · [`ColorDisplay`](#colordisplay) · [`Color256Dialog`](#color256dialog) · [`Color256Selector`](#color256selector) · [`ColorDialogText`](#colordialogtext)
* Справка: [`HelpFile`](#helpfile) · [`HelpTopic`, `Paragraph`, `CrossRef`](#helptopic-paragraph-crossref) · [`HelpViewer`](#helpviewer) · [`HelpWindow`](#helpwindow)

---

## `FileDialog`

**Выбор файла** (`TFileDialog`): строка ввода с историей, список файлов в
два столбца, сведения о файле, кнопки.

```rust
use tvision::*;

fn open(app: &mut Program) -> Option<std::path::PathBuf> {
    // шаблон, заголовок, подпись строки, кнопки, номер списка истории
    let d = FileDialog::new("*.txt", "Открыть файл", "~И~мя", fd::OPEN_BUTTON, 100);
    let (cmd, d) = app.exec_dialog(d);
    (cmd != cm::CANCEL).then(|| d.file_name())
}
```

| Флаг `fd` | Кнопка | Код завершения |
|-----------|--------|----------------|
| `OPEN_BUTTON` | «Открыть» | `cm::FILE_OPEN` |
| `OK_BUTTON` | «OK» (для «Сохранить как») | `cm::FILE_OPEN` |
| `REPLACE_BUTTON` | «Заменить» | `cm::FILE_REPLACE` |
| `CLEAR_BUTTON` | «Очистить» | `cm::FILE_CLEAR` |
| `HELP_BUTTON` | «Справка» | `cm::HELP` |
| `NO_LOAD_DIR` | — | не читать каталог при создании |

Кнопка «Отмена» есть всегда; проверяйте `cmd != cm::CANCEL` — разные
кнопки дают разные коды. Шаблон или каталог в строке + Enter — переход
туда, диалог остаётся открытым.

| Метод / поле | Что |
|--------------|-----|
| `FileDialog::new(wildcard, title, input_name, options, hist_id)` | |
| `file_name()` | полное имя выбранного файла |
| `wildcard`, `directory` | шаблон и каталог |
| `read_directory()` | перечитать каталог |
| `FileDialog::normalize(path)` | убрать `.` и `..` без обращения к ФС |
| `dlg` | сам диалог |
| данные | `Value::Str` — полное имя |

## `FileList`

**Список файлов** диалога (`TFileList`): два столбца, каталоги с
разделителем на конце, поиск по первым буквам. `FileList::new(bounds,
scroll)`, `set_entries(v)`, `focused_entry()`; поля `lb` (вложенный
`ListBox`), `entries`, `activated` (был двойной щелчок).

## `FileEntry`

**Запись о файле** (`TSearchRec`): `name`, `is_dir`, `size`, `modified`;
`display()` — текст для списка (каталог — с `/` на конце).

## `FileInfoPane`

**Сведения внизу диалога** (`TFileInfoPane`): каталог и шаблон, размер и
дата выбранного файла. `FileInfoPane::new(bounds)`; поля `path`, `entry`.

## `ChDirDialog`

**Смена каталога** (`TChDirDialog`): строка с именем, дерево каталогов,
кнопки «Перейти» (`cm::CHANGE_DIR`) и «Вернуть» (`cm::REVERT`). На OK
вызывается `std::env::set_current_dir`.

```rust
use tvision::*;

fn change_dir(app: &mut Program) -> Option<std::path::PathBuf> {
    let (cmd, d) = app.exec_dialog(ChDirDialog::new(cd::NORMAL, 0));
    (cmd == cm::OK).then(|| d.directory())
}
```

`ChDirDialog::new(options, hist_id)`; флаги `cd`: `NORMAL`,
`NO_LOAD_DIR`, `HELP_BUTTON`; `directory()` — выбранный каталог.

## `DirListBox`

**Дерево каталогов** диалога (`TDirListBox`): корень → путь → подкаталоги.
`DirListBox::new(bounds, scroll)`, `new_directory(path)`,
`focused_path()`; поля `lb`, `paths`, `dir`, `cur`, `activated`.

## `FileDialogText`

Тексты диалога (`FILES`, `OPEN`, `OK`, `REPLACE`, `CLEAR`, `CANCEL`,
`HELP`, `INVALID_DRIVE`, `INVALID_FILE`) — статические поля `TFileDialog`.
На русский переводятся `locale::set_language(Language::Ru)`.

## Функции `filedlg`

| Функция | Что |
|---------|-----|
| `filedlg::wild_match(pattern, name)` | `*` и `?`; на Windows без учёта регистра |
| `filedlg::is_wild(s)` | есть ли символы шаблона |
| `filedlg::read_directory(dir, wildcard) -> Vec<FileEntry>` | файлы по шаблону, все подкаталоги и `..` |
| `filedlg::civil_time(t)` | (год, месяц, день, час, минута) местного времени |

```rust
use tvision::widgets::filedlg::{is_wild, wild_match};

fn wild() {
    assert!(wild_match("*.rs", "main.rs"));
    assert!(wild_match("a?c", "abc"));
    assert!(is_wild("*.txt") && !is_wild("a.txt"));
}
```

---

## `ColorDialog`

**Настройка цветов** (`TColorDialog`): группы и элементы слева, сетки
16 цветов текста и фона, образец, кнопки выбора из 256 цветов. Правит
**копию** палитры приложения.

```rust
use tvision::*;

fn colors(app: &mut Program) {
    let (cmd, d) = app.exec_dialog(ColorDialog::new(&app.palette, standard_color_groups()));
    if cmd == cm::OK {
        app.palette = d.palette.clone(); // применить ко всей программе
    }
}
```

| Метод / поле | Что |
|--------------|-----|
| `ColorDialog::new(&palette, groups)` | |
| `palette`, `groups` | правленая палитра, группы |
| `current_attr()`, `set_current_attr(a)` | атрибут выбранного элемента |
| `dlg` | сам диалог |
| `standard_color_groups()` | группы стандартных элементов (рабочий стол, меню, окна, диалоги…) |

## `ColorGroup` и `ColorItem`

**Группа элементов** палитры (`TColorGroup`) и **элемент** — название и
номер цвета в палитре приложения (`TColorItem`).

```rust
use tvision::*;

fn my_groups() -> Vec<ColorGroup> {
    let mut g = standard_color_groups();
    // Свои цвета приложения: пары «название, номер в палитре приложения».
    g.push(ColorGroup::new("Мой редактор", &[("Текст", 6), ("Выделение", 7)]));
    g
}
```

Поля `ColorGroup`: `name`, `items`, `index` (последний выбранный);
`ColorItem`: `name`, `index`.

## `ColorSelector` и `ColorSel`

**Сетка 16 цветов** (`TColorSelector`): `ColorSelector::new(bounds,
ColorSel::Foreground / Background)`, поле `color`. `ColorSel` — что
выбирается: цвет текста или фона.

## `ColorDisplay`

**Образец текста** выбранным цветом (`TColorDisplay`):
`ColorDisplay::new(bounds, text)`, поля `text`, `color`.

## `Color256Dialog`

**Выбор одного из 256 цветов xterm** для текста или фона: сетка, номер,
RGB, образец.

```rust
use tvision::*;

fn pick_background(app: &mut Program, a: Attr) -> Option<Attr> {
    let (cmd, d) = app.exec_dialog(Color256Dialog::new(ColorSel::Background, a));
    (cmd == cm::OK).then(|| d.result()) // d.color() — сам номер 0–255
}
```

`Color256Dialog::new(sel_type, base)`; `color()` — номер, `result()` —
`base` с выбранным цветом; поля `dlg`, `sel_type`, `base`.

## `Color256Selector`

**Сетка 256 цветов** без диалога: `Color256Selector::new(x, y, цвет)` —
32 столбца × 8 строк, по две ячейки на цвет; поле `color`, константы
`COLS`, `ROWS`. Стрелки, Home/End, PgUp/PgDn, мышь.

## `ColorDialogText`

Тексты цветовых диалогов (`COLORS`, `GROUP`, `ITEM`, `FOREGROUND`,
`BACKGROUND`, `FORE_256`, `BACK_256`, …) — переводятся вместе с языком
интерфейса.

---

## `HelpFile`

**Файл справки** (`THelpFile`) — читается прямо из исходника в формате
утилиты `tvhc`, компилировать в `.hlp` не нужно.

```rust
use tvision::*;

const SRC: &str = "\
.topic NoContext=0
 Моя программа
Строки без пробела в начале собираются в абзацы. Ссылка: {окна:Windows}.

.topic Windows=10
 Окна
Окна можно двигать мышью.
";

fn help() {
    let h = HelpFile::parse(SRC).expect("справка");
    assert!(h.topic_for_context(10).is_some());
    assert!(h.topic_by_name("Windows").is_some());
    assert!(h.warnings.is_empty()); // нет ссылок на несуществующие темы
    assert!(h.context_constants().contains("HC_WINDOWS"));
}
```

Формат: `.topic Имя=номер, Синоним` начинает тему (номер — контекст
справки `help_ctx`; без номера — следующий по порядку, начиная с 2).
Строки без пробела в начале — абзацы с переносом, с пробелом — как есть.
`{текст:Тема}` и `{Тема}` — ссылки. Подробно — [глава 10](../10-std-dialogs.md#справка).

| Метод / поле | Что |
|--------------|-----|
| `HelpFile::parse(src)`, `HelpFile::load(path)` | разобрать (ошибка — `Err(текст)`) |
| `topic_for_context(n)`, `topic_by_name(name)` | тема |
| `topics`, `symbols`, `warnings` | темы, имена с номерами, предупреждения |
| `context_constants()` | `pub const HC_…: u16` на Rust (замена `.h` из `tvhc`) |
| `HelpFile::invalid_topic()` | тема «нет справки» (`help::NO_HELP`) |

## `HelpTopic`, `Paragraph`, `CrossRef`

**Тема** — абзацы и ссылки (`THelpTopic`): поля `paragraphs`,
`cross_refs`. **Абзац** (`TParagraph`): `text`, `wrap` (переносить ли по
ширине). **Ссылка** (`TCrossRef`): `offset` и `length` в тексте темы,
`target` — имя темы.

## `HelpViewer`

**Просмотр темы** (`THelpViewer`): Tab/Shift+Tab — по ссылкам, Enter или
щелчок — перейти, прокрутка. `HelpViewer::new(bounds, h, v, file,
context)`; `switch_to(name)` — перейти к теме; `text_lines()`,
`selected_target()` — для тестов; поля `file`, `scroller`. Встраивается
в своё окно, если стандартное не подходит.

## `HelpWindow`

**Окно справки** (`THelpWindow`) 50×18 по центру, его можно двигать и
растягивать.

```rust
use std::rc::Rc;
use tvision::*;

struct App {
    help: Rc<HelpFile>,
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(cm::HELP) {
            let ctx = app.help_ctx(); // контекст представления в фокусе
            app.exec_dialog(HelpWindow::new(self.help.clone(), ctx));
            ev.clear();
        }
    }
}
```

`HelpWindow::new(file, context)`; `viewer()`, поля `win`, `viewer` (id).
Чтобы F1 работал и в диалогах, добавьте в строку состояния
`StatusItem::new("~F1~ Справка", kb::F1, cm::HELP)` и обработайте
`cm::HELP` в `Program::modal_hook`. Заголовок — `help::HELP_WIN_TITLE`.
