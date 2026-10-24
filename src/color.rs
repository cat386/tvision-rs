//! Цвета и палитры.
//!
//! ## Атрибут
//!
//! В оригинальном Turbo Vision атрибут ячейки — один байт BIOS: младшие
//! 4 бита — цвет текста, старшие 4 бита — цвет фона. Цвета BIOS:
//! 0 чёрный, 1 синий, 2 зелёный, 3 бирюзовый, 4 красный, 5 малиновый,
//! 6 коричневый, 7 светло-серый, 8–15 — их яркие варианты.
//!
//! Здесь атрибут — 32-битное число ([`Attr`]), в котором байт BIOS остался
//! на своём месте, а выше добавлены цвета из **256-цветной палитры xterm**
//! (как `TColorAttr` с `TColorXTerm` в magiblot/tvision):
//!
//! | Биты | Что хранят |
//! |------|-----------|
//! | 0–3 | цвет текста BIOS (0–15) |
//! | 4–7 | цвет фона BIOS (0–15) |
//! | 8–15 | номер цвета текста xterm (0–255), если поднят флаг [`FG_256`] |
//! | 16–23 | номер цвета фона xterm (0–255), если поднят флаг [`BG_256`] |
//! | 24 | [`FG_256`] — текст задан номером xterm |
//! | 25 | [`BG_256`] — фон задан номером xterm |
//!
//! Поэтому:
//! * старые атрибуты-байты (`0x1E`, `0x70`, палитры из C++) работают как
//!   раньше — это просто числа меньше 256;
//! * у 256-цветного атрибута младший байт всегда хранит **ближайший цвет
//!   BIOS**: его показывает терминал, который умеет только 16 цветов, и его
//!   видит старый код, читающий `attr & 0x0F` / `attr >> 4`;
//! * текст и фон независимы: можно взять текст BIOS и фон xterm.
//!
//! Создавать атрибуты удобнее функциями: [`attr`] (BIOS), [`attr256`]
//! (xterm), [`with_fg256`], [`with_bg256`], [`rgb256`] (ближайший к RGB).
//!
//! ## Палитры
//!
//! Каждое представление имеет **палитру** — массив индексов, который
//! переводит «логический» номер цвета представления в номер цвета владельца.
//! Цепочка палитр заканчивается палитрой приложения ([`palettes::APP_COLOR`],
//! `Program::palette`), которая уже содержит атрибуты — в том числе
//! 256-цветные. Подробнее — в [`crate::DrawCtx::map_color`].

use std::sync::atomic::{AtomicU8, Ordering};

/// Атрибут ячейки: байт BIOS (фон << 4 | текст) плюс, по желанию, номера
/// цветов xterm-256 (см. описание модуля). Аналог `TColorAttr`.
pub type Attr = u32;

/// Пара атрибутов: младшие 32 бита — обычный цвет, старшие — цвет
/// подсвеченного текста между `~...~`. Аналог `TAttrPair`.
///
/// Для совместимости `DrawBuffer::move_cstr` понимает и старую 16-битную
/// запись пары BIOS: `0x7470` = обычный `0x70`, подсветка `0x74`.
pub type AttrPair = u64;

/// Флаг: цвет текста задан номером xterm-256 (биты 8–15).
pub const FG_256: Attr = 1 << 24;
/// Флаг: цвет фона задан номером xterm-256 (биты 16–23).
pub const BG_256: Attr = 1 << 25;

/// Атрибут ошибки палитры (мигающий белый на красном в оригинале).
pub const ERROR_ATTR: Attr = 0xCF;

/// Атрибут тени под окнами (тёмно-серый на чёрном), как `shadowAttr`.
pub const SHADOW_ATTR: Attr = 0x08;

/// Тень при 256-цветной палитре приложения (фон рабочего стола задан
/// номером xterm): тёмно-серый текст на почти чёрном фоне.
pub const SHADOW_ATTR_256: Attr = attr256(240, 232);

/// Атрибут BIOS из цвета текста и фона (0–15).
pub const fn attr(fg: u8, bg: u8) -> Attr {
    (((bg & 0x0F) as Attr) << 4) | (fg & 0x0F) as Attr
}

/// Атрибут из номеров цветов xterm-256 для текста и фона.
///
/// ```
/// use tvision::color::*;
/// let a = attr256(214, 17); // оранжевый на тёмно-синем
/// assert_eq!((fg256(a), bg256(a)), (214, 17));
/// assert_eq!(back(a), 1); // для 16-цветного терминала — синий фон BIOS
/// ```
pub const fn attr256(fg: u8, bg: u8) -> Attr {
    with_bg256(with_fg256(0, fg), bg)
}

/// Тот же атрибут с цветом текста xterm `fg` (фон не меняется).
pub const fn with_fg256(a: Attr, fg: u8) -> Attr {
    (a & !(0xFF << 8) & !0x0F) | FG_256 | ((fg as Attr) << 8) | xterm_to_bios(fg) as Attr
}

/// Тот же атрибут с цветом фона xterm `bg` (текст не меняется).
pub const fn with_bg256(a: Attr, bg: u8) -> Attr {
    (a & !(0xFF << 16) & !0xF0) | BG_256 | ((bg as Attr) << 16) | ((xterm_to_bios(bg) as Attr) << 4)
}

/// Тот же атрибут с цветом текста BIOS `fg` (0–15); флаг xterm снимается.
pub const fn with_fore(a: Attr, fg: u8) -> Attr {
    (a & !(0xFF << 8) & !FG_256 & !0x0F) | (fg & 0x0F) as Attr
}

/// Тот же атрибут с цветом фона BIOS `bg` (0–15); флаг xterm снимается.
pub const fn with_back(a: Attr, bg: u8) -> Attr {
    (a & !(0xFF << 16) & !BG_256 & !0xF0) | (((bg & 0x0F) as Attr) << 4)
}

