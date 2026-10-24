//! Строка ввода (`TInputLine`).
//!
//! Поддерживает: перемещение курсора, выделение (Shift+стрелки, мышь),
//! режимы вставки/замены (Ins), удаление, буфер обмена
//! (Ctrl+Ins/Shift+Ins/Shift+Del, а также Ctrl+C/Ctrl+V/Ctrl+X),
//! прокрутку длинного текста, ограничение длины, проверку (`validator`).

use crate::color::palettes;
use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Info, Key, Mods};
use crate::geometry::Rect;
use crate::text::char_width;
use crate::view::{default_handle_event, of, sf, Ctx, DrawCtx, Value, View, ViewState};

use crate::widgets::validate::Validator;

/// Стрелки прокрутки (`TInputLine::leftArrow` / `rightArrow`).
const LEFT_ARROW: char = '◄';
const RIGHT_ARROW: char = '►';

pub struct InputLine {
    pub vs: ViewState,
    /// Текст (как массив символов, чтобы позиции были «по символам»).
    data: Vec<char>,
    /// Максимальная длина в символах.
    pub max_len: usize,
    /// Позиция курсора (номер символа).
    cur_pos: usize,
    /// Первый видимый символ.
    first_pos: usize,
    sel_start: usize,
    sel_end: usize,
    /// Опорная точка выделения (откуда начали выделять).
    anchor: Option<usize>,
    /// Проверка ввода (`TValidator`): некорректный набор откатывается,
    /// а некорректное значение не даёт закрыть диалог.
    pub validator: Option<Box<dyn Validator>>,
    /// Номер списка истории ввода (связанный значок [`crate::History`]).
    pub history_id: Option<u16>,
}

