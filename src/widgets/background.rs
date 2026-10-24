//! Фон рабочего стола (`TBackground`): заливка экрана узором `░`.

use crate::color::palettes;
use crate::geometry::Rect;
use crate::view::{gf, DrawCtx, View, ViewState};

pub struct Background {
    pub vs: ViewState,
    /// Символ узора (в оригинале `'\xB0'` — `░`).
    pub pattern: char,
}

impl Background {
    pub fn new(bounds: Rect, pattern: char) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
        Background { vs, pattern }
    }
}

impl View for Background {
    crate::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let attr = dc.color(1);
        dc.fill(self.vs.extent(), self.pattern, attr);
    }

    fn palette(&self) -> &'static [u8] {
        palettes::BACKGROUND
    }
}
