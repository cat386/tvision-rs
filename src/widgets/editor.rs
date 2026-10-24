//! Текстовый редактор: `TEditor`, `TMemo`, `TFileEditor`, `TEditWindow`,
//! `TIndicator` (файлы `teditor1.cpp`, `teditor2.cpp`, `tmemo.cpp`,
//! `tfiledtr.cpp`, `teditwnd.cpp`, `tindictr.cpp`).
//!
//! Внутреннее устройство отличается от оригинала: вместо «буфера с разрывом»
//! (gap buffer) из байтов текст хранится как массив строк из символов
//! (`Vec<Vec<char>>`). Это проще для Unicode и даёт многоуровневую отмену
//! (в оригинале отменялась только последняя правка).
//!
//! Клавиши — как в оригинале (раскладка WordStar): стрелки, Ctrl+стрелки,
//! Home/End, PgUp/PgDn, Ctrl+Home/End, Shift+движение — выделение,
//! Ins — вставка/замена, Ctrl+Y — удалить строку, Ctrl+T — удалить слово,
//! Ctrl+U (и Ctrl+Z) — отмена, Ctrl+Ins/Shift+Ins/Shift+Del/Ctrl+Del —
//! буфер обмена, Ctrl+Q F/A — поиск/замена, Ctrl+L — искать дальше,
//! Ctrl+K B/K/C/Y/H — блочные команды, Ctrl+O — автоотступ.

use crate::locale::tr;
use std::cmp::{max, min};
use std::path::{Path, PathBuf};

use crate::color::palettes;
use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Info, Key, KeyEvent, Mods};
use crate::geometry::{Point, Rect};
use crate::text::char_width;
use crate::view::{default_handle_event, gf, of, sf, Ctx, DrawCtx, Value, View, ViewId, ViewState};
use crate::widgets::button::{bf, Button};
use crate::widgets::cluster::Cluster;
use crate::widgets::dialog::Dialog;
use crate::widgets::inputline::InputLine;
use crate::widgets::msgbox::{build_message_box, mf};
use crate::widgets::scrollbar::ScrollBar;
use crate::widgets::statictext::Label;
use crate::widgets::window::Window;

/// Флаги поиска и замены (`efXXX`).
pub mod ef {
    pub const CASE_SENSITIVE: u16 = 0x0001;
    pub const WHOLE_WORDS_ONLY: u16 = 0x0002;
    pub const PROMPT_ON_REPLACE: u16 = 0x0004;
    pub const REPLACE_ALL: u16 = 0x0008;
    pub const DO_REPLACE: u16 = 0x0010;
    pub const BACKUP_FILES: u16 = 0x0100;
}

/// Ширина табуляции.
pub const TAB_SIZE: usize = 8;

/// Метки модальных окон, которые открывает редактор (для `exec_result`).
mod tag {
    pub const FIND: u16 = 1;
    pub const REPLACE: u16 = 2;
    pub const PROMPT: u16 = 3;
    pub const SAVE_MODIFY: u16 = 4;
    pub const SAVE_AS: u16 = 5;
    pub const INFO: u16 = 6;
}

/// Позиция в тексте: номер строки и номер символа в строке.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

impl Pos {
    pub const fn new(line: usize, col: usize) -> Self {
        Pos { line, col }
    }
}

/// Правка для отмены/повтора.
#[derive(Clone, Debug)]
enum Edit {
    /// Вставлен `text` начиная с `at`.
    Insert { at: Pos, text: String },
    /// Удалён `text`, который начинался в `at`.
    Delete { at: Pos, text: String },
}

/// Запись в журнале отмены: несколько правок + положение курсора до них.
#[derive(Clone, Debug)]
struct UndoRec {
    edits: Vec<Edit>,
    cursor_before: Pos,
    /// Можно ли «приклеить» следующий набранный символ к этой записи.
    typing: bool,
}

/// Состояние префиксных клавиш WordStar (Ctrl+Q, Ctrl+K).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyState {
    Normal,
    Quick,
    Block,
}

/// Текстовый редактор (`TEditor`).
pub struct Editor {
    pub vs: ViewState,
    pub h_scroll: Option<ViewId>,
    pub v_scroll: Option<ViewId>,
    pub indicator: Option<ViewId>,
    lines: Vec<Vec<char>>,
    /// Курсор.
    cur: Pos,
    /// Второй конец выделения (если есть).
    anchor: Option<Pos>,
    /// Прокрутка: x — экранный столбец, y — строка.
    pub delta: Point,
    /// Текст изменён.
    pub modified: bool,
    /// Режим замены (в оригинале — `overwrite`).
    pub overwrite: bool,
    /// Автоотступ (`autoIndent`).
    pub auto_indent: bool,
    /// Можно ли отменять правки (`canUndo`).
    pub can_undo: bool,
    undo: Vec<UndoRec>,
    redo: Vec<UndoRec>,
    key_state: KeyState,
    /// Строка поиска и замены и флаги `ef::*` (в оригинале — статические поля).
    pub find_str: String,
    pub replace_str: String,
    pub editor_flags: u16,
    /// Выделение мышью.
    selecting: bool,
    /// Что последний раз было отправлено полосам и индикатору.
    last_sync: Option<(Point, Point, Pos, bool)>,
    /// Желаемый экранный столбец при движении вверх/вниз.
    goal_x: Option<usize>,
    /// Палитра (у `Memo` — своя).
    pub palette: &'static [u8],
}

