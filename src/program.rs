//! Программа (`TProgram` / `TApplication`): корень дерева представлений,
//! главный цикл событий, модальное выполнение диалогов, меню.
//!
//! ```no_run
//! use tvision::*;
//!
//! struct App;
//! impl AppHandler for App {
//!     fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
//!         if ev.command() == Some(cm::NEW) {
//!             let w = Window::new(Rect::new(5, 3, 45, 15), "Окно", 1);
//!             app.insert_window(w);
//!             ev.clear();
//!         }
//!     }
//! }
//!
//! fn main() -> std::io::Result<()> {
//!     let menu = Menu::new().submenu("~F~ile", Menu::new().item_key("~N~ew", cm::NEW, kb::F2, "F2"));
//!     let status = vec![StatusDef::all(vec![StatusItem::new("~Alt+X~ Exit", kb::ALT_X, cm::QUIT)])];
//!     let mut app = Program::with_terminal(Some(menu), Some(status))?;
//!     app.run(&mut App);
//!     Ok(())
//! }
//! ```

use std::time::{Duration, Instant};

use crate::color::palettes;
use crate::drawbuf::ScreenBuffer;
use crate::event::{cm, Event, Info, Message, MouseEvent};
use crate::geometry::{Point, Rect};
use crate::group::Group;
use crate::platform::{default_backend, Backend, RawEvent};
use crate::view::{sf, Ctx, DrawCtx, Phase, Request, View, ViewId};
use crate::widgets::desktop::Desktop;
use crate::widgets::menu::{Menu, MenuBar, MenuOutcome, MenuSession};
use crate::widgets::statusline::{StatusDef, StatusLine};
use crate::widgets::window::{wf, Window};

/// Обработчик событий приложения — то, что в C++ было переопределённым
/// `TApplication::handleEvent` и `idle`. Получает доступ ко всей программе,
/// поэтому отсюда можно открывать диалоги (`exec_view`) и окна.
pub trait AppHandler {
    /// Событие, которое не обработали представления (обычно — команды меню).
    fn handle_event(&mut self, _app: &mut Program, _ev: &mut Event) {}
    /// Вызывается, когда событий нет (`idle`).
    fn idle(&mut self, _app: &mut Program) {}
}

/// Обработчик «ничего не делать».
pub struct NoHandler;
impl AppHandler for NoHandler {}

/// Интервал двойного щелчка (`doubleDelay`).
const DOUBLE_CLICK: Duration = Duration::from_millis(400);

pub struct Program {
    /// Корневая группа: рабочий стол, строка состояния, строка меню.
    pub group: Group,
    /// Контекст событий (очередь, команды, буфер обмена...).
    pub ctx: Ctx,
    backend: Box<dyn Backend>,
    screen: ScreenBuffer,
    /// Палитра приложения (цветная, ч/б или монохромная).
    pub palette: &'static [u8],
    pub desktop_id: ViewId,
    pub menu_bar_id: Option<ViewId>,
    pub status_line_id: Option<ViewId>,
    /// Открытое меню.
    menu: Option<MenuSession>,
    /// Модальные представления (вершина — текущее).
    modal_stack: Vec<ViewId>,
    // Состояние мыши для превращения «сырых» событий в нажатия/отпускания.
    mouse_buttons: u8,
    last_mouse: MouseEvent,
    last_click: Option<(Instant, Point)>,
    running: bool,
}

impl Program {
    /// Создаёт программу в настоящем терминале.
    pub fn with_terminal(menu: Option<Menu>, status: Option<Vec<StatusDef>>) -> std::io::Result<Self> {
        Ok(Program::new(default_backend()?, menu, status))
    }

