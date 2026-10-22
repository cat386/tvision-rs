//! Дерево (`TOutlineViewer`, `TOutline`, `TNode` из `outline.h` / `toutline.cpp`).
//!
//! Узлы можно раскрывать и сворачивать: `+` / `-` / `*` (раскрыть всё),
//! щелчком по «ветке» слева от текста. Enter или двойной щелчок «выбирают»
//! узел — владельцу рассылается `cm::LIST_ITEM_SELECTED` с id дерева.

use crate::color::palettes;
use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Info, Key, Mods};
use crate::geometry::Rect;
use crate::text::str_width;
use crate::view::{default_handle_event, gf, of, Ctx, DrawCtx, View, ViewId, ViewState};
use crate::widgets::scroller::Scroller;

/// Флаги отрисовки узла (`ovXXX`).
pub mod ov {
    /// Узел раскрыт (или у него нет детей).
    pub const EXPANDED: u16 = 0x01;
    /// У узла есть видимые дети.
    pub const CHILDREN: u16 = 0x02;
    /// Узел — последний среди братьев.
    pub const LAST: u16 = 0x04;
}

/// Узел дерева (`TNode`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Node {
    pub text: String,
    pub children: Vec<Node>,
    pub expanded: bool,
}

impl Node {
    /// Лист.
    pub fn new(text: &str) -> Self {
        Node { text: text.into(), children: Vec::new(), expanded: true }
    }

    /// Узел с детьми (`TNode(text, children, next, initialState)`).
    pub fn with_children(text: &str, children: Vec<Node>, expanded: bool) -> Self {
        Node { text: text.into(), children, expanded }
    }
}

/// Видимая строка дерева: путь к узлу и параметры для рисования ветки.
#[derive(Clone, Debug)]
struct Row {
    /// Индексы детей от корня до узла.
    path: Vec<usize>,
    level: usize,
    /// Биты уровней, на которых нужна вертикальная линия.
    lines: u64,
    flags: u16,
}

/// Символы ветвей (`graphChars`): заполнитель, вертикаль, «├», «└»,
/// горизонталь, развилка, «свёрнуто», «раскрыто».
const GRAPH_CHARS: [char; 8] = [' ', '│', '├', '└', '─', '─', '+', '─'];

/// Строит графику ветви (`createGraph`).
pub fn create_graph(level: usize, mut lines: u64, flags: u16, lev_width: usize, end_width: usize, chars: &[char; 8]) -> String {
    let mut g = String::new();
    for _ in 0..level {
        g.push(if lines & 1 != 0 { chars[1] } else { chars[0] });
        for _ in 1..lev_width {
            g.push(chars[0]);
        }
        lines >>= 1;
    }
    let expanded = flags & ov::EXPANDED != 0;
    let children = flags & ov::CHILDREN != 0;
    let last = flags & ov::LAST != 0;
    let mut ew = end_width as isize - 1;
    if ew > 0 {
        g.push(if last { chars[3] } else { chars[2] });
        ew -= 1;
        if ew > 0 {
            ew -= 1;
            for _ in 0..ew.max(0) {
                g.push(chars[4]);
            }
            g.push(if children { chars[5] } else { chars[4] });
        }
        g.push(if expanded { chars[7] } else { chars[6] });
    }
    g
}

/// Дерево (`TOutline`).
pub struct Outline {
    pub vs: ViewState,
    pub scroller: Scroller,
    /// Узлы верхнего уровня (в C++ — корень и его «соседи» `next`).
    pub roots: Vec<Node>,
    /// Строка с фокусом (`foc`).
    pub foc: usize,
    rows: Vec<Row>,
    /// Состояние мыши: сколько раз сдвинули (для отличия щелчка от протяжки).
    dragged: u8,
    auto_count: u8,
}

