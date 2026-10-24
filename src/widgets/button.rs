//! Кнопка (`TButton`).
//!
//! Нажимается мышью, пробелом (когда в фокусе), Alt+горячая буква,
//! просто горячей буквой (если её не обработал элемент в фокусе),
//! а кнопка по умолчанию — ещё и клавишей Enter в диалоге.

use crate::color::palettes;
use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Info, Key};
use crate::geometry::Rect;
use crate::text::{cstr_width, hotkey, same_hotkey, to_upper};
use crate::view::{default_handle_event, of, sf, Ctx, DrawCtx, Phase, View, ViewState};

/// Флаги кнопки (`bfXXX`).
pub mod bf {
    pub const NORMAL: u8 = 0x00;
    /// Кнопка по умолчанию (срабатывает по Enter).
    pub const DEFAULT: u8 = 0x01;
    /// Текст выравнивается влево.
    pub const LEFT_JUST: u8 = 0x02;
    /// Вместо команды посылается широковещательное сообщение владельцу.
    pub const BROADCAST: u8 = 0x04;
    /// Кнопка забирает фокус при нажатии мышью.
    pub const GRAB_FOCUS: u8 = 0x08;
}

/// Символы тени кнопки (`TButton::shadows`).
const SHADOWS: [char; 3] = ['▄', '█', '▀'];

pub struct Button {
    pub vs: ViewState,
    /// Надпись с горячей буквой: `"~O~K"`.
    pub title: String,
    pub command: u16,
    pub flags: u8,
    /// Сейчас кнопка «по умолчанию».
    pub am_default: bool,
    /// Нажата мышью (рисуется «вдавленной»).
    down: bool,
}

impl Button {
    /// Создаёт кнопку (аналог `TButton(bounds, title, command, flags)`).
    /// Обычно высота кнопки 2 строки: надпись и тень.
    pub fn new(bounds: Rect, title: &str, command: u16, flags: u8) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.options |= of::SELECTABLE | of::FIRST_CLICK | of::PRE_PROCESS | of::POST_PROCESS;
        Button { vs, title: title.into(), command, flags, am_default: flags & bf::DEFAULT != 0, down: false }
    }

    /// Нажатие кнопки (`press`): команда в очередь или сообщение владельцу.
    pub fn press(&mut self, ctx: &mut Ctx) {
        ctx.message(cm::RECORD_HISTORY, Info::None);
        if self.flags & bf::BROADCAST != 0 {
            ctx.message(self.command, Info::Id(self.vs.id));
        } else {
            ctx.post_command(self.command, Info::Id(self.vs.id));
        }
    }

    /// Порт `TButton::drawState`.
    fn draw_state(&self, dc: &mut DrawCtx, down: bool, enabled: bool) {
        let c_button = if !enabled {
            dc.get_color(0x0404)
        } else if self.vs.is_active() && self.vs.is_selected() {
            dc.get_color(0x0703)
        } else if self.vs.is_active() && self.am_default {
            dc.get_color(0x0602)
        } else {
            dc.get_color(0x0501)
        };
        let c_shadow = dc.color(8);
        let c_lo = crate::color::lo(c_button);
        let w = self.vs.size.x.max(1) as usize;
        let s = w - 1;
        let t = self.vs.size.y / 2 - 1;
        let mut ch = ' ';
        for y in 0..(self.vs.size.y - 1).max(0) {
            let mut b = DrawBuffer::new(w);
            b.move_char(0, ' ', c_lo, w);
            b.put_attribute(0, c_shadow);
            let i;
            if down {
                b.put_attribute(1, c_shadow);
                ch = ' ';
                i = 2;
            } else {
                b.put_attribute(s, c_shadow);
                b.put_char(s, if y == 0 { SHADOWS[0] } else { SHADOWS[1] });
                ch = SHADOWS[2];
                i = 1;
            }
            if y == t && !self.title.is_empty() {
                let l = if self.flags & bf::LEFT_JUST != 0 { 1 } else { ((s as i32 - cstr_width(&self.title) as i32 - 1) / 2).max(1) as usize };
                b.move_cstr(i + l, &self.title, c_button);
            }
            dc.write_line(0, y, self.vs.size.x, 1, &b);
        }
        // Нижняя строка — тень.
        let mut b = DrawBuffer::new(w);
        b.move_char(0, ' ', c_shadow, 2);
        b.move_char(2, ch, c_shadow, s.saturating_sub(1));
        dc.write_line(0, self.vs.size.y - 1, self.vs.size.x, 1, &b);
    }
}

impl View for Button {
    crate::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let enabled = !self.vs.is_disabled() && dc.commands.is_enabled(self.command);
        self.draw_state(dc, self.down, enabled);
    }

    fn palette(&self) -> &'static [u8] {
        palettes::BUTTON
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        let id = self.vs.id;
        let enabled = !self.vs.is_disabled() && ctx.command_enabled(self.command);
        // Область щелчка — без левого столбца, правой тени и нижней строки тени.
        let e = self.vs.extent();
        let click = Rect::new(e.a.x + 1, e.a.y, e.b.x - 1, e.b.y - 1);
        if let Event::MouseDown(m) = ev {
            if !click.contains(self.vs.make_local(m.pos)) {
                ev.clear();
                return;
            }
        }
        // Выбор кнопки щелчком — только с флагом `bf::GRAB_FOCUS` (как в оригинале).
        if self.flags & bf::GRAB_FOCUS != 0 {
            default_handle_event(&self.vs, ev, ctx);
        }
        match ev {
            Event::MouseDown(_) => {
                if enabled {
                    self.down = true;
                    ctx.grab_mouse(id);
                }
                ev.clear();
            }
            Event::MouseMove(m) | Event::MouseAuto(m) if ctx.mouse_grabbed_by(id) => {
                let p = self.vs.make_local(m.pos);
                self.down = Rect::new(click.a.x, click.a.y, click.b.x + 1, click.b.y).contains(p);
                ev.clear();
            }
            Event::MouseUp(m) if ctx.mouse_grabbed_by(id) => {
                ctx.release_mouse();
                let p = self.vs.make_local(m.pos);
                let inside = Rect::new(click.a.x, click.a.y, click.b.x + 1, click.b.y).contains(p);
                self.down = false;
                if inside && enabled {
                    self.press(ctx);
                }
                ev.clear();
            }
            Event::KeyDown(k) => {
                let hot = hotkey(&self.title);
                let by_alt = matches!((k.alt_char(), hot), (Some(a), Some(h)) if same_hotkey(a, h));
                let by_letter = ctx.phase == Phase::PostProcess && hot.is_some() && k.text_char().map(to_upper) == hot;
                let by_space = self.vs.is_focused() && k.key == Key::Char(' ') && k.mods == Default::default();
                if (by_alt || by_letter || by_space) && enabled {
                    self.press(ctx);
                    ev.clear();
                }
            }
            Event::Broadcast(m) => match m.command {
                cm::DEFAULT => {
                    if self.am_default && enabled {
                        self.press(ctx);
                        ev.clear();
                    }
                }
                cm::GRAB_DEFAULT | cm::RELEASE_DEFAULT => {
                    if self.flags & bf::DEFAULT != 0 {
                        self.am_default = m.command == cm::RELEASE_DEFAULT;
                    }
                }
                cm::COMMAND_SET_CHANGED => {
                    // Кнопка с запрещённой командой недоступна для выбора.
                    let en = ctx.command_enabled(self.command);
                    self.set_state(sf::DISABLED, !en);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
