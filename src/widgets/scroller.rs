//! Прокручиваемая область (`TScroller`) и текстовый терминал (`TTextDevice` /
//! `TTerminal`).
//!
//! В C++ от `TScroller` наследуются и переопределяют `draw()`. Здесь
//! [`Scroller`] — общий «движок» прокрутки (смещение, предел, связь с
//! полосами), который встраивается в собственное представление, а
//! [`Terminal`] — готовое представление на его основе: окно, в которое
//! можно дописывать строки (журнал, вывод команды).

use std::collections::VecDeque;

use crate::color::palettes;
use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Info, Key, Mods};
use crate::geometry::{Point, Rect};
use crate::view::{default_handle_event, gf, of, Ctx, DrawCtx, View, ViewId, ViewState};

/// Движок прокрутки (`TScroller`).
#[derive(Clone, Debug, Default)]
pub struct Scroller {
    /// Смещение видимой области (`delta`).
    pub delta: Point,
    /// Размер всего содержимого (`limit`).
    pub limit: Point,
    pub h_scroll: Option<ViewId>,
    pub v_scroll: Option<ViewId>,
    last_sent: Option<(Point, Point, Point)>,
}

impl Scroller {
    pub fn new(h_scroll: Option<ViewId>, v_scroll: Option<ViewId>) -> Self {
        Scroller { h_scroll, v_scroll, ..Default::default() }
    }

    /// Прокрутить к точке (`scrollTo`), не выходя за пределы.
    pub fn scroll_to(&mut self, x: i32, y: i32, size: Point) {
        self.delta.x = x.clamp(0, (self.limit.x - size.x).max(0));
        self.delta.y = y.clamp(0, (self.limit.y - size.y).max(0));
    }

    /// Задать размер содержимого (`setLimit`).
    pub fn set_limit(&mut self, x: i32, y: i32, size: Point) {
        self.limit = Point::new(x, y);
        let d = self.delta;
        self.scroll_to(d.x, d.y, size);
    }

    /// Обработка прокрутки: полосы, колесо, клавиши-стрелки (если в фокусе).
    /// Возвращает `true`, если событие обработано.
    pub fn handle(&mut self, ev: &mut Event, size: Point, focused: bool) -> bool {
        match ev {
            Event::Broadcast(m) if m.command == cm::SCROLL_BAR_CHANGED => {
                if let Info::Scroll { id, value, .. } = m.info {
                    if Some(id) == self.v_scroll {
                        let x = self.delta.x;
                        self.scroll_to(x, value, size);
                        self.last_sent = None;
                        return false;
                    }
                    if Some(id) == self.h_scroll {
                        let y = self.delta.y;
                        self.scroll_to(value, y, size);
                        self.last_sent = None;
                        return false;
                    }
                }
                false
            }
            Event::MouseWheel(m) => {
                let (x, y) = (self.delta.x, self.delta.y + m.wheel as i32 * 3);
                self.scroll_to(x, y, size);
                ev.clear();
                true
            }
            Event::KeyDown(k) if focused && k.mods == Mods::NONE => {
                let (dx, dy) = match k.key {
                    Key::Up => (0, -1),
                    Key::Down => (0, 1),
                    Key::Left => (-1, 0),
                    Key::Right => (1, 0),
                    Key::PgUp => (0, -(size.y - 1).max(1)),
                    Key::PgDn => (0, (size.y - 1).max(1)),
                    Key::Home => (-self.delta.x, 0),
                    Key::End => (self.limit.x, 0),
                    _ => return false,
                };
                let (x, y) = (self.delta.x + dx, self.delta.y + dy);
                self.scroll_to(x, y, size);
                ev.clear();
                true
            }
            _ => false,
        }
    }

