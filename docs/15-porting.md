# 15. Перенос программ с C++ и Pascal

Эта глава — для тех, кто знает Turbo Vision (Borland C++ / Turbo Pascal
или magiblot/tvision) и переносит программу на Rust.

## Соответствие типов

| C++ / Pascal | Rust |
|--------------|------|
| `TPoint`, `TRect` | `Point`, `Rect` (`b` не включается — как в оригинале) |
| `TView` | трейт `View` + поле `vs: ViewState` |
| `TGroup` | `Group` |
| `TProgram`, `TApplication` | `Program` + трейт `AppHandler` |
| `TDeskTop`, `TBackground` | `Desktop`, `Background` |
| `TWindow`, `TFrame` | `Window` (рамку рисует само окно; `Frame` — функции рисования) |
| `TDialog` | `Dialog` |
| `TButton` | `Button` |
| `TStaticText`, `TLabel` | `StaticText`, `Label` |
| `TInputLine` | `InputLine` |
| `TCheckBoxes`, `TRadioButtons`, `TMultiCheckBoxes` | `Cluster::check_boxes`, `Cluster::radio_buttons`, `Cluster::multi_check_boxes` |
| `TListViewer`, `TListBox`, `TSortedListBox` | `ListBox` |
| — (в оригинале нет) | `Table`, `TableModel`, `VecTableModel`, `FnTableModel` — таблица ([глава 7](07-dialogs.md#таблица-table)) |
| `TScrollBar`, `TScroller` | `ScrollBar`, `Scroller` (встраиваемый движок) |
| `TTextDevice`, `TTerminal` | `Terminal` |
| `THistory`, `THistoryWindow`, `historyAdd` | `History`, `HistoryWindow`, `history_add` |
| `TMenuBar`, `TMenuBox`, `TMenuPopup`, `TMenu`, `TMenuItem`, `TSubMenu` | `MenuBar`, `MenuSession`, `Menu`, `MenuItem` |
| `TStatusLine`, `TStatusDef`, `TStatusItem` | `StatusLine`, `StatusDef`, `StatusItem` |
| `messageBox`, `messageBoxRect`, `inputBox` | `message_box`, `message_box_rect`, `input_box` |
| `TValidator`, `TPXPictureValidator`, `TFilterValidator`, `TRangeValidator`, `TStringLookupValidator` | трейт `Validator`, `PictureValidator`, `FilterValidator`, `RangeValidator`, `StringLookupValidator` (+ `FnValidator`) |
| `TEditor`, `TMemo`, `TFileEditor`, `TEditWindow`, `TIndicator` | `Editor`, `Memo`, `FileEditor`, `EditWindow`, `Indicator` |
| `TFileDialog`, `TChDirDialog`, `TFileList`, `TDirListBox`, `TFileInfoPane` | `FileDialog`, `ChDirDialog`, `FileList`, `DirListBox`, `FileInfoPane` |
| `TOutline`, `TNode` | `Outline`, `Node` |
| `TColorDialog` и др. | `ColorDialog`, `ColorGroup`, `ColorItem`, `ColorSelector`, `ColorDisplay` |
| `THelpFile`, `THelpWindow`, утилита `tvhc` | `HelpFile` (читает исходник напрямую), `HelpWindow`, пример `tvhc` |
| `TInputLong`, `TColoredText`, `TCombo` (Dialog Design) | `InputLong`, `StaticText::colored`, `Combo` |
| `TEvent` | `Event` (перечисление) |
| `TDrawBuffer` | `DrawBuffer` |
| `TPalette`, `cpXXX` | `&'static [u8]`, `color::palettes::*` (палитра приложения — `Vec<Attr>`) |
| `TColorAttr`, `TColorBIOS`, `TColorXTerm`, `TAttrPair` | `Attr` (`u32`: байт BIOS + номера xterm-256), `color::attr`, `color::attr256`, `AttrPair` (`u64`) |
| `TCommandSet` | `CommandSet` (через `enable_command`/`disable_command`) |
| `TStreamable`, ресурсы | нет (сохраняйте данные сами: serde, свой формат) |

## Главные отличия

### 1. Нет указателя на владельца

Всё, что делалось через `owner`, делается через `Ctx`
([глава 2](02-architecture.md#представление-не-знает-своего-владельца)):

```text
C++                                     Rust
select();                               ctx.select(self.vs.id);
owner->selectNext(False);               ctx.select_next(true);
message(owner, evBroadcast, cm, this);  ctx.message(cm, Info::Id(self.vs.id));
putEvent(e);                            ctx.put_event(e);
close();                                ctx.close(self.vs.id);
endModal(cmOK);                         ctx.end_modal(cm::OK);
owner->execView(d) / messageBox(...)    ctx.exec(Box::new(d), Some(self.vs.id), tag)  →  exec_result
TProgram::deskTop->...                  ctx.with_program(|app| app.desktop_mut()...)
```

Указатели на другие представления (`link`, `vScrollBar`, `hScrollBar`,
`indicator`) — это `ViewId`.

### 2. Наследование → вложение и трейт

```text
class TMyView : public TView {           struct MyView { vs: ViewState, … }
    virtual void draw();                 impl View for MyView {
    virtual void handleEvent(TEvent&);       tvision::impl_view_basics!();
    virtual TPalette& getPalette() const;    fn draw(&self, dc: &mut DrawCtx) { … }
};                                           fn handle_event(&mut self, ev, ctx) { … }
                                             fn palette(&self) -> &'static [u8] { … }
                                         }
```

Вызов «родительского» `TView::handleEvent(event)` →
`view::default_handle_event(&self.vs, ev, ctx)`; `TGroup::handleEvent` →
`self.group.handle_group_event(ev, ctx)`; `TWindow::handleEvent` →
`self.win.handle_window_event(ev, ctx)`. Наследник `TInputLine` —
структура с полем `il: InputLine` и макросом `impl_view_basics_field!(il)`
([глава 4](04-views.md#наследование-через-вложение)).

### 3. Обработчик приложения

```text
class TMyApp : public TApplication {     struct MyApp;
    void handleEvent(TEvent& e) {        impl AppHandler for MyApp {
        TApplication::handleEvent(e);        fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if (e.what == evCommand) …               match ev.command() { … }
    }                                        }
    static TMenuBar* initMenuBar(TRect);     fn idle(&mut self, app: &mut Program) { … }
    static TStatusLine* initStatusLine(TRect); }
};                                       Program::with_terminal(Some(menu), Some(status))?.run(&mut MyApp);
```

`AppHandler::handle_event` вызывается **после** того, как событие прошло
по дереву (как вызов `TApplication::handleEvent(e)` в начале
переопределённого метода). Во время модального диалога он не вызывается —
используйте `Program::modal_hook` ([глава 5](05-windows.md#modal_hook-команды-во-время-модального-цикла)).

### 4. Модальный запуск возвращает объект

```text
TDialog* d = new TDialog(...);           let mut d = Dialog::new(...);
d->setData(&rec);                        d.set_data(&rec.to_value());
ushort c = deskTop->execView(d);         let (c, d) = app.exec_dialog(d);
if (c == cmOK) d->getData(&rec);         if c == cm::OK { rec = Rec::from_value(&d.get_data()); }
destroy(d);                              // d освобождается сам
```

`validView` не нужен: `insert_window` вернёт 0, если `valid(cm::VALID)`
отказал.

### 5. Данные — `Value`, а не блок памяти

`dataSize`/`getData(void*)`/`setData(void*)` заменены на
`has_data`/`get_data() -> Value`/`set_data(&Value)`. Запись данных
диалога — `Value::List` значений элементов в порядке вставки
([глава 7](07-dialogs.md#обмен-данными)). `TListBoxRec` передавал
коллекцию и номер — здесь только номер (`Value::Int`), строки задаются
`set_items`.

### 6. Клавиши — структура, а не скан-код

```text
case kbF1:                    k.is(kb::F1)
case kbAltX:                  k.is(kb::ALT_X)
case kbCtrlIns:               k.is(KeyEvent::new(Key::Ins, Mods::CTRL))
event.keyDown.charScan.charCode   k.text_char()
getAltCode(c) == keyCode      k.alt_char() == Some(c)
```

### 7. Мышь — захват вместо вложенного цикла

```text
do { … } while (mouseEvent(event, evMouseMove));
```

заменяется на `ctx.grab_mouse(id)` в `MouseDown` и обработку
`MouseMove`/`MouseAuto`/`MouseUp` в следующих вызовах `handle_event`
([глава 3](03-events.md#мышь)). Координаты события — в системе владельца:
`makeLocal(event.mouse.where)` → `self.vs.make_local(m.pos)`.

### 8. Отрисовка — всегда целиком

`drawView()`, `redraw()`, `lock()/unlock()`, `ofBuffered`,
`exposed()` не нужны: экран перерисовывается на каждом шаге, в терминал
уходят только изменения. Просто меняйте поля.

### 9. Строки — UTF-8

Нет `char*`, `strncpy`, `maxLen + 1`: `InputLine::new(r, max_len)` — длина
в символах, текст — `String`. Кириллица и широкие символы работают везде.

### 10. Справка без компиляции

Исходник для `tvhc` читается `HelpFile::parse(include_str!("help.txt"))`;
`.hlp` и `.h` не нужны (константы даёт `context_constants()` или пример
`tvhc`).

## Мелочи, которые легко пропустить

* Метку (`TLabel`) вставляйте **после** связанного элемента — нужен его id.
* Значку истории нужен номер и у строки ввода: `il.history_id = Some(n)`
  (в C++ его хранил только `THistory`).
* Пользовательские команды: 100–255 (запрещаемые) или ≥ 1000. Номера
  `0xFF10`–`0xFF12`, `0xFF20`, `0xFF21` заняты внутренними сообщениями.
* `ofCentered` центрирует при вставке — для `Dialog::centered` это уже
  сделано.
* Кнопка `bfBroadcast` рассылает сообщение соседям (`ctx.message`), а не
  владельцу-диалогу лично.
* У окна по умолчанию `wfMove | wfGrow | wfClose | wfZoom`, у диалога —
  `wfMove | wfClose`.
* `TApplication::dosShell` → `Program::dos_shell(banner)`.
* `TScreen::screenMode`, 25/43/50 строк — размер задаёт окно терминала.
* `TStreamable` (сохранение рабочего стола в поток) не перенесён: сохраняйте
  состояние в своём формате (так сделано в примере `tvdemo`).

## Пример: `hello.cpp`

```text
// C++
class THelloApp : public TApplication {
public:
    THelloApp();
    virtual void handleEvent(TEvent& event);
    static TMenuBar* initMenuBar(TRect r);
    static TStatusLine* initStatusLine(TRect r);
private:
    void greetingBox();
};
```

```rust
// Rust: examples/hello.rs
use tvision::*;

const GREET_THEM_CMD: u16 = 100;

struct HelloApp;

impl HelloApp {
    fn greeting_box(&self, app: &mut Program) {
        let mut d = Dialog::new(Rect::new(25, 5, 55, 16), "Hello, World!");
        d.add(StaticText::new(Rect::new(3, 5, 15, 6), "How are you?"));
        d.add(Button::new(Rect::new(16, 2, 28, 4), "Terrific", cm::CANCEL, bf::NORMAL));
        d.add(Button::new(Rect::new(16, 4, 28, 6), "Ok", cm::CANCEL, bf::NORMAL));
        d.add(Button::new(Rect::new(16, 6, 28, 8), "Lousy", cm::CANCEL, bf::NORMAL));
        d.add(Button::new(Rect::new(16, 8, 28, 10), "Cancel", cm::CANCEL, bf::NORMAL));
        app.exec_dialog(d);
    }
}

impl AppHandler for HelloApp {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(GREET_THEM_CMD) {
            self.greeting_box(app);
            ev.clear();
        }
    }
}

fn main() -> std::io::Result<()> {
    let menu = Menu::new().submenu(
        "~H~ello",
        Menu::new().item_key("~G~reeting...", GREET_THEM_CMD, kb::alt('G'), "Alt-G").separator().item_key("E~x~it", cm::QUIT, kb::ALT_X, "Alt-X"),
    );
    let status = vec![StatusDef::all(vec![StatusItem::new("~Alt-X~ Exit", kb::ALT_X, cm::QUIT), StatusItem::new("", kb::F10, cm::MENU)])];
    let mut app = Program::with_terminal(Some(menu), Some(status))?;
    app.run(&mut HelloApp);
    Ok(())
}
```
