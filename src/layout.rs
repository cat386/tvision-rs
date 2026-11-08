//! Раскладки в духе Qt: [`HBox`], [`VBox`], [`Grid`], [`Form`] и
//! распорки ([`Spacer`]).
//!
//! Обычно положение каждого элемента задаётся прямоугольником
//! (`Rect::new(x1, y1, x2, y2)`), а растягивание с окном — `grow_mode`.
//! Раскладка делает это сама: вы говорите «эти элементы — в строку, эти —
//! в столбец, между ними — пружина», а координаты считаются по размерам
//! элементов и пересчитываются, когда окно меняет размер.
//!
//! ```
//! use tvision::layout::*;
//! use tvision::*;
//!
//! fn login() -> Dialog {
//!     let mut d = Dialog::centered(40, 10, "Вход");
//!     // Размеры не важны — их расставит раскладка.
//!     let name = d.add(InputLine::new(Rect::default(), 30));
//!     let pass = d.add(InputLine::new(Rect::default(), 30));
//!     let ln = d.add(Label::new(Rect::default(), "~И~мя:", Some(name)));
//!     let lp = d.add(Label::new(Rect::default(), "~П~ароль:", Some(pass)));
//!     let ok = d.add(Button::new(Rect::default(), "O~K~", cm::OK, bf::DEFAULT));
//!     let cancel = d.add(Button::new(Rect::default(), "Отмена", cm::CANCEL, bf::NORMAL));
//!     d.set_layout(
//!         VBox::new()
//!             .margins(2, 1, 2, 0)
//!             .add(Form::new().row(ln, name).row(lp, pass))
//!             .stretch(1) // пружина: кнопки — к низу окна
//!             .add(HBox::new().stretch(1).add(ok).add(cancel)), // и к правому краю
//!     );
//!     d.adjust_size(); // размер окна — по раскладке
//!     d
//! }
//!
//! let d = login();
//! let cancel = d.win.group.children.iter().filter(|c| c.as_any().is::<Button>()).last().unwrap();
//! assert_eq!(cancel.vs().bounds().b.x, d.win.group.vs.size.x - 3); // у правого края (рамка + поле 2)
//! ```
//!
//! ## Как считаются размеры
//!
//! У каждого элемента есть **желаемый размер** (`size_hint`: у кнопки —
//! по надписи, у строки ввода — по длине…), **минимум**, **максимум** и
//! **политика** ([`Policy`]) по каждой оси. Свободное место в строке
//! раздаётся так:
//!
//! 1. если у кого-то задан **коэффициент растяжения** (`stretch`), всё
//!    место, кроме желаемого размера остальных, делится по коэффициентам
//!    (`1:3` — четверть и три четверти, что бы ни было внутри);
//! 2. иначе — элементам с политикой `Expanding` (строки ввода, таблицы,
//!    списки), поровну;
//! 3. иначе — всем, кто может расти (`Preferred`, `Minimum`).
//!
//! Если места не хватает, сжимаются элементы, которые это позволяют
//! (`Preferred`, `Maximum`, `Expanding`), но не меньше минимума.
//!
//! Скрытые элементы (`sf::VISIBLE` снят) места не занимают — как в Qt.

use crate::geometry::{Point, Rect};
use crate::group::Group;
use crate::view::ViewId;

/// «Бесконечность» для максимального размера.
pub const MAX_SIZE: i32 = i32::MAX / 4;

// ----------------------------------------------------------------------
// Политики и выравнивание
// ----------------------------------------------------------------------

/// Как элемент относится к лишнему и недостающему месту по одной оси
/// (как `QSizePolicy::Policy`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Policy {
    /// Ровно желаемый размер: не растёт и не сжимается (кнопка).
    Fixed,
    /// Не меньше желаемого, может расти.
    Minimum,
    /// Не больше желаемого, может сжиматься.
    Maximum,
    /// Желаемый размер — лучший, но можно и больше, и меньше.
    #[default]
    Preferred,
    /// Хочет как можно больше места (строка ввода по ширине, таблица).
    Expanding,
    /// Желаемый размер не важен: занять, сколько дадут.
    Ignored,
}

impl Policy {
    fn can_grow(self) -> bool {
        !matches!(self, Policy::Fixed | Policy::Maximum)
    }
    fn can_shrink(self) -> bool {
        !matches!(self, Policy::Fixed | Policy::Minimum)
    }
    fn expanding(self) -> bool {
        matches!(self, Policy::Expanding | Policy::Ignored)
    }
}

/// Политики по ширине и высоте.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SizePolicy {
    pub h: Policy,
    pub v: Policy,
}

impl SizePolicy {
    pub const fn new(h: Policy, v: Policy) -> Self {
        SizePolicy { h, v }
    }
    /// Не менять размер вовсе (кнопки).
    pub const FIXED: SizePolicy = SizePolicy::new(Policy::Fixed, Policy::Fixed);
    /// По умолчанию: по ширине — «желаемо», по высоте — ровно (одна строка).
    pub const PREFERRED: SizePolicy = SizePolicy::new(Policy::Preferred, Policy::Fixed);
    /// Тянуться по ширине, высота постоянная (строка ввода).
    pub const EXPANDING_H: SizePolicy = SizePolicy::new(Policy::Expanding, Policy::Fixed);
    /// Тянуться во все стороны (таблица, список, редактор).
    pub const EXPANDING: SizePolicy = SizePolicy::new(Policy::Expanding, Policy::Expanding);
}

impl Default for SizePolicy {
    fn default() -> Self {
        SizePolicy::PREFERRED
    }
}

/// Выравнивание элемента в отведённом ему месте.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Alignment {
    /// Занять всё место, если политика позволяет расти (по умолчанию).
    #[default]
    Fill,
    /// Желаемый размер, прижать к началу (влево / вверх).
    Start,
    /// Желаемый размер, по центру.
    Center,
    /// Желаемый размер, прижать к концу (вправо / вниз).
    End,
}

/// Поля вокруг содержимого раскладки.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Margins {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

// ----------------------------------------------------------------------
// Элемент раскладки
// ----------------------------------------------------------------------

#[derive(Debug)]
enum Content {
    View(ViewId),
    Layout(Box<Layout>),
    Space,
}

