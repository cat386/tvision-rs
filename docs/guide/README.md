# Руководство пользователя tvision-rs

`tvision-rs` — библиотека для программ с **текстовым интерфейсом**:
окнами, меню, диалогами и кнопками прямо в терминале, как в
Norton Commander или Turbo Pascal. Это перевод на Rust знаменитой
библиотеки Turbo Vision.

```text
  Файл  Контакт                                                       ← меню
╔═[■]══════════════ Контакты — без имени ═════════════════1═[↑]═╗
║         Имя          │     Телефон      │       Почта         ▲    ← окно с таблицей
║──────────────────────┼──────────────────┼─────────────────────▒
║Анна Петрова          │+7 900 111-22-33  │anna@example.ru      ▒
║Борис Смирнов         │+7 900 222-33-44  │boris@example.ru     ▼
 Ins Добавить  F4 Изменить  Del Удалить  F2 Сохранить  Alt-X Выход   ← подсказки клавиш
```

> Всё руководство одним файлом для чтения и печати —
> [tvision-rs-guide.pdf](../tvision-rs-guide.pdf) (A4, с оглавлением,
> закладками и указателем). Пересобрать после правок:
> `python3 tools/guide-pdf/build.py`.

## Для кого это руководство

Для тех, кто **немного знает Rust** (умеет создать проект через `cargo
new`, понимает структуры, `impl` и `match`), но **никогда не работал** с
Turbo Vision и текстовыми интерфейсами. Всё остальное объясняется по
ходу дела.

Если вы знаете Turbo Vision для C++ или Pascal, почти все классы
называются так же, только без буквы `T`: `TButton` → `Button`,
`TDialog` → `Dialog`. Отличия собраны в [главе «Перенос
программ»](../15-porting.md).

## Часть 1. Уроки

Уроки идут по порядку, каждый — на 15–30 минут. В каждом — готовая
маленькая программа: её можно запустить одной командой
(`cargo run --example …`) и разобрать строку за строкой. В конце урока
— задания «Попробуйте» и короткий список «Важно запомнить».

| Урок | Чему научитесь | Программа |
|------|----------------|-----------|
| [1. Первая программа](01-first-program.md) | создать проект, запустить пустую программу, словарик терминов | `learn_01_hello` |
| [2. Меню и команды](02-menu.md) | меню, строка состояния, свои команды, окно сообщения | `learn_02_menu` |
| [3. Окна](03-windows.md) | открывать, двигать, распахивать и закрывать окна | `learn_03_windows` |
| [4. Координаты](04-coordinates.md) | где что находится; раскладка без ошибок «на одну клетку» | `learn_04_coords` |
| [5. Первый диалог](05-dialog.md) | спросить у пользователя имя и прочитать ответ | `learn_05_dialog` |
| [6. Анкета](06-form.md) | флажки, переключатели, список, проверка ввода, обмен данными | `learn_06_form` |
| [7. Своё представление](07-own-view.md) | нарисовать свой элемент и научить его отвечать на клавиши и мышь | `learn_07_view` |
| [8. Своё окно с данными](08-own-window.md) | окно, которое хранит данные и отвечает на команды | `learn_08_todo` |
| [9. «Контакты»](09-contacts.md) | всё вместе: таблица, файлы, запрет команд, вопрос при выходе | `guide_contacts` |
| [10. Тесты](10-tests.md) | проверять интерфейс через `cargo test` | — |
| [11. MVC: данные в JSON и SQLite](11-mvc.md) | программа из описания полей: модель, окно, контроллер; файл JSON или база SQLite | `mvc_contacts` |
| [12. Рецепты: данные](12-data-recipes.md) | свои команды над моделью, своё окно, поиск, своё хранилище (CSV), консоль SQL, настройки в JSON | `mvc_todo`, `mvc_books`, `mvc_csv`, `sqlite_query`, `json_settings` |
| [13. Раскладки: без координат](13-layouts.md) | `HBox`, `VBox`, `Grid`, `Form`, пружины — как в Qt: элементы расставляются и растягиваются сами | `layout_demo` |
| [14. TVCreator: формы мышью](14-tvcreator.md) | дизайнер форм как Qt Designer: палитра, дерево, свойства, файл `.tvui`, генерация кода | `cargo run -p tvcreator` |

