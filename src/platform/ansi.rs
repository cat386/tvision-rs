//! Работа с ANSI/VT-терминалом: вывод экрана и разбор входных последовательностей.
//!
//! Вывод — аналог `platform/ansiwrit.cpp`: сравниваем новый кадр с предыдущим
//! и посылаем в терминал только изменившиеся ячейки.
//!
//! Ввод — аналог `platform/termio.cpp`: разбираем последовательности
//! `ESC [ ... ~`, `ESC O P`, мышь в формате SGR (`ESC [ < b ; x ; y M`) и UTF-8.

use std::fmt::Write as _;

use crate::color::{back, bg256, bios_to_ansi, color_mode, fg256, fore, is_bg256, is_fg256, Attr, ColorMode};
use crate::drawbuf::ScreenBuffer;
use crate::event::{Key, KeyEvent, Mods};
use crate::geometry::Point;
use crate::platform::RawEvent;

/// Последовательности для входа в полноэкранный режим:
/// альтернативный экран, скрытый курсор, отслеживание мыши (1002 — движение
/// с нажатой кнопкой, 1003 — любое движение, 1006 — формат SGR).
pub const ENTER_SEQ: &str = "\x1b[?1049h\x1b[?25l\x1b[?1000h\x1b[?1002h\x1b[?1003h\x1b[?1006h\x1b[2J";

/// Последовательности для восстановления терминала при выходе.
pub const LEAVE_SEQ: &str = "\x1b[?1006l\x1b[?1003l\x1b[?1002l\x1b[?1000l\x1b[0m\x1b[0 q\x1b[?25h\x1b[?1049l";

/// Формирует вывод на экран с учётом предыдущего кадра.
#[derive(Default)]
pub struct AnsiWriter {
    prev: Option<ScreenBuffer>,
    /// Режим цвета, в котором выведен `prev` (при смене — полная перерисовка).
    prev_mode: Option<ColorMode>,
    /// Свой режим цвета; `None` — общий [`crate::color::color_mode`].
    pub mode: Option<ColorMode>,
}

impl AnsiWriter {
    pub fn new() -> Self {
        AnsiWriter { prev: None, prev_mode: None, mode: None }
    }

    /// Писатель с заданным режимом цвета (например, для тестов).
    pub fn with_mode(mode: ColorMode) -> Self {
        AnsiWriter { prev: None, prev_mode: None, mode: Some(mode) }
    }

    /// Забыть предыдущий кадр — следующий `render` перерисует всё
    /// (нужно после изменения размера терминала).
    pub fn invalidate(&mut self) {
        self.prev = None;
    }