    /// Создаёт программу с заданным терминалом (например, `HeadlessBackend` для тестов).
    /// Аналог конструктора `TProgram` с `initMenuBar`, `initStatusLine`, `initDesktop`.
    pub fn new(backend: Box<dyn Backend>, menu: Option<Menu>, status: Option<Vec<StatusDef>>) -> Self {
        let size = backend.size();
        let mut group = Group::new(Rect::new(0, 0, size.x, size.y));
        group.vs.state = sf::VISIBLE | sf::SELECTED | sf::FOCUSED | sf::ACTIVE | sf::MODAL | sf::EXPOSED;

        let top = if menu.is_some() { 1 } else { 0 };
        let bottom = if status.is_some() { size.y - 1 } else { size.y };

        let desktop_id = group.add(Desktop::new(Rect::new(0, top, size.x, bottom)));
        let status_line_id = status.map(|defs| group.add(StatusLine::new(Rect::new(0, size.y - 1, size.x, size.y), defs)));
        let menu_bar_id = menu.map(|m| group.add(MenuBar::new(Rect::new(0, 0, size.x, 1), m)));

        let mut ctx = Ctx::new();
        ctx.screen_size = size;
        // Команды окон запрещены, пока нет активного окна (`initCommands`).
        for c in [cm::ZOOM, cm::CLOSE, cm::RESIZE, cm::NEXT, cm::PREV] {
            ctx.disable_command(c);
        }

        Program {
            group,
            ctx,
            backend,
            screen: ScreenBuffer::new(size.x.max(1) as usize, size.y.max(1) as usize),
            palette: palettes::APP_COLOR,
            desktop_id,
            menu_bar_id,
            status_line_id,
            menu: None,
            modal_stack: Vec::new(),
            mouse_buttons: 0,
            last_mouse: MouseEvent::default(),
            last_click: None,
            running: true,
        }
    }

    // ------------------------------------------------------------------
    // Доступ к частям программы
    // ------------------------------------------------------------------

    pub fn desktop(&self) -> &Desktop {
        self.group.find::<Desktop>(self.desktop_id).expect("рабочий стол")
    }

    pub fn desktop_mut(&mut self) -> &mut Desktop {
        let id = self.desktop_id;
        self.group.find_mut::<Desktop>(id).expect("рабочий стол")
    }

    pub fn status_line_mut(&mut self) -> Option<&mut StatusLine> {
        let id = self.status_line_id?;
        self.group.find_mut::<StatusLine>(id)
    }

    /// Найти представление в любом месте дерева по id и типу.
    pub fn find<T: View>(&self, id: ViewId) -> Option<&T> {
        self.group.find::<T>(id)
    }

    pub fn find_mut<T: View>(&mut self, id: ViewId) -> Option<&mut T> {
        self.group.find_mut::<T>(id)
    }

    fn desktop_index(&self) -> usize {
        self.group.index_of(self.desktop_id).expect("рабочий стол")
    }

    /// Вставить окно на рабочий стол (`insertWindow`). Возвращает id окна.
    pub fn insert_window<T: View>(&mut self, w: T) -> ViewId {
        self.insert_window_boxed(Box::new(w))
    }

    pub fn insert_window_boxed(&mut self, mut w: Box<dyn View>) -> ViewId {
        if !w.valid(cm::VALID, &mut self.ctx) {
            return 0;
        }
        let id = w.vs().id;
        let di = self.desktop_index();
        {
            let desk = self.group.children[di].as_group_mut().unwrap();
            desk.insert(w);
            // Новое окно — текущее, остальным разошлём смену фокуса.
            self.ctx.depth = 1;
            desk.select_id(id, &mut self.ctx);
        }
        self.sync_view(id);
        id
    }

    /// Рассылает внутрь представления `cm::COMMAND_SET_CHANGED`, чтобы его
    /// элементы обновили своё состояние (доступность кнопок, полосы прокрутки).
    fn sync_view(&mut self, id: ViewId) {
        let di = self.desktop_index();
        let desk = self.group.children[di].as_group_mut().unwrap();
        if let Some(i) = desk.index_of(id) {
            self.ctx.depth = 1;
            self.ctx.owner_sizes = vec![self.ctx.screen_size];
            let mut e = Event::Broadcast(Message::new(cm::COMMAND_SET_CHANGED));
            desk.dispatch(i, &mut e, &mut self.ctx);
            if let Some(g) = desk.children[i].as_group_mut() {
                self.ctx.depth = 2;
                g.announce_focus(&mut self.ctx);
            }
            self.ctx.owner_sizes.clear();
        }
        self.ctx.depth = 0;
    }

    // Команды (`enableCommand` и т.п.)
    pub fn enable_command(&mut self, cmd: u16) {
        self.ctx.enable_command(cmd)
    }
    pub fn disable_command(&mut self, cmd: u16) {
        self.ctx.disable_command(cmd)
    }
    pub fn command_enabled(&self, cmd: u16) -> bool {
        self.ctx.command_enabled(cmd)
    }

