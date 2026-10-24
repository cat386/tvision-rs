//! Выпадающий список у строки ввода (`TCombo` из пакета Dialog Design
//! Л. Дэвида Болдуина).
//!
//! Похож на значок истории ([`crate::History`]), но список строк задаёт
//! программа. Флаги:
//! * [`cbx::ONLY_LIST`] — в строку можно ввести только элемент списка:
//!   набор букв выполняет пошаговый поиск, пробел (Shift+пробел) листает
//!   список вперёд (назад);
//! * [`cbx::DISPOSES_LIST`] — в оригинале список освобождался вместе с
//!   элементом; в Rust память освобождается сама, флаг хранится для
//!   совместимости;
//! * [`cbx::NO_TRANSFER`] — список не участвует в обмене данными диалога.
//!   Без этого флага данные элемента — список строк (`Value::List` из
//!   `Value::Str`).
//!
//! Строка ввода может быть обычной [`InputLine`] или [`InputLong`].

use crate::color::palettes;
use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Info, Key, Mods};
use crate::geometry::{Point, Rect};
use crate::program::Program;
use crate::view::{of, sf, Ctx, DrawCtx, Value, View, ViewId, ViewState};
use crate::widgets::inputline::InputLine;
use crate::widgets::inputlong::InputLong;
use crate::widgets::listbox::ListBox;
use crate::widgets::window::{wf, Window};

/// Флаги `TCombo`.
pub mod cbx {
    pub const ONLY_LIST: u16 = 1;
    pub const DISPOSES_LIST: u16 = 2;
    pub const NO_TRANSFER: u16 = 4;
}

pub struct Combo {
    pub vs: ViewState,
    pub flags: u16,
    /// Связанная строка ввода.
    pub link: ViewId,
    /// Отображаемый символ (по умолчанию `↓`).
    pub show_char: char,
    /// Клавиша вызова: `None` — стрелка вниз, иначе символ.
    pub activate: Option<char>,
    /// Список строк, упорядоченный без учёта регистра
    /// (`TNoCaseStringCollection`).
    pub list: Vec<String>,
    link_focused: bool,
}

/// Сравнение без учёта регистра (`TNoCaseStringCollection.Compare`).
fn nocase_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    a.to_uppercase().cmp(&b.to_uppercase())
}

/// Поиск в отсортированном списке: (найдено, позиция) как `Search`.
fn search(list: &[String], key: &str) -> (bool, usize) {
    match list.binary_search_by(|x| nocase_cmp(x, key)) {
        Ok(i) => (true, i),
        Err(i) => (false, i),
    }
}

/// Совпадают ли первые `count` символов без учёта регистра (`Equal`).
fn equal(s1: &str, s2: &str, count: usize) -> bool {
    let a: Vec<char> = s1.chars().collect();
    let b: Vec<char> = s2.chars().collect();
    if a.len() < count || b.len() < count {
        return false;
    }
    (0..count).all(|i| a[i].to_uppercase().eq(b[i].to_uppercase()))
}

/// Доступ к строке ввода, к которой привязан выпадающий список.
fn with_link(p: &mut Program, link: ViewId, f: impl FnOnce(&mut InputLine)) {
    if let Some(il) = p.find_mut::<InputLine>(link) {
        f(il);
    } else if let Some(il) = p.find_mut::<InputLong>(link) {
        f(&mut il.il);
    }
}

