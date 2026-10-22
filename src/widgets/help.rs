//! Справка: `THelpFile`, `THelpTopic`, `THelpViewer`, `THelpWindow`
//! (`helpbase.cpp`, `help.cpp`) и разбор исходника справки в формате `tvhc`.
//!
//! В оригинале исходный текст справки компилировался утилитой `tvhc` в
//! двоичный файл `.hlp`, который читала программа. Здесь программа читает
//! **исходный текст** напрямую ([`HelpFile::parse`] / [`HelpFile::load`]),
//! а пример `tvhc` только проверяет файл и генерирует константы контекстов.
//!
//! Формат (как у `tvhc`):
//! ```text
//! ; комментарий
//! .topic FileOpen=3, OpenFile
//!   File|Open                       <- строка с пробелом в начале не переносится
//!   ---------
//! Этот пункт меню открывает {диалог выбора файла:FileDialog}.
//! Строки без пробела в начале объединяются в абзац и переносятся.
//! ```

use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::color::palettes;
use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Key, Mods};
use crate::geometry::{Point, Rect};
use crate::text::{char_width, str_width};
use crate::view::{default_handle_event, of, Ctx, DrawCtx, View, ViewId, ViewState};
use crate::widgets::scroller::Scroller;
use crate::widgets::window::{wf, Window};

/// Абзац справки (`TParagraph`).
#[derive(Clone, Debug, PartialEq)]
pub struct Paragraph {
    /// Текст (строки разделены `' '` для переносимых абзацев и `'\n'` для остальных).
    pub text: Vec<char>,
    /// Переносить ли по ширине окна.
    pub wrap: bool,
}

/// Перекрёстная ссылка (`TCrossRef`): смещение в тексте темы, длина, цель.
#[derive(Clone, Debug, PartialEq)]
pub struct CrossRef {
    pub offset: usize,
    pub length: usize,
    /// Имя темы (символ `tvhc`).
    pub target: String,
}

/// Тема справки (`THelpTopic`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HelpTopic {
    pub paragraphs: Vec<Paragraph>,
    pub cross_refs: Vec<CrossRef>,
}

/// Строка разметки темы для заданной ширины.
#[derive(Clone, Debug)]
struct LaidLine {
    /// Смещение начала строки в тексте темы.
    start: usize,
    /// Сколько символов текста покрывает строка (с пробелами/переводом строки).
    consumed: usize,
    text: String,
}

impl HelpTopic {
    /// Разбивка темы на строки для ширины `width` (`wrapText` / `getLine`).
    fn layout(&self, width: usize) -> Vec<LaidLine> {
        let width = width.max(1);
        let mut out = Vec::new();
        let mut base = 0;
        for p in &self.paragraphs {
            let t = &p.text;
            let mut pos = 0;
            while pos < t.len() {
                // Строка до перевода строки (`getLineAtOffset`).
                let end = t[pos..].iter().position(|&c| c == '\n').map(|i| pos + i).unwrap_or(t.len());
                let mut take = end - pos;
                if p.wrap {
                    // Сколько символов помещается в ширину.
                    let mut w = 0;
                    let mut fit = 0;
                    for &c in &t[pos..end] {
                        let cw = char_width(c).max(1);
                        if w + cw > width {
                            break;
                        }
                        w += cw;
                        fit += 1;
                    }
                    if fit < take {
                        // Не разрезаем слово, если можно перенести по пробелу.
                        let mut n = fit;
                        while n > 0 && t[pos + n] != ' ' {
                            n -= 1;
                        }
                        if n == 0 {
                            n = fit.max(1);
                        }
                        // Пробел после слова «съедаем», чтобы следующая строка с него не начиналась.
                        if pos + n < end && t[pos + n] == ' ' {
                            n += 1;
                        }
                        take = n;
                    }
                }
                let mut consumed = take;
                if pos + take == end && end < t.len() {
                    consumed += 1; // перевод строки
                }
                let text: String = t[pos..pos + take].iter().collect::<String>().trim_end().to_string();
                out.push(LaidLine { start: base + pos, consumed, text });
                pos += consumed;
            }
            base += t.len();
        }
        out
    }
}

