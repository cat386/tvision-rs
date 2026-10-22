//! Статический текст (`TStaticText`) и метка (`TLabel`).

use crate::color::palettes;
use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Info};
use crate::geometry::Rect;
use crate::text::{hotkey, str_width, to_upper, wrap_text};
use crate::view::{of, Ctx, DrawCtx, Phase, View, ViewId, ViewState};

/// Статический текст с переносом по словам.
///
/// Строка, начинающаяся с символа `'\x03'`, выравнивается по центру
/// (как в оригинале: `"\x03Hello"`).
pub struct StaticText {
    pub vs: ViewState,
    pub text: String,
}

impl StaticText {
    pub fn new(bounds: Rect, text: &str) -> Self {
        StaticText { vs: ViewState::new(bounds), text: text.into() }
    }
}

impl View for StaticText {
    crate::impl_view_basics!();

    /// Порт `TStaticText::draw`.
    fn draw(&self, dc: &mut DrawCtx) {
        let color = dc.color(1);
        let w = self.vs.size.x.max(0) as usize;
        dc.fill(self.vs.extent(), ' ', color);
        let mut y = 0;
        for para in self.text.split('\n') {
            let (center, body) = match para.strip_prefix('\x03') {
                Some(rest) => (true, rest),
                None => (false, para),
            };
            for line in wrap_text(body, w) {
                if y >= self.vs.size.y {
                    return;
                }
                let x = if center { (w.saturating_sub(str_width(&line))) / 2 } else { 0 };
                let mut b = DrawBuffer::new(w);
                b.move_char(0, ' ', color, w);
                b.move_str(x, &line, color);
                dc.write_line(0, y, self.vs.size.x, 1, &b);
                y += 1;
            }
        }
    }

    fn palette(&self) -> &'static [u8] {
        palettes::STATIC_TEXT
    }
}

/// Метка, привязанная к элементу диалога (`TLabel`).
/// Щелчок по метке или её горячая буква передаёт фокус связанному элементу;
/// пока связанный элемент в фокусе, метка подсвечена.
pub struct Label {
    pub vs: ViewState,
    pub text: String,
    pub link: Option<ViewId>,
    /// Связанный элемент в фокусе (`light`).
    pub light: bool,
}

impl Label {
    pub fn new(bounds: Rect, text: &str, link: Option<ViewId>) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.options |= of::PRE_PROCESS | of::POST_PROCESS;
        Label { vs, text: text.into(), link, light: false }
    }

    fn focus_link(&mut self, ctx: &mut Ctx) {
        if let Some(l) = self.link {
            ctx.select(l);
        }
    }
}

impl View for Label {
    crate::impl_view_basics!();

    /// Порт `TLabel::draw`.
    fn draw(&self, dc: &mut DrawCtx) {
        let color = if self.light { dc.get_color(0x0402) } else { dc.get_color(0x0301) };
        let w = self.vs.size.x.max(0) as usize;
        let mut b = DrawBuffer::new(w);
        b.move_char(0, ' ', (color & 0xFF) as u8, w);
        b.move_cstr(1, &self.text, color);
        dc.write_line(0, 0, self.vs.size.x, 1, &b);
    }

    fn palette(&self) -> &'static [u8] {
        palettes::LABEL
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        match ev {
            Event::MouseDown(_) => {
                self.focus_link(ctx);
                ev.clear();
            }
            Event::KeyDown(k) => {
                let hot = hotkey(&self.text);
                if hot.is_none() {
                    return;
                }
                let by_alt = k.alt_char() == hot;
                let by_letter = ctx.phase == Phase::PostProcess && k.text_char().map(to_upper) == hot;
                if by_alt || by_letter {
                    self.focus_link(ctx);
                    ev.clear();
                }
            }
            Event::Broadcast(m) if m.command == cm::RECEIVED_FOCUS || m.command == cm::RELEASED_FOCUS => {
                if let (Some(l), Info::Id(id)) = (self.link, &m.info) {
                    if *id == l {
                        self.light = m.command == cm::RECEIVED_FOCUS;
                    }
                }
            }
            _ => {}
        }
    }
}