/// Элемент раскладки: представление (по `ViewId`), вложенная раскладка
/// или распорка — с необязательными уточнениями размера.
///
/// Обычно создаётся неявно: `.add(id)`, `.add(HBox::new()…)`. Явно — через
/// [`item`], чтобы уточнить: `.add(item(table).stretch(2).min(20, 5))`.
#[derive(Debug)]
pub struct Item {
    content: Content,
    hint: (Option<i32>, Option<i32>),
    min: (Option<i32>, Option<i32>),
    max: (Option<i32>, Option<i32>),
    policy: Option<SizePolicy>,
    stretch: u16,
    halign: Alignment,
    valign: Alignment,
    /// Размер представления при первом расчёте (если у него нет `size_hint`).
    natural: Option<Point>,
    /// Последнее измерение (для расстановки).
    last: Option<Measure>,
}

/// Уточнить элемент: `item(id).stretch(2)`, `item(id).fixed_width(10)`…
pub fn item(x: impl Into<Item>) -> Item {
    x.into()
}

impl Item {
    fn new(content: Content) -> Self {
        Item {
            content,
            hint: (None, None),
            min: (None, None),
            max: (None, None),
            policy: None,
            stretch: 0,
            halign: Alignment::Fill,
            valign: Alignment::Fill,
            natural: None,
            last: None,
        }
    }

    /// Коэффициент растяжения: лишнее место делится пропорционально им.
    pub fn stretch(mut self, f: u16) -> Self {
        self.stretch = f;
        self
    }
    /// Желаемый размер вместо `size_hint` представления.
    pub fn hint(mut self, w: i32, h: i32) -> Self {
        self.hint = (Some(w), Some(h));
        self
    }
    /// Минимальный размер.
    pub fn min(mut self, w: i32, h: i32) -> Self {
        self.min = (Some(w), Some(h));
        self
    }
    /// Максимальный размер.
    pub fn max(mut self, w: i32, h: i32) -> Self {
        self.max = (Some(w), Some(h));
        self
    }
    pub fn min_width(mut self, w: i32) -> Self {
        self.min.0 = Some(w);
        self
    }
    pub fn max_width(mut self, w: i32) -> Self {
        self.max.0 = Some(w);
        self
    }
    pub fn min_height(mut self, h: i32) -> Self {
        self.min.1 = Some(h);
        self
    }
    pub fn max_height(mut self, h: i32) -> Self {
        self.max.1 = Some(h);
        self
    }
    /// Ровно такой размер.
    pub fn fixed(self, w: i32, h: i32) -> Self {
        self.fixed_width(w).fixed_height(h)
    }
    /// Ровно такая ширина.
    pub fn fixed_width(mut self, w: i32) -> Self {
        self.hint.0 = Some(w);
        self.min.0 = Some(w);
        self.max.0 = Some(w);
        self
    }
    /// Ровно такая высота.
    pub fn fixed_height(mut self, h: i32) -> Self {
        self.hint.1 = Some(h);
        self.min.1 = Some(h);
        self.max.1 = Some(h);
        self
    }
    /// Политика размера вместо политики представления.
    pub fn policy(mut self, h: Policy, v: Policy) -> Self {
        self.policy = Some(SizePolicy::new(h, v));
        self
    }
    /// Тянуться во все стороны.
    pub fn expand(self) -> Self {
        self.policy(Policy::Expanding, Policy::Expanding)
    }
    /// Выравнивание по горизонтали (не `Fill` — занимать желаемую ширину).
    pub fn align_h(mut self, a: Alignment) -> Self {
        self.halign = a;
        self
    }
    /// Выравнивание по вертикали.
    pub fn align_v(mut self, a: Alignment) -> Self {
        self.valign = a;
        self
    }
    /// По центру в обе стороны.
    pub fn center(self) -> Self {
        self.align_h(Alignment::Center).align_v(Alignment::Center)
    }
}

impl From<ViewId> for Item {
    fn from(id: ViewId) -> Self {
        Item::new(Content::View(id))
    }
}

impl From<Layout> for Item {
    fn from(l: Layout) -> Self {
        Item::new(Content::Layout(Box::new(l)))
    }
}

/// Распорка: пустое место (как `QSpacerItem`).
///
/// В `HBox`/`VBox` удобнее методы `.space(n)` (ровно `n` клеток) и
/// `.stretch(f)` (пружина); `Spacer` нужен в сетке и для особых случаев.
#[derive(Clone, Copy, Debug)]
pub struct Spacer {
    pub size: Point,
    pub policy: SizePolicy,
}

impl Spacer {
    /// Распорка размера `w × h` с политиками.
    pub fn new(w: i32, h: i32, hp: Policy, vp: Policy) -> Self {
        Spacer { size: Point::new(w, h), policy: SizePolicy::new(hp, vp) }
    }
    /// Пружина по горизонтали: забирает лишнюю ширину.
    pub fn horizontal() -> Self {
        Spacer::new(0, 0, Policy::Expanding, Policy::Minimum)
    }
    /// Пружина по вертикали: забирает лишнюю высоту.
    pub fn vertical() -> Self {
        Spacer::new(0, 0, Policy::Minimum, Policy::Expanding)
    }
    /// Ровно `w × h`.
    pub fn fixed(w: i32, h: i32) -> Self {
        Spacer::new(w, h, Policy::Fixed, Policy::Fixed)
    }
}

impl From<Spacer> for Item {
    fn from(s: Spacer) -> Self {
        let mut i = Item::new(Content::Space);
        i.hint = (Some(s.size.x), Some(s.size.y));
        i.policy = Some(s.policy);
        i
    }
}

// ----------------------------------------------------------------------
// Ограничения по оси и раздача места
// ----------------------------------------------------------------------

/// Что известно о размере элемента по одной оси.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Ax {
    min: i32,
    hint: i32,
    max: i32,
    /// 0 — не растёт, 1 — может расти, 2 — хочет расти.
    grow: u8,
    shrink: bool,
    stretch: u16,
}

impl Ax {
    const ZERO: Ax = Ax { min: 0, hint: 0, max: 0, grow: 0, shrink: false, stretch: 0 };

