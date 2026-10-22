//! Консоль Windows (Windows 10 и новее).
//!
//! Аналог `platform/win32con.cpp` оригинала:
//! * ввод — через `ReadConsoleInputW` (клавиатура, мышь, изменение размера);
//! * вывод — VT-последовательности (`ENABLE_VIRTUAL_TERMINAL_PROCESSING`),
//!   записываемые через `WriteConsoleW` в UTF-16, чтобы не зависеть от
//!   кодовой страницы консоли.
//!
//! Функции kernel32 объявлены вручную, без крейтов `winapi`/`windows-sys`.

#![allow(non_snake_case, clippy::upper_case_acronyms)]

use std::io;
use std::os::raw::c_void;
use std::time::Duration;

use super::ansi::{AnsiWriter, ENTER_SEQ, LEAVE_SEQ};
use super::{Backend, RawEvent};
use crate::drawbuf::ScreenBuffer;
use crate::event::{Key, KeyEvent, Mods, MB_LEFT, MB_MIDDLE, MB_RIGHT};
use crate::geometry::Point;

type HANDLE = *mut c_void;
type BOOL = i32;
type DWORD = u32;

const STD_INPUT_HANDLE: DWORD = -10i32 as DWORD;
const STD_OUTPUT_HANDLE: DWORD = -11i32 as DWORD;
const WAIT_OBJECT_0: DWORD = 0;

// Режимы ввода
const ENABLE_WINDOW_INPUT: DWORD = 0x0008;
const ENABLE_MOUSE_INPUT: DWORD = 0x0010;
const ENABLE_EXTENDED_FLAGS: DWORD = 0x0080;
// Режимы вывода
const ENABLE_PROCESSED_OUTPUT: DWORD = 0x0001;
const ENABLE_VIRTUAL_TERMINAL_PROCESSING: DWORD = 0x0004;
const DISABLE_NEWLINE_AUTO_RETURN: DWORD = 0x0008;

// Типы событий INPUT_RECORD
const KEY_EVENT: u16 = 0x0001;
const MOUSE_EVENT: u16 = 0x0002;
const WINDOW_BUFFER_SIZE_EVENT: u16 = 0x0004;

// Состояние клавиш-модификаторов (dwControlKeyState)
const RIGHT_ALT_PRESSED: DWORD = 0x0001;
const LEFT_ALT_PRESSED: DWORD = 0x0002;
const RIGHT_CTRL_PRESSED: DWORD = 0x0004;
const LEFT_CTRL_PRESSED: DWORD = 0x0008;
const SHIFT_PRESSED: DWORD = 0x0010;

