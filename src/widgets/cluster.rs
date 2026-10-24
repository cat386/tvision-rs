//! Группы флажков и переключателей (`TCluster`, `TCheckBoxes`,
//! `TRadioButtons`, `TMultiCheckBoxes`).
//!
//! Элементы располагаются по столбцам: высота представления = число строк
//! в столбце, лишние элементы переходят в следующий столбец.

use crate::color::palettes;
use crate::drawbuf::DrawBuffer;
use crate::event::{Event, Key, Mods};
use crate::geometry::{Point, Rect};
use crate::text::{cstr_width, hotkey, same_hotkey, to_upper};
use crate::view::{default_handle_event, of, sf, Ctx, DrawCtx, Phase, Value, View, ViewState};

/// Вид группы.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClusterKind {
    /// Флажки: `value` — битовая маска отмеченных.
    CheckBoxes,
    /// Переключатели: `value` — номер выбранного.
    RadioButtons,
    /// Флажки с несколькими состояниями (`TMultiCheckBoxes` из TV 2.0):
    /// на каждый элемент отводится несколько бит `value`, а символ в
    /// скобках берётся из строки состояний.
    MultiCheckBoxes,
}

/// Флаги `TMultiCheckBoxes`: старший байт — число бит на элемент, младший —
/// маска значения элемента.
pub mod cf {
    pub const ONE_BIT: u16 = 0x0101;
    pub const TWO_BITS: u16 = 0x0203;
    pub const FOUR_BITS: u16 = 0x040F;
    pub const EIGHT_BITS: u16 = 0x08FF;
}

pub struct Cluster {
    pub vs: ViewState,
    pub kind: ClusterKind,
    pub strings: Vec<String>,
    /// Значение (маска для флажков, номер для переключателей).
    pub value: u32,
    /// Элемент под курсором.
    pub sel: usize,
    /// Маска доступных элементов (бит 1 — элемент доступен).
    pub enable_mask: u32,
    /// Только для `MultiCheckBoxes`: число состояний, флаги `cf::*` и
    /// символы состояний (`" X"`, `" +-"` ...).
    pub sel_range: u8,
    pub mcb_flags: u16,
    pub states: Vec<char>,
}

/// Флажки с несколькими состояниями — тот же [`Cluster`].
pub type MultiCheckBoxes = Cluster;

/// Флажки (`TCheckBoxes`). Это тот же [`Cluster`] — псевдоним для удобства.
pub type CheckBoxes = Cluster;
/// Переключатели (`TRadioButtons`).
pub type RadioButtons = Cluster;

