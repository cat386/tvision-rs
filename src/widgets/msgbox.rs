//! Окна сообщений (`messageBox`, `messageBoxRect`, `inputBox`).

use crate::event::cm;
use crate::geometry::Rect;
use crate::locale::tr;
use crate::program::Program;
use crate::view::{of, Value};
use crate::widgets::button::{bf, Button};
use crate::widgets::dialog::Dialog;
use crate::widgets::inputline::InputLine;
use crate::widgets::statictext::{Label, StaticText};

/// Флаги окна сообщения (`mfXXX`).
pub mod mf {
    // Тип сообщения (младшие биты) — определяет заголовок.
    pub const WARNING: u16 = 0x0000;
    pub const ERROR: u16 = 0x0001;
    pub const INFORMATION: u16 = 0x0002;
    pub const CONFIRMATION: u16 = 0x0003;
    // Кнопки.
    pub const YES_BUTTON: u16 = 0x0100;
    pub const NO_BUTTON: u16 = 0x0200;
    pub const OK_BUTTON: u16 = 0x0400;
    pub const CANCEL_BUTTON: u16 = 0x0800;
    pub const YES_NO_CANCEL: u16 = YES_BUTTON | NO_BUTTON | CANCEL_BUTTON;
    pub const OK_CANCEL: u16 = OK_BUTTON | CANCEL_BUTTON;
}

/// Заголовки окон сообщений. Можно перевести, как `MsgBoxText` в оригинале.
pub struct MsgBoxText;
impl MsgBoxText {
    pub const YES: &'static str = "~Y~es";
    pub const NO: &'static str = "~N~o";
    pub const OK: &'static str = "O~K~";
    pub const CANCEL: &'static str = "Cancel";
    pub const WARNING: &'static str = "Warning";
    pub const ERROR: &'static str = "Error";
    pub const INFORMATION: &'static str = "Information";
    pub const CONFIRM: &'static str = "Confirm";
}

/// Окно сообщения 40×9 в центре экрана. Возвращает команду нажатой кнопки
/// (`cm::YES`, `cm::NO`, `cm::OK`, `cm::CANCEL`).
pub fn message_box(app: &mut Program, msg: &str, options: u16) -> u16 {
    let r = Rect::new(0, 0, 40, 9);
    message_box_rect(app, r, msg, options, true)
}

/// Окно сообщения заданного размера (`messageBoxRect`).
pub fn message_box_rect(app: &mut Program, r: Rect, msg: &str, options: u16, centered: bool) -> u16 {
    let mut d = build_message_box(r, msg, options);
    if !centered {
        d.win.group.vs.options &= !of::CENTERED;
    }
    let (cmd, _) = app.exec_dialog(d);
    cmd
}

/// Строит диалог сообщения (отцентрированный), не запуская его.
/// Пригодится, когда сообщение нужно показать из представления через `ctx.exec`.
pub fn build_message_box(r: Rect, msg: &str, options: u16) -> Dialog {
    let titles = [tr(MsgBoxText::WARNING), tr(MsgBoxText::ERROR), tr(MsgBoxText::INFORMATION), tr(MsgBoxText::CONFIRM)];
    let mut d = Dialog::new(r, titles[(options & 3) as usize]);
    d.win.group.vs.options |= of::CENTERED;
    let w = r.width();
    let h = r.height();
    d.add(StaticText::new(Rect::new(3, 2, w - 2, h - 3), msg));

    let names = [tr(MsgBoxText::YES), tr(MsgBoxText::NO), tr(MsgBoxText::OK), tr(MsgBoxText::CANCEL)];
    let commands = [cm::YES, cm::NO, cm::OK, cm::CANCEL];
    let chosen: Vec<usize> = (0..4).filter(|i| options & (0x0100 << i) != 0).collect();
    // Кнопки шириной 10 через 2 столбца, по центру (как в оригинале).
    let total = chosen.len() as i32 * 12 - 2;
    let mut x = (w - total) / 2;
    let mut first = None;
    for (n, &i) in chosen.iter().enumerate() {
        let flags = if n == 0 { bf::DEFAULT } else { bf::NORMAL };
        let id = d.add(Button::new(Rect::new(x, h - 3, x + 10, h - 1), names[i], commands[i], flags));
        if first.is_none() {
            first = Some(id);
        }
        x += 12;
    }
    if let Some(id) = first {
        d.select(id);
    }
    d
}

/// Окно ввода строки (`inputBox`). Возвращает `Some(текст)` при OK.
pub fn input_box(app: &mut Program, title: &str, label: &str, initial: &str, limit: usize) -> Option<String> {
    let label_w = crate::text::cstr_width(label) as i32;
    let w = (label_w + 30).clamp(30, 70);
    let mut d = Dialog::centered(w, 8, title);
    let il = d.add(InputLine::new(Rect::new(4 + label_w, 2, w - 3, 3), limit));
    d.add(Label::new(Rect::new(2, 2, 3 + label_w, 3), label, Some(il)));
    d.add(Button::new(Rect::new(w - 24, 5, w - 14, 7), tr(MsgBoxText::OK), cm::OK, bf::DEFAULT));
    d.add(Button::new(Rect::new(w - 12, 5, w - 2, 7), tr(MsgBoxText::CANCEL), cm::CANCEL, bf::NORMAL));
    d.select(il);
    if let Some(v) = d.win.group.find_mut::<InputLine>(il) {
        v.set_text(initial);
    }
    let (cmd, d) = app.exec_dialog(d);
    if cmd == cm::OK {
        match d.win.group.get_data() {
            Value::List(v) => v.into_iter().find_map(|x| if let Value::Str(s) = x { Some(s) } else { None }),
            _ => None,
        }
    } else {
        None
    }
}