/// Файл справки (`THelpFile`): темы и номера контекстов.
#[derive(Clone, Debug, Default)]
pub struct HelpFile {
    pub topics: Vec<HelpTopic>,
    /// Имя символа (в нижнем регистре) → номер темы.
    names: HashMap<String, usize>,
    /// Номер контекста → номер темы.
    contexts: HashMap<u16, usize>,
    /// Символы в порядке объявления: (имя, номер контекста).
    pub symbols: Vec<(String, u16)>,
    /// Предупреждения разбора (ссылки на несуществующие темы).
    pub warnings: Vec<String>,
}

/// Текст при отсутствии справки (`invalidContext`).
pub const NO_HELP: &str = "\n No help available in this context.";

impl HelpFile {
    /// Загрузить исходник справки из файла.
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let s = std::fs::read(path)?;
        Self::parse(&String::from_utf8_lossy(&s)).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Разобрать исходник справки в формате `tvhc`.
    pub fn parse(src: &str) -> Result<Self, String> {
        let lines: Vec<String> = src.lines().filter(|l| !l.starts_with(';')).map(|l| l.trim_end_matches('\r').to_string()).collect();
        let mut hf = HelpFile::default();
        // Номера контекстов: начиная с 2 (1 — `hcDragging`), как в `tvhc`.
        let mut counter: u32 = 2;
        let mut i = 0;
        // Пропускаем всё до первой темы.
        while i < lines.len() && !lines[i].starts_with('.') {
            i += 1;
        }
        while i < lines.len() {
            let header = lines[i].clone();
            i += 1;
            let rest = header.trim_start_matches('.').trim();
            let (kw, defs) = rest.split_at(rest.find(char::is_whitespace).unwrap_or(rest.len()));
            if !kw.eq_ignore_ascii_case("topic") {
                return Err(format!("TOPIC expected: {}", header));
            }
            let topic_index = hf.topics.len();
            for def in defs.split(',') {
                let def = def.trim();
                if def.is_empty() {
                    return Err(format!("Topic name expected: {}", header));
                }
                let (name, num) = match def.split_once('=') {
                    Some((n, v)) => (n.trim(), Some(v.trim().parse::<u32>().map_err(|_| format!("Bad number: {}", def))?)),
                    None => (def, None),
                };
                // Явный номер задаёт счётчик; без номера — следующий по порядку.
                if let Some(n) = num {
                    counter = n;
                }
                let ctx = counter as u16;
                counter += 1;
                if hf.names.insert(name.to_lowercase(), topic_index).is_some() {
                    return Err(format!("Redefinition of {}", name));
                }
                hf.contexts.insert(ctx, topic_index);
                hf.symbols.push((name.to_string(), ctx));
            }
            // Абзацы темы.
            let mut topic = HelpTopic::default();
            let mut offset = 0usize;
            loop {
                let (p, refs, next) = read_paragraph(&lines, i, offset);
                i = next;
                match p {
                    Some(p) => {
                        offset += p.text.len();
                        topic.cross_refs.extend(refs);
                        topic.paragraphs.push(p);
                    }
                    None => break,
                }
            }
            hf.topics.push(topic);
            // Следующая тема.
            while i < lines.len() && !lines[i].starts_with('.') {
                i += 1;
            }
        }
        // Проверка ссылок (в `tvhc` — предупреждение о неопределённых темах).
        for t in &hf.topics {
            for r in &t.cross_refs {
                if !hf.names.contains_key(&r.target.to_lowercase()) {
                    hf.warnings.push(format!("Unresolved forward reference \"{}\"", r.target));
                }
            }
        }
        Ok(hf)
    }

    /// Тема по номеру контекста (`getTopic`).
    pub fn topic_for_context(&self, ctx: u16) -> Option<&HelpTopic> {
        self.contexts.get(&ctx).map(|&i| &self.topics[i])
    }

    /// Тема по имени символа.
    pub fn topic_by_name(&self, name: &str) -> Option<&HelpTopic> {
        self.names.get(&name.to_lowercase()).map(|&i| &self.topics[i])
    }