impl Outline {
    /// Аналог `TOutline(bounds, hScrollBar, vScrollBar, root)`.
    pub fn new(bounds: Rect, h_scroll: Option<ViewId>, v_scroll: Option<ViewId>, roots: Vec<Node>) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
        vs.options |= of::SELECTABLE | of::FIRST_CLICK;
        let mut o = Outline { vs, scroller: Scroller::new(h_scroll, v_scroll), roots, foc: 0, rows: Vec::new(), dragged: 0, auto_count: 0 };
        o.update();
        o
    }

    /// Узел по пути.
    fn node(&self, path: &[usize]) -> &Node {
        let mut n = &self.roots[path[0]];
        for &i in &path[1..] {
            n = &n.children[i];
        }
        n
    }

    fn node_mut(&mut self, path: &[usize]) -> &mut Node {
        let mut n = &mut self.roots[path[0]];
        for &i in &path[1..] {
            n = &mut n.children[i];
        }
        n
    }

    /// Пересобрать видимые строки (обход дерева `traverseTree`).
    fn collect(nodes: &[Node], prefix: &mut Vec<usize>, level: usize, lines: u64, out: &mut Vec<Row>) {
        for (i, n) in nodes.iter().enumerate() {
            let last = i + 1 == nodes.len();
            let has = !n.children.is_empty();
            let mut flags = 0;
            if last {
                flags |= ov::LAST;
            }
            if has && n.expanded {
                flags |= ov::CHILDREN;
            }
            if !has || n.expanded {
                flags |= ov::EXPANDED;
            }
            prefix.push(i);
            out.push(Row { path: prefix.clone(), level, lines, flags });
            if has && n.expanded {
                let child_lines = if last { lines } else { lines | (1 << level) };
                Self::collect(&n.children, prefix, level + 1, child_lines, out);
            }
            prefix.pop();
        }
    }

    /// Графика ветви строки (`getGraph`): ширина уровня 3.
    fn graph(&self, r: &Row) -> String {
        create_graph(r.level, r.lines, r.flags, 3, 3, &GRAPH_CHARS)
    }

    /// Пересчитать строки и пределы прокрутки (`update`).
    pub fn update(&mut self) {
        let mut rows = Vec::new();
        Self::collect(&self.roots, &mut Vec::new(), 0, 0, &mut rows);
        let max_x = rows.iter().map(|r| str_width(&self.graph(r)) + str_width(&self.node(&r.path).text)).max().unwrap_or(0);
        self.rows = rows;
        let size = self.vs.size;
        self.scroller.set_limit(max_x as i32, self.rows.len() as i32, size);
        let f = self.foc as i32;
        self.adjust_focus(f);
    }

    /// Число видимых строк.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Узел в строке `i` (`getNode`).
    pub fn node_at(&self, i: usize) -> Option<&Node> {
        self.rows.get(i).map(|r| self.node(&r.path))
    }

    /// Узел с фокусом.
    pub fn focused_node(&self) -> Option<&Node> {
        self.node_at(self.foc)
    }

    /// Путь (индексы от корня) к узлу с фокусом.
    pub fn focused_path(&self) -> Option<Vec<usize>> {
        self.rows.get(self.foc).map(|r| r.path.clone())
    }

    /// Раскрыть/свернуть узел в строке `i` (`adjust`).
    pub fn set_expanded(&mut self, i: usize, expand: bool) {
        if let Some(p) = self.rows.get(i).map(|r| r.path.clone()) {
            self.node_mut(&p).expanded = expand;
            self.update();
        }
    }

    /// Раскрыть узел и всех потомков (`expandAll`).
    pub fn expand_all(&mut self, i: usize) {
        fn rec(n: &mut Node) {
            if !n.children.is_empty() {
                n.expanded = true;
                n.children.iter_mut().for_each(rec);
            }
        }
        if let Some(p) = self.rows.get(i).map(|r| r.path.clone()) {
            rec(self.node_mut(&p));
            self.update();
        }
    }

    /// Держать фокус в пределах и на экране (`adjustFocus`).
    fn adjust_focus(&mut self, new_focus: i32) {
        let n = self.rows.len() as i32;
        let nf = new_focus.clamp(0, (n - 1).max(0));
        self.foc = nf as usize;
        let size = self.vs.size;
        let d = self.scroller.delta;
        if nf < d.y {
            self.scroller.scroll_to(d.x, nf, size);
        } else if nf - size.y >= d.y {
            self.scroller.scroll_to(d.x, nf - size.y + 1, size);
        }
    }

    /// Узел «выбран» (`selected`).
    fn selected(&mut self, ctx: &mut Ctx) {
        ctx.message(cm::LIST_ITEM_SELECTED, Info::Id(self.vs.id));
    }
}

impl View for Outline {
    crate::impl_view_basics!();

    /// Порт `TOutlineViewer::draw` / `drawTree`.
    fn draw(&self, dc: &mut DrawCtx) {
        let w = self.vs.size.x.max(0) as usize;
        let d = self.scroller.delta;
        let nrm = dc.get_color(0x0401);
        for y in 0..self.vs.size.y {
            let pos = (d.y + y) as usize;
            let mut b = DrawBuffer::new(w);
            match self.rows.get(pos) {
                Some(r) => {
                    let color = if pos == self.foc && self.vs.is_focused() {
                        dc.get_color(0x0202)
                    } else if pos == self.foc {
                        dc.get_color(0x0303)
                    } else {
                        nrm
                    };
                    let lo = (color & 0xFF) as u8;
                    b.move_char(0, ' ', lo, w);
                    let graph = self.graph(r);
                    // Горизонтальный сдвиг: пропускаем `delta.x` столбцов графики и текста.
                    let full: String = graph.clone() + &self.node(&r.path).text;
                    let skip = d.x.max(0) as usize;
                    let gw = str_width(&graph);
                    let visible: String = full.chars().skip(skip).collect();
                    // Графика — основным цветом, текст свёрнутого узла — цветом подсветки.
                    let text_attr = if r.flags & ov::EXPANDED != 0 { lo } else { (color >> 8) as u8 };
                    let g_vis = gw.saturating_sub(skip);
                    let (gpart, tpart): (String, String) = (visible.chars().take(g_vis).collect(), visible.chars().skip(g_vis).collect());
                    let x = b.move_str(0, &gpart, lo);
                    b.move_str(x, &tpart, text_attr);
                }
                None => b.move_char(0, ' ', (nrm & 0xFF) as u8, w),
            }
            dc.write_line(0, y, self.vs.size.x, 1, &b);
        }
    }