    /// Отправить полосам прокрутки текущие значения (`scrollDraw` + `setParams`).
    pub fn sync(&mut self, size: Point, ctx: &mut Ctx) {
        let key = (self.delta, self.limit, size);
        if self.last_sent == Some(key) {
            return;
        }
        self.last_sent = Some(key);
        if let Some(id) = self.v_scroll {
            ctx.message(
                cm::SCROLL_BAR_SET,
                Info::Scroll { id, value: self.delta.y, min: 0, max: (self.limit.y - size.y).max(0), page: (size.y - 1).max(1), arrow: 1 },
            );
        }
        if let Some(id) = self.h_scroll {
            ctx.message(
                cm::SCROLL_BAR_SET,
                Info::Scroll { id, value: self.delta.x, min: 0, max: (self.limit.x - size.x).max(0), page: (size.x / 2).max(1), arrow: 1 },
            );
        }
    }
}

/// Окно вывода текста (`TTerminal`): кольцевой буфер строк ограниченного
/// размера, прокрутка следует за концом, пока пользователь не прокрутил вверх.
pub struct Terminal {
    pub vs: ViewState,
    pub scroller: Scroller,
    lines: VecDeque<String>,
    /// Максимум хранимых строк (аналог размера буфера `TTerminal`).
    pub max_lines: usize,
}

impl Terminal {
    pub fn new(bounds: Rect, h_scroll: Option<ViewId>, v_scroll: Option<ViewId>, max_lines: usize) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
        vs.options |= of::SELECTABLE;
        Terminal { vs, scroller: Scroller::new(h_scroll, v_scroll), lines: VecDeque::from([String::new()]), max_lines: max_lines.max(1) }
    }

    /// Дописать текст (`do_sputn` / `operator<<`). `\n` начинает новую строку.
    pub fn write(&mut self, text: &str) {
        let at_end = self.scroller.delta.y + self.vs.size.y >= self.lines.len() as i32;
        let mut parts = text.split('\n');
        if let Some(first) = parts.next() {
            self.lines.back_mut().unwrap().push_str(first);
        }
        for p in parts {
            self.lines.push_back(p.to_string());
        }
        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
        }
        self.update_limit();
        if at_end {
            // Автопрокрутка к концу (`queEmpty`/`scrollTo` в оригинале).
            let size = self.vs.size;
            let y = self.lines.len() as i32 - size.y;
            let x = self.scroller.delta.x;
            self.scroller.scroll_to(x, y, size);
        }
    }

    /// Дописать строку с переводом строки.
    pub fn write_line(&mut self, text: &str) {
        self.write(text);
        self.write("\n");
    }

    pub fn clear(&mut self) {
        self.lines = VecDeque::from([String::new()]);
        self.scroller.delta = Point::default();
        self.update_limit();
    }

    pub fn lines(&self) -> impl Iterator<Item = &String> {
        self.lines.iter()
    }

    fn update_limit(&mut self) {
        let w = self.lines.iter().map(|l| crate::text::str_width(l)).max().unwrap_or(0) as i32;
        let size = self.vs.size;
        self.scroller.set_limit(w.max(size.x), self.lines.len() as i32, size);
    }
}

impl View for Terminal {
    crate::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let color = dc.color(1);
        let w = self.vs.size.x.max(0) as usize;
        let dx = self.scroller.delta.x.max(0) as usize;
        for y in 0..self.vs.size.y {
            let mut b = DrawBuffer::new(w);
            b.move_char(0, ' ', color, w);
            if let Some(l) = self.lines.get((self.scroller.delta.y + y) as usize) {
                let s: String = l.chars().skip(dx).collect();
                b.move_str_limited(0, &s, color, w);
            }
            dc.write_line(0, y, self.vs.size.x, 1, &b);
        }
    }

    fn palette(&self) -> &'static [u8] {
        palettes::SCROLLER
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        default_handle_event(&self.vs, ev, ctx);
        let size = self.vs.size;
        let focused = self.vs.is_focused();
        self.scroller.handle(ev, size, focused);
        self.scroller.sync(size, ctx);
    }

    fn change_bounds(&mut self, bounds: Rect) {
        self.vs.set_bounds(bounds);
        self.update_limit();
    }
}
