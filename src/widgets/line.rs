//! Линия: горизонтальная или вертикальная черта, одинарная или двойная.
//!
//! Линии соединяются сами: там, где они пересекаются, сходятся углом или
//! упираются друг в друга или в рамку окна, рисуется нужный символ —
//! `┼ ├ ┤ ┬ ┴ ┌ ┐ └ ┘`, двойные `╬ ╠ ╣ ╦ ╩ ╔ ╗ ╚ ╝` и смешанные
//! (`╟ ╢ ╤ ╧ ╪ ╫ ╒ ╓`…):
//!
//! ```text
//! ╔═════ Окно ═════╗
//! ║ Имя:  ...      ║
//! ╟───────┬────────╢    ← линия «до рамки» и вертикальная линия
//! ║ слева │ справа ║
//! ╚═══════╧════════╝
//! ```
//!
//! Соединяет линии группа, в которой они лежат (окно, диалог):
//! [`crate::group::Group::draw_children`] рисует линии последними и
//! смотрит, что уже нарисовано вокруг (рамка, другие линии). Отдельно от
//! группы линия рисуется простой чертой.

use std::collections::HashMap;

use crate::color::{palettes, Attr};
use crate::geometry::{Point, Rect};
use crate::layout::{Policy, SizePolicy};
use crate::view::{DrawCtx, View, ViewState};

/// Тип линии.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LineStyle {
    #[default]
    Single,
    Double,
}

impl LineStyle {
    /// Вес для таблицы символов: 1 — одинарная, 2 — двойная.
    fn weight(self) -> u8 {
        match self {
            LineStyle::Single => 1,
            LineStyle::Double => 2,
        }
    }
}

/// Линия.
pub struct Line {
    pub vs: ViewState,
    /// Вертикальная (иначе горизонтальная).
    pub vertical: bool,
    pub style: LineStyle,
    /// Тянуть линию до рамки окна по всей ширине (высоте) группы, где бы
    /// она ни стояла, — например, в раскладке с отступами. Концы
    /// соединяются с рамкой (`╟───╢`).
    pub to_frame: bool,
}

impl Line {
    /// Линия в прямоугольнике: вертикальная, если он шириной в 1 клетку и
    /// выше, чем шире.
    pub fn new(bounds: Rect) -> Line {
        let vertical = bounds.width() == 1 && bounds.height() > 1;
        Line { vs: ViewState::new(bounds), vertical, style: LineStyle::Single, to_frame: false }
    }

    /// Горизонтальная линия.
    pub fn horizontal(bounds: Rect) -> Line {
        Line { vertical: false, ..Line::new(bounds) }
    }

    /// Вертикальная линия.
    pub fn vertical(bounds: Rect) -> Line {
        Line { vertical: true, ..Line::new(bounds) }
    }

    /// Двойная линия.
    pub fn double(mut self) -> Line {
        self.style = LineStyle::Double;
        self
    }

    /// До рамки окна (см. [`Line::to_frame`]).
    pub fn to_frame(mut self) -> Line {
        self.to_frame = true;
        self
    }
}

impl View for Line {
    crate::impl_view_basics!();

    /// Простая черта; соединения дорисовывает группа ([`join_lines`]).
    fn draw(&self, dc: &mut DrawCtx) {
        let attr = dc.color(1);
        let ch =
            glyph(if self.vertical { [self.style.weight(), self.style.weight(), 0, 0] } else { [0, 0, self.style.weight(), self.style.weight()] });
        dc.fill(self.vs.extent(), ch, attr);
    }

    fn palette(&self) -> &'static [u8] {
        palettes::LINE
    }

    fn size_hint(&self) -> Option<Point> {
        Some(Point::new(1, 1))
    }

    /// Горизонтальная тянется по ширине, вертикальная — по высоте; толщина —
    /// одна клетка.
    fn size_policy(&self) -> SizePolicy {
        if self.vertical {
            SizePolicy::new(Policy::Fixed, Policy::Expanding)
        } else {
            SizePolicy::new(Policy::Expanding, Policy::Fixed)
        }
    }

    fn as_line(&self) -> Option<&Line> {
        Some(self)
    }
}

// ----------------------------------------------------------------------
// Символы рамок
// ----------------------------------------------------------------------

/// Направления соединения в клетке: вверх, вниз, влево, вправо; 0 — нет,
/// 1 — одинарная, 2 — двойная.
pub type Joints = [u8; 4];

const UP: usize = 0;
const DOWN: usize = 1;
const LEFT: usize = 2;
const RIGHT: usize = 3;