/// Атрибут из цветов RGB (`0xRRGGBB`): берутся ближайшие цвета xterm-256.
pub const fn rgb256(fg: u32, bg: u32) -> Attr {
    attr256(rgb_to_xterm(fg), rgb_to_xterm(bg))
}

/// Цвет текста BIOS (0–15). Для 256-цветного текста — ближайший цвет BIOS.
pub const fn fore(a: Attr) -> u8 {
    (a & 0x0F) as u8
}

/// Цвет фона BIOS (0–15). Для 256-цветного фона — ближайший цвет BIOS.
pub const fn back(a: Attr) -> u8 {
    ((a >> 4) & 0x0F) as u8
}

/// Байт BIOS атрибута (то, что увидит 16-цветный терминал).
pub const fn bios(a: Attr) -> u8 {
    (a & 0xFF) as u8
}

/// Задан ли текст номером xterm-256.
pub const fn is_fg256(a: Attr) -> bool {
    a & FG_256 != 0
}

/// Задан ли фон номером xterm-256.
pub const fn is_bg256(a: Attr) -> bool {
    a & BG_256 != 0
}

/// Номер цвета текста в палитре xterm-256 (для цвета BIOS — 0–15 в
/// порядке ANSI).
pub const fn fg256(a: Attr) -> u8 {
    if is_fg256(a) {
        ((a >> 8) & 0xFF) as u8
    } else {
        bios_to_ansi(fore(a))
    }
}

/// Номер цвета фона в палитре xterm-256 (для цвета BIOS — 0–15 в порядке ANSI).
pub const fn bg256(a: Attr) -> u8 {
    if is_bg256(a) {
        ((a >> 16) & 0xFF) as u8
    } else {
        bios_to_ansi(back(a))
    }
}

/// Перевод номера цвета BIOS в номер цвета ANSI (0–15).
/// В BIOS синий = 1, красный = 4, а в ANSI наоборот.
pub const fn bios_to_ansi(c: u8) -> u8 {
    const MAP: [u8; 8] = [0, 4, 2, 6, 1, 5, 3, 7];
    MAP[(c & 7) as usize] | (c & 8)
}

/// Обратный перевод: номер ANSI (0–15) → номер BIOS.
pub const fn ansi_to_bios(c: u8) -> u8 {
    // Перестановка симметрична: синий ↔ красный, бирюзовый ↔ коричневый.
    bios_to_ansi(c)
}

/// Цвета BIOS в RGB (`0xRRGGBB`), как у VGA. Порядок — BIOS.
pub const VGA_RGB: [u32; 16] = [
    0x000000, 0x0000AA, 0x00AA00, 0x00AAAA, 0xAA0000, 0xAA00AA, 0xAA5500, 0xAAAAAA, //
    0x555555, 0x5555FF, 0x55FF55, 0x55FFFF, 0xFF5555, 0xFF55FF, 0xFFFF55, 0xFFFFFF,
];

/// 16 цветов терминальной темы gruvbox (тёмной) в RGB, порядок — BIOS:
/// так выглядят цвета BIOS в окне tvision-gui по умолчанию.
pub const GRUVBOX_RGB: [u32; 16] = [
    0x282828, 0x458588, 0x98971A, 0x689D6A, 0xCC241D, 0xB16286, 0xD79921, 0xA89984, //
    0x928374, 0x83A598, 0xB8BB26, 0x8EC07C, 0xFB4934, 0xD3869B, 0xFABD2F, 0xEBDBB2,
];

/// Уровни яркости куба 6×6×6 палитры xterm (номера 16–231).
const CUBE: [u32; 6] = [0, 95, 135, 175, 215, 255];

/// Цвет xterm-256 в RGB (`0xRRGGBB`).
///
/// * 0–15 — 16 основных цветов (здесь — цвета VGA; настоящий терминал
///   показывает их цветами своей темы);
/// * 16–231 — куб 6×6×6: `16 + 36·r + 6·g + b`;
/// * 232–255 — 24 оттенка серого от `#080808` до `#EEEEEE`.
pub const fn xterm_rgb(idx: u8) -> u32 {
    if idx < 16 {
        VGA_RGB[ansi_to_bios(idx) as usize]
    } else if idx < 232 {
        let i = idx as usize - 16;
        (CUBE[i / 36] << 16) | (CUBE[(i / 6) % 6] << 8) | CUBE[i % 6]
    } else {
        let v = 8 + 10 * (idx as u32 - 232);
        (v << 16) | (v << 8) | v
    }
}

/// Квадрат расстояния между цветами RGB с поправкой на чувствительность глаза.
const fn dist(a: u32, b: u32) -> u32 {
    let dr = ((a >> 16) & 0xFF) as i32 - ((b >> 16) & 0xFF) as i32;
    let dg = ((a >> 8) & 0xFF) as i32 - ((b >> 8) & 0xFF) as i32;
    let db = (a & 0xFF) as i32 - (b & 0xFF) as i32;
    (3 * dr * dr + 4 * dg * dg + 2 * db * db) as u32
}

/// Ближайший уровень куба (0–5) к яркости канала `v`.
const fn cube_level(v: u32) -> u32 {
    if v < 48 {
        0
    } else if v < 115 {
        1
    } else {
        (v - 35) / 40
    }
}

/// Ближайший к RGB цвет из палитры xterm-256 (куб или оттенок серого).
pub const fn rgb_to_xterm(rgb: u32) -> u8 {
    let (r, g, b) = ((rgb >> 16) & 0xFF, (rgb >> 8) & 0xFF, rgb & 0xFF);
    let (cr, cg, cb) = (cube_level(r), cube_level(g), cube_level(b));
    let cube_idx = 16 + 36 * cr + 6 * cg + cb;
    let avg = (r + g + b) / 3;
    let gray_idx = if avg < 8 {
        232
    } else if avg > 238 {
        255
    } else {
        232 + (avg - 8 + 5) / 10
    };
    let gi = if gray_idx > 255 { 255 } else { gray_idx };
    if dist(rgb, xterm_rgb(gi as u8)) < dist(rgb, xterm_rgb(cube_idx as u8)) {
        gi as u8
    } else {
        cube_idx as u8
    }
}