    /// Номер темы по имени.
    fn index_by_name(&self, name: &str) -> Option<usize> {
        self.names.get(&name.to_lowercase()).copied()
    }

    /// Тема «нет справки» (`invalidContext`).
    pub fn invalid_topic() -> HelpTopic {
        HelpTopic { paragraphs: vec![Paragraph { text: NO_HELP.chars().collect(), wrap: false }], cross_refs: vec![] }
    }

    /// Константы контекстов в виде исходника Rust (замена `.h`-файла `tvhc`).
    pub fn context_constants(&self) -> String {
        let mut s = String::from("// Сгенерировано из исходника справки. Номера контекстов справки.\n");
        for (name, n) in &self.symbols {
            s.push_str(&format!("pub const HC_{}: u16 = {};\n", to_upper_snake(name), n));
        }
        s
    }
}

/// `FileOpen` → `FILE_OPEN`.
fn to_upper_snake(name: &str) -> String {
    let mut out = String::new();
    let mut prev_lower = false;
    for c in name.chars() {
        if c.is_uppercase() && prev_lower {
            out.push('_');
        }
        prev_lower = c.is_lowercase() || c.is_ascii_digit();
        if c.is_alphanumeric() {
            out.extend(c.to_uppercase());
        } else {
            out.push('_');
        }
    }
    out
}

/// Чтение одного абзаца (`readParagraph`). Возвращает абзац, его ссылки
/// (смещения считаются от начала темы) и номер следующей строки.
fn read_paragraph(lines: &[String], mut i: usize, offset: usize) -> (Option<Paragraph>, Vec<CrossRef>, usize) {
    #[derive(PartialEq)]
    enum State {
        Undefined,
        Wrapping,
        NotWrapping,
    }
    let mut state = State::Undefined;
    let mut buf: Vec<char> = Vec::new();
    let mut refs = Vec::new();
    // Пустые строки перед абзацем становятся пустыми строками текста.
    while i < lines.len() && lines[i].is_empty() {
        buf.push('\n');
        i += 1;
    }
    let is_end = |line: &str, st: &State| -> bool {
        line.is_empty()
            || line.starts_with('.')
            || line.starts_with('\x1A')
            || (line.starts_with(' ') && *st == State::Wrapping)
            || (!line.starts_with(' ') && *st == State::NotWrapping)
    };
    if i >= lines.len() || is_end(&lines[i], &state) {
        // Хвостовые пустые строки темы не нужны.
        return (None, refs, i);
    }
    while i < lines.len() && !is_end(&lines[i], &state) {
        if state == State::Undefined {
            state = if lines[i].starts_with(' ') { State::NotWrapping } else { State::Wrapping };
        }
        let base = offset + buf.len();
        let (text, mut r) = scan_cross_refs(&lines[i], base);
        refs.append(&mut r);
        buf.extend(text);
        buf.push(if state == State::Wrapping { ' ' } else { '\n' });
        i += 1;
    }
    (Some(Paragraph { text: buf, wrap: state == State::Wrapping }), refs, i)
}

/// Поиск ссылок `{текст[:тема]}` в строке (`scanForCrossRefs`).
/// `{{` вне ссылки — литерал `{`; внутри ссылки `::` и `}}` — литералы.
fn scan_cross_refs(line: &str, base: usize) -> (Vec<char>, Vec<CrossRef>) {
    let chars: Vec<char> = line.chars().collect();
    let mut out = Vec::new();
    let mut refs = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' {
            if i + 1 < chars.len() && chars[i + 1] == '{' {
                out.push('{');
                i += 2;
                continue;
            }
            // Ссылка до одиночной '}'.
            let mut text = Vec::new();
            let mut alias: Option<String> = None;
            let mut j = i + 1;
            let mut in_alias = false;
            let mut alias_buf = String::new();
            while j < chars.len() {
                let c = chars[j];
                if c == '}' {
                    if j + 1 < chars.len() && chars[j + 1] == '}' {
                        if in_alias {
                            alias_buf.push('}')
                        } else {
                            text.push('}')
                        }
                        j += 2;
                        continue;
                    }
                    break;
                }
                if c == ':' && !in_alias {
                    if j + 1 < chars.len() && chars[j + 1] == ':' {
                        text.push(':');
                        j += 2;
                        continue;
                    }
                    in_alias = true;
                    j += 1;
                    continue;
                }
                if in_alias {
                    alias_buf.push(c);
                } else {
                    text.push(c);
                }
                j += 1;
            }
            if in_alias {
                alias = Some(alias_buf.trim().to_string());
            }
            let target = alias.unwrap_or_else(|| text.iter().collect::<String>());
            refs.push(CrossRef { offset: base + out.len(), length: text.len(), target });
            out.extend(text);
            i = j + 1;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    (out, refs)
}

