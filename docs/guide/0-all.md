# Руководство пользователя tvision-rs

Полное руководство по библиотеке: урок, в котором шаг за шагом строится
настоящее приложение, и справочник по **каждому** публичному типу — что
это, чему соответствует в оригинальном Turbo Vision, как создать, какие
у него поля и методы, пример использования.

Все примеры кода в руководстве проверены: они компилируются с текущей
версией библиотеки, а проверки `assert!` в них выполняются.

## С чего начать

1. **[Урок: записная книжка «Контакты»](01-tutorial.md)** — программа с
   меню, окном-таблицей, диалогом с проверкой ввода, файлами и тестами.
   Готовый код — `examples/guide_contacts.rs`
   (`cargo run --example guide_contacts`).
2. Справочник ниже — по мере надобности. Каждая страница начинается со
   списка типов со ссылками.
3. За подробностями «как это устроено» — [тематические главы](../README.md)
   (архитектура, события, цвета, перенос программ с C++).

Если вы знаете Turbo Vision для C++ или Pascal, почти все типы называются
так же без буквы `T`: `TButton` → `Button`, `TDialog` → `Dialog`. Главное
отличие — представления не хранят указатель на владельца, а действуют
через контекст `Ctx` ([шаг 0 урока](01-tutorial.md#шаг-0-как-устроена-программа-на-tvision-rs),
[глава 15](../15-porting.md)).

## Справочник по классам

| Страница | Что внутри |
|----------|-----------|
| [Геометрия, цвета, буферы, текст](10-basics.md) | `Point`, `Rect`, `Attr`, `AttrPair`, `ColorMode`, палитры, `Cell`, `DrawBuffer`, `ScreenBuffer`, функции текста |
| [События, клавиши, команды](11-events.md) | `Key`, `Mods`, `KeyEvent`, `kb`, `MouseEvent`, `Event`, `Message`, `Info`, `cm` |
| [Представления](12-views.md) | `View`, `ViewState`, `ViewId`, флаги `sf` `of` `gf` `dm`, `Value`, `Ctx`, `Request`, `CommandSet`, `Phase`, `DrawCtx`, `Group`, макросы |
| [Программа, рабочий стол, окна, диалоги](13-program.md) | `Program`, `AppHandler`, `NoHandler`, `Desktop`, `Background`, `Window`, `wf`, `Frame`, `Dialog`, `DialogPalette` |
| [Меню и строка состояния](14-menus.md) | `Menu`, `MenuItem`, `MenuBar`, контекстное меню, `MenuSession`, `StatusLine`, `StatusDef`, `StatusItem`, `HintFn` |
| [Проверка ввода](15-validators.md) | `Validator`, `PictureValidator`, `PicResult`, `FilterValidator`, `RangeValidator`, `StringLookupValidator`, `FnValidator` |
| [Элементы управления](16-controls.md) | `Button`, `StaticText`, `Label`, `InputLine`, `InputLong`, `Cluster`, `ScrollBar`, `ListBox`, `History`, `Combo`, `ProgressBar`, окна сообщений |
| [Таблица, дерево, прокрутка](17-data-views.md) | `Table`, `TableModel`, `VecTableModel`, `FnTableModel`, `TableColumn`, `TreeView`, `TreeNode`, `Outline`, `Scroller`, `Terminal` |
| [Текстовый редактор](18-editor.md) | `Editor`, `Pos`, `Memo`, `FileEditor`, `EditWindow`, `Indicator` |
| [Стандартные диалоги](19-std-dialogs.md) | `FileDialog`, `ChDirDialog`, `ColorDialog`, `Color256Dialog`, `HelpFile`, `HelpWindow` и их части |
| [Файловые панели и операции](20-files.md) | `FilePanel`, `DualPanel`, `PanelEntry`, `FileFilter`, `FileJob`, `JobProgress` |
| [Терминал, буфер обмена, время, язык](21-platform.md) | `Backend`, `HeadlessBackend`, `UnixBackend`, `WindowsBackend`, `Clipboard`, `DateTime`, `Language` |

## Указатель типов

**A** — [`Align`](17-data-views.md#tablecolumn-и-align) · [`AnsiWriter`](21-platform.md#ansiwriter-inputparser-parsed) · [`AppHandler`](13-program.md#apphandler) · [`Attr`](10-basics.md#attr-и-attrpair) · [`AttrPair`](10-basics.md#attr-и-attrpair)

**B** — [`Backend`](21-platform.md#backend) · [`Background`](13-program.md#background) · [`Button`](16-controls.md#button)

**C** — [`Cell`](10-basics.md#cell) · [`ChDirDialog`](19-std-dialogs.md#chdirdialog) · [`CheckBoxes`](16-controls.md#cluster) · [`CheckState`](17-data-views.md#checkstate) · [`Clipboard`](21-platform.md#clipboard) · [`ClipboardBackend`](21-platform.md#clipboardbackend) · [`Cluster`](16-controls.md#cluster) · [`ClusterKind`](16-controls.md#cluster) · [`Color256Dialog`](19-std-dialogs.md#color256dialog) · [`Color256Selector`](19-std-dialogs.md#color256selector) · [`ColorDialog`](19-std-dialogs.md#colordialog) · [`ColorDialogText`](19-std-dialogs.md#colordialogtext) · [`ColorDisplay`](19-std-dialogs.md#colordisplay) · [`ColoredText`](16-controls.md#statictext-и-coloredtext) · [`ColorGroup`](19-std-dialogs.md#colorgroup-и-coloritem) · [`ColorItem`](19-std-dialogs.md#colorgroup-и-coloritem) · [`ColorMode`](10-basics.md#colormode) · [`ColorSel`](19-std-dialogs.md#colorselector-и-colorsel) · [`ColorSelector`](19-std-dialogs.md#colorselector-и-colorsel) · [`Combo`](16-controls.md#combo-и-combowindow) · [`ComboWindow`](16-controls.md#combo-и-combowindow) · [`CommandSet`](12-views.md#commandset) · [`Conflict`](20-files.md#conflict) · [`CrossRef`](19-std-dialogs.md#helptopic-paragraph-crossref) · [`Ctx`](12-views.md#ctx)

**D** — [`DateTime`](21-platform.md#datetime-и-функции-времени) · [`Desktop`](13-program.md#desktop) · [`Dialog`](13-program.md#dialog) · [`DialogPalette`](13-program.md#dialogpalette) · [`DirListBox`](19-std-dialogs.md#dirlistbox) · [`DrawBuffer`](10-basics.md#drawbuffer) · [`DrawCtx`](12-views.md#drawctx) · [`DualPanel`](20-files.md#dualpanel)

**E** — [`Editor`](18-editor.md#editor) · [`EditWindow`](18-editor.md#editwindow) · [`Event`](11-events.md#event)

**F** — [`FileClass`](20-files.md#fileclass) · [`FileDialog`](19-std-dialogs.md#filedialog) · [`FileDialogText`](19-std-dialogs.md#filedialogtext) · [`FileEditor`](18-editor.md#fileeditor) · [`FileEntry`](19-std-dialogs.md#fileentry) · [`FileFilter`](20-files.md#filefilter) · [`FileInfoPane`](19-std-dialogs.md#fileinfopane) · [`FileJob`](20-files.md#filejob) · [`FileKind`](20-files.md#filekind) · [`FileList`](19-std-dialogs.md#filelist) · [`FilePanel`](20-files.md#filepanel) · [`FilePanelColors`](20-files.md#filepanelcolors) · [`FilterValidator`](15-validators.md#filtervalidator) · [`FnTableModel`](17-data-views.md#fntablemodel) · [`FnValidator`](15-validators.md#fnvalidator) · [`Frame`](13-program.md#frame-и-frameinfo) · [`FrameInfo`](13-program.md#frame-и-frameinfo)

**G** — [`Group`](12-views.md#group)

**H** — [`HeadlessBackend`](21-platform.md#headlessbackend) · [`HelpFile`](19-std-dialogs.md#helpfile) · [`HelpTopic`](19-std-dialogs.md#helptopic-paragraph-crossref) · [`HelpViewer`](19-std-dialogs.md#helpviewer) · [`HelpWindow`](19-std-dialogs.md#helpwindow) · [`HintFn`](14-menus.md#hintfn) · [`History`](16-controls.md#history-и-historywindow) · [`HistoryWindow`](16-controls.md#history-и-historywindow)

**I** — [`Indicator`](18-editor.md#indicator) · [`Info`](11-events.md#info) · [`InputLine`](16-controls.md#inputline) · [`InputLong`](16-controls.md#inputlong) · [`InputParser`](21-platform.md#ansiwriter-inputparser-parsed)

**J** — [`JobKind`](20-files.md#jobkind) · [`JobProgress`](20-files.md#jobprogress)

**K** — [`Key`](11-events.md#key) · [`KeyEvent`](11-events.md#keyevent)

**L** — [`Label`](16-controls.md#label) · [`Language`](21-platform.md#язык-интерфейса) · [`ListBox`](16-controls.md#listbox)

**M** — [`Memo`](18-editor.md#memo) · [`Menu`](14-menus.md#menu) · [`MenuBar`](14-menus.md#menubar) · [`MenuItem`](14-menus.md#menuitem) · [`MenuOutcome`](14-menus.md#menusession-и-menuoutcome) · [`MenuSession`](14-menus.md#menusession-и-menuoutcome) · [`Message`](11-events.md#message) · [`Mods`](11-events.md#mods) · [`MouseEvent`](11-events.md#mouseevent) · [`MsgBoxText`](16-controls.md#окна-сообщений) · [`MultiCheckBoxes`](16-controls.md#cluster)

**N** — [`Node`](17-data-views.md#outline-и-node) · [`NoHandler`](13-program.md#nohandler)

**O** — [`Outline`](17-data-views.md#outline-и-node)

**P** — [`PanelEntry`](20-files.md#panelentry) · [`Paragraph`](19-std-dialogs.md#helptopic-paragraph-crossref) · [`Parsed`](21-platform.md#ansiwriter-inputparser-parsed) · [`Phase`](12-views.md#phase) · [`PicResult`](15-validators.md#picresult) · [`PictureValidator`](15-validators.md#picturevalidator) · [`PlaceFn`](12-views.md#request-и-placefn) · [`Point`](10-basics.md#point) · [`Pos`](18-editor.md#pos) · [`Program`](13-program.md#program) · [`ProgressBar`](16-controls.md#progressbar) · [`ProgressStyle`](16-controls.md#progressbar)

**R** — [`RadioButtons`](16-controls.md#cluster) · [`RangeValidator`](15-validators.md#rangevalidator) · [`RawEvent`](21-platform.md#rawevent) · [`Rect`](10-basics.md#rect) · [`Request`](12-views.md#request-и-placefn)

**S** — [`ScreenBuffer`](10-basics.md#screenbuffer) · [`ScrollBar`](16-controls.md#scrollbar) · [`Scroller`](17-data-views.md#scroller) · [`SelectionMode`](17-data-views.md#selectionmode) · [`SortMode`](20-files.md#sortmode) · [`StaticText`](16-controls.md#statictext-и-coloredtext) · [`StatusDef`](14-menus.md#statusdef) · [`StatusItem`](14-menus.md#statusitem) · [`StatusLine`](14-menus.md#statusline) · [`StringLookupValidator`](15-validators.md#stringlookupvalidator)

**T** — [`Table`](17-data-views.md#table) · [`TableColors`](17-data-views.md#tablecolors) · [`TableColumn`](17-data-views.md#tablecolumn-и-align) · [`TableModel`](17-data-views.md#tablemodel) · [`Terminal`](17-data-views.md#terminal) · [`TreeColors`](17-data-views.md#treecolors) · [`TreeGlyphs`](17-data-views.md#treeglyphs) · [`TreeLoader`](17-data-views.md#treepath-и-treeloader) · [`TreeNode`](17-data-views.md#treenode) · [`TreePath`](17-data-views.md#treepath-и-treeloader) · [`TreeView`](17-data-views.md#treeview)

**U** — [`UnixBackend`](21-platform.md#unixbackend)

**V** — [`Validator`](15-validators.md#validator) · [`Value`](12-views.md#value) · [`VecTableModel`](17-data-views.md#vectablemodel) · [`View`](12-views.md#view) · [`ViewId`](12-views.md#viewid) · [`ViewState`](12-views.md#viewstate)

**W** — [`Window`](13-program.md#window) · [`WindowsBackend`](21-platform.md#windowsbackend)

## Модули констант

| Модуль | Что | Где описан |
|--------|-----|-----------|
| `cm` | команды | [события](11-events.md#cm--команды), [номера](../16-reference.md#cm--команды) |
| `kb` | клавиши | [события](11-events.md#kb--константы-клавиш) |
| `sf`, `of`, `gf`, `dm` | состояние, параметры, растяжение, перетаскивание | [представления](12-views.md#флаги) |
| `wf` | флаги окна | [окна](13-program.md#wf--флаги-окна) |
| `bf` | флаги кнопки | [Button](16-controls.md#button) |
| `mf` | окна сообщений | [окна сообщений](16-controls.md#окна-сообщений) |
| `cf` | многосостоятельные флажки | [Cluster](16-controls.md#cluster) |
| `cbx`, `il` | `Combo`, `InputLong` | [Combo](16-controls.md#combo-и-combowindow), [InputLong](16-controls.md#inputlong) |
| `ef`, `ec` | поиск в редакторе, команды редактора | [редактор](18-editor.md#ef-и-ec) |
| `fd`, `cd` | файловый диалог, смена каталога | [диалоги](19-std-dialogs.md#filedialog) |
| `color::palettes` | палитры | [цвета](10-basics.md#палитры-colorpalettes) |

Все константы с номерами — [глава 16](../16-reference.md).

## Как подключить

```toml
[dependencies]
tvision-rs = { git = "https://github.com/cat386/tvision-rs" }
```

```rust,ignore
use tvision::*; // крейт называется `tvision`
```

`use tvision::*;` подключает все типы и модули констант. Отдельно
импортируются только редкие вещи: `tvision::event::MB_LEFT`,
`tvision::platform::{HeadlessBackend, RawEvent}`,
`tvision::platform::clipboard::*`, `tvision::fileops::*`,
`tvision::program::NoHandler`.


# Урок: записная книжка «Контакты»

В этом уроке мы шаг за шагом построим настоящее приложение — записную
книжку с таблицей контактов, диалогом правки, проверкой ввода, файлами и
тестами. Готовый код — `examples/guide_contacts.rs`:

```text
cargo run --example guide_contacts            # с образцом данных
cargo run --example guide_contacts -- my.tsv  # открыть или создать файл
cargo test --example guide_contacts           # тесты из шага 6
```

```text
  Файл  Контакт
┌──────────────────── Контакты — без имени ──────────────────────1─────┐
│         Имя          │     Телефон      │        Почта         │ ★ │▲
│──────────────────────┼──────────────────┼──────────────────────┼───┼▒
│Анна Петрова          │+7 900 111-22-33  │anna@example.ru       │ ★ │▒
│Борис Смирнов         │+7 900 222-33-44  │boris@example.ru      │   │▒
 Ins Добавить  F4 Изменить  Del Удалить  F2 Сохранить  Alt-X Выход
```

Фрагменты кода ниже взяты из этого файла; чтобы увидеть их вместе, держите
его открытым рядом. Подробности о каждом типе — в
[справочнике](README.md#справочник-по-классам).

## Шаг 0. Как устроена программа на tvision-rs

Прежде чем писать код, полезно знать четыре идеи — они такие же, как в
Turbo Vision для DOS:

1. **Всё на экране — представления** (`View`): строка меню, рабочий стол,
   окно, кнопка. Представления вложены друг в друга: программа → рабочий
   стол → окно → таблица. Группы (`Group`, `Window`, `Dialog`) владеют
   детьми.
2. **Программа живёт в цикле событий.** `Program::run` ждёт нажатие
   клавиши или мыши, доставляет событие нужному представлению, перерисовывает
   экран — и так до команды выхода.
3. **Команды** — числа (`u16`). Пункт меню, кнопка, клавиша строки
   состояния посылают команду; кто-то её обрабатывает. Команды можно
   запрещать — тогда пункты меню становятся серыми.
4. **Свой код приложения** — обработчик `AppHandler`. Ему достаются
   команды, которые не обработали представления.

Важное отличие от C++: представление не хранит указатель на своё окно.
Чтобы что-то сделать «снаружи себя» (послать команду, закрыть себя,
открыть модальный диалог), оно пишет запрос в контекст `Ctx`, и владелец
выполняет его сразу после обработки события.

## Шаг 1. Программа, меню и строка состояния

Сначала — номера своих команд. Стандартные (`cm::OK`, `cm::QUIT`,
`cm::SAVE`…) занимают номера до 100, свои начинаем со 100:

```rust,ignore
const CM_ADD: u16 = 100;
const CM_EDIT: u16 = 101;
const CM_DELETE: u16 = 102;
const CM_SORT: u16 = 103;
const CM_EXIT: u16 = 104;
```

> Номера до 255 можно **запрещать** (`disable_command`); команды с
> большими номерами разрешены всегда. Поэтому команды, которые бывают
> недоступны, нумеруйте от 100 до 255.

Меню собирается цепочкой вызовов. Тильды выделяют горячую букву: в
`"~Ф~айл"` подменю открывается по Alt+Ф (и по Alt+A в латинской
раскладке — см. [язык и раскладки](../13-locale.md)).

```rust,ignore
fn menu() -> Menu {
    Menu::new()
        .submenu(
            "~Ф~айл",
            Menu::new()
                .item("~Н~овый", cm::NEW)
                .item_key("~О~ткрыть...", cm::OPEN, kb::F3, "F3")
                .item_key("~С~охранить", cm::SAVE, kb::F2, "F2")
                .item("Сохранить ~к~ак...", cm::SAVE_AS)
                .separator()
                .item_key("~В~ыход", CM_EXIT, kb::ALT_X, "Alt-X"),
        )
        .submenu(
            "~К~онтакт",
            Menu::new()
                .item_key("~Д~обавить...", CM_ADD, kb::key(Key::Ins), "Ins")
                .item_key("~И~зменить...", CM_EDIT, kb::f(4), "F4")
                .item_key("~У~далить", CM_DELETE, kb::key(Key::Del), "Del")
                .separator()
                .item("~С~ортировать по имени", CM_SORT),
        )
}
```

`item_key(название, команда, клавиша, подпись)` — пункт с клавишей
быстрого вызова: F3 работает, даже когда меню закрыто. Подпись `"F3"`
только рисуется справа от названия.

Строка состояния — список наборов пунктов (`StatusDef`); набор выбирается
по контексту справки. Нам хватит одного набора на все случаи:

```rust,ignore
fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![
        StatusItem::new("~Ins~ Добавить", kb::key(Key::Ins), CM_ADD),
        StatusItem::new("~F4~ Изменить", kb::f(4), CM_EDIT),
        StatusItem::new("~Del~ Удалить", kb::key(Key::Del), CM_DELETE),
        StatusItem::new("~F2~ Сохранить", kb::F2, cm::SAVE),
        StatusItem::new("~Alt-X~ Выход", kb::ALT_X, CM_EXIT),
        StatusItem::new("", kb::F10, cm::MENU), // невидимый пункт: F10 — меню
    ])]
}
```

Пункт с пустым текстом не виден, но его клавиша работает — так F10
открывает меню.

> **Осторожно с Enter.** Строка состояния видит клавиши раньше всех, даже
> во время модальных диалогов. Пункт на Enter забрал бы Enter у диалогов,
> поэтому «Изменить» висит на F4.

Программа создаётся одной строкой и запускается методом `run`, которому
передаётся обработчик приложения:

```rust,ignore
fn main() -> std::io::Result<()> {
    let mut app = Program::with_terminal(Some(menu()), Some(status()))?;
    // …открыть окно (шаг 2)…
    app.run(&mut App);
    Ok(())
}
```

`with_terminal` переводит терминал в полноэкранный режим (на Windows —
консоль), а при завершении программы возвращает его в обычное состояние.

Обработчик пока может быть пустым — `struct App; impl AppHandler for App {}`.
Такая программа уже работает: меню открывается по F10 и мышью, Alt-X…
пока ничего не делает, потому что `CM_EXIT` никто не обрабатывает. Это мы
исправим в шаге 4.

## Шаг 2. Своё окно с таблицей

Данные — обычная структура Rust:

```rust,ignore
pub struct Contact {
    pub name: String,
    pub phone: String,
    pub email: String,
    /// Номер группы (0 — «Семья», 1 — «Работа», 2 — «Друзья»).
    pub group: u32,
    pub favorite: bool,
}
```

Окно, которое показывает контакты, — **своё представление**. Оно хранит
стандартное окно `Window` (рамка, заголовок, дети) и данные:

```rust,ignore
pub struct ContactsWindow {
    pub win: Window,
    pub table: ViewId,
    pub contacts: Vec<Contact>,
    pub file: Option<PathBuf>,
    pub modified: bool,
}
```

`table` — **идентификатор** таблицы, а не ссылка на неё: таблицей владеет
группа окна, а найти её можно в любой момент через `find_mut`.

Конструктор строит содержимое окна:

```rust,ignore
pub fn new(bounds: Rect, contacts: Vec<Contact>, file: Option<PathBuf>) -> Self {
    let mut win = Window::new(bounds, "", 1);
    // Полосы прокрутки — на рамке окна.
    let v = win.standard_scroll_bar(true);
    let h = win.standard_scroll_bar(false);
    let model = VecTableModel::new(&["Имя", "Телефон", "Почта", "Группа", "★"], Vec::new());
    let cols = vec![
        TableColumn::new(22),
        TableColumn::new(18),
        TableColumn::new(22),
        TableColumn::new(8),
        TableColumn::new(3).align(Align::Center),
    ];
    let table = Table::new(win.interior(), Box::new(model), Some(v), Some(h))
        .with_columns(cols)
        .selection(SelectionMode::Row)
        .striped(true)
        .sortable(false);
    let table = win.add(table);
    let mut w = ContactsWindow { win, table, contacts, file, modified: false };
    w.reload();
    w
}
```

Что здесь происходит:

* `Window::new(границы, заголовок, номер)` — окно; номер 1 показывается в
  рамке, и окно выбирается по Alt+1.
* `standard_scroll_bar(true)` — вертикальная полоса прокрутки на правой
  рамке; возвращает id полосы. Таблица получает эти id и сама двигает
  полосы.
* `win.interior()` — внутренность окна без рамки: таблица занимает её
  целиком и растягивается вместе с окном.
* `Table` берёт строки из **модели** (`VecTableModel` — строки в памяти).
  Столбцы описывает `TableColumn`: ширина и выравнивание.
* `.sortable(false)` — щелчок по заголовку не сортирует. Мы будем
  сортировать сами (команда «Сортировать»), чтобы номер строки таблицы
  всегда совпадал с номером контакта в `contacts`.

Метод `reload` перекладывает контакты в таблицу:

```rust,ignore
pub fn reload(&mut self) {
    let rows: Vec<Vec<String>> = self.contacts.iter().map(|c| c.cells()).collect();
    let t = self.table_mut();
    let (row, _) = t.cursor();
    let cols = t.columns.clone();
    t.set_model(Box::new(VecTableModel::new(&["Имя", "Телефон", "Почта", "Группа", "★"], rows)));
    t.set_columns(cols); // `set_model` пересоздаёт столбцы — возвращаем свои
    t.set_cursor(row, 0);
    t.refresh();
    // …заголовок окна: имя файла и «*», если есть несохранённые изменения…
}
```

Теперь самое интересное — **реализация трейта `View`**. Окно построено
вокруг `Window`, поэтому почти всё делегируется ему; рутину берёт на себя
макрос `impl_group_basics!`:

```rust,ignore
impl View for ContactsWindow {
    tvision::impl_group_basics!(win.group);

    fn as_window(&self) -> Option<&Window> { Some(&self.win) }
    fn draw(&self, dc: &mut DrawCtx) { self.win.draw_window(dc); }
    fn palette(&self) -> &'static [u8] { self.win.palette }
    fn set_state(&mut self, flag: u16, enable: bool) { self.win.set_window_state(flag, enable); }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        let double = matches!(ev, Event::MouseDown(m) if m.double);
        self.win.handle_window_event(ev, ctx);
        // Таблица не поглощает Enter, и он возвращается к окну: превращаем
        // его (и двойной щелчок) в команду «Изменить».
        if matches!(ev, Event::KeyDown(k) if k.key == Key::Enter && k.mods == Mods::NONE) || double {
            ctx.post_command(CM_EDIT, Info::None);
            ev.clear();
        }
    }
}
```

* `as_window` сообщает программе, что это окно: так работают команды
  «Закрыть» (Alt-F3), «Распахнуть» (F5) и выбор по Alt+номер.
* `handle_event` сначала отдаёт событие окну (а оно — таблице). Если
  событие осталось необработанным (`ev` не очищен), окно решает, что с ним
  делать. `ctx.post_command` ставит команду в очередь — её получит
  обработчик приложения.
* `ev.clear()` помечает событие обработанным, чтобы оно не пошло дальше.

Это и есть «наследование» по-растовски: вместо `class ContactsWindow :
public TWindow` — структура с полем `win` и делегирование.

Окно вставляется на рабочий стол:

```rust,ignore
let r = app.desktop().group.vs.extent(); // весь рабочий стол
app.insert_window(ContactsWindow::new(r, contacts, file));
```

## Шаг 3. Диалог «Контакт»

Диалог — это окно с элементами управления, которое обычно выполняется
**модально**: пока оно открыто, остальная программа ждёт.

```rust,ignore
pub fn contact_dialog(title: &str, c: &Contact) -> Dialog {
    let mut d = Dialog::centered(52, 17, title);

    // Имя: не может быть пустым. Проверка — при нажатии OK.
    let name = InputLine::new(Rect::new(13, 2, 49, 3), 60)
        .with_validator(FnValidator { f: |s: &str| !s.trim().is_empty(), message: "Имя не может быть пустым.".into() });
    let name = d.add(name);
    d.add(Label::new(Rect::new(2, 2, 12, 3), "~И~мя:", Some(name)));

    // Телефон: только цифры и знаки + - ( ) и пробел — лишнее не наберётся.
    let phone = d.add(InputLine::new(Rect::new(13, 4, 49, 5), 30).with_validator(FilterValidator::new("0123456789+-() ")));
    d.add(Label::new(Rect::new(2, 4, 12, 5), "~Т~елефон:", Some(phone)));

    // Почта: пусто или с «@».
    let email = InputLine::new(Rect::new(13, 6, 49, 7), 60)
        .with_validator(FnValidator { f: |s: &str| s.trim().is_empty() || s.contains('@'), message: "В адресе почты нет «@».".into() });
    let email = d.add(email);
    d.add(Label::new(Rect::new(2, 6, 12, 7), "~П~очта:", Some(email)));

    let group = d.add(Cluster::radio_buttons(Rect::new(3, 9, 22, 12), &["~С~емья", "~Р~абота", "~Д~рузья"]));
    d.add(Label::new(Rect::new(2, 8, 12, 9), "~Г~руппа", Some(group)));

    d.add(Cluster::check_boxes(Rect::new(26, 9, 49, 10), &["Из~б~ранный"]));

    d.add(Button::new(Rect::new(12, 13, 24, 15), "O~K~", cm::OK, bf::DEFAULT));
    d.add(Button::new(Rect::new(27, 13, 39, 15), "Отмена", cm::CANCEL, bf::NORMAL));

    d.set_data(&c.to_value());
    d.select(name);
    d
}
```

Разберём по элементам:

* **Координаты** элементов отсчитываются от левого верхнего угла диалога.
  Строка 0 и столбец 0 — рамка. `Rect::new(13, 2, 49, 3)` — от столбца 13
  до 48 (правый край **не входит**), строка 2: поле высотой в одну строку.
* **`InputLine`** — строка ввода. Второй аргумент — наибольшая длина
  текста. Поле шире видимой части прокручивается.
* **Валидаторы** проверяют ввод. `FilterValidator` просто не даёт набрать
  лишние символы. `FnValidator` проверяет готовое значение, когда
  пользователь нажимает OK: если проверка не прошла, появляется сообщение,
  фокус возвращается в поле, а диалог остаётся открытым. Есть ещё маски
  (`PictureValidator`), диапазоны чисел и списки допустимых значений — см.
  [справочник](15-validators.md).
* **`Label`** — подпись, привязанная к полю: щелчок по ней или Alt+буква
  переводят фокус в поле, а сама подпись подсвечивается, когда поле в
  фокусе.
* **`Cluster`** — группа переключателей (`radio_buttons`) или флажков
  (`check_boxes`). Стрелки двигают выбор, пробел отмечает.
* **`Button`** с флагом `bf::DEFAULT` нажимается по Enter. Команды
  `cm::OK` и `cm::CANCEL` завершают модальный диалог; Esc — тоже
  `cm::CANCEL`.
* **Порядок `add`** — это порядок перехода по Tab и порядок значений в
  данных диалога.

### Обмен данными

Диалог заполняется и читается **одним значением** `Value` — списком
значений всех элементов, у которых есть данные, в порядке вставки. У
строки ввода значение — `Value::Str`, у переключателей — номер
выбранного (`Value::U32`), у флажков — битовая маска (`Value::U32`).
Подписи и кнопки данных не имеют и в список не входят.

```rust,ignore
fn to_value(&self) -> Value {
    Value::List(vec![
        Value::Str(self.name.clone()),
        Value::Str(self.phone.clone()),
        Value::Str(self.email.clone()),
        Value::U32(self.group),               // переключатели: номер выбранного
        Value::U32(self.favorite as u32),     // флажки: битовая маска
    ])
}

fn from_value(v: &Value) -> Self {
    let l = v.as_list().unwrap_or(&[]);
    let s = |i: usize| l.get(i).and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
    let n = |i: usize| l.get(i).and_then(|x| x.as_u32()).unwrap_or(0);
    Contact { name: s(0), phone: s(1), email: s(2), group: n(3), favorite: n(4) & 1 != 0 }
}
```

Это прямой аналог записи `struct TDialogData` из C++: там данные
копировались в память побайтно, здесь — типизированным списком.

### Запуск диалога

```rust,ignore
fn edit_contact(app: &mut Program, title: &str, c: &Contact) -> Option<Contact> {
    let (cmd, d) = app.exec_dialog(contact_dialog(title, c));
    (cmd == cm::OK).then(|| Contact::from_value(&d.get_data()))
}
```

`exec_dialog` выполняет диалог модально и **возвращает его обратно** вместе
с командой, которой он закрылся. Поэтому прочитать данные можно уже после
закрытия — диалог никуда не делся.

## Шаг 4. Команды: добавить, изменить, удалить

Все команды, которые не обработали окна, приходят в `AppHandler::handle_event`:

```rust,ignore
impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        let Some(cmd) = ev.command() else { return };
        match cmd {
            CM_ADD => self.add(app),
            CM_EDIT => self.edit(app),
            CM_DELETE => self.delete(app),
            CM_SORT => self.sort(app),
            // …команды файлов (шаг 5)…
            CM_EXIT => {
                if self.confirm_save(app) {
                    app.quit();
                }
            }
            _ => return,
        }
        ev.clear();
    }
}
```

Найти окно контактов на рабочем столе помогает приведение типов:
представления хранятся как `Box<dyn View>`, а `as_any_mut().downcast_mut`
возвращает конкретный тип:

```rust,ignore
fn window(app: &mut Program) -> Option<&mut ContactsWindow> {
    let desk = &mut app.desktop_mut().group;
    desk.children.iter_mut().find_map(|w| w.as_any_mut().downcast_mut::<ContactsWindow>())
}
```

Правка контакта:

```rust,ignore
fn edit(&mut self, app: &mut Program) {
    // Данные копируем до диалога: пока он открыт, окно «взаймы» не держим.
    let Some((i, c)) = Self::window(app).and_then(|w| w.current().map(|i| (i, w.contacts[i].clone()))) else { return };
    if let Some(new) = edit_contact(app, "Контакт", &c) {
        let w = Self::window(app).unwrap();
        if w.contacts[i] != new {
            w.contacts[i] = new;
            w.modified = true;
            w.reload();
        }
    }
}
```

Обратите внимание на порядок: сначала **копируем** нужные данные из
окна, потом показываем диалог, потом **снова** находим окно. Так требует
проверка заимствований Rust: `exec_dialog` берёт `&mut app` целиком, и
держать ссылку на окно в это время нельзя. Этот приём встречается во всех
примерах библиотеки.

Удаление спрашивает подтверждение стандартным окном сообщения:

```rust,ignore
let q = format!("\x03Удалить «{}»?", name);
if message_box(app, &q, mf::CONFIRMATION | mf::YES_BUTTON | mf::NO_BUTTON) == cm::YES {
    // …удалить…
}
```

Символ `\x03` в начале строки центрирует текст (как в оригинале).

### Запрет недоступных команд

Редактировать нечего, если список пуст. Запрещённая команда рисуется
серым в меню и строке состояния, а её клавиша не срабатывает:

```rust,ignore
fn idle(&mut self, app: &mut Program) {
    let (has_window, has_rows) = match Self::window(app) {
        Some(w) => (true, !w.contacts.is_empty()),
        None => (false, false),
    };
    for (cmd, on) in [(CM_ADD, has_window), (CM_EDIT, has_rows), (CM_DELETE, has_rows) /* … */] {
        if on { app.enable_command(cmd); } else { app.disable_command(cmd); }
    }
}
```

`AppHandler::idle` вызывается, когда событий нет. Другой способ — метод
`View::update_commands(&self, ctx)` у представления: программа вызывает
его после каждого события, и так команды могут зависеть от того, какое
окно в фокусе (так сделан редактор).

## Шаг 5. Файлы

Чтение и запись — обычный Rust. Интерфейсная часть — стандартный диалог
выбора файла:

```rust,ignore
let d = FileDialog::new("*.tsv", "Открыть", "~И~мя", fd::OPEN_BUTTON, 100);
let (cmd, d) = app.exec_dialog(d);
if cmd == cm::CANCEL {
    return;
}
let path = d.file_name();
```

`FileDialog::new(шаблон, заголовок, подпись поля, кнопки, номер истории)`.
Последний аргумент — номер списка истории: диалог запоминает введённые
имена. Для «Сохранить как» используется кнопка `fd::OK_BUTTON`.

Перед открытием другого файла и при выходе программа спрашивает, сохранить
ли изменения:

```rust,ignore
fn confirm_save(&mut self, app: &mut Program) -> bool {
    if !Self::window(app).map(|w| w.modified).unwrap_or(false) {
        return true;
    }
    match message_box(app, "\x03Сохранить изменения?", mf::CONFIRMATION | mf::YES_NO_CANCEL) {
        cm::YES => self.save(app, false),
        cm::NO => true,
        _ => false,
    }
}
```

Выход сделан своей командой `CM_EXIT`, а не `cm::QUIT`, чтобы спросить до
завершения. Есть и способ «как в оригинале»: `cm::QUIT` вызывает у всех
окон `View::valid(cm::QUIT, ctx)`, и окно может отказаться закрываться.
Но показать вопрос изнутри `valid` можно только асинхронно, через
`ctx.exec` и `View::exec_result` — так устроен `FileEditor`. Для
приложения своя команда проще.

## Шаг 6. Тесты

Интерфейс проверяется обычными тестами `cargo test` — без настоящего
терминала. `HeadlessBackend` — «невидимый» терминал: события берутся из
очереди, кадр хранится в памяти.

```rust,ignore
fn program(input: Vec<RawEvent>) -> Program {
    let mut b = HeadlessBackend::new(100, 30);
    b.input.extend(input);
    let mut app = Program::new(Box::new(b), Some(menu()), Some(status()));
    App::open_window(&mut app, sample_contacts(), None);
    app
}

#[test]
fn add_contact_through_dialog() {
    // Ins → диалог; имя, Tab, телефон (буквы фильтр не пропустит), Enter — OK.
    let mut input = vec![key(Key::Ins)];
    input.extend(text("Дарья"));
    input.push(key(Key::Tab));
    input.extend(text("+7 (999) abc 12"));
    input.push(key(Key::Enter));
    let mut app = program(input);
    app.pump(&mut App);
    let w = App::window(&mut app).unwrap();
    assert_eq!(w.contacts.len(), 5);
    let c = w.contacts.last().unwrap();
    assert_eq!((c.name.as_str(), c.phone.as_str()), ("Дарья", "+7 (999)  12"));
}
```

* `app.pump(&mut App)` обрабатывает все события очереди, включая
  вложенные модальные диалоги.
* Когда очередь кончается, тестовый терминал сам «нажимает» Esc, чтобы
  забытое модальное окно не подвесило тест.
* `app.screen_text()` — последний кадр текстом, его можно проверять через
  `contains`.

Больше приёмов — в главе [«Тестирование»](../14-testing.md).

## Что дальше

* Добавьте поле «День рождения» с маской `PictureValidator::new("##.##.####", true)`.
* Сделайте поиск: строка ввода над таблицей и фильтрация строк
  (подсказка: `examples/three_panes.rs` — окно из нескольких частей).
* Покажите группы деревом слева (`TreeView`, [справочник](17-data-views.md#treeview)).
* Добавьте справку по F1 (`HelpFile`, `HelpWindow`, [справочник](19-std-dialogs.md#helpfile)).

# Справочник: геометрия, цвета, буферы, текст

Базовые типы, на которых стоит всё остальное. Подключаются вместе с
`use tvision::*;`; функции цвета — из модуля `tvision::color`, текста — из
`tvision::text`.

* [`Point`](#point) · [`Rect`](#rect)
* [`Attr`, `AttrPair` и функции цвета](#attr-и-attrpair) · [`ColorMode`](#colormode) · [`palettes`](#палитры-colorpalettes)
* [`Cell`](#cell) · [`DrawBuffer`](#drawbuffer) · [`ScreenBuffer`](#screenbuffer)
* [Функции текста](#функции-текста-text)

---

## `Point`

**Точка или размер** в ячейках экрана: `x` — столбец, `y` — строка (ось
Y направлена вниз). Аналог `TPoint`. Модуль `geometry`.

```rust
use tvision::*;

fn points() {
    let a = Point::new(10, 5);
    let b = Point::new(2, 1);
    assert_eq!(a + b, Point::new(12, 6));
    assert_eq!(a - b, Point::new(8, 4));
    let mut c = a;
    c += b; // есть и `-=`
    assert_eq!(c.x, 12);
}
```

| Поле / метод | Что |
|--------------|-----|
| `x`, `y` (`i32`) | координаты |
| `Point::new(x, y)` | создать (`const fn`) |
| `+`, `-`, `+=`, `-=` | сложение и вычитание по координатам |

`Point` используется и как **размер** (`ViewState::size`, `Rect::size()`):
тогда `x` — ширина, `y` — высота.

## `Rect`

**Прямоугольник**: `a` — левый верхний угол (включительно), `b` — правый
нижний (**не включительно**). Ширина — `b.x - a.x`. Аналог `TRect`.

```rust
use tvision::*;

fn rects() {
    let r = Rect::new(2, 1, 12, 4); // столбцы 2..11, строки 1..3
    assert_eq!((r.width(), r.height()), (10, 3));
    assert!(r.contains(Point::new(11, 3)));
    assert!(!r.contains(Point::new(12, 3))); // правая граница не входит

    let inner = r.grown(-1, -1); // сжать на 1 с каждой стороны: (3,2)-(11,3)
    let moved = r.moved(5, 0); // (7,1)-(17,4)
    assert_eq!(r.intersect(&moved), Rect::new(7, 1, 12, 4));
    assert_eq!(r.union(&moved), Rect::new(2, 1, 17, 4));
    assert!(!inner.is_empty());

    let full = Rect::from_origin_size(Point::new(0, 0), Point::new(80, 25));
    assert_eq!(full.size(), Point::new(80, 25));
}
```

| Метод | Что делает |
|-------|-----------|
| `Rect::new(ax, ay, bx, by)` | по углам |
| `Rect::from_origin_size(origin, size)` | по началу и размеру |
| `a`, `b` | углы (поля) |
| `width()`, `height()`, `size()` | размеры |
| `moved(dx, dy)` | сдвинутая копия (`TRect::move`) |
| `grown(dx, dy)` | расширенная на `dx`/`dy` с каждой стороны (отрицательные — сжать) |
| `intersect(&r)`, `union(&r)` | пересечение, охватывающий прямоугольник |
| `contains(p)` | лежит ли точка внутри |
| `is_empty()` | нулевая или отрицательная площадь |

`moved` и `grown` принимают `self` по значению и возвращают новый
прямоугольник — их удобно сцеплять: `r.grown(-1, -1).moved(0, 1)`.

---

## `Attr` и `AttrPair`

`Attr` (`u32`) — **цвет ячейки**. Младший байт — как в BIOS: младшие 4
бита — цвет текста, старшие 4 — фон (`0x1E` — жёлтый на синем). Выше
лежат необязательные номера цветов xterm-256 и флаги `FG_256` / `BG_256`.

`AttrPair` (`u64`) — **пара** атрибутов для текста с горячей буквой:
обычный цвет и цвет букв между тильдами.

| Номер | Цвет BIOS | Номер | Цвет BIOS |
|-------|-----------|-------|-----------|
| 0 | чёрный | 8 | тёмно-серый |
| 1 | синий | 9 | ярко-синий |
| 2 | зелёный | 10 | ярко-зелёный |
| 3 | бирюзовый | 11 | ярко-бирюзовый |
| 4 | красный | 12 | ярко-красный |
| 5 | малиновый | 13 | ярко-малиновый |
| 6 | коричневый | 14 | жёлтый |
| 7 | светло-серый | 15 | белый |

```rust
use tvision::color::*;

fn colors() {
    // 16 цветов BIOS.
    let a = attr(14, 1);
    assert_eq!(a, 0x1E);
    assert_eq!((fore(a), back(a)), (14, 1));
    assert_eq!(with_back(a, 4), 0x4E); // тот же текст на красном

    // 256 цветов xterm.
    let b = attr256(208, 236); // оранжевый на тёмно-сером
    assert!(is_fg256(b) && is_bg256(b));
    assert_eq!((fg256(b), bg256(b)), (208, 236));
    let c = rgb256(0xFFD700, 0x303030); // ближайшие к RGB
    assert_eq!(fg256(c), 220);
    assert_eq!(to_bios(c) & !0xFF, 0); // свести к 16 цветам

    // Пара для C-строк.
    let p = pair(0x70, 0x74);
    assert_eq!((lo(p), hi(p)), (0x70, 0x74));
}
```

### Функции цвета

| Функция | Что делает |
|---------|-----------|
| `attr(fg, bg)` | атрибут BIOS из цветов 0–15 |
| `attr256(fg, bg)` | атрибут из номеров xterm 0–255 |
| `rgb256(fg_rgb, bg_rgb)` | из цветов `0xRRGGBB` (ближайшие xterm) |
| `with_fore(a, c)`, `with_back(a, c)` | заменить текст / фон цветом BIOS |
| `with_fg256(a, n)`, `with_bg256(a, n)` | заменить текст / фон номером xterm |
| `fore(a)`, `back(a)` | цвет BIOS текста / фона (для xterm — ближайший) |
| `fg256(a)`, `bg256(a)` | номер xterm (у цвета BIOS — 0–15 в порядке ANSI) |
| `is_fg256(a)`, `is_bg256(a)` | задан ли цвет номером xterm |
| `bios(a)` | байт BIOS — то, что покажет 16-цветный терминал |
| `to_bios(a)` | атрибут без цветов xterm |
| `xterm_rgb(n)`, `rgb_to_xterm(rgb)`, `xterm_to_bios(n)` | переводы между xterm, RGB и BIOS |
| `bios_to_ansi(c)`, `ansi_to_bios(c)` | порядок цветов BIOS ↔ ANSI (у BIOS 1 — синий, у ANSI — красный) |
| `fg_rgb(a, &base)`, `bg_rgb(a, &base)` | цвет в RGB; `base` — RGB 16 цветов BIOS (`VGA_RGB` или `GRUVBOX_RGB`) |
| `contrast256(n)` | чёрный (16) или белый (231) — читаемый на фоне `n` |
| `shade_bg(a)` | чуть другой фон (для «зебры» таблиц) |
| `remap(palette, &map)` | перекрасить палитру: BIOS `i` → xterm `map[i]` |
| `pair(normal, hi)`, `unpair(p)`, `lo(p)`, `hi(p)` | собрать / разобрать пару |

| Константа | Что |
|-----------|-----|
| `FG_256`, `BG_256` | флаги «цвет задан номером xterm» (биты 24, 25) |
| `ERROR_ATTR` | `0xCF` — ошибка палитры (номер цвета вне палитры) |
| `SHADOW_ATTR`, `SHADOW_ATTR_256` | цвет тени окон |
| `VGA_RGB`, `GRUVBOX_RGB` | RGB 16 цветов BIOS: классика VGA и тема gruvbox |

Подробно о том, как атрибут хранит цвета, — [глава 11](../11-colors.md).

## `ColorMode`

**Сколько цветов выводить в терминал.** `ColorMode::Ansi16` — 16 цветов
(цвета xterm заменяются ближайшими BIOS); `ColorMode::Xterm256` —
256 цветов.

```rust
use tvision::color::{color_mode, set_color_mode, ColorMode};

fn force_16() {
    set_color_mode(Some(ColorMode::Ansi16)); // None — снова определять по терминалу
    assert_eq!(color_mode(), ColorMode::Ansi16);
    set_color_mode(None);
}
```

| Функция | Что |
|---------|-----|
| `color_mode()` | текущий режим: заданный или определённый |
| `set_color_mode(Some(m) / None)` | задать для всей программы / вернуть автоопределение |
| `detect_color_mode()` | определить по `COLORTERM`, `TERM`, Windows Terminal… |
| `color_mode_from(forced, colorterm, term, wt, term_program)` | то же без чтения окружения (для тестов) |

Переменная окружения `TVISION_COLORS=16` или `256` задаёт режим без
изменения программы.

## Палитры (`color::palettes`)

**Палитра** — цепочка номеров цветов. У каждого представления своя
палитра: номер цвета ищется в ней, результат — в палитре владельца и так
до **палитры приложения**, которая даёт настоящий атрибут. Так одна и та
же кнопка выглядит по-разному в сером и синем диалоге.

| Константа | Что |
|-----------|-----|
| `GRUVBOX` | палитра приложения по умолчанию (256 цветов, тема gruvbox) |
| `APP_COLOR`, `APP_BLACK_WHITE`, `APP_MONOCHROME` | классические палитры приложения (`cpAppColor`…) |
| `APP_COLOR_BYTES`… | они же байтами, как в C++ |
| `gruvbox::*` | номера цветов gruvbox в палитре xterm: `BG0`, `FG1`, `RED`, `AQUA`… |
| `BLUE_WINDOW`, `CYAN_WINDOW`, `GRAY_WINDOW` | палитры окон |
| `GRAY_DIALOG`, `BLUE_DIALOG`, `CYAN_DIALOG` | палитры диалогов |
| `BUTTON`, `CLUSTER`, `INPUT_LINE`, `LABEL`, `STATIC_TEXT`, `LIST_VIEWER`, `SCROLL_BAR`, `FRAME`, `MENU_VIEW`, `STATUS_LINE`, `BACKGROUND`, `EDITOR`, `MEMO`, `HELP_VIEWER`, … | палитры элементов (`cpButton`…) |

```rust
use tvision::*;

fn classic(app: &mut Program) {
    // Классические цвета Borland вместо gruvbox.
    app.palette = color::palettes::APP_COLOR.to_vec();
}
```

Палитра приложения — обычный `Vec<Attr>` в `Program::palette`: её можно
менять на ходу (это делает `ColorDialog`) и хранить в файле. Подробно —
[глава 11](../11-colors.md).

---

## `Cell`

**Ячейка экрана**: символ и атрибут.

| Поле / метод | Что |
|--------------|-----|
| `ch: char` | символ; у второй половины широкого символа — `'\0'` |
| `attr: Attr` | цвет |
| `Cell::new(ch, attr)` | создать |
| `is_wide_tail()` | вторая половина иероглифа или эмодзи |

## `DrawBuffer`

**Строка ячеек** для рисования: заполняется методами `move_*` и выводится
`DrawCtx::write_line`. Аналог `TDrawBuffer`. Широкие символы (занимающие
две ячейки) учитываются сами.

```rust
use tvision::*;

/// Строка «[ OK ]  Сохранить» с жёлтой буквой «С».
fn line(width: usize) -> DrawBuffer {
    let mut b = DrawBuffer::new(width); // пробелы
    b.move_char(0, ' ', 0x70, width); // залить серым
    let x = b.move_str(0, "[ OK ]", 0x2F); // зелёный; вернёт ширину = 6
    b.move_cstr(x + 2, "~С~охранить", color::pair(0x70, 0x7E)); // текст между ~ — цветом подсветки
    b.put_attribute(0, 0x4F); // перекрасить одну ячейку
    b.put_char(1, '*'); // заменить символ, не трогая цвет
    b
}
```

| Метод | Что делает |
|-------|-----------|
| `DrawBuffer::new(w)` | `w` пробелов |
| `move_char(x, ch, attr, count)` | повторить символ (`moveChar`) |
| `move_str(x, s, attr) -> ширина` | строка (`moveStr`) |
| `move_str_limited(x, s, attr, max) -> ширина` | не шире `max` ячеек |
| `move_cstr(x, s, pair) -> ширина` | C-строка: текст между `~` — цветом подсветки (`moveCStr`) |
| `move_buf(x, &cells, attr)` | готовые ячейки; `attr ≠ 0` — перекрасить |
| `put_char(x, ch)`, `put_attribute(x, attr)` | символ без цвета / цвет без символа |
| `len()`, `is_empty()`, `cells` | размер и сами ячейки |

> Атрибут `0` в `move_str`, `move_char` и `move_cstr` означает «не менять
> цвет ячейки»: так можно написать текст поверх уже закрашенного фона.

## `ScreenBuffer`

**Весь экран** — прямоугольный массив ячеек. Программа рисует кадр в него,
а терминал выводит только изменившиеся ячейки. В своём коде он нужен в
основном в тестах.

```rust
use tvision::*;

fn check_frame(app: &Program) {
    let s: &ScreenBuffer = app.screen();
    let top = s.line_text(0); // строка меню текстом
    let cell = s.get(2, 0); // ячейка (x, y)
    let _ = (top, cell.attr, s.to_text(), s.width, s.height);
}
```

| Метод | Что |
|-------|-----|
| `ScreenBuffer::new(w, h)` | пустой экран |
| `width`, `height`, `cells` | размер и ячейки (построчно) |
| `get(x, y)`, `get_mut(x, y)` | ячейка (за пределами — паника) |
| `clear(attr)` | залить пробелами |
| `line_text(y)`, `to_text()` | строка / весь экран текстом (для тестов) |

---

## Функции текста (`text`)

Ширина строк на экране и «C-строки» — тексты с горячей буквой между
тильдами (`"~F~ile"`).

```rust
use tvision::text::*;

fn texts() {
    assert_eq!(char_width('Ж'), 1);
    assert_eq!(char_width('漢'), 2); // иероглиф — две ячейки
    assert_eq!(str_width("Привет"), 6);
    assert_eq!(cstr_width("~Ф~айл"), 4); // без тильд
    assert_eq!(strip_tildes("~Ф~айл"), "Файл");
    assert_eq!(hotkey("~Ф~айл"), Some('Ф'));
    assert!(same_hotkey('a', 'Ф')); // Alt+A в латинской раскладке = Alt+Ф
    assert_eq!(truncate_width("Привет, мир", 6), "Привет");
    assert_eq!(wrap_text("один два три", 8), vec!["один два", "три"]);
    assert_eq!(to_upper('ё'), 'Ё');
}
```

| Функция | Что делает |
|---------|-----------|
| `char_width(c)` | ширина символа: 0 (комбинирующие, управляющие), 1 или 2 |
| `str_width(s)` | ширина строки (`strwidth`) |
| `cstr_width(s)` | ширина C-строки без тильд (`cstrlen`) |
| `strip_tildes(s)` | убрать тильды |
| `hotkey(s)` | горячая буква в верхнем регистре |
| `same_hotkey(нажата, горячая)` | сравнить с учётом регистра и раскладки (й↔q, ф↔a…) |
| `to_upper(c)` | верхний регистр, в том числе кириллицы |
| `truncate_width(s, max)` | начало строки не шире `max` ячеек |
| `wrap_text(s, w)` | перенос по словам (как у `StaticText`) |

# Справочник: события, клавиши, команды

Всё, что приходит в программу извне (клавиши, мышь) и что части
программы посылают друг другу (команды, сообщения). Модуль `event`;
всё подключается через `use tvision::*;`.

* [`Key`](#key) · [`Mods`](#mods) · [`KeyEvent`](#keyevent) · [`kb`](#kb--константы-клавиш)
* [`MouseEvent`](#mouseevent)
* [`Event`](#event) · [`Message`](#message) · [`Info`](#info)
* [`cm` — команды](#cm--команды)
* [Как событие находит получателя](#как-событие-находит-получателя)

---

## `Key`

**Клавиша** без модификаторов. Вместо скан-кодов DOS (`kbF1 = 0x3B00`) —
перечисление:

| Вариант | Клавиша |
|---------|---------|
| `Char(c)` | печатный символ: буква (в той раскладке, что включена), цифра, знак, пробел |
| `Enter`, `Esc`, `Tab`, `BackTab`, `Backspace` | `BackTab` — Shift+Tab, если терминал присылает его отдельно |
| `Up`, `Down`, `Left`, `Right` | стрелки |
| `Home`, `End`, `PgUp`, `PgDn`, `Ins`, `Del` | |
| `F(n)` | F1–F12 |

## `Mods`

**Модификаторы**: поля `shift`, `ctrl`, `alt` (`bool`). Готовые значения:
`Mods::NONE`, `SHIFT`, `CTRL`, `ALT`, `CTRL_SHIFT`.

## `KeyEvent`

**Нажатие клавиши**: `key` и `mods`. Аналог `KeyDownEvent`.

```rust
use tvision::*;

fn keys() {
    let k = KeyEvent::new(Key::Char('x'), Mods::ALT);
    assert!(k.is(kb::ALT_X)); // буквы сравниваются без учёта регистра
    assert_eq!(k.alt_char(), Some('X'));
    assert_eq!(k.text_char(), None); // с Alt — не печатный символ
    assert_eq!(k.name(), "Alt+X");

    let t = KeyEvent::new(Key::Char('ж'), Mods::NONE);
    assert_eq!(t.text_char(), Some('ж'));
    assert_eq!(kb::ctrl_f(5).name(), "Ctrl+F5");
}
```

| Метод | Что |
|-------|-----|
| `KeyEvent::new(key, mods)` | создать (`const fn`) |
| `is(other)` | то же ли сочетание (буквы — без учёта регистра; Shift+Tab = BackTab) |
| `text_char()` | символ для вставки в текст: `Char` без Ctrl и Alt |
| `alt_char()` | буква Alt+буква в верхнем регистре |
| `name()` | `"Alt+X"`, `"Ctrl+F5"`, `"F10"` — для подписей |

## `kb` — константы клавиш

Конструкторы `const fn` и частые сочетания (аналог `kbXXX` из `tkeys.h`):

| Имя | Что |
|-----|-----|
| `kb::key(Key::Ins)` | клавиша без модификаторов |
| `kb::alt('X')`, `kb::ctrl('N')` | Alt+буква, Ctrl+буква |
| `kb::f(n)`, `kb::alt_f(n)`, `kb::ctrl_f(n)`, `kb::shift_f(n)` | функциональные клавиши |
| `kb::ENTER`, `ESC`, `TAB`, `SHIFT_TAB` | |
| `kb::F1`, `F2`, `F3`, `F5`, `F6`, `F10` | |
| `kb::ALT_X`, `ALT_F3`, `CTRL_F5`, `SHIFT_F6` | стандартные клавиши Turbo Vision |

> Ctrl+буква в терминале — управляющий код, и некоторые из них совпадают
> с другими клавишами: Ctrl+J — это Enter, Ctrl+I — Tab, Ctrl+H —
> Backspace, Ctrl+M — Enter. Не назначайте на них команды.

---

## `MouseEvent`

**Состояние мыши** в событии. Аналог `MouseEventType`.

| Поле | Что |
|------|-----|
| `pos: Point` | позиция **в координатах владельца** получателя (как `vs.origin`); в свои координаты — `self.vs.make_local(m.pos)` |
| `buttons: u8` | нажатые кнопки: `event::MB_LEFT` (1), `MB_RIGHT` (2), `MB_MIDDLE` (4) — константы в модуле `tvision::event` |
| `double: bool` | двойной щелчок (интервал — `Program::double_delay`) |
| `mods: Mods` | Shift / Ctrl / Alt в момент события |
| `wheel: i8` | колесо: −1 вверх, +1 вниз |

```rust
use tvision::event::MB_LEFT;
use tvision::*;

/// Куда щёлкнули внутри своего представления.
fn local_click(vs: &ViewState, ev: &Event) -> Option<Point> {
    match ev {
        Event::MouseDown(m) if m.buttons & MB_LEFT != 0 => Some(vs.make_local(m.pos)),
        _ => None,
    }
}
```

---

## `Event`

**Событие** — то, что получает `View::handle_event`. Аналог `TEvent`.

| Вариант | Когда | Кому доставляется |
|---------|-------|-------------------|
| `KeyDown(KeyEvent)` | нажата клавиша | по цепочке фокуса |
| `MouseDown(m)`, `MouseUp(m)` | кнопка нажата / отпущена | представлению под курсором (или захватившему мышь) |
| `MouseMove(m)`, `MouseAuto(m)` | движение; кнопка удерживается (повтор) | тоже |
| `MouseWheel(m)` | колесо | представлению под курсором |
| `Command(Message)` | команда (меню, кнопка, `post_command`) | по цепочке фокуса, затем `AppHandler` |
| `Broadcast(Message)` | сообщение всем | всем детям группы, пока кто-то не обработает |
| `Nothing` | событие обработано | никому |

```rust
use tvision::*;

/// Типичная обработка в `handle_event`.
fn handle(ev: &mut Event) -> Option<u16> {
    if let Some(k) = ev.key() {
        if k.is(kb::F2) {
            ev.clear(); // обработано — дальше не пойдёт
            return Some(1);
        }
    }
    match ev.command() {
        Some(cm::OK) => {
            ev.clear();
            Some(2)
        }
        _ => None,
    }
}

fn check() {
    let mut e = Event::KeyDown(kb::F2);
    assert_eq!(handle(&mut e), Some(1));
    assert!(e.is_nothing());
    let mut c = Event::command_event(cm::OK);
    assert_eq!(handle(&mut c), Some(2));
}
```

| Метод | Что |
|-------|-----|
| `clear()` | пометить обработанным (`clearEvent`) — превращает событие в `Nothing` |
| `is_nothing()` | обработано ли |
| `key()` | `Some(KeyEvent)` для `KeyDown` |
| `command()` | номер команды для `Command` |
| `broadcast()` | `Some(&Message)` для `Broadcast` |
| `mouse()`, `mouse_mut()` | данные мыши любого события мыши |
| `is_mouse()`, `is_focused()` | позиционное (мышь) или фокусное (клавиши, команды) |
| `Event::command_event(cmd)` | создать команду без данных |

## `Message`

**Команда или сообщение с данными**: `command: u16` и `info: Info`.
`Message::new(cmd)`, `Message::with(cmd, info)`.

## `Info`

**Данные команды или сообщения** — вместо `void *infoPtr` из C++:

| Вариант | Где встречается |
|---------|-----------------|
| `None` | большинство команд |
| `Id(ViewId)` | кто послал: `cm::LIST_ITEM_SELECTED`, `TABLE_CURSOR_MOVED`, `RECEIVED_FOCUS`… |
| `Int(i64)` | номер окна (`SELECT_WINDOW_NUM`), свои числа |
| `Str(String)` | свои строки |
| `Text { id, text }` | текст для представления (выбор из истории) |
| `Indicator { id, line, col, modified }` | позиция редактора для индикатора |
| `Progress { id, value }` | новое значение `ProgressBar` (`cm::PROGRESS_SET`) |
| `Scroll { id, value, min, max, page, arrow }` | параметры полосы прокрутки |

```rust
use tvision::*;

/// Узнать, какой список выбрали.
fn who_selected(ev: &Event) -> Option<ViewId> {
    match ev.broadcast() {
        Some(Message { command: cm::LIST_ITEM_SELECTED, info: Info::Id(id) }) => Some(*id),
        _ => None,
    }
}
```

---

## `cm` — команды

Номера стандартных команд совпадают с оригиналом. Полный список с
номерами — [глава 16](../16-reference.md#cm--команды); основные:

| Группа | Команды |
|--------|---------|
| программа и окна | `QUIT`, `MENU`, `CLOSE`, `ZOOM`, `RESIZE`, `NEXT`, `PREV`, `TILE`, `CASCADE`, `HELP`, `DOS_SHELL` |
| диалоги | `OK`, `CANCEL`, `YES`, `NO`, `DEFAULT`, `VALID` |
| правка | `CUT`, `COPY`, `PASTE`, `UNDO`, `REDO`, `CLEAR`, `FIND`, `REPLACE`, `SEARCH_AGAIN`, `SELECT_ALL` |
| файлы | `NEW`, `OPEN`, `SAVE`, `SAVE_AS`, `SAVE_ALL`, `CH_DIR`, `CLOSE_ALL` |
| сообщения | `RECEIVED_FOCUS`, `RELEASED_FOCUS`, `COMMAND_SET_CHANGED`, `SCROLL_BAR_CHANGED`, `SCROLL_BAR_CLICKED`, `SELECT_WINDOW_NUM`, `LIST_ITEM_SELECTED`, `TABLE_CURSOR_MOVED`, `TABLE_CELL_EDITED`, `TREE_CURSOR_MOVED`, `TREE_NODE_TOGGLED`, `TREE_CHECK_CHANGED`, `FILE_PANEL_DIR_CHANGED` |

Свои команды нумеруйте от **100 до 255**, если их нужно запрещать (номера
выше 255 запретить нельзя — как в оригинале), или от 1000, если не нужно.
Номера `0xFF00` и выше заняты внутренними сообщениями библиотеки.

---

## Как событие находит получателя

1. **Клавиши и команды** (фокусные события) идут от программы вниз по
   цепочке **выбранных** представлений: программа → рабочий стол →
   активное окно → элемент в фокусе. Строка состояния и строка меню
   видят клавиши раньше (`of::PRE_PROCESS`), кнопки по умолчанию — позже
   (`of::POST_PROCESS`).
2. **Мышь** достаётся самому верхнему представлению под курсором (или
   тому, кто захватил мышь — `ctx.grab_mouse(id)`).
3. **Сообщения** (`Broadcast`) получает каждый ребёнок группы по очереди,
   пока кто-то не очистит событие. `ctx.message(cmd, info)` рассылает
   сообщение **соседям** того, кто его послал, — детям его владельца.
4. Если событие никто не обработал, **команда** попадает в
   `AppHandler::handle_event`.

Представление решает, обработало ли оно событие: вызывает `ev.clear()`.
Необработанное событие (например, Enter в таблице) возвращается к
владельцу, и тот может обработать его сам — так окно урока превращает
Enter в команду «Изменить».

Подробно — [глава 3](../03-events.md).

# Справочник: представления

Представление (view) — всё, что видно на экране. Здесь — трейт `View` и
то, что ему нужно: общие поля, флаги, данные, контекст событий, контекст
отрисовки и группа. Модули `view` и `group`.

* [`View`](#view) — трейт представления
* [`ViewState`](#viewstate) · [`ViewId`](#viewid) · [флаги `sf`, `of`, `gf`, `dm`](#флаги)
* [`Value`](#value) — данные для диалогов
* [`Ctx`](#ctx) · [`Request`, `PlaceFn`](#request-и-placefn) · [`CommandSet`](#commandset) · [`Phase`](#phase)
* [`DrawCtx`](#drawctx) — рисование
* [`Group`](#group) — представление с детьми
* [Макросы `impl_*`](#макросы) · [`default_handle_event`, `calc_bounds`](#вспомогательные-функции)

---

## `View`

**Трейт представления** — аналог класса `TView`. Любой видимый объект
реализует его: кнопка, окно, рабочий стол и ваши собственные элементы.

Обязательны два метода — `draw` и пять «технических» (`vs`, `vs_mut`,
`as_any`, `as_any_mut`, `into_any`), которые пишет макрос
[`impl_view_basics!`](#макросы). Остальные имеют разумное поведение по
умолчанию.

### Пример: своё представление «Оценка»

Звёзды `★★★☆☆`: ←/→ и щелчок меняют оценку, значение участвует в обмене
данными диалога.

```rust
use tvision::*;

pub struct Rating {
    pub vs: ViewState,
    pub value: u32, // 0..=5
}

impl Rating {
    pub fn new(x: i32, y: i32) -> Self {
        let mut vs = ViewState::new(Rect::new(x, y, x + 5, y + 1));
        vs.options |= of::SELECTABLE | of::FIRST_CLICK; // получает фокус; первый щелчок не теряется
        Rating { vs, value: 3 }
    }
}

impl View for Rating {
    tvision::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        // Номера цветов — в своей палитре (здесь — как у строки ввода):
        // 1 — обычный, 2 — в фокусе.
        let a = dc.color(if self.vs.is_focused() { 2 } else { 1 });
        let s: String = (0..5).map(|i| if i < self.value { '★' } else { '☆' }).collect();
        dc.write_str(0, 0, &s, a); // представление закрашивает всю свою область
    }

    fn palette(&self) -> &'static [u8] {
        color::palettes::INPUT_LINE
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        view::default_handle_event(&self.vs, ev, ctx); // щелчок по невыбранному — выбрать
        match ev {
            Event::MouseDown(m) => {
                let p = self.vs.make_local(m.pos); // координаты мыши — владельца
                self.value = (p.x + 1).clamp(0, 5) as u32;
                ev.clear();
            }
            Event::KeyDown(k) if self.vs.is_focused() => {
                match k.key {
                    Key::Left => self.value = self.value.saturating_sub(1),
                    Key::Right => self.value = (self.value + 1).min(5),
                    _ => return, // не наше — пусть обработает кто-то другой
                }
                ev.clear();
            }
            _ => {}
        }
    }

    fn has_data(&self) -> bool {
        true
    }
    fn get_data(&self) -> Value {
        Value::U32(self.value)
    }
    fn set_data(&mut self, v: &Value) {
        self.value = v.as_u32().unwrap_or(0).min(5);
    }
}

fn rating_dialog() -> Dialog {
    let mut d = Dialog::centered(30, 8, "Оценка");
    let r = d.add(Rating::new(12, 2));
    d.add(Label::new(Rect::new(2, 2, 11, 3), "~О~ценка:", Some(r)));
    d.add(Button::new(Rect::new(9, 4, 21, 6), "O~K~", cm::OK, bf::DEFAULT));
    d
}

fn test_rating() {
    let mut r = Rating::new(0, 0);
    r.set_data(&Value::U32(9));
    assert_eq!(r.get_data(), Value::U32(5));
    let _ = rating_dialog();
}
```

### Методы трейта

| Метод | Когда вызывается | По умолчанию |
|-------|------------------|--------------|
| `draw(&self, dc)` | при каждой перерисовке; **обязателен** | — |
| `handle_event(&mut self, ev, ctx)` | пришло событие | выбор щелчком мыши |
| `palette(&self)` | при рисовании: палитра для `dc.color(n)` | пустая — номера идут владельцу как есть |
| `set_state(&mut self, flag, on)` | меняется флаг `sf::*` (фокус, активность…) | просто меняет `vs.state` |
| `valid(&mut self, cmd, ctx) -> bool` | перед закрытием окна или выходом (`cmd` — команда) | `true` |
| `exec_result(&mut self, tag, cmd, view, ctx)` | закрылось модальное окно, запущенное через `ctx.exec` | ничего |
| `update_commands(&self, ctx)` | после каждого события — разрешить/запретить команды | ничего |
| `monitor_event(&mut self, ev)` | все события программы (если представление — `Program::event_monitor`) | ничего |
| `change_bounds(&mut self, r)` | меняются границы (окно растянули) | `vs.set_bounds(r)` |
| `size_limits(&self, owner)` | при изменении размера: (мин, макс) | (0, размер владельца) |
| `has_data`, `get_data`, `set_data` | обмен данными диалога | данных нет |
| `help_ctx(&self)` | строке состояния и справке нужен контекст | `vs.help_ctx` |
| `as_window`, `as_group`, `as_group_mut` | программе нужно окно / дети | `None` |
| `as_any`, `as_any_mut`, `into_any` | приведение типа (`downcast`) | пишет макрос |
| `contains_id(id)` | есть ли `id` в поддереве | только сам |

### Приведение типа

Представления хранятся как `Box<dyn View>`. Конкретный тип возвращает
`as_any().downcast_ref::<T>()`; удобнее `group.find::<T>(id)`,
`app.find_mut::<T>(id)`, а у модального окна — `app.exec_dialog(d)`,
который отдаёт диалог обратно его собственным типом.

```rust
use tvision::*;

fn text_of(v: &dyn View) -> Option<String> {
    v.as_any().downcast_ref::<InputLine>().map(|il| il.text())
}
```

---

## `ViewState`

**Общие поля** любого представления (поля `TView`). Хранится в поле `vs`.

| Поле | Что |
|------|-----|
| `id: ViewId` | уникальный номер (выдаётся в `ViewState::new`) |
| `origin: Point` | левый верхний угол **в координатах владельца** |
| `size: Point` | ширина и высота |
| `cursor: Point` | позиция курсора внутри представления |
| `state: u16` | флаги состояния `sf::*` |
| `options: u16` | флаги параметров `of::*` |
| `grow_mode: u8` | как меняться при изменении владельца `gf::*` |
| `drag_mode: u8` | как перетаскивать `dm::*` |
| `help_ctx: u16` | контекст справки (для строки состояния и F1) |

| Метод | Что |
|-------|-----|
| `ViewState::new(bounds)` | новые поля, видимое (`sf::VISIBLE`) |
| `bounds()`, `extent()` | границы у владельца / свои, от (0, 0) |
| `set_bounds(r)` | задать границы |
| `make_local(p)` | точку владельца — в свои координаты (для мыши) |
| `get_state(flag)` | установлен ли флаг |
| `is_visible()`, `is_selected()`, `is_focused()`, `is_active()`, `is_disabled()`, `is_selectable()` | частые проверки |
| `show_cursor()`, `hide_cursor()`, `set_cursor(x, y)`, `block_cursor()`, `normal_cursor()` | текстовый курсор |

## `ViewId`

`u32` — **номер представления**. В C++ представления знали друг друга по
указателям, здесь — по номерам: `d.add(v)` возвращает id, а
`group.find::<T>(id)` находит представление. Новый номер выдаёт
`view::next_id()` (его вызывает `ViewState::new`).

## Флаги

### `sf` — состояние (`vs.state`)

| Флаг | Значит |
|------|--------|
| `VISIBLE` | видно |
| `SELECTED` | текущее у своего владельца |
| `FOCUSED` | выбрано на всей цепочке до корня — получает клавиши |
| `ACTIVE` | лежит в активном окне |
| `DISABLED` | недоступно |
| `MODAL` | выполняется модально |
| `CURSOR_VIS`, `CURSOR_INS` | курсор виден; блочный (режим вставки) |
| `SHADOW` | рисовать тень (окна) |
| `DRAGGING` | окно перетаскивают |
| `DEFAULT` | кнопка сейчас «по умолчанию» |
| `EXPOSED` | видно на экране |

Флаги состояния меняет программа (через `set_state`); сами вы обычно
только читаете их: `self.vs.is_focused()`.

### `of` — параметры (`vs.options`)

| Флаг | Значит |
|------|--------|
| `SELECTABLE` | может получить фокус |
| `FIRST_CLICK` | щелчок по невыбранному не только выбирает, но и обрабатывается |
| `TOP_SELECT` | при выборе поднимается наверх (окна) |
| `PRE_PROCESS`, `POST_PROCESS` | получать клавиши до / после элемента в фокусе (строка состояния, кнопки) |
| `TILEABLE` | участвует в «Мозаике» и «Каскаде» |
| `CENTER_X`, `CENTER_Y`, `CENTERED` | центрировать при вставке |
| `VALIDATE` | вызывать `valid(cm::RELEASED_FOCUS)` при уходе фокуса |
| `FRAMED`, `BUFFERED` | оставлены для совместимости |

### `gf` — изменение размера (`vs.grow_mode`)

Что происходит с представлением, когда владелец меняет размер на `delta`:
`GROW_LO_X` / `GROW_LO_Y` двигают левый / верхний край, `GROW_HI_X` /
`GROW_HI_Y` — правый / нижний. `GROW_ALL` — все четыре (представление
едет вместе с правым нижним углом), `GROW_HI_X | GROW_HI_Y` —
растягивается. `GROW_REL` — пропорционально (окна на рабочем столе),
`FIXED` — без ограничения размером владельца.

```rust
use tvision::*;

/// Кнопка в правом нижнем углу окна, которая остаётся там при растягивании.
fn corner_button(w: &mut Window) {
    let s = w.group.vs.size;
    let mut b = Button::new(Rect::new(s.x - 14, s.y - 3, s.x - 2, s.y - 1), "O~K~", cm::OK, bf::DEFAULT);
    b.vs.grow_mode = gf::GROW_ALL;
    w.add(b);
}
```

### `dm` — перетаскивание (`vs.drag_mode`)

`DRAG_MOVE`, `DRAG_GROW`, `DRAG_GROW_LEFT` и ограничения `LIMIT_LO_X`,
`LIMIT_LO_Y`, `LIMIT_HI_X`, `LIMIT_HI_Y`, `LIMIT_ALL` (не выходить за
края владельца). Нужны окнам; что разрешено пользователю, задают флаги
окна `wf::*`.

---

## `Value`

**Данные элемента диалога** для `get_data`/`set_data` — замена записи
`TDialogData`, которую в C++ копировали побайтно.

| Вариант | У кого |
|---------|--------|
| `Str(String)` | `InputLine`, `Memo`, `Editor` |
| `U32(u32)` | флажки (маска), переключатели (номер) |
| `Int(i64)` | `ListBox` (номер строки), `InputLong`, `ProgressBar` |
| `List(Vec<Value>)` | группа: значения детей с данными в порядке вставки |
| `None` | нет данных |

Методы: `as_str()`, `as_u32()`, `as_int()`, `as_list()` — `Option` с
содержимым, если вариант подходит.

```rust
use tvision::*;

fn values() {
    let v = Value::List(vec![Value::Str("Аня".into()), Value::U32(0b101)]);
    let l = v.as_list().unwrap();
    assert_eq!(l[0].as_str(), Some("Аня"));
    assert_eq!(l[1].as_u32(), Some(5));
    assert_eq!(l[1].as_int(), None);
}
```

---

## `Ctx`

**Контекст событий** — то, через что представление действует «снаружи
себя». Передаётся в `handle_event`, `valid`, `exec_result`,
`update_commands`. Аналог доступа к `owner`, `TProgram::application` и
`TEventQueue` из C++.

Запросы выполняются **сразу после** того, как представление вернётся из
`handle_event`: владелец (или программа) забирает их и исполняет.

| Метод | Что делает | В C++ |
|-------|-----------|-------|
| `post_command(cmd, info)`, `put_event(ev)` | поставить команду / событие в очередь | `putEvent` |
| `message(cmd, info)` | разослать сообщение соседям (детям своего владельца) | `message(owner, evBroadcast, …)` |
| `select(id)` | сделать представление текущим у владельца | `view->select()` |
| `select_next(forward)` | перейти к следующему / предыдущему | `owner->selectNext` |
| `focus(id)` | передать фокус представлению где угодно (с окном и всеми владельцами) | — |
| `make_first(id)` | поднять наверх | `makeFirst` |
| `close(id)` | удалить представление из владельца | `destroy`, `close` |
| `end_modal(cmd)` | завершить модальное окно с кодом | `endModal` |
| `exec(view, reply_to, tag)` | открыть модальное окно; результат — в `exec_result` представления `reply_to` | `owner->execView` |
| `exec_near(…, anchor)`, `exec_placed(…, anchor, place)` | то же, с границами относительно другого представления | — |
| `popup_menu(menu, pos)` | контекстное меню | `TMenuPopup` |
| `with_program(f)` | выполнить `f(&mut Program)` после события | обращение к `application` |
| `grab_mouse(id)`, `release_mouse()`, `mouse_grabbed_by(id)` | захват мыши (протяжка) | `mouseEvent` в цикле |
| `command_enabled(cmd)`, `enable_command`, `disable_command`, `set_command(cmd, on)` | набор команд | `commandEnabled`… |
| `owner_size()` | размер владельца | `owner->size` |
| `screen_size`, `clipboard`, `phase` | поля: размер экрана, буфер обмена, фаза доставки | — |

### Модальное окно изнутри представления

Модальный цикл умеет запускать только программа, поэтому представление
**просит** её и получает ответ позже:

```rust
use tvision::*;

const TAG_NAME: u16 = 1;

/// Кнопка-подпись: щелчок спрашивает имя и показывает его.
pub struct AskName {
    pub vs: ViewState,
    pub name: String,
}

impl View for AskName {
    tvision::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let a = dc.color(1);
        dc.fill(self.vs.extent(), ' ', a);
        let t = if self.name.is_empty() { "[ввести имя]" } else { self.name.as_str() };
        dc.write_str(0, 0, t, a);
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        if let Event::MouseDown(_) = ev {
            let mut d = Dialog::centered(40, 8, "Имя");
            d.add(InputLine::new(Rect::new(3, 2, 37, 3), 30));
            d.add(Button::new(Rect::new(14, 4, 26, 6), "O~K~", cm::OK, bf::DEFAULT));
            ctx.exec(Box::new(d), Some(self.vs.id), TAG_NAME); // откроется после этого события
            ev.clear();
        }
    }

    fn exec_result(&mut self, tag: u16, cmd: u16, view: Box<dyn View>, _ctx: &mut Ctx) {
        if tag == TAG_NAME && cmd == cm::OK {
            let data = view.get_data(); // [Str(имя)]
            self.name = data.as_list().and_then(|l| l.first()).and_then(|v| v.as_str()).unwrap_or("").to_string();
        }
    }
}
```

Так работают сообщения валидаторов, окно истории, вопрос «Сохранить
изменения?» редактора. Из кода приложения (у которого есть `&mut Program`)
проще вызвать `app.exec_dialog(d)` напрямую.

### Действие с программой

```rust
use tvision::*;

/// Из любого представления: сменить узор рабочего стола.
fn change_pattern(ctx: &mut Ctx) {
    ctx.with_program(|app| app.desktop_mut().set_pattern('▒'));
}
```

## `Request` и `PlaceFn`

`Request` — запрос к программе, который `Ctx` отправляет методом
`ctx.request(r)`: открыть меню (`MenuBar`), контекстное меню (`Popup`),
модальное окно (`Exec`), передать фокус (`Focus`), выполнить функцию
(`WithProgram`). Обычно вместо него вызывают готовые методы `Ctx`
(`exec`, `popup_menu`, `focus`, `with_program`).

`PlaceFn` — `Box<dyn Fn(Rect, Point) -> Rect>`: по границам якоря и
размеру его владельца вычисляет границы модального окна (так окно истории
встаёт под строкой ввода). Передаётся в `ctx.exec_placed`.

## `CommandSet`

**Набор запрещённых команд** (`TCommandSet`). Хранится в программе;
читают его меню, строка состояния и кнопки (`DrawCtx::commands`).
Методы: `is_enabled(cmd)`, `enable(cmd)`, `disable(cmd)` (возвращают,
изменилось ли что-то). Команды больше 255 разрешены всегда. В своём коде
удобнее `app.enable_command` / `ctx.disable_command`.

## `Phase`

Фаза доставки фокусного события (`ctx.phase`): `PreProcess` (получают
представления с `of::PRE_PROCESS`), `Focused` (элемент в фокусе),
`PostProcess` (с `of::POST_PROCESS`). Нужна редко — например, кнопке,
чтобы отличить «Enter для меня» от «Enter для кнопки по умолчанию».

---

## `DrawCtx`

**Контекст рисования** одного представления: окно в экранный буфер с
обрезкой по видимой области и цепочкой палитр. Координаты — **свои**, от
(0, 0). Аналог методов `writeLine`, `writeStr`, `getColor` у `TView`.

```rust
use tvision::*;

/// Рамка из точек и надпись по центру.
fn draw_box(dc: &mut DrawCtx, size: Point, text: &str) {
    let a = dc.color(1);
    dc.fill(Rect::new(0, 0, size.x, size.y), '·', a);
    let w = text::str_width(text) as i32;
    dc.write_str((size.x - w) / 2, size.y / 2, text, a);
    let mut b = DrawBuffer::new(size.x as usize);
    b.move_char(0, '─', a, size.x as usize);
    dc.write_line(0, 0, size.x, 1, &b); // верхняя строка
}
```

| Метод / поле | Что |
|--------------|-----|
| `color(n)` | атрибут по номеру цвета своей палитры (через цепочку до палитры приложения) |
| `get_color(0xHHLL)` | пара цветов (`AttrPair`): младший байт — обычный, старший — подсветка |
| `map_color(n)` | то же, что `color` |
| `owner_palette_len()` | длина палитры владельца: 8 — окно, 32 — диалог, 135 — рабочий стол (так `Table` подбирает цвета) |
| `write_line(x, y, w, h, &buf)` | вывести буфер в `h` строк подряд |
| `write_str(x, y, s, attr) -> ширина`, `write_char(x, y, ch, attr, n)`, `write_cells(x, y, &cells)` | строка, символ, ячейки |
| `fill(r, ch, attr)` | залить прямоугольник |
| `recolor(r, attr)` | перекрасить уже нарисованное |
| `shadow(r)` | тень справа и снизу от `r` |
| `size`, `commands` | размер представления; набор команд (серые пункты) |
| `origin()`, `clip()` | положение на экране и видимая часть (для отладки) |
| `DrawCtx::new`, `sub` | создать корневой / дочерний контекст — это делает программа и группы |

---

## `Group`

**Представление с детьми** (`TGroup`). Рабочий стол, окно, диалог — это
группы или построены вокруг них. Дети лежат в `children` в порядке Z:
первый — нижний, последний — верхний; текущий — `current`.

```rust
use tvision::*;

/// Панель из двух строк: подпись и поле ввода как одно целое.
pub struct Field {
    pub group: Group,
    pub input: ViewId,
}

impl Field {
    pub fn new(r: Rect, title: &str) -> Self {
        let mut group = Group::new(r);
        group.vs.options |= of::SELECTABLE;
        let w = r.width();
        group.add(StaticText::new(Rect::new(0, 0, w, 1), title)); // координаты — внутри группы
        let input = group.add(InputLine::new(Rect::new(0, 1, w, 2), 80));
        Field { group, input }
    }

    pub fn text(&self) -> String {
        self.group.find::<InputLine>(self.input).map(|il| il.text()).unwrap_or_default()
    }
}

impl View for Field {
    tvision::impl_group_basics!(group); // данные, справка, размеры, дети — у группы

    fn draw(&self, dc: &mut DrawCtx) {
        self.group.draw_children(dc);
    }
    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        self.group.handle_group_event(ev, ctx);
    }
    fn set_state(&mut self, flag: u16, on: bool) {
        self.group.set_group_state(flag, on);
    }
    fn valid(&mut self, cmd: u16, ctx: &mut Ctx) -> bool {
        self.group.valid(cmd, ctx)
    }
}
```

| Метод | Что делает |
|-------|-----------|
| `Group::new(bounds)` | пустая группа |
| `add(v) -> ViewId`, `insert(Box<dyn View>)` | добавить ребёнка наверх |
| `remove(id) -> Option<Box<dyn View>>` | вынуть ребёнка |
| `index_of(id)`, `child(id)` | среди прямых детей |
| `view(id)`, `view_mut(id)` | в **поддереве** (с внуками) |
| `find::<T>(id)`, `find_mut::<T>(id)` | то же с приведением типа |
| `current_view()`, `current_view_mut()` | текущий ребёнок (активное окно рабочего стола) |
| `select_id(id, ctx)`, `select_index(i, ctx)`, `set_current(i, ctx)` | выбрать ребёнка |
| `select_next(forward, ctx)` | Tab / Shift+Tab |
| `reset_current()` | текущим — верхний видимый выбираемый |
| `make_first(i)`, `put_at(i, new)` | поменять место в Z-порядке |
| `focus_path(id, ctx)` | выбрать `id` и всех его владельцев |
| `call_on(id, ctx, f)` | выполнить `f` над представлением так, будто оно обрабатывает событие |
| `for_each_view(f)` | обойти поддерево |
| `broadcast_children(ev, ctx)`, `dispatch(i, ev, ctx)` | разослать / доставить событие |
| `get_data()`, `set_data(v)`, `has_data()` | данные всех детей (`Value::List`) |
| `draw_children(dc)`, `handle_group_event(ev, ctx)`, `set_group_state(flag, on)`, `change_bounds(r)`, `valid(cmd, ctx)` | «родительские» реализации методов `View` для своих групп |
| `geometry_of(id)`, `owner_origin_of(id)`, `cursor_pos(…)`, `announce_focus(ctx)`, `help_ctx()` | служебные |

---

## Макросы

| Макрос | Для чего | Что пишет |
|--------|----------|-----------|
| `impl_view_basics!()` | представление с полем `vs: ViewState` | `vs`, `vs_mut`, `as_any`, `as_any_mut`, `into_any` |
| `impl_view_basics_field!(f)` | обёртка: `ViewState` лежит в `self.f.vs` (как `Memo` вокруг `Editor`) | то же |
| `impl_group_basics!(путь)` | построено вокруг группы: `group`, `win.group`, `dlg.win.group` | всё выше + `as_group`, `as_group_mut`, данные (`has_data`/`get_data`/`set_data`), `help_ctx`, `change_bounds` |
| `impl_group_core!(путь)` | то же, но данные свои (`has_data`/`get_data`/`set_data` пишете сами) | |

Макросы вызываются внутри `impl View for …` (они разворачиваются в методы)
и экспортированы из корня крейта: `tvision::impl_view_basics!()`.

## Вспомогательные функции

* `view::default_handle_event(&vs, ev, ctx)` — поведение `TView::handleEvent`:
  щелчок по невыбранному выбираемому представлению выбирает его (а без
  `of::FIRST_CLICK` ещё и «съедает» щелчок). Вызывайте первой строкой в
  своём `handle_event`.
* `view::calc_bounds(v, delta, owner_size)` — новые границы по `grow_mode`
  при изменении владельца на `delta` (`calcBounds`); нужна своим группам.
* `view::HC_NO_CONTEXT` (0), `view::HC_DRAGGING` (1) — стандартные
  контексты справки.

# Справочник: программа, рабочий стол, окна, диалоги

Каркас любого приложения: программа с циклом событий, рабочий стол с
фоном, окна и диалоги. Модули `program` и `widgets::{desktop, background,
window, frame, dialog}`.

* [`Program`](#program) · [`AppHandler`](#apphandler) · [`NoHandler`](#nohandler)
* [`Desktop`](#desktop) · [`Background`](#background)
* [`Window`](#window) · [`wf`](#wf--флаги-окна) · [`Frame`, `FrameInfo`](#frame-и-frameinfo)
* [`Dialog`](#dialog) · [`DialogPalette`](#dialogpalette)

---

## `Program`

**Программа**: корневая группа (строка меню, рабочий стол, строка
состояния), цикл событий, палитра, набор команд, буфер обмена. Аналог
`TProgram` + `TApplication`.

```rust
use tvision::*;

struct App;

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(cm::NEW) {
            message_box(app, "\x03Привет!", mf::INFORMATION | mf::OK_BUTTON);
            ev.clear();
        }
    }
}

fn run() -> std::io::Result<()> {
    let menu = Menu::new().submenu("~Ф~айл", Menu::new().item_key("~П~ривет", cm::NEW, kb::F2, "F2").item_key("~В~ыход", cm::QUIT, kb::ALT_X, "Alt-X"));
    let status = vec![StatusDef::all(vec![StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT)])];
    let mut app = Program::with_terminal(Some(menu), Some(status))?;
    app.run(&mut App);
    Ok(())
}
```

### Создание

| Конструктор | Что |
|-------------|-----|
| `Program::with_terminal(menu, status)` | в настоящем терминале (Unix — termios + ANSI, Windows — консоль); при выходе терминал восстанавливается |
| `Program::new(backend, menu, status)` | с любым терминалом — например, `HeadlessBackend` для тестов |

`menu` — `Option<Menu>` (без меню — `None`, рабочий стол начнётся с первой
строки), `status` — `Option<Vec<StatusDef>>`.

### Цикл событий

| Метод | Что |
|-------|-----|
| `run(&mut handler)` | главный цикл до `cm::QUIT` или `quit()` |
| `step(&mut handler) -> bool` | одна итерация: отрисовка, событие, обработка (`false` — событий не было) |
| `pump(&mut handler)` | обработать всю очередь (тесты) |
| `quit()`, `is_running()` | завершить цикл / идёт ли он |
| `put_event(ev)` | поставить событие в очередь |
| `get_event()`, `handle(ev, handler)` | получить и обработать одно событие вручную |
| `redraw()` | перерисовать сейчас |

`cm::QUIT` сначала спрашивает `valid(cm::QUIT)` у всех окон: любое может
отказаться.

### Окна и модальность

| Метод | Что |
|-------|-----|
| `insert_window(w) -> ViewId` | вставить окно на рабочий стол и сделать текущим (0 — окно не прошло `valid(cm::VALID)`) |
| `insert_window_boxed(b)` | то же для `Box<dyn View>` |
| `exec_dialog(d) -> (cmd, d)` | **модально** выполнить и вернуть представление того же типа |
| `exec_view(b) -> (cmd, b)` | то же для `Box<dyn View>` (`execView`) |
| `exec_dialog_idle(d, f)`, `exec_view_idle(b, f)` | модально с фоновой работой: `f(&mut Program, id) -> Option<u16>` вызывается, когда событий нет; `Some(cmd)` закрывает окно |
| `modal_top()` | id верхнего модального окна |
| `focus(id)` | передать фокус представлению и всем его владельцам |

```rust
use tvision::*;

/// Модальный диалог с работой «в фоне»: полоса доходит до конца сама.
fn countdown(app: &mut Program) -> u16 {
    let mut d = Dialog::centered(40, 8, "Подождите");
    let bar = d.add(ProgressBar::new(Rect::new(3, 2, 37, 3), 50));
    d.add(Button::new(Rect::new(14, 4, 26, 6), "Отмена", cm::CANCEL, bf::DEFAULT));
    let (cmd, _) = app.exec_dialog_idle(d, |app, _dialog_id| {
        let b = app.find_mut::<ProgressBar>(bar)?;
        b.step(1);
        b.is_done().then_some(cm::OK)
    });
    cmd // cm::OK — дошло до конца, cm::CANCEL — нажали «Отмена»
}
```

### Доступ к частям

| Метод / поле | Что |
|--------------|-----|
| `desktop()`, `desktop_mut()` | рабочий стол |
| `status_line_mut()` | строка состояния |
| `find::<T>(id)`, `find_mut::<T>(id)` | представление по id во всём дереве (в том числе в модальном диалоге) |
| `group` | корневая группа |
| `desktop_id`, `menu_bar_id`, `status_line_id` | id стандартных частей |
| `help_ctx()` | контекст справки представления в фокусе (для F1) |
| `screen()`, `screen_text()` | последний кадр (тесты) |

### Команды, настройки, оболочка

| Метод / поле | Что |
|--------------|-----|
| `enable_command(cmd)`, `disable_command(cmd)`, `command_enabled(cmd)` | доступность команд (запрещённые пункты — серые) |
| `palette: Vec<Attr>` | палитра приложения (по умолчанию `palettes::GRUVBOX`) |
| `ctx: Ctx` | контекст событий; буфер обмена — `app.ctx.clipboard` |
| `double_delay`, `reverse_buttons` | интервал двойного щелчка, левша |
| `event_monitor: Option<ViewId>` | представление, которому показываются все события (`View::monitor_event`) |
| `modal_hook` | обработчик команд, которые не обработало **модальное** окно |
| `dos_shell(banner)` | выйти в оболочку ОС (`$SHELL` / `cmd.exe`) и вернуться |
| `suspended(\|\| …)` | выполнить функцию в обычном режиме терминала (запустить `vim`, `less`) и вернуться |

### `modal_hook`

Во время модального диалога `AppHandler` не вызывается. Если кнопка
диалога посылает свою команду (не `OK`/`CANCEL`), её можно поймать здесь:

```rust
use tvision::*;

const CM_BROWSE: u16 = 130;

fn export_dialog(app: &mut Program) -> Option<String> {
    let mut d = Dialog::centered(52, 9, "Экспорт");
    let field = d.add(InputLine::new(Rect::new(3, 3, 36, 4), 200));
    d.add(Button::new(Rect::new(37, 2, 49, 4), "~О~бзор…", CM_BROWSE, bf::NORMAL));
    d.add(Button::new(Rect::new(12, 6, 24, 8), "O~K~", cm::OK, bf::DEFAULT));
    app.modal_hook = Some(Box::new(move |app, ev| {
        if ev.command() == Some(CM_BROWSE) {
            let (cmd, f) = app.exec_dialog(FileDialog::new("*", "Файл", "~И~мя", fd::OK_BUTTON, 0));
            if cmd != cm::CANCEL {
                if let Some(il) = app.find_mut::<InputLine>(field) {
                    il.set_text(&f.file_name().display().to_string());
                }
            }
            ev.clear();
        }
    }));
    let (cmd, d) = app.exec_dialog(d);
    app.modal_hook = None;
    (cmd == cm::OK).then(|| d.win.group.find::<InputLine>(field).unwrap().text())
}
```

| Константа | Что |
|-----------|-----|
| `program::DOUBLE_CLICK` | 400 мс — интервал двойного щелчка по умолчанию |

## `AppHandler`

**Код приложения** — то, что в C++ было переопределёнными методами
наследника `TApplication`.

| Метод | Когда | По умолчанию |
|-------|-------|--------------|
| `handle_event(&mut self, app, ev)` | событие, которое не обработали представления (обычно команды меню) | ничего |
| `idle(&mut self, app)` | событий нет (часы, фоновая работа, обновление команд); во время модального окна не вызывается | ничего |

Обработчик получает `&mut Program` — из него можно открывать диалоги,
вставлять окна, менять палитру. Не забывайте `ev.clear()` для
обработанных команд.

## `NoHandler`

Пустой обработчик: `app.run(&mut program::NoHandler)` (тип лежит в модуле `program`) — для программ, где всё
делают сами представления.

---

## `Desktop`

**Рабочий стол** — группа между строкой меню и строкой состояния: фон и
окна. Аналог `TDesktop`.

```rust
use tvision::*;

fn arrange(app: &mut Program) {
    app.desktop_mut().set_pattern('▒');
    let r = app.desktop().group.vs.extent(); // весь стол, от (0, 0)
    app.desktop_mut().tile(r); // мозаика окон с of::TILEABLE
}
```

| Метод / поле | Что |
|--------------|-----|
| `pattern()`, `set_pattern(c)` | узор фона (по умолчанию `░`) |
| `tile(r)`, `cascade(r)` | мозаика / каскад окон с `of::TILEABLE` |
| `tile_columns_first` | мозаика сначала по столбцам |
| `group` | группа: окна — её дети (`children`), активное — `current_view()` |
| `background_id` | id фона |

Сам обрабатывает `cm::NEXT` (F6), `cm::PREV`, `cm::TILE`, `cm::CASCADE`.
Координаты окон — от левого верхнего угла стола (первая строка под меню).

## `Background`

**Фон рабочего стола**: залит символом `pattern`. Аналог `TBackground`.
`Background::new(bounds, pattern)`; обычно меняется через
`desktop.set_pattern(c)`. Свой фон (картинку, часы) можно сделать своим
представлением и вставить на стол самым нижним.

---

## `Window`

**Окно**: рамка с заголовком, номером и значками, группа детей. Аналог
`TWindow`.

```rust
use tvision::*;

fn list_window(app: &mut Program) {
    let r = Rect::new(2, 1, 42, 15);
    let mut w = Window::new(r, "Список", 1); // номер 1 — Alt+1
    w.group.vs.options |= of::TILEABLE;
    let sb = w.standard_scroll_bar(true); // полоса на правой рамке
    let mut lb = ListBox::new(w.interior(), 1, Some(sb));
    lb.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y; // тянется с окном
    lb.set_items((1..=50).map(|i| format!("Строка {}", i)).collect());
    w.add(lb);
    app.insert_window(w);
}
```

| Метод / поле | Что |
|--------------|-----|
| `Window::new(bounds, title, number)` | окно (`number` 1–9 видно в рамке; 0 — без номера) |
| `add(v)`, `insert(box)` | вставить элемент (координаты — от угла окна, рамка — строка/столбец 0) |
| `interior()` | внутренность без рамки |
| `standard_scroll_bar(vertical) -> ViewId` | полоса прокрутки на правой / нижней рамке |
| `zoom(owner_size)` | распахнуть / восстановить |
| `title`, `number`, `flags`, `palette` | заголовок, номер, флаги `wf::*`, палитра (`BLUE_WINDOW`, `CYAN_WINDOW`, `GRAY_WINDOW`) |
| `group` | группа: дети, `vs` окна |
| `id()` | id окна |
| `handle_window_event(ev, ctx)`, `draw_window(dc)`, `set_window_state(flag, on)` | «родительские» методы — для своих окон на основе `Window` |

Окно само обрабатывает перемещение и изменение размера мышью, Ctrl+F5
(`cm::RESIZE`), F5 (`cm::ZOOM`), Alt+F3 (`cm::CLOSE`), выбор по Alt+номер.

### Своё окно

Своё окно — структура с полем `Window` и делегированием (как
`ContactsWindow` в [уроке](01-tutorial.md#шаг-2-своё-окно-с-таблицей)):

```rust
use tvision::*;

pub struct NotesWindow {
    pub win: Window,
    pub memo: ViewId,
}

impl View for NotesWindow {
    tvision::impl_group_basics!(win.group);

    fn as_window(&self) -> Option<&Window> {
        Some(&self.win) // без этого не работают Alt+F3, F5 и Alt+номер
    }
    fn draw(&self, dc: &mut DrawCtx) {
        self.win.draw_window(dc);
    }
    fn palette(&self) -> &'static [u8] {
        self.win.palette
    }
    fn set_state(&mut self, flag: u16, on: bool) {
        self.win.set_window_state(flag, on);
    }
    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        self.win.handle_window_event(ev, ctx);
        // …своя обработка того, что осталось в `ev`…
    }
}
```

## `wf` — флаги окна

`Window::flags`: `MOVE` — можно двигать, `GROW` — менять размер, `CLOSE` —
значок закрытия `[■]`, `ZOOM` — значок распахивания `[↑]`. У окна по
умолчанию все четыре, у диалога — `MOVE | CLOSE`.

`window::MIN_WIN_SIZE` — наименьший размер окна (16×6).

## `Frame` и `FrameInfo`

**Рамка окна.** В этом порте рамка — не отдельное представление, а
рисовальщик: `Frame::draw_frame(dc, &FrameInfo { title, number, flags,
active, dragging, zoomed })` рисует рамку на всю область `dc`, палитра —
`Frame::palette()`. Окно вызывает его само в `draw_window`; вручную это
нужно, только если вы рисуете окно без `Window`.

Значки рамки — константы `frame::CLOSE_ICON` (`[■]`), `ZOOM_ICON` (`[↑]`),
`UNZOOM_ICON` (`[↕]`), `DRAG_ICON`, `DRAG_LEFT_ICON`. Активное окно
рисуется двойной рамкой, неактивное — одинарной.

---

## `Dialog`

**Диалог** — окно с серой палитрой для элементов управления; обычно
выполняется модально. Аналог `TDialog`.

```rust
use tvision::*;

fn ask_name(app: &mut Program) -> Option<String> {
    let mut d = Dialog::centered(40, 9, "Знакомство");
    let name = d.add(InputLine::new(Rect::new(10, 2, 37, 3), 40));
    d.add(Label::new(Rect::new(2, 2, 9, 3), "~И~мя:", Some(name)));
    d.add(Button::new(Rect::new(6, 5, 18, 7), "O~K~", cm::OK, bf::DEFAULT));
    d.add(Button::new(Rect::new(21, 5, 33, 7), "Отмена", cm::CANCEL, bf::NORMAL));
    let (cmd, d) = app.exec_dialog(d);
    (cmd == cm::OK).then(|| d.win.group.find::<InputLine>(name).unwrap().text())
}
```

| Метод / поле | Что |
|--------------|-----|
| `Dialog::new(bounds, title)` | в заданных границах |
| `Dialog::centered(w, h, title)` | размера `w`×`h`, центрируется при вставке |
| `add(v)`, `insert(box)` | вставить элемент; **порядок = порядок Tab = порядок данных** |
| `select(id)` | элемент, который получит фокус первым |
| `set_palette(DialogPalette::…)` | цветовая схема |
| `get_data()`, `set_data(v)` | данные всех элементов (`Value::List`) |
| `win` | окно: `win.title`, `win.flags`, `win.group` |
| `id()` | id диалога |

Поведение:

* **Esc** — `cm::CANCEL`, **Enter** — кнопка по умолчанию;
* `cm::OK`, `CANCEL`, `YES`, `NO` завершают модальный диалог (после
  `valid`; «Отмена» допустима всегда — валидаторы её не задерживают);
* остальные команды кнопок проходят дальше — к `modal_hook` или к
  `AppHandler`.

Диалог можно вставить и немодально — `app.insert_window(d)`: тогда он
живёт на рабочем столе как окно, а его кнопки посылают команды
приложению.

## `DialogPalette`

Цветовая схема диалога: `Gray` (по умолчанию), `Blue`, `Cyan` — палитры
`GRAY_DIALOG`, `BLUE_DIALOG`, `CYAN_DIALOG`
(`dpGrayDialog`, `dpBlueDialog`, `dpCyanDialog`).

# Справочник: меню и строка состояния

Модули `widgets::menu` и `widgets::statusline`.

* [`Menu`](#menu) · [`MenuItem`](#menuitem) · [`MenuBar`](#menubar)
* [Контекстное меню](#контекстное-меню) · [`MenuSession`, `MenuOutcome`](#menusession-и-menuoutcome)
* [`StatusLine`](#statusline) · [`StatusDef`](#statusdef) · [`StatusItem`](#statusitem) · [`HintFn`](#hintfn)

---

## `Menu`

**Меню** — список пунктов (`TMenu`). Собирается цепочкой вызовов и
передаётся программе (`Program::with_terminal(Some(menu), …)`), где
становится строкой меню; вложенные `Menu` — выпадающие подменю.

```rust
use tvision::*;

const CM_ABOUT: u16 = 120;
const HC_ABOUT: u16 = 2;

fn menu() -> Menu {
    Menu::new()
        // «≡» открывается ещё и по Alt+Пробел.
        .submenu_key("~≡~", KeyEvent::new(Key::Char(' '), Mods::ALT), Menu::new().item("~О~ программе...", CM_ABOUT).help(HC_ABOUT))
        .submenu(
            "~Ф~айл",
            Menu::new()
                .item_key("~О~ткрыть...", cm::OPEN, kb::F3, "F3")
                .submenu("~Н~едавние", Menu::new().item("a.txt", 200).item("b.txt", 201))
                .separator()
                .item_key("~В~ыход", cm::QUIT, kb::ALT_X, "Alt-X"),
        )
}

fn check() {
    let m = menu();
    assert_eq!(m.items.len(), 2);
    assert_eq!(m.find_hot_key(&kb::F3), Some(cm::OPEN)); // поиск по всему дереву
    assert_eq!(m.find_item('Ф'), Some(1));
}
```

| Метод | Что делает |
|-------|-----------|
| `Menu::new()` | пустое меню |
| `item(name, cmd)` | пункт-команда |
| `item_key(name, cmd, key, "подпись")` | пункт с клавишей быстрого вызова — работает и при закрытом меню; подпись рисуется справа |
| `submenu(name, menu)` | подменю |
| `submenu_key(name, key, menu)` | подменю, которое открывается ещё и клавишей |
| `separator()` | разделитель |
| `help(ctx)` | контекст справки **последнего** пункта (строка состояния и F1, пока пункт подсвечен) |
| `items` | пункты (`Vec<MenuItem>`) |
| `find_hot_key(&key) -> Option<cmd>` | команда по клавише во всём дереве (`findHotKey`) |
| `find_item(c) -> Option<i>` | пункт с горячей буквой (с учётом раскладки) |
| `box_size()`, `bar_positions()` | размеры для рисования (служебные) |

Как работает меню: F10 (`cm::MENU`) или щелчок открывает строку меню;
Alt+буква — подменю с этой горячей буквой (в любой раскладке); стрелки,
Enter, Esc, буквы — внутри; запрещённые команды серые. Выбранный пункт
посылает команду обычным путём: к активному окну, затем к `AppHandler`.

> Клавиши быстрого вызова ловятся **раньше** элемента в фокусе. Не
> назначайте пунктам меню клавиши, нужные строкам ввода и редактору
> (буквы, Ctrl+стрелки, Enter).

## `MenuItem`

**Пункт меню** — перечисление:

| Вариант | Поля |
|---------|------|
| `Command` | `name`, `command`, `key: Option<KeyEvent>`, `param` (подпись клавиши), `help_ctx` |
| `SubMenu` | `name`, `menu: Rc<Menu>`, `help_ctx`, `key` |
| `Separator` | — |

Методы: `name()`, `is_separator()`, `enabled(&commands)` (доступен ли —
команда не запрещена), `help_ctx()`. Обычно пункты создают методы `Menu`,
а не вручную.

## `MenuBar`

**Строка меню** (`TMenuBar`) — первая строка экрана. Программа создаёт её
сама из переданного `Menu`; её id — `app.menu_bar_id`.

| Поле / метод | Что |
|--------------|-----|
| `MenuBar::new(bounds, menu)` | создать (обычно не нужно) |
| `menu: Rc<Menu>` | меню — его можно заменить на ходу |
| `active` | подсвеченный пункт во время работы с меню |
| `item_at(x)` | пункт под столбцом `x` |

```rust
use std::rc::Rc;
use tvision::*;

/// Заменить меню целиком (например, после смены языка).
fn replace_menu(app: &mut Program, m: Menu) {
    if let Some(id) = app.menu_bar_id {
        if let Some(bar) = app.find_mut::<MenuBar>(id) {
            bar.menu = Rc::new(m);
        }
    }
}
```

## Контекстное меню

Всплывающее меню в любом месте экрана (`TMenuPopup`). Из обработчика
приложения — `app.ctx.popup_menu(menu, pos)`, из представления —
`ctx.popup_menu(menu, pos)`. Позиция — **абсолютная** (экранная).

```rust
use std::rc::Rc;
use tvision::*;

const CM_COPY_PATH: u16 = 131;

/// Правая кнопка мыши по свободному месту рабочего стола.
fn context_menu(app: &mut Program, ev: &mut Event) {
    if let Event::MouseDown(m) = ev {
        if m.buttons & event::MB_RIGHT != 0 {
            let menu = Rc::new(Menu::new().item("~К~опировать путь", CM_COPY_PATH).separator().item("~М~озаика", cm::TILE));
            app.ctx.popup_menu(menu, m.pos); // у AppHandler координаты мыши уже экранные
            ev.clear();
        }
    }
}
```

## `MenuSession` и `MenuOutcome`

Пока меню открыто, программа ведёт **сеанс меню**: `MenuSession`
перехватывает все события, рисует выпадающие окна поверх экрана и
возвращает `MenuOutcome` — `Continue` (меню открыто), `Closed` (закрыто
без выбора) или `Command(cmd)` (выбрана команда). Вызывать его самому не
нужно; он описан для тех, кто строит своё меню.
`menu::draw_menu_box(dc, &menu, sel)` рисует выпадающее окно.

---

## `StatusLine`

**Строка состояния** (`TStatusLine`) — последняя строка экрана: подсказки
клавиш, которые сами превращают нажатия в команды. Набор пунктов зависит
от **контекста справки** представления в фокусе.

```rust
use tvision::*;

const HC_EDITOR: u16 = 100;
const CM_BUILD: u16 = 140;

fn status() -> Vec<StatusDef> {
    vec![
        // Когда в фокусе представление с vs.help_ctx == HC_EDITOR:
        StatusDef::range(HC_EDITOR, HC_EDITOR, vec![
            StatusItem::new("~F2~ Сохранить", kb::F2, cm::SAVE),
            StatusItem::new("~F9~ Собрать", kb::f(9), CM_BUILD),
            StatusItem::new("", kb::F10, cm::MENU),
        ]),
        // Во всех остальных случаях (первый подходящий набор побеждает):
        StatusDef::all(vec![
            StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT),
            StatusItem::new("", kb::F10, cm::MENU), // невидимые пункты — только клавиши
            StatusItem::new("", kb::ALT_F3, cm::CLOSE),
            StatusItem::new("", kb::F5, cm::ZOOM),
            StatusItem::new("", kb::F6, cm::NEXT),
            StatusItem::new("", kb::CTRL_F5, cm::RESIZE),
        ]),
    ]
}

fn add_hint(app: &mut Program) {
    if let Some(sl) = app.status_line_mut() {
        sl.hint = Some(Box::new(|ctx| if ctx == HC_EDITOR { "Правка текста".into() } else { String::new() }));
    }
}
```

| Поле / метод | Что |
|--------------|-----|
| `StatusLine::new(bounds, defs)` | создать (обычно делает программа) |
| `defs: Vec<StatusDef>` | наборы пунктов — можно заменить на ходу |
| `help_ctx` | текущий контекст (ставит программа) |
| `hint: Option<HintFn>` | подсказка справа от пунктов |
| `items()` | пункты для текущего контекста |

* Пункт с пустым текстом не виден, но его клавиша работает.
* Пункты с запрещённой командой серые и не срабатывают; пункт с командой
  0 — просто надпись.
* Строка видит клавиши **раньше всех и во время модальных диалогов** —
  так F1 открывает справку из любого диалога. Поэтому не вешайте пункты на
  Enter, Esc, Tab и буквы.
* Контекст справки берётся у самого глубокого представления в фокусе с
  ненулевым `help_ctx`; пока открыто меню — у подсвеченного пункта.

## `StatusDef`

**Набор пунктов** для диапазона контекстов `min..=max` (`TStatusDef`).
`StatusDef::all(items)` — для всех контекстов, `StatusDef::range(min, max,
items)` — для диапазона. Поля `min`, `max`, `items`.

## `StatusItem`

**Пункт строки состояния** (`TStatusItem`):
`StatusItem::new("~F2~ Сохранить", kb::F2, cm::SAVE)`. Поля `text`
(C-строка: часть между `~` выделяется), `key: Option<KeyEvent>`,
`command`.

## `HintFn`

`Box<dyn Fn(u16) -> String>` — функция подсказки для `StatusLine::hint`:
получает контекст справки, возвращает текст справа (`TStatusLine::hint`).

# Справочник: проверка ввода

Валидатор прикрепляется к строке ввода (`InputLine::with_validator`) и
проверяет текст по ходу набора и перед закрытием диалога. Модуль
`widgets::validate`.

* [`Validator`](#validator) — трейт
* [`PictureValidator`](#picturevalidator) · [`PicResult`](#picresult)
* [`FilterValidator`](#filtervalidator) · [`RangeValidator`](#rangevalidator) · [`StringLookupValidator`](#stringlookupvalidator) · [`FnValidator`](#fnvalidator)
* [Свой валидатор](#свой-валидатор)

Как работает проверка:

* **по ходу набора** — после каждой правки строка спрашивает
  `is_valid_input`; «нет» — правка откатывается, неправильный символ просто
  не появляется (валидатор может и дописать строку — автозаполнение маски);
* **при закрытии** — командой, отличной от `cm::CANCEL` (обычно OK),
  строка спрашивает `is_valid`; «нет» — окно с `error_message()`, фокус
  возвращается в поле, диалог остаётся открытым;
* `status_ok()` — правилен ли сам валидатор (синтаксис маски); окно с
  неправильной маской `insert_window` не вставит.

```rust
use tvision::*;

fn fields(d: &mut Dialog) {
    d.add(InputLine::new(Rect::new(3, 2, 15, 3), 10).with_validator(PictureValidator::new("##.##.####", true)));
    d.add(InputLine::new(Rect::new(3, 4, 15, 5), 8).with_validator(FilterValidator::new("0123456789ABCDEFabcdef")));
    d.add(InputLine::new(Rect::new(3, 6, 15, 7), 3).with_validator(RangeValidator::new(1, 100)));
    d.add(InputLine::new(Rect::new(3, 8, 15, 9), 10).with_validator(StringLookupValidator::new(&["да", "нет"])));
}
```

---

## `Validator`

**Трейт проверки** (`TValidator`). Обязателен только `error_message`.

| Метод | Когда | По умолчанию |
|-------|-------|--------------|
| `is_valid_input(&self, s: &mut String, suppress_fill) -> bool` | после каждой правки; может изменить `s` | `true` |
| `is_valid(&self, s) -> bool` | при закрытии диалога | `true` |
| `error_message(&self) -> String` | текст ошибки для окна сообщения | — |
| `status_ok(&self) -> bool` | правилен ли сам валидатор | `true` |

`suppress_fill = true` передаётся при удалении символов: не дописывайте
строку, иначе пользователь не сможет стереть литерал маски.

## `PictureValidator`

**Маска ввода** в стиле Paradox (`TPXPictureValidator`).
`PictureValidator::new(маска, auto_fill)`; при `auto_fill = true`
литералы маски дописываются сами.

| Символ маски | Значит |
|--------------|--------|
| `#` | цифра |
| `?` | буква |
| `&` | буква, переводится в верхний регистр |
| `!` | любой символ в верхнем регистре |
| `@` | любой символ |
| `;` | следующий символ — буквально (`;#` — знак «#») |
| `*` | повтор: `*3#` — три цифры, `*#` — сколько угодно |
| `[ ]` | необязательная часть |
| `{ }` | группа |
| `,` | варианты: `{Да,Нет}` |
| остальное | литерал |

```rust
use tvision::*;

fn masks() {
    let date = PictureValidator::new("##.##.####", true);
    assert!(date.status_ok());
    assert!(date.is_valid("05.11.2026"));
    assert!(!date.is_valid("05.11"));      // значение должно соответствовать маске полностью
    assert!(!date.is_valid("ab.11.2026"));

    let yes_no = PictureValidator::new("{Да,Нет}", true);
    assert!(yes_no.is_valid("Да") && !yes_no.is_valid("Может"));

    let broken = PictureValidator::new("[##", false); // нет закрывающей скобки
    assert!(!broken.status_ok());
}
```

| Маска | Принимает |
|-------|-----------|
| `##.##.####` | дату `05.11.2026` |
| `(###) ###-####` | телефон |
| `#[#][#]` | число из 1–3 цифр |
| `&*?` | слово с заглавной буквы (`иван` → `Иван`) |
| `*#[.##]` | число с необязательными двумя знаками после точки |

Пустая строка маску не проходит. Для необязательного поля оберните
проверку в `FnValidator` (`|s| s.is_empty() || …`) или напишите свой
валидатор. Метод `picture(&mut s, auto_fill) -> PicResult` разбирает
строку по маске — пригодится для своих проверок. Текст ошибки —
`validate::PICTURE_ERROR` (переводится, см. [язык](21-platform.md#язык-интерфейса)).

## `PicResult`

Результат разбора маски (`TPicResult`): `Complete` (строка полностью
соответствует), `Incomplete` (можно продолжать набор), `Empty`, `Error`
(не соответствует), `Syntax` (ошибка в самой маске), `Ambiguous`,
`IncompNoFill`.

## `FilterValidator`

**Только символы из набора** (`TFilterValidator`):
`FilterValidator::new("0123456789")`. Набор — поле `valid_chars`. Лишний
символ просто не набирается.

```rust
use tvision::*;

fn filter() {
    let v = FilterValidator::new("0123456789+-() ");
    let mut s = "+7 (900)".to_string();
    assert!(v.is_valid_input(&mut s, false));
    let mut bad = "+7 abc".to_string();
    assert!(!v.is_valid_input(&mut bad, false));
}
```

## `RangeValidator`

**Целое в диапазоне** `min..=max` (`TRangeValidator`):
`RangeValidator::new(1, 100)`. По ходу набора пропускает цифры и `-`
(только при `min < 0`), при закрытии проверяет диапазон. Поля `min`,
`max`. Данные строки остаются строкой (`Value::Str`); для числа в данных
удобнее [`InputLong`](16-controls.md#inputlong).

```rust
use tvision::*;

fn range() {
    let v = RangeValidator::new(1, 100);
    assert!(v.is_valid("42"));
    assert!(!v.is_valid("0") && !v.is_valid("101"));
}
```

## `StringLookupValidator`

**Значение из списка** (`TStringLookupValidator`):
`StringLookupValidator::new(&["да", "нет"])`, поле `strings`. Набор не
ограничивается — проверяется только готовое значение. Для выбора из
списка с подсказкой есть [`Combo`](16-controls.md#combo-и-combowindow) с `cbx::ONLY_LIST`.

## `FnValidator`

**Проверка функцией** — удобство Rust-версии. Поля `f: Fn(&str) -> bool`
и `message`. Проверяет только окончательное значение.

```rust
use tvision::*;

fn email() -> InputLine {
    InputLine::new(Rect::new(3, 2, 40, 3), 60).with_validator(FnValidator {
        f: |s: &str| s.is_empty() || (s.contains('@') && s.contains('.')),
        message: "Введите адрес вида имя@домен.ru".into(),
    })
}
```

## Свой валидатор

```rust
use tvision::*;

/// Логин: латиница и цифры, 3–16 символов, всегда в нижнем регистре.
struct Login;

impl Validator for Login {
    fn is_valid_input(&self, s: &mut String, _suppress_fill: bool) -> bool {
        *s = s.to_lowercase(); // можно исправлять ввод
        s.chars().all(|c| c.is_ascii_alphanumeric()) && s.chars().count() <= 16
    }
    fn is_valid(&self, s: &str) -> bool {
        (3..=16).contains(&s.chars().count())
    }
    fn error_message(&self) -> String {
        "Логин: от 3 до 16 латинских букв и цифр".into()
    }
}

fn login() {
    let mut s = "Ivan".to_string();
    assert!(Login.is_valid_input(&mut s, false));
    assert_eq!(s, "ivan");
    assert!(!Login.is_valid("iv"));
    let _field = InputLine::new(Rect::new(3, 2, 22, 3), 16).with_validator(Login);
}
```

# Справочник: элементы управления

Всё, что кладут в диалоги и окна: кнопки, надписи, поля ввода, флажки,
списки, полосы, окна сообщений. Модули `widgets::*`.

* [`Button`](#button) · [`StaticText`, `ColoredText`](#statictext-и-coloredtext) · [`Label`](#label)
* [`InputLine`](#inputline) · [`InputLong`](#inputlong)
* [`Cluster`: `CheckBoxes`, `RadioButtons`, `MultiCheckBoxes`](#cluster)
* [`ScrollBar`](#scrollbar) · [`ListBox`](#listbox)
* [`History`, `HistoryWindow`](#history-и-historywindow) · [`Combo`, `ComboWindow`](#combo-и-combowindow)
* [`ProgressBar`](#progressbar)
* [Окна сообщений: `message_box`, `input_box`, `MsgBoxText`](#окна-сообщений)

Координаты элементов — от левого верхнего угла окна или диалога; строка
0 и столбец 0 заняты рамкой.

---

## `Button`

**Кнопка** (`TButton`): нажатие посылает команду. Тень и «нажатие» рисует
сама.

```rust
use tvision::*;

const CM_APPLY: u16 = 150;

fn buttons(d: &mut Dialog) {
    d.add(Button::new(Rect::new(4, 10, 16, 12), "O~K~", cm::OK, bf::DEFAULT)); // Enter
    d.add(Button::new(Rect::new(18, 10, 32, 12), "~П~рименить", CM_APPLY, bf::NORMAL));
    d.add(Button::new(Rect::new(34, 10, 46, 12), "Отмена", cm::CANCEL, bf::NORMAL)); // и Esc
}
```

| Поле / метод | Что |
|--------------|-----|
| `Button::new(bounds, title, cmd, flags)` | высота 2 (вторая строка — тень) |
| `title` | C-строка: Alt+буква нажимает кнопку, в диалоге — и просто буква |
| `command`, `flags`, `am_default` | команда, флаги, «по умолчанию сейчас» |
| `press(ctx)` | нажать программно |

Флаги `bf`: `NORMAL`, `DEFAULT` (по Enter), `LEFT_JUST` (текст влево),
`BROADCAST` (вместо команды — сообщение соседям с `Info::Id` кнопки;
так кнопка общается с соседними элементами, не выходя из диалога),
`GRAB_FOCUS` (забирать фокус при нажатии мышью).

Нажатие с `bf::NORMAL` ставит команду в очередь (`Info::Id` кнопки);
`OK`/`CANCEL`/`YES`/`NO` закрывают модальный диалог. Кнопка с
запрещённой командой рисуется серой и не нажимается.

## `StaticText` и `ColoredText`

**Текст** с переносом по словам (`TStaticText`). Строка, начинающаяся с
`\x03`, центрируется; `\n` — новая строка.

```rust
use tvision::*;

fn texts(d: &mut Dialog) {
    d.add(StaticText::new(Rect::new(2, 2, 40, 4), "\x03Длинный текст переносится по словам в пределах прямоугольника."));
    d.add(StaticText::colored(Rect::new(2, 5, 40, 6), "Внимание!", color::attr(14, 4))); // жёлтый на красном
}
```

| Поле / метод | Что |
|--------------|-----|
| `StaticText::new(bounds, text)` | текст цветом палитры |
| `StaticText::colored(bounds, text, attr)` | свой цвет (`TColoredText` из Dialog Design) |
| `text`, `attr` | можно менять на ходу (`app.find_mut::<StaticText>(id)`) |

`ColoredText` — псевдоним `StaticText`.

## `Label`

**Подпись** элемента (`TLabel`): щелчок по ней или Alt+буква передают
фокус элементу, а сама подпись подсвечивается, пока элемент в фокусе.

```rust
use tvision::*;

fn labeled(d: &mut Dialog) {
    let name = d.add(InputLine::new(Rect::new(12, 2, 40, 3), 60));
    d.add(Label::new(Rect::new(2, 2, 11, 3), "~И~мя:", Some(name)));
}
```

Поля: `text`, `link: Option<ViewId>` (связанный элемент), `light`
(подсвечена ли).

---

## `InputLine`

**Строка ввода** (`TInputLine`): прокрутка, выделение, буфер обмена,
вставка/замена, история, проверка.

```rust
use tvision::*;

fn inputs(d: &mut Dialog) -> ViewId {
    let mut il = InputLine::new(Rect::new(3, 2, 30, 3), 80).with_validator(FilterValidator::new("0123456789"));
    il.set_text("42"); // курсор в конец, текст выделен — набор заменит его
    il.history_id = Some(7); // запоминать введённое (вместе со значком History)
    d.add(il)
}

fn read(app: &Program, id: ViewId) -> String {
    app.find::<InputLine>(id).map(|il| il.text()).unwrap_or_default()
}
```

| Поле / метод | Что |
|--------------|-----|
| `InputLine::new(bounds, max_len)` | `max_len` — наибольшая длина в символах |
| `with_validator(v)`, `validator` | [проверка ввода](15-validators.md) |
| `text()`, `set_text(s)` | текст (`set_text` выделяет всё, как `setData`) |
| `cursor_pos()`, `set_cursor_pos(i)` | позиция курсора (номер символа) |
| `select_all(on)` | выделить всё / снять |
| `history_id: Option<u16>` | номер списка истории |
| данные | `Value::Str` |

Клавиши: ←/→, Home/End, Ctrl+←/→ (по словам), Shift+стрелки
(выделение), Backspace, Del, Ins (вставка/замена), Ctrl+Y (очистить),
Ctrl+Ins / Shift+Ins / Shift+Del (буфер обмена, в том числе системный).

## `InputLong`

**Поле для целого числа** в диапазоне (`TInputLong` из Dialog Design).

```rust
use tvision::*;

fn number(d: &mut Dialog) -> ViewId {
    let n = d.add(InputLong::new(Rect::new(14, 2, 24, 3), 8, -100, 1000, il::HEX, "~С~чётчик"));
    d.add(Label::new(Rect::new(2, 2, 13, 3), "~С~чётчик", Some(n)));
    n
}

fn parse() {
    assert_eq!(parse_long("$1F"), Some(31)); // как Val в Turbo Pascal
    assert_eq!(parse_long("-12"), Some(-12));
    let mut f = InputLong::new(Rect::new(0, 0, 10, 1), 8, 0, 100, 0, "N");
    f.set_value(500); // подрезается по диапазону
    assert_eq!(f.value(), 100);
}
```

| Поле / метод | Что |
|--------------|-----|
| `InputLong::new(bounds, max_len, low, high, flags, label)` | `label` — для сообщения об ошибке |
| `value()`, `set_value(v)`, `range_check()` | значение (с подрезкой) и проверка |
| `il`, `il_options`, `low`, `high` | вложенная `InputLine`, флаги, диапазон |
| данные | `Value::Int` |

Флаги `il`: `HEX` — можно вводить `$1F`, `BLANK_EQ_ZERO` — пустое = 0,
`DISPLAY_HEX` — показывать в шестнадцатеричном виде. Выход из поля и
закрытие диалога проверяют диапазон.

---

## `Cluster`

**Группа флажков или переключателей** (`TCluster`). Один тип на три вида;
`CheckBoxes`, `RadioButtons`, `MultiCheckBoxes` — псевдонимы `Cluster`.

```rust
use tvision::*;

fn clusters(d: &mut Dialog) {
    // 6 пунктов в 3 строках — два столбца.
    let mut cb = Cluster::check_boxes(Rect::new(3, 2, 30, 5), &["~А~", "~Б~", "~В~", "~Г~", "~Д~", "~Е~"]);
    cb.set_button_state(0b100, false); // пункт 2 («В») недоступен
    d.add(cb);
    d.add(Cluster::radio_buttons(Rect::new(32, 2, 50, 4), &["~Д~а", "~Н~ет"]));
    // Три состояния: « », «+», «-»; 2 бита на пункт.
    d.add(Cluster::multi_check_boxes(Rect::new(3, 6, 20, 8), &["~Ч~тение", "~З~апись"], 3, cf::TWO_BITS, " +-"));
}

fn marks() {
    let mut c = Cluster::check_boxes(Rect::new(0, 0, 20, 2), &["a", "b"]);
    c.press(1);
    assert!(c.mark(1) && !c.mark(0));
    assert_eq!(c.value, 0b10);
}
```

| Конструктор | Значение (`Value::U32`) |
|-------------|------------------------|
| `Cluster::check_boxes(bounds, items)` | маска: бит `i` — пункт `i` отмечен |
| `Cluster::radio_buttons(bounds, items)` | номер выбранного |
| `Cluster::multi_check_boxes(bounds, items, sel_range, flags, states)` | по `flags >> 8` бит на пункт; `states` — символ каждого состояния |

| Поле / метод | Что |
|--------------|-----|
| `value`, `sel`, `strings`, `kind: ClusterKind` | значение, пункт под курсором, надписи, вид |
| `mark(i)`, `multi_mark(i)` | отмечен ли / состояние пункта |
| `press(i)` | нажать пункт |
| `button_state(i)`, `set_button_state(mask, on)`, `enable_mask` | доступность пунктов |

Пункты раскладываются по столбцам: высота группы — число строк в столбце.
Клавиши: стрелки, пробел, горячая буква. Флаги `cf` (для
многосостоятельных): `ONE_BIT`, `TWO_BITS`, `FOUR_BITS`, `EIGHT_BITS`.

`ClusterKind` — `CheckBoxes`, `RadioButtons`, `MultiCheckBoxes`.

---

## `ScrollBar`

**Полоса прокрутки** (`TScrollBar`). `ScrollBar::new(bounds)` —
вертикальная, если ширина 1. Поля `value`, `min`, `max`, `pg_step`,
`ar_step`; метод `set_params(value, min, max, pg_step, ar_step)`.

Полоса и её хозяин (список, таблица, редактор) обмениваются сообщениями
внутри **общего владельца**: полоса рассылает `cm::SCROLL_BAR_CHANGED`,
хозяин присылает `cm::SCROLL_BAR_SET`. Поэтому полоса лежит в той же
группе, а хозяин получает её id в конструкторе. В окне полосы на рамке
создаёт `window.standard_scroll_bar(vertical)`.

## `ListBox`

**Список строк** (`TListBox`), в один или несколько столбцов.

```rust
use tvision::*;

fn languages(d: &mut Dialog) -> ViewId {
    let sb = d.add(ScrollBar::new(Rect::new(30, 2, 31, 8)));
    let mut lb = ListBox::new(Rect::new(3, 2, 30, 8), 2, Some(sb)); // 2 столбца
    lb.set_items(["Rust", "C++", "Pascal", "Go", "Zig"].iter().map(|s| s.to_string()).collect());
    lb.focus_item(2);
    d.add(lb)
}

fn chosen(d: &Dialog, id: ViewId) -> Option<String> {
    d.win.group.find::<ListBox>(id)?.selected_text().map(String::from)
}
```

| Поле / метод | Что |
|--------------|-----|
| `ListBox::new(bounds, num_cols, v_scroll)` | |
| `set_items(v)`, `items` | строки |
| `focused`, `focus_item(i)`, `top_item` | выбранная строка, первая видимая |
| `selected_text()`, `range()` | текст выбранной, число строк |
| `palette` | своя палитра (у списка истории — другая) |
| данные | `Value::Int` — номер выбранной |

Enter и двойной щелчок рассылают `cm::LIST_ITEM_SELECTED` с
`Info::Id(список)`; буква — поиск по первым буквам. Пустой список
показывает `listbox::EMPTY_TEXT` (`<empty>`).

---

## `History` и `HistoryWindow`

**История ввода** (`THistory`): значок `▐↓▌` справа от строки ввода
открывает окно с прошлыми значениями.

```rust
use tvision::*;

fn with_history(d: &mut Dialog) {
    let mut il = InputLine::new(Rect::new(3, 2, 30, 3), 80);
    il.history_id = Some(5);
    let id = d.add(il);
    d.add(History::new(Rect::new(30, 2, 33, 3), id, 5));
}

fn store() {
    history_add(9, "первый");
    history_add(9, "второй");
    assert_eq!(history_str(9, 0).as_deref(), Some("второй")); // 0 — самая свежая
    assert_eq!(history_count(9), 2);
}
```

* `History::new(bounds, link, history_id)` — значок 3×1; поля `link`,
  `history_id`. Окно открывается щелчком или ↓ в строке.
* Строки запоминаются при нажатии кнопки диалога и уходе фокуса.
* Общее хранилище: `history_add(id, s)`, `history_count(id)`,
  `history_str(id, i)`, `history_list(id)`, `clear_history()`; в списке не
  больше `history::HISTORY_LIMIT` (32), повтор переносится в начало.
* `HistoryWindow::new(bounds, id)` — само окно со списком;
  `selection()` — выбранная строка. Открывается значком само.

## `Combo` и `ComboWindow`

**Выпадающий список** (`TCombo` из Dialog Design) — значок рядом со
строкой ввода со списком готовых значений.

```rust
use tvision::*;

fn city(d: &mut Dialog) {
    let il = d.add(InputLine::new(Rect::new(3, 2, 25, 3), 30));
    let mut cb = Combo::new(Rect::new(25, 2, 26, 3), il, cbx::ONLY_LIST, &["Москва", "Тверь", "Казань"]);
    cb.activate_char('▼'); // свой значок
    d.add(cb);
}
```

| Поле / метод | Что |
|--------------|-----|
| `Combo::new(bounds, link, flags, items)` | обычно 1×1 или 3×1 справа от строки (`InputLine` или `InputLong`) |
| `new_list(v)`, `list` | список (сортируется без учёта регистра) |
| `activate_char(c)`, `show_char`, `activate` | значок и клавиша вызова (для стрелок — ↓) |

Флаги `cbx`: `ONLY_LIST` — ввести можно только значение из списка (буквы
ищут, пробел листает); `NO_TRANSFER` — не участвовать в обмене данными
(иначе данные — `Value::List` строк списка); `DISPOSES_LIST` — для
совместимости. `ComboWindow` — открывающееся окно списка;
`selection()` — выбранная строка.

---

## `ProgressBar`

**Полоса выполнения** — в оригинале её нет, сделана в том же стиле.

```rust
use tvision::*;

fn bars(d: &mut Dialog) {
    d.add(ProgressBar::new(Rect::new(3, 2, 43, 3), 200)); // «25%» по центру
    d.add(ProgressBar::new(Rect::new(3, 4, 43, 5), 10).with_style(ProgressStyle::Blocks).with_caption("{value} из {max} файлов"));
    d.add(ProgressBar::new(Rect::new(3, 6, 43, 7), 0).indeterminate()); // неизвестная длительность
}

fn values() {
    let mut p = ProgressBar::new(Rect::new(0, 0, 20, 1), 200);
    p.set_value(50);
    assert_eq!((p.percent(), p.fraction()), (25, 0.25));
    p.step(1000); // подрезается
    assert!(p.is_done());
}
```

| Поле / метод | Что |
|--------------|-----|
| `ProgressBar::new(bounds, max)` | диапазон `0..=max` |
| `with_range(min, max)`, `set_range` | другой диапазон |
| `with_style(ProgressStyle::Smooth / Blocks)`, `style` | плавно (1/8 клетки) или блоками `█░` |
| `with_caption("… {percent}% …")`, `caption`, `show_text` | надпись: `{value}`, `{min}`, `{max}`, `{percent}` |
| `indeterminate()`, `indeterminate` | бегающий блок |
| `with_colors(empty, filled)`, `colors` | свои цвета (нужны в окне — палитра рассчитана на диалог) |
| `set_value`, `step`, `value`, `fraction`, `percent`, `is_done`, `text` | значение |
| данные | `Value::Int` |

Обновлять — напрямую (`app.find_mut::<ProgressBar>(id)`) или сообщением
`ctx.message(cm::PROGRESS_SET, Info::Progress { id, value })`. В
модальном диалоге — функцией фоновой работы `exec_dialog_idle`
([пример](13-program.md#окна-и-модальность)); долгую работу ведите в
потоке ([глава 7](../07-dialogs.md#полоса-выполнения-progressbar),
`examples/progress.rs`, `examples/json_load.rs`). Палитра —
`progress::PROGRESS_PALETTE`.

---

## Окна сообщений

Готовые модальные окна (`messageBox`, `inputBox`).

```rust
use tvision::*;

fn dialogs(app: &mut Program) {
    // Возвращает команду нажатой кнопки.
    if message_box(app, "\x03Удалить файл?", mf::CONFIRMATION | mf::YES_BUTTON | mf::NO_BUTTON) == cm::YES {
        // …
    }
    message_box_rect(app, Rect::new(0, 0, 60, 12), "Длинное сообщение\nв несколько строк", mf::ERROR | mf::OK_BUTTON, true);
    if let Some(name) = input_box(app, "Новый файл", "~И~мя:", "без имени.txt", 100) {
        let _ = name;
    }
}
```

| Функция | Что |
|---------|-----|
| `message_box(app, msg, options) -> cmd` | окно 40×9 в центре |
| `message_box_rect(app, r, msg, options, centered) -> cmd` | своего размера |
| `build_message_box(r, msg, options) -> Dialog` | построить, не запуская (для `ctx.exec` из представления) |
| `input_box(app, title, label, initial, limit) -> Option<String>` | строка ввода; `Some` — нажали OK |

Флаги `mf` — вид (заголовок): `WARNING`, `ERROR`, `INFORMATION`,
`CONFIRMATION`; кнопки: `YES_BUTTON`, `NO_BUTTON`, `OK_BUTTON`,
`CANCEL_BUTTON`, `YES_NO_CANCEL`, `OK_CANCEL`. Первая кнопка — по
умолчанию. Текст переносится по словам; `\x03` в начале — по центру.

`MsgBoxText` — тексты кнопок и заголовков (`YES`, `NO`, `OK`, `CANCEL`,
`WARNING`, `ERROR`, `INFORMATION`, `CONFIRM`); на русский их переводит
`locale::set_language(Language::Ru)`.

# Справочник: таблица, дерево, прокрутка

Представления для больших объёмов данных. Модули `widgets::table`,
`widgets::treeview`, `widgets::outline`, `widgets::scroller`.

* [`Table`](#table) · [`TableModel`](#tablemodel) · [`VecTableModel`](#vectablemodel) · [`FnTableModel`](#fntablemodel) · [`TableColumn`, `Align`](#tablecolumn-и-align) · [`SelectionMode`](#selectionmode) · [`TableColors`](#tablecolors)
* [`TreeView`](#treeview) · [`TreeNode`](#treenode) · [`TreePath`, `TreeLoader`](#treepath-и-treeloader) · [`CheckState`](#checkstate) · [`TreeGlyphs`](#treeglyphs) · [`TreeColors`](#treecolors)
* [`Outline`, `Node`](#outline-и-node)
* [`Scroller`](#scroller) · [`Terminal`](#terminal)

---

## `Table`

**Таблица** с заголовками, сеткой, прокруткой, закреплёнными столбцами,
сортировкой, правкой ячеек и выделением строк. В оригинале её нет.
Данные таблица берёт из **модели** (`TableModel`) и запрашивает только
видимые ячейки — миллион строк не проблема.

```rust
use tvision::*;

fn people_window(app: &mut Program) -> ViewId {
    let mut w = Window::new(Rect::new(0, 0, 60, 16), "Люди", 1);
    let v = w.standard_scroll_bar(true);
    let h = w.standard_scroll_bar(false);
    let rows = vec![
        vec!["Анна".to_string(), "34".to_string(), "Тула".to_string()],
        vec!["Борис".to_string(), "27".to_string(), "Казань".to_string()],
    ];
    let model = VecTableModel::new(&["Имя", "Возраст", "Город"], rows).editable(true);
    let mut t = Table::new(w.interior(), Box::new(model), Some(v), Some(h))
        .with_columns(vec![TableColumn::new(16), TableColumn::new(8).align(Align::Right), TableColumn::new(16)])
        .row_numbers(true)
        .striped(true)
        .editable(true); // F2, Enter или буква — правка ячейки
    t.auto_width(100); // ширина по первым 100 строкам
    let id = w.add(t);
    app.insert_window(w);
    id
}

fn read_cell(app: &Program, id: ViewId) -> Option<String> {
    let t = app.find::<Table>(id)?;
    let (row, col) = t.cursor();
    Some(t.model().cell(row, col))
}
```

### Настройка (цепочкой и полями)

| Метод → поле | По умолчанию | Что |
|--------------|--------------|-----|
| `header(on)` → `show_header` | да | строка заголовков |
| `grid(on)` → `show_grid` | да | линии между столбцами |
| `row_numbers(on)` → `show_row_numbers`, `row_number_base` | нет, с 1 | столбец номеров строк |
| `frozen(n)` → `frozen_cols` | 0 | закрепить первые `n` столбцов слева |
| `selection(m)` → `selection_mode` | `Cell` | курсор на ячейке или на строке |
| `multi_select(on)` → `multi_select` | нет | выделение строк (Shift+стрелки, Ctrl+щелчок, пробел, Ctrl+A) |
| `editable(on)` → `allow_edit` | нет | правка ячеек (если модель разрешает) |
| `sortable(on)` → `allow_sort` | да | сортировка щелчком по заголовку |
| `resizable(on)` → `allow_resize` | да | ширина столбца мышью и Ctrl+←/→ |
| `striped(on)` → `zebra` | нет | чередование цвета строк |
| `with_colors(c)` → `colors` | по владельцу | свои цвета |
| `with_columns(v)` → `columns` | ширина 12 | описания столбцов |

### Методы

| Метод | Что |
|-------|-----|
| `Table::new(bounds, Box<dyn TableModel>, v_scroll, h_scroll)` | таблица; `h_scroll` прокручивает столбцы |
| `model()`, `model_mut()`, `set_model(m)` | модель (замена пересоздаёт столбцы) |
| `set_columns(v)`, `auto_width(n)`, `resize_column(col, d)` | столбцы |
| `refresh()` | после изменения данных модели: поправить курсор и прокрутку |
| `cursor() -> (строка, столбец)`, `set_cursor(r, c)` | курсор |
| `selected_rows()`, `select_rows(&rows)`, `clear_selection()` | выделение |
| `sort_by(col, asc) -> bool`, `sort_state()` | сортировка |
| `is_editing()`, `copy_text()` | идёт ли правка; текст для буфера обмена |

Сообщения соседям: `cm::TABLE_CURSOR_MOVED`, `cm::TABLE_CELL_EDITED`,
`cm::LIST_ITEM_SELECTED` (Enter или двойной щелчок по нередактируемой
ячейке; Enter таблица не поглощает — в диалоге он нажмёт кнопку по
умолчанию). Ctrl+C / Ctrl+Ins — копировать.

Цвета без настройки берутся у владельца: синее окно, диалог, рабочий
стол — по длине его палитры. Подробно — [глава 7](../07-dialogs.md#таблица-table),
пример — `examples/table.rs`.

## `TableModel`

**Источник данных таблицы** — трейт. Обязательны три метода.

| Метод | Что | По умолчанию |
|-------|-----|--------------|
| `row_count()`, `col_count()` | размеры | — |
| `cell(row, col) -> String` | текст ячейки (вызывается только для видимых) | — |
| `header(col)` | заголовок | `A`, `B`, … `AA` |
| `editable(row, col)` | можно ли править | нет |
| `set_cell(row, col, value) -> bool` | записать; `false` — не принято | `false` |
| `sort(col, asc) -> bool` | отсортировать строки | не умеет |

```rust
use tvision::*;

/// Таблица умножения 1000×1000 без хранения данных.
struct Multiply;

impl TableModel for Multiply {
    fn row_count(&self) -> usize {
        1000
    }
    fn col_count(&self) -> usize {
        1000
    }
    fn cell(&self, r: usize, c: usize) -> String {
        ((r + 1) * (c + 1)).to_string()
    }
    fn header(&self, c: usize) -> String {
        (c + 1).to_string()
    }
}

fn model() {
    let m = Multiply;
    assert_eq!(m.cell(2, 3), "12");
    let _t = Table::new(Rect::new(0, 0, 40, 10), Box::new(m), None, None);
}
```

## `VecTableModel`

**Данные в памяти**: `VecTableModel::new(&["Имя", …], rows)`, где `rows:
Vec<Vec<String>>`. `.editable(true)` разрешает правку; сортирует сама
(числа — как числа, см. `natural_cmp`). Поля `headers`, `rows`, `editable`.
После изменения `rows` через `model_mut` вызовите `table.refresh()`.

## `FnTableModel`

**Ячейки вычисляются функцией**: `FnTableModel::new(строк, столбцов,
|r, c| …).headers(&["A", "B"])`. Поля `rows`, `cols`, `headers`, `f`.
Самый короткий путь к «виртуальной» таблице.

```rust
use tvision::*;

fn squares() -> Table {
    let m = FnTableModel::new(1_000_000, 2, |r, c| if c == 0 { r.to_string() } else { (r * r).to_string() }).headers(&["n", "n²"]);
    Table::new(Rect::new(0, 0, 30, 10), Box::new(m), None, None)
}
```

Помощники модуля: `column_letters(i)` (0 → `A`, 26 → `AA`) и
`natural_cmp(a, b)` (если обе строки — числа, сравнивает как числа,
иначе — текст без учёта регистра).

## `TableColumn` и `Align`

**Описание столбца**: `TableColumn::new(ширина)` и цепочка `title(s)`,
`align(Align::Left / Center / Right)`, `limits(min, max)`,
`visible(false)` (скрытый, курсор пропускает), `resizable(false)`,
`sortable(false)`. Поля — те же (`title`, `width`, `min_width`,
`max_width`, `align`, `visible`, `resizable`, `sortable`).

## `SelectionMode`

`Cell` — курсор на ячейке, ←/→ ходят по столбцам; `Row` — курсор на
строке, ←/→ прокручивают столбцы.

## `TableColors`

**Свои цвета таблицы** (готовые атрибуты): `normal`, `zebra`, `header`,
`header_sorted`, `cursor`, `cursor_inactive`, `selected`, `grid`,
`row_number`. Запасной набор — `table::DEFAULT_TABLE_COLORS`.

---

## `TreeView`

**Дерево** с ленивой загрузкой, флажками с тремя состояниями, линиями
ветвей, быстрым поиском и правкой из программы. В оригинале его нет
(есть `TOutline` — см. [`Outline`](#outline-и-node)). Полное описание —
[глава 18](../18-treeview.md), пример — `examples/treeview.rs`.

```rust
use tvision::*;

fn project_tree(r: Rect) -> TreeView {
    let roots = vec![
        TreeNode::new("src")
            .expanded(true)
            .child(TreeNode::new("main.rs").data("src/main.rs"))
            .child(TreeNode::new("widgets").child(TreeNode::new("tree.rs"))),
        TreeNode::new("Cargo.toml"),
    ];
    TreeView::new(r, roots, None, None)
}

fn tree_ops() {
    let mut t = project_tree(Rect::new(0, 0, 30, 10));
    assert_eq!(t.row_count(), 4); // src, main.rs, widgets (свёрнут), Cargo.toml
    assert!(t.select_path(&[0, 1, 0])); // раскроет «widgets» и встанет на tree.rs
    assert_eq!(t.path_texts(&t.cursor_path().unwrap()), ["src", "widgets", "tree.rs"]);
    let p = t.push(Some(&[0]), TreeNode::new("lib.rs")).unwrap();
    assert_eq!(p, vec![0, 2]);
    t.remove(&p);
    assert_eq!(t.find_text("lib.rs"), None);
}
```

| Группа | Методы |
|--------|--------|
| создание | `TreeView::new(bounds, roots, v_scroll, h_scroll)`; цепочкой `lines(on)`, `checkboxes(on)`, `enter_toggles(on)`, `glyphs(g)`, `with_colors(c)`, `with_loader(f)` |
| узлы | `roots()`, `roots_mut()`, `set_roots(v)`, `node(path)`, `node_mut(path)` + `refresh()`, `path_texts(path)`, `find(pred)`, `find_text(s)`, `for_each(f)` |
| курсор | `cursor_path()`, `current()`, `select_path(path)`, `cursor_row()`, `set_cursor_row(i)`, `row_count()`, `row_path(i)` |
| раскрытие | `expand`, `collapse`, `toggle`, `set_expanded(path, on)`, `expand_subtree(path, глубина)`, `expand_all()`, `collapse_all()` |
| флажки | `check_state(path)`, `set_checked(path, on)`, `toggle_check(path)`, `checked_paths()` |
| правка | `insert(parent, index, node)`, `push(parent, node)`, `remove(path)` |
| поля | `show_lines`, `checkboxes`, `enter_toggles`, `glyphs`, `colors`, `v_scroll`, `h_scroll` |

Сообщения соседям: `cm::TREE_CURSOR_MOVED`, `TREE_NODE_TOGGLED`,
`TREE_CHECK_CHANGED`, `LIST_ITEM_SELECTED`.

## `TreeNode`

**Узел дерева**: собирается цепочкой `TreeNode::new(текст)` →
`.child(n)`, `.children(v)`, `.expanded(on)`, `.lazy()`, `.checked(on)`,
`.icon(c)`, `.color(attr)`, `.data(s)`. Поля открыты: `text`, `children`,
`expanded`, `lazy`, `checked`, `icon`, `color`, `data`. Помощники:
`has_children()`, `check_state()`, `set_checked_deep(on)`, `count()`.

## `TreePath` и `TreeLoader`

`TreePath` = `Vec<usize>` — путь к узлу, индексы детей от корня (`[0, 2]`
— третий ребёнок первого узла).

`TreeLoader` = `Box<dyn FnMut(&[usize], &TreeNode) -> Vec<TreeNode>>` —
функция ленивой загрузки: получает путь и узел с `lazy = true`, возвращает
детей. Задаётся `with_loader`; вызывается один раз, при первом раскрытии.

```rust
use tvision::*;

/// Бесконечное дерево: у каждого узла по три ребёнка, загружаемых по требованию.
fn infinite(r: Rect) -> TreeView {
    TreeView::new(r, vec![TreeNode::new("корень").lazy()], None, None)
        .with_loader(|path, node| (1..=3).map(|i| TreeNode::new(&format!("{}.{} (уровень {})", node.text, i, path.len()))).map(|n| n.lazy()).collect())
}
```

## `CheckState`

Состояние флажка: `Checked`, `Unchecked`, `Partial` (отмечена часть
детей). У узла с детьми вычисляется по детям.

## `TreeGlyphs`

Символы дерева: `vert` (`│`), `tee` (`├`), `corner` (`└`), `horiz` (`─`),
`collapsed` (`▸`), `expanded` (`▾`), `check_on`, `check_partial`,
`check_off`. Готовые наборы — `TreeGlyphs::UNICODE` (по умолчанию) и
`TreeGlyphs::ASCII`.

## `TreeColors`

`normal`, `cursor`, `cursor_inactive`, `lines`. Без настройки — по
палитре владельца; запасной набор — `treeview::DEFAULT_TREE_COLORS`.

---

## `Outline` и `Node`

**Дерево из оригинала** (`TOutline`, `TNode`) — для программ,
перенесённых с C++. Для новых удобнее `TreeView`.

```rust
use tvision::*;

fn outline(r: Rect) -> Outline {
    let root = Node::with_children("Проект", vec![Node::new("main.rs"), Node::with_children("docs", vec![Node::new("guide.md")], false)], true);
    Outline::new(r, None, None, vec![root])
}
```

| Поле / метод | Что |
|--------------|-----|
| `Node::new(text)`, `Node::with_children(text, children, expanded)` | узлы; поля `text`, `children`, `expanded` |
| `Outline::new(bounds, h_scroll, v_scroll, roots)` | дерево (обратите внимание: сначала горизонтальная полоса) |
| `roots`, `foc` | узлы и строка с фокусом |
| `update()` | пересчитать после правки `roots` |
| `row_count()`, `node_at(i)`, `focused_node()`, `focused_path()` | строки |
| `set_expanded(i, on)`, `expand_all(i)` | раскрытие по номеру строки |

Клавиши: стрелки, `+` `-` `*`, Enter — `cm::LIST_ITEM_SELECTED`.
Флаги рисования `outline::ov::*` и функция `create_graph` — для своей
отрисовки веток.

---

## `Scroller`

**Движок прокрутки** (`TScroller` был базовым классом, здесь — компонент,
который встраивают в своё представление): хранит смещение `delta` и
размер содержимого `limit`, связан с полосами прокрутки.

```rust
use tvision::*;

/// Просмотр длинного текста с прокруткой.
pub struct TextView {
    pub vs: ViewState,
    pub scroller: Scroller,
    pub lines: Vec<String>,
}

impl TextView {
    pub fn new(r: Rect, h: Option<ViewId>, v: Option<ViewId>, lines: Vec<String>) -> Self {
        let mut vs = ViewState::new(r);
        vs.options |= of::SELECTABLE;
        let mut scroller = Scroller::new(h, v);
        let w = lines.iter().map(|l| text::str_width(l)).max().unwrap_or(0) as i32;
        scroller.set_limit(w, lines.len() as i32, r.size());
        TextView { vs, scroller, lines }
    }
}

impl View for TextView {
    tvision::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let a = dc.color(1);
        let d = self.scroller.delta;
        let w = self.vs.size.x.max(0) as usize;
        for y in 0..self.vs.size.y {
            let mut b = DrawBuffer::new(w);
            b.move_char(0, ' ', a, w);
            if let Some(l) = self.lines.get((d.y + y) as usize) {
                let s: String = l.chars().skip(d.x.max(0) as usize).collect();
                b.move_str_limited(0, &s, a, w);
            }
            dc.write_line(0, y, self.vs.size.x, 1, &b);
        }
    }

    fn palette(&self) -> &'static [u8] {
        color::palettes::SCROLLER
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        view::default_handle_event(&self.vs, ev, ctx);
        let size = self.vs.size;
        self.scroller.handle(ev, size, self.vs.is_focused()); // полосы, колесо, стрелки
        self.scroller.sync(size, ctx); // сообщить полосам новые значения
    }
}
```

| Метод / поле | Что |
|--------------|-----|
| `Scroller::new(h, v)` | движок с id полос (`None` — без полосы) |
| `delta`, `limit` | смещение и размер содержимого |
| `set_limit(x, y, size)`, `scroll_to(x, y, size)` | задать размер / прокрутить (в пределах) |
| `handle(ev, size, focused) -> bool` | обработать полосы, колесо, стрелки и PgUp/PgDn |
| `sync(size, ctx)` | отправить полосам параметры (только при изменениях) |

## `Terminal`

**Окно вывода** (`TTerminal`): журнал с кольцевым буфером строк;
прокрутка следует за концом, пока пользователь не прокрутил вверх.

```rust
use tvision::*;

fn log_window(app: &mut Program) -> ViewId {
    let mut w = Window::new(Rect::new(0, 0, 60, 15), "Журнал", 0);
    let v = w.standard_scroll_bar(true);
    let log = w.add(Terminal::new(w.interior(), None, Some(v), 1000)); // хранить 1000 строк
    app.insert_window(w);
    log
}

fn log(app: &mut Program, id: ViewId, msg: &str) {
    if let Some(t) = app.find_mut::<Terminal>(id) {
        t.write_line(msg);
    }
}
```

Методы: `write(text)` (`\n` — новая строка), `write_line(text)`,
`clear()`, `lines()`; поля `max_lines`, `scroller`.

# Справочник: текстовый редактор

Модуль `widgets::editor`. Порт `TEditor`, `TMemo`, `TFileEditor`,
`TEditWindow`, `TIndicator`; текст хранится массивом строк, поэтому
Unicode работает полностью, а отмена многоуровневая.

* [`Editor`](#editor) · [`Pos`](#pos) · [`ef`, `ec`](#ef-и-ec)
* [`Memo`](#memo) · [`FileEditor`](#fileeditor) · [`EditWindow`](#editwindow) · [`Indicator`](#indicator)
* [`find_dialog`, `replace_dialog`, `display_width`, `absolute`](#функции-модуля)

---

## `Editor`

**Редактор текста** (`TEditor`): курсор, выделение, многоуровневая
отмена и повтор, поиск и замена, буфер обмена, автоотступ, мышь.

```rust
use tvision::*;

fn notes_window(app: &mut Program) -> ViewId {
    let mut w = Window::new(Rect::new(5, 2, 65, 18), "Заметки", 1);
    let h = w.standard_scroll_bar(false);
    let v = w.standard_scroll_bar(true);
    let mut ed = Editor::new(w.interior(), Some(h), Some(v), None);
    ed.set_text("Пишите здесь…");
    let id = w.add(ed);
    app.insert_window(w);
    id
}

fn editing() {
    let mut ed = Editor::new(Rect::new(0, 0, 40, 10), None, None, None);
    ed.set_text("первая строка\nвторая строка"); // сбрасывает отмену и «изменён»
    ed.set_select(Pos::new(1, 0), Pos::new(1, 6)); // выделить «вторая»
    assert_eq!(ed.selected_text(), "вторая");
    ed.insert_text("третья", false); // заменить выделение
    assert_eq!(ed.line(1), "третья строка");
    assert!(ed.modified);
    ed.undo(); // вставка поверх выделения отменяется одним шагом
    assert_eq!(ed.line(1), "вторая строка");
    ed.set_select(Pos::new(0, 0), Pos::new(0, 0));
    assert!(ed.search("строка", 0));
    assert_eq!(ed.selection(), Some((Pos::new(0, 7), Pos::new(0, 13))));
}
```

| Метод | Что |
|-------|-----|
| `Editor::new(bounds, h_scroll, v_scroll, indicator)` | редактор; полосы и индикатор — id соседей |
| `text()`, `set_text(s)` | весь текст (строки через `\n`); `set_text` сбрасывает отмену |
| `line_count()`, `line(i)` | строки |
| `cursor() -> Pos`, `selection()`, `selected_text()` | курсор и выделение |
| `set_select(from, to)` | выделить и поставить курсор в конец (`set_select(p, p)` — просто курсор) |
| `insert_text(s, select)` | вставить на место курсора, заменив выделение |
| `delete_select()` | удалить выделение |
| `undo()`, `redo()` | отмена, повтор |
| `search(what, flags) -> bool` | найти от курсора вперёд и выделить |
| `track_cursor(center)`, `scroll_to(x, y)` | прокрутка к курсору / в точку |
| `col_to_x(line, col)`, `x_to_col(line, x)` | номер символа ↔ экранный столбец (табуляции) |

| Поле | Что |
|------|-----|
| `modified`, `overwrite`, `auto_indent`, `can_undo` | флаги |
| `delta` | прокрутка (x — столбец, y — строка) |
| `find_str`, `replace_str`, `editor_flags` | строки и флаги поиска (`ef::*`) |
| `h_scroll`, `v_scroll`, `indicator` | id связанных представлений |
| `palette` | палитра (`palettes::EDITOR`; у `Memo` — `MEMO`) |

Редактор в фокусе сам обрабатывает и **разрешает** команды `cm::UNDO`,
`REDO`, `CUT`, `COPY`, `PASTE`, `CLEAR`, `SELECT_ALL`, `FIND`, `REPLACE`,
`SEARCH_AGAIN` — достаточно поставить их в меню, и пункты будут серыми,
когда действие невозможно. Клавиши — как в оригинале (WordStar:
Ctrl+K B, Ctrl+Q F…) и привычные (Shift+стрелки, Ctrl+Ins/Shift+Ins);
полная таблица — [глава 9](../09-editor.md#клавиши). Данные — `Value::Str`.

## `Pos`

**Позиция в тексте**: `line` и `col` (номер **символа** в строке, не
экранный столбец). `Pos::new(line, col)`; сравнивается (`<`, `==`) — так
удобно упорядочивать концы выделения.

## `ef` и `ec`

Флаги поиска и замены `ef`: `CASE_SENSITIVE`, `WHOLE_WORDS_ONLY`,
`PROMPT_ON_REPLACE`, `REPLACE_ALL`, `DO_REPLACE`, `BACKUP_FILES` (при
сохранении оставлять `.bak`).

`ec` — внутренние команды редактора (`CHAR_LEFT` … `DEL_WORD_LEFT`,
номера 500–525), как `cmCharLeft` в оригинале. Их можно послать
редактору, но обычно не нужно.

## `Memo`

**Многострочное поле в диалоге** (`TMemo`): Tab переходит к следующему
элементу, данные — весь текст (`Value::Str`).

```rust
use tvision::*;

fn comment_dialog() -> Dialog {
    let mut d = Dialog::centered(50, 14, "Комментарий");
    let v = d.add(ScrollBar::new(Rect::new(46, 2, 47, 10)));
    let memo = d.add(Memo::new(Rect::new(3, 2, 46, 10), None, Some(v)));
    d.add(Label::new(Rect::new(2, 1, 12, 2), "~Т~екст", Some(memo)));
    d.add(Button::new(Rect::new(13, 11, 25, 13), "O~K~", cm::OK, bf::DEFAULT));
    d.set_data(&Value::List(vec![Value::Str("Первая строка\nвторая".into())]));
    d
}
```

`Memo::new(bounds, h_scroll, v_scroll)`; внутренний редактор — поле
`editor`.

## `FileEditor`

**Редактор файла** (`TFileEditor`): загрузка, сохранение, «Сохранить
как», вопрос о сохранении при закрытии.

| Метод / поле | Что |
|--------------|-----|
| `FileEditor::new(bounds, h, v, indicator, path)` | существующий файл загружается сразу |
| `load_file()`, `save_file()` | `true` — успех; иначе текст в `last_error` |
| `file_name` | путь; пустой — «Untitled» (`editor::UNTITLED`) |
| `title()` | имя для заголовка окна |
| `editor` | внутренний `Editor` |

Обрабатывает `cm::SAVE`, `cm::SAVE_AS` (диалог выбора имени). Перед
закрытием изменённого текста `valid` спрашивает «Сохранить?» (Да / Нет /
Отмена) через `ctx.exec` — это образец асинхронного вопроса из
представления.

## `EditWindow`

**Окно редактора файла** (`TEditWindow`): `FileEditor`, полосы прокрутки,
индикатор в нижней рамке, имя файла в заголовке.

```rust
use std::path::Path;
use tvision::*;

fn open_file(app: &mut Program, file: &Path, number: i16) -> ViewId {
    let r = app.desktop().group.vs.extent();
    app.insert_window(EditWindow::new(r, file, number)) // пустой путь — новый файл
}

fn text_of(app: &Program, id: ViewId) -> Option<String> {
    Some(app.find::<EditWindow>(id)?.editor().editor.text())
}
```

`EditWindow::new(bounds, path, number)`; `editor()`, `editor_mut()`
(`&FileEditor`), поля `win`, `editor_id`. Полный пример — `examples/tvedit.rs`.

## `Indicator`

**Индикатор `строка:столбец`** и признак изменения `*` в нижней рамке
(`TIndicator`). `Indicator::new(bounds)`; поля `location`, `modified`.
Редактор обновляет его сам сообщением `cm::INDICATOR_SET`, если получил
его id в конструкторе.

## Функции модуля

| Функция | Что |
|---------|-----|
| `editor::find_dialog(find, flags) -> Dialog` | диалог «Поиск»; данные `[текст, флажки]` |
| `editor::replace_dialog(find, replace, flags) -> Dialog` | диалог «Замена»; данные `[что, на что, флажки]` |
| `editor::display_width(s)` | ширина строки на экране с табуляциями |
| `editor::absolute(path)` | абсолютный путь без обращения к ФС (`fexpand`) |

`editor::TAB_SIZE` — ширина табуляции (8).

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


# Справочник: терминал, буфер обмена, время, язык

Всё, что связывает программу с операционной системой. Модули `platform`,
`platform::clipboard`, `platform::time`, `platform::ansi`, `locale`.

* [`Backend`](#backend) · [`RawEvent`](#rawevent) · [`default_backend`](#default_backend)
* [`HeadlessBackend`](#headlessbackend) · [`UnixBackend`](#unixbackend) · [`WindowsBackend`](#windowsbackend)
* [`AnsiWriter`, `InputParser`, `Parsed`](#ansiwriter-inputparser-parsed)
* [`Clipboard`](#clipboard) · [`ClipboardBackend`](#clipboardbackend) · [`CommandClipboard`, `Osc52Clipboard`, `WinClipboard`](#готовые-источники-буфера)
* [`DateTime` и функции времени](#datetime-и-функции-времени)
* [Язык интерфейса: `Language`, `set_language`, `tr`](#язык-интерфейса)

---

## `Backend`

**Терминал** глазами программы — трейт (аналог каталога
`source/platform` оригинала). Программа знает о терминале только то, что
даёт этот трейт: так одна и та же программа работает в Unix, Windows, в
тестах и в графическом окне (tvision-gui).

```rust
use std::time::Duration;
use tvision::platform::{Backend, RawEvent};
use tvision::*;

/// Каркас своего терминала (вывод в окно, по сети…).
struct MyBackend {
    size: Point,
}

impl Backend for MyBackend {
    fn size(&self) -> Point {
        self.size
    }
    fn read(&mut self, _timeout: Duration) -> Option<RawEvent> {
        None // ждать событие не дольше timeout
    }
    fn flush(&mut self, _screen: &ScreenBuffer, _cursor: Option<Point>, _insert: bool) {
        // показать кадр; cursor — видимый курсор, insert — блочный курсор
    }
}

fn custom() {
    let mut app = Program::new(Box::new(MyBackend { size: Point::new(80, 25) }), None, None);
    app.pump(&mut program::NoHandler);
}
```

| Метод | Обязателен | Что |
|-------|-----------|-----|
| `size()` | да | размер экрана в ячейках |
| `read(timeout)` | да | следующее событие или `None` по тайм-ауту |
| `flush(screen, cursor, insert)` | да | показать кадр |
| `suspend()`, `resume()` | нет | вернуть терминал в обычный режим и обратно (`dos_shell`, `suspended`) |
| `system_clipboard()` | нет | подключать ли системный буфер обмена |
| `last_frame()` | нет | последний кадр (для тестов) |

## `RawEvent`

«Сырое» событие от терминала: `Key(KeyEvent)`, `Mouse { pos, buttons,
mods, wheel }` (**текущее** состояние мыши — нажатие, отпускание, двойной
щелчок и автоповтор программа вычисляет сама), `Resize(Point)`.

## `default_backend`

`platform::default_backend() -> io::Result<Box<dyn Backend>>` — терминал
для текущей ОС. Его вызывает `Program::with_terminal`.

## `HeadlessBackend`

**«Невидимый» терминал** для тестов: события из очереди, кадр в памяти.

```rust
use tvision::platform::{HeadlessBackend, RawEvent};
use tvision::*;

fn headless() {
    let mut b = HeadlessBackend::new(80, 25);
    b.push_key(kb::F10); // открыть меню
    b.push_key(kb::ESC); // закрыть
    b.input.push_back(RawEvent::Key(KeyEvent::new(Key::Char('x'), Mods::ALT)));
    let menu = Menu::new().submenu("~Ф~айл", Menu::new().item_key("~В~ыход", cm::QUIT, kb::ALT_X, "Alt-X"));
    let mut app = Program::new(Box::new(b), Some(menu), None);
    app.pump(&mut program::NoHandler);
    assert!(!app.is_running()); // Alt+X — выход
    assert!(app.screen_text().contains("Файл"));
}
```

| Поле / метод | Что |
|--------------|-----|
| `HeadlessBackend::new(w, h)` | экран `w`×`h` |
| `input: VecDeque<RawEvent>` | очередь ввода |
| `push_key(k)`, `push_click(x, y)` | добавить клавишу / щелчок |
| `frame`, `cursor`, `flushes` | последний кадр, курсор, число выводов |
| `size` | размер (смените и положите `RawEvent::Resize`, чтобы проверить изменение размера) |
| `esc_budget` | когда очередь пуста, после нескольких пустых чтений терминал «нажимает» Esc (не больше `esc_budget` раз, по умолчанию 50), чтобы забытое модальное окно не подвесило тест; `0` — не нажимать (нужно для окон, ждущих поток) |

Приёмы — [глава 14](../14-testing.md) и шаг 6 [урока](01-tutorial.md#шаг-6-тесты).

## `UnixBackend`

**Терминал Unix** (Linux, macOS, BSD): `termios` в «сыром» режиме,
альтернативный экран, мышь (SGR 1006), ANSI-цвета, `SIGWINCH`.
`UnixBackend::new()`; при удалении (`Drop`) терминал восстанавливается,
в том числе при панике.

## `WindowsBackend`

**Консоль Windows 10+**: Console API для ввода (клавиши, мышь, размер) и
VT-последовательности для вывода. `WindowsBackend::new()`. Alt+буква
приходит латинской буквой той же клавиши в любой раскладке.

## `AnsiWriter`, `InputParser`, `Parsed`

Строительные блоки своих терминалов, говорящих на ANSI:

* `AnsiWriter` — превращает кадр в управляющие последовательности, выводя
  только изменившиеся ячейки: `new()`, `with_mode(ColorMode)`,
  `render(&screen, cursor, insert) -> String`, `invalidate()` (следующий
  кадр целиком), функция `ansi::push_sgr(&mut out, attr, mode)`;
* `InputParser` — разбирает входные байты терминала: `parse(&buf) ->
  Parsed`, где `Parsed::Event(RawEvent, длина)`, `Incomplete` (дочитать),
  `Skip(n)` (пропустить мусор);
* `ansi::ENTER_SEQ`, `ansi::LEAVE_SEQ` — последовательности входа в
  полноэкранный режим и выхода из него.

---

## `Clipboard`

**Буфер обмена** программы — поле `ctx.clipboard` (из приложения —
`app.ctx.clipboard`). Строки ввода и редактор пользуются им сами.

```rust
use tvision::platform::clipboard::Clipboard;
use tvision::*;

fn clipboard(app: &mut Program) {
    app.ctx.clipboard.set("текст");
    let s = app.ctx.clipboard.get();
    let _ = (s, app.ctx.clipboard.backend_name()); // "wl-clipboard", "osc52", "windows"… или None
}

fn internal() {
    let mut c = Clipboard::internal(); // без системного буфера
    c.set("привет");
    assert_eq!(c.get(), "привет");
    assert!(c.has_text());
}
```

| Метод | Что |
|-------|-----|
| `Clipboard::system()` | связанный с буфером ОС (так у настоящего терминала) |
| `Clipboard::internal()` | только внутренний (так у `HeadlessBackend`) |
| `Clipboard::with_backend(b)` | со своим источником |
| `get()`, `set(s)` | прочитать (из системы, если можно) / записать (и в систему) |
| `has_text()`, `backend_name()` | есть ли что вставить; имя источника |

| ОС | Как |
|----|-----|
| Windows | WinAPI, `CF_UNICODETEXT` |
| macOS | `pbcopy` / `pbpaste` |
| Linux / BSD, Wayland | `wl-copy` / `wl-paste` |
| Linux / BSD, X11 | `xclip` или `xsel` |
| ничего из этого (SSH) | копирование — OSC 52, вставка — из внутреннего буфера |

## `ClipboardBackend`

**Источник системного буфера** — трейт: `get() -> Option<String>`,
`set(text) -> bool`, `name()`.

```rust
use tvision::platform::clipboard::{Clipboard, ClipboardBackend};

/// Буфер в файле (например, для общего буфера двух программ).
struct FileClipboard(std::path::PathBuf);

impl ClipboardBackend for FileClipboard {
    fn get(&mut self) -> Option<String> {
        std::fs::read_to_string(&self.0).ok()
    }
    fn set(&mut self, text: &str) -> bool {
        std::fs::write(&self.0, text).is_ok()
    }
    fn name(&self) -> &'static str {
        "file"
    }
}

fn use_file(app: &mut tvision::Program) {
    app.ctx.clipboard = Clipboard::with_backend(Box::new(FileClipboard("/tmp/clip.txt".into())));
}
```

## Готовые источники буфера

* `CommandClipboard` — через внешние утилиты (`pbcopy`, `wl-copy`,
  `xclip`, `xsel`); `CommandClipboard::detect()` находит доступную;
* `Osc52Clipboard` — копирование последовательностью терминала OSC 52
  (работает по SSH в большинстве терминалов; вставки нет);
* `WinClipboard` — WinAPI на Windows;
* `clipboard::base64_encode(bytes)` — для OSC 52, без внешних крейтов.

---

## `DateTime` и функции времени

`platform::time` — местное время без внешних крейтов (`localtime_r` на
Unix, WinAPI на Windows).

```rust
use tvision::platform::time;

fn clock() -> String {
    let t = time::now(); // DateTime { year, month, day, hour, minute, second }
    format!("{:02}.{:02}.{} {:02}:{:02}", t.day, t.month, t.year, t.hour, t.minute)
}

fn epoch() {
    let t = time::utc_from_secs(86_400);
    assert_eq!((t.year, t.month, t.day), (1970, 1, 2));
}
```

| Функция | Что |
|---------|-----|
| `time::now()` | текущее местное время |
| `time::local(system_time)` | местное время для момента |
| `time::utc_from_secs(secs)` | UTC по секундам от 1970 года |

---

## Язык интерфейса

Надписи, которые рисует сама библиотека (кнопки окон сообщений, файловые
диалоги, сообщения валидаторов, справка, поиск в редакторе), по умолчанию
английские. Русский включается одной строкой в начале `main`.

```rust
use tvision::locale::{self, Language};

fn russian() {
    locale::set_language(Language::Ru);
    assert_eq!(locale::tr("~Y~es"), "~Д~а");
    assert_eq!(locale::tr("Моя надпись"), "Моя надпись"); // без перевода — как есть
    locale::set_language(Language::En);
}
```

| Тип / функция | Что |
|---------------|-----|
| `Language` | `En` (по умолчанию), `Ru` |
| `locale::set_language(l)`, `locale::language()` | выбрать / узнать язык |
| `locale::tr(s)` | перевод встроенной надписи (английская строка — ключ) |

Надписи **своей** программы пишите сразу на нужном языке. Горячие буквы
работают в любой раскладке: Alt+Ф и Alt+A равнозначны
([глава 13](../13-locale.md)).