Как запустить программу урока: скачайте репозиторий библиотеки и
выполните в его каталоге, например, `cargo run --example learn_02_menu`.

## Часть 2. Справочник

Когда уроки пройдены — здесь можно найти любой класс. Для каждого: **что
это**, **когда нужен**, **как создать**, **что важно знать**, главные
методы и пример. Редко нужные методы перечислены отдельно, чтобы не
мешать.

| Страница | Что там |
|----------|---------|
| [Точки, прямоугольники, цвета, текст](20-basics.md) | `Point`, `Rect`, цвета, палитры, ширина текста |
| [События, клавиши, команды](21-events.md) | `Event`, `Key`, `KeyEvent`, `MouseEvent`, `cm`, `kb`; куда попадает событие |
| [Представления](22-views.md) | `View`, `ViewState`, `Ctx`, `DrawCtx`, `Group`, флаги, **координаты подробно** |
| [Программа, окна, диалоги](23-program.md) | `Program`, `AppHandler`, `Desktop`, `Window`, `Dialog` |
| [Меню и строка состояния](24-menus.md) | `Menu`, `MenuBar`, контекстное меню, `StatusLine` |
| [Проверка ввода](25-validators.md) | маски, диапазоны, фильтры, своя проверка |
| [Элементы управления](26-controls.md) | кнопки, поля, флажки, списки, история, выпадающий список, полоса выполнения, окна сообщений |
| [Таблица, дерево, прокрутка](27-data-views.md) | `Table`, `TreeView`, `Outline`, `Scroller`, `Terminal` |
| [Текстовый редактор](28-editor.md) | `EditWindow`, `Editor`, `Memo` |
| [Готовые диалоги](29-std-dialogs.md) | выбор файла, каталога, цветов; справка по F1 |
| [Файловые панели](30-files.md) | панели как в Norton Commander, копирование в фоне |
| [Терминал, буфер обмена, время, язык](31-platform.md) | тестовый терминал, буфер обмена, русский язык надписей |
| [Данные: JSON, SQLite, MVC](32-data.md) | разбор JSON, запросы к SQLite, `Model`, `ModelWindow`, `CrudController` |
| [Раскладки](33-layout.md) | `HBox`, `VBox`, `Grid`, `Form`, `Spacer`, политики размера, `set_layout`, `adjust_size` |

Все примеры кода в справочнике проверены: они компилируются, а проверки
`assert!` в них выполняются.

### Указатель классов