// ----------------------------------------------------------------------
// THelpViewer / THelpWindow
// ----------------------------------------------------------------------

/// Просмотр темы справки (`THelpViewer`): Tab/Shift+Tab — по ссылкам,
/// Enter или щелчок — переход, Esc — закрыть.
pub struct HelpViewer {
    pub vs: ViewState,
    pub scroller: Scroller,
    pub file: Rc<HelpFile>,
    topic: HelpTopic,
    /// Номер выбранной ссылки (с 1, как в оригинале; 0 — нет ссылок).
    selected: usize,
    lines: Vec<LaidLine>,
    laid_width: usize,
}

impl HelpViewer {
    pub fn new(bounds: Rect, h: Option<ViewId>, v: Option<ViewId>, file: Rc<HelpFile>, context: u16) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.options |= of::SELECTABLE;
        vs.grow_mode = crate::view::gf::GROW_HI_X | crate::view::gf::GROW_HI_Y;
        let topic = file.topic_for_context(context).cloned().unwrap_or_else(HelpFile::invalid_topic);
        let mut hv = HelpViewer { vs, scroller: Scroller::new(h, v), file, topic, selected: 1, lines: Vec::new(), laid_width: 0 };
        hv.relayout();
        hv
    }

    fn relayout(&mut self) {
        let w = self.vs.size.x.max(1) as usize;
        self.lines = self.topic.layout(w);
        self.laid_width = w;
        let longest = self.lines.iter().map(|l| str_width(&l.text)).max().unwrap_or(0) as i32;
        let size = self.vs.size;
        self.scroller.set_limit(longest, self.lines.len() as i32, size);
    }

    /// Позиция ссылки на экране темы: (строка, столбец).
    fn ref_pos(&self, r: &CrossRef) -> Option<Point> {
        for (y, l) in self.lines.iter().enumerate() {
            if r.offset >= l.start && r.offset < l.start + l.consumed.max(1) {
                return Some(Point::new((r.offset - l.start) as i32, y as i32));
            }
        }
        None
    }

    /// Перейти к теме (`switchToTopic`).
    pub fn switch_to(&mut self, name: &str) {
        self.topic = self.file.index_by_name(name).map(|i| self.file.topics[i].clone()).unwrap_or_else(HelpFile::invalid_topic);
        let size = self.vs.size;
        self.scroller.delta = Point::default();
        self.selected = 1;
        self.relayout();
        self.scroller.scroll_to(0, 0, size);
    }

    /// Прокрутить так, чтобы выбранная ссылка была видна (`makeSelectVisible`).
    fn make_select_visible(&mut self) {
        let r = match self.topic.cross_refs.get(self.selected.wrapping_sub(1)) {
            Some(r) => r.clone(),
            None => return,
        };
        if let Some(p) = self.ref_pos(&r) {
            let size = self.vs.size;
            let mut d = self.scroller.delta;
            if p.x < d.x {
                d.x = p.x;
            }
            if p.x > d.x + size.x {
                d.x = p.x - size.x;
            }
            if p.y < d.y {
                d.y = p.y;
            }
            if p.y >= d.y + size.y {
                d.y = p.y - size.y + 1;
            }
            self.scroller.scroll_to(d.x, d.y, size);
        }
    }

    /// Имя текущей темы-ссылки (для тестов).
    pub fn selected_target(&self) -> Option<&str> {
        self.topic.cross_refs.get(self.selected.wrapping_sub(1)).map(|r| r.target.as_str())
    }

    /// Строки текущей темы (для тестов и отладки).
    pub fn text_lines(&self) -> Vec<String> {
        self.lines.iter().map(|l| l.text.clone()).collect()
    }
}

