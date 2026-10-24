//! Ячейки экрана, буфер строки для отрисовки и экранный буфер.
//!
//! * [`Cell`] — аналог `TScreenCell`: символ + атрибут.
//! * [`DrawBuffer`] — аналог `TDrawBuffer`: строка ячеек, которую представление
//!   заполняет, а потом выводит методом `write_line`.
//! * [`ScreenBuffer`] — весь экран целиком (аналог `TScreen::screenBuffer`).

use crate::color::{Attr, AttrPair};
use crate::text::char_width;

/// Одна ячейка экрана.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    /// Символ. Для второй половины широкого символа (иероглифа) хранится `'\0'`.
    pub ch: char,
    /// Атрибут цвета BIOS.
    pub attr: Attr,
}

impl Cell {
    pub const fn new(ch: char, attr: Attr) -> Self {
        Cell { ch, attr }
    }

    /// Является ли ячейка «хвостом» широкого символа.
    pub fn is_wide_tail(&self) -> bool {
        self.ch == '\0'
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::new(' ', 0x07)
    }
}

/// Буфер одной строки (или нескольких строк одинакового содержания) для отрисовки.
///
/// Методы повторяют `TDrawBuffer`: `move_char`, `move_str`, `move_cstr`,
/// `move_buf`, `put_char`, `put_attribute`. Позиции и длины — в ячейках экрана.
#[derive(Clone, Debug)]
pub struct DrawBuffer {
    pub cells: Vec<Cell>,
}

impl DrawBuffer {
    /// Создаёт буфер заданной ширины, заполненный пробелами.
    pub fn new(width: usize) -> Self {
        DrawBuffer { cells: vec![Cell::default(); width.max(1)] }
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Заполняет `count` ячеек начиная с `indent` символом `ch` цвета `attr`.
    /// Если `ch == '\0'`, меняется только цвет; если `attr == 0` — только символ
    /// (так же, как в оригинальном `TDrawBuffer::moveChar`).
    pub fn move_char(&mut self, indent: usize, ch: char, attr: Attr, count: usize) {
        let end = (indent + count).min(self.cells.len());
        for c in self.cells.iter_mut().take(end).skip(indent) {
            if attr != 0 {
                c.attr = attr;
            }
            if ch != '\0' {
                c.ch = ch;
            }
        }
    }

    /// Пишет строку `s` цвета `attr` начиная с позиции `indent`.
    /// Возвращает ширину выведенного текста в ячейках.
    pub fn move_str(&mut self, indent: usize, s: &str, attr: Attr) -> usize {
        self.move_str_limited(indent, s, attr, usize::MAX)
    }

    /// Как `move_str`, но выводит не более `max_width` ячеек.
    pub fn move_str_limited(&mut self, indent: usize, s: &str, attr: Attr, max_width: usize) -> usize {
        let mut x = indent;
        let limit = indent.saturating_add(max_width).min(self.cells.len());
        for ch in s.chars() {
            let w = char_width(ch);
            if w == 0 {
                continue;
            }
            if x + w > limit {
                break;
            }
            self.cells[x] = Cell::new(ch, if attr != 0 { attr } else { self.cells[x].attr });
            if w == 2 {
                self.cells[x + 1] = Cell::new('\0', self.cells[x].attr);
            }
            x += w;
        }
        x - indent
    }

    /// Пишет C-строку: текст между `~` выводится цветом подсветки из пары
    /// `attrs`, остальной — обычным цветом. Пара — из `DrawCtx::get_color`,
    /// [`crate::color::pair`] или старой 16-битной записи `0xHHLL` (байты
    /// BIOS). Аналог `TDrawBuffer::moveCStr`.
    pub fn move_cstr(&mut self, indent: usize, s: &str, attrs: AttrPair) -> usize {
        let (lo, hi) = crate::color::unpair(attrs);
        let mut cur = lo;
        let mut x = indent;
        for ch in s.chars() {
            if ch == '~' {
                cur = if cur == lo { hi } else { lo };
                continue;
            }
            let w = char_width(ch);
            if w == 0 {
                continue;
            }
            if x + w > self.cells.len() {
                break;
            }
            self.cells[x] = Cell::new(ch, cur);
            if w == 2 {
                self.cells[x + 1] = Cell::new('\0', cur);
            }
            x += w;
        }
        x - indent
    }

    /// Копирует готовые ячейки (аналог `moveBuf`). Если `attr != 0`,
    /// цвет ячеек заменяется на `attr`.
    pub fn move_buf(&mut self, indent: usize, src: &[Cell], attr: Attr) {
        for (i, c) in src.iter().enumerate() {
            let x = indent + i;
            if x >= self.cells.len() {
                break;
            }
            self.cells[x] = Cell::new(c.ch, if attr != 0 { attr } else { c.attr });
        }
    }

    /// Ставит символ в позицию `indent`, не меняя цвет (аналог `putChar`).
    pub fn put_char(&mut self, indent: usize, ch: char) {
        if let Some(c) = self.cells.get_mut(indent) {
            c.ch = ch;
        }
    }

    /// Меняет цвет ячейки, не трогая символ (аналог `putAttribute`).
    pub fn put_attribute(&mut self, indent: usize, attr: Attr) {
        if let Some(c) = self.cells.get_mut(indent) {
            c.attr = attr;
        }
    }
}

/// Весь экран: прямоугольный массив ячеек.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenBuffer {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
}

impl ScreenBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        ScreenBuffer { width, height, cells: vec![Cell::default(); width * height] }
    }

    /// Ячейка по координатам (без проверки выхода за границы — паникует).
    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[y * self.width + x]
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.cells[y * self.width + x]
    }

    /// Очищает экран.
    pub fn clear(&mut self, attr: Attr) {
        for c in &mut self.cells {
            *c = Cell::new(' ', attr);
        }
    }

    /// Текст одной строки (для тестов и отладки). Хвосты широких символов пропускаются.
    pub fn line_text(&self, y: usize) -> String {
        (0..self.width).map(|x| self.get(x, y)).filter(|c| !c.is_wide_tail()).map(|c| c.ch).collect()
    }

    /// Весь экран как текст (строки через `\n`). Удобно для тестов.
    pub fn to_text(&self) -> String {
        (0..self.height).map(|y| self.line_text(y)).collect::<Vec<_>>().join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cstr_colors() {
        let mut b = DrawBuffer::new(10);
        let n = b.move_cstr(0, "~F~ile", 0x7470);
        assert_eq!(n, 4);
        assert_eq!(b.cells[0], Cell::new('F', 0x74));
        assert_eq!(b.cells[1], Cell::new('i', 0x70));
    }

    #[test]
    fn wide() {
        let mut b = DrawBuffer::new(5);
        let n = b.move_str(0, "日本語", 0x07);
        // Третий иероглиф не помещается (нужно 6 ячеек).
        assert_eq!(n, 4);
        assert!(b.cells[1].is_wide_tail());
    }
}