impl Combo {
    /// `TCombo(bounds, link, flags, strings)`: обычно 1×1 или 3×1 справа от
    /// строки ввода.
    pub fn new(bounds: Rect, link: ViewId, flags: u16, items: &[&str]) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.options |= of::PRE_PROCESS;
        let mut c = Combo { vs, flags, link, show_char: '↓', activate: None, list: Vec::new(), link_focused: false };
        c.new_list(items.iter().map(|s| s.to_string()).collect());
        c
    }

    /// Сменить отображаемый символ (`ActivateChar`). Для стрелок вниз
    /// (`↓`, `▼`, а также их кодов 25 и 31) вызов — клавишей ↓, иначе
    /// самим символом.
    pub fn activate_char(&mut self, ch: char) {
        self.show_char = ch;
        self.activate = match ch {
            '↓' | '▼' | '\u{19}' | '\u{1F}' => None,
            c => Some(c),
        };
        if ch == '\u{19}' {
            self.show_char = '↓';
        } else if ch == '\u{1F}' {
            self.show_char = '▼';
        }
    }

    /// Новый список (`NewList`); список сортируется без учёта регистра.
    pub fn new_list(&mut self, mut items: Vec<String>) {
        items.sort_by(|a, b| nocase_cmp(a, b));
        items.dedup_by(|a, b| nocase_cmp(a, b) == std::cmp::Ordering::Equal);
        self.list = items;
    }

    /// Открыть окно со списком под строкой ввода (`PopUp`).
    fn popup(&mut self, ctx: &mut Ctx) {
        let (list, link, me) = (self.list.clone(), self.link, self.vs.id);
        // Текст строки ввода нужен, чтобы выделить текущий элемент, — читаем
        // его уже «на уровне программы».
        ctx.with_program(move |p| {
            let mut text = String::new();
            with_link(p, link, |il| text = il.text());
            let (_, idx) = search(&list, &text);
            let idx = idx.min(list.len().saturating_sub(1));
            let ht = (list.len() as i32 + 2).clamp(3, 9);
            let w = ComboWindow::new(Rect::new(0, 0, 10, ht), list, idx);
            // Размещение как в `TCombo.MakeDialog`.
            let place = Box::new(move |r: Rect, owner: Point| {
                let mut r = r;
                r.a.x -= 1;
                r.b.x += 2;
                r.b.y += ht;
                if r.b.y >= owner.y {
                    r.b.y = owner.y - 1;
                }
                r.a.y = r.b.y - ht;
                if r.a.y <= 0 {
                    r.a.y = 1;
                }
                r
            });
            p.ctx.exec_placed(Box::new(w), Some(me), 0, link, place);
        });
    }

    /// Пробел в режиме `ONLY_LIST`: следующий/предыдущий элемент.
    fn cycle(&self, back: bool, ctx: &mut Ctx) {
        let (list, link) = (self.list.clone(), self.link);
        if list.is_empty() {
            return;
        }
        ctx.with_program(move |p| {
            with_link(p, link, |il| {
                let (found, i) = search(&list, &il.text());
                let n = list.len() as isize;
                let mut v = if found { i as isize } else { -1 };
                v += if back { -1 } else { 1 };
                if v < 0 {
                    v = n - 1;
                } else if v >= n {
                    v = 0;
                }
                il.set_text(&list[v as usize]);
                il.select_all(false);
            });
        });
    }

    /// Пошаговый поиск в режиме `ONLY_LIST` (`IncrementalSearch`):
    /// `key` — набранный символ, либо `None` для Backspace/←, либо Home.
    fn incremental(&self, key: Option<char>, home: bool, ctx: &mut Ctx) {
        let (list, link) = (self.list.clone(), self.link);
        if list.is_empty() {
            return;
        }
        ctx.with_program(move |p| {
            with_link(p, link, |il| {
                let text = il.text();
                let mut search_pos = il.cursor_pos();
                let old_pos = search_pos;
                let mut cur: Vec<char> = text.chars().collect();
                match key {
                    None => {
                        if search_pos == 0 {
                            return;
                        }
                        search_pos = if home { 0 } else { search_pos - 1 };
                        cur.truncate(search_pos);
                    }
                    Some(c) => {
                        cur.truncate(search_pos);
                        cur.push(c);
                        search_pos += 1;
                    }
                }
                let cur: String = cur.into_iter().collect();
                let (_, value) = search(&list, &cur);
                if value < list.len() && equal(&list[value], &cur, search_pos) {
                    if !nocase_cmp(&list[value], &text).is_eq() {
                        il.set_text(&list[value]);
                    }
                    il.set_cursor_pos(search_pos);
                } else {
                    il.set_cursor_pos(old_pos);
                }
            });
        });
    }
}

