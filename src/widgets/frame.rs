//! Рамка окна (`TFrame`).
//!
//! В C++ рамка — отдельное дочернее представление окна, которое читает
//! заголовок, номер и флаги прямо из `owner`. В этом порте рамка не хранится
//! как ребёнок: окно само вызывает [`Frame::draw_frame`], передавая нужные
//! сведения. Обработку мыши на рамке (кнопки, перетаскивание) выполняет
//! окно — см. `Window::handle_event`.

use crate::color::palettes;
use crate::drawbuf::DrawBuffer;
use crate::text::{str_width, truncate_width};
use crate::view::DrawCtx;
use crate::widgets::window::wf;

/// Символы рамки: одинарная (неактивное окно) и двойная (активное).
/// В оригинале — таблица `TFrame::frameChars` в кодировке CP437.
struct FrameChars {
    tl: char,
    h: char,
    tr: char,
    v: char,
    bl: char,
    br: char,
}

const SINGLE: FrameChars = FrameChars { tl: '┌', h: '─', tr: '┐', v: '│', bl: '└', br: '┘' };
const DOUBLE: FrameChars = FrameChars { tl: '╔', h: '═', tr: '╗', v: '║', bl: '╚', br: '╝' };

/// Иконки рамки (`closeIcon`, `zoomIcon`, `unZoomIcon`, `dragIcon`).
pub const CLOSE_ICON: &str = "[~■~]";
pub const ZOOM_ICON: &str = "[~↑~]";
pub const UNZOOM_ICON: &str = "[~↕~]";
pub const DRAG_ICON: &str = "~─┘~";
pub const DRAG_LEFT_ICON: &str = "~└─~";

/// Сведения об окне, нужные для рисования рамки.
pub struct FrameInfo<'a> {
    pub title: &'a str,
    pub number: i16,
    pub flags: u8,
    pub active: bool,
    pub dragging: bool,
    pub zoomed: bool,
}

/// Рисовальщик рамки.
pub struct Frame;

impl Frame {
    /// Палитра рамки (`cpFrame`).
    pub fn palette() -> &'static [u8] {
        palettes::FRAME
    }

    /// Рисует рамку на всю область `dc` (порт `TFrame::draw`).
    /// `dc` должен быть создан с палитрой рамки.
    pub fn draw_frame(dc: &mut DrawCtx, info: &FrameInfo) {
        let size = dc.size;
        let width = size.x.max(0) as usize;
        if width < 2 || size.y < 2 {
            return;
        }
        // Выбор цветов и типа линии, как в оригинале.
        let (c_frame, c_title, chars) = if info.dragging {
            (dc.get_color(0x0505), dc.get_color(0x0005), &SINGLE)
        } else if !info.active {
            (dc.get_color(0x0101), dc.get_color(0x0002), &SINGLE)
        } else {
            (dc.get_color(0x0503), dc.get_color(0x0004), &DOUBLE)
        };
        let lo = crate::color::lo(c_frame);

        // --- Верхняя строка: линия, номер, заголовок, кнопки ---
        let mut b = DrawBuffer::new(width);
        b.move_char(0, chars.h, lo, width);
        b.put_char(0, chars.tl);
        b.put_char(width - 1, chars.tr);

        let mut l = width as i32 - 10;
        if info.flags & (wf::CLOSE | wf::ZOOM) != 0 {
            l -= 6;
        }
        if info.number > 0 && info.number < 10 {
            l -= 4;
            let i = if info.flags & wf::ZOOM != 0 { 7 } else { 3 };
            if width >= i {
                b.put_char(width - i, char::from(b'0' + info.number as u8));
            }
        }
        // `l` в оригинале — только подсказка для `getTitle(l)`, которую
        // `TWindow` игнорирует; заголовок ограничивается шириной `width - 10`.
        let _ = l;
        if !info.title.is_empty() && width > 10 {
            let t = truncate_width(info.title, width - 10);
            let tl = str_width(t);
            let i = (width - tl) / 2;
            if i >= 1 {
                b.put_char(i - 1, ' ');
                b.move_str(i, t, crate::color::lo(c_title));
                b.put_char(i + tl, ' ');
                // Пробелы вокруг заголовка — цветом рамки.
                b.put_attribute(i - 1, lo);
                b.put_attribute(i + tl, lo);
            }
        }
        if info.active {
            if info.flags & wf::CLOSE != 0 && width >= 8 {
                b.move_cstr(2, CLOSE_ICON, c_frame);
            }
            if info.flags & wf::ZOOM != 0 && width >= 8 {
                b.move_cstr(width - 5, if info.zoomed { UNZOOM_ICON } else { ZOOM_ICON }, c_frame);
            }
        }
        dc.write_line(0, 0, size.x, 1, &b);

        // --- Средние строки ---
        let mut m = DrawBuffer::new(width);
        m.move_char(0, ' ', lo, width);
        m.put_char(0, chars.v);
        m.put_char(width - 1, chars.v);
        if size.y > 2 {
            dc.write_line(0, 1, size.x, size.y - 2, &m);
        }

        // --- Нижняя строка (с «уголком» изменения размера) ---
        let mut bt = DrawBuffer::new(width);
        bt.move_char(0, chars.h, lo, width);
        bt.put_char(0, chars.bl);
        bt.put_char(width - 1, chars.br);
        if info.active && info.flags & wf::GROW != 0 {
            bt.move_cstr(0, DRAG_LEFT_ICON, c_frame);
            bt.move_cstr(width - 2, DRAG_ICON, c_frame);
        }
        dc.write_line(0, size.y - 1, size.x, 1, &bt);
    }
}