impl Editor {
    /// Аналог `TEditor(bounds, hScrollBar, vScrollBar, indicator, bufSize)`.
    pub fn new(bounds: Rect, h_scroll: Option<ViewId>, v_scroll: Option<ViewId>, indicator: Option<ViewId>) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
        vs.options |= of::SELECTABLE;
        vs.state |= sf::CURSOR_VIS;
        Editor {
            vs,
            h_scroll,
            v_scroll,
            indicator,
            lines: vec![Vec::new()],
            cur: Pos::default(),
            anchor: None,
            delta: Point::default(),
            modified: false,
            overwrite: false,
            auto_indent: true,
            can_undo: true,
            undo: Vec::new(),
            redo: Vec::new(),
            key_state: KeyState::Normal,
            find_str: String::new(),
            replace_str: String::new(),
            editor_flags: ef::PROMPT_ON_REPLACE,
            selecting: false,
            last_sync: None,
            goal_x: None,
            palette: palettes::EDITOR,
        }
    }

    // ------------------------------------------------------------------
    // Доступ к тексту
    // ------------------------------------------------------------------

    /// Весь текст (строки через `\n`).
    pub fn text(&self) -> String {
        let mut s = String::new();
        for (i, l) in self.lines.iter().enumerate() {
            if i > 0 {
                s.push('\n');
            }
            s.extend(l.iter());
        }
        s
    }

    /// Заменить весь текст (сбрасывает отмену и флаг изменения).
    pub fn set_text(&mut self, s: &str) {
        self.lines = split_lines(s);
        self.cur = Pos::default();
        self.anchor = None;
        self.delta = Point::default();
        self.undo.clear();
        self.redo.clear();
        self.modified = false;
        self.last_sync = None;
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Текст строки.
    pub fn line(&self, i: usize) -> String {
        self.lines.get(i).map(|l| l.iter().collect()).unwrap_or_default()
    }

    pub fn cursor(&self) -> Pos {
        self.cur
    }

    /// Выделение (начало, конец), если оно непустое.
    pub fn selection(&self) -> Option<(Pos, Pos)> {
        let a = self.anchor?;
        if a == self.cur {
            return None;
        }
        Some((min(a, self.cur), max(a, self.cur)))
    }

    /// Выделенный текст.
    pub fn selected_text(&self) -> String {
        match self.selection() {
            Some((a, b)) => self.text_range(a, b),
            None => String::new(),
        }
    }

    /// Выделить диапазон и поставить курсор в его конец (`setSelect`).
    pub fn set_select(&mut self, from: Pos, to: Pos) {
        self.anchor = Some(from);
        self.cur = self.clamp(to);
    }

    fn clamp(&self, p: Pos) -> Pos {
        let line = p.line.min(self.lines.len() - 1);
        Pos::new(line, p.col.min(self.lines[line].len()))
    }

    /// Текст между двумя позициями.
    fn text_range(&self, a: Pos, b: Pos) -> String {
        let mut s = String::new();
        if a.line == b.line {
            s.extend(&self.lines[a.line][a.col..b.col]);
            return s;
        }
        s.extend(&self.lines[a.line][a.col..]);
        for l in a.line + 1..b.line {
            s.push('\n');
            s.extend(&self.lines[l]);
        }
        s.push('\n');
        s.extend(&self.lines[b.line][..b.col]);
        s
    }

    // ------------------------------------------------------------------
    // Низкоуровневая правка (без журнала отмены)
    // ------------------------------------------------------------------

    /// Вставляет текст в позицию, возвращает позицию после вставки.
    fn raw_insert(&mut self, at: Pos, text: &str) -> Pos {
        let at = self.clamp(at);
        let tail: Vec<char> = self.lines[at.line].split_off(at.col);
        let mut line = at.line;
        let mut parts = text.split('\n');
        if let Some(first) = parts.next() {
            self.lines[line].extend(first.chars().filter(|&c| c != '\r'));
        }
        for part in parts {
            line += 1;
            self.lines.insert(line, part.chars().filter(|&c| c != '\r').collect());
        }
        let col = self.lines[line].len();
        self.lines[line].extend(tail);
        Pos::new(line, col)
    }

    /// Удаляет текст между позициями, возвращает удалённое.
    fn raw_delete(&mut self, a: Pos, b: Pos) -> String {
        let (a, b) = (self.clamp(min(a, b)), self.clamp(max(a, b)));
        let removed = self.text_range(a, b);
        let tail: Vec<char> = self.lines[b.line][b.col..].to_vec();
        self.lines[a.line].truncate(a.col);
        self.lines[a.line].extend(tail);
        if b.line > a.line {
            self.lines.drain(a.line + 1..=b.line);
        }
        removed
    }

    /// Позиция после текста `text`, вставленного в `at`.
    fn end_of(at: Pos, text: &str) -> Pos {
        let n = text.matches('\n').count();
        if n == 0 {
            Pos::new(at.line, at.col + text.chars().count())
        } else {
            Pos::new(at.line + n, text.rsplit('\n').next().unwrap().chars().count())
        }
    }

    // ------------------------------------------------------------------
    // Правка с журналом отмены
    // ------------------------------------------------------------------

    fn push_undo(&mut self, edit: Edit, cursor_before: Pos, typing: bool) {
        self.modified = true;
        self.redo.clear();
        if !self.can_undo {
            return;
        }
        // Набор подряд идущих символов — одна запись отмены.
        if typing {
            if let Some(last) = self.undo.last_mut() {
                if last.typing {
                    if let (Some(Edit::Insert { at, text }), Edit::Insert { at: nat, text: ntext }) = (last.edits.last_mut(), &edit) {
                        if Self::end_of(*at, text) == *nat && !ntext.contains('\n') {
                            text.push_str(ntext);
                            return;
                        }
                    }
                }
            }
        }
        self.undo.push(UndoRec { edits: vec![edit], cursor_before, typing });
    }

    /// Вставить текст на место курсора, заменив выделение (`insertText`).
    pub fn insert_text(&mut self, text: &str, select: bool) {
        let before = self.cur;
        if self.selection().is_some() {
            self.delete_select();
        }
        let at = self.cur;
        let end = self.raw_insert(at, text);
        self.push_undo(Edit::Insert { at, text: text.replace('\r', "") }, before, false);
        if let Some(r) = self.undo.last_mut() {
            r.typing = false;
        }
        self.anchor = if select { Some(at) } else { None };
        self.cur = end;
        self.goal_x = None;
    }

    /// Удалить выделение (`deleteSelect`).
    pub fn delete_select(&mut self) {
        if let Some((a, b)) = self.selection() {
            let before = self.cur;
            let text = self.raw_delete(a, b);
            self.push_undo(Edit::Delete { at: a, text }, before, false);
            self.cur = a;
            self.anchor = None;
        }
    }

    /// Удалить диапазон (`deleteRange`).
    fn delete_range(&mut self, a: Pos, b: Pos) {
        if a == b {
            return;
        }
        let (a, b) = (min(a, b), max(a, b));
        let before = self.cur;
        let text = self.raw_delete(a, b);
        self.push_undo(Edit::Delete { at: a, text }, before, false);
        self.cur = a;
        self.anchor = None;
        self.goal_x = None;
    }

    /// Набор одного символа (с учётом режима замены).
    fn type_char(&mut self, c: char) {
        if self.selection().is_some() {
            self.delete_select();
        }
        let before = self.cur;
        if self.overwrite && self.cur.col < self.lines[self.cur.line].len() {
            let at = self.cur;
            let old = self.raw_delete(at, Pos::new(at.line, at.col + 1));
            self.push_undo(Edit::Delete { at, text: old }, before, false);
        }
        let at = self.cur;
        let mut tmp = [0u8; 4];
        let s = c.encode_utf8(&mut tmp);
        self.cur = self.raw_insert(at, s);
        self.push_undo(Edit::Insert { at, text: s.to_string() }, before, !self.overwrite);
        self.anchor = None;
        self.goal_x = None;
    }

    /// Новая строка с автоотступом (`newLine`).
    fn new_line(&mut self) {
        let indent: String = if self.auto_indent {
            self.lines[self.cur.line].iter().take_while(|c| **c == ' ' || **c == '\t').take(self.cur.col).collect()
        } else {
            String::new()
        };
        self.insert_text(&format!("\n{}", indent), false);
    }

    /// Отмена (`undo`).
    pub fn undo(&mut self) {
        if let Some(rec) = self.undo.pop() {
            for e in rec.edits.iter().rev() {
                match e {
                    Edit::Insert { at, text } => {
                        let end = Self::end_of(*at, text);
                        self.raw_delete(*at, end);
                    }
                    Edit::Delete { at, text } => {
                        self.raw_insert(*at, text);
                    }
                }
            }
            self.cur = self.clamp(rec.cursor_before);
            self.anchor = None;
            self.redo.push(rec);
            self.modified = true;
        }
    }

    /// Повтор отменённой правки.
    pub fn redo(&mut self) {
        if let Some(rec) = self.redo.pop() {
            let mut cur = rec.cursor_before;
            for e in rec.edits.iter() {
                match e {
                    Edit::Insert { at, text } => cur = self.raw_insert(*at, text),
                    Edit::Delete { at, text } => {
                        let end = Self::end_of(*at, text);
                        self.raw_delete(*at, end);
                        cur = *at;
                    }
                }
            }
            self.cur = self.clamp(cur);
            self.anchor = None;
            let mut rec = rec;
            rec.typing = false;
            self.undo.push(rec);
            self.modified = true;
        }
    }

    // ------------------------------------------------------------------
    // Экранные столбцы
    // ------------------------------------------------------------------

    /// Экранный столбец символа `col` в строке `line` (с учётом табуляций).
    pub fn col_to_x(&self, line: usize, col: usize) -> usize {
        let mut x = 0;
        for &c in self.lines[line].iter().take(col) {
            x += if c == '\t' { TAB_SIZE - x % TAB_SIZE } else { char_width(c).max(1) };
        }
        x
    }

    /// Номер символа, ближайшего к экранному столбцу `x`.
    pub fn x_to_col(&self, line: usize, x: usize) -> usize {
        let mut cx = 0;
        for (i, &c) in self.lines[line].iter().enumerate() {
            let w = if c == '\t' { TAB_SIZE - cx % TAB_SIZE } else { char_width(c).max(1) };
            if cx + w > x {
                return i;
            }
            cx += w;
        }
        self.lines[line].len()
    }

    fn max_width(&self) -> usize {
        (0..self.lines.len()).map(|l| self.col_to_x(l, self.lines[l].len())).max().unwrap_or(0)
    }

    // ------------------------------------------------------------------
    // Движение курсора
    // ------------------------------------------------------------------

    fn is_word_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    /// Начало следующего слова (`nextWord`).
    fn next_word(&self, p: Pos) -> Pos {
        let l = &self.lines[p.line];
        let mut c = p.col;
        if c >= l.len() {
            return if p.line + 1 < self.lines.len() { Pos::new(p.line + 1, 0) } else { p };
        }
        while c < l.len() && Self::is_word_char(l[c]) {
            c += 1;
        }
        while c < l.len() && !Self::is_word_char(l[c]) {
            c += 1;
        }
        Pos::new(p.line, c)
    }

    /// Начало предыдущего слова (`prevWord`).
    fn prev_word(&self, p: Pos) -> Pos {
        if p.col == 0 {
            return if p.line > 0 { Pos::new(p.line - 1, self.lines[p.line - 1].len()) } else { p };
        }
        let l = &self.lines[p.line];
        let mut c = p.col;
        while c > 0 && !Self::is_word_char(l[c - 1]) {
            c -= 1;
        }
        while c > 0 && Self::is_word_char(l[c - 1]) {
            c -= 1;
        }
        Pos::new(p.line, c)
    }

    fn char_left(&self, p: Pos) -> Pos {
        if p.col > 0 {
            Pos::new(p.line, p.col - 1)
        } else if p.line > 0 {
            Pos::new(p.line - 1, self.lines[p.line - 1].len())
        } else {
            p
        }
    }

    fn char_right(&self, p: Pos) -> Pos {
        if p.col < self.lines[p.line].len() {
            Pos::new(p.line, p.col + 1)
        } else if p.line + 1 < self.lines.len() {
            Pos::new(p.line + 1, 0)
        } else {
            p
        }
    }

    /// Вертикальное движение с сохранением желаемого столбца.
    fn line_move(&mut self, p: Pos, dy: isize) -> Pos {
        let gx = *self.goal_x.get_or_insert(self.col_to_x(p.line, p.col));
        let line = (p.line as isize + dy).clamp(0, self.lines.len() as isize - 1) as usize;
        Pos::new(line, self.x_to_col(line, gx))
    }

    /// Переместить курсор; `select` — расширять выделение (Shift).
    fn move_to(&mut self, p: Pos, select: bool) {
        if select {
            if self.anchor.is_none() {
                self.anchor = Some(self.cur);
            }
        } else {
            self.anchor = None;
        }
        self.cur = self.clamp(p);
    }

    /// Прокрутить так, чтобы курсор был виден (`trackCursor`).
    /// `center` — поставить курсор по центру экрана (после поиска).
    pub fn track_cursor(&mut self, center: bool) {
        let x = self.col_to_x(self.cur.line, self.cur.col) as i32;
        let y = self.cur.line as i32;
        let size = self.vs.size;
        if center {
            self.delta.x = max(0, x - size.x / 2).min(max(0, x - size.x + 1).max(self.delta.x.min(x)));
            self.delta.y = max(0, y - size.y / 2);
        }
        if x < self.delta.x {
            self.delta.x = x;
        } else if x >= self.delta.x + size.x {
            self.delta.x = x - size.x + 1;
        }
        if y < self.delta.y {
            self.delta.y = y;
        } else if y >= self.delta.y + size.y {
            self.delta.y = y - size.y + 1;
        }
        self.update_cursor();
    }

    /// Обновляет позицию экранного курсора.
    fn update_cursor(&mut self) {
        let x = self.col_to_x(self.cur.line, self.cur.col) as i32 - self.delta.x;
        let y = self.cur.line as i32 - self.delta.y;
        self.vs.set_cursor(x, y);
        if self.overwrite {
            self.vs.state |= sf::CURSOR_INS;
        } else {
            self.vs.state &= !sf::CURSOR_INS;
        }
    }

    /// Прокрутка к позиции (`scrollTo`) без перемещения курсора.
    pub fn scroll_to(&mut self, x: i32, y: i32) {
        let max_y = max(0, self.lines.len() as i32 - self.vs.size.y);
        self.delta.x = x.max(0);
        self.delta.y = y.clamp(0, max_y.max(0));
        self.update_cursor();
    }

    // ------------------------------------------------------------------
    // Поиск и замена
    // ------------------------------------------------------------------

    /// Поиск строки от курсора вперёд (`search`). При успехе выделяет найденное.
    pub fn search(&mut self, what: &str, flags: u16) -> bool {
        if what.is_empty() {
            return false;
        }
        let case = flags & ef::CASE_SENSITIVE != 0;
        let norm = |c: char| if case { c } else { crate::text::to_upper(c) };
        let pat: Vec<char> = what.chars().map(norm).collect();
        // Поиск в пределах строки (шаблон без перевода строки).
        let start = self.selection().map(|(_, b)| b).unwrap_or(self.cur);
        for line in start.line..self.lines.len() {
            let l: Vec<char> = self.lines[line].iter().map(|&c| norm(c)).collect();
            let from = if line == start.line { start.col } else { 0 };
            if l.len() < pat.len() {
                continue;
            }
            let mut i = from;
            while i + pat.len() <= l.len() {
                if l[i..i + pat.len()] == pat[..] {
                    let whole_ok = flags & ef::WHOLE_WORDS_ONLY == 0
                        || ((i == 0 || !Self::is_word_char(l[i - 1])) && (i + pat.len() == l.len() || !Self::is_word_char(l[i + pat.len()])));
                    if whole_ok {
                        self.set_select(Pos::new(line, i), Pos::new(line, i + pat.len()));
                        self.track_cursor(true);
                        return true;
                    }
                }
                i += 1;
            }
        }
        false
    }

    /// Поиск/замена по текущим настройкам (`doSearchReplace`).
    /// Если нужен вопрос «Заменить?», открывает окно и продолжит в `exec_result`.
    fn do_search_replace(&mut self, ctx: &mut Ctx) {
        loop {
            if !self.search(&self.find_str.clone(), self.editor_flags) {
                if self.editor_flags & (ef::REPLACE_ALL | ef::DO_REPLACE) != (ef::REPLACE_ALL | ef::DO_REPLACE) {
                    let d = build_message_box(Rect::new(0, 0, 40, 9), tr("\x03Search string not found."), mf::ERROR | mf::OK_BUTTON);
                    ctx.exec(Box::new(d), Some(self.vs.id), tag::INFO);
                }
                return;
            }
            if self.editor_flags & ef::DO_REPLACE == 0 {
                return;
            }
            if self.editor_flags & ef::PROMPT_ON_REPLACE != 0 {
                let mut d = build_message_box(Rect::new(0, 0, 40, 9), tr("\x03Replace this occurrence?"), mf::YES_NO_CANCEL | mf::INFORMATION);
                // Окно вопроса не должно закрывать найденный текст (как `replacePrompt`).
                d.win.group.vs.options &= !of::CENTERED;
                let cy = self.vs.cursor.y;
                let y = if cy < self.vs.size.y / 2 { 14 } else { 1 };
                d.win.group.vs.origin = Point::new(10, y);
                ctx.exec(Box::new(d), Some(self.vs.id), tag::PROMPT);
                return;
            }
            let r = self.replace_str.clone();
            self.insert_text(&r, false);
            self.track_cursor(false);
            if self.editor_flags & ef::REPLACE_ALL == 0 {
                return;
            }
        }
    }

    // ------------------------------------------------------------------
    // Команды
    // ------------------------------------------------------------------

    /// Выполняет команду редактора. Возвращает `true`, если команда известна.
    fn command(&mut self, c: u16, select: bool, ctx: &mut Ctx) -> bool {
        let p = self.cur;
        let page = max(1, self.vs.size.y - 1) as isize;
        match c {
            ec::CHAR_LEFT => {
                let np = if !select && self.selection().is_some() { self.selection().unwrap().0 } else { self.char_left(p) };
                self.goal_x = None;
                self.move_to(np, select);
            }
            ec::CHAR_RIGHT => {
                let np = if !select && self.selection().is_some() { self.selection().unwrap().1 } else { self.char_right(p) };
                self.goal_x = None;
                self.move_to(np, select);
            }
            ec::WORD_LEFT => {
                self.goal_x = None;
                let np = self.prev_word(p);
                self.move_to(np, select);
            }
            ec::WORD_RIGHT => {
                self.goal_x = None;
                let np = self.next_word(p);
                self.move_to(np, select);
            }
            ec::LINE_START => {
                self.goal_x = None;
                self.move_to(Pos::new(p.line, 0), select);
            }
            ec::LINE_END => {
                self.goal_x = None;
                self.move_to(Pos::new(p.line, self.lines[p.line].len()), select);
            }
            ec::LINE_UP => {
                let np = self.line_move(p, -1);
                self.move_to(np, select);
            }
            ec::LINE_DOWN => {
                let np = self.line_move(p, 1);
                self.move_to(np, select);
            }
            ec::PAGE_UP => {
                let np = self.line_move(p, -page);
                self.delta.y = max(0, self.delta.y - page as i32);
                self.move_to(np, select);
            }
            ec::PAGE_DOWN => {
                let np = self.line_move(p, page);
                self.delta.y += page as i32;
                self.move_to(np, select);
            }
            ec::TEXT_START => {
                self.goal_x = None;
                self.move_to(Pos::new(0, 0), select);
            }
            ec::TEXT_END => {
                self.goal_x = None;
                let l = self.lines.len() - 1;
                self.move_to(Pos::new(l, self.lines[l].len()), select);
            }
            ec::NEW_LINE => self.new_line(),
            ec::BACK_SPACE => {
                if self.selection().is_some() {
                    self.delete_select();
                } else {
                    let a = self.char_left(p);
                    self.delete_range(a, p);
                }
            }
            ec::DEL_CHAR => {
                if self.selection().is_some() {
                    self.delete_select();
                } else {
                    let b = self.char_right(p);
                    self.delete_range(p, b);
                }
            }
            ec::DEL_WORD => {
                if self.selection().is_some() {
                    self.delete_select();
                } else {
                    let b = self.next_word(p);
                    self.delete_range(p, b);
                }
            }
            ec::DEL_WORD_LEFT => {
                let a = self.prev_word(p);
                self.delete_range(a, p);
            }
            ec::DEL_START => self.delete_range(Pos::new(p.line, 0), p),
            ec::DEL_END => {
                let e = Pos::new(p.line, self.lines[p.line].len());
                self.delete_range(p, e);
            }
            ec::DEL_LINE => {
                let a = Pos::new(p.line, 0);
                let b = if p.line + 1 < self.lines.len() { Pos::new(p.line + 1, 0) } else { Pos::new(p.line, self.lines[p.line].len()) };
                self.delete_range(a, b);
            }
            ec::INS_MODE => self.overwrite = !self.overwrite,
            ec::INDENT_MODE => self.auto_indent = !self.auto_indent,
            ec::START_SELECT => self.anchor = Some(self.cur),
            ec::HIDE_SELECT => self.anchor = None,
            cm::SELECT_ALL => {
                let l = self.lines.len() - 1;
                self.set_select(Pos::new(0, 0), Pos::new(l, self.lines[l].len()));
            }
            cm::UNDO => self.undo(),
            cm::REDO => self.redo(),
            cm::CUT => {
                if self.selection().is_some() {
                    ctx.clipboard.set(&self.selected_text());
                    self.delete_select();
                }
            }
            cm::COPY => {
                if self.selection().is_some() {
                    ctx.clipboard.set(&self.selected_text());
                }
            }
            cm::PASTE => {
                let s = ctx.clipboard.get();
                if !s.is_empty() {
                    self.insert_text(&s, false);
                }
            }
            cm::CLEAR => self.delete_select(),
            cm::FIND => {
                let d = find_dialog(&self.find_str, self.editor_flags);
                ctx.exec(Box::new(d), Some(self.vs.id), tag::FIND);
            }
            cm::REPLACE => {
                let d = replace_dialog(&self.find_str, &self.replace_str, self.editor_flags);
                ctx.exec(Box::new(d), Some(self.vs.id), tag::REPLACE);
            }
            cm::SEARCH_AGAIN => self.do_search_replace(ctx),
            _ => return false,
        }
        self.track_cursor(false);
        true
    }

    /// Клавиша → команда по таблицам `firstKeys`/`quickKeys`/`blockKeys`.
    fn key_to_command(&mut self, k: KeyEvent) -> Option<u16> {
        let state = self.key_state;
        self.key_state = KeyState::Normal;
        let letter = match k.key {
            Key::Char(c) => Some(c.to_ascii_uppercase()),
            _ => None,
        };
        match state {
            KeyState::Quick => {
                return match letter? {
                    'A' => Some(cm::REPLACE),
                    'C' => Some(ec::TEXT_END),
                    'D' => Some(ec::LINE_END),
                    'F' => Some(cm::FIND),
                    'H' => Some(ec::DEL_START),
                    'R' => Some(ec::TEXT_START),
                    'S' => Some(ec::LINE_START),
                    'Y' => Some(ec::DEL_END),
                    _ => None,
                }
            }
            KeyState::Block => {
                return match letter? {
                    'B' => Some(ec::START_SELECT),
                    'C' => Some(cm::PASTE),
                    'H' => Some(ec::HIDE_SELECT),
                    'K' => Some(cm::COPY),
                    'Y' => Some(cm::CUT),
                    _ => None,
                }
            }
            KeyState::Normal => {}
        }
        let m = k.mods;
        let plain = !m.ctrl && !m.alt;
        if m.ctrl && !m.alt {
            if let Some(c) = letter {
                return match c {
                    'A' => Some(cm::SELECT_ALL),
                    'C' => Some(ec::PAGE_DOWN),
                    'D' => Some(ec::CHAR_RIGHT),
                    'E' => Some(ec::LINE_UP),
                    'F' => Some(ec::WORD_RIGHT),
                    'G' => Some(ec::DEL_CHAR),
                    'H' => Some(ec::BACK_SPACE),
                    'K' => {
                        self.key_state = KeyState::Block;
                        Some(0)
                    }
                    'L' => Some(cm::SEARCH_AGAIN),
                    'M' => Some(ec::NEW_LINE),
                    'O' => Some(ec::INDENT_MODE),
                    'Q' => {
                        self.key_state = KeyState::Quick;
                        Some(0)
                    }
                    'R' => Some(ec::PAGE_UP),
                    'S' => Some(ec::CHAR_LEFT),
                    'T' => Some(ec::DEL_WORD),
                    'U' | 'Z' => Some(cm::UNDO),
                    'V' => Some(ec::INS_MODE),
                    'X' => Some(ec::LINE_DOWN),
                    'Y' => Some(ec::DEL_LINE),
                    _ => None,
                };
            }
        }
        match (k.key, m.ctrl, m.alt) {
            (Key::Left, false, false) => Some(ec::CHAR_LEFT),
            (Key::Right, false, false) => Some(ec::CHAR_RIGHT),
            (Key::Left, true, false) => Some(ec::WORD_LEFT),
            (Key::Right, true, false) => Some(ec::WORD_RIGHT),
            (Key::Home, false, false) => Some(ec::LINE_START),
            (Key::End, false, false) => Some(ec::LINE_END),
            (Key::Home, true, false) => Some(ec::TEXT_START),
            (Key::End, true, false) => Some(ec::TEXT_END),
            (Key::Up, false, false) => Some(ec::LINE_UP),
            (Key::Down, false, false) => Some(ec::LINE_DOWN),
            (Key::PgUp, false, false) => Some(ec::PAGE_UP),
            (Key::PgDn, false, false) => Some(ec::PAGE_DOWN),
            (Key::Backspace, false, false) => Some(ec::BACK_SPACE),
            (Key::Backspace, _, _) => Some(ec::DEL_WORD_LEFT),
            (Key::Enter, false, false) => Some(ec::NEW_LINE),
            (Key::Del, false, false) if !m.shift => Some(ec::DEL_CHAR),
            (Key::Del, false, false) => Some(cm::CUT),
            (Key::Del, true, false) => Some(cm::CLEAR),
            (Key::Ins, false, false) if m.shift => Some(cm::PASTE),
            (Key::Ins, false, false) => Some(ec::INS_MODE),
            (Key::Ins, true, false) => Some(cm::COPY),
            _ if plain => None,
            _ => None,
        }
    }

    /// Точка мыши (координаты владельца) → позиция в тексте.
    fn pos_at(&self, mouse: Point) -> Pos {
        let p = self.vs.make_local(mouse);
        let line = (self.delta.y + p.y).clamp(0, self.lines.len() as i32 - 1) as usize;
        let x = max(0, self.delta.x + p.x) as usize;
        Pos::new(line, self.x_to_col(line, x))
    }

    /// Шлёт полосам прокрутки и индикатору актуальные значения.
    fn sync(&mut self, ctx: &mut Ctx) {
        // Прокрутка не должна уходить за конец текста.
        let max_y = max(0, self.lines.len() as i32 - 1);
        self.delta.y = self.delta.y.clamp(0, max_y);
        self.update_cursor();
        let limit = Point::new(max(self.max_width() as i32 + 1, self.vs.size.x), self.lines.len() as i32);
        let key = (self.delta, limit, self.cur, self.modified);
        if self.last_sync == Some(key) {
            return;
        }
        self.last_sync = Some(key);
        let size = self.vs.size;
        if let Some(id) = self.v_scroll {
            ctx.message(
                cm::SCROLL_BAR_SET,
                Info::Scroll { id, value: self.delta.y, min: 0, max: max(0, limit.y - size.y), page: max(1, size.y - 1), arrow: 1 },
            );
        }
        if let Some(id) = self.h_scroll {
            ctx.message(
                cm::SCROLL_BAR_SET,
                Info::Scroll { id, value: self.delta.x, min: 0, max: max(0, limit.x - size.x), page: max(1, size.x / 2), arrow: 1 },
            );
        }
        if let Some(id) = self.indicator {
            let col = self.col_to_x(self.cur.line, self.cur.col);
            ctx.message(cm::INDICATOR_SET, Info::Indicator { id, line: self.cur.line, col, modified: self.modified });
        }
    }

    /// Общая обработка событий; `memo` — поведение `TMemo` (Tab не перехватывается).
    fn handle_editor_event(&mut self, ev: &mut Event, ctx: &mut Ctx, memo: bool) {
        default_handle_event(&self.vs, ev, ctx);
        let id = self.vs.id;
        match ev {
            Event::MouseDown(m) => {
                let shift = m.mods.shift;
                let p = self.pos_at(m.pos);
                if m.double {
                    // Двойной щелчок — выделить слово.
                    let l = &self.lines[p.line];
                    let mut a = p.col;
                    let mut b = p.col;
                    while a > 0 && Self::is_word_char(l[a - 1]) {
                        a -= 1;
                    }
                    while b < l.len() && Self::is_word_char(l[b]) {
                        b += 1;
                    }
                    self.set_select(Pos::new(p.line, a), Pos::new(p.line, b));
                } else {
                    self.goal_x = None;
                    self.move_to(p, shift);
                    if !shift {
                        self.anchor = Some(p);
                    }
                    self.selecting = true;
                    ctx.grab_mouse(id);
                }
                self.update_cursor();
                ev.clear();
            }
            Event::MouseMove(m) | Event::MouseAuto(m) if ctx.mouse_grabbed_by(id) && self.selecting => {
                let local = self.vs.make_local(m.pos);
                // Автопрокрутка при выходе за край.
                if local.y < 0 {
                    self.delta.y = max(0, self.delta.y - 1);
                } else if local.y >= self.vs.size.y {
                    self.delta.y += 1;
                }
                if local.x < 0 {
                    self.delta.x = max(0, self.delta.x - 1);
                } else if local.x >= self.vs.size.x {
                    self.delta.x += 1;
                }
                let p = self.pos_at(m.pos);
                self.cur = p;
                self.goal_x = None;
                self.update_cursor();
                ev.clear();
            }
            Event::MouseUp(_) if ctx.mouse_grabbed_by(id) => {
                ctx.release_mouse();
                self.selecting = false;
                ev.clear();
            }
            Event::MouseWheel(m) => {
                let ny = self.delta.y + m.wheel as i32 * 3;
                self.scroll_to(self.delta.x, ny);
                ev.clear();
            }
            Event::KeyDown(k) if self.vs.is_focused() => {
                let k = *k;
                if memo && (k.key == Key::Tab || k.key == Key::BackTab) && self.key_state == KeyState::Normal {
                    // В `TMemo` Tab переключает элементы диалога.
                } else if let Some(c) = self.key_to_command(k) {
                    if c != 0 {
                        // Shift + движение — выделение.
                        let select = k.mods.shift && !matches!(c, cm::PASTE | cm::CUT);
                        self.command(c, select, ctx);
                    }
                    ev.clear();
                } else if let Some(ch) = k.text_char() {
                    self.type_char(ch);
                    self.track_cursor(false);
                    ev.clear();
                } else if k.key == Key::Tab && k.mods == Mods::NONE {
                    self.type_char('\t');
                    self.track_cursor(false);
                    ev.clear();
                }
            }
            Event::Command(m) if self.vs.is_focused() => {
                let c = m.command;
                if matches!(
                    c,
                    cm::UNDO | cm::REDO | cm::CUT | cm::COPY | cm::PASTE | cm::CLEAR | cm::FIND | cm::REPLACE | cm::SEARCH_AGAIN | cm::SELECT_ALL
                ) && self.command(c, false, ctx)
                {
                    ev.clear();
                }
            }
            Event::Broadcast(m) => match (&m.command, &m.info) {
                (&cm::SCROLL_BAR_CHANGED, Info::Scroll { id: sb, value, .. }) if Some(*sb) == self.v_scroll => {
                    self.scroll_to(self.delta.x, *value);
                    self.last_sync = None;
                }
                (&cm::SCROLL_BAR_CHANGED, Info::Scroll { id: sb, value, .. }) if Some(*sb) == self.h_scroll => {
                    self.scroll_to(*value, self.delta.y);
                    self.last_sync = None;
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
        self.sync(ctx);
    }

    /// Результаты диалогов поиска/замены и вопроса о замене.
    fn editor_exec_result(&mut self, tag_: u16, command: u16, view: Box<dyn View>, ctx: &mut Ctx) {
        match tag_ {
            tag::FIND | tag::REPLACE if command == cm::OK => {
                let data = view.get_data();
                if let Some(items) = data.as_list() {
                    let mut it = items.iter();
                    self.find_str = it.next().and_then(|v| v.as_str()).unwrap_or("").to_string();
                    if tag_ == tag::REPLACE {
                        self.replace_str = it.next().and_then(|v| v.as_str()).unwrap_or("").to_string();
                    }
                    let opts = it.next().and_then(|v| v.as_u32()).unwrap_or(0) as u16;
                    // Флажки диалога: 0 — регистр, 1 — целые слова, (замена) 2 — спрашивать, 3 — заменить все.
                    self.editor_flags = opts & (ef::CASE_SENSITIVE | ef::WHOLE_WORDS_ONLY | ef::PROMPT_ON_REPLACE | ef::REPLACE_ALL);
                    if tag_ == tag::REPLACE {
                        self.editor_flags |= ef::DO_REPLACE;
                    }
                }
                self.do_search_replace(ctx);
            }
            tag::PROMPT => match command {
                cm::YES => {
                    let r = self.replace_str.clone();
                    self.insert_text(&r, false);
                    self.track_cursor(false);
                    if self.editor_flags & ef::REPLACE_ALL != 0 {
                        self.do_search_replace(ctx);
                    }
                }
                cm::NO => {
                    if self.editor_flags & ef::REPLACE_ALL != 0 {
                        self.do_search_replace(ctx);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        self.sync(ctx);
    }

    /// Обновление команд (`updateCommands`).
    fn editor_update_commands(&self, ctx: &mut Ctx) {
        let focused = self.vs.is_focused();
        let sel = self.selection().is_some();
        ctx.set_command(cm::UNDO, focused && !self.undo.is_empty());
        ctx.set_command(cm::REDO, focused && !self.redo.is_empty());
        ctx.set_command(cm::CUT, focused && sel);
        ctx.set_command(cm::COPY, focused && sel);
        ctx.set_command(cm::CLEAR, focused && sel);
        ctx.set_command(cm::PASTE, focused && ctx.clipboard.has_text());
        ctx.set_command(cm::FIND, focused);
        ctx.set_command(cm::REPLACE, focused);
        ctx.set_command(cm::SEARCH_AGAIN, focused);
    }

    /// Отрисовка (`TEditor::draw` / `drawLines`).
    fn draw_editor(&self, dc: &mut DrawCtx) {
        let normal = dc.color(1);
        let selected = dc.color(2);
        let w = self.vs.size.x.max(0) as usize;
        let sel = self.selection();
        for row in 0..self.vs.size.y {
            let mut b = DrawBuffer::new(w);
            b.move_char(0, ' ', normal, w);
            let li = (self.delta.y + row) as usize;
            if li < self.lines.len() {
                let line = &self.lines[li];
                let mut x = 0usize;
                let dx = self.delta.x.max(0) as usize;
                // Цвет хвоста строки: выделение, переходящее на следующую строку.
                let in_sel = |col: usize| sel.map(|(a, b)| Pos::new(li, col) >= a && Pos::new(li, col) < b).unwrap_or(false);
                for (ci, &c) in line.iter().enumerate() {
                    let cw = if c == '\t' { TAB_SIZE - x % TAB_SIZE } else { char_width(c).max(1) };
                    let attr = if in_sel(ci) { selected } else { normal };
                    // Видимая часть символа на экране: [x, x + cw) ∩ [dx, dx + w).
                    let vis_a = x.max(dx);
                    let vis_b = (x + cw).min(dx + w);
                    if vis_a < vis_b {
                        if c == '\t' || vis_b - vis_a < cw {
                            // Табуляция или широкий символ, обрезанный краем, — пробелы.
                            b.move_char(vis_a - dx, ' ', attr, vis_b - vis_a);
                        } else {
                            let mut tmp = [0u8; 4];
                            b.move_str(x - dx, c.encode_utf8(&mut tmp), attr);
                        }
                    }
                    x += cw;
                    if x >= dx + w {
                        break;
                    }
                }
                // Выделенный перевод строки показываем одной клеткой.
                if let Some((a, bnd)) = sel {
                    let eol = Pos::new(li, line.len());
                    if eol >= a && eol < bnd && x >= dx && x - dx < w {
                        b.move_char(x - dx, ' ', selected, 1);
                    }
                }
            }
            dc.write_line(0, row, self.vs.size.x, 1, &b);
        }
    }
}

/// Внутренние команды редактора (`cmCharLeft` ... `cmDelWordLeft`).
pub mod ec {
    pub const CHAR_LEFT: u16 = 500;
    pub const CHAR_RIGHT: u16 = 501;
    pub const WORD_LEFT: u16 = 502;
    pub const WORD_RIGHT: u16 = 503;
    pub const LINE_START: u16 = 504;
    pub const LINE_END: u16 = 505;
    pub const LINE_UP: u16 = 506;
    pub const LINE_DOWN: u16 = 507;
    pub const PAGE_UP: u16 = 508;
    pub const PAGE_DOWN: u16 = 509;
    pub const TEXT_START: u16 = 510;
    pub const TEXT_END: u16 = 511;
    pub const NEW_LINE: u16 = 512;
    pub const BACK_SPACE: u16 = 513;
    pub const DEL_CHAR: u16 = 514;
    pub const DEL_WORD: u16 = 515;
    pub const DEL_START: u16 = 516;
    pub const DEL_END: u16 = 517;
    pub const DEL_LINE: u16 = 518;
    pub const INS_MODE: u16 = 519;
    pub const START_SELECT: u16 = 520;
    pub const HIDE_SELECT: u16 = 521;
    pub const INDENT_MODE: u16 = 522;
    pub const DEL_WORD_LEFT: u16 = 525;
}

/// Разбивает текст на строки (понимает `\n`, `\r\n` и `\r`).
fn split_lines(s: &str) -> Vec<Vec<char>> {
    let s = s.replace("\r\n", "\n").replace('\r', "\n");
    s.split('\n').map(|l| l.chars().collect()).collect()
}

impl View for Editor {
    crate::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        self.draw_editor(dc);
    }

    fn palette(&self) -> &'static [u8] {
        self.palette
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        self.handle_editor_event(ev, ctx, false);
    }

    fn exec_result(&mut self, tag: u16, command: u16, view: Box<dyn View>, ctx: &mut Ctx) {
        self.editor_exec_result(tag, command, view, ctx);
    }

    fn update_commands(&self, ctx: &mut Ctx) {
        self.editor_update_commands(ctx);
    }

    fn change_bounds(&mut self, bounds: Rect) {
        self.vs.set_bounds(bounds);
        self.last_sync = None;
        self.update_cursor();
    }
}

// ----------------------------------------------------------------------
// TMemo
// ----------------------------------------------------------------------

/// Многострочное поле в диалоге (`TMemo`): данные — весь текст,
/// Tab переключает элементы диалога.
pub struct Memo {
    pub editor: Editor,
}

impl Memo {
    pub fn new(bounds: Rect, h_scroll: Option<ViewId>, v_scroll: Option<ViewId>) -> Self {
        let mut editor = Editor::new(bounds, h_scroll, v_scroll, None);
        editor.palette = palettes::MEMO;
        editor.vs.grow_mode = 0;
        Memo { editor }
    }
}

impl View for Memo {
    crate::impl_view_basics_field!(editor);

    fn draw(&self, dc: &mut DrawCtx) {
        self.editor.draw_editor(dc);
    }

    fn palette(&self) -> &'static [u8] {
        self.editor.palette
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        self.editor.handle_editor_event(ev, ctx, true);
    }

    fn exec_result(&mut self, tag: u16, command: u16, view: Box<dyn View>, ctx: &mut Ctx) {
        self.editor.editor_exec_result(tag, command, view, ctx);
    }

    fn update_commands(&self, ctx: &mut Ctx) {
        self.editor.editor_update_commands(ctx);
    }

    fn has_data(&self) -> bool {
        true
    }

    fn get_data(&self) -> Value {
        Value::Str(self.editor.text())
    }

    fn set_data(&mut self, value: &Value) {
        if let Value::Str(s) = value {
            self.editor.set_text(s);
        }
    }
}

// ----------------------------------------------------------------------
// TFileEditor
// ----------------------------------------------------------------------

/// Редактор файла (`TFileEditor`): загрузка, сохранение, «Сохранить как»,
/// вопрос о сохранении изменений при закрытии.
pub struct FileEditor {
    pub editor: Editor,
    /// Имя файла; пустое — «Untitled».
    pub file_name: PathBuf,
    /// Команда, которую надо повторить после ответа на «Сохранить изменения?».
    pending: Option<u16>,
    /// Ошибка последней операции с файлом (для тестов/отладки).
    pub last_error: Option<String>,
}

/// Заголовок безымянного файла (`untitled`).
pub const UNTITLED: &str = "Untitled";

impl FileEditor {
    /// Аналог `TFileEditor(bounds, h, v, indicator, fileName)`: если файл
    /// существует, он загружается.
    pub fn new(bounds: Rect, h: Option<ViewId>, v: Option<ViewId>, indicator: Option<ViewId>, file_name: &Path) -> Self {
        let mut fe = FileEditor { editor: Editor::new(bounds, h, v, indicator), file_name: PathBuf::new(), pending: None, last_error: None };
        if !file_name.as_os_str().is_empty() {
            fe.file_name = absolute(file_name);
            if fe.file_name.exists() {
                fe.load_file();
            }
        }
        fe
    }

    /// Загрузить файл (`loadFile`).
    pub fn load_file(&mut self) -> bool {
        match std::fs::read(&self.file_name) {
            Ok(bytes) => {
                let s = String::from_utf8_lossy(&bytes);
                self.editor.set_text(&s);
                true
            }
            Err(e) => {
                self.last_error = Some(format!("{} {}.\n{}", tr("Error reading file"), self.file_name.display(), e));
                false
            }
        }
    }

    /// Записать файл (`saveFile`). С флагом `ef::BACKUP_FILES` старая версия
    /// сохраняется с расширением `.bak`.
    pub fn save_file(&mut self) -> bool {
        if self.editor.editor_flags & ef::BACKUP_FILES != 0 && self.file_name.exists() {
            let bak = self.file_name.with_extension("bak");
            let _ = std::fs::rename(&self.file_name, bak);
        }
        let text = self.editor.text();
        match std::fs::write(&self.file_name, text) {
            Ok(()) => {
                self.editor.modified = false;
                self.editor.last_sync = None;
                true
            }
            Err(e) => {
                self.last_error = Some(format!("{} {}.\n{}", tr("Error writing file"), self.file_name.display(), e));
                false
            }
        }
    }

    /// Сохранить (`save`): безымянный файл — через «Сохранить как».
    fn save(&mut self, ctx: &mut Ctx) -> bool {
        if self.file_name.as_os_str().is_empty() {
            self.save_as(ctx);
            false
        } else {
            let ok = self.save_file();
            if !ok {
                self.show_error(ctx);
            }
            ok
        }
    }

    /// Открывает диалог «Сохранить как» (`saveAs`).
    fn save_as(&mut self, ctx: &mut Ctx) {
        let d = crate::widgets::filedlg::FileDialog::new("*", tr("Save file as"), tr("~N~ame"), crate::widgets::filedlg::fd::OK_BUTTON, 101);
        ctx.exec(Box::new(d), Some(self.editor.vs.id), tag::SAVE_AS);
    }

    fn show_error(&mut self, ctx: &mut Ctx) {
        if let Some(e) = self.last_error.take() {
            let d = build_message_box(Rect::new(0, 0, 50, 10), &e, mf::ERROR | mf::OK_BUTTON);
            ctx.exec(Box::new(d), None, 0);
        }
    }

    /// Имя для заголовка окна.
    pub fn title(&self) -> String {
        if self.file_name.as_os_str().is_empty() {
            tr(UNTITLED).into()
        } else {
            self.file_name.display().to_string()
        }
    }
}

/// Абсолютный путь без обращения к ФС (аналог `fexpand`).
pub fn absolute(p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir().map(|d| d.join(p)).unwrap_or_else(|_| p.to_path_buf())
    }
}

impl View for FileEditor {
    crate::impl_view_basics_field!(editor);

    fn draw(&self, dc: &mut DrawCtx) {
        self.editor.draw_editor(dc);
    }

    fn palette(&self) -> &'static [u8] {
        self.editor.palette
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        if self.editor.vs.is_focused() {
            match ev.command() {
                Some(cm::SAVE) => {
                    self.save(ctx);
                    ev.clear();
                    return;
                }
                Some(cm::SAVE_AS) => {
                    self.save_as(ctx);
                    ev.clear();
                    return;
                }
                _ => {}
            }
        }
        self.editor.handle_editor_event(ev, ctx, false);
    }

    fn update_commands(&self, ctx: &mut Ctx) {
        self.editor.editor_update_commands(ctx);
        let f = self.editor.vs.is_focused();
        ctx.set_command(cm::SAVE, f);
        ctx.set_command(cm::SAVE_AS, f);
    }

    /// `TFileEditor::valid`: при закрытии изменённого файла — вопрос.
    fn valid(&mut self, command: u16, ctx: &mut Ctx) -> bool {
        if command == cm::VALID {
            return true;
        }
        if !self.editor.modified || !matches!(command, cm::CLOSE | cm::QUIT) {
            return true;
        }
        let msg = if self.file_name.as_os_str().is_empty() {
            tr("\x03Save untitled file?").to_string()
        } else {
            format!(
                "\x03{} {}",
                self.file_name.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
                tr("has been modified. Save?")
            )
        };
        let d = build_message_box(Rect::new(0, 0, 50, 9), &msg, mf::INFORMATION | mf::YES_NO_CANCEL);
        self.pending = Some(command);
        ctx.exec(Box::new(d), Some(self.editor.vs.id), tag::SAVE_MODIFY);
        false
    }

    fn exec_result(&mut self, tag_: u16, command: u16, view: Box<dyn View>, ctx: &mut Ctx) {
        match tag_ {
            tag::SAVE_MODIFY => {
                let pending = self.pending.take();
                match command {
                    cm::YES => {
                        if self.save(ctx) {
                            if let Some(c) = pending {
                                ctx.post_command(c, Info::None);
                            }
                        } else {
                            // Безымянный файл: после «Сохранить как» повторим команду.
                            self.pending = pending;
                        }
                    }
                    cm::NO => {
                        self.editor.modified = false;
                        if let Some(c) = pending {
                            ctx.post_command(c, Info::None);
                        }
                    }
                    _ => {}
                }
            }
            tag::SAVE_AS => {
                if command != cm::CANCEL {
                    if let Ok(d) = view.into_any().downcast::<crate::widgets::filedlg::FileDialog>() {
                        let name = d.file_name();
                        if !name.as_os_str().is_empty() {
                            self.file_name = name;
                            if self.save_file() {
                                if let Some(c) = self.pending.take() {
                                    ctx.post_command(c, Info::None);
                                }
                            } else {
                                self.show_error(ctx);
                            }
                        }
                    }
                } else {
                    self.pending = None;
                }
                self.editor.last_sync = None;
            }
            _ => self.editor.editor_exec_result(tag_, command, view, ctx),
        }
        self.editor.sync(ctx);
    }
}

// ----------------------------------------------------------------------
// TIndicator
// ----------------------------------------------------------------------

/// Индикатор строки:столбца в нижней рамке окна редактора (`TIndicator`).
pub struct Indicator {
    pub vs: ViewState,
    pub location: Point,
    pub modified: bool,
}

impl Indicator {
    pub fn new(bounds: Rect) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.grow_mode = gf::GROW_LO_Y | gf::GROW_HI_Y;
        Indicator { vs, location: Point::default(), modified: false }
    }
}

impl View for Indicator {
    crate::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let (color, frame) = if self.vs.state & sf::DRAGGING == 0 { (dc.color(1), '═') } else { (dc.color(2), '─') };
        let w = self.vs.size.x.max(0) as usize;
        let mut b = DrawBuffer::new(w);
        b.move_char(0, frame, color, w);
        if self.modified {
            b.put_char(0, '☼');
        }
        let s = format!(" {}:{} ", self.location.y + 1, self.location.x + 1);
        let colon = s.find(':').unwrap_or(0);
        b.move_str(8usize.saturating_sub(colon), &s, color);
        dc.write_line(0, 0, self.vs.size.x, 1, &b);
    }

    fn palette(&self) -> &'static [u8] {
        palettes::INDICATOR
    }

    fn handle_event(&mut self, ev: &mut Event, _ctx: &mut Ctx) {
        if let Event::Broadcast(m) = ev {
            if let (cm::INDICATOR_SET, Info::Indicator { id, line, col, modified }) = (m.command, &m.info) {
                if *id == self.vs.id {
                    self.location = Point::new(*col as i32, *line as i32);
                    self.modified = *modified;
                }
            }
        }
    }
}

