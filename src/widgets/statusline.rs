//! Строка состояния (`TStatusLine`, `TStatusDef`, `TStatusItem`).
//!
//! Показывает подсказки клавиш для текущего контекста справки и превращает
//! нажатия этих клавиш в команды. Пункты без текста не видны, но их клавиши
//! работают (так обычно задают F10 → меню, Alt+F3 → закрыть окно и т.п.).

use crate::color::palettes;
use crate::drawbuf::DrawBuffer;
use crate::event::{Event, Info, KeyEvent};
use crate::geometry::Rect;
use crate::text::cstr_width;
use crate::view::{gf, of, Ctx, DrawCtx, View, ViewState};

/// Пункт строки состояния.
#[derive(Clone, Debug)]
pub struct StatusItem {
    /// Текст с горячей частью: `"~Alt-X~ Exit"`. Пустой — пункт невидим.
    pub text: String,
    pub key: Option<KeyEvent>,
    pub command: u16,
}

impl StatusItem {
    pub fn new(text: &str, key: KeyEvent, command: u16) -> Self {
        StatusItem { text: text.into(), key: Some(key), command }
    }
}

/// Набор пунктов для диапазона контекстов справки `min..=max`.
#[derive(Clone, Debug)]
pub struct StatusDef {
    pub min: u16,
    pub max: u16,
    pub items: Vec<StatusItem>,
}

impl StatusDef {
    /// Набор для всех контекстов (0..=0xFFFF).
    pub fn all(items: Vec<StatusItem>) -> Self {
        StatusDef { min: 0, max: 0xFFFF, items }
    }

    pub fn range(min: u16, max: u16, items: Vec<StatusItem>) -> Self {
        StatusDef { min, max, items }
    }
}

/// Функция-подсказка: текст справа для данного контекста справки (`hint()`).
pub type HintFn = Box<dyn Fn(u16) -> String>;

pub struct StatusLine {
    pub vs: ViewState,
    pub defs: Vec<StatusDef>,
    /// Текущий контекст справки (устанавливает программа перед отрисовкой).
    pub help_ctx: u16,
    pub hint: Option<HintFn>,
    /// Пункт, нажатый мышью.
    pressed: Option<usize>,
}

impl StatusLine {
    pub fn new(bounds: Rect, defs: Vec<StatusDef>) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.options |= of::PRE_PROCESS;
        vs.grow_mode = gf::GROW_LO_Y | gf::GROW_HI_X | gf::GROW_HI_Y;
        StatusLine { vs, defs, help_ctx: 0, hint: None, pressed: None }
    }

    /// Пункты для текущего контекста справки (`findItems`).
    pub fn items(&self) -> &[StatusItem] {
        self.defs.iter().find(|d| d.min <= self.help_ctx && self.help_ctx <= d.max).map(|d| d.items.as_slice()).unwrap_or(&[])
    }

    /// Номер видимого пункта под позицией `x` (`itemMouseIsIn`).
    fn item_at(&self, x: i32) -> Option<usize> {
        let mut i = 0;
        for (n, it) in self.items().iter().enumerate() {
            if it.text.is_empty() {
                continue;
            }
            let k = i + cstr_width(&it.text) as i32 + 2;
            if x >= i && x < k {
                return Some(n);
            }
            i = k;
        }
        None
    }
}

impl View for StatusLine {
    crate::impl_view_basics!();

    /// Порт `TStatusLine::drawSelect`.
    fn draw(&self, dc: &mut DrawCtx) {
        let c_normal = dc.get_color(0x0301);
        let c_select = dc.get_color(0x0604);
        let c_norm_dis = dc.get_color(0x0202);
        let c_sel_dis = dc.get_color(0x0505);
        let w = self.vs.size.x.max(0) as usize;
        let mut b = DrawBuffer::new(w);
        b.move_char(0, ' ', crate::color::lo(c_normal), w);
        let mut i = 0usize;
        for (n, it) in self.items().iter().enumerate() {
            if it.text.is_empty() {
                continue;
            }
            let l = cstr_width(&it.text);
            if i + l >= w {
                break;
            }
            let enabled = dc.commands.is_enabled(it.command);
            let color = match (self.pressed == Some(n), enabled) {
                (true, true) => c_select,
                (true, false) => c_sel_dis,
                (false, true) => c_normal,
                (false, false) => c_norm_dis,
            };
            b.move_char(i, ' ', crate::color::lo(color), 1);
            b.move_cstr(i + 1, &it.text, color);
            b.move_char(i + l + 1, ' ', crate::color::lo(color), 1);
            i += l + 2;
        }
        // Подсказка справа от пунктов через разделитель «│ ».
        if let Some(h) = &self.hint {
            let text = h(self.help_ctx);
            if !text.is_empty() && i + 2 < w {
                b.move_str(i, "│ ", crate::color::lo(c_normal));
                b.move_str_limited(i + 2, &text, crate::color::lo(c_normal), w - i - 2);
            }
        }
        dc.write_line(0, 0, self.vs.size.x, 1, &b);
    }

    fn palette(&self) -> &'static [u8] {
        palettes::STATUS_LINE
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        let id = self.vs.id;
        match ev {
            Event::KeyDown(k) => {
                // Клавиша пункта → команда (событие подменяется на месте, как в оригинале).
                let cmd = self.items().iter().find(|it| it.key.map(|key| k.is(key)).unwrap_or(false)).map(|it| it.command);
                if let Some(c) = cmd {
                    if ctx.command_enabled(c) {
                        *ev = Event::command_event(c);
                    }
                }
            }
            Event::MouseDown(m) => {
                let x = self.vs.make_local(m.pos).x;
                self.pressed = self.item_at(x);
                ctx.grab_mouse(id);
                ev.clear();
            }
            Event::MouseMove(m) | Event::MouseAuto(m) if ctx.mouse_grabbed_by(id) => {
                let local = self.vs.make_local(m.pos);
                self.pressed = if local.y == 0 { self.item_at(local.x) } else { None };
                ev.clear();
            }
            Event::MouseUp(_) if ctx.mouse_grabbed_by(id) => {
                ctx.release_mouse();
                if let Some(n) = self.pressed.take() {
                    let c = self.items()[n].command;
                    // Команда 0 — пункт-надпись (подсказка без действия).
                    if c != 0 && ctx.command_enabled(c) {
                        ctx.post_command(c, Info::None);
                    }
                }
                ev.clear();
            }
            _ => {}
        }
    }
}