/// Ближайший цвет BIOS (0–15) к цвету xterm `idx`.
pub const fn xterm_to_bios(idx: u8) -> u8 {
    if idx < 16 {
        return ansi_to_bios(idx);
    }
    let rgb = xterm_rgb(idx);
    let mut best = 0;
    let mut best_d = u32::MAX;
    let mut i = 0;
    while i < 16 {
        let d = dist(rgb, VGA_RGB[i]);
        if d < best_d {
            best_d = d;
            best = i;
        }
        i += 1;
    }
    best as u8
}

/// Контрастный к цвету xterm `idx` цвет текста: чёрный (16) или белый
/// (231) из куба — они не зависят от темы терминала.
pub const fn contrast256(idx: u8) -> u8 {
    let rgb = xterm_rgb(idx);
    let (r, g, b) = ((rgb >> 16) & 0xFF, (rgb >> 8) & 0xFF, rgb & 0xFF);
    if r * 299 + g * 587 + b * 114 > 128_000 {
        16
    } else {
        231
    }
}

/// Цвет текста атрибута в RGB. `base` — RGB 16 цветов BIOS (в порядке BIOS):
/// ими рисуются цвета BIOS и номера xterm 0–15. Нужна графическим
/// «экранам» (tvision-gui).
pub const fn fg_rgb(a: Attr, base: &[u32; 16]) -> u32 {
    let i = fg256(a);
    if i < 16 {
        base[ansi_to_bios(i) as usize]
    } else {
        xterm_rgb(i)
    }
}

/// Цвет фона атрибута в RGB (см. [`fg_rgb`]).
pub const fn bg_rgb(a: Attr, base: &[u32; 16]) -> u32 {
    let i = bg256(a);
    if i < 16 {
        base[ansi_to_bios(i) as usize]
    } else {
        xterm_rgb(i)
    }
}

/// Атрибут, сведённый к 16 цветам BIOS (флаги xterm сброшены).
pub const fn to_bios(a: Attr) -> Attr {
    a & 0xFF
}

/// Тот же атрибут с чуть отличающимся фоном — для «зебры» в таблицах и
/// списках. Для фона BIOS переключается бит яркости фона (как `attr ^ 0x80`);
/// для фона xterm берётся соседний более светлый (или, у светлых, более
/// тёмный) цвет.
pub const fn shade_bg(a: Attr) -> Attr {
    if !is_bg256(a) {
        return a ^ 0x80;
    }
    let i = bg256(a);
    let n = if i < 16 {
        i ^ 8
    } else if i >= 232 {
        if i <= 251 {
            i + 2
        } else {
            i - 2
        }
    } else {
        let rgb = xterm_rgb(i);
        let (r, g, b) = ((rgb >> 16) & 0xFF, (rgb >> 8) & 0xFF, rgb & 0xFF);
        let light = r + g + b > 3 * 160;
        let m = rgb_to_xterm((shade_step(r, light) << 16) | (shade_step(g, light) << 8) | shade_step(b, light));
        if m == i {
            if light {
                252
            } else {
                236
            }
        } else {
            m
        }
    };
    with_bg256(a, n)
}

/// Шаг яркости канала для [`shade_bg`]: светлые темнеют, тёмные светлеют.
const fn shade_step(v: u32, light: bool) -> u32 {
    if light {
        v.saturating_sub(30)
    } else if v < 225 {
        v + 30
    } else {
        255
    }
}

/// Собрать пару атрибутов (обычный, подсветка) для `move_cstr`.
pub const fn pair(normal: Attr, highlight: Attr) -> AttrPair {
    normal as AttrPair | ((highlight as AttrPair) << 32)
}

/// Разобрать пару атрибутов: (обычный, подсветка). Понимает и старую
/// 16-битную запись пары байтов BIOS (`0x7470`).
pub const fn unpair(p: AttrPair) -> (Attr, Attr) {
    if p <= 0xFFFF {
        ((p & 0xFF) as Attr, (p >> 8) as Attr)
    } else {
        ((p & 0xFFFF_FFFF) as Attr, (p >> 32) as Attr)
    }
}

/// Обычный цвет пары (аналог `TAttrPair` → младший байт).
pub const fn lo(p: AttrPair) -> Attr {
    unpair(p).0
}

/// Цвет подсветки пары (аналог `TAttrPair` → старший байт).
pub const fn hi(p: AttrPair) -> Attr {
    unpair(p).1
}

// ----------------------------------------------------------------------
// Возможности терминала
// ----------------------------------------------------------------------

/// Сколько цветов выводить в терминал.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    /// 16 цветов: `ESC[30–37m`, `ESC[90–97m`. Цвета xterm заменяются
    /// ближайшими цветами BIOS (младший байт атрибута).
    Ansi16,
    /// 256 цветов: цвета xterm выводятся как `ESC[38;5;Nm` / `ESC[48;5;Nm`,
    /// цвета BIOS — как раньше (цветами темы терминала).
    Xterm256,
}

/// 0 — определять автоматически, 1 — 16 цветов, 2 — 256 цветов.
static COLOR_MODE: AtomicU8 = AtomicU8::new(0);

/// Задать режим цвета для всех терминалов программы (`None` — определять
/// автоматически по переменным окружения, см. [`detect_color_mode`]).
/// Действует со следующей перерисовки.
pub fn set_color_mode(mode: Option<ColorMode>) {
    let v = match mode {
        None => 0,
        Some(ColorMode::Ansi16) => 1,
        Some(ColorMode::Xterm256) => 2,
    };
    COLOR_MODE.store(v, Ordering::Relaxed);
}

/// Текущий режим цвета: заданный [`set_color_mode`] или определённый
/// автоматически.
pub fn color_mode() -> ColorMode {
    match COLOR_MODE.load(Ordering::Relaxed) {
        1 => ColorMode::Ansi16,
        2 => ColorMode::Xterm256,
        _ => detect_color_mode(),
    }
}