impl Cluster {
    fn new(bounds: Rect, items: &[&str], kind: ClusterKind) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.options |= of::SELECTABLE | of::FIRST_CLICK | of::PRE_PROCESS | of::POST_PROCESS;
        vs.state |= sf::CURSOR_VIS;
        vs.set_cursor(2, 0);
        Cluster {
            vs,
            kind,
            strings: items.iter().map(|s| s.to_string()).collect(),
            value: 0,
            sel: 0,
            enable_mask: u32::MAX,
            sel_range: 2,
            mcb_flags: cf::ONE_BIT,
            states: vec![' ', 'X'],
        }
    }

    /// Флажки с несколькими состояниями (`TMultiCheckBoxes`): `sel_range` —
    /// сколько состояний у элемента, `flags` — `cf::*`, `states` — символ
    /// для каждого состояния.
    pub fn multi_check_boxes(bounds: Rect, items: &[&str], sel_range: u8, flags: u16, states: &str) -> Self {
        let mut c = Self::new(bounds, items, ClusterKind::MultiCheckBoxes);
        c.sel_range = sel_range.max(1);
        c.mcb_flags = flags;
        c.states = states.chars().collect();
        c
    }

    /// Состояние элемента многосостоятельных флажков (`multiMark`).
    pub fn multi_mark(&self, item: usize) -> u32 {
        let bits = (self.mcb_flags >> 8) as usize;
        let shift = item * bits;
        if shift >= 32 {
            return 0;
        }
        (self.value >> shift) & (self.mcb_flags & 0xFF) as u32
    }

    /// Группа флажков.
    pub fn check_boxes(bounds: Rect, items: &[&str]) -> Self {
        Self::new(bounds, items, ClusterKind::CheckBoxes)
    }

    /// Группа переключателей.
    pub fn radio_buttons(bounds: Rect, items: &[&str]) -> Self {
        Self::new(bounds, items, ClusterKind::RadioButtons)
    }

    /// Отмечен ли элемент (`mark`).
    pub fn mark(&self, item: usize) -> bool {
        match self.kind {
            ClusterKind::CheckBoxes => item < 32 && self.value & (1 << item) != 0,
            ClusterKind::RadioButtons => item as u32 == self.value,
            ClusterKind::MultiCheckBoxes => self.multi_mark(item) != 0,
        }
    }

    /// Нажатие на элемент (`press`).
    pub fn press(&mut self, item: usize) {
        match self.kind {
            ClusterKind::CheckBoxes => {
                if item < 32 {
                    self.value ^= 1 << item;
                }
            }
            ClusterKind::RadioButtons => self.value = item as u32,
            ClusterKind::MultiCheckBoxes => {
                // Состояние «прокручивается» вниз: 0 → selRange-1 → ... → 0.
                let bits = (self.mcb_flags >> 8) as usize;
                let shift = item * bits;
                if shift >= 32 {
                    return;
                }
                let mask = (self.mcb_flags & 0xFF) as u32;
                let cur = ((self.value >> shift) & mask) as i32 - 1;
                let cur = if cur < 0 || cur >= self.sel_range as i32 { self.sel_range as i32 - 1 } else { cur };
                self.value = (self.value & !(mask << shift)) | ((cur as u32) << shift);
            }
        }
    }

    /// Курсор переместился на элемент (`movedTo`): переключатель следует за курсором.
    fn moved_to(&mut self, item: usize) {
        if self.kind == ClusterKind::RadioButtons {
            self.value = item as u32;
        }
    }

    /// Доступен ли элемент (`buttonState`).
    pub fn button_state(&self, item: usize) -> bool {
        item >= 32 || self.enable_mask & (1 << item) != 0
    }

    /// Разрешить/запретить элементы по маске (`setButtonState`).
    pub fn set_button_state(&mut self, mask: u32, enable: bool) {
        if enable {
            self.enable_mask |= mask;
        } else {
            self.enable_mask &= !mask;
        }
        // Если все элементы запрещены — группа недоступна для выбора.
        let any = (0..self.strings.len()).any(|i| self.button_state(i));
        self.vs.options = if any { self.vs.options | of::SELECTABLE } else { self.vs.options & !of::SELECTABLE };
    }

    fn rows(&self) -> usize {
        self.vs.size.y.max(1) as usize
    }

    /// X-координата столбца, в котором стоит элемент (`column`).
    fn column(&self, item: usize) -> i32 {
        let rows = self.rows();
        if item < rows {
            return 0;
        }
        let mut width = 0;
        let mut col = -6;
        let mut l = 0;
        for i in 0..=item {
            if i % rows == 0 {
                col += width + 6;
                width = 0;
            }
            if i < self.strings.len() {
                l = cstr_width(&self.strings[i]) as i32;
            }
            if l > width {
                width = l;
            }
        }
        col
    }

    fn row(&self, item: usize) -> i32 {
        (item % self.rows()) as i32
    }

    /// Элемент под точкой (локальные координаты) (`findSel`).
    fn find_sel(&self, p: Point) -> Option<usize> {
        if !self.vs.extent().contains(p) {
            return None;
        }
        let rows = self.rows();
        let mut i = 0;
        // Ищем столбец, в который попала точка.
        while i < self.strings.len() && p.x >= self.column(i + rows) {
            i += rows;
        }
        let s = i + p.y as usize;
        if s < self.strings.len() {
            Some(s)
        } else {
            None
        }
    }

    fn move_sel(&mut self, s: usize) {
        if s < self.strings.len() {
            self.sel = s;
            self.moved_to(s);
        }
    }

    fn update_cursor(&mut self) {
        let x = self.column(self.sel) + 2;
        let y = self.row(self.sel);
        self.vs.set_cursor(x, y);
    }
}

impl View for Cluster {
    crate::impl_view_basics!();