impl View for HelpViewer {
    crate::impl_view_basics!();

    /// Порт `THelpViewer::draw`.
    fn draw(&self, dc: &mut DrawCtx) {
        let normal = dc.color(1);
        let keyword = dc.color(2);
        let sel_keyword = dc.color(3);
        let w = self.vs.size.x.max(0) as usize;
        let d = self.scroller.delta;
        for y in 0..self.vs.size.y {
            let li = (d.y + y) as usize;
            let mut b = DrawBuffer::new(w);
            b.move_char(0, ' ', normal, w);
            if let Some(l) = self.lines.get(li) {
                let s: String = l.text.chars().skip(d.x.max(0) as usize).collect();
                b.move_str(0, &s, normal);
                // Подсветка ссылок этой строки.
                for (k, r) in self.topic.cross_refs.iter().enumerate() {
                    if let Some(p) = self.ref_pos(r) {
                        if p.y as usize == li {
                            let attr = if k + 1 == self.selected && self.vs.is_focused() { sel_keyword } else { keyword };
                            let x0 = p.x - d.x;
                            for k2 in 0..r.length as i32 {
                                let x = x0 + k2;
                                if x >= 0 && (x as usize) < w {
                                    b.put_attribute(x as usize, attr);
                                }
                            }
                        }
                    }
                }
            }
            dc.write_line(0, y, self.vs.size.x, 1, &b);
        }
    }

    fn palette(&self) -> &'static [u8] {
        palettes::HELP_VIEWER
    }

    fn change_bounds(&mut self, bounds: Rect) {
        self.vs.set_bounds(bounds);
        self.relayout();
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        default_handle_event(&self.vs, ev, ctx);
        let size = self.vs.size;
        let focused = self.vs.is_focused();
        let n = self.topic.cross_refs.len();
        match ev {
            Event::KeyDown(k) if focused => match (k.key, k.mods) {
                (Key::Tab, Mods::NONE) => {
                    self.selected += 1;
                    if self.selected > n {
                        self.selected = 1;
                    }
                    self.make_select_visible();
                    ev.clear();
                }
                (Key::Tab, Mods::SHIFT) | (Key::BackTab, _) => {
                    self.selected = if self.selected <= 1 { n } else { self.selected - 1 };
                    self.make_select_visible();
                    ev.clear();
                }
                (Key::Enter, _) => {
                    if self.selected >= 1 && self.selected <= n {
                        let t = self.topic.cross_refs[self.selected - 1].target.clone();
                        self.switch_to(&t);
                    }
                    ev.clear();
                }
                (Key::Esc, _) => {
                    ctx.post_command(cm::CLOSE, Default::default());
                    ev.clear();
                }
                _ => {
                    self.scroller.handle(ev, size, focused);
                }
            },
            Event::MouseDown(m) => {
                let p = self.vs.make_local(m.pos) + self.scroller.delta;
                let hit = self
                    .topic
                    .cross_refs
                    .iter()
                    .position(|r| self.ref_pos(r).map(|q| q.y == p.y && p.x >= q.x && p.x < q.x + r.length as i32).unwrap_or(false));
                if let Some(k) = hit {
                    self.selected = k + 1;
                    let t = self.topic.cross_refs[k].target.clone();
                    self.switch_to(&t);
                }
                ev.clear();
            }
            _ => {
                self.scroller.handle(ev, size, false);
            }
        }
        if self.laid_width != self.vs.size.x.max(1) as usize {
            self.relayout();
        }
        self.scroller.sync(size, ctx);
    }
}

/// Заголовок окна справки (`helpWinTitle`).
pub const HELP_WIN_TITLE: &str = "Help";