/// Определить, умеет ли терминал 256 цветов, по переменным окружения.
///
/// 1. `TVISION_COLORS=16` или `256` — явный выбор пользователя;
/// 2. `COLORTERM=truecolor|24bit`, `TERM=*256color*|*direct*`, Windows
///    Terminal (`WT_SESSION`), известные `TERM_PROGRAM` — 256 цветов;
/// 3. `TERM=linux|dumb|vt*|cons25|ansi|screen|…` без «256» — 16 цветов;
/// 4. иначе (в том числе консоль Windows 10+) — 256 цветов.
pub fn detect_color_mode() -> ColorMode {
    let var = |n: &str| std::env::var(n).unwrap_or_default().to_ascii_lowercase();
    color_mode_from(&var("TVISION_COLORS"), &var("COLORTERM"), &var("TERM"), !var("WT_SESSION").is_empty(), &var("TERM_PROGRAM"))
}

/// Логика [`detect_color_mode`] без чтения окружения (для тестов).
pub fn color_mode_from(forced: &str, colorterm: &str, term: &str, wt: bool, term_program: &str) -> ColorMode {
    match forced.trim() {
        "16" | "8" | "ansi16" => return ColorMode::Ansi16,
        "256" | "xterm256" | "truecolor" | "24bit" => return ColorMode::Xterm256,
        _ => {}
    }
    if colorterm == "truecolor" || colorterm == "24bit" || term.contains("256") || term.contains("direct") || wt {
        return ColorMode::Xterm256;
    }
    if ["iterm.app", "apple_terminal", "vscode", "wezterm", "hyper", "tabby"].contains(&term_program) {
        return ColorMode::Xterm256;
    }
    let limited = ["linux", "dumb", "cons25", "ansi", "screen", "sun", "pcansi", "cygwin", "mach"];
    if limited.iter().any(|t| term == *t || term.starts_with(&format!("{}-", t)) || term.starts_with(&format!("{}.", t))) || term.starts_with("vt") {
        return ColorMode::Ansi16;
    }
    ColorMode::Xterm256
}

/// Перекрасить палитру: каждый цвет BIOS (0–15) заменяется цветом xterm
/// `map[цвет BIOS]`. Цвета, уже заданные номерами xterm, не меняются.
/// Так из классической палитры получается 256-цветная тема.
///
/// ```
/// use tvision::color::*;
/// // Синий BIOS (1) → xterm 24, светло-серый (7) → xterm 252, остальные — как были.
/// let mut map = [0u8; 16];
/// for i in 0..16 {
///     map[i] = bios_to_ansi(i as u8);
/// }
/// map[1] = 24;
/// map[7] = 252;
/// let pal = remap(palettes::APP_COLOR, &map);
/// assert_eq!(pal[0], attr256(24, 252)); // рабочий стол: было 0x71
/// ```
pub fn remap(palette: &[Attr], map: &[u8; 16]) -> Vec<Attr> {
    palette
        .iter()
        .map(|&a| {
            let mut r = a;
            if !is_fg256(a) {
                r = with_fg256(r, map[fore(a) as usize]);
            }
            if !is_bg256(a) {
                r = with_bg256(r, map[back(a) as usize]);
            }
            r
        })
        .collect()
}

/// Расширяет байтовую палитру C++ до массива атрибутов (на этапе компиляции).
const fn widen<const N: usize>(b: &[u8; N]) -> [Attr; N] {
    let mut a = [0; N];
    let mut i = 0;
    while i < N {
        a[i] = b[i] as Attr;
        i += 1;
    }
    a
}

/// Палитры. Байтовые строки перенесены из C++ без изменений
/// (`#define cpXXX "..."`), только без завершающего нуля.
pub mod palettes {
    use super::{widen, Attr};

    /// Палитра приложения в цвете (`cpAppColor`, 135 атрибутов).
    pub const APP_COLOR: &[Attr] = &widen(APP_COLOR_BYTES);
    /// Чёрно-белая палитра приложения (`cpAppBlackWhite`).
    pub const APP_BLACK_WHITE: &[Attr] = &widen(APP_BLACK_WHITE_BYTES);
    /// Монохромная палитра приложения (`cpAppMonochrome`).
    pub const APP_MONOCHROME: &[Attr] = &widen(APP_MONOCHROME_BYTES);

    /// Цвета gruvbox (номера палитры xterm-256 из `colors/gruvbox.vim`
    /// темы <https://github.com/morhetz/gruvbox>): фоны `BG0H`…`BG4`,
    /// серый, тексты `FG4`…`FG0`, яркие и приглушённые (`N…`) цвета.
    pub mod gruvbox {
        pub const BG0H: u8 = 234;
        pub const BG0: u8 = 235;
        pub const BG1: u8 = 237;
        pub const BG2: u8 = 239;
        pub const BG3: u8 = 241;
        pub const BG4: u8 = 243;
        pub const GRAY: u8 = 245;
        pub const FG4: u8 = 246;
        pub const FG3: u8 = 248;
        pub const FG2: u8 = 250;
        pub const FG1: u8 = 223;
        pub const FG0: u8 = 229;
        pub const RED: u8 = 167;
        pub const GREEN: u8 = 142;
        pub const YELLOW: u8 = 214;
        pub const BLUE: u8 = 109;
        pub const PURPLE: u8 = 175;
        pub const AQUA: u8 = 108;
        pub const ORANGE: u8 = 208;
        pub const NRED: u8 = 124;
        pub const NGREEN: u8 = 106;
        pub const NYELLOW: u8 = 172;
        pub const NBLUE: u8 = 66;
        pub const NPURPLE: u8 = 132;
        pub const NAQUA: u8 = 72;
        pub const NORANGE: u8 = 166;
    }