    /// Возвращает строку с управляющими последовательностями,
    /// которая переводит терминал из предыдущего кадра в `screen`.
    pub fn render(&mut self, screen: &ScreenBuffer, cursor: Option<Point>, insert: bool) -> String {
        let mut out = String::with_capacity(4096);
        // Пока рисуем — прячем курсор, чтобы он не «бегал» по экрану.
        out.push_str("\x1b[?25l");

        let mode = self.mode.unwrap_or_else(color_mode);
        let full = match &self.prev {
            Some(p) => p.width != screen.width || p.height != screen.height || self.prev_mode != Some(mode),
            None => true,
        };
        self.prev_mode = Some(mode);
        if full {
            out.push_str("\x1b[0m\x1b[2J");
        }

        let mut cur_attr: Option<Attr> = None;
        for y in 0..screen.height {
            // Позиция, в которой окажется курсор терминала после последнего вывода
            // (чтобы не посылать лишние команды перемещения).
            let mut expect_x: Option<usize> = None;
            let mut x = 0;
            while x < screen.width {
                let cell = screen.get(x, y);
                let changed = full || self.prev.as_ref().map(|p| p.get(x, y) != cell).unwrap_or(true);
                // Если изменился хвост широкого символа — надо перерисовать сам символ.
                let head_changed = cell.is_wide_tail() && changed;
                if !changed && !head_changed {
                    x += 1;
                    continue;
                }
                let mut start = x;
                if cell.is_wide_tail() && x > 0 {
                    start = x - 1;
                }
                let c = screen.get(start, y);
                if expect_x != Some(start) {
                    let _ = write!(out, "\x1b[{};{}H", y + 1, start + 1);
                }
                if cur_attr != Some(c.attr) {
                    push_sgr(&mut out, c.attr, mode);
                    cur_attr = Some(c.attr);
                }
                if c.is_wide_tail() {
                    // Осиротевший хвост (не должно случаться) — выводим пробел.
                    out.push(' ');
                    expect_x = Some(start + 1);
                    x = start + 1;
                } else {
                    let ch = if (c.ch as u32) < 0x20 { ' ' } else { c.ch };
                    out.push(ch);
                    let w = crate::text::char_width(ch).max(1);
                    expect_x = Some(start + w);
                    x = start + w;
                }
            }
        }

        out.push_str("\x1b[0m");
        if let Some(p) = cursor {
            let _ = write!(out, "\x1b[{};{}H", p.y + 1, p.x + 1);
            // Форма курсора: в режиме вставки — блок, иначе — подчёркивание
            // (как в оригинальном tvision).
            out.push_str(if insert { "\x1b[2 q" } else { "\x1b[4 q" });
            out.push_str("\x1b[?25h");
        }
        self.prev = Some(screen.clone());
        out
    }
}

/// Добавляет последовательность SGR для установки цвета.
///
/// Цвета BIOS всегда выводятся 16-цветными кодами (`30–37`, `90–97`,
/// `40–47`, `100–107`) — цветами темы терминала, как в оригинале. Цвета
/// xterm-256 в режиме [`ColorMode::Xterm256`] выводятся как `38;5;N` /
/// `48;5;N`, в режиме [`ColorMode::Ansi16`] — ближайшим цветом BIOS.
pub fn push_sgr(out: &mut String, a: Attr, mode: ColorMode) {
    let x256 = mode == ColorMode::Xterm256;
    out.push_str("\x1b[0;");
    if x256 && is_fg256(a) {
        let _ = write!(out, "38;5;{}", fg256(a));
    } else {
        let fg = bios_to_ansi(fore(a));
        let _ = write!(out, "{}", if fg < 8 { 30 + fg as u32 } else { 90 + (fg - 8) as u32 });
    }
    if x256 && is_bg256(a) {
        let _ = write!(out, ";48;5;{}m", bg256(a));
    } else {
        let bg = bios_to_ansi(back(a));
        let _ = write!(out, ";{}m", if bg < 8 { 40 + bg as u32 } else { 100 + (bg - 8) as u32 });
    }
}

/// Результат попытки разобрать начало входного буфера.
#[derive(Debug, PartialEq)]
pub enum Parsed {
    /// Распознано событие, занявшее `usize` байт.
    Event(RawEvent, usize),
    /// Данных пока не хватает — нужно дочитать.
    Incomplete,
    /// Нераспознанные байты — пропустить `usize` байт.
    Skip(usize),
}

/// Разборщик входного потока терминала. Хранит состояние кнопок мыши.
#[derive(Default)]
pub struct InputParser {
    buttons: u8,
}

impl InputParser {
    pub fn new() -> Self {
        InputParser { buttons: 0 }
    }

    /// Разбирает начало буфера `buf`.
    pub fn parse(&mut self, buf: &[u8]) -> Parsed {
        if buf.is_empty() {
            return Parsed::Incomplete;
        }
        let b0 = buf[0];
        if b0 == 0x1b {
            return self.parse_escape(buf);
        }
        match b0 {
            b'\r' | b'\n' => Parsed::Event(key(Key::Enter, Mods::NONE), 1),
            b'\t' => Parsed::Event(key(Key::Tab, Mods::NONE), 1),
            0x7f | 0x08 => Parsed::Event(key(Key::Backspace, Mods::NONE), 1),
            0x00 => Parsed::Event(key(Key::Char(' '), Mods::CTRL), 1),
            // Ctrl+A .. Ctrl+Z
            0x01..=0x1a => Parsed::Event(key(Key::Char((b'a' + b0 - 1) as char), Mods::CTRL), 1),
            0x1c..=0x1f => Parsed::Skip(1),
            _ => match decode_utf8(buf) {
                Utf8::Char(c, n) => Parsed::Event(key(Key::Char(c), Mods::NONE), n),
                Utf8::Incomplete => Parsed::Incomplete,
                Utf8::Invalid => Parsed::Skip(1),
            },
        }
    }