// ----------------------------------------------------------------------
// TEditWindow
// ----------------------------------------------------------------------

/// Окно редактора файла (`TEditWindow`): рамка, полосы прокрутки, индикатор.
pub struct EditWindow {
    pub win: Window,
    pub editor_id: ViewId,
}

impl EditWindow {
    /// Пустое `file_name` — новый безымянный файл.
    pub fn new(bounds: Rect, file_name: &Path, number: i16) -> Self {
        let mut win = Window::new(bounds, "", number);
        win.group.vs.options |= of::TILEABLE;
        let size = win.group.vs.size;
        let mut hsb = ScrollBar::new(Rect::new(18, size.y - 1, size.x - 2, size.y));
        hsb.vs.state &= !sf::VISIBLE;
        let hsb = win.add(hsb);
        let vsb = win.standard_scroll_bar(true);
        let ind = win.add(Indicator::new(Rect::new(2, size.y - 1, 16, size.y)));
        let fe = FileEditor::new(win.interior(), Some(hsb), Some(vsb), Some(ind), file_name);
        let editor_id = win.add(fe);
        let mut w = EditWindow { win, editor_id };
        w.show_h_scroll(true);
        w
    }

    fn show_h_scroll(&mut self, show: bool) {
        // Горизонтальная полоса видна только у активного окна (как в оригинале).
        let ids: Vec<ViewId> = self.win.group.children.iter().filter(|c| c.as_any().is::<ScrollBar>()).map(|c| c.vs().id).collect();
        for id in ids {
            if let Some(sb) = self.win.group.find_mut::<ScrollBar>(id) {
                if sb.vs.size.y == 1 {
                    if show {
                        sb.vs.state |= sf::VISIBLE;
                    } else {
                        sb.vs.state &= !sf::VISIBLE;
                    }
                }
            }
        }
    }