    /// Палитра приложения в цветах **gruvbox** (тёмная) — палитра по
    /// умолчанию. Номера элементов те же, что у [`APP_COLOR`].
    pub const GRUVBOX: &[Attr] = &{
        use gruvbox::*;
        const fn g(fg: u8, bg: u8) -> Attr {
            super::attr256(fg, bg)
        }
        [
            g(BG1, BG0H),    // 1: рабочий стол (узор ░)
            g(FG1, BG1),     // 2: меню: обычный пункт
            g(GRAY, BG1),    // 3: меню: недоступный
            g(ORANGE, BG1),  // 4: меню: горячая буква
            g(BG0, YELLOW),  // 5: меню: выбранный
            g(BG3, YELLOW),  // 6: меню: выбранный недоступный
            g(NRED, YELLOW), // 7: меню: выбранная горячая буква
            g(BG4, BG0),     // 8: синее окно: рамка неактивного
            g(FG1, BG0),     // 9: синее окно: рамка активного
            g(GREEN, BG0),   // 10: синее окно: значки рамки
            g(BG3, BG1),     // 11: синее окно: поле полосы прокрутки
            g(FG4, BG1),     // 12: синее окно: значки полосы
            g(FG1, BG0),     // 13: синее окно: текст
            g(BG0, BLUE),    // 14: синее окно: выделенный текст
            g(FG0, BG0),     // 15: синее окно: резерв
            g(BG4, BG1),     // 16: бирюзовое окно: рамка неактивного
            g(FG1, BG1),     // 17: бирюзовое окно: рамка активного
            g(AQUA, BG1),    // 18: бирюзовое окно: значки рамки
            g(BG3, BG2),     // 19: бирюзовое окно: поле полосы прокрутки
            g(FG4, BG2),     // 20: бирюзовое окно: значки полосы
            g(FG1, BG1),     // 21: бирюзовое окно: текст
            g(BG0, AQUA),    // 22: бирюзовое окно: выделенный текст
            g(FG0, BG1),     // 23: бирюзовое окно: резерв
            g(BG4, BG2),     // 24: серое окно: рамка неактивного
            g(FG1, BG2),     // 25: серое окно: рамка активного
            g(YELLOW, BG2),  // 26: серое окно: значки рамки
            g(BG3, BG1),     // 27: серое окно: поле полосы прокрутки
            g(FG4, BG1),     // 28: серое окно: значки полосы
            g(FG1, BG2),     // 29: серое окно: текст
            g(BG0, YELLOW),  // 30: серое окно: выделенный текст
            g(FG0, BG2),     // 31: серое окно: резерв
            g(BG4, BG1),     // 32: серый диалог: рамка неактивная
            g(FG1, BG1),     // 33: серый диалог: рамка активная
            g(GREEN, BG1),   // 34: серый диалог: значки рамки
            g(BG3, BG2),     // 35: серый диалог: поле полосы прокрутки
            g(FG4, BG2),     // 36: серый диалог: значки полосы
            g(FG1, BG1),     // 37: серый диалог: статический текст
            g(FG2, BG1),     // 38: серый диалог: метка
            g(YELLOW, BG1),  // 39: серый диалог: метка подсвеченная
            g(ORANGE, BG1),  // 40: серый диалог: метка: горячая буква
            g(FG1, BG3),     // 41: серый диалог: кнопка
            g(AQUA, BG3),    // 42: серый диалог: кнопка по умолчанию
            g(YELLOW, BG3),  // 43: серый диалог: кнопка выбранная
            g(GRAY, BG2),    // 44: серый диалог: кнопка недоступная
            g(ORANGE, BG3),  // 45: серый диалог: кнопка: горячая буква
            g(BG0H, BG1),    // 46: серый диалог: тень кнопки
            g(FG2, BG1),     // 47: серый диалог: флажки
            g(YELLOW, BG1),  // 48: серый диалог: флажки: выбранный
            g(ORANGE, BG1),  // 49: серый диалог: флажки: горячая буква
            g(FG1, BG2),     // 50: серый диалог: строка ввода
            g(BG0, AQUA),    // 51: серый диалог: строка ввода: выделение
            g(GREEN, BG2),   // 52: серый диалог: строка ввода: стрелки
            g(BG0, GREEN),   // 53: серый диалог: значок истории
            g(GREEN, BG1),   // 54: серый диалог: значок истории: края
            g(BG3, BG2),     // 55: серый диалог: полоса окна истории
            g(FG4, BG2),     // 56: серый диалог: значки полосы истории
            g(FG1, BG2),     // 57: серый диалог: список
            g(BG0, BLUE),    // 58: серый диалог: список: в фокусе
            g(YELLOW, BG2),  // 59: серый диалог: список: выбранный
            g(BG3, BG2),     // 60: серый диалог: список: разделитель
            g(AQUA, BG2),    // 61: серый диалог: панель сведений
            g(GRAY, BG1),    // 62: серый диалог: флажки: недоступный
            g(FG1, BG1),     // 63: серый диалог: резерв
            g(BG4, BG0),     // 64: синий диалог: рамка неактивная
            g(FG1, BG0),     // 65: синий диалог: рамка активная
            g(GREEN, BG0),   // 66: синий диалог: значки рамки
            g(BG3, BG1),     // 67: синий диалог: поле полосы прокрутки
            g(FG4, BG1),     // 68: синий диалог: значки полосы
            g(FG1, BG0),     // 69: синий диалог: статический текст
            g(FG2, BG0),     // 70: синий диалог: метка
            g(YELLOW, BG0),  // 71: синий диалог: метка подсвеченная
            g(ORANGE, BG0),  // 72: синий диалог: метка: горячая буква
            g(FG1, BG3),     // 73: синий диалог: кнопка
            g(AQUA, BG3),    // 74: синий диалог: кнопка по умолчанию
            g(YELLOW, BG3),  // 75: синий диалог: кнопка выбранная
            g(GRAY, BG1),    // 76: синий диалог: кнопка недоступная
            g(ORANGE, BG3),  // 77: синий диалог: кнопка: горячая буква
            g(BG0H, BG0),    // 78: синий диалог: тень кнопки
            g(FG2, BG0),     // 79: синий диалог: флажки
            g(YELLOW, BG0),  // 80: синий диалог: флажки: выбранный
            g(ORANGE, BG0),  // 81: синий диалог: флажки: горячая буква
            g(FG1, BG1),     // 82: синий диалог: строка ввода
            g(BG0, AQUA),    // 83: синий диалог: строка ввода: выделение
            g(GREEN, BG1),   // 84: синий диалог: строка ввода: стрелки
            g(BG0, GREEN),   // 85: синий диалог: значок истории
            g(GREEN, BG0),   // 86: синий диалог: значок истории: края
            g(BG3, BG1),     // 87: синий диалог: полоса окна истории
            g(FG4, BG1),     // 88: синий диалог: значки полосы истории
            g(FG1, BG1),     // 89: синий диалог: список
            g(BG0, BLUE),    // 90: синий диалог: список: в фокусе
            g(YELLOW, BG1),  // 91: синий диалог: список: выбранный
            g(BG3, BG1),     // 92: синий диалог: список: разделитель
            g(AQUA, BG1),    // 93: синий диалог: панель сведений
            g(GRAY, BG0),    // 94: синий диалог: флажки: недоступный
            g(FG1, BG0),     // 95: синий диалог: резерв
            g(BG4, BG2),     // 96: бирюзовый диалог: рамка неактивная
            g(FG1, BG2),     // 97: бирюзовый диалог: рамка активная
            g(GREEN, BG2),   // 98: бирюзовый диалог: значки рамки
            g(BG3, BG3),     // 99: бирюзовый диалог: поле полосы прокрутки
            g(FG4, BG3),     // 100: бирюзовый диалог: значки полосы
            g(FG1, BG2),     // 101: бирюзовый диалог: статический текст
            g(FG2, BG2),     // 102: бирюзовый диалог: метка
            g(YELLOW, BG2),  // 103: бирюзовый диалог: метка подсвеченная
            g(ORANGE, BG2),  // 104: бирюзовый диалог: метка: горячая буква
            g(FG1, BG4),     // 105: бирюзовый диалог: кнопка
            g(AQUA, BG4),    // 106: бирюзовый диалог: кнопка по умолчанию
            g(YELLOW, BG4),  // 107: бирюзовый диалог: кнопка выбранная
            g(GRAY, BG3),    // 108: бирюзовый диалог: кнопка недоступная
            g(ORANGE, BG4),  // 109: бирюзовый диалог: кнопка: горячая буква
            g(BG0H, BG2),    // 110: бирюзовый диалог: тень кнопки
            g(FG2, BG2),     // 111: бирюзовый диалог: флажки
            g(YELLOW, BG2),  // 112: бирюзовый диалог: флажки: выбранный
            g(ORANGE, BG2),  // 113: бирюзовый диалог: флажки: горячая буква
            g(FG1, BG3),     // 114: бирюзовый диалог: строка ввода
            g(BG0, AQUA),    // 115: бирюзовый диалог: строка ввода: выделение
            g(GREEN, BG3),   // 116: бирюзовый диалог: строка ввода: стрелки
            g(BG0, GREEN),   // 117: бирюзовый диалог: значок истории
            g(GREEN, BG2),   // 118: бирюзовый диалог: значок истории: края
            g(BG3, BG3),     // 119: бирюзовый диалог: полоса окна истории
            g(FG4, BG3),     // 120: бирюзовый диалог: значки полосы истории
            g(FG1, BG3),     // 121: бирюзовый диалог: список
            g(BG0, BLUE),    // 122: бирюзовый диалог: список: в фокусе
            g(YELLOW, BG3),  // 123: бирюзовый диалог: список: выбранный
            g(BG3, BG3),     // 124: бирюзовый диалог: список: разделитель
            g(AQUA, BG3),    // 125: бирюзовый диалог: панель сведений
            g(GRAY, BG2),    // 126: бирюзовый диалог: флажки: недоступный
            g(FG1, BG2),     // 127: бирюзовый диалог: резерв
            g(BG4, BG1),     // 128: справка: рамка неактивная
            g(FG1, BG1),     // 129: справка: рамка активная
            g(AQUA, BG1),    // 130: справка: значки рамки
            g(BG3, BG2),     // 131: справка: поле полосы
            g(FG4, BG2),     // 132: справка: значки полосы
            g(FG1, BG1),     // 133: справка: текст
            g(YELLOW, BG1),  // 134: справка: ссылка
            g(BG0, YELLOW),  // 135: справка: выбранная ссылка
        ]
    };