impl InputLine {
    /// Создаёт строку ввода (аналог `TInputLine(bounds, maxLen)`).
    pub fn new(bounds: Rect, max_len: usize) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.state |= sf::CURSOR_VIS;
        vs.options |= of::SELECTABLE | of::FIRST_CLICK;
        InputLine {
            vs,
            data: Vec::new(),
            max_len,
            cur_pos: 0,
            first_pos: 0,
            sel_start: 0,
            sel_end: 0,
            anchor: None,
            validator: None,
            history_id: None,
        }
    }

    /// Задать валидатор (аналог `setValidator`).
    pub fn with_validator(mut self, v: impl Validator + 'static) -> Self {
        self.validator = Some(Box::new(v));
        self
    }

    /// Проверка после правки (`checkValid`): некорректная правка откатывается,
    /// автозаполнение маски дописывает литералы.
    fn check_valid(&mut self, no_auto_fill: bool, old: &(Vec<char>, usize, usize, usize, usize)) {
        let v = match &self.validator {
            Some(v) => v,
            None => return,
        };
        let old_len = old.0.len();
        let mut s: String = self.data.iter().collect();
        if !v.is_valid_input(&mut s, no_auto_fill) {
            // Откат к состоянию до правки (`restoreState`).
            self.data = old.0.clone();
            self.cur_pos = old.1;
            self.first_pos = old.2;
            self.sel_start = old.3;
            self.sel_end = old.4;
        } else {
            let new: Vec<char> = s.chars().take(self.max_len).collect();
            if new != self.data {
                self.data = new;
                if self.cur_pos >= old_len && self.data.len() > old_len {
                    self.cur_pos = self.data.len();
                }
            }
        }
        self.cur_pos = self.cur_pos.min(self.data.len());
        self.adjust_first();
    }

    /// Запомнить текст в истории ввода.
    fn record_history(&self) {
        if let Some(h) = self.history_id {
            crate::widgets::history::history_add(h, &self.text());
        }
    }

    /// Текущий текст.
    pub fn text(&self) -> String {
        self.data.iter().collect()
    }

    /// Установить текст (курсор в конец, всё выделено — как `setData`).
    pub fn set_text(&mut self, s: &str) {
        self.data = s.chars().take(self.max_len).collect();
        self.select_all(true);
    }

    /// Позиция курсора (номер символа, `curPos`).
    pub fn cursor_pos(&self) -> usize {
        self.cur_pos
    }

    /// Поставить курсор (выделение снимается).
    pub fn set_cursor_pos(&mut self, pos: usize) {
        self.cur_pos = pos.min(self.data.len());
        self.sel_start = 0;
        self.sel_end = 0;
        self.anchor = None;
        self.adjust_first();
    }

    /// Выделить всё / снять выделение (`selectAll`).
    pub fn select_all(&mut self, enable: bool) {
        self.sel_start = 0;
        self.cur_pos = self.data.len();
        self.sel_end = if enable { self.data.len() } else { 0 };
        self.first_pos = 0;
        self.anchor = None;
        self.adjust_first();
    }

    fn has_selection(&self) -> bool {
        self.sel_start < self.sel_end
    }

    fn delete_selection(&mut self) {
        if self.has_selection() {
            self.data.drain(self.sel_start..self.sel_end);
            self.cur_pos = self.sel_start;
        }
        self.sel_start = 0;
        self.sel_end = 0;
    }

    fn selected_text(&self) -> String {
        self.data[self.sel_start..self.sel_end].iter().collect()
    }

    /// Ширина видимой части (без стрелок по краям).
    fn view_width(&self) -> usize {
        (self.vs.size.x - 2).max(1) as usize
    }

    /// Экранная ширина символов `from..to`.
    fn width_of(&self, from: usize, to: usize) -> usize {
        self.data[from..to].iter().map(|&c| char_width(c)).sum()
    }

    /// Прокручивает текст так, чтобы курсор был виден.
    fn adjust_first(&mut self) {
        if self.cur_pos < self.first_pos {
            self.first_pos = self.cur_pos;
        }
        let w = self.view_width();
        while self.first_pos < self.cur_pos && self.width_of(self.first_pos, self.cur_pos) >= w {
            self.first_pos += 1;
        }
    }

    /// Обновляет выделение после перемещения курсора (с Shift — расширяет).
    fn move_cursor(&mut self, new_pos: usize, extend: bool) {
        let new_pos = new_pos.min(self.data.len());
        if extend {
            let a = *self.anchor.get_or_insert(if self.has_selection() {
                if self.cur_pos == self.sel_start {
                    self.sel_end
                } else {
                    self.sel_start
                }
            } else {
                self.cur_pos
            });
            self.sel_start = a.min(new_pos);
            self.sel_end = a.max(new_pos);
        } else {
            self.anchor = None;
            self.sel_start = 0;
            self.sel_end = 0;
        }
        self.cur_pos = new_pos;
        self.adjust_first();
    }

    /// Номер символа под экранной колонкой `x` (локальные координаты).
    fn pos_at(&self, x: i32) -> usize {
        let mut col = 1;
        let mut i = self.first_pos;
        while i < self.data.len() {
            let w = char_width(self.data[i]) as i32;
            if x < col + w.max(1) {
                break;
            }
            col += w;
            i += 1;
        }
        i
    }

    fn insert_str(&mut self, s: &str) {
        self.delete_selection();
        // Как в оригинале: флаг CURSOR_INS (блочный курсор) означает режим замены.
        let ins = self.vs.state & sf::CURSOR_INS == 0;
        for c in s.chars().filter(|c| (*c as u32) >= 0x20) {
            if !ins && self.cur_pos < self.data.len() {
                // Режим замены.
                self.data[self.cur_pos] = c;
            } else {
                if self.data.len() >= self.max_len {
                    break;
                }
                self.data.insert(self.cur_pos, c);
            }
            self.cur_pos += 1;
        }
        self.adjust_first();
    }
}

impl View for InputLine {
    crate::impl_view_basics!();

    /// Порт `TInputLine::draw`.
    fn draw(&self, dc: &mut DrawCtx) {
        let color = if self.vs.is_focused() { dc.color(2) } else { dc.color(1) };
        let w = self.vs.size.x.max(0) as usize;
        let mut b = DrawBuffer::new(w);
        b.move_char(0, ' ', color, w);
        let visible: String = self.data[self.first_pos.min(self.data.len())..].iter().collect();
        b.move_str_limited(1, &visible, color, w.saturating_sub(2));
        // Стрелка вправо — есть невидимый текст справа.
        if self.width_of(self.first_pos.min(self.data.len()), self.data.len()) > self.view_width() {
            b.move_char(w - 1, RIGHT_ARROW, dc.color(4), 1);
        }
        if self.vs.is_selected() {
            if self.first_pos > 0 {
                b.move_char(0, LEFT_ARROW, dc.color(4), 1);
            }
            // Подсветка выделения.
            if self.has_selection() {
                let l = self.sel_start.max(self.first_pos);
                let r = self.sel_end;
                if l < r {
                    let x0 = 1 + self.width_of(self.first_pos, l);
                    let x1 = (1 + self.width_of(self.first_pos, r)).min(w.saturating_sub(1));
                    if x1 > x0 {
                        b.move_char(x0, '\0', dc.color(3), x1 - x0);
                    }
                }
            }
        }
        dc.write_line(0, 0, self.vs.size.x, 1, &b);
    }