    /// Поставить событие в очередь.
    pub fn put_event(&mut self, ev: Event) {
        self.ctx.put_event(ev);
    }

    /// Завершить главный цикл.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Последний нарисованный кадр в виде текста (удобно для тестов).
    pub fn screen_text(&self) -> String {
        self.screen.to_text()
    }

    pub fn screen(&self) -> &ScreenBuffer {
        &self.screen
    }

    // ------------------------------------------------------------------
    // Главный цикл
    // ------------------------------------------------------------------

    /// Главный цикл программы (`TProgram::run`). Возвращается после `cm::QUIT`.
    pub fn run(&mut self, handler: &mut dyn AppHandler) {
        self.running = true;
        while self.running {
            self.step(handler);
        }
    }

    /// Одна итерация: отрисовка, получение события, обработка.
    /// Возвращает `false`, если событий не было.
    pub fn step(&mut self, handler: &mut dyn AppHandler) -> bool {
        self.redraw();
        let ev = self.get_event();
        let had = !ev.is_nothing();
        self.handle(ev, Some(handler));
        had
    }

    /// Обрабатывает все накопившиеся события (для тестов с `HeadlessBackend`).
    pub fn pump(&mut self, handler: &mut dyn AppHandler) {
        let mut idle = 0;
        while self.running && idle < 2 {
            if self.step(handler) {
                idle = 0;
            } else {
                idle += 1;
            }
        }
        self.redraw();
    }

    /// Модальное выполнение представления (`execView`).
    ///
    /// Представление вставляется на рабочий стол и получает все события, пока
    /// не вызовет `ctx.end_modal(cmd)` (диалоги делают это на OK/Cancel/Yes/No).
    /// Возвращает код завершения и само представление (чтобы прочитать данные).
    pub fn exec_view(&mut self, mut v: Box<dyn View>) -> (u16, Box<dyn View>) {
        let id = v.vs().id;
        v.set_state(sf::MODAL, true);
        let di = self.desktop_index();
        {
            let desk = self.group.children[di].as_group_mut().unwrap();
            desk.insert(v);
            self.ctx.depth = 1;
            desk.select_id(id, &mut self.ctx);
            self.ctx.depth = 0;
        }
        self.modal_stack.push(id);
        self.sync_view(id);
        self.ctx.end_state = None;

        let cmd = loop {
            if !self.running {
                break cm::CANCEL;
            }
            self.redraw();
            let ev = self.get_event();
            self.handle(ev, None);
            if let Some(c) = self.ctx.end_state.take() {
                // Перед завершением спрашиваем `valid` (как `TGroup::execute`).
                let ctx = &mut self.ctx;
                let ok = self.group.view_mut(id).map(|v| v.valid(c, ctx)).unwrap_or(true);
                if ok {
                    break c;
                }
            }
        };
        self.modal_stack.pop();
        let di = self.desktop_index();
        let desk = self.group.children[di].as_group_mut().unwrap();
        let mut v = desk.remove(id).expect("модальное представление");
        v.set_state(sf::MODAL, false);
        self.ctx.release_mouse();
        (cmd, v)
    }

    /// Модальное выполнение с возвратом исходного типа:
    /// `let (cmd, dlg) = app.exec_dialog(dlg);`
    pub fn exec_dialog<T: View>(&mut self, v: T) -> (u16, T) {
        let (cmd, b) = self.exec_view(Box::new(v));
        let t = b.into_any().downcast::<T>().expect("тип представления");
        (cmd, *t)
    }

    // ------------------------------------------------------------------
    // Получение событий
    // ------------------------------------------------------------------

    /// Получить следующее событие (`getEvent`): сначала очередь, потом терминал.
    pub fn get_event(&mut self) -> Event {
        if let Some(e) = self.ctx.queue.pop_front() {
            return e;
        }
        // Пока кнопка мыши нажата — короткий тайм-аут для автоповтора.
        let timeout = if self.mouse_buttons != 0 { Duration::from_millis(60) } else { Duration::from_millis(100) };
        match self.backend.read(timeout) {
            Some(RawEvent::Key(k)) => Event::KeyDown(k),
            Some(RawEvent::Mouse { pos, buttons, mods, wheel }) => self.convert_mouse(pos, buttons, mods, wheel),
            Some(RawEvent::Resize(s)) => {
                self.resize(s);
                Event::Nothing
            }
            None => {
                if self.mouse_buttons != 0 {
                    Event::MouseAuto(self.last_mouse)
                } else {
                    Event::Nothing
                }
            }
        }
    }