    /// `cpAppColor` в виде байтов, как в C++.
    pub const APP_COLOR_BYTES: &[u8; 135] = b"\x71\x70\x78\x74\x20\x28\x24\x17\x1F\x1A\x31\x31\x1E\x71\x1F\
\x37\x3F\x3A\x13\x13\x3E\x21\x3F\x70\x7F\x7A\x13\x13\x70\x7F\x7E\
\x70\x7F\x7A\x13\x13\x70\x70\x7F\x7E\x20\x2B\x2F\x78\x2E\x70\x30\
\x3F\x3E\x1F\x2F\x1A\x20\x72\x31\x31\x30\x2F\x3E\x31\x13\x38\x00\
\x17\x1F\x1A\x71\x71\x1E\x17\x1F\x1E\x20\x2B\x2F\x78\x2E\x10\x30\
\x3F\x3E\x70\x2F\x7A\x20\x12\x31\x31\x30\x2F\x3E\x31\x13\x38\x00\
\x37\x3F\x3A\x13\x13\x3E\x30\x3F\x3E\x20\x2B\x2F\x78\x2E\x30\x70\
\x7F\x7E\x1F\x2F\x1A\x20\x32\x31\x71\x70\x2F\x7E\x71\x13\x78\x00\
\x37\x3F\x3A\x13\x13\x30\x3E\x1E";