    pub fn editor(&self) -> &FileEditor {
        self.win.group.find::<FileEditor>(self.editor_id).unwrap()
    }

    pub fn editor_mut(&mut self) -> &mut FileEditor {
        let id = self.editor_id;
        self.win.group.find_mut::<FileEditor>(id).unwrap()
    }
}

impl View for EditWindow {
    crate::impl_group_basics!(win.group);

    fn as_window(&self) -> Option<&Window> {
        Some(&self.win)
    }

    fn draw(&self, dc: &mut DrawCtx) {
        // Заголовок (имя файла) обновляется в `handle_event`.
        self.win.draw_window(dc);
    }

    fn palette(&self) -> &'static [u8] {
        self.win.palette
    }

    fn set_state(&mut self, flag: u16, enable: bool) {
        self.win.set_window_state(flag, enable);
        if flag & sf::ACTIVE != 0 || flag & sf::SELECTED != 0 {
            let active = self.win.group.vs.is_active();
            self.show_h_scroll(active);
        }
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        self.win.handle_window_event(ev, ctx);
        let t = self.editor().title();
        if self.win.title != t {
            self.win.title = t;
        }
    }

    fn valid(&mut self, command: u16, ctx: &mut Ctx) -> bool {
        self.win.group.valid(command, ctx)
    }

    fn size_limits(&self, owner_size: Point) -> (Point, Point) {
        (crate::widgets::window::MIN_WIN_SIZE, owner_size)
    }
}