    /// Превращает состояние мыши в событие нажатия/отпускания/движения.
    fn convert_mouse(&mut self, pos: Point, buttons: u8, mods: crate::event::Mods, wheel: i8) -> Event {
        let old = self.mouse_buttons;
        let mut me = MouseEvent { pos, buttons, double: false, mods, wheel };
        let ev = if wheel != 0 {
            Event::MouseWheel(me)
        } else if buttons & !old != 0 {
            let now = Instant::now();
            if let Some((t, p)) = self.last_click {
                me.double = p == pos && now.duration_since(t) < DOUBLE_CLICK;
            }
            self.last_click = if me.double { None } else { Some((now, pos)) };
            Event::MouseDown(me)
        } else if old & !buttons != 0 {
            me.buttons = old & !buttons;
            Event::MouseUp(me)
        } else if pos != self.last_mouse.pos {
            Event::MouseMove(me)
        } else {
            Event::Nothing
        };
        if wheel == 0 {
            self.mouse_buttons = buttons;
        }
        self.last_mouse = MouseEvent { buttons, wheel: 0, double: false, ..me };
        ev
    }

    /// Изменение размера терминала.
    fn resize(&mut self, s: Point) {
        self.screen = ScreenBuffer::new(s.x.max(1) as usize, s.y.max(1) as usize);
        self.ctx.screen_size = s;
        self.group.change_bounds(Rect::new(0, 0, s.x, s.y));
        self.menu = None;
    }

    // ------------------------------------------------------------------
    // Обработка событий
    // ------------------------------------------------------------------

    /// Обработать событие (`TProgram::handleEvent` + доставка).
    pub fn handle(&mut self, mut ev: Event, handler: Option<&mut dyn AppHandler>) {
        let mut handler = handler;
        if ev.is_nothing() {
            self.after_event(&ev);
            if let Some(h) = handler.as_deref_mut() {
                h.idle(self);
            }
            return;
        }

        // Открытое меню перехватывает всё.
        if self.menu.is_some() {
            self.handle_menu(&ev);
            self.after_event(&ev);
            return;
        }

        if let Some(&modal) = self.modal_stack.last() {
            self.handle_modal(modal, &mut ev);
        } else {
            // Alt+1..9 — выбор окна по номеру.
            if let Some(k) = ev.key() {
                if let Some(c) = k.alt_char() {
                    if let Some(d) = c.to_digit(10) {
                        if d >= 1 {
                            let di = self.desktop_index();
                            let mut b = Event::Broadcast(Message::with(cm::SELECT_WINDOW_NUM, Info::Int(d as i64)));
                            self.ctx.depth = 0;
                            self.group.dispatch(di, &mut b, &mut self.ctx);
                            ev.clear();
                        }
                    }
                }
            }
            self.ctx.depth = 0;
            self.ctx.owner_sizes.clear();
            self.group.handle_group_event(&mut ev, &mut self.ctx);

            if ev.command() == Some(cm::QUIT) {
                if self.group.valid(cm::QUIT, &mut self.ctx) {
                    self.running = false;
                }
                ev.clear();
            }
            if !ev.is_nothing() {
                if let Some(h) = handler.as_deref_mut() {
                    h.handle_event(self, &mut ev);
                }
            }
        }
        self.after_event(&ev);
    }

