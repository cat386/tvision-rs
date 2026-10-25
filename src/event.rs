//! События: клавиатура, мышь, команды и широковещательные сообщения.
//!
//! Аналог `TEvent` из `include/tvision/system.h`. В C++ событие — это
//! структура с полем `what` (evKeyDown, evMouseDown, evCommand, ...) и union
//! с данными. В Rust это перечисление [`Event`].
//!
//! «Обработать событие» в tvision означает вызвать `clearEvent(event)`, то есть
//! превратить его в `evNothing`. Здесь для этого есть [`Event::clear`].

use crate::geometry::Point;
use crate::view::ViewId;

/// Модификаторы клавиатуры (Shift, Ctrl, Alt).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Mods {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl Mods {
    pub const NONE: Mods = Mods { shift: false, ctrl: false, alt: false };
    pub const SHIFT: Mods = Mods { shift: true, ctrl: false, alt: false };
    pub const CTRL: Mods = Mods { shift: false, ctrl: true, alt: false };
    pub const ALT: Mods = Mods { shift: false, ctrl: false, alt: true };
    pub const CTRL_SHIFT: Mods = Mods { shift: true, ctrl: true, alt: false };
}

/// Клавиша. Вместо 16-битных скан-кодов DOS (`kbF1 = 0x3B00` и т.п.)
/// используется понятное перечисление, а сочетания задаются модификаторами.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    /// Печатный символ (буква, цифра, знак).
    Char(char),
    Enter,
    Esc,
    Tab,
    /// Shift+Tab (в некоторых терминалах приходит отдельной последовательностью).
    BackTab,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PgUp,
    PgDn,
    Ins,
    Del,
    /// Функциональная клавиша F1..F12.
    F(u8),
}

/// Нажатие клавиши (аналог `KeyDownEvent`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub key: Key,
    pub mods: Mods,
}

impl KeyEvent {
    pub const fn new(key: Key, mods: Mods) -> Self {
        KeyEvent { key, mods }
    }

    /// Совпадает ли нажатие с заданной клавишей и модификаторами.
    /// Для букв регистр не учитывается.
    pub fn is(&self, other: KeyEvent) -> bool {
        match (self.key, other.key) {
            (Key::Char(a), Key::Char(b)) => {
                crate::text::to_upper(a) == crate::text::to_upper(b) && self.mods.alt == other.mods.alt && self.mods.ctrl == other.mods.ctrl
            }
            // Shift+Tab может прийти как BackTab.
            (Key::BackTab, Key::Tab) | (Key::Tab, Key::BackTab) => other.mods.shift || self.mods.shift,
            (a, b) => a == b && self.mods == other.mods,
        }
    }

    /// Печатный символ без Ctrl/Alt (то, что нужно вставлять в строку ввода).
    pub fn text_char(&self) -> Option<char> {
        match self.key {
            Key::Char(c) if !self.mods.ctrl && !self.mods.alt => Some(c),
            _ => None,
        }
    }

    /// Если это Alt+буква — возвращает букву в верхнем регистре.
    /// Аналог проверки `keyCode == getAltCode(c)`.
    pub fn alt_char(&self) -> Option<char> {
        match self.key {
            Key::Char(c) if self.mods.alt && !self.mods.ctrl => Some(crate::text::to_upper(c)),
            _ => None,
        }
    }

    /// Человекочитаемое имя сочетания, например `"Alt+X"` или `"F10"`.
    pub fn name(&self) -> String {
        let mut s = String::new();
        if self.mods.ctrl {
            s.push_str("Ctrl+");
        }
        if self.mods.alt {
            s.push_str("Alt+");
        }
        if self.mods.shift {
            s.push_str("Shift+");
        }
        match self.key {
            Key::Char(c) => s.push(crate::text::to_upper(c)),
            Key::F(n) => s.push_str(&format!("F{}", n)),
            k => s.push_str(&format!("{:?}", k)),
        }
        s
    }
}