/// Окно справки (`THelpWindow`), 50×18 по центру.
pub struct HelpWindow {
    pub win: Window,
    pub viewer: ViewId,
}

impl HelpWindow {
    pub fn new(file: Rc<HelpFile>, context: u16) -> Self {
        let mut win = Window::new(Rect::new(0, 0, 50, 18), HELP_WIN_TITLE, 0);
        win.group.vs.options |= of::CENTERED;
        win.palette = palettes::HELP_WINDOW;
        win.flags = wf::MOVE | wf::GROW | wf::CLOSE | wf::ZOOM;
        let h = win.standard_scroll_bar(false);
        let v = win.standard_scroll_bar(true);
        let r = win.group.vs.extent().grown(-2, -1);
        let viewer = win.add(HelpViewer::new(r, Some(h), Some(v), file, context));
        HelpWindow { win, viewer }
    }

    pub fn viewer(&self) -> &HelpViewer {
        self.win.group.find::<HelpViewer>(self.viewer).unwrap()
    }
}

impl View for HelpWindow {
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

    fn size_limits(&self, owner_size: Point) -> (Point, Point) {
        (crate::widgets::window::MIN_WIN_SIZE, owner_size)
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        // В модальном режиме закрытие окна завершает его (`THelpViewer`, cmClose).
        let modal = self.win.group.vs.state & crate::view::sf::MODAL != 0;
        if modal && matches!(ev.command(), Some(cm::CLOSE) | Some(cm::CANCEL)) {
            ctx.end_modal(cm::CLOSE);
            ev.clear();
            return;
        }
        self.win.handle_window_event(ev, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "; demo
.topic NoContext=0
 Demo help
 ---------
Welcome to the {file viewer:Viewer} and the {Puzzle}. This paragraph is
long enough to be wrapped.

  Up    Move up
  Down  Move down

.topic Viewer=5, FileView
 File viewer
Use {{braces} and {a::b:Puzzle}.

.topic Puzzle
 Puzzle
Back to {NoContext}.
";

    #[test]
    fn parse_topics() {
        let hf = HelpFile::parse(SRC).unwrap();
        assert_eq!(hf.topics.len(), 3);
        assert_eq!(hf.symbols, vec![("NoContext".into(), 0), ("Viewer".into(), 5), ("FileView".into(), 6), ("Puzzle".into(), 7)]);
        assert!(hf.warnings.is_empty(), "{:?}", hf.warnings);
        let t = hf.topic_for_context(0).unwrap();
        assert_eq!(t.cross_refs.len(), 2);
        assert_eq!(t.cross_refs[0].target, "Viewer");
        assert_eq!(t.cross_refs[1].target, "Puzzle");
        assert!(hf.topic_for_context(6).is_some());
        assert!(hf.context_constants().contains("pub const HC_FILE_VIEW: u16 = 6;"));
    }

    #[test]
    fn escapes() {
        let hf = HelpFile::parse(SRC).unwrap();
        let t = hf.topic_by_name("viewer").unwrap();
        let lines: Vec<String> = t.layout(60).into_iter().map(|l| l.text).collect();
        assert_eq!(lines, vec![" File viewer", "Use {braces} and a:b."]);
        assert_eq!(t.cross_refs[0].target, "Puzzle");
        assert_eq!(t.cross_refs[0].length, 3);
    }

    #[test]
    fn wrapping_and_refs() {
        let hf = Rc::new(HelpFile::parse(SRC).unwrap());
        let v = HelpViewer::new(Rect::new(0, 0, 30, 10), None, None, hf, 0);
        let lines = v.text_lines();
        assert_eq!(lines[0], " Demo help");
        assert_eq!(lines[2], "Welcome to the file viewer and");
        assert_eq!(lines[3], "the Puzzle. This paragraph is");
        // Ссылка «file viewer» — на третьей строке, с 15-го столбца.
        let r = v.topic.cross_refs[0].clone();
        assert_eq!(v.ref_pos(&r), Some(Point::new(15, 2)));
        assert!(lines.iter().any(|l| l == "  Up    Move up"));
    }
}
