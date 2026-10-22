//! Группа представлений (`TGroup`).
//!
//! Группа владеет дочерними представлениями, следит за текущим (выбранным)
//! из них, раздаёт события и рисует детей.
//!
//! ## Порядок детей
//! В C++ дети хранятся в кольцевом списке, где `first()` — самое верхнее
//! представление. Здесь используется `Vec`: индекс 0 — самое **нижнее**,
//! последний элемент — самое **верхнее** (рисуется последним).
//! Порядок перехода по Tab совпадает с порядком вставки.
//!
//! ## Доставка событий (как в `TGroup::handleEvent`)
//! * **Фокусные** (клавиши, команды): сначала дети с `of::PRE_PROCESS`,
//!   затем текущий ребёнок, затем дети с `of::POST_PROCESS`.
//! * **Позиционные** (мышь): самый верхний видимый ребёнок под курсором
//!   (или тот, кто захватил мышь через `Ctx::grab_mouse`).
//! * **Широковещательные**: все дети по порядку, пока кто-то не обработает.

use crate::event::{cm, Event, Info};
use crate::geometry::{Point, Rect};
use crate::view::{calc_bounds, default_handle_event, of, sf, Ctx, DrawCtx, Msg, Phase, Value, View, ViewId, ViewState};

/// Группа представлений.
pub struct Group {
    pub vs: ViewState,
    /// Дети: индекс 0 — нижний, последний — верхний.
    pub children: Vec<Box<dyn View>>,
    /// Индекс текущего (выбранного) ребёнка.
    pub current: Option<usize>,
}

impl Group {
    pub fn new(bounds: Rect) -> Self {
        Group { vs: ViewState::new(bounds), children: Vec::new(), current: None }
    }

    // ------------------------------------------------------------------
    // Вставка, удаление, поиск
    // ------------------------------------------------------------------

    /// Вставляет представление поверх остальных (`insert`). Возвращает его id.
    ///
    /// Представления с `of::CENTERED` центрируются. Новое представление
    /// становится текущим, если текущего ещё нет или у него есть
    /// `of::TOP_SELECT` (так новые окна получают фокус, а в диалоге фокус
    /// остаётся на первом элементе).
    pub fn insert(&mut self, mut v: Box<dyn View>) -> ViewId {
        {
            let size = self.vs.size;
            let vs = v.vs_mut();
            if vs.options & of::CENTER_X != 0 {
                vs.origin.x = (size.x - vs.size.x) / 2;
            }
            if vs.options & of::CENTER_Y != 0 {
                vs.origin.y = (size.y - vs.size.y) / 2;
            }
        }
        // Активность наследуется от группы (элементы в активном окне).
        if self.vs.state & sf::ACTIVE != 0 {
            v.set_state(sf::ACTIVE, true);
        }
        let id = v.vs().id;
        let take_focus = {
            let vs = v.vs();
            vs.is_selectable() && vs.is_visible() && !vs.is_disabled() && (self.current.is_none() || vs.options & of::TOP_SELECT != 0)
        };
        self.children.push(v);
        if take_focus {
            self.set_current_raw(Some(self.children.len() - 1));
        }
        id
    }

    /// Вставляет представление и возвращает его id (удобная обёртка).
    pub fn add<T: View>(&mut self, v: T) -> ViewId {
        self.insert(Box::new(v))
    }

    /// Индекс ребёнка с данным id.
    pub fn index_of(&self, id: ViewId) -> Option<usize> {
        self.children.iter().position(|c| c.vs().id == id)
    }

    /// Удаляет ребёнка (`remove`/`destroy`) и возвращает его.
    pub fn remove(&mut self, id: ViewId) -> Option<Box<dyn View>> {
        let idx = self.index_of(id)?;
        if self.current == Some(idx) {
            let c = &mut self.children[idx];
            if self.vs.state & sf::FOCUSED != 0 {
                c.set_state(sf::FOCUSED, false);
            }
            c.set_state(sf::SELECTED, false);
            self.current = None;
        } else if let Some(cur) = self.current {
            if cur > idx {
                self.current = Some(cur - 1);
            }
        }
        let v = self.children.remove(idx);
        if self.current.is_none() {
            self.reset_current();
        }
        Some(v)
    }