    /// Доставка события в модальном режиме: строка состояния + модальное представление.
    fn handle_modal(&mut self, modal: ViewId, ev: &mut Event) {
        self.ctx.depth = 0;
        self.ctx.owner_sizes.clear();
        // Строка состояния видит клавиши первой и принимает щелчки по себе.
        if let Some(sl) = self.status_line_id.and_then(|id| self.group.index_of(id)) {
            let sl_bounds = self.group.children[sl].vs().bounds();
            let to_status = if matches!(ev, Event::KeyDown(_)) {
                true
            } else if ev.is_mouse() {
                match self.ctx.mouse_grab {
                    Some(g) => self.group.children[sl].contains_id(g),
                    None => ev.mouse().map(|m| sl_bounds.contains(m.pos)).unwrap_or(false),
                }
            } else {
                false
            };
            if to_status {
                self.ctx.phase = Phase::PreProcess;
                self.group.dispatch(sl, ev, &mut self.ctx);
                self.ctx.phase = Phase::Focused;
                if ev.is_mouse() {
                    ev.clear();
                }
            }
        }
        if ev.is_nothing() {
            return;
        }
        let di = self.desktop_index();
        let screen = self.ctx.screen_size;
        let desk_origin = self.group.children[di].vs().origin;
        let desk_size = self.group.children[di].vs().size;
        let desk = self.group.children[di].as_group_mut().unwrap();
        let mi = match desk.index_of(modal) {
            Some(i) => i,
            None => return,
        };
        if let Some(m) = ev.mouse_mut() {
            m.pos -= desk_origin;
        }
        // Щелчки вне модального окна игнорируются (если мышь никто не захватил).
        if ev.is_mouse() && self.ctx.mouse_grab.is_none() {
            let inside = ev.mouse().map(|m| desk.children[mi].vs().bounds().contains(m.pos)).unwrap_or(false);
            if !inside {
                return;
            }
        }
        self.ctx.depth = 1;
        self.ctx.owner_sizes = vec![screen];
        let _ = desk_size;
        desk.dispatch(mi, ev, &mut self.ctx);
        self.ctx.depth = 0;
        self.ctx.owner_sizes.clear();
    }

    /// Работа после каждого события: запросы, смена команд, снятие захвата мыши.
    fn after_event(&mut self, ev: &Event) {
        // Сообщения, адресованные «владельцу программы», некому получать.
        self.ctx.outbox.clear();
        self.ctx.depth = 0;
        if matches!(ev, Event::MouseUp(_)) && self.mouse_buttons == 0 {
            self.ctx.release_mouse();
        }
        // Запросы представлений к программе.
        let reqs = std::mem::take(&mut self.ctx.requests);
        for r in reqs {
            match r {
                Request::MenuBar { item, mouse } => self.open_menu_bar(item, mouse),
                Request::Popup { menu, pos } => {
                    self.menu = Some(MenuSession::popup(menu, pos, self.ctx.screen_size));
                }
                Request::Exec { view, reply_to, tag, anchor, place } => self.exec_request(view, reply_to, tag, anchor, place),
                Request::Focus(id) => {
                    self.ctx.depth = 0;
                    self.group.focus_path(id, &mut self.ctx);
                    self.ctx.outbox.clear();
                }
            }
        }
        self.update_window_commands();
        self.update_view_commands();
        if self.ctx.commands_changed {
            self.ctx.commands_changed = false;
            let mut b = Event::Broadcast(Message::new(cm::COMMAND_SET_CHANGED));
            self.ctx.depth = 0;
            self.group.broadcast_children(&mut b, &mut self.ctx);
            self.ctx.outbox.clear();
        }
    }

    /// Выполняет запрос `Ctx::exec`: модальный запуск и доставка результата.
    fn exec_request(
        &mut self,
        mut view: Box<dyn View>,
        reply_to: Option<ViewId>,
        tag: u16,
        anchor: Option<ViewId>,
        place: Option<crate::view::PlaceFn>,
    ) {
        // Размещение относительно якоря.
        if let (Some(a), Some(f)) = (anchor, place) {
            if let Some((b, owner)) = self.group.geometry_of(a) {
                let nb = f(b, owner);
                // `change_bounds`, чтобы дети окна подстроились под новый размер.
                view.change_bounds(nb);
            }
        }
        // Перевод координат: от владельца `anchor` к рабочему столу.
        if let Some(a) = anchor {
            if let Some(o) = self.group.owner_origin_of(a) {
                let desk_origin = self.group.children[self.desktop_index()].vs().origin;
                let shift = o - desk_origin;
                let b = view.vs().bounds().moved(shift.x, shift.y);
                view.vs_mut().set_bounds(b);
            }
        }
        let (cmd, v) = self.exec_view(view);
        if let Some(id) = reply_to {
            self.ctx.depth = 0;
            self.ctx.owner_sizes.clear();
            let mut slot = Some(v);
            self.group.call_on(id, &mut self.ctx, &mut |target, ctx| {
                if let Some(v) = slot.take() {
                    target.exec_result(tag, cmd, v, ctx);
                }
            });
            self.ctx.outbox.clear();
        }
    }