    fn palette(&self) -> &'static [u8] {
        palettes::INPUT_LINE
    }

    fn set_state(&mut self, flag: u16, enable: bool) {
        if enable {
            self.vs.state |= flag;
        } else {
            self.vs.state &= !flag;
        }
        // При получении фокуса весь текст выделяется (как в оригинале).
        if flag & sf::SELECTED != 0 {
            self.select_all(enable);
        }
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        default_handle_event(&self.vs, ev, ctx);
        let id = self.vs.id;
        match ev {
            Event::MouseDown(m) => {
                let x = self.vs.make_local(m.pos).x;
                if m.double {
                    self.select_all(true);
                } else {
                    let p = self.pos_at(x);
                    self.anchor = Some(p);
                    self.move_cursor(p, false);
                    self.anchor = Some(p);
                    ctx.grab_mouse(id);
                }
                ev.clear();
            }
            Event::MouseMove(m) | Event::MouseAuto(m) if ctx.mouse_grabbed_by(id) => {
                let x = self.vs.make_local(m.pos).x;
                // Автопрокрутка при выходе мыши за края.
                if x < 1 && self.first_pos > 0 {
                    self.first_pos -= 1;
                }
                let p = if x >= self.vs.size.x - 1 { (self.cur_pos + 1).min(self.data.len()) } else { self.pos_at(x) };
                self.move_cursor(p, true);
                ev.clear();
            }
            Event::MouseUp(_) if ctx.mouse_grabbed_by(id) => {
                ctx.release_mouse();
                ev.clear();
            }
            Event::Broadcast(m) => match (&m.command, &m.info) {
                // Кнопка нажата или фокус ушёл — запомнить в истории (как `THistory`).
                (&cm::RECORD_HISTORY, Info::None) => self.record_history(),
                (&cm::RECORD_HISTORY, Info::Id(x)) if *x == id => self.record_history(),
                (&cm::RELEASED_FOCUS, Info::Id(x)) if *x == id => self.record_history(),
                // Выбор из окна истории.
                (&cm::HISTORY_SET, Info::Text { id: target, text }) if *target == id => {
                    let t = text.clone();
                    self.set_text(&t);
                    ev.clear();
                }
                _ => {}
            },
            Event::KeyDown(k) if self.vs.is_focused() => {
                let k = *k;
                let saved = (self.data.clone(), self.cur_pos, self.first_pos, self.sel_start, self.sel_end);
                let deleting = matches!(k.key, Key::Backspace | Key::Del)
                    || (k.mods.ctrl && matches!(k.key, Key::Char('x') | Key::Char('X') | Key::Char('y') | Key::Char('Y')));
                let shift = k.mods.shift;
                let plain = k.mods == Mods::NONE || k.mods == Mods::SHIFT;
                let handled = match k.key {
                    Key::Left if !k.mods.ctrl && !k.mods.alt => {
                        self.move_cursor(self.cur_pos.saturating_sub(1), shift);
                        true
                    }
                    Key::Right if !k.mods.ctrl && !k.mods.alt => {
                        self.move_cursor(self.cur_pos + 1, shift);
                        true
                    }
                    Key::Left if k.mods.ctrl => {
                        // Ctrl+← — к началу предыдущего слова.
                        let mut p = self.cur_pos;
                        while p > 0 && self.data[p - 1] == ' ' {
                            p -= 1;
                        }
                        while p > 0 && self.data[p - 1] != ' ' {
                            p -= 1;
                        }
                        self.move_cursor(p, shift);
                        true
                    }
                    Key::Right if k.mods.ctrl => {
                        let mut p = self.cur_pos;
                        let n = self.data.len();
                        while p < n && self.data[p] != ' ' {
                            p += 1;
                        }
                        while p < n && self.data[p] == ' ' {
                            p += 1;
                        }
                        self.move_cursor(p, shift);
                        true
                    }
                    Key::Home => {
                        self.move_cursor(0, shift);
                        true
                    }
                    Key::End => {
                        self.move_cursor(self.data.len(), shift);
                        true
                    }
                    Key::Backspace if plain => {
                        if self.has_selection() {
                            self.delete_selection();
                        } else if self.cur_pos > 0 {
                            self.cur_pos -= 1;
                            self.data.remove(self.cur_pos);
                        }
                        self.anchor = None;
                        self.adjust_first();
                        true
                    }
                    Key::Del if k.mods == Mods::NONE => {
                        if self.has_selection() {
                            self.delete_selection();
                        } else if self.cur_pos < self.data.len() {
                            self.data.remove(self.cur_pos);
                        }
                        self.anchor = None;
                        self.adjust_first();
                        true
                    }
                    Key::Del if shift => {
                        // Shift+Del — вырезать.
                        if self.has_selection() {
                            ctx.clipboard.set(&self.selected_text());
                            self.delete_selection();
                            self.adjust_first();
                        }
                        true
                    }
                    Key::Ins if k.mods.ctrl => {
                        if self.has_selection() {
                            ctx.clipboard.set(&self.selected_text());
                        }
                        true
                    }
                    Key::Ins if shift => {
                        let s = ctx.clipboard.get();
                        self.insert_str(&s);
                        true
                    }
                    Key::Ins if k.mods == Mods::NONE => {
                        // Переключение вставка/замена; в режиме замены курсор — блок.
                        self.vs.state ^= sf::CURSOR_INS;
                        true
                    }
                    Key::Char(c) if k.mods.ctrl && !k.mods.alt => match c.to_ascii_lowercase() {
                        'c' => {
                            if self.has_selection() {
                                ctx.clipboard.set(&self.selected_text());
                            }
                            true
                        }
                        'x' => {
                            if self.has_selection() {
                                ctx.clipboard.set(&self.selected_text());
                                self.delete_selection();
                                self.adjust_first();
                            }
                            true
                        }
                        'v' => {
                            let s = ctx.clipboard.get();
                            self.insert_str(&s);
                            true
                        }
                        'y' => {
                            // Ctrl+Y — очистить строку.
                            self.data.clear();
                            self.move_cursor(0, false);
                            true
                        }
                        _ => false,
                    },
                    Key::Char(c) if !k.mods.ctrl && !k.mods.alt => {
                        let mut tmp = [0u8; 4];
                        self.insert_str(c.encode_utf8(&mut tmp));
                        self.anchor = None;
                        self.sel_start = 0;
                        self.sel_end = 0;
                        true
                    }
                    _ => false,
                };
                if handled {
                    if self.data != saved.0 {
                        self.check_valid(deleting, &saved);
                    }
                    ev.clear();
                }
            }
            _ => {}
        }
        // Курсор: позиция в строке. Режим вставки — обычный курсор,
        // режим замены — блочный (флаг CURSOR_INS переключается клавишей Ins).
        let x = 1 + self.width_of(self.first_pos.min(self.cur_pos), self.cur_pos) as i32;
        self.vs.set_cursor(x, 0);
    }

    /// `TInputLine::valid`: при ошибке показывается сообщение валидатора
    /// и фокус возвращается в строку ввода.
    fn valid(&mut self, command: u16, ctx: &mut Ctx) -> bool {
        let v = match &self.validator {
            Some(v) => v,
            None => return true,
        };
        if command == cm::VALID {
            return v.status_ok();
        }
        if command == cm::CANCEL || v.is_valid(&self.text()) {
            return true;
        }
        let msg = v.error_message();
        let dlg = crate::widgets::msgbox::build_message_box(
            Rect::new(0, 0, 40, 9),
            &msg,
            crate::widgets::msgbox::mf::ERROR | crate::widgets::msgbox::mf::OK_BUTTON,
        );
        ctx.exec(Box::new(dlg), None, 0);
        ctx.focus(self.vs.id);
        false
    }

    fn has_data(&self) -> bool {
        true
    }

    fn get_data(&self) -> Value {
        Value::Str(self.text())
    }

    fn set_data(&mut self, value: &Value) {
        if let Value::Str(s) = value {
            self.set_text(s);
        }
    }
}