    /// Делает текущим самого верхнего видимого выбираемого ребёнка (`resetCurrent`).
    pub fn reset_current(&mut self) {
        let i = self.children.iter().rposition(|c| {
            let vs = c.vs();
            vs.is_visible() && vs.is_selectable() && !vs.is_disabled()
        });
        self.set_current_raw(i);
    }

    /// Ребёнок по id (только прямые дети).
    pub fn child(&self, id: ViewId) -> Option<&dyn View> {
        self.children.iter().find(|c| c.vs().id == id).map(|c| c.as_ref())
    }

    /// Представление по id во всём поддереве.
    pub fn view(&self, id: ViewId) -> Option<&dyn View> {
        for c in &self.children {
            if c.vs().id == id {
                return Some(c.as_ref());
            }
            if let Some(g) = c.as_group() {
                if let Some(v) = g.view(id) {
                    return Some(v);
                }
            }
        }
        None
    }

    /// Изменяемое представление по id во всём поддереве.
    pub fn view_mut(&mut self, id: ViewId) -> Option<&mut dyn View> {
        for c in self.children.iter_mut() {
            if c.vs().id == id {
                return Some(c.as_mut());
            }
            if let Some(g) = c.as_group_mut() {
                if let Some(v) = g.view_mut(id) {
                    return Some(v);
                }
            }
        }
        None
    }

    /// Представление по id с приведением к конкретному типу.
    pub fn find<T: View>(&self, id: ViewId) -> Option<&T> {
        self.view(id).and_then(|v| v.as_any().downcast_ref::<T>())
    }

    /// Изменяемое представление по id с приведением к конкретному типу.
    pub fn find_mut<T: View>(&mut self, id: ViewId) -> Option<&mut T> {
        self.view_mut(id).and_then(|v| v.as_any_mut().downcast_mut::<T>())
    }

    /// Текущий ребёнок.
    pub fn current_view(&self) -> Option<&dyn View> {
        self.current.map(|i| self.children[i].as_ref())
    }

    pub fn current_view_mut(&mut self) -> Option<&mut dyn View> {
        match self.current {
            Some(i) => Some(self.children[i].as_mut()),
            None => None,
        }
    }

    // ------------------------------------------------------------------
    // Выбор (фокус)
    // ------------------------------------------------------------------

    /// Меняет текущего ребёнка, обновляя флаги SELECTED/FOCUSED (`setCurrent`),
    /// без рассылки сообщений.
    pub(crate) fn set_current_raw(&mut self, i: Option<usize>) {
        if self.current == i {
            return;
        }
        let focused = self.vs.state & sf::FOCUSED != 0;
        if let Some(old) = self.current {
            if let Some(c) = self.children.get_mut(old) {
                if focused {
                    c.set_state(sf::FOCUSED, false);
                }
                c.set_state(sf::SELECTED, false);
            }
        }
        self.current = i;
        if let Some(new) = i {
            let c = &mut self.children[new];
            c.set_state(sf::SELECTED, true);
            if focused {
                c.set_state(sf::FOCUSED, true);
            }
        }
    }

    /// Меняет текущего ребёнка и рассылает детям `cm::RELEASED_FOCUS` /
    /// `cm::RECEIVED_FOCUS` (их, например, слушают метки `Label`).
    pub fn set_current(&mut self, i: Option<usize>, ctx: &mut Ctx) {
        if self.current == i {
            return;
        }
        let old_id = self.current.map(|o| self.children[o].vs().id);
        self.set_current_raw(i);
        if let Some(id) = old_id {
            let mut e = Event::Broadcast(crate::event::Message::with(cm::RELEASED_FOCUS, Info::Id(id)));
            self.broadcast_children(&mut e, ctx);
        }
        if let Some(n) = self.current {
            let id = self.children[n].vs().id;
            let mut e = Event::Broadcast(crate::event::Message::with(cm::RECEIVED_FOCUS, Info::Id(id)));
            self.broadcast_children(&mut e, ctx);
        }
    }

