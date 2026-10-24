//! Работа с текстом: ширина символов на экране и «C-строки» с горячими клавишами.
//!
//! В tvision строки вида `"~F~ile"` означают, что буква между тильдами
//! подсвечивается и служит горячей клавишей (Alt+F). Здесь — порт функций
//! `cstrlen()`, `hotKey()` и `strwidth()` из `source/tvision/` и `ttext.cpp`.

/// Ширина символа в ячейках терминала: 0 (комбинирующие/управляющие),
/// 1 (обычные) или 2 (широкие — иероглифы, эмодзи).
///
/// Это упрощённая замена `wcwidth()`: оригинальный tvision использует
/// системную функцию или свою таблицу (`platform/winwidth.cpp`).
pub fn char_width(c: char) -> usize {
    let u = c as u32;
    // Управляющие символы — нулевая ширина (на экран мы их не выводим).
    if u < 0x20 || (0x7f..0xa0).contains(&u) {
        return 0;
    }
    // Комбинирующие диакритические знаки и символы нулевой ширины.
    if (0x0300..=0x036f).contains(&u)
        || (0x0483..=0x0489).contains(&u)
        || (0x0591..=0x05bd).contains(&u)
        || (0x1ab0..=0x1aff).contains(&u)
        || (0x1dc0..=0x1dff).contains(&u)
        || (0x200b..=0x200f).contains(&u)
        || (0x20d0..=0x20ff).contains(&u)
        || (0xfe00..=0xfe0f).contains(&u)
        || (0xfe20..=0xfe2f).contains(&u)
    {
        return 0;
    }
    // Широкие символы Восточной Азии и эмодзи.
    if (0x1100..=0x115f).contains(&u)
        || (0x2e80..=0x303e).contains(&u)
        || (0x3041..=0x33ff).contains(&u)
        || (0x3400..=0x4dbf).contains(&u)
        || (0x4e00..=0x9fff).contains(&u)
        || (0xa000..=0xa4cf).contains(&u)
        || (0xac00..=0xd7a3).contains(&u)
        || (0xf900..=0xfaff).contains(&u)
        || (0xfe30..=0xfe4f).contains(&u)
        || (0xff00..=0xff60).contains(&u)
        || (0xffe0..=0xffe6).contains(&u)
        || (0x1f300..=0x1f64f).contains(&u)
        || (0x1f900..=0x1f9ff).contains(&u)
        || (0x20000..=0x3fffd).contains(&u)
    {
        return 2;
    }
    1
}

/// Ширина строки на экране (аналог `strwidth()`).
pub fn str_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

/// Ширина «C-строки» без тильд (аналог `cstrlen()`).
/// `"~O~pen"` → 4.
pub fn cstr_width(s: &str) -> usize {
    s.chars().filter(|&c| c != '~').map(char_width).sum()
}

/// Горячая клавиша C-строки — первый символ после `~`, в верхнем регистре
/// (аналог `hotKey()`). `"E~x~it"` → `Some('X')`.
pub fn hotkey(s: &str) -> Option<char> {
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '~' {
            return it.next().map(to_upper);
        }
    }
    None
}

/// Перевод символа в верхний регистр (с поддержкой кириллицы).
pub fn to_upper(c: char) -> char {
    c.to_uppercase().next().unwrap_or(c)
}

/// Пары «латинская буква — русская буква на той же клавише» (US QWERTY и
/// русская раскладка ЙЦУКЕН).
const JCUKEN: [(char, char); 33] = [
    ('Q', 'Й'),
    ('W', 'Ц'),
    ('E', 'У'),
    ('R', 'К'),
    ('T', 'Е'),
    ('Y', 'Н'),
    ('U', 'Г'),
    ('I', 'Ш'),
    ('O', 'Щ'),
    ('P', 'З'),
    ('[', 'Х'),
    (']', 'Ъ'),
    ('A', 'Ф'),
    ('S', 'Ы'),
    ('D', 'В'),
    ('F', 'А'),
    ('G', 'П'),
    ('H', 'Р'),
    ('J', 'О'),
    ('K', 'Л'),
    ('L', 'Д'),
    (';', 'Ж'),
    ('\'', 'Э'),
    ('Z', 'Я'),
    ('X', 'Ч'),
    ('C', 'С'),
    ('V', 'М'),
    ('B', 'И'),
    ('N', 'Т'),
    ('M', 'Ь'),
    (',', 'Б'),
    ('.', 'Ю'),
    ('`', 'Ё'),
];

/// Совпадает ли нажатая буква с горячей буквой без учёта регистра и
/// раскладки: Alt+буква в Windows приходит латинской (по коду клавиши), а
/// горячая буква может быть русской — `A` подходит к `Ф`, и наоборот.
pub fn same_hotkey(pressed: char, hot: char) -> bool {
    let (p, h) = (to_upper(pressed), to_upper(hot));
    p == h || JCUKEN.iter().any(|&(l, r)| (p == l && h == r) || (p == r && h == l))
}

/// Убирает тильды из C-строки: `"~F~ile"` → `"File"`.
pub fn strip_tildes(s: &str) -> String {
    s.chars().filter(|&c| c != '~').collect()
}

/// Обрезает строку так, чтобы её экранная ширина не превышала `max`.
pub fn truncate_width(s: &str, max: usize) -> &str {
    let mut w = 0;
    for (i, c) in s.char_indices() {
        let cw = char_width(c);
        if w + cw > max {
            return &s[..i];
        }
        w += cw;
    }
    s
}

/// Разбивает текст на строки по ширине `width` с переносом по словам.
/// Используется в [`crate::StaticText`] (аналог логики `TStaticText::draw`).
/// Символ `\x03` в начале строки (как в оригинале) означает центрирование —
/// это обрабатывается вызывающим кодом.
pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    if width == 0 {
        return lines;
    }
    for para in text.split('\n') {
        let mut line = String::new();
        let mut line_w = 0;
        for word in para.split(' ') {
            let ww = str_width(word);
            // Слово не помещается в текущую строку — переносим.
            if line_w > 0 && line_w + 1 + ww > width {
                lines.push(std::mem::take(&mut line));
                line_w = 0;
            }
            if line_w > 0 {
                line.push(' ');
                line_w += 1;
            }
            // Очень длинное слово режем принудительно.
            let mut rest = word;
            while line_w == 0 && str_width(rest) > width {
                let head = truncate_width(rest, width);
                if head.is_empty() {
                    break;
                }
                lines.push(head.to_string());
                rest = &rest[head.len()..];
            }
            line.push_str(rest);
            line_w += str_width(rest);
        }
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cstr() {
        assert_eq!(cstr_width("~O~pen"), 4);
        assert_eq!(hotkey("E~x~it"), Some('X'));
        assert_eq!(hotkey("Файл"), None);
        assert_eq!(hotkey("~ф~айл"), Some('Ф'));
    }

    #[test]
    fn widths() {
        assert_eq!(str_width("abc"), 3);
        assert_eq!(str_width("Привет"), 6);
        assert_eq!(str_width("日本"), 4);
    }

    #[test]
    fn wrap() {
        let l = wrap_text("один два три четыре", 9);
        assert_eq!(l, vec!["один два", "три", "четыре"]);
    }
}

#[cfg(test)]
mod hotkey_layout_tests {
    use super::*;

    #[test]
    fn layout_insensitive() {
        assert!(same_hotkey('a', 'Ф'));
        assert!(same_hotkey('ф', 'A'));
        assert!(same_hotkey('ф', 'ф'));
        assert!(!same_hotkey('a', 'П'));
    }
}