    /// Порт `TCluster::drawMultiBox`.
    fn draw(&self, dc: &mut DrawCtx) {
        let icon = if self.kind == ClusterKind::RadioButtons { " ( ) " } else { " [ ] " };
        let mark_char = |i: usize| -> char {
            match self.kind {
                ClusterKind::CheckBoxes => {
                    if self.mark(i) {
                        'X'
                    } else {
                        ' '
                    }
                }
                ClusterKind::RadioButtons => {
                    if self.mark(i) {
                        '•'
                    } else {
                        ' '
                    }
                }
                ClusterKind::MultiCheckBoxes => self.states.get(self.multi_mark(i) as usize).copied().unwrap_or(' '),
            }
        };
        let c_norm = dc.get_color(0x0301);
        let c_sel = dc.get_color(0x0402);
        let c_dis = dc.get_color(0x0505);
        let w = self.vs.size.x.max(0) as usize;
        let rows = self.rows();
        for i in 0..rows {
            let mut b = DrawBuffer::new(w);
            b.move_char(0, ' ', (c_norm & 0xFF) as u8, w);
            let cols = (self.strings.len().saturating_sub(1)) / rows + 1;
            for j in 0..=cols {
                let cur = j * rows + i;
                if cur >= self.strings.len() {
                    continue;
                }
                let col = self.column(cur);
                if col < 0 || col as usize >= w {
                    continue;
                }
                let col = col as usize;
                let color = if !self.button_state(cur) {
                    c_dis
                } else if cur == self.sel && self.vs.is_selected() {
                    c_sel
                } else {
                    c_norm
                };
                b.move_char(col, ' ', (color & 0xFF) as u8, w - col);
                b.move_cstr(col, icon, color);
                b.put_char(col + 2, mark_char(cur));
                b.move_cstr(col + 5, &self.strings[cur], color);
            }
            dc.write_line(0, i as i32, self.vs.size.x, 1, &b);
        }
    }

    fn palette(&self) -> &'static [u8] {
        palettes::CLUSTER
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        default_handle_event(&self.vs, ev, ctx);
        let n = self.strings.len();
        if n == 0 {
            return;
        }
        match ev {
            Event::MouseDown(m) => {
                let p = self.vs.make_local(m.pos);
                if let Some(s) = self.find_sel(p) {
                    if self.button_state(s) {
                        self.sel = s;
                        self.press(s);
                        self.moved_to(s);
                    }
                }
                ev.clear();
            }
            Event::KeyDown(k) => {
                let k = *k;
                let focused = self.vs.is_focused();
                let rows = self.rows();
                // Навигация стрелками — только когда группа в фокусе.
                if focused && k.mods == Mods::NONE {
                    let step = |s: usize, d: isize| -> usize { ((s as isize + d).rem_euclid(n as isize)) as usize };
                    let target = match k.key {
                        Key::Down => Some(1isize),
                        Key::Up => Some(-1),
                        Key::Right => Some(rows as isize),
                        Key::Left => Some(-(rows as isize)),
                        _ => None,
                    };
                    if let Some(d) = target {
                        let mut s = self.sel;
                        for _ in 0..n {
                            s = step(s, d);
                            if self.button_state(s) {
                                break;
                            }
                        }
                        self.move_sel(s);
                        self.update_cursor();
                        ev.clear();
                        return;
                    }
                    if k.key == Key::Char(' ') {
                        if self.button_state(self.sel) {
                            self.press(self.sel);
                        }
                        ev.clear();
                        return;
                    }
                }
                // Горячая буква элемента: Alt+буква всегда; просто буква —
                // в фазе пост-обработки или когда группа в фокусе.
                let (c, by_alt) = match k.alt_char() {
                    Some(c) => (Some(c), true),
                    None if k.mods == Mods::NONE && (focused || ctx.phase == Phase::PostProcess) => (k.text_char().map(to_upper), false),
                    None => (None, false),
                };
                if let Some(c) = c {
                    // Alt+буква — без учёта раскладки (в Windows она приходит латинской).
                    let hit = |i: usize| match hotkey(&self.strings[i]) {
                        Some(h) => h == c || (by_alt && same_hotkey(c, h)),
                        None => false,
                    };
                    if let Some(i) = (0..n).find(|&i| hit(i)) {
                        if self.button_state(i) {
                            if !self.vs.is_selected() {
                                ctx.select(self.vs.id);
                            }
                            self.sel = i;
                            self.press(i);
                            self.moved_to(i);
                            self.update_cursor();
                            ev.clear();
                        }
                    }
                }
            }
            _ => {}
        }
        self.update_cursor();
    }

    fn has_data(&self) -> bool {
        true
    }

    fn get_data(&self) -> Value {
        Value::U32(self.value)
    }

    fn set_data(&mut self, value: &Value) {
        if let Value::U32(v) = value {
            self.value = *v;
            if self.kind == ClusterKind::RadioButtons {
                self.sel = (*v as usize).min(self.strings.len().saturating_sub(1));
            }
            self.update_cursor();
        }
    }
}