/// Событие мыши (аналог `MouseEventType`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MouseEvent {
    /// Позиция курсора мыши. ВНИМАНИЕ: при доставке в представление координаты
    /// заданы в системе координат **владельца** этого представления
    /// (то есть в той же системе, что и `ViewState::origin`).
    /// Перевести в локальные координаты можно методом `ViewState::make_local`.
    pub pos: Point,
    /// Нажатые кнопки: бит 0 — левая, бит 1 — правая, бит 2 — средняя.
    pub buttons: u8,
    /// Двойной щелчок (аналог `meDoubleClick`).
    pub double: bool,
    /// Модификаторы клавиатуры в момент события.
    pub mods: Mods,
    /// Прокрутка колесом: -1 — вверх, +1 — вниз, 0 — нет.
    pub wheel: i8,
}

pub const MB_LEFT: u8 = 1;
pub const MB_RIGHT: u8 = 2;
pub const MB_MIDDLE: u8 = 4;

/// Дополнительные данные команды или сообщения.
/// В C++ это `void* infoPtr` / `long infoLong`; здесь — безопасный вариант.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Info {
    #[default]
    None,
    /// Идентификатор представления (вместо указателя `infoPtr` на `TView`).
    Id(ViewId),
    /// Целое число (`infoLong`, `infoInt`).
    Int(i64),
    /// Строка.
    Str(String),
    /// Текст для конкретного представления (например, выбор из истории ввода).
    Text { id: ViewId, text: String },
    /// Позиция в редакторе для индикатора строки/столбца.
    Indicator { id: ViewId, line: usize, col: usize, modified: bool },
    /// Новое значение полосы выполнения ([`crate::ProgressBar`]).
    Progress { id: ViewId, value: i64 },
    /// Параметры полосы прокрутки: id полосы, значение, мин, макс, шаг страницы, шаг строки.
    Scroll { id: ViewId, value: i32, min: i32, max: i32, page: i32, arrow: i32 },
}

/// Команда или широковещательное сообщение (аналог `MessageEvent`).
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub command: u16,
    pub info: Info,
}

impl Message {
    pub fn new(command: u16) -> Self {
        Message { command, info: Info::None }
    }
    pub fn with(command: u16, info: Info) -> Self {
        Message { command, info }
    }
}

/// Событие. Аналог `TEvent`.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum Event {
    /// Нет события / событие уже обработано (`evNothing`).
    #[default]
    Nothing,
    /// Нажатие клавиши (`evKeyDown`).
    KeyDown(KeyEvent),
    /// Кнопка мыши нажата (`evMouseDown`).
    MouseDown(MouseEvent),
    /// Кнопка мыши отпущена (`evMouseUp`).
    MouseUp(MouseEvent),
    /// Мышь сдвинулась (`evMouseMove`).
    MouseMove(MouseEvent),
    /// Кнопка удерживается — повторяющееся событие (`evMouseAuto`).
    MouseAuto(MouseEvent),
    /// Колесо мыши (`evMouseWheel`).
    MouseWheel(MouseEvent),
    /// Команда (`evCommand`) — идёт по цепочке фокуса.
    Command(Message),
    /// Широковещательное сообщение (`evBroadcast`) — получают все дочерние представления.
    Broadcast(Message),
}

impl Event {
    /// Пометить событие как обработанное (аналог `clearEvent`).
    pub fn clear(&mut self) {
        *self = Event::Nothing;
    }

    pub fn is_nothing(&self) -> bool {
        matches!(self, Event::Nothing)
    }

    /// Событие мыши — «позиционное»: доставляется представлению под курсором.
    pub fn is_mouse(&self) -> bool {
        matches!(self, Event::MouseDown(_) | Event::MouseUp(_) | Event::MouseMove(_) | Event::MouseAuto(_) | Event::MouseWheel(_))
    }

    /// «Фокусное» событие: доставляется по цепочке выбранных представлений.
    pub fn is_focused(&self) -> bool {
        matches!(self, Event::KeyDown(_) | Event::Command(_))
    }