    fn from_policy(p: Policy, hint: i32) -> Ax {
        let hint = hint.max(0);
        let (min, max) = match p {
            Policy::Fixed => (hint, hint),
            Policy::Minimum => (hint, MAX_SIZE),
            Policy::Maximum => (0, hint),
            Policy::Preferred | Policy::Expanding => (0, MAX_SIZE),
            Policy::Ignored => (0, MAX_SIZE),
        };
        let hint = if p == Policy::Ignored { 0 } else { hint };
        let grow = if p.expanding() {
            2
        } else if p.can_grow() {
            1
        } else {
            0
        };
        Ax { min, hint, max, grow, shrink: p.can_shrink(), stretch: 0 }
    }

    /// Применить явные уточнения элемента.
    fn refine(mut self, hint: Option<i32>, min: Option<i32>, max: Option<i32>, stretch: u16) -> Ax {
        if let Some(h) = hint {
            self.hint = h.max(0);
        }
        if let Some(m) = min {
            self.min = m.max(0);
        }
        if let Some(m) = max {
            self.max = m.max(0);
            if self.max <= self.hint {
                self.grow = 0;
            }
        }
        if stretch > 0 {
            self.stretch = stretch;
        }
        self.max = self.max.max(self.min);
        self.hint = self.hint.clamp(self.min, self.max);
        if self.min >= self.hint {
            self.shrink = false;
        }
        if self.max <= self.hint && stretch == 0 {
            self.grow = 0;
        }
        self
    }
}

/// Разделить `amount` пропорционально весам (целыми, остаток — тем, у
/// кого больше дробная часть).
fn split(amount: i32, weights: &[i64]) -> Vec<i32> {
    let total: i64 = weights.iter().sum();
    if total <= 0 || amount <= 0 {
        return vec![0; weights.len()];
    }
    let mut out: Vec<i32> = weights.iter().map(|w| (amount as i64 * w / total) as i32).collect();
    let mut rest = amount - out.iter().sum::<i32>();
    let mut order: Vec<usize> = (0..weights.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse((amount as i64 * weights[i]) % total));
    for &i in order.iter().cycle() {
        if rest == 0 {
            break;
        }
        if weights[i] > 0 {
            out[i] += 1;
            rest -= 1;
        }
    }
    out
}

/// Раздать `total` клеток элементам по их ограничениям (алгоритм Qt).
fn distribute(total: i32, axes: &[Ax]) -> Vec<i32> {
    if axes.iter().any(|a| a.stretch > 0) {
        if let Some(v) = distribute_stretch(total, axes) {
            return v;
        }
    }
    let mut sizes: Vec<i32> = axes.iter().map(|a| a.hint).collect();
    let sum: i32 = sizes.iter().sum();
    if total >= sum {
        let mut extra = total - sum;
        // Кто получает лишнее: «хочет расти» → «может расти».
        let pick = |a: &Ax| -> i64 {
            if axes.iter().any(|a| a.grow == 2) {
                (a.grow == 2) as i64
            } else {
                (a.grow >= 1) as i64
            }
        };
        let mut open: Vec<bool> = axes.iter().map(|a| pick(a) > 0 && a.max > a.hint).collect();
        while extra > 0 && open.iter().any(|&o| o) {
            let w: Vec<i64> = axes.iter().zip(&open).map(|(a, &o)| if o { pick(a) } else { 0 }).collect();
            let shares = split(extra, &w);
            let mut used = 0;
            for i in 0..axes.len() {
                if shares[i] == 0 {
                    continue;
                }
                let give = shares[i].min(axes[i].max - sizes[i]);
                sizes[i] += give;
                used += give;
                if sizes[i] >= axes[i].max {
                    open[i] = false;
                }
            }
            if used == 0 {
                break;
            }
            extra -= used;
        }
    } else {
        let mut deficit = sum - total;
        loop {
            let w: Vec<i64> = axes.iter().zip(&sizes).map(|(a, &s)| if a.shrink && s > a.min { (s - a.min) as i64 } else { 0 }).collect();
            if deficit == 0 || w.iter().all(|&x| x == 0) {
                break;
            }
            let shares = split(deficit, &w);
            let mut taken = 0;
            for i in 0..axes.len() {
                let t = shares[i].min(sizes[i] - axes[i].min).max(0);
                sizes[i] -= t;
                taken += t;
            }
            if taken == 0 {
                break;
            }
            deficit -= taken;
        }
    }
    sizes
}

/// Есть коэффициенты растяжения: элементы без них получают желаемый
/// размер, остальное место делится по коэффициентам — независимо от
/// желаемых размеров, но в пределах [минимум, максимум] (как в Qt).
/// `None` — места не хватает даже на минимумы (тогда — обычное сжатие).
fn distribute_stretch(total: i32, axes: &[Ax]) -> Option<Vec<i32>> {
    let mut sizes: Vec<i32> = axes.iter().map(|a| if a.stretch > 0 { 0 } else { a.hint }).collect();
    let mut rest = total - sizes.iter().sum::<i32>();
    let need: i32 = axes.iter().filter(|a| a.stretch > 0).map(|a| a.min).sum();
    if rest < need {
        return None;
    }
    let mut open: Vec<bool> = axes.iter().map(|a| a.stretch > 0).collect();
    // Повторять, пока кто-то упирается в минимум или максимум.
    loop {
        let w: Vec<i64> = axes.iter().zip(&open).map(|(a, &o)| if o { a.stretch as i64 } else { 0 }).collect();
        if w.iter().all(|&x| x == 0) {
            break;
        }
        let shares = split(rest.max(0), &w);
        let mut clamped = false;
        for i in 0..axes.len() {
            if !open[i] {
                continue;
            }
            let v = shares[i];
            if v < axes[i].min || v > axes[i].max {
                sizes[i] = v.clamp(axes[i].min, axes[i].max);
                rest -= sizes[i];
                open[i] = false;
                clamped = true;
            }
        }
        if !clamped {
            for i in 0..axes.len() {
                if open[i] {
                    sizes[i] = shares[i];
                }
            }
            break;
        }
    }
    Some(sizes)
}

/// Размер и смещение элемента в отведённом месте `avail`.
fn fit(a: &Ax, avail: i32, align: Alignment) -> (i32, i32) {
    let size = if align == Alignment::Fill && a.grow > 0 { avail.min(a.max) } else { a.hint.min(avail) };
    let size = size.max(a.min.min(avail)).max(0);
    let off = match align {
        Alignment::Fill | Alignment::Start => 0,
        Alignment::Center => (avail - size).max(0) / 2,
        Alignment::End => (avail - size).max(0),
    };
    (size, off)
}