// ----------------------------------------------------------------------
// Диалоги поиска и замены (из примера TVEDIT оригинала)
// ----------------------------------------------------------------------

/// Диалог «Find». Данные: [текст, флажки].
pub fn find_dialog(find: &str, flags: u16) -> Dialog {
    let mut d = Dialog::centered(38, 12, tr("Find"));
    let mut il = InputLine::new(Rect::new(3, 3, 32, 4), 80);
    il.history_id = Some(10);
    let il = d.add(il);
    d.add(Label::new(Rect::new(2, 2, 15, 3), tr("~T~ext to find"), Some(il)));
    d.add(crate::widgets::history::History::new(Rect::new(32, 3, 35, 4), il, 10));
    d.add(Cluster::check_boxes(Rect::new(3, 5, 35, 7), &[tr("~C~ase sensitive"), tr("~W~hole words only")]));
    d.add(Button::new(Rect::new(14, 9, 24, 11), tr("O~K~"), cm::OK, bf::DEFAULT));
    d.add(Button::new(Rect::new(26, 9, 36, 11), tr("Cancel"), cm::CANCEL, bf::NORMAL));
    d.set_data(&Value::List(vec![Value::Str(find.into()), Value::U32((flags & 3) as u32)]));
    d.select(il);
    d
}

