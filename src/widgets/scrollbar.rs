//! Полоса прокрутки (`TScrollBar`).
//!
//! В C++ владелец (например, список) вызывал `scrollBar->setParams()` напрямую,
//! а полоса сообщала об изменениях вызовом `message(owner, cmScrollBarChanged)`.
//! Здесь связь двусторонняя через широковещательные сообщения владельцу:
//! * полоса → соседям: `cm::SCROLL_BAR_CHANGED` с `Info::Scroll { id: полоса, value, .. }`;
//! * сосед → полосе: `cm::SCROLL_BAR_SET` с `Info::Scroll { id: полоса, value, min, max, page, arrow }`.

use crate::color::palettes;
use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Info};
use crate::geometry::Rect;
use crate::view::{gf, Ctx, DrawCtx, View, ViewState};

/// Части полосы (`sbXXX`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    ArrowLo,
    PageLo,
    Indicator,
    PageHi,
    ArrowHi,
}

/// Символы полосы: стрелка «к началу», стрелка «к концу», дорожка, бегунок, пустая дорожка.
const V_CHARS: [char; 5] = ['▲', '▼', '▒', '■', '▓'];
const H_CHARS: [char; 5] = ['◄', '►', '▒', '■', '▓'];

pub struct ScrollBar {
    pub vs: ViewState,
    pub value: i32,
    pub min: i32,
    pub max: i32,
    /// Шаг «страницы».
    pub pg_step: i32,
    /// Шаг стрелки.
    pub ar_step: i32,
    vertical: bool,
    /// Удерживаемая мышью часть (для автоповтора и перетаскивания бегунка).
    held: Option<Part>,
}

impl ScrollBar {
    /// Создаёт полосу; вертикальная, если ширина равна 1.
    pub fn new(bounds: Rect) -> Self {
        let mut vs = ViewState::new(bounds);
        let vertical = bounds.width() == 1;
        vs.grow_mode = if vertical { gf::GROW_LO_X | gf::GROW_HI_X | gf::GROW_HI_Y } else { gf::GROW_LO_Y | gf::GROW_HI_X | gf::GROW_HI_Y };
        ScrollBar { vs, value: 0, min: 0, max: 0, pg_step: 1, ar_step: 1, vertical, held: None }
    }

    /// Установить параметры (`setParams`). Возвращает `true`, если значение изменилось.
    pub fn set_params(&mut self, value: i32, min: i32, max: i32, pg_step: i32, ar_step: i32) -> bool {
        let max = max.max(min);
        let value = value.clamp(min, max);
        let changed = value != self.value;
        self.value = value;
        self.min = min;
        self.max = max;
        self.pg_step = pg_step;
        self.ar_step = ar_step;
        changed
    }

    /// Длина полосы вдоль оси (`getSize`).
    fn get_size(&self) -> i32 {
        let s = if self.vertical { self.vs.size.y } else { self.vs.size.x };
        s.max(3)
    }

    /// Позиция бегунка (`getPos`).
    fn get_pos(&self) -> i32 {
        let r = self.max - self.min;
        if r == 0 {
            1
        } else {
            (((self.value - self.min) as i64 * (self.get_size() - 3) as i64 + (r / 2) as i64) / r as i64) as i32 + 1
        }
    }

    /// Часть полосы под точкой (локальные координаты).
    fn part_at(&self, x: i32, y: i32) -> Option<Part> {
        if !self.vs.extent().contains(crate::geometry::Point::new(x, y)) {
            return None;
        }
        let mark = if self.vertical { y } else { x };
        let s = self.get_size() - 1;
        let p = self.get_pos();
        Some(if mark == p {
            Part::Indicator
        } else if mark < 1 {
            Part::ArrowLo
        } else if mark < p {
            Part::PageLo
        } else if mark < s {
            Part::PageHi
        } else {
            Part::ArrowHi
        })
    }

    /// Изменение значения на шаг части `part` (`scrollStep`).
    fn step(&self, part: Part) -> i32 {
        match part {
            Part::ArrowLo => -self.ar_step,
            Part::ArrowHi => self.ar_step,
            Part::PageLo => -self.pg_step,
            Part::PageHi => self.pg_step,
            Part::Indicator => 0,
        }
    }

