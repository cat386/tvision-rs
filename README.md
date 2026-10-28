# tvision-rs

Порт библиотеки текстового интерфейса **Turbo Vision** ([magiblot/tvision](https://github.com/magiblot/tvision)) с C++ на Rust.

* без внешних зависимостей (работа с терминалом реализована внутри, как в оригинале);
* Linux / macOS / BSD (termios + ANSI) и Windows 10+ (Console API + VT);
* мышь, Unicode (кириллица, широкие символы), 16 цветов с оригинальными палитрами
  и 256 цветов xterm (`color::attr256`, темы, выбор из 256 в `ColorDialog`);
* тема [gruvbox](https://github.com/morhetz/gruvbox) по умолчанию
  (`color::palettes::GRUVBOX`; классическая — `APP_COLOR`, она же ставится
  в 16-цветных терминалах);
* комментарии в коде — на русском.

## Документация

Подробное руководство на русском — в каталоге [`docs/`](docs/README.md):
быстрый старт, архитектура, события, свои представления, окна, меню,
диалоги и элементы, проверка ввода, редактор, стандартные диалоги, цвета,
платформа, язык интерфейса, тестирование, перенос программ с C++/Pascal и
справочник констант. Справка по отдельным типам — `cargo doc --open`.

## Запуск примеров

```
cargo run --example hello   # порт hello.cpp
cargo run --example demo    # окна, меню, диалог со всеми элементами, список, контекстное меню
cargo run --example tvedit  # порт примера tvedit: многооконный редактор файлов
cargo run --example tvhc -- examples/demohelp.txt hc.rs   # проверка справки и константы контекстов
cargo run --example progress  # полоса выполнения: копирование, работа в потоке, все виды полос
cargo run --example json_parse  # разбор JSON в редакторе: проверка с переходом к ошибке, форматирование, сжатие
cargo run --example json_table -- examples/data/books.json  # JSON-файл в таблице: сортировка, окно записи
cargo run --example json_load   # загрузка большого JSON в потоке с диалогом ProgressBar и отменой
cargo run --example table   # таблица: миллион строк, своя модель, правка, сортировка, диалог
cargo run --example colors  # 256 цветов: палитра xterm, темы gruvbox/классика/сумерки, выбор цвета, режим 16/256
cargo run --example lfm     # файловый менеджер по мотивам lfm: две панели, вкладки, F5/F6/F8 с прогрессом, PowerCLI
cargo run --example tvdemo  # порт tvdemo: калькулятор, календарь, пятнашки, ASCII, просмотр файлов...
cargo test --all-targets    # все тесты, включая тесты tvdemo
cargo test                  # тесты на «невидимом» терминале
```

## Что уже перенесено

| C++                                   | Rust                                        |
|---------------------------------------|---------------------------------------------|
| `TPoint`, `TRect`                     | `Point`, `Rect`                             |
| `TDrawBuffer`, `TScreenCell`          | `DrawBuffer`, `Cell`, `ScreenBuffer`        |
| `TEvent`, `kbXXX`, `cmXXX`            | `Event`, `KeyEvent`, модули `kb`, `cm`      |
| `TView`                               | трейт `View` + `ViewState`                  |
| `TGroup`                              | `Group`                                     |
| `TProgram`, `TApplication`            | `Program` + трейт `AppHandler`              |
| `TDesktop`, `TBackground`             | `Desktop`, `Background` (мозаика, каскад)   |
| `TWindow`, `TFrame`                   | `Window`, `Frame`                           |
| `TMenuBar`, `TMenuBox`, `TMenuPopup`  | `MenuBar`, `MenuSession`, `Menu`, `MenuItem`|
| `TStatusLine`, `TStatusDef`, `TStatusItem` | `StatusLine`, `StatusDef`, `StatusItem` |
| `TDialog`                             | `Dialog`                                    |
| `TButton`                             | `Button`                                    |
| `TStaticText`, `TLabel`               | `StaticText`, `Label`                       |
| `TInputLine`                          | `InputLine` (выделение, буфер обмена, проверка) |
| `TCheckBoxes`, `TRadioButtons`        | `Cluster::check_boxes`, `Cluster::radio_buttons` |
| `TScrollBar`                          | `ScrollBar`                                 |
| `TListViewer`, `TListBox`             | `ListBox` (несколько столбцов)              |
| — (нет в оригинале)                   | `Table` — таблица: модель данных, правка, сортировка, закреплённые столбцы |
| — (нет в оригинале)                   | `FilePanel`, `DualPanel`, `fileops` — файловые панели (по мотивам lfm) и операции в потоке |
| `messageBox`, `inputBox`              | `message_box`, `message_box_rect`, `input_box` |
| `platform/*`                          | `platform::{unix, windows, ansi}`           |
| `TValidator`, `TPXPictureValidator`, `TFilterValidator`, `TRangeValidator`, `TStringLookupValidator` | трейт `Validator`, `PictureValidator`, `FilterValidator`, `RangeValidator`, `StringLookupValidator` |
| `THistory`, `THistoryWindow`, `historyAdd` | `History`, `HistoryWindow`, `history_add` |
| `TScroller`, `TTerminal`              | `Scroller`, `Terminal`                      |
| `TEditor`, `TMemo`, `TFileEditor`     | `Editor`, `Memo`, `FileEditor` (многоуровневая отмена, поиск/замена) |
| `TEditWindow`, `TIndicator`           | `EditWindow`, `Indicator`                   |
| `TFileDialog`, `TFileList`, `TFileInfoPane` | `FileDialog`, `FileList`, `FileInfoPane` |
| `TChDirDialog`, `TDirListBox`         | `ChDirDialog`, `DirListBox`                 |
| `TOutlineViewer`, `TOutline`, `TNode` | `Outline`, `Node`                           |
| `TColorDialog`, `TColorSelector`, `TColorDisplay`, `TColorGroup`, `TColorItem` | `ColorDialog`, `ColorSelector`, `ColorDisplay`, `ColorGroup`, `ColorItem` |
| `THelpFile`, `THelpTopic`, `THelpViewer`, `THelpWindow` | `HelpFile`, `HelpTopic`, `HelpViewer`, `HelpWindow` |
| утилита `tvhc`                        | пример `tvhc` (проверка + константы `HC_*`) |
| `TMultiCheckBoxes`                    | `Cluster::multi_check_boxes`                |
| `TInputLong`, `TColoredText`, `TCombo` (Dialog Design) | `InputLong`, `StaticText::colored`, `Combo` |
| `TColorAttr`, `TColorXTerm`           | `Attr` (`u32`): байт BIOS + 256 цветов xterm |

## Чего ещё нет (следующие этапы)

* `TMonoSelector` (выбор цвета в монохромном режиме);
* `TStreamable` / ресурсы (сохранение объектов в поток);
* TrueColor (24-битные RGB-цвета; 256 цветов xterm уже есть);
* кодировки, отличные от UTF-8, в редакторе; время в `FileInfoPane` пока в UTC.

## Системный буфер обмена

`Ctx::clipboard` связан с буфером обмена ОС (строки ввода и редактор
используют его для Ctrl+Ins / Shift+Ins / Shift+Del):

| ОС | Как |
|----|-----|
| Windows | WinAPI, `CF_UNICODETEXT` |
| macOS | `pbcopy` / `pbpaste` |
| Linux / BSD, Wayland | `wl-copy` / `wl-paste` |
| Linux / BSD, X11 | `xclip` или `xsel` |
| нет ничего из этого (например, SSH) | копирование — последовательностью терминала OSC 52, вставка — из внутреннего буфера |

## tvdemo

`examples/tvdemo/` — порт демонстрационной программы целиком: меню «≡»
(Alt+Пробел), About, Puzzle, Calendar, Ascii Table, Calculator, Event Viewer
(Alt+0), просмотр файлов, смена каталога, выход в оболочку (DOS Shell),
настройки мыши, цвета, узор фона, сохранение/восстановление рабочего стола,
часы и счётчик памяти, контекстная справка по F1 из оригинального
`demohelp.txt`. Отличия: «Video mode» переключает палитры (цветная, ч/б,
монохромная), а не 25/43 строки; рабочий стол сохраняется в простой
текстовый формат вместо потоков объектов `TStreamable`.

Для этого в библиотеке появились: `Program::dos_shell`, `Program::double_delay`
и `reverse_buttons`, `Program::event_monitor` + `View::monitor_event`,
`Ctx::with_program` (действие с программой из любого представления),
`Desktop::set_pattern`, `platform::time` (местное время),
`Menu::submenu_key` и `Menu::help`.

## Справка

Исходник справки пишется в формате `tvhc` (см. `examples/demohelp.txt`) и
читается программой напрямую — двоичный `.hlp` не нужен:

```rust
let help = Rc::new(HelpFile::parse(include_str!("demohelp.txt"))?);
// по F1:
let ctx = app.help_ctx();
app.exec_dialog(HelpWindow::new(help.clone(), ctx));
```

Палитра приложения (`Program::palette`) теперь обычный `Vec<u8>`; её меняет
`ColorDialog`, а сохранить её между запусками можно как угодно (например, в файл).

## Модальные окна изнутри представлений

В C++ любое представление могло вызвать `owner->execView(dialog)` или
`messageBox(...)` прямо из `handleEvent`. Здесь для этого есть
`ctx.exec(Box::new(dialog), Some(self.vs.id), tag)`: программа запускает окно
после текущего события, а результат приходит в `View::exec_result(tag, cmd, view, ctx)`.
Так работают сообщения валидаторов, окно истории, вопрос «Сохранить изменения?»
и диалоги поиска/замены в редакторе. `View::valid(cmd, ctx)` тоже получает контекст.

## Главное отличие от C++

В C++ представление хранит указатель на владельца и вызывает его напрямую
(`owner->setCurrent(this)`, `message(owner, ...)`, `destroy(this)`).
Здесь представления владельца не знают: они пишут запрос в контекст `Ctx`
(`ctx.select(id)`, `ctx.message(cmd, info)`, `ctx.close(id)`), а владелец
выполняет его сразу после возврата из `handle_event`. Модальные диалоги
запускаются из обработчика приложения: `let (cmd, dlg) = app.exec_dialog(dlg);`.

## Мини-пример

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

fn main() -> std::io::Result<()> {
    let menu = Menu::new().submenu("~Ф~айл", Menu::new()
        .item_key("~П~ривет", cm::NEW, kb::F2, "F2")
        .item_key("~В~ыход", cm::QUIT, kb::ALT_X, "Alt-X"));
    let status = vec![StatusDef::all(vec![StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT)])];
    Program::with_terminal(Some(menu), Some(status))?.run(&mut App);
    Ok(())
}
```

Горячие буквы работают в любой раскладке: на Windows Alt+буква приходит
латинской буквой клавиши (Alt+Ф — это Alt+A), и меню, метки, кнопки и
флажки сравнивают горячие буквы с учётом пар QWERTY ↔ ЙЦУКЕН. Встроенные
надписи библиотеки (окна сообщений, диалог файлов) переводятся на русский
вызовом `locale::set_language(locale::Language::Ru)`.