/// Где ставить промежуток `spacing` (как в Qt: перед каждым видимым
/// не-распоркой, если до него уже был такой же).
fn gap_flags(ms: &[Measure]) -> Vec<bool> {
    let mut seen = false;
    ms.iter()
        .map(|m| {
            if m.hidden || m.spacer {
                return false;
            }
            let f = seen;
            seen = true;
            f
        })
        .collect()
}

fn gaps(ms: &[Measure], spacing: i32) -> i32 {
    gap_flags(ms).iter().filter(|&&f| f).count() as i32 * spacing
}

/// Элементы друг за другом: размеры складываются.
fn sum_axes(it: impl Iterator<Item = Ax>, gaps: i32) -> Ax {
    let mut r = Ax { min: gaps, hint: gaps, max: gaps, grow: 0, shrink: false, stretch: 0 };
    for a in it {
        r.min += a.min;
        r.hint += a.hint;
        r.max = (r.max + a.max).min(MAX_SIZE);
        r.grow = r.grow.max(a.grow);
        r.shrink |= a.shrink;
    }
    r
}

/// Элементы рядом поперёк: берётся наибольший.
fn max_axes(it: impl Iterator<Item = Ax>) -> Ax {
    let mut r = Ax::ZERO;
    for a in it {
        r.min = r.min.max(a.min);
        r.hint = r.hint.max(a.hint);
        r.max = r.max.max(a.max);
        r.grow = r.grow.max(a.grow);
        r.shrink |= a.shrink;
    }
    r.max = r.max.max(r.hint);
    r
}

// ----------------------------------------------------------------------
// Раскладка
// ----------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    HBox,
    VBox,
    Grid,
}

/// Результат измерения элемента.
#[derive(Clone, Copy, Debug)]
struct Measure {
    x: Ax,
    y: Ax,
    /// Распорка: промежуток `spacing` рядом с ней не ставится.
    spacer: bool,
    /// Скрытое представление: места не занимает вовсе.
    hidden: bool,
}

/// Раскладка: строка ([`HBox`]), столбец ([`VBox`]), сетка ([`Grid`]) или
/// форма «подпись — поле» ([`Form`]).
///
/// Создаётся построителями `HBox::new()`, `VBox::new()`, `Grid::new()`,
/// `Form::new()`; ставится в окно `Window::set_layout` /
/// `Dialog::set_layout` (или в любую группу — `Group::set_layout`) и
/// дальше пересчитывается сама при каждом изменении размера.
#[derive(Debug)]
pub struct Layout {
    kind: Kind,
    items: Vec<Item>,
    /// Для сетки: (строка, столбец, строк, столбцов) каждого элемента.
    cells: Vec<(usize, usize, usize, usize)>,
    spacing: Point,
    margins: Margins,
    col_stretch: Vec<u16>,
    row_stretch: Vec<u16>,
    /// Для формы: следующая строка.
    next_row: usize,
    /// Отступ от края группы (у окна — 1, рамка).
    pub(crate) inset: i32,
    /// Минимальный размер при последнем расчёте (для `size_limits` окна).
    pub(crate) last_min: Point,
}

/// Строка: элементы слева направо (как `QHBoxLayout`).
pub struct HBox;
/// Столбец: элементы сверху вниз (как `QVBoxLayout`).
pub struct VBox;
/// Сетка: элементы по строкам и столбцам, можно на несколько клеток
/// (как `QGridLayout`).
pub struct Grid;
/// Форма: строки «подпись — поле»; столбец подписей по самой длинной,
/// поля забирают остальную ширину (как `QFormLayout`).
pub struct Form;

impl HBox {
    /// Промежуток между элементами — 1 клетка.
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Layout {
        Layout::new(Kind::HBox, Point::new(1, 0))
    }
}
impl VBox {
    /// Промежуток между элементами — 0 (строки вплотную).
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Layout {
        Layout::new(Kind::VBox, Point::new(0, 0))
    }
}
impl Grid {
    /// Промежутки: 1 между столбцами, 0 между строками.
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Layout {
        Layout::new(Kind::Grid, Point::new(1, 0))
    }
}
impl Form {
    /// Промежутки: 1 между подписью и полем и 1 между строками (как
    /// в диалогах Turbo Vision).
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Layout {
        let mut l = Layout::new(Kind::Grid, Point::new(1, 1));
        l.col_stretch = vec![0, 1];
        l
    }
}

impl Layout {
    fn new(kind: Kind, spacing: Point) -> Layout {
        Layout {
            kind,
            items: Vec::new(),
            cells: Vec::new(),
            spacing,
            margins: Margins::default(),
            col_stretch: Vec::new(),
            row_stretch: Vec::new(),
            next_row: 0,
            inset: 0,
            last_min: Point::default(),
        }
    }

    // ---------- построение ----------

    /// Добавить элемент: `ViewId`, вложенную раскладку, [`Spacer`] или
    /// [`item`]. В сетке — в новую строку, в первый столбец.
    pub fn add(mut self, x: impl Into<Item>) -> Layout {
        let it = x.into();
        if self.kind == Kind::Grid {
            let r = self.next_row;
            self.next_row += 1;
            self.cells.push((r, 0, 1, 1));
        }
        self.items.push(it);
        self
    }

    /// Пружина с коэффициентом `f`: забирает лишнее место (как `addStretch`).
    pub fn stretch(self, f: u16) -> Layout {
        let sp = match self.kind {
            Kind::VBox => Spacer::vertical(),
            _ => Spacer::horizontal(),
        };
        self.add(item(sp).stretch(f.max(1)))
    }

    /// Пустое место ровно `n` клеток вдоль раскладки (как `addSpacing`).
    pub fn space(self, n: i32) -> Layout {
        let sp = match self.kind {
            Kind::VBox => Spacer::new(0, n, Policy::Minimum, Policy::Fixed),
            _ => Spacer::new(n, 0, Policy::Fixed, Policy::Minimum),
        };
        self.add(sp)
    }

