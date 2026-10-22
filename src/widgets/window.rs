//! Окно (`TWindow`): группа с рамкой, заголовком и номером.
//!
//! Умеет: перемещение и изменение размера мышью (за заголовок и правый нижний
//! угол) и клавиатурой (Ctrl+F5 → стрелки / Shift+стрелки → Enter),
//! распахивание (кнопка `[↑]`, двойной щелчок по заголовку, F5),
//! закрытие (кнопка `[■]`, Alt+F3), переход между элементами по Tab.

use crate::color::palettes;
use crate::event::{cm, Event, Info, Key};
use crate::geometry::{Point, Rect};
use crate::group::Group;
use crate::view::{default_handle_event, gf, of, sf, Ctx, DrawCtx, View, ViewId};
use crate::widgets::frame::{Frame, FrameInfo};

/// Флаги окна (`wfXXX`).
pub mod wf {
    /// Можно перемещать.
    pub const MOVE: u8 = 0x01;
    /// Можно менять размер.
    pub const GROW: u8 = 0x02;
    /// Есть кнопка закрытия.
    pub const CLOSE: u8 = 0x04;
    /// Есть кнопка распахивания.
    pub const ZOOM: u8 = 0x08;
}

/// Минимальный размер окна (`minWinSize`).
pub const MIN_WIN_SIZE: Point = Point::new(16, 6);

/// Состояние перетаскивания мышью.
#[derive(Clone, Copy, Debug)]
struct Drag {
    /// `true` — изменение размера, `false` — перемещение.
    grow: bool,
    /// Смещение мыши относительно начала окна (или от правого нижнего угла).
    offset: Point,
}

/// Окно.
pub struct Window {
    pub group: Group,
    /// Флаги `wf::*`.
    pub flags: u8,
    /// Номер окна (1..9 показывается в рамке и выбирается по Alt+цифра; 0 — без номера).
    pub number: i16,
    /// Заголовок.
    pub title: String,
    /// Палитра окна (синяя, бирюзовая, серая или палитра диалога).
    pub palette: &'static [u8],
    /// Границы до распахивания (`zoomRect`).
    zoom_rect: Rect,
    /// Окно распахнуто (для иконки `[↕]` в рамке).
    zoomed: bool,
    drag: Option<Drag>,
    /// Режим перемещения с клавиатуры (после Ctrl+F5).
    kb_move: bool,
}

impl Window {
    /// Создаёт окно (аналог `TWindow(bounds, title, number)`).
    pub fn new(bounds: Rect, title: &str, number: i16) -> Self {
        let mut group = Group::new(bounds);
        group.vs.state |= sf::SHADOW;
        group.vs.options |= of::SELECTABLE | of::TOP_SELECT;
        group.vs.grow_mode = gf::GROW_ALL | gf::GROW_REL;
        Window {
            group,
            flags: wf::MOVE | wf::GROW | wf::CLOSE | wf::ZOOM,
            number,
            title: title.to_string(),
            palette: palettes::BLUE_WINDOW,
            zoom_rect: bounds,
            zoomed: false,
            drag: None,
            kb_move: false,
        }
    }

    /// Вставляет элемент внутрь окна. Возвращает его id.
    pub fn insert(&mut self, v: Box<dyn View>) -> ViewId {
        self.group.insert(v)
    }

    /// Вставляет элемент внутрь окна (без явного `Box`).
    pub fn add<T: View>(&mut self, v: T) -> ViewId {
        self.group.add(v)
    }

    pub fn id(&self) -> ViewId {
        self.group.vs.id
    }

    /// Внутренняя область окна (без рамки) в локальных координатах.
    pub fn interior(&self) -> Rect {
        self.group.vs.extent().grown(-1, -1)
    }

    /// Стандартная полоса прокрутки на рамке (`standardScrollBar`).
    /// `vertical` — справа, иначе — снизу.
    pub fn standard_scroll_bar(&mut self, vertical: bool) -> ViewId {
        let e = self.group.vs.extent();
        let r = if vertical {
            Rect::new(e.b.x - 1, e.a.y + 1, e.b.x, e.b.y - 1)
        } else {
            Rect::new(e.a.x + 2, e.b.y - 1, e.b.x - 2, e.b.y)
        };
        self.group.add(crate::widgets::ScrollBar::new(r))
    }

    /// Распахнуть / восстановить (`zoom`).
    pub fn zoom(&mut self, owner: Point) {
        let (_, max) = self.size_limits(owner);
        if self.group.vs.size != max {
            self.zoom_rect = self.group.vs.bounds();
            self.group.change_bounds(Rect::from_origin_size(Point::default(), max));
        } else {
            let r = self.zoom_rect;
            self.group.change_bounds(r);
        }
        self.zoomed = self.group.vs.size == max;
    }