/// Диалог «Replace». Данные: [что, на что, флажки].
pub fn replace_dialog(find: &str, replace: &str, flags: u16) -> Dialog {
    let mut d = Dialog::centered(40, 16, tr("Replace"));
    let mut il = InputLine::new(Rect::new(3, 3, 34, 4), 80);
    il.history_id = Some(10);
    let il = d.add(il);
    d.add(Label::new(Rect::new(2, 2, 15, 3), tr("~T~ext to find"), Some(il)));
    d.add(crate::widgets::history::History::new(Rect::new(34, 3, 37, 4), il, 10));
    let mut il2 = InputLine::new(Rect::new(3, 6, 34, 7), 80);
    il2.history_id = Some(11);
    let il2 = d.add(il2);
    d.add(Label::new(Rect::new(2, 5, 16, 6), tr("~N~ew text"), Some(il2)));
    d.add(crate::widgets::history::History::new(Rect::new(34, 6, 37, 7), il2, 11));
    d.add(Cluster::check_boxes(
        Rect::new(3, 8, 37, 12),
        &[tr("~C~ase sensitive"), tr("~W~hole words only"), tr("~P~rompt on replace"), tr("~R~eplace all")],
    ));
    d.add(Button::new(Rect::new(17, 13, 27, 15), tr("O~K~"), cm::OK, bf::DEFAULT));
    d.add(Button::new(Rect::new(28, 13, 38, 15), tr("Cancel"), cm::CANCEL, bf::NORMAL));
    d.set_data(&Value::List(vec![Value::Str(find.into()), Value::Str(replace.into()), Value::U32((flags & 0x0F) as u32)]));
    d.select(il);
    d
}

