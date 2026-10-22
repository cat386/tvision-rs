//! Группы флажков и переключателей (`TCluster`, `TCheckBoxes`, `TRadioButtons`).
//!
//! Элементы располагаются по столбцам: высота представления = число строк
//! в столбце, лишние элементы переходят в следующий столбец.

use crate::color::palettes;
use crate::drawbuf::DrawBuffer;
use crate::event::{Event, Key, Mods};
use crate::geometry::{Point, Rect};
use crate::text::{cstr_width, hotkey, to_upper};
use crate::view::{default_handle_event, of, sf, Ctx, DrawCtx, Phase, Value, View, ViewState};

/// Вид группы.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClusterKind {
    /// Флажки: `value` — битовая маска отмеченных.
    CheckBoxes,
    /// Переключатели: `value` — номер выбранного.
    RadioButtons,
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
}

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
        Cluster { vs, kind, strings: items.iter().map(|s| s.to_string()).collect(), value: 0, sel: 0, enable_mask: u32::MAX }
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
        let (icon, marks): (&str, [char; 2]) = match self.kind {
            ClusterKind::CheckBoxes => (" [ ] ", [' ', 'X']),
            ClusterKind::RadioButtons => (" ( ) ", [' ', '•']),
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
                b.put_char(col + 2, marks[self.mark(cur) as usize]);
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
                let c = match k.alt_char() {
                    Some(c) => Some(c),
                    None if k.mods == Mods::NONE && (focused || ctx.phase == Phase::PostProcess) => k.text_char().map(to_upper),
                    None => None,
                };
                if let Some(c) = c {
                    if let Some(i) = (0..n).find(|&i| hotkey(&self.strings[i]) == Some(c)) {
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