    /// Выбирает ребёнка с индексом `i` (`TView::select`): если у него
    /// `of::TOP_SELECT`, он сначала поднимается наверх.
    pub fn select_index(&mut self, i: usize, ctx: &mut Ctx) {
        let vs = self.children[i].vs();
        if !vs.is_selectable() || vs.is_disabled() {
            return;
        }
        let i = if vs.options & of::TOP_SELECT != 0 { self.make_first(i) } else { i };
        self.set_current(Some(i), ctx);
    }

    /// Выбирает ребёнка по id.
    pub fn select_id(&mut self, id: ViewId, ctx: &mut Ctx) {
        if let Some(i) = self.index_of(id) {
            self.select_index(i, ctx);
        }
    }

    /// Поднимает ребёнка наверх (`makeFirst`), возвращает его новый индекс.
    pub fn make_first(&mut self, i: usize) -> usize {
        let last = self.children.len() - 1;
        if i == last {
            return i;
        }
        let v = self.children.remove(i);
        self.children.push(v);
        // Поправляем индекс текущего.
        if let Some(cur) = self.current {
            self.current = Some(if cur == i {
                last
            } else if cur > i {
                cur - 1
            } else {
                cur
            });
        }
        last
    }

    /// Опускает ребёнка в самый низ, но выше `min_index` (например, над фоном).
    pub fn put_at(&mut self, i: usize, new_index: usize) {
        if i == new_index || i >= self.children.len() {
            return;
        }
        let cur_id = self.current.map(|c| self.children[c].vs().id);
        let v = self.children.remove(i);
        let ni = new_index.min(self.children.len());
        self.children.insert(ni, v);
        self.current = cur_id.and_then(|id| self.index_of(id));
    }

    /// Выбирает следующего (или предыдущего) выбираемого ребёнка (`selectNext`).
    pub fn select_next(&mut self, forward: bool, ctx: &mut Ctx) {
        let n = self.children.len();
        if n == 0 {
            return;
        }
        let start = self.current.unwrap_or(if forward { n - 1 } else { 0 });
        let mut i = start;
        for _ in 0..n {
            i = if forward { (i + 1) % n } else { (i + n - 1) % n };
            let vs = self.children[i].vs();
            if vs.is_visible() && vs.is_selectable() && !vs.is_disabled() {
                // Перед уходом проверяем текущий элемент, если он этого требует.
                if let Some(c) = self.current {
                    if self.children[c].vs().options & of::VALIDATE != 0 && !self.children[c].valid(cm::RELEASED_FOCUS) {
                        return;
                    }
                }
                self.select_index(i, ctx);
                return;
            }
        }
    }

    // ------------------------------------------------------------------
    // Состояние, границы, данные
    // ------------------------------------------------------------------

    /// `TGroup::setState`: ACTIVE и DRAGGING передаются всем детям,
    /// FOCUSED — только текущему.
    pub fn set_group_state(&mut self, flag: u16, enable: bool) {
        if enable {
            self.vs.state |= flag;
        } else {
            self.vs.state &= !flag;
        }
        let all = flag & (sf::ACTIVE | sf::DRAGGING);
        if all != 0 {
            for c in self.children.iter_mut() {
                c.set_state(all, enable);
            }
        }
        if flag & sf::FOCUSED != 0 {
            if let Some(i) = self.current {
                self.children[i].set_state(sf::FOCUSED, enable);
            }
        }
    }

    /// `TGroup::changeBounds`: при изменении размера пересчитывает детей
    /// по их `grow_mode`.
    pub fn change_bounds(&mut self, bounds: Rect) {
        let old = self.vs.size;
        self.vs.set_bounds(bounds);
        let new = self.vs.size;
        let delta = new - old;
        if delta == Point::default() {
            return;
        }
        for c in self.children.iter_mut() {
            let nb = calc_bounds(c.as_ref(), delta, new);
            c.change_bounds(nb);
        }
    }

    /// `TGroup::valid`: для `cm::RELEASED_FOCUS` проверяется только текущий
    /// ребёнок, иначе — все.
    pub fn valid(&mut self, command: u16) -> bool {
        if command == cm::RELEASED_FOCUS {
            if let Some(i) = self.current {
                if self.children[i].vs().options & of::VALIDATE != 0 {
                    return self.children[i].valid(command);
                }
            }
            return true;
        }
        self.children.iter_mut().all(|c| c.valid(command))
    }