    /// Разбор последовательностей, начинающихся с ESC.
    fn parse_escape(&mut self, buf: &[u8]) -> Parsed {
        if buf.len() < 2 {
            return Parsed::Incomplete;
        }
        match buf[1] {
            b'[' => self.parse_csi(buf),
            b'O' => {
                // SS3: ESC O P = F1 и т.п. (режим «приложения» клавиатуры)
                if buf.len() < 3 {
                    return Parsed::Incomplete;
                }
                let k = match buf[2] {
                    b'P' => Key::F(1),
                    b'Q' => Key::F(2),
                    b'R' => Key::F(3),
                    b'S' => Key::F(4),
                    b'A' => Key::Up,
                    b'B' => Key::Down,
                    b'C' => Key::Right,
                    b'D' => Key::Left,
                    b'H' => Key::Home,
                    b'F' => Key::End,
                    b'M' => Key::Enter,
                    _ => return Parsed::Skip(3),
                };
                Parsed::Event(key(k, Mods::NONE), 3)
            }
            // ESC ESC — считаем отдельным нажатием Esc.
            0x1b => Parsed::Event(key(Key::Esc, Mods::NONE), 1),
            // Alt + управляющая клавиша.
            b'\r' => Parsed::Event(key(Key::Enter, Mods::ALT), 2),
            0x7f | 0x08 => Parsed::Event(key(Key::Backspace, Mods::ALT), 2),
            b'\t' => Parsed::Event(key(Key::Tab, Mods::ALT), 2),
            _ => {
                // ESC + символ = Alt + символ.
                match decode_utf8(&buf[1..]) {
                    Utf8::Char(c, n) if (c as u32) >= 0x20 => Parsed::Event(key(Key::Char(c), Mods::ALT), 1 + n),
                    Utf8::Incomplete => Parsed::Incomplete,
                    _ => Parsed::Event(key(Key::Esc, Mods::NONE), 1),
                }
            }
        }
    }