    /// Промежуток между элементами. В сетке и форме — между столбцами и
    /// между строками сразу.
    pub fn spacing(mut self, n: i32) -> Layout {
        match self.kind {
            Kind::HBox => self.spacing.x = n,
            Kind::VBox => self.spacing.y = n,
            Kind::Grid => self.spacing = Point::new(n, n),
        }
        self
    }
    /// Промежуток между столбцами (сетка, форма).
    pub fn hspacing(mut self, n: i32) -> Layout {
        self.spacing.x = n;
        self
    }
    /// Промежуток между строками (сетка, форма).
    pub fn vspacing(mut self, n: i32) -> Layout {
        self.spacing.y = n;
        self
    }

    /// Поля: слева, сверху, справа, снизу.
    pub fn margins(mut self, left: i32, top: i32, right: i32, bottom: i32) -> Layout {
        self.margins = Margins { left, top, right, bottom };
        self
    }
    /// Одинаковые поля со всех сторон.
    pub fn margin(self, n: i32) -> Layout {
        self.margins(n, n, n, n)
    }

    /// Сетка: элемент в клетку (`строка`, `столбец`).
    pub fn cell(self, row: usize, col: usize, x: impl Into<Item>) -> Layout {
        self.span(row, col, 1, 1, x)
    }
    /// Сетка: элемент на `rows × cols` клеток, начиная с (`строка`, `столбец`).
    pub fn span(mut self, row: usize, col: usize, rows: usize, cols: usize, x: impl Into<Item>) -> Layout {
        self.items.push(x.into());
        self.cells.push((row, col, rows.max(1), cols.max(1)));
        self.next_row = self.next_row.max(row + rows.max(1));
        self
    }
    /// Сетка: коэффициент растяжения столбца.
    pub fn col_stretch(mut self, col: usize, f: u16) -> Layout {
        if self.col_stretch.len() <= col {
            self.col_stretch.resize(col + 1, 0);
        }
        self.col_stretch[col] = f;
        self
    }
    /// Сетка: коэффициент растяжения строки.
    pub fn row_stretch(mut self, row: usize, f: u16) -> Layout {
        if self.row_stretch.len() <= row {
            self.row_stretch.resize(row + 1, 0);
        }
        self.row_stretch[row] = f;
        self
    }

    /// Форма: строка «подпись — поле».
    pub fn row(mut self, label: impl Into<Item>, field: impl Into<Item>) -> Layout {
        let r = self.next_row;
        self = self.span(r, 0, 1, 1, item(label).align_v(Alignment::Start));
        self.span(r, 1, 1, 1, field)
    }
    /// Форма: элемент на всю ширину (оба столбца).
    pub fn row_wide(self, x: impl Into<Item>) -> Layout {
        let r = self.next_row;
        self.span(r, 0, 1, 2, x)
    }

    // ---------- расчёт ----------

    /// Желаемый размер раскладки (с полями).
    pub fn size_hint(&mut self, g: &Group) -> Point {
        let m = self.measure(g);
        Point::new(m.x.hint, m.y.hint)
    }

    /// Минимальный размер раскладки (с полями).
    pub fn min_size(&mut self, g: &Group) -> Point {
        let m = self.measure(g);
        Point::new(m.x.min, m.y.min)
    }

    /// Рассчитать и расставить представления группы `g` в прямоугольнике
    /// `r` (координаты группы). Обычно вызывать не нужно: это делает
    /// окно при каждом изменении размера.
    pub fn apply(&mut self, g: &mut Group, r: Rect) {
        let m = self.measure(g);
        self.last_min = Point::new(m.x.min, m.y.min);
        let mut out = Vec::new();
        self.place(r, &mut out);
        for (id, b) in out {
            if let Some(v) = g.view_mut(id) {
                if v.vs().bounds() != b {
                    v.change_bounds(b);
                }
            }
        }
    }

    /// Где окажется каждое представление (без изменения группы) — для
    /// тестов и отладки.
    pub fn geometry(&mut self, g: &Group, r: Rect) -> Vec<(ViewId, Rect)> {
        self.measure(g);
        let mut out = Vec::new();
        self.place(r, &mut out);
        out
    }

    fn measure_item(it: &mut Item, g: &Group) -> Measure {
        let (mut x, mut y, spacer, hidden) = match &mut it.content {
            Content::View(id) => match g.view(*id) {
                None => (Ax::ZERO, Ax::ZERO, false, true),
                Some(v) if !v.vs().is_visible() => (Ax::ZERO, Ax::ZERO, false, true),
                Some(v) => {
                    let natural = *it.natural.get_or_insert(v.vs().size);
                    let hint = v.size_hint().unwrap_or(natural);
                    let pol = it.policy.unwrap_or_else(|| v.size_policy());
                    // Пустой размер у растягивающихся — разумный минимум.
                    let hx = if hint.x <= 0 {
                        if pol.h.expanding() {
                            10
                        } else {
                            1
                        }
                    } else {
                        hint.x
                    };
                    let hy = if hint.y <= 0 {
                        if pol.v.expanding() {
                            3
                        } else {
                            1
                        }
                    } else {
                        hint.y
                    };
                    let (mut x, mut y) = (Ax::from_policy(pol.h, hx), Ax::from_policy(pol.v, hy));
                    // Минимум по умолчанию (как minimumSizeHint в Qt): подпись не
                    // уже своего текста, растягивающиеся — до 5×3.
                    for (a, p, lim) in [(&mut x, pol.h, 5), (&mut y, pol.v, 3)] {
                        match p {
                            Policy::Preferred => a.min = a.hint,
                            Policy::Expanding => a.min = a.hint.min(lim),
                            _ => {}
                        }
                        if a.min >= a.hint {
                            a.shrink = false;
                        }
                    }
                    (x, y, false, false)
                }
            },
            Content::Space => {
                let pol = it.policy.unwrap_or(SizePolicy::new(Policy::Minimum, Policy::Minimum));
                (Ax::from_policy(pol.h, it.hint.0.unwrap_or(0)), Ax::from_policy(pol.v, it.hint.1.unwrap_or(0)), true, false)
            }
            Content::Layout(l) => {
                let m = l.measure(g);
                let mut x = m.x;
                let mut y = m.y;
                if let Some(p) = it.policy {
                    x = Ax::from_policy(p.h, x.hint);
                    y = Ax::from_policy(p.v, y.hint);
                }
                (x, y, false, false)
            }
        };
        if hidden {
            let m = Measure { x, y, spacer, hidden };
            it.last = Some(m);
            return m;
        }
        let (hint_x, hint_y) = match it.content {
            Content::Space => (None, None), // уже учтено
            _ => it.hint,
        };
        x = x.refine(hint_x, it.min.0, it.max.0, it.stretch);
        y = y.refine(hint_y, it.min.1, it.max.1, it.stretch);
        let m = Measure { x, y, spacer, hidden };
        it.last = Some(m);
        m
    }

