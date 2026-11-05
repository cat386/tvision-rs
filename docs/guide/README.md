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