    /// Данные мыши, если это событие мыши.
    pub fn mouse(&self) -> Option<&MouseEvent> {
        match self {
            Event::MouseDown(m) | Event::MouseUp(m) | Event::MouseMove(m) | Event::MouseAuto(m) | Event::MouseWheel(m) => Some(m),
            _ => None,
        }
    }

    pub fn mouse_mut(&mut self) -> Option<&mut MouseEvent> {
        match self {
            Event::MouseDown(m) | Event::MouseUp(m) | Event::MouseMove(m) | Event::MouseAuto(m) | Event::MouseWheel(m) => Some(m),
            _ => None,
        }
    }

    /// Номер команды, если это `Command`.
    pub fn command(&self) -> Option<u16> {
        match self {
            Event::Command(m) => Some(m.command),
            _ => None,
        }
    }

    /// Номер команды, если это `Broadcast`.
    pub fn broadcast(&self) -> Option<&Message> {
        match self {
            Event::Broadcast(m) => Some(m),
            _ => None,
        }
    }

    /// Нажатая клавиша, если это `KeyDown`.
    pub fn key(&self) -> Option<KeyEvent> {
        match self {
            Event::KeyDown(k) => Some(*k),
            _ => None,
        }
    }

    pub fn command_event(command: u16) -> Event {
        Event::Command(Message::new(command))
    }
}

/// Стандартные команды. Номера совпадают с оригиналом (`views.h`, `dialogs.h`,
/// `app.h`), чтобы облегчить перенос программ.
///
/// Команды 0..=255 можно запрещать (`Ctx::disable_command`), команды больше 255
/// запретить нельзя — как и в оригинале.
pub mod cm {
    pub const VALID: u16 = 0;
    pub const QUIT: u16 = 1;
    pub const ERROR: u16 = 2;
    pub const MENU: u16 = 3;
    pub const CLOSE: u16 = 4;
    pub const ZOOM: u16 = 5;
    pub const RESIZE: u16 = 6;
    pub const NEXT: u16 = 7;
    pub const PREV: u16 = 8;
    pub const HELP: u16 = 9;

    // Команды диалогов
    pub const OK: u16 = 10;
    pub const CANCEL: u16 = 11;
    pub const YES: u16 = 12;
    pub const NO: u16 = 13;
    pub const DEFAULT: u16 = 14;

    // Стандартные команды приложения
    pub const CUT: u16 = 20;
    pub const COPY: u16 = 21;
    pub const PASTE: u16 = 22;
    pub const UNDO: u16 = 23;
    pub const CLEAR: u16 = 24;
    pub const TILE: u16 = 25;
    pub const CASCADE: u16 = 26;
    pub const REDO: u16 = 27;

    pub const NEW: u16 = 30;
    pub const OPEN: u16 = 31;
    pub const SAVE: u16 = 32;
    pub const SAVE_AS: u16 = 33;
    pub const SAVE_ALL: u16 = 34;
    pub const CH_DIR: u16 = 35;
    pub const DOS_SHELL: u16 = 36;
    pub const CLOSE_ALL: u16 = 37;

    // Широковещательные сообщения
    pub const RECEIVED_FOCUS: u16 = 50;
    pub const RELEASED_FOCUS: u16 = 51;
    pub const COMMAND_SET_CHANGED: u16 = 52;
    pub const SCROLL_BAR_CHANGED: u16 = 53;
    pub const SCROLL_BAR_CLICKED: u16 = 54;
    pub const SELECT_WINDOW_NUM: u16 = 55;
    pub const LIST_ITEM_SELECTED: u16 = 56;
    pub const RECORD_HISTORY: u16 = 60;
    pub const GRAB_DEFAULT: u16 = 61;
    pub const RELEASE_DEFAULT: u16 = 62;
    /// Таблица: курсор перешёл в другую ячейку (`Info::Id` таблицы).
    pub const TABLE_CURSOR_MOVED: u16 = 57;
    /// Таблица: пользователь изменил значение ячейки (`Info::Id` таблицы).
    pub const TABLE_CELL_EDITED: u16 = 58;