    /// `cpAppBlackWhite` в виде байтов.
    pub const APP_BLACK_WHITE_BYTES: &[u8; 135] = b"\x70\x70\x78\x7F\x07\x07\x0F\x07\x0F\x07\x70\x70\x07\x70\x0F\
\x07\x0F\x07\x70\x70\x07\x70\x0F\x70\x7F\x7F\x70\x07\x70\x07\x0F\
\x70\x7F\x7F\x70\x07\x70\x70\x7F\x7F\x07\x0F\x0F\x78\x0F\x78\x07\
\x0F\x0F\x0F\x70\x0F\x07\x70\x70\x70\x07\x70\x0F\x07\x07\x08\x00\
\x07\x0F\x0F\x07\x70\x07\x07\x0F\x0F\x70\x78\x7F\x08\x7F\x08\x70\
\x7F\x7F\x7F\x0F\x70\x70\x07\x70\x70\x70\x07\x7F\x70\x07\x78\x00\
\x70\x7F\x7F\x70\x07\x70\x70\x7F\x7F\x07\x0F\x0F\x78\x0F\x78\x07\
\x0F\x0F\x0F\x70\x0F\x07\x70\x70\x70\x07\x70\x0F\x07\x07\x08\x00\
\x07\x0F\x07\x70\x70\x07\x0F\x70";

    /// `cpAppMonochrome` в виде байтов.
    pub const APP_MONOCHROME_BYTES: &[u8; 135] = b"\x70\x07\x07\x0F\x70\x70\x70\x07\x0F\x07\x70\x70\x07\x70\x00\
\x07\x0F\x07\x70\x70\x07\x70\x00\x70\x70\x70\x07\x07\x70\x07\x00\
\x70\x70\x70\x07\x07\x70\x70\x70\x0F\x07\x07\x0F\x70\x0F\x70\x07\
\x0F\x0F\x07\x70\x07\x07\x70\x07\x07\x07\x70\x0F\x07\x07\x70\x00\
\x70\x70\x70\x07\x07\x70\x70\x70\x0F\x07\x07\x0F\x70\x0F\x70\x07\
\x0F\x0F\x07\x70\x07\x07\x70\x07\x07\x07\x70\x0F\x07\x07\x01\x00\
\x70\x70\x70\x07\x07\x70\x70\x70\x0F\x07\x07\x0F\x70\x0F\x70\x07\
\x0F\x0F\x07\x70\x07\x07\x70\x07\x07\x07\x70\x0F\x07\x07\x01\x00\
\x07\x0F\x07\x70\x70\x07\x0F\x70";

    // --- Палитры окон (views.h) ---
    pub const BLUE_WINDOW: &[u8] = b"\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
    pub const CYAN_WINDOW: &[u8] = b"\x10\x11\x12\x13\x14\x15\x16\x17";
    pub const GRAY_WINDOW: &[u8] = b"\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F";

    // --- Палитры диалогов (dialogs.h) ---
    pub const GRAY_DIALOG: &[u8] = b"\x20\x21\x22\x23\x24\x25\x26\x27\x28\x29\x2A\x2B\x2C\x2D\x2E\x2F\
\x30\x31\x32\x33\x34\x35\x36\x37\x38\x39\x3A\x3B\x3C\x3D\x3E\x3F";
    pub const BLUE_DIALOG: &[u8] = b"\x40\x41\x42\x43\x44\x45\x46\x47\x48\x49\x4a\x4b\x4c\x4d\x4e\x4f\
\x50\x51\x52\x53\x54\x55\x56\x57\x58\x59\x5a\x5b\x5c\x5d\x5e\x5f";
    pub const CYAN_DIALOG: &[u8] = b"\x60\x61\x62\x63\x64\x65\x66\x67\x68\x69\x6a\x6b\x6c\x6d\x6e\x6f\
\x70\x71\x72\x73\x74\x75\x76\x77\x78\x79\x7a\x7b\x7c\x7d\x7e\x7f";

    // --- Палитры отдельных элементов (из соответствующих .cpp) ---
    pub const BACKGROUND: &[u8] = b"\x01";
    pub const FRAME: &[u8] = b"\x01\x01\x02\x02\x03";
    pub const MENU_VIEW: &[u8] = b"\x02\x03\x04\x05\x06\x07";
    pub const STATUS_LINE: &[u8] = b"\x02\x03\x04\x05\x06\x07";
    pub const BUTTON: &[u8] = b"\x0A\x0B\x0C\x0D\x0E\x0E\x0E\x0F";
    pub const CLUSTER: &[u8] = b"\x10\x11\x12\x12\x1f";
    pub const INPUT_LINE: &[u8] = b"\x13\x13\x14\x15";
    pub const LABEL: &[u8] = b"\x07\x08\x09\x09";
    pub const STATIC_TEXT: &[u8] = b"\x06";
    pub const LIST_VIEWER: &[u8] = b"\x1A\x1A\x1B\x1C\x1D";
    pub const SCROLL_BAR: &[u8] = b"\x04\x05\x05";
    pub const SCROLLER: &[u8] = b"\x06\x07";
    pub const HISTORY: &[u8] = b"\x16\x17";
    /// `cpHistoryWindow`, уже пересчитанная через палитру серого диалога:
    /// в C++ окно истории вставлялось в диалог, а здесь — на рабочий стол.
    pub const HISTORY_WINDOW: &[u8] = b"\x32\x32\x34\x37\x36\x32\x33";
    pub const OUTLINE_VIEWER: &[u8] = b"\x06\x07\x03\x08";
    pub const HELP_VIEWER: &[u8] = b"\x06\x07\x08";
    /// `cHelpWindow`: последние 8 цветов палитры приложения (128..135).
    pub const HELP_WINDOW: &[u8] = b"\x80\x81\x82\x83\x84\x85\x86\x87";
    pub const INFO_PANE: &[u8] = b"\x1E";
    pub const HISTORY_VIEWER: &[u8] = b"\x06\x06\x07\x06\x06";
    pub const INDICATOR: &[u8] = b"\x02\x03";
    pub const EDITOR: &[u8] = b"\x06\x07";
    pub const MEMO: &[u8] = b"\x1A\x1B";
}

