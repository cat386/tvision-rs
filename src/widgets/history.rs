//! История ввода: хранилище (`histlist.cpp`), значок (`THistory`),
//! окно (`THistoryWindow`) со списком (`THistoryViewer`).
//!
//! Строка ввода с `history_id` запоминает свой текст, когда теряет фокус или
//! когда в диалоге нажимают кнопку. Значок `▐↓▌` справа от строки (или стрелка
//! вниз в строке) открывает список прошлых значений.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::color::palettes;
use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Info, Key, Mods};
use crate::geometry::{Point, Rect};
use crate::view::{sf, Ctx, DrawCtx, View, ViewId, ViewState};
use crate::widgets::listbox::ListBox;
use crate::widgets::window::{wf, Window};

/// Максимум строк в одном списке истории.
pub const HISTORY_LIMIT: usize = 32;

thread_local! {
    /// Списки истории: номер → строки (самая свежая — первая).
    static HISTORY: RefCell<HashMap<u16, Vec<String>>> = RefCell::new(HashMap::new());
}

/// Добавить строку в историю (`historyAdd`). Повтор переносится в начало.
pub fn history_add(id: u16, s: &str) {
    if s.is_empty() {
        return;
    }
    HISTORY.with(|h| {
        let mut h = h.borrow_mut();
        let list = h.entry(id).or_default();
        list.retain(|x| x != s);
        list.insert(0, s.to_string());
        list.truncate(HISTORY_LIMIT);
    });
}

/// Число строк в истории (`historyCount`).
pub fn history_count(id: u16) -> usize {
    HISTORY.with(|h| h.borrow().get(&id).map(|l| l.len()).unwrap_or(0))
}

/// Строка истории номер `i` (0 — самая свежая) (`historyStr`).
pub fn history_str(id: u16, i: usize) -> Option<String> {
    HISTORY.with(|h| h.borrow().get(&id).and_then(|l| l.get(i).cloned()))
}

/// Все строки истории.
pub fn history_list(id: u16) -> Vec<String> {
    HISTORY.with(|h| h.borrow().get(&id).cloned().unwrap_or_default())
}

/// Очистить все списки истории (`clearHistory`).
pub fn clear_history() {
    HISTORY.with(|h| h.borrow_mut().clear());
}

/// Значок истории рядом со строкой ввода (`THistory`).
pub struct History {
    pub vs: ViewState,
    /// Связанная строка ввода.
    pub link: ViewId,
    pub history_id: u16,
    /// Строка ввода в фокусе (отслеживается по сообщениям о фокусе).
    link_focused: bool,
}

/// Значок `THistory::icon`.
const ICON: &str = "▐~↓~▌";

impl History {
    /// Обычно размер значка 3×1, справа от строки ввода.
    /// Не забудьте задать строке ввода `history_id` — тот же номер.
    pub fn new(bounds: Rect, link: ViewId, history_id: u16) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.options |= crate::view::of::POST_PROCESS;
        History { vs, link, history_id, link_focused: false }
    }

    /// Открывает окно истории под строкой ввода.
    fn open(&mut self, ctx: &mut Ctx) {
        ctx.select(self.link);
        // Запомнить текущий текст строки (`recordHistory(link->data)`).
        ctx.message(cm::RECORD_HISTORY, Info::Id(self.link));
        let w = HistoryWindow::new(Rect::new(0, 0, 20, 8), self.history_id);
        // Размещение как в `THistory::handleEvent`.
        let place = Box::new(|link: Rect, owner: Point| {
            let mut r = link;
            r.a.x -= 1;
            r.b.x += 1;
            r.b.y += 7;
            r.a.y -= 1;
            let mut r = r.intersect(&Rect::new(0, 0, owner.x, owner.y));
            r.b.y -= 1;
            r
        });
        ctx.exec_placed(Box::new(w), Some(self.vs.id), 0, self.link, place);
    }
}