/// Все символы рамок кодовой страницы 437 и их соединения.
const GLYPHS: [(char, Joints); 40] = [
    ('─', [0, 0, 1, 1]),
    ('│', [1, 1, 0, 0]),
    ('┌', [0, 1, 0, 1]),
    ('┐', [0, 1, 1, 0]),
    ('└', [1, 0, 0, 1]),
    ('┘', [1, 0, 1, 0]),
    ('├', [1, 1, 0, 1]),
    ('┤', [1, 1, 1, 0]),
    ('┬', [0, 1, 1, 1]),
    ('┴', [1, 0, 1, 1]),
    ('┼', [1, 1, 1, 1]),
    ('═', [0, 0, 2, 2]),
    ('║', [2, 2, 0, 0]),
    ('╔', [0, 2, 0, 2]),
    ('╗', [0, 2, 2, 0]),
    ('╚', [2, 0, 0, 2]),
    ('╝', [2, 0, 2, 0]),
    ('╠', [2, 2, 0, 2]),
    ('╣', [2, 2, 2, 0]),
    ('╦', [0, 2, 2, 2]),
    ('╩', [2, 0, 2, 2]),
    ('╬', [2, 2, 2, 2]),
    ('╒', [0, 1, 0, 2]),
    ('╓', [0, 2, 0, 1]),
    ('╕', [0, 1, 2, 0]),
    ('╖', [0, 2, 1, 0]),
    ('╘', [1, 0, 0, 2]),
    ('╙', [2, 0, 0, 1]),
    ('╛', [1, 0, 2, 0]),
    ('╜', [2, 0, 1, 0]),
    ('╞', [1, 1, 0, 2]),
    ('╟', [2, 2, 0, 1]),
    ('╡', [1, 1, 2, 0]),
    ('╢', [2, 2, 1, 0]),
    ('╤', [0, 1, 2, 2]),
    ('╥', [0, 2, 1, 1]),
    ('╧', [1, 0, 2, 2]),
    ('╨', [2, 0, 1, 1]),
    ('╪', [1, 1, 2, 2]),
    ('╫', [2, 2, 1, 1]),
];

/// Соединения символа рамки (`None` — не символ рамки).
pub fn joints(ch: char) -> Option<Joints> {
    GLYPHS.iter().find(|(c, _)| *c == ch).map(|(_, j)| *j)
}

/// Символ рамки для соединений. Сочетания, которых нет среди символов
/// (одинарная вверх и двойная вниз, конец линии с одной стороны),
/// заменяются ближайшими.
pub fn glyph(j: Joints) -> char {
    if let Some((c, _)) = GLYPHS.iter().find(|(_, g)| *g == j) {
        return *c;
    }
    let mut j = j;
    // Конец линии: черта продолжается в обе стороны своей оси.
    if j[LEFT] == 0 && j[RIGHT] == 0 {
        let v = j[UP].max(j[DOWN]);
        j[UP] = v;
        j[DOWN] = v;
    }
    if j[UP] == 0 && j[DOWN] == 0 {
        let h = j[LEFT].max(j[RIGHT]);
        j[LEFT] = h;
        j[RIGHT] = h;
    }
    if j == [0, 0, 0, 0] {
        return ' ';
    }
    if let Some((c, _)) = GLYPHS.iter().find(|(_, g)| *g == j) {
        return *c;
    }
    // Разные линии на одной оси — по более толстой.
    for (a, b) in [(UP, DOWN), (LEFT, RIGHT)] {
        if j[a] != 0 && j[b] != 0 {
            let m = j[a].max(j[b]);
            j[a] = m;
            j[b] = m;
        }
    }
    if let Some((c, _)) = GLYPHS.iter().find(|(_, g)| *g == j) {
        return *c;
    }
    // Остались одиночные смешанные концы — всё одной толщины.
    let m = *j.iter().max().unwrap();
    glyph(j.map(|v| if v == 0 { 0 } else { m }))
}

fn merge(a: &mut Joints, b: Joints) {
    for i in 0..4 {
        a[i] = a[i].max(b[i]);
    }
}

// ----------------------------------------------------------------------
// Соединение линий в группе
// ----------------------------------------------------------------------

/// Линия для [`join_lines`]: клетки в координатах группы, тип и цвет.
pub struct Segment {
    pub start: Point,
    pub len: i32,
    pub vertical: bool,
    pub style: LineStyle,
    pub attr: Attr,
}