#[cfg(test)]
mod tests {
    use super::palettes::*;
    use super::*;

    #[test]
    fn bios_attrs_unchanged() {
        // Старые атрибуты — просто числа до 255, как раньше.
        assert_eq!(attr(14, 1), 0x1E);
        assert_eq!((fore(0x1E), back(0x1E)), (14, 1));
        assert!(!is_fg256(0x1E) && !is_bg256(0x1E));
        // Номер xterm для цвета BIOS — в порядке ANSI: синий BIOS 1 = ANSI 4.
        assert_eq!((fg256(0x1E), bg256(0x1E)), (11, 4));
        assert_eq!(APP_COLOR[0], 0x71);
    }

    #[test]
    fn xterm_attrs() {
        let a = attr256(208, 236); // оранжевый на тёмно-сером
        assert!(is_fg256(a) && is_bg256(a));
        assert_eq!((fg256(a), bg256(a)), (208, 236));
        // Младший байт — ближайшие цвета BIOS.
        assert_eq!(back(a), 8); // #303030 ближе всего к тёмно-серому BIOS
        assert_eq!(fore(a), xterm_to_bios(208));
        // Замена одной части не трогает другую.
        let b = with_fore(a, 15);
        assert!(!is_fg256(b) && is_bg256(b));
        assert_eq!((fore(b), bg256(b)), (15, 236));
        let c = with_back(b, 1);
        assert_eq!(c, 0x1F);
        assert_eq!(to_bios(a), bios(a) as Attr);
    }

    #[test]
    fn xterm_rgb_table() {
        assert_eq!(xterm_rgb(16), 0x000000);
        assert_eq!(xterm_rgb(231), 0xFFFFFF);
        assert_eq!(xterm_rgb(196), 0xFF0000);
        assert_eq!(xterm_rgb(232), 0x080808);
        assert_eq!(xterm_rgb(255), 0xEEEEEE);
        assert_eq!(xterm_rgb(1), 0xAA0000); // ANSI 1 — красный
                                            // Обратный перевод для всех цветов куба и серых — точный.
        for i in 16..=255u8 {
            assert_eq!(xterm_rgb(rgb_to_xterm(xterm_rgb(i))), xterm_rgb(i), "цвет {}", i);
        }
        assert_eq!(rgb_to_xterm(0xFF8700), 208);
        assert_eq!(rgb256(0xFFFFFF, 0x000000), attr256(231, 16));
    }

    #[test]
    fn bios_approximation() {
        for i in 0..16u8 {
            assert_eq!(xterm_to_bios(i), ansi_to_bios(i));
        }
        assert_eq!(xterm_to_bios(196), 4); // #FF0000 → красный
        assert_eq!(xterm_to_bios(203), 12); // #FF5F5F → ярко-красный
        assert_eq!(xterm_to_bios(21), 1); // #0000FF → синий
        assert_eq!(xterm_to_bios(226), 14); // #FFFF00 → жёлтый
        assert_eq!(xterm_to_bios(16), 0);
        assert_eq!(xterm_to_bios(231), 15);
    }

    #[test]
    fn rgb_for_gui() {
        let vga = VGA_RGB;
        assert_eq!(fg_rgb(0x1E, &vga), 0xFFFF55);
        assert_eq!(bg_rgb(0x1E, &vga), 0x0000AA);
        let a = attr256(1, 208);
        assert_eq!(fg_rgb(a, &vga), 0xAA0000); // xterm 1 — цвет темы (из base)
        assert_eq!(bg_rgb(a, &vga), 0xFF8700);
    }

    #[test]
    fn pairs() {
        assert_eq!(unpair(0x7470), (0x70, 0x74)); // старая запись
        let p = pair(attr256(1, 2), 0x1E);
        assert_eq!(unpair(p), (attr256(1, 2), 0x1E));
        assert_eq!((lo(p), hi(p)), (attr256(1, 2), 0x1E));
        assert_eq!(unpair(pair(0x70, 0x74)), (0x70, 0x74));
    }

    #[test]
    fn shade() {
        assert_eq!(shade_bg(0x1E), 0x9E);
        let a = attr256(15, 236);
        assert_eq!(bg256(shade_bg(a)), 238);
        let b = attr256(0, 231); // белый куб — темнеет
        assert_ne!(bg256(shade_bg(b)), 231);
        let c = attr256(0, 17);
        assert_ne!(bg256(shade_bg(c)), 17);
        assert_eq!(fg256(shade_bg(c)), 0);
    }

    #[test]
    fn detect_mode() {
        use ColorMode::*;
        assert_eq!(color_mode_from("", "", "xterm-256color", false, ""), Xterm256);
        assert_eq!(color_mode_from("", "truecolor", "xterm", false, ""), Xterm256);
        assert_eq!(color_mode_from("", "", "linux", false, ""), Ansi16);
        assert_eq!(color_mode_from("", "", "screen", false, ""), Ansi16);
        assert_eq!(color_mode_from("", "", "screen-256color", false, ""), Xterm256);
        assert_eq!(color_mode_from("", "", "vt100", false, ""), Ansi16);
        assert_eq!(color_mode_from("", "", "", true, ""), Xterm256); // Windows Terminal
        assert_eq!(color_mode_from("", "", "", false, ""), Xterm256); // консоль Windows 10+
        assert_eq!(color_mode_from("16", "truecolor", "xterm-256color", false, ""), Ansi16);
        assert_eq!(color_mode_from("256", "", "linux", false, ""), Xterm256);
    }

    #[test]
    fn app_palette_sizes() {
        // В оригинале палитра приложения содержит ровно 135 цветов.
        assert_eq!(APP_COLOR.len(), 135);
        assert_eq!(APP_BLACK_WHITE.len(), 135);
        assert_eq!(APP_MONOCHROME.len(), 135);
        assert_eq!(GRAY_DIALOG.len(), 32);
    }
}
