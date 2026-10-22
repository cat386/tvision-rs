//! Список (`TListViewer` + `TListBox`).
//!
//! Показывает строки в один или несколько столбцов, связан с полосами
//! прокрутки. Двойной щелчок или пробел «выбирает» элемент — владельцу
//! рассылается `cm::LIST_ITEM_SELECTED` с `Info::Id(id списка)`.

use crate::color::palettes;
use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Info, Key, Mods};
use crate::geometry::Rect;
use crate::view::{default_handle_event, of, sf, Ctx, DrawCtx, Value, View, ViewId, ViewState};

/// Текст пустого списка (`emptyText`).
pub const EMPTY_TEXT: &str = "<empty>";

pub struct ListBox {
    pub vs: ViewState,
    pub items: Vec<String>,
    /// Элемент с фокусом (`focused`).
    pub focused: usize,
    /// Первый видимый элемент (`topItem`).
    pub top_item: usize,
    /// Число столбцов.
    pub num_cols: usize,
    pub v_scroll: Option<ViewId>,
    pub h_scroll: Option<ViewId>,
    /// Горизонтальный сдвиг текста (значение горизонтальной полосы).
    indent: usize,
    /// Последние параметры, отправленные полосе (чтобы не слать лишнего).
    last_sent: Option<(usize, usize)>,
    /// Палитра (у списка истории своя — `cpHistoryViewer`).
    pub palette: &'static [u8],
}

impl ListBox {
    /// Аналог `TListBox(bounds, numCols, vScrollBar)`.
    pub fn new(bounds: Rect, num_cols: usize, v_scroll: Option<ViewId>) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.options |= of::SELECTABLE | of::FIRST_CLICK;
        ListBox {
            vs,
            items: Vec::new(),
            focused: 0,
            top_item: 0,
            num_cols: num_cols.max(1),
            v_scroll,
            h_scroll: None,
            indent: 0,
            last_sent: None,
            palette: palettes::LIST_VIEWER,
        }
    }

    /// Заменить содержимое (`newList`).
    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.focused = 0;
        self.top_item = 0;
        self.last_sent = None;
    }

    pub fn range(&self) -> usize {
        self.items.len()
    }

    /// Текст выбранного элемента.
    pub fn selected_text(&self) -> Option<&str> {
        self.items.get(self.focused).map(|s| s.as_str())
    }

    fn page(&self) -> usize {
        (self.vs.size.y.max(1) as usize) * self.num_cols
    }

    /// Перевести фокус на элемент (`focusItem`), прокрутив список при необходимости.
    pub fn focus_item(&mut self, item: usize) {
        if self.items.is_empty() {
            self.focused = 0;
            self.top_item = 0;
            return;
        }
        let item = item.min(self.items.len() - 1);
        self.focused = item;
        let rows = self.vs.size.y.max(1) as usize;
        if item < self.top_item {
            if self.num_cols == 1 {
                self.top_item = item;
            } else {
                self.top_item = item - item % rows;
            }
        } else if item >= self.top_item + self.page() {
            if self.num_cols == 1 {
                self.top_item = item + 1 - rows;
            } else {
                let t = item - item % rows;
                self.top_item = t.saturating_sub(rows * (self.num_cols - 1));
            }
        }
    }

    /// Синхронизирует полосу прокрутки (вместо `vScrollBar->setParams`).
    fn sync_scroll(&mut self, ctx: &mut Ctx) {
        let key = (self.focused, self.items.len());
        if self.last_sent == Some(key) {
            return;
        }
        self.last_sent = Some(key);
        if let Some(sb) = self.v_scroll {
            let page = if self.num_cols == 1 { (self.vs.size.y - 1).max(1) } else { self.vs.size.y.max(1) };
            ctx.message(
                cm::SCROLL_BAR_SET,
                Info::Scroll {
                    id: sb,
                    value: self.focused as i32,
                    min: 0,
                    max: self.items.len().saturating_sub(1) as i32,
                    page,
                    arrow: if self.num_cols == 1 { 1 } else { self.vs.size.y.max(1) },
                },
            );
        }
    }

    /// «Выбрать» элемент (`selectItem`).
    fn select_item(&mut self, ctx: &mut Ctx) {
        ctx.message(cm::LIST_ITEM_SELECTED, Info::Id(self.vs.id));
    }

    /// Номер элемента под точкой (локальные координаты).
    fn item_at(&self, x: i32, y: i32) -> usize {
        let col_width = self.vs.size.x / self.num_cols as i32 + 1;
        let col = (x / col_width.max(1)).max(0) as usize;
        self.top_item + col * self.vs.size.y.max(1) as usize + y.max(0) as usize
    }
}

impl View for ListBox {
    crate::impl_view_basics!();