// Флаги событий мыши (dwEventFlags)
const MOUSE_WHEELED: DWORD = 0x0004;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct COORD {
    X: i16,
    Y: i16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct SMALL_RECT {
    Left: i16,
    Top: i16,
    Right: i16,
    Bottom: i16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct CONSOLE_SCREEN_BUFFER_INFO {
    dwSize: COORD,
    dwCursorPosition: COORD,
    wAttributes: u16,
    srWindow: SMALL_RECT,
    dwMaximumWindowSize: COORD,
}

/// `INPUT_RECORD`: тип события + объединение из 16 байт.
/// Поля объединения разбираем вручную (см. `read_key`/`read_mouse`).
#[repr(C)]
#[derive(Clone, Copy)]
struct INPUT_RECORD {
    EventType: u16,
    _pad: u16,
    Event: [u8; 16],
}

#[link(name = "kernel32")]
extern "system" {
    fn GetStdHandle(nStdHandle: DWORD) -> HANDLE;
    fn GetConsoleMode(h: HANDLE, mode: *mut DWORD) -> BOOL;
    fn SetConsoleMode(h: HANDLE, mode: DWORD) -> BOOL;
    fn GetConsoleScreenBufferInfo(h: HANDLE, info: *mut CONSOLE_SCREEN_BUFFER_INFO) -> BOOL;
    fn WaitForSingleObject(h: HANDLE, ms: DWORD) -> DWORD;
    fn GetNumberOfConsoleInputEvents(h: HANDLE, n: *mut DWORD) -> BOOL;
    fn ReadConsoleInputW(h: HANDLE, buf: *mut INPUT_RECORD, len: DWORD, read: *mut DWORD) -> BOOL;
    fn WriteConsoleW(h: HANDLE, buf: *const u16, len: DWORD, written: *mut DWORD, reserved: *mut c_void) -> BOOL;
}

fn u16_at(b: &[u8; 16], off: usize) -> u16 {
    u16::from_le_bytes([b[off], b[off + 1]])
}
fn u32_at(b: &[u8; 16], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

/// Консоль Windows.
pub struct WindowsBackend {
    hin: HANDLE,
    hout: HANDLE,
    saved_in: DWORD,
    saved_out: DWORD,
    size: Point,
    writer: AnsiWriter,
    /// Старшая половина суррогатной пары UTF-16 (для символов вне BMP).
    high_surrogate: Option<u16>,
    /// Готовые события (одна запись консоли может дать несколько событий).
    pending: std::collections::VecDeque<RawEvent>,
}

impl WindowsBackend {
    pub fn new() -> io::Result<Self> {
        unsafe {
            let hin = GetStdHandle(STD_INPUT_HANDLE);
            let hout = GetStdHandle(STD_OUTPUT_HANDLE);
            let mut saved_in = 0;
            let mut saved_out = 0;
            if GetConsoleMode(hin, &mut saved_in) == 0 || GetConsoleMode(hout, &mut saved_out) == 0 {
                return Err(io::Error::new(io::ErrorKind::Other, "программа запущена не в консоли"));
            }
            // Ввод: события окна и мыши; отключаем «быстрое выделение» (QuickEdit),
            // эхо, построчный режим и обработку Ctrl+C системой.
            SetConsoleMode(hin, ENABLE_WINDOW_INPUT | ENABLE_MOUSE_INPUT | ENABLE_EXTENDED_FLAGS);
            // Вывод: включаем поддержку VT-последовательностей.
            if SetConsoleMode(
                hout,
                ENABLE_PROCESSED_OUTPUT | ENABLE_VIRTUAL_TERMINAL_PROCESSING | DISABLE_NEWLINE_AUTO_RETURN,
            ) == 0
            {
                SetConsoleMode(hin, saved_in);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "консоль не поддерживает VT-последовательности (нужна Windows 10 или новее)",
                ));
            }
            let mut b = WindowsBackend {
                hin,
                hout,
                saved_in,
                saved_out,
                size: Point::new(80, 25),
                writer: AnsiWriter::new(),
                high_surrogate: None,
                pending: Default::default(),
            };
            b.write_str(ENTER_SEQ);
            b.size = b.query_size();
            Ok(b)
        }
    }

    /// Пишет строку в консоль (в UTF-16).
    fn write_str(&self, s: &str) {
        let w: Vec<u16> = s.encode_utf16().collect();
        // Пишем порциями — у WriteConsoleW есть ограничение на размер буфера.
        for chunk in w.chunks(8192) {
            let mut written = 0;
            unsafe {
                WriteConsoleW(self.hout, chunk.as_ptr(), chunk.len() as DWORD, &mut written, std::ptr::null_mut());
            }
        }
    }

    /// Размер видимой области окна консоли.
    fn query_size(&self) -> Point {
        let mut info = CONSOLE_SCREEN_BUFFER_INFO::default();
        unsafe {
            if GetConsoleScreenBufferInfo(self.hout, &mut info) != 0 {
                let w = info.srWindow.Right - info.srWindow.Left + 1;
                let h = info.srWindow.Bottom - info.srWindow.Top + 1;
                if w > 0 && h > 0 {
                    return Point::new(w as i32, h as i32);
                }
            }
        }
        self.size
    }

    /// Переводит модификаторы Windows в наши.
    fn mods(state: DWORD) -> Mods {
        Mods {
            shift: state & SHIFT_PRESSED != 0,
            ctrl: state & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED) != 0,
            alt: state & (LEFT_ALT_PRESSED | RIGHT_ALT_PRESSED) != 0,
        }
    }

    /// Разбор KEY_EVENT_RECORD:
    /// bKeyDown(4) wRepeatCount(2) wVirtualKeyCode(2) wVirtualScanCode(2) UnicodeChar(2) dwControlKeyState(4)
    fn read_key(&mut self, e: &[u8; 16]) {
        let down = u32_at(e, 0) != 0;
        if !down {
            return;
        }
        let repeat = u16_at(e, 4).max(1);
        let vk = u16_at(e, 6);
        let uchar = u16_at(e, 10);
        let state = u32_at(e, 12);
        let mut mods = Self::mods(state);

        // AltGr (правый Alt + левый Ctrl) с печатным символом — это просто символ
        // (например, «@» на немецкой раскладке).
        let altgr = state & RIGHT_ALT_PRESSED != 0 && state & LEFT_CTRL_PRESSED != 0;
        if altgr && uchar >= 0x20 {
            mods.alt = false;
            mods.ctrl = false;
        }

        let key = match vk {
            0x08 => Some(Key::Backspace),
            0x09 => Some(Key::Tab),
            0x0D => Some(Key::Enter),
            0x1B => Some(Key::Esc),
            0x21 => Some(Key::PgUp),
            0x22 => Some(Key::PgDn),
            0x23 => Some(Key::End),
            0x24 => Some(Key::Home),
            0x25 => Some(Key::Left),
            0x26 => Some(Key::Up),
            0x27 => Some(Key::Right),
            0x28 => Some(Key::Down),
            0x2D => Some(Key::Ins),
            0x2E => Some(Key::Del),
            0x70..=0x7B => Some(Key::F((vk - 0x70 + 1) as u8)),
            // Сами модификаторы (Shift, Ctrl, Alt, CapsLock) событий не дают.
            0x10 | 0x11 | 0x12 | 0x14 | 0x5B | 0x5C => return,
            _ => None,
        };

        let key = match key {
            Some(k) => k,
            None => {
                // Alt/Ctrl + буква или цифра: берём латинскую букву по коду клавиши,
                // чтобы Alt+X работал и в русской раскладке.
                if (mods.alt || mods.ctrl) && ((0x41..=0x5A).contains(&vk) || (0x30..=0x39).contains(&vk)) {
                    Key::Char((vk as u8).to_ascii_lowercase() as char)
                } else if uchar >= 0x20 {
                    // Суррогатные пары (эмодзи и т.п.).
                    if (0xD800..0xDC00).contains(&uchar) {
                        self.high_surrogate = Some(uchar);
                        return;
                    }
                    let c = if (0xDC00..0xE000).contains(&uchar) {
                        match self.high_surrogate.take() {
                            Some(hi) => char::decode_utf16([hi, uchar]).next().and_then(|r| r.ok()),
                            None => None,
                        }
                    } else {
                        char::from_u32(uchar as u32)
                    };
                    match c {
                        Some(c) => {
                            // Shift уже учтён в самом символе.
                            mods.shift = false;
                            Key::Char(c)
                        }
                        None => return,
                    }
                } else {
                    return;
                }
            }
        };
        for _ in 0..repeat {
            self.pending.push_back(RawEvent::Key(KeyEvent::new(key, mods)));
        }
    }

    /// Разбор MOUSE_EVENT_RECORD:
    /// dwMousePosition(COORD 4) dwButtonState(4) dwControlKeyState(4) dwEventFlags(4)
    fn read_mouse(&mut self, e: &[u8; 16]) {
        let x = u16_at(e, 0) as i16 as i32;
        let y = u16_at(e, 2) as i16 as i32;
        let bstate = u32_at(e, 4);
        let state = u32_at(e, 8);
        let flags = u32_at(e, 12);
        let mut buttons = 0;
        if bstate & 0x1 != 0 {
            buttons |= MB_LEFT;
        }
        if bstate & 0x2 != 0 {
            buttons |= MB_RIGHT;
        }
        if bstate & 0x4 != 0 {
            buttons |= MB_MIDDLE;
        }
        let wheel = if flags & MOUSE_WHEELED != 0 {
            // Старшее слово — направление: положительное = от пользователя (вверх).
            if (bstate as i32) >> 16 > 0 { -1 } else { 1 }
        } else {
            0
        };
        // Координаты отсчитываются от начала буфера — переводим в координаты окна.
        let mut info = CONSOLE_SCREEN_BUFFER_INFO::default();
        let top = unsafe {
            if GetConsoleScreenBufferInfo(self.hout, &mut info) != 0 {
                info.srWindow.Top as i32
            } else {
                0
            }
        };
        self.pending.push_back(RawEvent::Mouse { pos: Point::new(x, y - top), buttons, mods: Self::mods(state), wheel });
    }
}