    fn measure_cached(it: &Item) -> Measure {
        it.last.unwrap_or(Measure { x: Ax::ZERO, y: Ax::ZERO, spacer: false, hidden: true })
    }

    fn measure(&mut self, g: &Group) -> Measure {
        let ms: Vec<Measure> = self.items.iter_mut().map(|it| Self::measure_item(it, g)).collect();
        let (mut x, mut y) = match self.kind {
            Kind::HBox => {
                let gaps = gaps(&ms, self.spacing.x);
                (sum_axes(ms.iter().filter(|m| !m.hidden).map(|m| m.x), gaps), max_axes(ms.iter().filter(|m| !m.hidden).map(|m| m.y)))
            }
            Kind::VBox => {
                let gaps = gaps(&ms, self.spacing.y);
                (max_axes(ms.iter().filter(|m| !m.hidden).map(|m| m.x)), sum_axes(ms.iter().filter(|m| !m.hidden).map(|m| m.y), gaps))
            }
            Kind::Grid => {
                let (cols, rows) = self.grid_axes(&ms);
                let cg = self.spacing.x * (cols.iter().filter(|c| c.1).count() as i32 - 1).max(0);
                let rg = self.spacing.y * (rows.iter().filter(|c| c.1).count() as i32 - 1).max(0);
                (sum_axes(cols.iter().map(|c| c.0), cg), sum_axes(rows.iter().map(|c| c.0), rg))
            }
        };
        let mx = self.margins.left + self.margins.right;
        let my = self.margins.top + self.margins.bottom;
        for (a, m) in [(&mut x, mx), (&mut y, my)] {
            a.min += m;
            a.hint += m;
            a.max = (a.max + m).min(MAX_SIZE);
            a.stretch = 0;
        }
        Measure { x, y, spacer: false, hidden: false }
    }

    /// Ограничения столбцов и строк сетки: (ось, есть ли видимые элементы).
    fn grid_axes(&self, ms: &[Measure]) -> (Vec<(Ax, bool)>, Vec<(Ax, bool)>) {
        let ncols = self.cells.iter().map(|c| c.1 + c.3).max().unwrap_or(0).max(self.col_stretch.len());
        let nrows = self.cells.iter().map(|c| c.0 + c.2).max().unwrap_or(0).max(self.row_stretch.len());
        let lines =
            |n: usize, stretch: &[u16], pos: &dyn Fn(&(usize, usize, usize, usize)) -> (usize, usize), ax: &dyn Fn(&Measure) -> Ax, gap: i32| {
                let mut v: Vec<(Ax, bool)> = (0..n)
                    .map(|i| (Ax { min: 0, hint: 0, max: 0, grow: 0, shrink: true, stretch: stretch.get(i).copied().unwrap_or(0) }, false))
                    .collect();
                // Сначала элементы на одну клетку.
                for (c, m) in self.cells.iter().zip(ms) {
                    let (p, span) = pos(c);
                    if m.hidden || span != 1 {
                        continue;
                    }
                    let a = ax(m);
                    let l = &mut v[p];
                    l.0.min = l.0.min.max(a.min);
                    l.0.hint = l.0.hint.max(a.hint);
                    l.0.max = l.0.max.max(a.max);
                    l.0.grow = l.0.grow.max(a.grow);
                    l.0.shrink &= a.shrink;
                    l.1 = true;
                }
                // Потом широкие: не хватает места — добавить последней линии.
                for (c, m) in self.cells.iter().zip(ms) {
                    let (p, span) = pos(c);
                    if m.hidden || span == 1 {
                        continue;
                    }
                    let a = ax(m);
                    let have: i32 = v[p..p + span].iter().map(|l| l.0.hint).sum::<i32>() + gap * (span as i32 - 1);
                    let last = &mut v[p + span - 1];
                    if a.hint > have {
                        last.0.hint += a.hint - have;
                        last.0.min = last.0.min.max(last.0.hint.min(a.min));
                    }
                    last.0.max = last.0.max.max(last.0.hint);
                    if a.grow > 0 && v[p..p + span].iter().all(|l| l.0.grow == 0) {
                        v[p + span - 1].0.grow = a.grow;
                        v[p + span - 1].0.max = MAX_SIZE;
                    }
                    for l in &mut v[p..p + span] {
                        l.1 = true;
                    }
                }
                for l in &mut v {
                    l.0.max = l.0.max.max(l.0.hint);
                    if l.0.stretch > 0 && l.0.max <= l.0.hint {
                        l.0.max = MAX_SIZE; // растягиваемый столбец может расти
                    }
                }
                v
            };
        let cols = lines(ncols, &self.col_stretch, &|c| (c.1, c.3), &|m| m.x, self.spacing.x);
        let rows = lines(nrows, &self.row_stretch, &|c| (c.0, c.2), &|m| m.y, self.spacing.y);
        (cols, rows)
    }