    fn palette(&self) -> &'static [u8] {
        palettes::OUTLINE_VIEWER
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        default_handle_event(&self.vs, ev, ctx);
        let id = self.vs.id;
        let size = self.vs.size;
        // Колесо и полосы прокрутки — через `Scroller`.
        if matches!(ev, Event::MouseWheel(_) | Event::Broadcast(_)) {
            self.scroller.handle(ev, size, false);
        }
        match ev {
            Event::MouseDown(m) => {
                let p = self.vs.make_local(m.pos);
                let i = self.scroller.delta.y + p.y;
                if (i as usize) < self.rows.len() {
                    self.adjust_focus(i);
                }
                if m.double {
                    self.selected(ctx);
                    self.dragged = 2;
                } else {
                    self.dragged = 1;
                    // Щелчок по ветке — раскрыть/свернуть.
                    if let Some(r) = self.rows.get(self.foc) {
                        let gw = str_width(&self.graph(r)) as i32;
                        if p.x + self.scroller.delta.x < gw && (i as usize) == self.foc {
                            let exp = self.node(&r.path.clone()).expanded;
                            let f = self.foc;
                            self.set_expanded(f, !exp);
                            self.dragged = 2;
                        }
                    }
                }
                self.auto_count = 0;
                ctx.grab_mouse(id);
                ev.clear();
            }
            Event::MouseMove(_) | Event::MouseAuto(_) if ctx.mouse_grabbed_by(id) => {
                let is_auto = matches!(ev, Event::MouseAuto(_));
                let m = *ev.mouse().unwrap();
                self.dragged = 2;
                let p = self.vs.make_local(m.pos);
                let mut nf = self.foc as i32;
                if self.vs.extent().contains(p) {
                    let i = self.scroller.delta.y + p.y;
                    if (i as usize) < self.rows.len() {
                        nf = i;
                    }
                } else if is_auto {
                    // Автопрокрутка через каждые 3 повтора (`mouseAutoToSkip`).
                    self.auto_count += 1;
                    if self.auto_count >= 3 {
                        self.auto_count = 0;
                        if p.y < 0 {
                            nf -= 1;
                        } else if p.y >= size.y {
                            nf += 1;
                        }
                    }
                }
                self.adjust_focus(nf);
                ev.clear();
            }
            Event::MouseUp(_) if ctx.mouse_grabbed_by(id) => {
                ctx.release_mouse();
                ev.clear();
            }
            Event::KeyDown(k) if self.vs.is_focused() => {
                let k = *k;
                let mut nf = self.foc as i32;
                let page = (size.y - 1).max(1);
                match (k.key, k.mods) {
                    (Key::Up | Key::Left, Mods::NONE) => nf -= 1,
                    (Key::Down | Key::Right, Mods::NONE) => nf += 1,
                    (Key::PgDn, Mods::NONE) => nf += page,
                    (Key::PgUp, Mods::NONE) => nf -= page,
                    (Key::Home, Mods::NONE) => nf = self.scroller.delta.y,
                    (Key::End, Mods::NONE) => nf = self.scroller.delta.y + size.y - 1,
                    (Key::PgUp, Mods::CTRL) => nf = 0,
                    (Key::PgDn, Mods::CTRL) => nf = self.rows.len() as i32 - 1,
                    (Key::Enter, _) => self.selected(ctx),
                    (Key::Char('+'), _) => self.set_expanded(self.foc, true),
                    (Key::Char('-'), _) => self.set_expanded(self.foc, false),
                    (Key::Char('*'), _) => self.expand_all(self.foc),
                    _ => {
                        self.scroller.sync(size, ctx);
                        return;
                    }
                }
                self.adjust_focus(nf);
                ev.clear();
            }
            _ => {}
        }
        self.scroller.sync(size, ctx);
    }

    fn change_bounds(&mut self, bounds: Rect) {
        self.vs.set_bounds(bounds);
        self.update();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph() {
        assert_eq!(create_graph(0, 0, ov::EXPANDED, 3, 3, &GRAPH_CHARS), "├──");
        assert_eq!(create_graph(0, 0, ov::LAST, 3, 3, &GRAPH_CHARS), "└─+");
        assert_eq!(create_graph(1, 1, ov::LAST | ov::EXPANDED, 3, 3, &GRAPH_CHARS), "│  └──");
    }

    #[test]
    fn rows_and_expand() {
        let tree = vec![
            Node::with_children("src", vec![Node::new("main.rs"), Node::with_children("widgets", vec![Node::new("a.rs")], false)], true),
            Node::new("Cargo.toml"),
        ];
        let mut o = Outline::new(Rect::new(0, 0, 30, 10), None, None, tree);
        assert_eq!(o.row_count(), 4); // src, main.rs, widgets (свёрнут), Cargo.toml
        o.expand_all(2);
        assert_eq!(o.row_count(), 5);
        assert_eq!(o.node_at(3).unwrap().text, "a.rs");
        o.set_expanded(0, false);
        assert_eq!(o.row_count(), 2);
    }
}
