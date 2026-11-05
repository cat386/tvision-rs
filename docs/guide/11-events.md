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