    /// Обработка мыши при перетаскивании (вместо `dragView` из C++).
    fn handle_drag(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        let owner = ctx.owner_size();
        let drag = match self.drag {
            Some(d) => d,
            None => return,
        };
        match ev {
            Event::MouseMove(m) | Event::MouseAuto(m) | Event::MouseDown(m) => {
                let p = m.pos;
                if drag.grow {
                    let (min, max) = self.size_limits(owner);
                    let origin = self.group.vs.origin;
                    let mut size = p - origin + drag.offset;
                    size.x = size.x.clamp(min.x, max.x.max(min.x));
                    size.y = size.y.clamp(min.y, max.y.max(min.y));
                    self.group.change_bounds(Rect::from_origin_size(origin, size));
                } else {
                    let mut o = p - drag.offset;
                    self.clamp_origin(&mut o, owner);
                    let size = self.group.vs.size;
                    self.group.change_bounds(Rect::from_origin_size(o, size));
                }
                self.zoomed = false;
                ev.clear();
            }
            Event::MouseUp(_) => {
                self.drag = None;
                ctx.release_mouse();
                self.set_state(sf::DRAGGING, false);
                ev.clear();
            }
            _ => {}
        }
    }

    /// Ограничивает позицию окна при перемещении (режимы `dm::LIMIT_*`).
    /// Окно всегда остаётся хотя бы частично видимым.
    fn clamp_origin(&self, o: &mut Point, owner: Point) {
        let size = self.group.vs.size;
        let dmode = self.group.vs.drag_mode;
        let lo_x = if dmode & crate::view::dm::LIMIT_LO_X != 0 { 0 } else { -(size.x - 2) };
        let lo_y = if dmode & crate::view::dm::LIMIT_LO_Y != 0 { 0 } else { -(size.y - 1) };
        let hi_x = if dmode & crate::view::dm::LIMIT_HI_X != 0 { owner.x - size.x } else { owner.x - 2 };
        let hi_y = if dmode & crate::view::dm::LIMIT_HI_Y != 0 { owner.y - size.y } else { owner.y - 1 };
        o.x = o.x.clamp(lo_x.min(hi_x), hi_x.max(lo_x));
        o.y = o.y.clamp(lo_y.min(hi_y), hi_y.max(lo_y));
    }

    /// Режим перемещения клавиатурой (`TView::dragView` с клавиатуры).
    fn handle_kb_move(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        let k = match ev.key() {
            Some(k) => k,
            None => return,
        };
        let owner = ctx.owner_size();
        let b = self.group.vs.bounds();
        let (dx, dy) = match k.key {
            Key::Left => (-1, 0),
            Key::Right => (1, 0),
            Key::Up => (0, -1),
            Key::Down => (0, 1),
            Key::Enter | Key::Esc => {
                self.kb_move = false;
                self.set_state(sf::DRAGGING, false);
                ev.clear();
                return;
            }
            _ => {
                ev.clear();
                return;
            }
        };
        self.zoomed = false;
        if k.mods.shift && self.flags & wf::GROW != 0 {
            // Shift+стрелки — изменить размер.
            let (min, max) = self.size_limits(owner);
            let mut size = b.size() + Point::new(dx, dy);
            size.x = size.x.clamp(min.x, max.x.max(min.x));
            size.y = size.y.clamp(min.y, max.y.max(min.y));
            self.group.change_bounds(Rect::from_origin_size(b.a, size));
        } else if self.flags & wf::MOVE != 0 {
            let mut o = b.a + Point::new(dx, dy);
            self.clamp_origin(&mut o, owner);
            self.group.change_bounds(Rect::from_origin_size(o, b.size()));
        }
        ev.clear();
    }

    /// Общая часть обработки событий окна; диалог вызывает её из своего обработчика.
    pub fn handle_window_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        let id = self.id();

