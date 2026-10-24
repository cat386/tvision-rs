//! Строка ввода длинного целого (`TInputLong` из пакета Dialog Design
//! Л. Дэвида Болдуина).
//!
//! Принимает только числа (и `$` для шестнадцатеричных, если разрешено),
//! проверяет диапазон при уходе клавишей Tab и при закрытии диалога.
//! Данные — [`Value::Int`].

use crate::event::{cm, Event, Key};
use crate::geometry::Rect;
use crate::locale::tr;
use crate::view::{Ctx, DrawCtx, Value, View};
use crate::widgets::inputline::InputLine;
use crate::widgets::msgbox::{build_message_box, mf};

/// Флаги `TInputLong`.
pub mod il {
    /// Разрешён шестнадцатеричный ввод с ведущим `$`.
    pub const HEX: u16 = 1;
    /// Пустое поле считается нулём.
    pub const BLANK_EQ_ZERO: u16 = 2;
    /// Показывать число в шестнадцатеричном виде.
    pub const DISPLAY_HEX: u16 = 4;
}

pub struct InputLong {
    /// Обычная строка ввода, которая делает всю работу с текстом.
    pub il: InputLine,
    pub il_options: u16,
    pub low: i64,
    pub high: i64,
    /// Текст метки — для сообщения об ошибке (в оригинале метку искали
    /// среди соседей; здесь её передают явно, как в C++-версии).
    pub label: String,
}

/// `$1F`, `-$1F`: шестнадцатеричная запись как в оригинале (`FormHexStr`).
fn hex_str(v: i64) -> String {
    if v < 0 {
        format!("-${:X}", -v)
    } else {
        format!("${:X}", v)
    }
}

/// Разбор числа как `Val` в Turbo Pascal: десятичное или `$`-hex.
pub fn parse_long(s: &str) -> Option<i64> {
    let s = s.trim();
    let (neg, body) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s),
    };
    let v = if let Some(h) = body.strip_prefix('$') {
        i64::from_str_radix(h, 16).ok()?
    } else {
        if body.is_empty() || !body.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        body.parse::<i64>().ok()?
    };
    let v = if neg { -v } else { v };
    // LongInt: 32 бита со знаком.
    if v < i32::MIN as i64 || v > u32::MAX as i64 {
        return None;
    }
    Some(v)
}

impl InputLong {
    /// `TInputLong(bounds, maxLen, lowerLim, upperLim, flags, label)`.
    pub fn new(bounds: Rect, max_len: usize, low: i64, high: i64, flags: u16, label: &str) -> Self {
        let flags = if flags & il::DISPLAY_HEX != 0 { flags | il::HEX } else { flags };
        let mut il = InputLine::new(bounds, max_len);
        if flags & il::BLANK_EQ_ZERO != 0 {
            il.set_text("0");
        }
        InputLong { il, il_options: flags, low, high, label: label.into() }
    }

    /// Значение (`GetData`): то, что удалось разобрать, иначе 0.
    pub fn value(&self) -> i64 {
        parse_long(&self.il.text()).unwrap_or(0)
    }

    /// Установить значение (`SetData`): с подрезкой по диапазону.
    pub fn set_value(&mut self, v: i64) {
        let v = v.clamp(self.low, self.high);
        let s = if self.il_options & il::DISPLAY_HEX != 0 { hex_str(v) } else { v.to_string() };
        self.il.set_text(&s);
    }

    /// Значение в допустимом диапазоне? (`RangeCheck`)
    pub fn range_check(&mut self) -> bool {
        if self.il.text().is_empty() && self.il_options & il::BLANK_EQ_ZERO != 0 {
            self.il.set_text("0");
        }
        matches!(parse_long(&self.il.text()), Some(v) if v >= self.low && v <= self.high)
    }

    /// Сообщение об ошибке диапазона (`Error`).
    fn error(&self, ctx: &mut Ctx) {
        let (mut sl, mut su) = (self.low.to_string(), self.high.to_string());
        if self.il_options & il::HEX != 0 {
            sl = format!("{}({})", sl, hex_str(self.low));
            su = format!("{}({})", su, hex_str(self.high));
        }
        let lbl: String = self.label.chars().filter(|&c| c != '~').collect();
        let lbl = if lbl.is_empty() { String::new() } else { format!("\"{}\"\n", lbl) };
        let msg = format!("{}{} {} {} {}", lbl, tr("Value not within range"), sl, tr("to"), su);
        let dlg = build_message_box(Rect::new(0, 0, 40, 9), &msg, mf::ERROR | mf::OK_BUTTON);
        ctx.exec(Box::new(dlg), None, 0);
    }
}

impl View for InputLong {
    crate::impl_view_basics_field!(il);

    fn draw(&self, dc: &mut DrawCtx) {
        self.il.draw(dc);
    }

    fn palette(&self) -> &'static [u8] {
        self.il.palette()
    }

    fn set_state(&mut self, flag: u16, enable: bool) {
        self.il.set_state(flag, enable);
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        if let Event::KeyDown(k) = ev {
            let k = *k;
            if self.il.vs.is_focused() {
                // Tab / Shift+Tab: уйти можно только с правильным числом.
                if matches!(k.key, Key::Tab | Key::BackTab) && !k.mods.ctrl && !k.mods.alt && !self.range_check() {
                    self.error(ctx);
                    self.il.select_all(true);
                    ev.clear();
                    return;
                }
                // Фильтр символов.
                if let Some(c) = k.text_char() {
                    let c = c.to_ascii_uppercase();
                    let ok = match c {
                        '0'..='9' => true,
                        '-' => self.low < 0 && self.il.cursor_pos() == 0,
                        '$' => self.il_options & il::HEX != 0,
                        'A'..='F' => self.il.text().contains('$'),
                        _ => false,
                    };
                    if !ok {
                        ev.clear();
                        return;
                    }
                    *ev = Event::KeyDown(crate::event::KeyEvent::new(Key::Char(c), k.mods));
                }
            }
        }
        self.il.handle_event(ev, ctx);
    }

    fn valid(&mut self, command: u16, ctx: &mut Ctx) -> bool {
        let mut ok = self.il.valid(command, ctx);
        if ok && command != cm::VALID && command != cm::CANCEL {
            ok = self.range_check();
            if !ok {
                self.error(ctx);
                ctx.focus(self.il.vs.id);
                self.il.select_all(true);
            }
        }
        ok
    }

    fn has_data(&self) -> bool {
        true
    }

    fn get_data(&self) -> Value {
        Value::Int(self.value())
    }

    fn set_data(&mut self, value: &Value) {
        match value {
            Value::Int(v) => self.set_value(*v),
            Value::U32(v) => self.set_value(*v as i64),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        assert_eq!(parse_long("123"), Some(123));
        assert_eq!(parse_long("-$1F"), Some(-31));
        assert_eq!(parse_long("$ff"), Some(255));
        assert_eq!(parse_long("12a"), None);
        assert_eq!(parse_long(""), None);
        let mut l = InputLong::new(Rect::new(0, 0, 10, 1), 8, -5, 100, il::DISPLAY_HEX, "~C~ount");
        l.set_value(500);
        assert_eq!(l.il.text(), "$64");
        assert_eq!(l.value(), 100);
        assert!(l.range_check());
    }
}
