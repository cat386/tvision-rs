//! Геометрия: точка и прямоугольник.
//! Порт `TPoint` и `TRect` из `include/tvision/objects.h`.

use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Точка на экране (столбец `x`, строка `y`). Аналог `TPoint`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
}

impl Add for Point {
    type Output = Point;
    fn add(self, o: Point) -> Point {
        Point::new(self.x + o.x, self.y + o.y)
    }
}
impl Sub for Point {
    type Output = Point;
    fn sub(self, o: Point) -> Point {
        Point::new(self.x - o.x, self.y - o.y)
    }
}
impl AddAssign for Point {
    fn add_assign(&mut self, o: Point) {
        self.x += o.x;
        self.y += o.y;
    }
}
impl SubAssign for Point {
    fn sub_assign(&mut self, o: Point) {
        self.x -= o.x;
        self.y -= o.y;
    }
}

/// Прямоугольник: `a` — левый верхний угол (включительно),
/// `b` — правый нижний угол (НЕ включительно), как в оригинальном `TRect`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rect {
    pub a: Point,
    pub b: Point,
}

impl Rect {
    /// Создаёт прямоугольник по координатам углов (как `TRect(ax, ay, bx, by)`).
    pub const fn new(ax: i32, ay: i32, bx: i32, by: i32) -> Self {
        Rect { a: Point::new(ax, ay), b: Point::new(bx, by) }
    }

    /// Прямоугольник из начала координат и размера.
    pub fn from_origin_size(origin: Point, size: Point) -> Self {
        Rect { a: origin, b: origin + size }
    }

    /// Ширина и высота.
    pub fn size(&self) -> Point {
        self.b - self.a
    }

    pub fn width(&self) -> i32 {
        self.b.x - self.a.x
    }

    pub fn height(&self) -> i32 {
        self.b.y - self.a.y
    }

    /// Сдвигает прямоугольник на (dx, dy). Аналог `TRect::move`.
    pub fn moved(mut self, dx: i32, dy: i32) -> Self {
        self.a.x += dx;
        self.a.y += dy;
        self.b.x += dx;
        self.b.y += dy;
        self
    }

    /// Расширяет (или сжимает при отрицательных значениях) прямоугольник
    /// на dx по горизонтали и dy по вертикали с каждой стороны. Аналог `TRect::grow`.
    pub fn grown(mut self, dx: i32, dy: i32) -> Self {
        self.a.x -= dx;
        self.a.y -= dy;
        self.b.x += dx;
        self.b.y += dy;
        self
    }

    /// Пересечение двух прямоугольников. Аналог `TRect::intersect`.
    pub fn intersect(&self, r: &Rect) -> Rect {
        Rect::new(
            self.a.x.max(r.a.x),
            self.a.y.max(r.a.y),
            self.b.x.min(r.b.x),
            self.b.y.min(r.b.y),
        )
    }

    /// Объединение (наименьший охватывающий прямоугольник). Аналог `TRect::Union`.
    pub fn union(&self, r: &Rect) -> Rect {
        Rect::new(
            self.a.x.min(r.a.x),
            self.a.y.min(r.a.y),
            self.b.x.max(r.b.x),
            self.b.y.max(r.b.y),
        )
    }

    /// Лежит ли точка внутри прямоугольника.
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.a.x && p.x < self.b.x && p.y >= self.a.y && p.y < self.b.y
    }

    /// Пустой ли прямоугольник (нулевая или отрицательная площадь).
    pub fn is_empty(&self) -> bool {
        self.a.x >= self.b.x || self.a.y >= self.b.y
    }
}