impl Backend for WindowsBackend {
    fn size(&self) -> Point {
        self.size
    }

    fn read(&mut self, timeout: Duration) -> Option<RawEvent> {
        if let Some(e) = self.pending.pop_front() {
            return Some(e);
        }
        let ms = timeout.as_millis().min(u32::MAX as u128) as DWORD;
        unsafe {
            if WaitForSingleObject(self.hin, ms) != WAIT_OBJECT_0 {
                // Размер мог измениться и без события (например, при смене шрифта).
                let s = self.query_size();
                if s != self.size {
                    self.size = s;
                    self.writer.invalidate();
                    return Some(RawEvent::Resize(s));
                }
                return None;
            }
            let mut n = 0;
            GetNumberOfConsoleInputEvents(self.hin, &mut n);
            if n == 0 {
                return None;
            }
            let mut recs = vec![INPUT_RECORD { EventType: 0, _pad: 0, Event: [0; 16] }; n.min(128) as usize];
            let mut read = 0;
            if ReadConsoleInputW(self.hin, recs.as_mut_ptr(), recs.len() as DWORD, &mut read) == 0 {
                return None;
            }
            for r in recs.iter().take(read as usize) {
                match r.EventType {
                    KEY_EVENT => self.read_key(&r.Event),
                    MOUSE_EVENT => self.read_mouse(&r.Event),
                    WINDOW_BUFFER_SIZE_EVENT => {
                        let s = self.query_size();
                        if s != self.size {
                            self.size = s;
                            self.writer.invalidate();
                            self.pending.push_back(RawEvent::Resize(s));
                        }
                    }
                    _ => {}
                }
            }
        }
        self.pending.pop_front()
    }

    fn flush(&mut self, screen: &ScreenBuffer, cursor: Option<Point>, insert: bool) {
        let s = self.writer.render(screen, cursor, insert);
        self.write_str(&s);
    }
}

impl Drop for WindowsBackend {
    fn drop(&mut self) {
        self.write_str(LEAVE_SEQ);
        unsafe {
            SetConsoleMode(self.hin, self.saved_in);
            SetConsoleMode(self.hout, self.saved_out);
        }
    }
}