    fn place(&mut self, r: Rect, out: &mut Vec<(ViewId, Rect)>) {
        let inner = Rect::new(r.a.x + self.margins.left, r.a.y + self.margins.top, r.b.x - self.margins.right, r.b.y - self.margins.bottom);
        // Измерения уже свежие (measure вызван выше по дереву).
        let ms: Vec<Measure> = self.items.iter_mut().map(|it| Self::measure_cached(it)).collect();
        match self.kind {
            Kind::HBox | Kind::VBox => {
                let horiz = self.kind == Kind::HBox;
                let spacing = if horiz { self.spacing.x } else { self.spacing.y };
                let (len, cross) =
                    if horiz { (inner.b.x - inner.a.x, inner.b.y - inner.a.y) } else { (inner.b.y - inner.a.y, inner.b.x - inner.a.x) };
                let flags = gap_flags(&ms);
                let total_gaps = flags.iter().filter(|&&f| f).count() as i32 * spacing;
                let axes: Vec<Ax> = ms
                    .iter()
                    .map(|m| {
                        if m.hidden {
                            Ax::ZERO
                        } else if horiz {
                            m.x
                        } else {
                            m.y
                        }
                    })
                    .collect();
                let sizes = distribute(len - total_gaps, &axes);
                let mut pos = 0;
                for i in 0..self.items.len() {
                    if ms[i].hidden {
                        continue;
                    }
                    if flags[i] {
                        pos += spacing;
                    }
                    let it = &self.items[i];
                    let (main_al, cross_al) = if horiz { (it.halign, it.valign) } else { (it.valign, it.halign) };
                    let (ma, ca) = if horiz { (ms[i].x, ms[i].y) } else { (ms[i].y, ms[i].x) };
                    let (msz, moff) = if main_al == Alignment::Fill { (sizes[i], 0) } else { fit(&ma, sizes[i], main_al) };
                    let (csz, coff) = fit(&ca, cross, cross_al);
                    let b = if horiz {
                        Rect::new(inner.a.x + pos + moff, inner.a.y + coff, inner.a.x + pos + moff + msz, inner.a.y + coff + csz)
                    } else {
                        Rect::new(inner.a.x + coff, inner.a.y + pos + moff, inner.a.x + coff + csz, inner.a.y + pos + moff + msz)
                    };
                    Self::place_item(&mut self.items[i], b, out);
                    pos += sizes[i];
                }
            }
            Kind::Grid => {
                let (cols, rows) = self.grid_axes(&ms);
                let offsets = |lines: &[(Ax, bool)], len: i32, gap: i32| -> (Vec<i32>, Vec<i32>) {
                    let n_vis = lines.iter().filter(|l| l.1).count() as i32;
                    let axes: Vec<Ax> = lines.iter().map(|l| if l.1 { l.0 } else { Ax::ZERO }).collect();
                    let sizes = distribute(len - gap * (n_vis - 1).max(0), &axes);
                    let mut off = Vec::with_capacity(lines.len());
                    let mut p = 0;
                    let mut first = true;
                    for (i, l) in lines.iter().enumerate() {
                        if l.1 {
                            if !first {
                                p += gap;
                            }
                            first = false;
                        }
                        off.push(p);
                        p += sizes[i];
                    }
                    (off, sizes)
                };
                let (cx, cw) = offsets(&cols, inner.b.x - inner.a.x, self.spacing.x);
                let (ry, rh) = offsets(&rows, inner.b.y - inner.a.y, self.spacing.y);
                for i in 0..self.items.len() {
                    if ms[i].hidden {
                        continue;
                    }
                    let (r0, c0, rs, cs) = self.cells[i];
                    let x = cx[c0];
                    let w = cx[c0 + cs - 1] + cw[c0 + cs - 1] - x;
                    let y = ry[r0];
                    let h = ry[r0 + rs - 1] + rh[r0 + rs - 1] - y;
                    let it = &self.items[i];
                    let (sw, ox) = fit(&ms[i].x, w, it.halign);
                    let (sh, oy) = fit(&ms[i].y, h, it.valign);
                    let b = Rect::new(inner.a.x + x + ox, inner.a.y + y + oy, inner.a.x + x + ox + sw, inner.a.y + y + oy + sh);
                    Self::place_item(&mut self.items[i], b, out);
                }
            }
        }
    }

