# tvision-rs

Порт библиотеки текстового интерфейса **Turbo Vision** ([magiblot/tvision](https://github.com/magiblot/tvision)) с C++ на Rust.

* без внешних зависимостей (работа с терминалом реализована внутри, как в оригинале);
* Linux / macOS / BSD (termios + ANSI) и Windows 10+ (Console API + VT);
* мышь, Unicode (кириллица, широкие символы), 16 цветов с оригинальными палитрами;
* комментарии в коде — на русском.

## Запуск примеров

```
cargo run --example hello   # порт hello.cpp
cargo run --example demo    # окна, меню, диалог со всеми элементами, список, контекстное меню
cargo run --example tvedit  # порт примера tvedit: многооконный редактор файлов
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
| `messageBox`, `inputBox`              | `message_box`, `message_box_rect`, `input_box` |
| `platform/*`                          | `platform::{unix, windows, ansi}`           |
| `TValidator`, `TPXPictureValidator`, `TFilterValidator`, `TRangeValidator`, `TStringLookupValidator` | трейт `Validator`, `PictureValidator`, `FilterValidator`, `RangeValidator`, `StringLookupValidator` |
| `THistory`, `THistoryWindow`, `historyAdd` | `History`, `HistoryWindow`, `history_add` |
| `TScroller`, `TTerminal`              | `Scroller`, `Terminal`                      |
| `TEditor`, `TMemo`, `TFileEditor`     | `Editor`, `Memo`, `FileEditor` (многоуровневая отмена, поиск/замена) |
| `TEditWindow`, `TIndicator`           | `EditWindow`, `Indicator`                   |
| `TFileDialog`, `TFileList`, `TFileInfoPane` | `FileDialog`, `FileList`, `FileInfoPane` |
| `TChDirDialog`, `TDirListBox`         | `ChDirDialog`, `DirListBox`                 |

## Чего ещё нет (следующие этапы)

* `TOutline` (дерево), `THelpWindow` и компилятор справки `tvhc`, `TColorDialog`;
* `TStreamable` / ресурсы (сохранение объектов в поток);
* 256 цветов и TrueColor (`TColorAttr` с RGB), системный буфер обмена;
* кодировки, отличные от UTF-8, в редакторе; время в `FileInfoPane` пока в UTC.

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

Замечание для Windows: Alt+буква сопоставляется по латинской клавише
(Alt+F работает и в русской раскладке), поэтому горячие клавиши вида `~Ф~айл`
срабатывают в меню и диалогах по самой букве, а по Alt — только на Unix.
