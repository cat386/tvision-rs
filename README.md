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
cargo test                  # тесты на «невидимом» терминале
```

## Что уже перенесено (этап 1)

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

## Чего ещё нет (следующие этапы)

* `TEditor`, `TMemo`, `TFileEditor`, `TEditWindow`, `TIndicator`;
* `TFileDialog`, `TChDirDialog`, `TDirListBox`, `TFileList`;
* `THistory` (история строк ввода), `TValidator` и его наследники (сейчас — замыкание `InputLine::validator`);
* `TScroller`, `TTextDevice`/`TTerminal`, `TOutline`, `THelpWindow`, `TColorDialog`;
* `TStreamable` / ресурсы (сохранение объектов в поток);
* 256 цветов и TrueColor (`TColorAttr` с RGB), системный буфер обмена.

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