    /// Разбор CSI: `ESC [ параметры финальный_символ`.
    fn parse_csi(&mut self, buf: &[u8]) -> Parsed {
        // Консоль Linux: ESC [ [ A .. ESC [ [ E = F1..F5
        if buf.len() >= 3 && buf[2] == b'[' {
            if buf.len() < 4 {
                return Parsed::Incomplete;
            }
            return match buf[3] {
                c @ b'A'..=b'E' => Parsed::Event(key(Key::F(c - b'A' + 1), Mods::NONE), 4),
                _ => Parsed::Skip(4),
            };
        }
        // Ищем финальный байт (0x40..0x7E).
        let mut i = 2;
        while i < buf.len() {
            let c = buf[i];
            if (0x40..=0x7e).contains(&c) {
                break;
            }
            i += 1;
        }
        if i >= buf.len() {
            // Защита от мусора: слишком длинная последовательность.
            if buf.len() > 32 {
                return Parsed::Skip(buf.len());
            }
            return Parsed::Incomplete;
        }
        let fin = buf[i];
        let params_raw = &buf[2..i];
        let len = i + 1;

        // Мышь в формате SGR: ESC [ < b ; x ; y M (нажатие/движение) или m (отпускание)
        if params_raw.first() == Some(&b'<') {
            let nums = parse_params(&params_raw[1..]);
            if nums.len() < 3 {
                return Parsed::Skip(len);
            }
            return Parsed::Event(self.sgr_mouse(nums[0], nums[1], nums[2], fin == b'M'), len);
        }

        let nums = parse_params(params_raw);
        let mods = nums.get(1).map(|&m| decode_mods(m)).unwrap_or(Mods::NONE);
        let k = match fin {
            b'A' => Key::Up,
            b'B' => Key::Down,
            b'C' => Key::Right,
            b'D' => Key::Left,
            b'H' => Key::Home,
            b'F' => Key::End,
            b'P' => Key::F(1),
            b'Q' => Key::F(2),
            b'R' => Key::F(3),
            b'S' => Key::F(4),
            b'Z' => return Parsed::Event(key(Key::Tab, Mods::SHIFT), len),
            b'~' => match nums.first().copied().unwrap_or(0) {
                1 | 7 => Key::Home,
                2 => Key::Ins,
                3 => Key::Del,
                4 | 8 => Key::End,
                5 => Key::PgUp,
                6 => Key::PgDn,
                11 => Key::F(1),
                12 => Key::F(2),
                13 => Key::F(3),
                14 => Key::F(4),
                15 => Key::F(5),
                17 => Key::F(6),
                18 => Key::F(7),
                19 => Key::F(8),
                20 => Key::F(9),
                21 => Key::F(10),
                23 => Key::F(11),
                24 => Key::F(12),
                _ => return Parsed::Skip(len),
            },
            _ => return Parsed::Skip(len),
        };
        Parsed::Event(key(k, mods), len)
    }

    /// Преобразует отчёт мыши SGR в событие с текущим состоянием кнопок.
    fn sgr_mouse(&mut self, b: u32, x: u32, y: u32, press: bool) -> RawEvent {
        let pos = Point::new(x as i32 - 1, y as i32 - 1);
        let mods = Mods { shift: b & 4 != 0, alt: b & 8 != 0, ctrl: b & 16 != 0 };
        // Колесо: биты 64 (+0 вверх, +1 вниз).
        if b & 64 != 0 {
            let wheel = if b & 1 != 0 { 1 } else { -1 };
            return RawEvent::Mouse { pos, buttons: self.buttons, mods, wheel };
        }
        let motion = b & 32 != 0;
        let btn = match b & 3 {
            0 => crate::event::MB_LEFT,
            1 => crate::event::MB_MIDDLE,
            2 => crate::event::MB_RIGHT,
            _ => 0,
        };
        if !motion {
            if press {
                self.buttons |= btn;
            } else {
                self.buttons &= !btn;
            }
        } else if btn == 0 && b & 3 == 3 {
            // Движение без нажатых кнопок (режим 1003).
            self.buttons = 0;
        }
        RawEvent::Mouse { pos, buttons: self.buttons, mods, wheel: 0 }
    }
}

fn key(k: Key, mods: Mods) -> RawEvent {
    RawEvent::Key(KeyEvent::new(k, mods))
}

/// Модификатор в xterm-кодировке: 1 + (1 shift | 2 alt | 4 ctrl).
fn decode_mods(m: u32) -> Mods {
    let m = m.saturating_sub(1);
    Mods { shift: m & 1 != 0, alt: m & 2 != 0, ctrl: m & 4 != 0 }
}

/// Разбирает числа, разделённые `;`. Пустое значение считается 0.
fn parse_params(raw: &[u8]) -> Vec<u32> {
    raw.split(|&c| c == b';')
        .map(|p| p.iter().filter(|c| c.is_ascii_digit()).fold(0u32, |a, &c| a.saturating_mul(10).saturating_add((c - b'0') as u32)))
        .collect()
}

enum Utf8 {
    Char(char, usize),
    Incomplete,
    Invalid,
}