**A** — [`Align`](27-data-views.md#tablecolumn-и-align--столбцы) · [`AnsiWriter`](31-platform.md#ansiwriter-inputparser-parsed) · [`AppHandler`](23-program.md#apphandler--код-приложения) · [`Attr`](20-basics.md#attr--цвет) · [`AttrPair`](20-basics.md#attrpair--пара-цветов) · [`Alignment`](33-layout.md#alignment)

**B** — [`Backend`](31-platform.md#backend--терминал) · [`Background`](23-program.md#background--фон) · [`Button`](26-controls.md#button--кнопка)

**C** — [`Cell`](20-basics.md#cell--клетка-экрана) · [`ChDirDialog`](29-std-dialogs.md#chdirdialog--смена-каталога) · [`CheckBoxes`](26-controls.md#cluster--флажки-и-переключатели) · [`CheckState`](27-data-views.md#checkstate) · [`Clipboard`](31-platform.md#clipboard--буфер-обмена) · [`ClipboardBackend`](31-platform.md#clipboardbackend--свой-буфер-обмена) · [`Cluster`](26-controls.md#cluster--флажки-и-переключатели) · [`ClusterKind`](26-controls.md#cluster--флажки-и-переключатели) · [`Color256Dialog`](29-std-dialogs.md#color256dialog--выбор-одного-цвета) · [`Color256Selector`](29-std-dialogs.md#части-цветовых-диалогов) · [`ColorDialog`](29-std-dialogs.md#colordialog--настройка-цветов) · [`ColorDialogText`](29-std-dialogs.md#части-цветовых-диалогов) · [`ColorDisplay`](29-std-dialogs.md#части-цветовых-диалогов) · [`ColoredText`](26-controls.md#statictext--текст) · [`ColorGroup`](29-std-dialogs.md#colorgroup-и-coloritem) · [`ColorItem`](29-std-dialogs.md#colorgroup-и-coloritem) · [`ColorMode`](20-basics.md#colormode--сколько-цветов-у-терминала) · [`ColorSel`](29-std-dialogs.md#части-цветовых-диалогов) · [`ColorSelector`](29-std-dialogs.md#части-цветовых-диалогов) · [`Combo`](26-controls.md#combo--выпадающий-список) · [`ComboWindow`](26-controls.md#combo--выпадающий-список) · [`CommandSet`](22-views.md#commandset) · [`Conflict`](30-files.md#conflict) · [`Connection`](32-data.md#connection--база-sqlite) · [`CrossRef`](29-std-dialogs.md#helptopic-paragraph-crossref) · [`CrudController`](32-data.md#crudcontroller--контроллер) · [`Ctx`](22-views.md#ctx)

**D** — [`Data`](32-data.md#data--значение-поля) · [`DateTime`](31-platform.md#datetime--местное-время) · [`Desktop`](23-program.md#desktop--рабочий-стол) · [`Dialog`](23-program.md#dialog--диалог) · [`DialogPalette`](23-program.md#dialogpalette) · [`DirListBox`](29-std-dialogs.md#части-файловых-диалогов) · [`DrawBuffer`](20-basics.md#drawbuffer--строка-для-рисования) · [`DrawCtx`](22-views.md#drawctx) · [`DualPanel`](30-files.md#dualpanel--две-панели)

**E** — [`Editor`](28-editor.md#editor--редактор) · [`EditWindow`](28-editor.md#editwindow--окно-для-файла) · [`Event`](21-events.md#event--событие)

**F** — [`FileClass`](30-files.md#fileclass) · [`FileDialog`](29-std-dialogs.md#filedialog--выбор-файла) · [`FileDialogText`](29-std-dialogs.md#части-файловых-диалогов) · [`FileEditor`](28-editor.md#fileeditor--редактор-файла) · [`FileEntry`](29-std-dialogs.md#части-файловых-диалогов) · [`FileFilter`](30-files.md#filefilter--что-скрыть) · [`FileInfoPane`](29-std-dialogs.md#части-файловых-диалогов) · [`FileJob`](30-files.md#filejob--копирование-перенос-удаление) · [`FileKind`](30-files.md#filekind) · [`FileList`](29-std-dialogs.md#части-файловых-диалогов) · [`FilePanel`](30-files.md#filepanel--файловая-панель) · [`FilePanelColors`](30-files.md#filepanelcolors) · [`Field`](32-data.md#field--поле) · [`FieldKind`](32-data.md#fieldkind) · [`Form`](33-layout.md#form--форма) · [`FilterValidator`](25-validators.md#filtervalidator--только-разрешённые-символы) · [`FnTableModel`](27-data-views.md#fntablemodel--ячейки-из-функции) · [`FnValidator`](25-validators.md#fnvalidator--своя-проверка-функцией) · [`Frame`](23-program.md#frame-и-frameinfo--рамка) · [`FrameInfo`](23-program.md#frame-и-frameinfo--рамка)

**G** — [`Grid`](33-layout.md#grid--сетка) · [`Group`](22-views.md#group)

**H** — [`HBox`](33-layout.md#hbox-и-vbox--строка-и-столбец) · [`HeadlessBackend`](31-platform.md#headlessbackend--невидимый-терминал) · [`HelpFile`](29-std-dialogs.md#helpfile--файл-справки) · [`HelpTopic`](29-std-dialogs.md#helptopic-paragraph-crossref) · [`HelpViewer`](29-std-dialogs.md#helpviewer) · [`HelpWindow`](29-std-dialogs.md#helpwindow--окно-справки) · [`HintFn`](24-menus.md#hintfn--подсказка) · [`History`](26-controls.md#history--история-ввода) · [`HistoryWindow`](26-controls.md#history--история-ввода)

**I** — [`Indicator`](28-editor.md#indicator--строка-и-столбец) · [`Info`](21-events.md#message-и-info--команда-с-данными) · [`InputLine`](26-controls.md#inputline--строка-ввода) · [`InputLong`](26-controls.md#inputlong--поле-для-числа) · [`InputParser`](31-platform.md#ansiwriter-inputparser-parsed) · [`Item`](33-layout.md#item-и-item--уточнения-элемента)

**J** — [`JobKind`](30-files.md#jobkind) · [`JobProgress`](30-files.md#jobprogress) · [`Json`](32-data.md#json--значение-json) · [`JsonError`](32-data.md#jsonerror) · [`JsonStorage`](32-data.md#jsonstorage)

**K** — [`Key`](21-events.md#key--клавиша) · [`KeyEvent`](21-events.md#keyevent--нажатие-клавиши)

**L** — [`Label`](26-controls.md#label--подпись) · [`Language`](31-platform.md#язык-интерфейса-language) · [`Layout`](33-layout.md#layout--раскладка) · [`ListBox`](26-controls.md#listbox--список)

**M** — [`Margins`](33-layout.md#margins) · [`Memo`](28-editor.md#memo--многострочное-поле-в-диалоге) · [`Menu`](24-menus.md#menu--меню) · [`MenuBar`](24-menus.md#menubar--строка-меню) · [`MenuItem`](24-menus.md#menuitem--пункт-меню) · [`MenuOutcome`](24-menus.md#menusession-и-menuoutcome) · [`MenuSession`](24-menus.md#menusession-и-menuoutcome) · [`MemoryStorage`](32-data.md#memorystorage) · [`Message`](21-events.md#message-и-info--команда-с-данными) · [`Model`](32-data.md#model--модель) · [`ModelTableModel`](32-data.md#modeltablemodel-и-model_table) · [`ModelWindow`](32-data.md#modelwindow--окно-с-таблицей) · [`Mods`](21-events.md#mods--shift-ctrl-alt) · [`MouseEvent`](21-events.md#mouseevent--мышь) · [`MsgBoxText`](26-controls.md#окна-сообщений) · [`MultiCheckBoxes`](26-controls.md#cluster--флажки-и-переключатели)

**N** — [`Node`](27-data-views.md#outline-и-node--дерево-из-оригинала) · [`NoHandler`](23-program.md#nohandler)

**O** — [`Outline`](27-data-views.md#outline-и-node--дерево-из-оригинала)

**P** — [`PanelEntry`](30-files.md#panelentry--файл-в-панели) · [`Paragraph`](29-std-dialogs.md#helptopic-paragraph-crossref) · [`Parsed`](31-platform.md#ansiwriter-inputparser-parsed) · [`Phase`](22-views.md#phase) · [`PicResult`](25-validators.md#picresult) · [`PictureValidator`](25-validators.md#picturevalidator--маска) · [`PlaceFn`](22-views.md#request-и-placefn) · [`Point`](20-basics.md#point--точка) · [`Policy`](33-layout.md#policy-и-sizepolicy) · [`Pos`](28-editor.md#pos--место-в-тексте) · [`Program`](23-program.md#program--программа) · [`ProgressBar`](26-controls.md#progressbar--полоса-выполнения) · [`ProgressStyle`](26-controls.md#progressbar--полоса-выполнения)

**R** — [`RadioButtons`](26-controls.md#cluster--флажки-и-переключатели) · [`RangeValidator`](25-validators.md#rangevalidator--число-в-диапазоне) · [`RawEvent`](31-platform.md#rawevent) · [`Record`](32-data.md#record--запись) · [`Rect`](20-basics.md#rect--прямоугольник) · [`Request`](22-views.md#request-и-placefn)

**S** — [`Schema`](32-data.md#schema--схема) · [`ScreenBuffer`](20-basics.md#screenbuffer--весь-экран) · [`ScrollBar`](26-controls.md#scrollbar--полоса-прокрутки) · [`Scroller`](27-data-views.md#scroller--прокрутка-для-своих-представлений) · [`SelectionMode`](27-data-views.md#selectionmode) · [`SizePolicy`](33-layout.md#policy-и-sizepolicy) · [`SortMode`](30-files.md#sortmode) · [`Spacer`](33-layout.md#spacer--распорка) · [`SqlError`](32-data.md#sqlerror-и-rows) · [`SqliteStorage`](32-data.md#sqlitestorage) · [`SqlValue`](32-data.md#sqlvalue) · [`StaticText`](26-controls.md#statictext--текст) · [`StatusDef`](24-menus.md#statusdef--набор-пунктов) · [`StatusItem`](24-menus.md#statusitem--пункт-строки-состояния) · [`StatusLine`](24-menus.md#statusline--строка-состояния) · [`Storage`](32-data.md#storage--хранилище) · [`StringLookupValidator`](25-validators.md#stringlookupvalidator--значение-из-списка)

**T** — [`Table`](27-data-views.md#table--таблица) · [`TableColors`](27-data-views.md#tablecolors) · [`TableColumn`](27-data-views.md#tablecolumn-и-align--столбцы) · [`TableModel`](27-data-views.md#tablemodel--откуда-таблица-берёт-данные) · [`Terminal`](27-data-views.md#terminal--окно-журнала) · [`TreeColors`](27-data-views.md#treecolors) · [`TreeGlyphs`](27-data-views.md#treeglyphs) · [`TreeLoader`](27-data-views.md#treepath-и-treeloader) · [`TreeNode`](27-data-views.md#treenode--узел-дерева) · [`TreePath`](27-data-views.md#treepath-и-treeloader) · [`TreeView`](27-data-views.md#treeview--дерево)

**U** — [`UnixBackend`](31-platform.md#unixbackend-и-windowsbackend)

**V** — [`VBox`](33-layout.md#hbox-и-vbox--строка-и-столбец) · [`Validator`](25-validators.md#validator--свой-валидатор) · [`Value`](22-views.md#value) · [`VecTableModel`](27-data-views.md#vectablemodel--данные-в-памяти) · [`View`](22-views.md#view) · [`ViewId`](22-views.md#viewid) · [`ViewState`](22-views.md#viewstate)

**W** — [`Window`](23-program.md#window--окно) · [`WindowsBackend`](31-platform.md#unixbackend-и-windowsbackend)

### Модули констант

| Модуль | Что в нём | Где описан |
|--------|-----------|-----------|
| `cm` | команды | [события](21-events.md#cm--стандартные-команды) |
| `kb` | клавиши | [события](21-events.md#kb--готовые-клавиши) |
| `sf`, `of`, `gf`, `dm` | флаги представлений | [представления](22-views.md#флаги-sf-of-gf-dm) |
| `wf` | что можно делать с окном | [окна](23-program.md#wf--что-можно-делать-с-окном) |
| `bf` | флаги кнопки | [кнопка](26-controls.md#button--кнопка) |
| `mf` | окна сообщений | [окна сообщений](26-controls.md#окна-сообщений) |
| `cf`, `cbx`, `il` | флажки, выпадающий список, поле для числа | [элементы](26-controls.md) |
| `ef`, `ec` | поиск в редакторе | [редактор](28-editor.md#ef-и-ec) |
| `fd`, `cd` | файловые диалоги | [готовые диалоги](29-std-dialogs.md#filedialog--выбор-файла) |
| `color::palettes` | палитры | [цвета](20-basics.md#палитры) |
| `mvc::CM_RECORD_*` | команды записей | [данные](32-data.md#crudcontroller--контроллер) |

Все номера команд и константы — в [главе 16](../16-reference.md).

## Что ещё почитать

* **Примеры** в каталоге `examples/`: `demo` (все элементы диалогов),
  `tvdemo` (демонстрационная программа Turbo Vision), `tvedit` (текстовый
  редактор), `table`, `treeview`, `json_table`, `mvc_contacts` (записи в
  JSON или SQLite), `mvc_todo`, `mvc_books`, `mvc_csv`, `sqlite_query`,
  `json_settings` (рецепты урока 12), `layout_demo` (раскладки), `lfm` (файловый менеджер).
* **[Тематические главы](../README.md)** — как библиотека устроена
  внутри: архитектура, события, цвета и палитры, перенос программ.