    /// Установить значение и сообщить соседям (`setValue` + `scrollDraw`).
    fn set_value_notify(&mut self, v: i32, ctx: &mut Ctx) {
        let v = v.clamp(self.min, self.max);
        if v != self.value {
            self.value = v;
            ctx.message(
                cm::SCROLL_BAR_CHANGED,
                Info::Scroll { id: self.vs.id, value: v, min: self.min, max: self.max, page: self.pg_step, arrow: self.ar_step },
            );
        }
    }
}

impl View for ScrollBar {
    crate::impl_view_basics!();

    /// Порт `TScrollBar::drawPos`.
    fn draw(&self, dc: &mut DrawCtx) {
        let chars = if self.vertical { V_CHARS } else { H_CHARS };
        let s = self.get_size() - 1;
        let mut b = DrawBuffer::new((s + 1) as usize);
        b.move_char(0, chars[0], dc.color(2), 1);
        if self.max == self.min {
            b.move_char(1, chars[4], dc.color(1), (s - 1).max(0) as usize);
        } else {
            b.move_char(1, chars[2], dc.color(1), (s - 1).max(0) as usize);
            b.move_char(self.get_pos() as usize, chars[3], dc.color(3), 1);
        }
        b.move_char(s as usize, chars[1], dc.color(2), 1);
        if self.vertical {
            for (y, c) in b.cells.iter().enumerate() {
                dc.write_cells(0, y as i32, std::slice::from_ref(c));
            }
        } else {
            dc.write_line(0, 0, s + 1, 1, &b);
        }
    }

    fn palette(&self) -> &'static [u8] {
        palettes::SCROLL_BAR
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        let id = self.vs.id;
        match ev {
            Event::MouseDown(m) => {
                // Сообщаем соседям, что по полосе щёлкнули (список заберёт фокус).
                ctx.message(cm::SCROLL_BAR_CLICKED, Info::Id(id));
                let p = self.vs.make_local(m.pos);
                if let Some(part) = self.part_at(p.x, p.y) {
                    self.held = Some(part);
                    let d = self.step(part);
                    if d != 0 {
                        self.set_value_notify(self.value + d, ctx);
                    }
                    ctx.grab_mouse(id);
                }
                ev.clear();
            }
            Event::MouseAuto(_) | Event::MouseMove(_) if ctx.mouse_grabbed_by(id) => {
                let is_auto = matches!(ev, Event::MouseAuto(_));
                let m = *ev.mouse().unwrap();
                let p = self.vs.make_local(m.pos);
                match self.held {
                    Some(Part::Indicator) => {
                        // Перетаскивание бегунка.
                        let s = self.get_size() - 1;
                        let mark = if self.vertical { p.y } else { p.x };
                        let pos = mark.clamp(1, (s - 1).max(1));
                        let span = (s - 2).max(1);
                        let v = (((pos - 1) as i64 * (self.max - self.min) as i64 + (span / 2) as i64) / span as i64) as i32 + self.min;
                        self.set_value_notify(v, ctx);
                    }
                    Some(part) if is_auto => {
                        // Автоповтор, пока мышь над той же частью.
                        if self.part_at(p.x, p.y) == Some(part) {
                            let d = self.step(part);
                            self.set_value_notify(self.value + d, ctx);
                        }
                    }
                    _ => {}
                }
                ev.clear();
            }
            Event::MouseUp(_) if ctx.mouse_grabbed_by(id) => {
                ctx.release_mouse();
                self.held = None;
                ev.clear();
            }
            Event::MouseWheel(m) => {
                let d = m.wheel as i32 * 3 * self.ar_step;
                self.set_value_notify(self.value + d, ctx);
                ev.clear();
            }
            Event::Broadcast(msg) if msg.command == cm::SCROLL_BAR_SET => {
                if let Info::Scroll { id: target, value, min, max, page, arrow } = msg.info {
                    if target == id {
                        self.set_params(value, min, max, page, arrow);
                        ev.clear();
                    }
                }
            }
            _ => {}
        }
    }
}