/// Декодирует один символ UTF-8 из начала буфера.
fn decode_utf8(buf: &[u8]) -> Utf8 {
    let b0 = buf[0];
    let n = if b0 < 0x80 {
        1
    } else if b0 & 0xE0 == 0xC0 {
        2
    } else if b0 & 0xF0 == 0xE0 {
        3
    } else if b0 & 0xF8 == 0xF0 {
        4
    } else {
        return Utf8::Invalid;
    };
    if buf.len() < n {
        return Utf8::Incomplete;
    }
    match std::str::from_utf8(&buf[..n]) {
        Ok(s) => Utf8::Char(s.chars().next().unwrap(), n),
        Err(_) => Utf8::Invalid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(p: &mut InputParser, s: &[u8]) -> RawEvent {
        match p.parse(s) {
            Parsed::Event(e, n) => {
                assert_eq!(n, s.len(), "должна быть разобрана вся последовательность");
                e
            }
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn colors_256() {
        use crate::color::{attr256, with_bg256, ColorMode};
        let mut s = String::new();
        push_sgr(&mut s, 0x1E, ColorMode::Xterm256);
        assert_eq!(s, "\x1b[0;93;44m"); // BIOS — старые коды
        s.clear();
        push_sgr(&mut s, attr256(208, 236), ColorMode::Xterm256);
        assert_eq!(s, "\x1b[0;38;5;208;48;5;236m");
        s.clear();
        push_sgr(&mut s, with_bg256(0x0F, 17), ColorMode::Xterm256);
        assert_eq!(s, "\x1b[0;97;48;5;17m");
        s.clear();
        // 16-цветный терминал: ближайшие цвета BIOS.
        push_sgr(&mut s, attr256(203, 16), ColorMode::Ansi16);
        assert_eq!(s, "\x1b[0;91;40m");
    }

    #[test]
    fn mode_change_redraws_all() {
        use crate::color::{attr256, ColorMode};
        use crate::drawbuf::Cell;
        let mut scr = ScreenBuffer::new(4, 1);
        *scr.get_mut(0, 0) = Cell::new('A', attr256(208, 17));
        let mut w = AnsiWriter::with_mode(ColorMode::Xterm256);
        assert!(w.render(&scr, None, false).contains("38;5;208"));
        assert!(!w.render(&scr, None, false).contains('A')); // ничего не изменилось
        w.mode = Some(ColorMode::Ansi16);
        let out = w.render(&scr, None, false);
        assert!(out.contains('A') && !out.contains("38;5"));
    }

    #[test]
    fn keys() {
        let mut p = InputParser::new();
        assert_eq!(ev(&mut p, b"\x1b[A"), key(Key::Up, Mods::NONE));
        assert_eq!(ev(&mut p, b"\x1b[1;5C"), key(Key::Right, Mods::CTRL));
        assert_eq!(ev(&mut p, b"\x1b[21~"), key(Key::F(10), Mods::NONE));
        assert_eq!(ev(&mut p, b"\x1bOP"), key(Key::F(1), Mods::NONE));
        assert_eq!(ev(&mut p, b"\x1bx"), key(Key::Char('x'), Mods::ALT));
        assert_eq!(ev(&mut p, "ж".as_bytes()), key(Key::Char('ж'), Mods::NONE));
        assert_eq!(ev(&mut p, b"\x18"), key(Key::Char('x'), Mods::CTRL));
        assert_eq!(p.parse(b"\x1b"), Parsed::Incomplete);
        assert_eq!(p.parse(b"\x1b[1;"), Parsed::Incomplete);
    }

    #[test]
    fn mouse() {
        let mut p = InputParser::new();
        assert_eq!(ev(&mut p, b"\x1b[<0;10;5M"), RawEvent::Mouse { pos: Point::new(9, 4), buttons: 1, mods: Mods::NONE, wheel: 0 });
        assert_eq!(ev(&mut p, b"\x1b[<0;10;5m"), RawEvent::Mouse { pos: Point::new(9, 4), buttons: 0, mods: Mods::NONE, wheel: 0 });
        assert_eq!(ev(&mut p, b"\x1b[<65;1;1M"), RawEvent::Mouse { pos: Point::new(0, 0), buttons: 0, mods: Mods::NONE, wheel: 1 });
    }
}