    pub fn has_data(&self) -> bool {
        self.children.iter().any(|c| c.has_data())
    }

    /// Данные всех детей с данными, в порядке вставки (`TGroup::getData`).
    pub fn get_data(&self) -> Value {
        Value::List(self.children.iter().filter(|c| c.has_data()).map(|c| c.get_data()).collect())
    }

    /// Раздаёт данные детям в порядке вставки (`TGroup::setData`).
    pub fn set_data(&mut self, value: &Value) {
        if let Value::List(items) = value {
            let mut it = items.iter();
            for c in self.children.iter_mut().filter(|c| c.has_data()) {
                match it.next() {
                    Some(v) => c.set_data(v),
                    None => break,
                }
            }
        }
    }

    /// Контекст справки: у текущего ребёнка, если он задан, иначе свой.
    pub fn help_ctx(&self) -> u16 {
        if let Some(i) = self.current {
            let h = self.children[i].help_ctx();
            if h != 0 {
                return h;
            }
        }
        self.vs.help_ctx
    }

    // ------------------------------------------------------------------
    // Отрисовка
    // ------------------------------------------------------------------

    /// Рисует детей снизу вверх, затем их тени.
    pub fn draw_children(&self, dc: &mut DrawCtx) {
        for c in &self.children {
            let vs = c.vs();
            if !vs.is_visible() {
                continue;
            }
            let b = vs.bounds();
            {
                let mut sub = dc.sub(b, c.palette());
                c.draw(&mut sub);
            }
            if vs.state & sf::SHADOW != 0 {
                dc.shadow(b);
            }
        }
    }

    /// Ищет позицию видимого курсора в фокусе (абсолютные координаты).
    /// `origin` — абсолютное положение группы, `clip` — видимая область.
    pub fn cursor_pos(&self, origin: Point, clip: Rect) -> Option<(Point, bool)> {
        let cur = self.current?;
        let child = &self.children[cur];
        let vs = child.vs();
        if !vs.is_visible() {
            return None;
        }
        let abs = vs.bounds().moved(origin.x, origin.y);
        let cclip = clip.intersect(&abs);
        let found = if let Some(g) = child.as_group() {
            g.cursor_pos(abs.a, cclip)
        } else if vs.state & (sf::FOCUSED | sf::CURSOR_VIS) == sf::FOCUSED | sf::CURSOR_VIS {
            let p = abs.a + vs.cursor;
            if cclip.contains(p) {
                Some((p, vs.state & sf::CURSOR_INS != 0))
            } else {
                None
            }
        } else {
            None
        };
        let (p, ins) = found?;
        // Курсор не должен просвечивать сквозь представления, лежащие выше.
        for above in &self.children[cur + 1..] {
            let avs = above.vs();
            if avs.is_visible() && avs.bounds().moved(origin.x, origin.y).contains(p) {
                return None;
            }
        }
        Some((p, ins))
    }

    // ------------------------------------------------------------------
    // События
    // ------------------------------------------------------------------

    /// Доставляет событие ребёнку с индексом `i` (`doHandleEvent`), а затем
    /// обрабатывает сообщения, которые ребёнок оставил владельцу.
    pub fn dispatch(&mut self, i: usize, ev: &mut Event, ctx: &mut Ctx) {
        if ev.is_nothing() || i >= self.children.len() {
            return;
        }
        {
            let child = &mut self.children[i];
            let vs = child.vs();
            // Недоступные представления не получают клавиатуру и мышь.
            if vs.is_disabled() && (ev.is_focused() || ev.is_mouse()) {
                return;
            }
            ctx.depth += 1;
            ctx.owner_sizes.push(self.vs.size);
            child.handle_event(ev, ctx);
            ctx.owner_sizes.pop();
            ctx.depth -= 1;
        }
        self.process_outbox(ctx);
    }