/// Ширина строки на экране (с табуляциями) — для внешних пользователей.
pub fn display_width(s: &str) -> usize {
    let mut x = 0;
    for c in s.chars() {
        x += if c == '\t' { TAB_SIZE - x % TAB_SIZE } else { char_width(c).max(1) };
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ed(text: &str) -> Editor {
        let mut e = Editor::new(Rect::new(0, 0, 20, 5), None, None, None);
        e.set_text(text);
        e
    }

    #[test]
    fn insert_delete_undo() {
        let mut e = ed("hello\nworld");
        e.cur = Pos::new(0, 5);
        for c in " there".chars() {
            e.type_char(c);
        }
        assert_eq!(e.text(), "hello there\nworld");
        // Набор подряд — одна запись отмены.
        e.undo();
        assert_eq!(e.text(), "hello\nworld");
        e.redo();
        assert_eq!(e.text(), "hello there\nworld");
        e.set_select(Pos::new(0, 5), Pos::new(1, 2));
        e.delete_select();
        assert_eq!(e.text(), "hellorld");
        e.undo();
        assert_eq!(e.text(), "hello there\nworld");
    }

    #[test]
    fn newline_autoindent() {
        let mut e = ed("    abc");
        e.cur = Pos::new(0, 7);
        e.new_line();
        assert_eq!(e.text(), "    abc\n    ");
        assert_eq!(e.cursor(), Pos::new(1, 4));
    }

    #[test]
    fn tabs_and_columns() {
        let e = ed("a\tb");
        assert_eq!(e.col_to_x(0, 2), 8);
        assert_eq!(e.x_to_col(0, 5), 1);
        assert_eq!(e.x_to_col(0, 8), 2);
    }

    #[test]
    fn search_options() {
        let mut e = ed("Foo foobar foo");
        assert!(e.search("foo", 0));
        assert_eq!(e.selection(), Some((Pos::new(0, 0), Pos::new(0, 3))));
        assert!(e.search("foo", ef::CASE_SENSITIVE));
        assert_eq!(e.selection(), Some((Pos::new(0, 4), Pos::new(0, 7))));
        assert!(e.search("foo", ef::CASE_SENSITIVE | ef::WHOLE_WORDS_ONLY));
        assert_eq!(e.selection(), Some((Pos::new(0, 11), Pos::new(0, 14))));
        assert!(!e.search("foo", 0));
    }

    #[test]
    fn words() {
        let e = ed("one two  three");
        assert_eq!(e.next_word(Pos::new(0, 0)), Pos::new(0, 4));
        assert_eq!(e.prev_word(Pos::new(0, 9)), Pos::new(0, 4));
    }
}