impl Segment {
    fn cell(&self, k: i32) -> Point {
        if self.vertical {
            Point::new(self.start.x, self.start.y + k)
        } else {
            Point::new(self.start.x + k, self.start.y)
        }
    }
}

/// Отрезок для линии `l`, лежащей в группе размером `size`, с учётом
/// «до рамки»: клетки `x = 0` и `x = size.x - 1` (рамка, если она
/// нарисована там символами рамки) пропускаются.
pub fn segment_of(l: &Line, dc: &DrawCtx, attr: Attr) -> Option<Segment> {
    let b = l.vs.bounds();
    if b.is_empty() {
        return None;
    }
    let (mut start, mut len) = if l.vertical { (Point::new(b.a.x, b.a.y), b.height()) } else { (Point::new(b.a.x, b.a.y), b.width()) };
    if l.to_frame {
        let size = dc.size;
        let is_frame = |p: Point| dc.cell_at(p.x, p.y).and_then(|c| joints(c.ch)).is_some();
        if l.vertical {
            let top = if is_frame(Point::new(b.a.x, 0)) { 1 } else { 0 };
            let bottom = if is_frame(Point::new(b.a.x, size.y - 1)) { size.y - 2 } else { size.y - 1 };
            start = Point::new(b.a.x, top);
            len = bottom - top + 1;
        } else {
            let left = if is_frame(Point::new(0, b.a.y)) { 1 } else { 0 };
            let right = if is_frame(Point::new(size.x - 1, b.a.y)) { size.x - 2 } else { size.x - 1 };
            start = Point::new(left, b.a.y);
            len = right - left + 1;
        }
    }
    (len > 0).then_some(Segment { start, len, vertical: l.vertical, style: l.style, attr })
}