    /// Даёт представлениям обновить свои команды: сначала тем, что вне фокуса,
    /// затем тем, что в фокусе (`updateCommands` в C++).
    fn update_view_commands(&mut self) {
        let ctx = &mut self.ctx;
        self.group.for_each_view(&mut |v| {
            if !v.vs().is_focused() {
                v.update_commands(ctx);
            }
        });
        self.group.for_each_view(&mut |v| {
            if v.vs().is_focused() {
                v.update_commands(ctx);
            }
        });
    }

    /// Разрешает команды окна в зависимости от активного окна
    /// (то, что в C++ делал `TWindow::setState(sfSelected)`).
    fn update_window_commands(&mut self) {
        let flags = {
            let desk = self.desktop();
            desk.group.current_view().and_then(|v| {
                if let Some(w) = v.as_any().downcast_ref::<Window>() {
                    Some(w.flags)
                } else {
                    v.as_any().downcast_ref::<crate::widgets::Dialog>().map(|d| d.win.flags)
                }
            })
        };
        let has = flags.is_some();
        let f = flags.unwrap_or(0);
        self.ctx.set_command(cm::NEXT, has);
        self.ctx.set_command(cm::PREV, has);
        self.ctx.set_command(cm::CLOSE, f & wf::CLOSE != 0);
        self.ctx.set_command(cm::ZOOM, f & wf::ZOOM != 0);
        self.ctx.set_command(cm::RESIZE, f & (wf::MOVE | wf::GROW) != 0);
    }

    /// Открывает сеанс меню из строки меню.
    fn open_menu_bar(&mut self, item: Option<usize>, _mouse: bool) {
        let id = match self.menu_bar_id {
            Some(id) => id,
            None => return,
        };
        let (menu, origin) = match self.group.find::<MenuBar>(id) {
            Some(b) => (b.menu.clone(), b.vs.origin),
            None => return,
        };
        let (session, out) = MenuSession::from_bar(menu, origin, item, self.ctx.screen_size, &self.ctx.commands);
        self.menu = Some(session);
        self.apply_menu_outcome(out);
    }

    fn handle_menu(&mut self, ev: &Event) {
        let out = match &mut self.menu {
            Some(s) => s.handle(ev, &self.ctx.commands),
            None => return,
        };
        self.apply_menu_outcome(out);
    }

    fn apply_menu_outcome(&mut self, out: MenuOutcome) {
        match out {
            MenuOutcome::Continue => {}
            MenuOutcome::Closed => self.menu = None,
            MenuOutcome::Command(c) => {
                self.menu = None;
                self.ctx.put_event(crate::widgets::menu::command_event(c));
            }
        }
        if self.menu.is_none() {
            self.ctx.release_mouse();
        }
    }

    // ------------------------------------------------------------------
    // Отрисовка
    // ------------------------------------------------------------------

    /// Перерисовать экран и вывести изменения в терминал.
    pub fn redraw(&mut self) {
        // Строке состояния — текущий контекст справки, строке меню — подсветку.
        let help = match &self.menu {
            Some(s) if s.help_ctx() != 0 => s.help_ctx(),
            _ => self.group.help_ctx(),
        };
        if let Some(sl) = self.status_line_mut() {
            sl.help_ctx = help;
        }
        let bar_sel = self.menu.as_ref().and_then(|s| s.bar_sel);
        if let Some(id) = self.menu_bar_id {
            if let Some(b) = self.group.find_mut::<MenuBar>(id) {
                b.active = bar_sel;
            }
        }

        {
            let mut dc = DrawCtx::new(&mut self.screen, self.palette, &self.ctx.commands);
            self.group.draw_children(&mut dc);
            if let Some(s) = &self.menu {
                s.draw(&mut dc);
            }
        }
        let size = self.ctx.screen_size;
        let cursor = if self.menu.is_some() {
            None
        } else {
            self.group.cursor_pos(Point::default(), Rect::new(0, 0, size.x, size.y))
        };
        let (pos, ins) = match cursor {
            Some((p, i)) => (Some(p), i),
            None => (None, false),
        };
        self.backend.flush(&self.screen, pos, ins);
    }
}