impl View for History {
    crate::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let mut b = DrawBuffer::new(self.vs.size.x.max(1) as usize);
        b.move_cstr(0, ICON, dc.get_color(0x0102));
        dc.write_line(0, 0, self.vs.size.x, self.vs.size.y, &b);
    }

    fn palette(&self) -> &'static [u8] {
        palettes::HISTORY
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        match ev {
            Event::MouseDown(_) => {
                self.open(ctx);
                ev.clear();
            }
            Event::KeyDown(k) if k.key == Key::Down && k.mods == Mods::NONE && self.link_focused => {
                self.open(ctx);
                ev.clear();
            }
            Event::Broadcast(m) => match (&m.command, &m.info) {
                (&cm::RECEIVED_FOCUS, Info::Id(x)) => self.link_focused = *x == self.link,
                (&cm::RELEASED_FOCUS, Info::Id(x)) if *x == self.link => self.link_focused = false,
                _ => {}
            },
            _ => {}
        }
    }

    /// Окно истории закрыто: если выбрано значение — передаём его строке ввода.
    fn exec_result(&mut self, _tag: u16, command: u16, view: Box<dyn View>, ctx: &mut Ctx) {
        if command != cm::OK {
            return;
        }
        if let Ok(w) = view.into_any().downcast::<HistoryWindow>() {
            if let Some(text) = w.selection() {
                ctx.message(cm::HISTORY_SET, Info::Text { id: self.link, text });
            }
        }
    }
}

/// Окно истории (`THistoryWindow` + `THistoryViewer`).
pub struct HistoryWindow {
    pub win: Window,
    viewer: ViewId,
    history_id: u16,
}

impl HistoryWindow {
    pub fn new(bounds: Rect, history_id: u16) -> Self {
        let mut win = Window::new(bounds, "", 0);
        win.flags = wf::CLOSE;
        win.palette = palettes::HISTORY_WINDOW;
        win.group.vs.state &= !sf::SHADOW;
        let hsb = win.standard_scroll_bar(false);
        let vsb = win.standard_scroll_bar(true);
        let mut lb = ListBox::new(win.interior(), 1, Some(vsb));
        lb.h_scroll = Some(hsb);
        lb.palette = palettes::HISTORY_VIEWER;
        lb.vs.grow_mode = crate::view::gf::GROW_HI_X | crate::view::gf::GROW_HI_Y;
        let viewer = win.add(lb);
        HistoryWindow { win, viewer, history_id }
    }

    /// Выбранная строка (`getSelection`).
    pub fn selection(&self) -> Option<String> {
        self.win.group.find::<ListBox>(self.viewer).and_then(|l| l.selected_text().map(|s| s.to_string()))
    }

    /// Загружает строки истории в список (при открытии окна).
    fn load(&mut self) {
        let items = history_list(self.history_id);
        if let Some(lb) = self.win.group.find_mut::<ListBox>(self.viewer) {
            if lb.items != items {
                let n = items.len();
                lb.set_items(items);
                // Как в оригинале: если строк больше одной, фокус на второй
                // (первая — то, что уже написано в строке ввода).
                if n > 1 {
                    lb.focus_item(1);
                }
            }
        }
    }
}

impl View for HistoryWindow {
    crate::impl_group_basics!(win.group);

    fn as_window(&self) -> Option<&Window> {
        Some(&self.win)
    }

    fn draw(&self, dc: &mut DrawCtx) {
        self.win.draw_window(dc);
    }

    fn palette(&self) -> &'static [u8] {
        self.win.palette
    }

    fn set_state(&mut self, flag: u16, enable: bool) {
        self.win.set_window_state(flag, enable);
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        // Программа шлёт COMMAND_SET_CHANGED при запуске — подгружаем строки.
        if ev.broadcast().map(|m| m.command) == Some(cm::COMMAND_SET_CHANGED) {
            self.load();
        }
        // Двойной щелчок по списку или Enter — выбор; Esc — отмена (`THistoryViewer`).
        let double_in_list = match ev {
            Event::MouseDown(m) if m.double => {
                let local = self.win.group.vs.make_local(m.pos);
                self.win.group.child(self.viewer).map(|v| v.vs().bounds().contains(local)).unwrap_or(false)
            }
            _ => false,
        };
        match ev {
            Event::KeyDown(k) if k.key == Key::Enter && k.mods == Mods::NONE => {
                ctx.end_modal(cm::OK);
                ev.clear();
                return;
            }
            Event::KeyDown(k) if k.key == Key::Esc && k.mods == Mods::NONE => {
                ctx.end_modal(cm::CANCEL);
                ev.clear();
                return;
            }
            Event::Command(m) if m.command == cm::CANCEL => {
                ctx.end_modal(cm::CANCEL);
                ev.clear();
                return;
            }
            _ => {}
        }
        self.win.handle_window_event(ev, ctx);
        if double_in_list {
            ctx.end_modal(cm::OK);
        }
    }
}