impl View for Combo {
    crate::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let w = self.vs.size.x.max(1) as usize;
        let mut b = DrawBuffer::new(w);
        if self.vs.size.x == 1 {
            b.move_char(0, self.show_char, dc.color(1), 1);
        } else {
            b.move_cstr(0, &format!("▐~{}~▌", self.show_char), dc.get_color(0x0102));
        }
        dc.write_line(0, 0, self.vs.size.x, self.vs.size.y, &b);
    }

    fn palette(&self) -> &'static [u8] {
        palettes::HISTORY
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        match ev {
            Event::MouseDown(_) => {
                ctx.select(self.link);
                self.popup(ctx);
                ev.clear();
            }
            Event::KeyDown(k) if self.link_focused => {
                let k = *k;
                let activated = match self.activate {
                    None => k.key == Key::Down && k.mods == Mods::NONE,
                    Some(c) => k.text_char() == Some(c),
                };
                if activated {
                    self.popup(ctx);
                    ev.clear();
                    return;
                }
                if self.flags & cbx::ONLY_LIST == 0 || k.mods.ctrl || k.mods.alt {
                    return;
                }
                match k.key {
                    Key::Char(' ') => {
                        self.cycle(k.mods.shift, ctx);
                        ev.clear();
                    }
                    Key::Char(c) => {
                        self.incremental(Some(c), false, ctx);
                        ev.clear();
                    }
                    Key::Backspace | Key::Left => {
                        self.incremental(None, false, ctx);
                        ev.clear();
                    }
                    Key::Home => {
                        self.incremental(None, true, ctx);
                        ev.clear();
                    }
                    Key::Del | Key::End | Key::Ins | Key::Right => ev.clear(),
                    _ => {}
                }
            }
            Event::Broadcast(m) => match (&m.command, &m.info) {
                (&cm::RECEIVED_FOCUS, Info::Id(x)) => self.link_focused = *x == self.link,
                (&cm::RELEASED_FOCUS, Info::Id(x)) if *x == self.link => self.link_focused = false,
                _ => {}
            },
            _ => {}
        }
    }

    /// Окно со списком закрыто: выбранная строка уходит в строку ввода.
    fn exec_result(&mut self, _tag: u16, command: u16, view: Box<dyn View>, ctx: &mut Ctx) {
        if command != cm::OK {
            return;
        }
        if let Ok(w) = view.into_any().downcast::<ComboWindow>() {
            if let Some(text) = w.selection() {
                ctx.message(cm::HISTORY_SET, Info::Text { id: self.link, text });
            }
        }
    }

    fn has_data(&self) -> bool {
        self.flags & cbx::NO_TRANSFER == 0
    }

    fn get_data(&self) -> Value {
        Value::List(self.list.iter().map(|s| Value::Str(s.clone())).collect())
    }

    fn set_data(&mut self, value: &Value) {
        if let Value::List(v) = value {
            self.new_list(v.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect());
        }
    }
}

/// Окно выпадающего списка (`TListDialog` + `TStringListBox`).
pub struct ComboWindow {
    pub win: Window,
    lb: ViewId,
}

impl ComboWindow {
    fn new(bounds: Rect, items: Vec<String>, focused: usize) -> Self {
        let mut win = Window::new(bounds, "", 0);
        win.flags = wf::GROW;
        win.palette = palettes::HISTORY_WINDOW;
        win.group.vs.state &= !sf::SHADOW;
        let vsb = win.standard_scroll_bar(true);
        let mut lb = ListBox::new(win.interior(), 1, Some(vsb));
        lb.palette = palettes::HISTORY_VIEWER;
        lb.vs.grow_mode = crate::view::gf::GROW_HI_X | crate::view::gf::GROW_HI_Y;
        lb.set_items(items);
        lb.focus_item(focused);
        let id = win.add(lb);
        ComboWindow { win, lb: id }
    }

    /// Выбранная строка.
    pub fn selection(&self) -> Option<String> {
        self.win.group.find::<ListBox>(self.lb).and_then(|l| l.selected_text().map(|s| s.to_string()))
    }
}

impl View for ComboWindow {
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
        let double_in_list = match ev {
            Event::MouseDown(m) if m.double => {
                let local = self.win.group.vs.make_local(m.pos);
                self.win.group.child(self.lb).map(|v| v.vs().bounds().contains(local)).unwrap_or(false)
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
            Event::Command(m) if m.command == cm::CANCEL || m.command == cm::CLOSE => {
                ctx.end_modal(cm::CANCEL);
                ev.clear();
                return;
            }
            // Пошаговый поиск по первым буквам (`TStringListBox`).
            Event::KeyDown(k) if k.mods == Mods::NONE || k.mods == Mods::SHIFT => {
                if let Key::Char(c) = k.key {
                    if c != ' ' {
                        if let Some(lb) = self.win.group.find_mut::<ListBox>(self.lb) {
                            let cu: String = c.to_uppercase().collect();
                            if let Some(i) = lb.items.iter().position(|s| s.to_uppercase().starts_with(&cu)) {
                                lb.focus_item(i);
                            }
                        }
                        ev.clear();
                        return;
                    }
                }
            }
            _ => {}
        }
        self.win.handle_window_event(ev, ctx);
        if double_in_list {
            ctx.end_modal(cm::OK);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_nocase() {
        let c = Combo::new(Rect::new(0, 0, 1, 1), 0, 0, &["beta", "Alpha", "gamma", "alpha"]);
        assert_eq!(c.list, vec!["Alpha", "beta", "gamma"]);
        assert_eq!(search(&c.list, "BE"), (false, 1));
        assert!(equal("Beta", "be", 2));
    }
}