        // 1. Окно захватило мышь — идёт перетаскивание.
        if self.drag.is_some() && ctx.mouse_grabbed_by(id) && ev.is_mouse() {
            self.handle_drag(ev, ctx);
            return;
        }
        // 2. Режим перемещения клавиатурой.
        if self.kb_move {
            self.handle_kb_move(ev, ctx);
            if ev.is_nothing() {
                return;
            }
        }
        // 3. Щелчок по невыбранному окну только выбирает его.
        default_handle_event(&self.group.vs, ev, ctx);
        if ev.is_nothing() {
            return;
        }
        // 4. Щелчок по рамке: кнопки, перетаскивание, изменение размера.
        if let Event::MouseDown(m) = ev {
            let local = self.group.vs.make_local(m.pos);
            let size = self.group.vs.size;
            let active = self.group.vs.is_active();
            if local.y == 0 && local.x >= 0 && local.x < size.x {
                if active && self.flags & wf::CLOSE != 0 && (2..=4).contains(&local.x) {
                    ctx.post_command(cm::CLOSE, Info::Id(id));
                    ev.clear();
                    return;
                }
                if active && self.flags & wf::ZOOM != 0 && (size.x - 5..=size.x - 3).contains(&local.x) {
                    ctx.post_command(cm::ZOOM, Info::Id(id));
                    ev.clear();
                    return;
                }
                if m.double && self.flags & wf::ZOOM != 0 {
                    ctx.post_command(cm::ZOOM, Info::Id(id));
                    ev.clear();
                    return;
                }
                if self.flags & wf::MOVE != 0 {
                    self.drag = Some(Drag { grow: false, offset: local });
                    ctx.grab_mouse(id);
                    self.set_state(sf::DRAGGING, true);
                    ev.clear();
                    return;
                }
            } else if local.y == size.y - 1 && local.x >= size.x - 2 && local.x < size.x && self.flags & wf::GROW != 0 {
                self.drag = Some(Drag { grow: true, offset: size - local });
                ctx.grab_mouse(id);
                self.set_state(sf::DRAGGING, true);
                ev.clear();
                return;
            }
        }

        // 5. Дети окна.
        self.group.handle_group_event(ev, ctx);

        // 6. Команды и клавиши самого окна (`TWindow::handleEvent`).
        let mine = |info: &Info| matches!(info, Info::None) || *info == Info::Id(id);
        match ev {
            Event::Command(m) if m.command == cm::RESIZE && self.flags & (wf::MOVE | wf::GROW) != 0 && mine(&m.info) => {
                self.kb_move = true;
                self.set_state(sf::DRAGGING, true);
                ev.clear();
            }
            Event::Command(m) if m.command == cm::CLOSE && self.flags & wf::CLOSE != 0 && mine(&m.info) => {
                ev.clear();
                if self.group.vs.state & sf::MODAL != 0 {
                    // Модальное окно закрывается как «Отмена».
                    ctx.post_command(cm::CANCEL, Info::None);
                } else if self.valid(cm::CLOSE, ctx) {
                    ctx.close(id);
                }
            }
            Event::Command(m) if m.command == cm::ZOOM && self.flags & wf::ZOOM != 0 && mine(&m.info) => {
                let owner = ctx.owner_size();
                self.zoom(owner);
                ev.clear();
            }
            Event::KeyDown(k) if k.key == Key::Tab || k.key == Key::BackTab => {
                if k.mods.ctrl || k.mods.alt {
                    return;
                }
                let forward = !(k.mods.shift || k.key == Key::BackTab);
                self.group.select_next(forward, ctx);
                ev.clear();
            }
            Event::Broadcast(m) if m.command == cm::SELECT_WINDOW_NUM => {
                if m.info == Info::Int(self.number as i64) && self.group.vs.is_selectable() {
                    ctx.select(id);
                    ev.clear();
                }
            }
            _ => {}
        }
    }

    /// Отрисовка окна: рамка, затем дети.
    pub fn draw_window(&self, dc: &mut DrawCtx) {
        let vs = &self.group.vs;
        {
            let mut fdc = dc.sub(vs.extent(), Frame::palette());
            let zoomed = self.zoomed;
            Frame::draw_frame(
                &mut fdc,
                &FrameInfo {
                    title: &self.title,
                    number: self.number,
                    flags: self.flags,
                    active: vs.is_active(),
                    dragging: vs.state & sf::DRAGGING != 0,
                    zoomed,
                },
            );
        }
        self.group.draw_children(dc);
    }

    /// `TWindow::setState`: выбранное окно становится активным.
    pub fn set_window_state(&mut self, flag: u16, enable: bool) {
        self.group.set_group_state(flag, enable);
        if flag & sf::SELECTED != 0 {
            self.group.set_group_state(sf::ACTIVE, enable);
        }
    }
}

impl View for Window {
    crate::impl_group_basics!(group);

    fn draw(&self, dc: &mut DrawCtx) {
        self.draw_window(dc);
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        self.handle_window_event(ev, ctx);
    }

    fn palette(&self) -> &'static [u8] {
        self.palette
    }

    fn set_state(&mut self, flag: u16, enable: bool) {
        self.set_window_state(flag, enable);
    }

    fn valid(&mut self, command: u16, ctx: &mut Ctx) -> bool {
        self.group.valid(command, ctx)
    }

    /// Окно не может быть меньше 16×6 (`minWinSize`).
    fn size_limits(&self, owner_size: Point) -> (Point, Point) {
        (MIN_WIN_SIZE, owner_size)
    }
}
