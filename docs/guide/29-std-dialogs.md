# Справочник: готовые диалоги — файлы, цвета, справка

Диалоги, которые нужны почти каждой программе, уже написаны.

На этой странице:

* **Файлы:** [`FileDialog`](#filedialog--выбор-файла) ·
  [`ChDirDialog`](#chdirdialog--смена-каталога) ·
  [части диалогов](#части-файловых-диалогов) ·
  [функции](#функции-для-файлов)
* **Цвета:** [`ColorDialog`](#colordialog--настройка-цветов) ·
  [`ColorGroup`, `ColorItem`](#colorgroup-и-coloritem) ·
  [`Color256Dialog`](#color256dialog--выбор-одного-цвета) ·
  [части диалогов](#части-цветовых-диалогов)
* **Справка:** [`HelpFile`](#helpfile--файл-справки) ·
  [`HelpWindow`](#helpwindow--окно-справки) ·
  [`HelpViewer`](#helpviewer) ·
  [`HelpTopic`, `Paragraph`, `CrossRef`](#helptopic-paragraph-crossref)

---

## `FileDialog` — выбор файла

**Что это.** Стандартный диалог «Открыть» / «Сохранить как»: поле для
имени, список файлов и каталогов, сведения о выбранном файле.

**Аналог:** `TFileDialog`.

```rust
use tvision::*;

fn open(app: &mut Program) -> Option<std::path::PathBuf> {
    // шаблон, заголовок, подпись поля, кнопки, номер истории
    let d = FileDialog::new("*.txt", "Открыть файл", "~И~мя", fd::OPEN_BUTTON, 100);
    let (cmd, d) = app.exec_dialog(d);
    (cmd != cm::CANCEL).then(|| d.file_name()) // полный путь
}
```

`FileDialog::new(шаблон, заголовок, подпись, кнопки, номер_истории)`:

* **шаблон** — какие файлы показывать: `"*.txt"`, `"*"`;
* **кнопки**: `fd::OPEN_BUTTON` («Открыть»), `fd::OK_BUTTON` («OK» — для
  «Сохранить как»), `fd::REPLACE_BUTTON`, `fd::CLEAR_BUTTON`,
  `fd::HELP_BUTTON`; «Отмена» есть всегда;
* **номер истории** — диалог запоминает введённые имена; у каждого
  диалога свой номер.

**Что важно знать.**

* Проверяйте `cmd != cm::CANCEL`, а не `cmd == cm::OK`: кнопки «Открыть»
  и «OK» возвращают `cm::FILE_OPEN`, «Заменить» — `cm::FILE_REPLACE`.
* Если ввести в поле шаблон (`*.rs`) или каталог и нажать Enter, диалог
  перейдёт туда и останется открытым.
* `d.file_name()` — полный путь к выбранному файлу.

**Реже:** поля `wildcard`, `directory`, `dlg`; методы `read_directory()`,
`FileDialog::normalize(путь)`; флаг `fd::NO_LOAD_DIR`.

## `ChDirDialog` — смена каталога

**Что это.** Диалог выбора каталога с деревом каталогов. На OK меняет
текущий каталог программы.

**Аналог:** `TChDirDialog`.

```rust
use tvision::*;

fn change_dir(app: &mut Program) -> Option<std::path::PathBuf> {
    let (cmd, d) = app.exec_dialog(ChDirDialog::new(cd::NORMAL, 0));
    (cmd == cm::OK).then(|| d.directory())
}
```

`ChDirDialog::new(флаги, номер_истории)`; флаги `cd::NORMAL`,
`cd::NO_LOAD_DIR`, `cd::HELP_BUTTON`; `directory()` — выбранный каталог.

## Части файловых диалогов

Из этих частей собраны диалоги выше; отдельно нужны редко.

* **`FileList`** — список файлов в два столбца (`TFileList`):
  `set_entries(…)`, `focused_entry()`; поля `lb`, `entries`, `activated`.
* **`FileEntry`** — запись о файле: `name`, `is_dir`, `size`, `modified`;
  `display()` — как показывать в списке.
* **`FileInfoPane`** — строки сведений внизу диалога (`TFileInfoPane`):
  поля `path`, `entry`.
* **`DirListBox`** — дерево каталогов (`TDirListBox`): `new_directory(путь)`,
  `focused_path()`; поля `lb`, `paths`, `dir`, `cur`, `activated`.
* **`FileDialogText`** — надписи диалогов («Открыть», «Неверное имя
  файла»…); по-русски — с `locale::set_language(Language::Ru)`.

## Функции для файлов

```rust
use tvision::widgets::filedlg::{is_wild, wild_match};

fn wild() {
    assert!(wild_match("*.rs", "main.rs")); // подходит ли имя под шаблон
    assert!(wild_match("a?c", "abc"));
    assert!(is_wild("*.txt") && !is_wild("a.txt")); // шаблон ли это
}
```

В модуле `tvision::widgets::filedlg`: `wild_match(шаблон, имя)`,
`is_wild(s)`, `read_directory(каталог, шаблон)` (список файлов),
`civil_time(время)` (год, месяц, день, час, минута).

---

## `ColorDialog` — настройка цветов

**Что это.** Диалог, в котором пользователь сам перекрашивает программу:
слева — что красить (рабочий стол, меню, окна, диалоги…), справа — цвета
текста и фона и образец. Есть кнопки для выбора из 256 цветов.

**Аналог:** `TColorDialog`.

```rust
use tvision::*;

fn colors(app: &mut Program) {
    let (cmd, d) = app.exec_dialog(ColorDialog::new(&app.palette, standard_color_groups()));
    if cmd == cm::OK {
        app.palette = d.palette.clone(); // применить ко всей программе
    }
}
```

`ColorDialog::new(&палитра, группы)` правит **копию** палитры; чтобы
применить, присвойте `d.palette` программе. `standard_color_groups()` —
стандартный список того, что можно красить. Палитра — обычный
`Vec<Attr>`, её легко сохранить в файл.

**Реже:** `current_attr()`, `set_current_attr(a)`, поля `groups`, `dlg`.

## `ColorGroup` и `ColorItem`

**Что это.** Строки левой части диалога: **группа** («Меню», «Диалоги»)
и **элемент** внутри неё («Обычный текст», «Выбранный пункт») с номером
цвета в палитре программы.

```rust
use tvision::*;

fn my_groups() -> Vec<ColorGroup> {
    let mut g = standard_color_groups();
    // Своя группа: пары «название, номер цвета в палитре программы».
    g.push(ColorGroup::new("Мой редактор", &[("Текст", 6), ("Выделение", 7)]));
    g
}
```

Поля: у `ColorGroup` — `name`, `items`, `index`; у `ColorItem` — `name`,
`index`.

## `Color256Dialog` — выбор одного цвета

**Что это.** Сетка всех 256 цветов: выбрать цвет текста или фона.

```rust
use tvision::*;

fn pick_background(app: &mut Program, a: Attr) -> Option<Attr> {
    let (cmd, d) = app.exec_dialog(Color256Dialog::new(ColorSel::Background, a));
    (cmd == cm::OK).then(|| d.result()) // `a` с новым фоном
}
```

`Color256Dialog::new(что_выбираем, исходный_цвет)`; `ColorSel::Foreground`
или `ColorSel::Background`. `result()` — исходный цвет с выбранным,
`color()` — номер выбранного (0–255).

## Части цветовых диалогов

* **`ColorSelector`** — сетка 16 цветов (`TColorSelector`):
  `ColorSelector::new(прямоугольник, ColorSel::…)`, поле `color`.
* **`ColorSel`** — что выбирать: `Foreground` (текст) или `Background`
  (фон).
* **`ColorDisplay`** — образец текста выбранным цветом (`TColorDisplay`):
  поля `text`, `color`.
* **`Color256Selector`** — сетка 256 цветов без диалога:
  `Color256Selector::new(x, y, цвет)`, поле `color`, константы `COLS`,
  `ROWS`.
* **`ColorDialogText`** — надписи диалогов (переводятся вместе с
  языком).

---

## `HelpFile` — файл справки

**Что это.** Справка программы: темы с текстом и ссылками друг на друга.
Пишется простым текстом — программа читает его сама.

**Аналог:** `THelpFile` (в оригинале справку нужно было компилировать
утилитой `tvhc`; здесь не нужно).

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
    let h = HelpFile::parse(SRC).expect("ошибка в справке");
    assert!(h.topic_for_context(10).is_some()); // тема с номером 10
    assert!(h.warnings.is_empty()); // ссылки ведут на существующие темы
}
```

**Как писать справку:**

* `.topic Имя=номер` — начало темы. **Номер** — тот же, что в
  `vs.help_ctx` элемента: по F1 откроется тема элемента в фокусе;
* строка, которая начинается **без пробела**, — часть абзаца (абзацы
  переносятся по ширине окна); **с пробелом** — выводится как есть (для
  заголовков и таблиц);
* `{текст:Тема}` — ссылка на тему `Тема` с текстом `текст`; `{Тема}` —
  ссылка с именем темы.

Справку удобно хранить в файле и встроить в программу:
`HelpFile::parse(include_str!("help.txt"))`. Подробно — [глава 10](../10-std-dialogs.md#справка).

**Методы:** `HelpFile::parse(текст)`, `HelpFile::load(путь)`,
`topic_for_context(номер)`, `topic_by_name(имя)`. **Реже:** поля
`topics`, `symbols`, `warnings`; `context_constants()` (константы
`HC_…` для кода), `HelpFile::invalid_topic()`.

## `HelpWindow` — окно справки

**Что это.** Окно, которое показывает тему и даёт ходить по ссылкам:
Tab — к следующей ссылке, Enter — перейти, Esc — закрыть.

**Аналог:** `THelpWindow`.

```rust
use std::rc::Rc;
use tvision::*;

struct App {
    help: Rc<HelpFile>,
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(cm::HELP) {
            let topic = app.help_ctx(); // номер темы элемента в фокусе
            app.exec_dialog(HelpWindow::new(self.help.clone(), topic));
            ev.clear();
        }
    }
}
```

`HelpWindow::new(справка, номер_темы)`. Чтобы F1 работал везде, добавьте
в строку состояния `StatusItem::new("~F1~ Справка", kb::F1, cm::HELP)`,
а для открытых диалогов обработайте `cm::HELP` ещё и в `app.modal_hook`.
**Реже:** `viewer()`, поля `win`, `viewer`; заголовок —
`help::HELP_WIN_TITLE`.

## `HelpViewer`

**Что это.** Сам просмотр темы внутри окна справки — если нужно
встроить справку в своё окно. `HelpViewer::new(прямоугольник, гор.,
верт., справка, номер_темы)`, `switch_to(имя_темы)`; для тестов —
`text_lines()`, `selected_target()`; поля `file`, `scroller`.

## `HelpTopic`, `Paragraph`, `CrossRef`

Как справка устроена внутри: **тема** (`THelpTopic`) — абзацы
`paragraphs` и ссылки `cross_refs`; **абзац** (`TParagraph`) — `text` и
`wrap` (переносить ли); **ссылка** (`TCrossRef`) — где в тексте (`offset`,
`length`) и на какую тему (`target`). Тема-заглушка «справки нет» —
`help::NO_HELP`.