    /// Внутреннее сообщение этого порта: «установить параметры полосы прокрутки».
    /// В C++ список вызывал `vScrollBar->setParams(...)` напрямую.
    /// Номер выбран выше 255, чтобы не пересекаться с командами пользователя
    /// из нижнего диапазона; пользовательские команды рекомендуется
    /// нумеровать от 100 до 255 или от 1000 и выше.
    pub const SCROLL_BAR_SET: u16 = 0xFF10;
    /// Внутреннее: установить текст строки ввода из истории (`Info::Text`).
    pub const HISTORY_SET: u16 = 0xFF11;
    /// Внутреннее: обновить индикатор редактора (`Info::Indicator`).
    pub const INDICATOR_SET: u16 = 0xFF12;
    /// Установить значение полосы выполнения (`Info::Progress`).
    pub const PROGRESS_SET: u16 = 0xFF13;

    // Команды редактора (`editors.h`)
    pub const FIND: u16 = 82;
    pub const REPLACE: u16 = 83;
    pub const SEARCH_AGAIN: u16 = 84;
    /// Обновить заголовок окна редактора (имя файла).
    pub const UPDATE_TITLE: u16 = 523;
    pub const SELECT_ALL: u16 = 524;

    // Команды файловых диалогов (`stddlg.h`)
    pub const FILE_OPEN: u16 = 1001;
    pub const FILE_REPLACE: u16 = 1002;
    pub const FILE_CLEAR: u16 = 1003;
    pub const FILE_INIT: u16 = 1004;
    pub const CHANGE_DIR: u16 = 1005;
    pub const REVERT: u16 = 1006;
    pub const FILE_FOCUSED: u16 = 102;
    pub const FILE_DOUBLE_CLICKED: u16 = 103;
}

/// Удобные константы для частых сочетаний клавиш (аналог `kbXXX` из `tkeys.h`).
pub mod kb {
    use super::{Key, KeyEvent, Mods};

    pub const fn key(k: Key) -> KeyEvent {
        KeyEvent::new(k, Mods::NONE)
    }
    pub const fn alt(c: char) -> KeyEvent {
        KeyEvent::new(Key::Char(c), Mods::ALT)
    }
    pub const fn ctrl(c: char) -> KeyEvent {
        KeyEvent::new(Key::Char(c), Mods::CTRL)
    }
    pub const fn f(n: u8) -> KeyEvent {
        KeyEvent::new(Key::F(n), Mods::NONE)
    }
    pub const fn alt_f(n: u8) -> KeyEvent {
        KeyEvent::new(Key::F(n), Mods::ALT)
    }
    pub const fn ctrl_f(n: u8) -> KeyEvent {
        KeyEvent::new(Key::F(n), Mods::CTRL)
    }
    pub const fn shift_f(n: u8) -> KeyEvent {
        KeyEvent::new(Key::F(n), Mods::SHIFT)
    }

    pub const ENTER: KeyEvent = key(Key::Enter);
    pub const ESC: KeyEvent = key(Key::Esc);
    pub const TAB: KeyEvent = key(Key::Tab);
    pub const SHIFT_TAB: KeyEvent = KeyEvent::new(Key::Tab, Mods::SHIFT);
    pub const F1: KeyEvent = f(1);
    pub const F2: KeyEvent = f(2);
    pub const F3: KeyEvent = f(3);
    pub const F5: KeyEvent = f(5);
    pub const F6: KeyEvent = f(6);
    pub const F10: KeyEvent = f(10);
    pub const ALT_X: KeyEvent = alt('X');
    pub const ALT_F3: KeyEvent = alt_f(3);
    pub const CTRL_F5: KeyEvent = ctrl_f(5);
    pub const SHIFT_F6: KeyEvent = shift_f(6);
}