/// Нарисовать линии с соединениями (координаты — в `dc`). Вызывается после
/// того, как нарисовано всё остальное: символы рамок, которые уже есть на
/// экране (рамка окна, линии вложенных групп), тоже соединяются.
///
/// * пересечение и общие клетки — объединение соединений (`┼`, `├`);
/// * конец линии вплотную к другой линии или к символу рамки поперёк —
///   соединение (`╟`, `┬`); вплотную к линии вдоль — продолжение;
/// * свободный конец — просто черта.
pub fn join_lines(dc: &mut DrawCtx, segs: &[Segment]) {
    // Соединения клеток линий и их цвета.
    let mut cells: HashMap<(i32, i32), (Joints, Attr)> = HashMap::new();
    // Клетки чужих символов рамок, которые надо дорисовать.
    let mut others: HashMap<(i32, i32), Joints> = HashMap::new();

    // Направления «назад» и «вперёд» вдоль линии.
    let back = |vertical: bool| -> (usize, usize) {
        if vertical {
            (UP, DOWN)
        } else {
            (LEFT, RIGHT)
        }
    };
    for s in segs {
        let w = s.style.weight();
        let (bk, fw) = back(s.vertical);
        for k in 0..s.len {
            let p = s.cell(k);
            let mut j = [0; 4];
            if k > 0 {
                j[bk] = w;
            }
            if k < s.len - 1 {
                j[fw] = w;
            }
            // Под линией уже символ рамки (линия поверх рамки) — соединить.
            let mut attr = s.attr;
            if let Some(c) = dc.cell_at(p.x, p.y) {
                if let Some(g) = joints(c.ch) {
                    merge(&mut j, g);
                    attr = c.attr;
                }
            }
            let e = cells.entry((p.x, p.y)).or_insert(([0; 4], attr));
            merge(&mut e.0, j);
            e.1 = attr;
        }
    }

    // Концы: что за ними.
    let occupied: Vec<(i32, i32)> = cells.keys().copied().collect();
    for s in segs {
        let w = s.style.weight();
        let (bk, fw) = back(s.vertical);
        for (k, out, dir_out, dir_in) in [(0, -1, bk, fw), (s.len - 1, s.len, fw, bk)] {
            let end = s.cell(k);
            let beyond = s.cell(out);
            let key = (beyond.x, beyond.y);
            if occupied.contains(&key) {
                // Другая линия.
                if let Some(e) = cells.get_mut(&key) {
                    e.0[dir_in] = e.0[dir_in].max(w);
                }
                if let Some(e) = cells.get_mut(&(end.x, end.y)) {
                    e.0[dir_out] = e.0[dir_out].max(w);
                }
            } else if let Some(c) = dc.cell_at(beyond.x, beyond.y) {
                // Чужой символ рамки поперёк линии (рамка окна).
                if let Some(g) = joints(c.ch) {
                    let across = if s.vertical { g[LEFT] != 0 || g[RIGHT] != 0 } else { g[UP] != 0 || g[DOWN] != 0 };
                    if across {
                        let e = others.entry(key).or_insert(g);
                        e[dir_in] = e[dir_in].max(w);
                        if let Some(e) = cells.get_mut(&(end.x, end.y)) {
                            e.0[dir_out] = e.0[dir_out].max(w);
                        }
                    }
                }
            }
        }
    }

    for ((x, y), j) in others {
        if let Some(c) = dc.cell_at(x, y) {
            dc.write_char(x, y, glyph(j), c.attr, 1);
        }
    }
    for ((x, y), (j, attr)) in cells {
        dc.write_char(x, y, glyph(j), attr, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::HeadlessBackend;
    use crate::widgets::{Dialog, Window};
    use crate::Program;

    #[test]
    fn glyph_table() {
        for (c, j) in GLYPHS {
            assert_eq!(glyph(j), c);
            assert_eq!(joints(c), Some(j));
        }
        assert_eq!(glyph([0, 0, 0, 1]), '─', "свободный конец");
        assert_eq!(glyph([2, 1, 0, 0]), '║', "разная толщина на оси — по толстой");
        assert_eq!(glyph([2, 1, 0, 1]), '╟');
        assert_eq!(glyph([0, 0, 0, 0]), ' ');
    }

    /// Окно с линиями → текст его области на экране.
    fn render<V: View>(w: V) -> Vec<String> {
        let mut app = Program::new(Box::new(HeadlessBackend::new(40, 14)), None, None);
        let size = w.vs().size;
        app.insert_window(w);
        app.redraw();
        let sc = app.screen();
        (0..size.y).map(|y| (0..size.x).map(|x| sc.get(x as usize, y as usize).ch).collect()).collect()
    }

    #[test]
    fn joins_with_frame_and_each_other() {
        // 20 × 7, без заголовка и кнопок, чтобы рамка была видна целиком.
        let mut w = Window::new(Rect::new(0, 0, 20, 7), "", 0);
        w.flags = 0;
        w.add(Line::horizontal(Rect::new(1, 3, 19, 4)));
        w.add(Line::vertical(Rect::new(8, 1, 9, 6)));
        w.add(Line::vertical(Rect::new(13, 4, 14, 6)).double());
        let s = render(w);
        assert_eq!(
            s,
            vec![
                "╔═══════╤══════════╗",
                "║       │          ║",
                "║       │          ║",
                "╟───────┼────╥─────╢",
                "║       │    ║     ║",
                "║       │    ║     ║",
                "╚═══════╧════╩═════╝",
            ],
            "\n{}",
            s.join("\n")
        );
    }

    #[test]
    fn corners_and_free_ends() {
        let mut w = Window::new(Rect::new(0, 0, 20, 7), "", 0);
        w.flags = 0;
        // Угол из двух линий и свободная двойная линия.
        w.add(Line::horizontal(Rect::new(3, 2, 9, 3)));
        w.add(Line::vertical(Rect::new(9, 2, 10, 5)));
        w.add(Line::horizontal(Rect::new(12, 4, 16, 5)).double());
        let s = render(w);
        assert_eq!(s[2], "║  ──────┐         ║");
        assert_eq!(s[3], "║        │         ║");
        assert_eq!(s[4], "║        │  ════   ║");
    }

    #[test]
    fn to_frame_in_dialog_layout() {
        use crate::layout::VBox;
        let mut d = Dialog::new(Rect::new(0, 0, 24, 7), "");
        d.win.flags = 0;
        let a = d.add(crate::widgets::StaticText::new(Rect::default(), "верх"));
        let l = d.add(Line::horizontal(Rect::default()).to_frame());
        let b = d.add(crate::widgets::StaticText::new(Rect::default(), "низ"));
        d.set_layout(VBox::new().margins(3, 1, 3, 1).add(a).add(l).add(b));
        let s = render(d);
        // Рамка + отступ сверху: «верх» в строке 2, линия — в строке 3.
        let row = &s[3];
        let first = row.chars().next().unwrap();
        let last = row.chars().last().unwrap();
        assert!(matches!(first, '╟' | '├'), "{}", s.join("\n"));
        assert!(matches!(last, '╢' | '┤'), "{}", s.join("\n"));
        assert!(row.chars().skip(1).take(22).all(|c| c == '─'), "{}", s.join("\n"));
    }
}