    /// Обрабатывает сообщения, адресованные этой группе как владельцу.
    pub(crate) fn process_outbox(&mut self, ctx: &mut Ctx) {
        let child_depth = ctx.depth + 1;
        while let Some(p) = ctx.outbox.iter().position(|(d, _)| *d == child_depth) {
            let (_, msg) = ctx.outbox.remove(p);
            match msg {
                Msg::Broadcast(m) => {
                    let mut e = Event::Broadcast(m);
                    self.broadcast_children(&mut e, ctx);
                }
                Msg::Select(id) => self.select_id(id, ctx),
                Msg::Close(id) => {
                    let was_current = self.current.map(|c| self.children[c].vs().id) == Some(id);
                    self.remove(id);
                    // Новому текущему сообщаем о получении фокуса.
                    if was_current {
                        if let Some(n) = self.current {
                            let nid = self.children[n].vs().id;
                            let mut e = Event::Broadcast(crate::event::Message::with(cm::RECEIVED_FOCUS, Info::Id(nid)));
                            self.broadcast_children(&mut e, ctx);
                        }
                    }
                }
                Msg::SelectNext(f) => self.select_next(f, ctx),
                Msg::MakeFirst(id) => {
                    if let Some(i) = self.index_of(id) {
                        self.make_first(i);
                    }
                }
            }
        }
    }

    /// Рассылает событие всем детям, пока кто-нибудь его не обработает.
    pub fn broadcast_children(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        let mut i = 0;
        while i < self.children.len() && !ev.is_nothing() {
            self.dispatch(i, ev, ctx);
            i += 1;
        }
    }

    /// Обработка событий группой (`TGroup::handleEvent`).
    pub fn handle_group_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        default_handle_event(&self.vs, ev, ctx);
        if ev.is_nothing() {
            return;
        }
        if ev.is_focused() {
            let saved = ctx.phase;
            ctx.phase = Phase::PreProcess;
            let mut i = 0;
            while i < self.children.len() && !ev.is_nothing() {
                if self.children[i].vs().options & of::PRE_PROCESS != 0 {
                    self.dispatch(i, ev, ctx);
                }
                i += 1;
            }
            ctx.phase = Phase::Focused;
            if let Some(c) = self.current {
                self.dispatch(c, ev, ctx);
            }
            ctx.phase = Phase::PostProcess;
            let mut i = 0;
            while i < self.children.len() && !ev.is_nothing() {
                if self.children[i].vs().options & of::POST_PROCESS != 0 {
                    self.dispatch(i, ev, ctx);
                }
                i += 1;
            }
            ctx.phase = saved;
        } else if ev.is_mouse() {
            // Переводим координаты мыши в систему координат группы:
            // для детей группа — владелец.
            let origin = self.vs.origin;
            if let Some(m) = ev.mouse_mut() {
                m.pos -= origin;
            }
            let pos = ev.mouse().map(|m| m.pos).unwrap_or_default();
            let target = match ctx.mouse_grab {
                Some(g) => self.children.iter().position(|c| c.contains_id(g)),
                None => self.children.iter().rposition(|c| c.vs().is_visible() && c.vs().bounds().contains(pos)),
            };
            if let Some(i) = target {
                self.dispatch(i, ev, ctx);
            }
            if let Some(m) = ev.mouse_mut() {
                m.pos += origin;
            }
        } else if let Event::Broadcast(_) = ev {
            self.broadcast_children(ev, ctx);
        }
    }
}

impl View for Group {
    crate::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        self.draw_children(dc);
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        self.handle_group_event(ev, ctx);
    }

    fn set_state(&mut self, flag: u16, enable: bool) {
        self.set_group_state(flag, enable);
    }

    fn valid(&mut self, command: u16) -> bool {
        Group::valid(self, command)
    }

    fn change_bounds(&mut self, bounds: Rect) {
        Group::change_bounds(self, bounds);
    }

    fn has_data(&self) -> bool {
        Group::has_data(self)
    }
    fn get_data(&self) -> Value {
        Group::get_data(self)
    }
    fn set_data(&mut self, value: &Value) {
        Group::set_data(self, value)
    }
    fn help_ctx(&self) -> u16 {
        Group::help_ctx(self)
    }
    fn as_group(&self) -> Option<&Group> {
        Some(self)
    }
    fn as_group_mut(&mut self) -> Option<&mut Group> {
        Some(self)
    }
}