    fn place_item(it: &mut Item, b: Rect, out: &mut Vec<(ViewId, Rect)>) {
        match &mut it.content {
            Content::View(id) => out.push((*id, b)),
            Content::Layout(l) => l.place(b, out),
            Content::Space => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::{sf, View};
    use crate::widgets::*;
    use crate::{cm, Rect};

    fn ax(hint: i32, p: Policy) -> Ax {
        Ax::from_policy(p, hint)
    }

    #[test]
    fn distribute_rules() {
        use Policy::*;
        // Лишнее — растягиваемым («Expanding» важнее «Preferred»).
        assert_eq!(distribute(30, &[ax(5, Fixed), ax(5, Preferred), ax(5, Expanding)]), [5, 5, 20]);
        // Нет «Expanding» — всем, кто может расти, поровну.
        assert_eq!(distribute(21, &[ax(5, Fixed), ax(5, Preferred), ax(5, Minimum)]), [5, 8, 8]);
        // Коэффициенты растяжения главнее политики.
        let mut a = ax(0, Preferred);
        a.stretch = 1;
        let mut b = ax(0, Preferred);
        b.stretch = 3;
        assert_eq!(distribute(20, &[a, b, ax(4, Expanding)]), [4, 12, 4]);
        // Максимум: упёрся — остальное другим.
        let c = ax(2, Expanding).refine(None, None, Some(4), 0);
        assert_eq!(distribute(20, &[c, ax(2, Expanding)]), [4, 16]);
        // Мало места: сжимаются сжимаемые, Fixed остаётся.
        assert_eq!(distribute(12, &[ax(10, Fixed), ax(10, Preferred)]), [10, 2]);
        // Пропорционально запасу сжатия.
        assert_eq!(distribute(15, &[ax(10, Preferred), ax(10, Preferred)]), [7, 8]);
        // Совсем мало: не меньше минимума (вылезет за край).
        assert_eq!(distribute(3, &[ax(10, Fixed), ax(10, Minimum)]), [10, 10]);
    }

    #[test]
    fn split_is_exact() {
        assert_eq!(split(10, &[1, 1, 1]), [4, 3, 3]);
        assert_eq!(split(7, &[0, 2, 1]).iter().sum::<i32>(), 7);
        assert_eq!(split(5, &[0, 0]), [0, 0]);
    }

    fn group() -> Group {
        Group::new(Rect::new(0, 0, 60, 20))
    }

    fn bounds(g: &Group, id: ViewId) -> Rect {
        g.view(id).unwrap().vs().bounds()
    }

    #[test]
    fn buttons_to_the_right() {
        let mut g = group();
        let ok = g.insert(Box::new(Button::new(Rect::default(), "O~K~", cm::OK, bf::DEFAULT)));
        let no = g.insert(Box::new(Button::new(Rect::default(), "Отмена", cm::CANCEL, bf::NORMAL)));
        g.set_layout(HBox::new().stretch(1).add(ok).add(no).margins(0, 0, 2, 0));
        assert_eq!(bounds(&g, no), Rect::new(48, 0, 58, 2));
        assert_eq!(bounds(&g, ok), Rect::new(37, 0, 47, 2)); // промежуток 1
    }

    #[test]
    fn form_columns_and_rows() {
        let mut g = group();
        let a = g.insert(Box::new(InputLine::new(Rect::default(), 20)));
        let b = g.insert(Box::new(InputLine::new(Rect::default(), 5)));
        let la = g.insert(Box::new(Label::new(Rect::default(), "~И~мя:", Some(a))));
        let lb = g.insert(Box::new(Label::new(Rect::default(), "~Д~линная подпись:", Some(b))));
        g.set_layout(Form::new().margin(1).row(la, a).row(lb, b));
        // Столбец подписей — по самой длинной («Длинная подпись:» 16 + 1), дальше промежуток 1.
        assert_eq!(bounds(&g, la), Rect::new(1, 1, 18, 2));
        assert_eq!(bounds(&g, lb), Rect::new(1, 3, 18, 4));
        // Поля — с одного столбца и до правого поля (строки ввода тянутся).
        assert_eq!(bounds(&g, a), Rect::new(19, 1, 59, 2));
        assert_eq!(bounds(&g, b), Rect::new(19, 3, 59, 4));
    }

    #[test]
    fn grid_spans_and_stretch() {
        let mut g = group();
        let ids: Vec<ViewId> = (0..4).map(|_| g.insert(Box::new(Button::new(Rect::default(), "X", cm::OK, bf::NORMAL)))).collect();
        let wide = g.insert(Box::new(InputLine::new(Rect::default(), 10)));
        g.set_layout(
            Grid::new()
                .span(0, 0, 1, 2, wide)
                .cell(1, 0, ids[0])
                .cell(1, 1, ids[1])
                .cell(2, 0, ids[2])
                .cell(2, 1, ids[3])
                .col_stretch(0, 1)
                .col_stretch(1, 1),
        );
        // Два столбца поровну (60 - 1 промежуток → 30 и 29); кнопки — 10 шириной, слева в клетке.
        assert_eq!(bounds(&g, wide), Rect::new(0, 0, 60, 1));
        assert_eq!(bounds(&g, ids[1]), Rect::new(31, 1, 41, 3));
        assert_eq!(bounds(&g, ids[2]), Rect::new(0, 3, 10, 5));
    }

    #[test]
    fn hidden_items_take_no_space() {
        let mut g = group();
        let a = g.insert(Box::new(Button::new(Rect::default(), "A", cm::OK, bf::NORMAL)));
        let b = g.insert(Box::new(Button::new(Rect::default(), "B", cm::OK, bf::NORMAL)));
        let c = g.insert(Box::new(Button::new(Rect::default(), "C", cm::OK, bf::NORMAL)));
        g.set_layout(HBox::new().add(a).add(b).add(c));
        assert_eq!(bounds(&g, c).a.x, 22);
        g.view_mut(b).unwrap().vs_mut().state &= !sf::VISIBLE;
        g.apply_layout();
        assert_eq!(bounds(&g, c).a.x, 11);
    }

    #[test]
    fn nested_with_alignment_and_space() {
        let mut g = group();
        let t = g.insert(Box::new(StaticText::new(Rect::default(), "Заголовок")));
        let list = g.insert(Box::new(ListBox::new(Rect::default(), 1, None)));
        let ok = g.insert(Box::new(Button::new(Rect::default(), "OK", cm::OK, bf::NORMAL)));
        g.set_layout(VBox::new().add(item(t).align_h(Alignment::Center)).space(1).add(list).add(item(ok).align_h(Alignment::End)));
        assert_eq!(bounds(&g, t), Rect::new(25, 0, 34, 1)); // по центру
        assert_eq!(bounds(&g, list), Rect::new(0, 2, 60, 18)); // всё оставшееся
        assert_eq!(bounds(&g, ok), Rect::new(50, 18, 60, 20)); // справа внизу
    }

    #[test]
    fn item_overrides() {
        let mut g = group();
        let a = g.insert(Box::new(InputLine::new(Rect::default(), 30)));
        let b = g.insert(Box::new(InputLine::new(Rect::default(), 30)));
        g.set_layout(HBox::new().spacing(0).add(item(a).fixed_width(10)).add(item(b).max_width(20)));
        assert_eq!(bounds(&g, a), Rect::new(0, 0, 10, 1));
        assert_eq!(bounds(&g, b), Rect::new(10, 0, 30, 1)); // упёрлась в максимум
    }

    fn dialog() -> (Dialog, ViewId, ViewId) {
        let mut d = Dialog::centered(30, 8, "Тест");
        let name = d.add(InputLine::new(Rect::default(), 20));
        let ln = d.add(Label::new(Rect::default(), "~И~мя:", Some(name)));
        let ok = d.add(Button::new(Rect::default(), "O~K~", cm::OK, bf::DEFAULT));
        d.set_layout(VBox::new().margins(1, 1, 1, 0).add(HBox::new().add(ln).add(name)).stretch(1).add(HBox::new().stretch(1).add(ok)));
        (d, name, ok)
    }

    #[test]
    fn window_resize_relayouts() {
        let (mut d, name, ok) = dialog();
        assert_eq!(d.win.group.view(ok).unwrap().vs().bounds(), Rect::new(18, 5, 28, 7));
        // Окно шире и выше: строка ввода тянется, кнопка едет в угол.
        d.change_bounds(Rect::new(0, 0, 50, 12));
        assert_eq!(d.win.group.view(name).unwrap().vs().bounds(), Rect::new(8, 2, 48, 3));
        assert_eq!(d.win.group.view(ok).unwrap().vs().bounds(), Rect::new(38, 9, 48, 11));
        // Меньше минимума раскладки окно не сделать.
        let (min, _) = d.size_limits(Point::new(80, 25));
        assert_eq!(min, crate::widgets::window::MIN_WIN_SIZE); // раскладке хватает 13×6
    }

    #[test]
    fn adjust_size_fits_hint() {
        let (mut d, name, _) = dialog();
        d.adjust_size();
        // Ширина: 1 + «Имя:»(5) + 1 + строка(22) + 1 + рамка 2 = 32; высота: 1 + 1 + 2 + рамка 2 = 6.
        assert_eq!(d.win.group.vs.size, Point::new(32, 6));
        assert_eq!(d.win.group.view(name).unwrap().vs().bounds(), Rect::new(8, 2, 30, 3));
    }
}
