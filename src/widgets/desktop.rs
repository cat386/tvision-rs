//! Рабочий стол (`TDesktop`): фон + окна. Умеет переключать окна (F6 / Shift+F6),
//! раскладывать их «мозаикой» (`cm::TILE`) и «каскадом» (`cm::CASCADE`).

use crate::event::{cm, Event};
use crate::geometry::{Point, Rect};
use crate::group::Group;
use crate::view::{gf, of, Ctx, DrawCtx, View, ViewId};
use crate::widgets::background::Background;
use crate::widgets::window::MIN_WIN_SIZE;

pub struct Desktop {
    pub group: Group,
    pub background_id: ViewId,
    /// Раскладывать мозаику сначала по столбцам (`tileColumnsFirst`).
    pub tile_columns_first: bool,
}

impl Desktop {
    pub fn new(bounds: Rect) -> Self {
        let mut group = Group::new(bounds);
        group.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
        group.vs.options |= of::SELECTABLE;
        let bg = Background::new(group.vs.extent(), '░');
        let background_id = group.add(bg);
        Desktop { group, background_id, tile_columns_first: false }
    }

    /// Окна, которые участвуют в мозаике/каскаде (индексы сверху вниз).
    fn tileable(&self) -> Vec<usize> {
        (0..self.group.children.len())
            .rev()
            .filter(|&i| {
                let vs = self.group.children[i].vs();
                vs.options & of::TILEABLE != 0 && vs.is_visible()
            })
            .collect()
    }

    /// Каскад окон (`TDesktop::cascade`).
    pub fn cascade(&mut self, r: Rect) {
        let wins = self.tileable();
        let n = wins.len() as i32;
        if n == 0 {
            return;
        }
        // Проверяем, что самое смещённое окно не станет меньше минимума.
        let min = MIN_WIN_SIZE;
        if r.width() - (n - 1) < min.x || r.height() - (n - 1) < min.y {
            return;
        }
        let mut num = n - 1;
        for i in wins {
            let nr = Rect::new(r.a.x + num, r.a.y + num, r.b.x, r.b.y);
            locate(self.group.children[i].as_mut(), nr, self.group.vs.size);
            num -= 1;
        }
    }

    /// Мозаика окон (`TDesktop::tile`).
    pub fn tile(&mut self, r: Rect) {
        let wins = self.tileable();
        let n = wins.len() as i32;
        if n == 0 {
            return;
        }
        let (cols, rows) = most_equal_divisors(n, !self.tile_columns_first);
        if r.width() / cols == 0 || r.height() / rows == 0 {
            return;
        }
        let left_over = n % cols;
        let mut tile_num = n - 1;
        for i in wins {
            let nr = calc_tile_rect(tile_num, r, cols, rows, left_over);
            locate(self.group.children[i].as_mut(), nr, self.group.vs.size);
            tile_num -= 1;
        }
    }

    /// Следующее окно (F6): текущее опускается вниз, выбирается верхнее.
    fn next_window(&mut self, ctx: &mut Ctx) {
        let n = self.group.children.len();
        if n < 3 {
            return;
        }
        if let Some(cur) = self.group.current {
            // Индекс 0 — фон, поэтому окно уходит на позицию 1.
            self.group.put_at(cur, 1);
        }
        if let Some(top) = self.top_window() {
            self.group.select_index(top, ctx);
        }
    }

    /// Предыдущее окно (Shift+F6): выбирается самое нижнее окно.
    fn prev_window(&mut self, ctx: &mut Ctx) {
        let i = (1..self.group.children.len()).find(|&i| {
            let vs = self.group.children[i].vs();
            vs.is_visible() && vs.is_selectable()
        });
        if let Some(i) = i {
            self.group.select_index(i, ctx);
        }
    }

    fn top_window(&self) -> Option<usize> {
        self.group.children.iter().rposition(|c| c.vs().is_visible() && c.vs().is_selectable())
    }
}

/// Помещает представление в прямоугольник с учётом его ограничений размера (`locate`).
fn locate(v: &mut dyn View, mut r: Rect, owner: Point) {
    let (min, max) = v.size_limits(owner);
    r.b.x = r.a.x + r.width().clamp(min.x, max.x.max(min.x));
    r.b.y = r.a.y + r.height().clamp(min.y, max.y.max(min.y));
    v.change_bounds(r);
}

/// Подбор числа столбцов и строк для мозаики (`mostEqualDivisors`).
fn most_equal_divisors(n: i32, favor_y: bool) -> (i32, i32) {
    let mut i = (n as f64).sqrt() as i32;
    if i == 0 {
        i = 1;
    }
    if n % i != 0 && n % (i + 1) == 0 {
        i += 1;
    }
    if i < n / i {
        i = n / i;
    }
    if favor_y {
        (n / i, i)
    } else {
        (i, n / i)
    }
}

fn divider_loc(lo: i32, hi: i32, num: i32, pos: i32) -> i32 {
    ((hi - lo) as i64 * pos as i64 / num as i64) as i32 + lo
}

/// Прямоугольник окна номер `pos` в мозаике (`calcTileRect`).
fn calc_tile_rect(pos: i32, r: Rect, cols: i32, rows: i32, left_over: i32) -> Rect {
    let d = (cols - left_over) * rows;
    let (x, y) = if pos < d { (pos / rows, pos % rows) } else { ((pos - d) / (rows + 1) + (cols - left_over), (pos - d) % (rows + 1)) };
    let ax = divider_loc(r.a.x, r.b.x, cols, x);
    let bx = divider_loc(r.a.x, r.b.x, cols, x + 1);
    let (ay, by) = if pos >= d {
        (divider_loc(r.a.y, r.b.y, rows + 1, y), divider_loc(r.a.y, r.b.y, rows + 1, y + 1))
    } else {
        (divider_loc(r.a.y, r.b.y, rows, y), divider_loc(r.a.y, r.b.y, rows, y + 1))
    };
    Rect::new(ax, ay, bx, by)
}

impl View for Desktop {
    crate::impl_group_basics!(group);

    fn draw(&self, dc: &mut DrawCtx) {
        self.group.draw_children(dc);
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        self.group.handle_group_event(ev, ctx);
        let extent = self.group.vs.extent();
        match ev.command() {
            Some(cm::NEXT) => {
                self.next_window(ctx);
                ev.clear();
            }
            Some(cm::PREV) => {
                self.prev_window(ctx);
                ev.clear();
            }
            Some(cm::TILE) => {
                self.tile(extent);
                ev.clear();
            }
            Some(cm::CASCADE) => {
                self.cascade(extent);
                ev.clear();
            }
            _ => {}
        }
    }

    fn set_state(&mut self, flag: u16, enable: bool) {
        self.group.set_group_state(flag, enable);
    }

    fn valid(&mut self, command: u16) -> bool {
        self.group.valid(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divisors() {
        assert_eq!(most_equal_divisors(4, true), (2, 2));
        assert_eq!(most_equal_divisors(3, true), (1, 3));
        assert_eq!(most_equal_divisors(6, true), (2, 3));
    }
}