    /// Порт `TListViewer::draw`.
    fn draw(&self, dc: &mut DrawCtx) {
        let active = self.vs.state & (sf::SELECTED | sf::ACTIVE) == sf::SELECTED | sf::ACTIVE;
        let (normal, focused_c, selected) = if active { (dc.color(1), dc.color(3), dc.color(4)) } else { (dc.color(2), dc.color(2), dc.color(4)) };
        let w = self.vs.size.x.max(0) as usize;
        let col_width = w / self.num_cols + 1;
        let rows = self.vs.size.y.max(0) as usize;
        for i in 0..rows {
            let mut b = DrawBuffer::new(w);
            for j in 0..self.num_cols {
                let item = j * rows + i + self.top_item;
                let cur_col = j * col_width;
                let color = if active && self.focused == item && !self.items.is_empty() {
                    focused_c
                } else if item < self.items.len() && item == self.focused {
                    selected
                } else {
                    normal
                };
                b.move_char(cur_col, ' ', color, col_width);
                if item < self.items.len() {
                    let text: String = self.items[item].chars().skip(self.indent).collect();
                    b.move_str_limited(cur_col + 1, &text, color, col_width.saturating_sub(1));
                } else if i == 0 && j == 0 {
                    b.move_str(cur_col + 1, EMPTY_TEXT, dc.color(1));
                }
                // Разделитель столбцов.
                if self.num_cols > 1 && cur_col + col_width >= 1 {
                    b.move_char(cur_col + col_width - 1, '│', dc.color(5), 1);
                }
            }
            dc.write_line(0, i as i32, self.vs.size.x, 1, &b);
        }
    }

    fn palette(&self) -> &'static [u8] {
        self.palette
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        default_handle_event(&self.vs, ev, ctx);
        let id = self.vs.id;
        let rows = self.vs.size.y.max(1) as usize;
        match ev {
            Event::MouseDown(m) => {
                let p = self.vs.make_local(m.pos);
                let item = self.item_at(p.x, p.y);
                if item < self.items.len() {
                    self.focus_item(item);
                    if m.double {
                        self.select_item(ctx);
                    }
                }
                ctx.grab_mouse(id);
                ev.clear();
            }
            Event::MouseMove(m) | Event::MouseAuto(m) if ctx.mouse_grabbed_by(id) => {
                let p = self.vs.make_local(m.pos);
                // Выход мыши за край — прокрутка.
                if p.y < 0 {
                    self.focus_item(self.focused.saturating_sub(1));
                } else if p.y >= self.vs.size.y {
                    self.focus_item(self.focused + 1);
                } else {
                    let item = self.item_at(p.x, p.y);
                    if item < self.items.len() {
                        self.focus_item(item);
                    }
                }
                ev.clear();
            }
            Event::MouseUp(_) if ctx.mouse_grabbed_by(id) => {
                ctx.release_mouse();
                ev.clear();
            }
            Event::MouseWheel(m) => {
                let n = self.items.len();
                if n > 0 {
                    let new_top = (self.top_item as i64 + m.wheel as i64 * 3).clamp(0, n.saturating_sub(rows) as i64) as usize;
                    self.top_item = new_top;
                    if self.focused < self.top_item {
                        self.focused = self.top_item;
                    } else if self.focused >= self.top_item + self.page() {
                        self.focused = self.top_item + self.page() - 1;
                    }
                }
                ev.clear();
            }
            Event::KeyDown(k) if self.vs.is_focused() && !self.items.is_empty() => {
                let f = self.focused;
                let n = self.items.len();
                let page = self.page();
                let new = match (k.key, k.mods) {
                    (Key::Up, Mods::NONE) => Some(f.saturating_sub(1)),
                    (Key::Down, Mods::NONE) => Some((f + 1).min(n - 1)),
                    (Key::Left, Mods::NONE) if self.num_cols > 1 => Some(f.saturating_sub(rows)),
                    (Key::Right, Mods::NONE) if self.num_cols > 1 => Some((f + rows).min(n - 1)),
                    (Key::PgUp, Mods::NONE) => Some(f.saturating_sub(page)),
                    (Key::PgDn, Mods::NONE) => Some((f + page).min(n - 1)),
                    (Key::Home, Mods::NONE) => Some(self.top_item),
                    (Key::End, Mods::NONE) => Some((self.top_item + page - 1).min(n - 1)),
                    (Key::PgUp, Mods::CTRL) | (Key::Home, Mods::CTRL) => Some(0),
                    (Key::PgDn, Mods::CTRL) | (Key::End, Mods::CTRL) => Some(n - 1),
                    (Key::Char(' '), Mods::NONE) => {
                        self.select_item(ctx);
                        None
                    }
                    _ => {
                        self.sync_scroll(ctx);
                        return;
                    }
                };
                if let Some(i) = new {
                    self.focus_item(i);
                }
                ev.clear();
            }
            Event::Broadcast(m) => match (&m.command, &m.info) {
                (&cm::SCROLL_BAR_CHANGED, Info::Scroll { id: sb, value, .. }) if Some(*sb) == self.v_scroll => {
                    self.focus_item((*value).max(0) as usize);
                    self.last_sent = Some((self.focused, self.items.len()));
                }
                (&cm::SCROLL_BAR_CHANGED, Info::Scroll { id: sb, value, .. }) if Some(*sb) == self.h_scroll => {
                    self.indent = (*value).max(0) as usize;
                }
                (&cm::SCROLL_BAR_CLICKED, Info::Id(sb)) if Some(*sb) == self.v_scroll || Some(*sb) == self.h_scroll => {
                    if !self.vs.is_selected() {
                        ctx.select(id);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        self.sync_scroll(ctx);
    }

    fn has_data(&self) -> bool {
        true
    }

    /// Данные списка — номер выбранного элемента.
    fn get_data(&self) -> Value {
        Value::Int(self.focused as i64)
    }

    fn set_data(&mut self, value: &Value) {
        if let Value::Int(i) = value {
            self.focus_item((*i).max(0) as usize);
            self.last_sent = None;
        }
    }
}
