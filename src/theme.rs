//! Темы: палитра приложения в отдельном файле `.tvtheme`.
//!
//! Тема — это 135 цветов палитры приложения (рабочий стол, меню, окна,
//! диалоги, справка), записанные понятным текстом:
//!
//! ```json
//! {
//!   "tvtheme": 1,
//!   "name": "Морская",
//!   "description": "Тёмно-синяя с бирюзой",
//!   "base": "classic",
//!   "colors": [
//!     {"index": 1, "item": "Рабочий стол: фон", "color": "cyan/blue"},
//!     {"index": 33, "item": "Серый диалог: рамка и фон", "color": "white/cyan"},
//!     {"index": 42, "item": "Серый диалог: кнопка по умолчанию", "color": "#ffd75f/#005f87"}
//!   ]
//! }
//! ```
//!
//! * `base` — с какой встроенной палитры начать: `classic`, `gruvbox`,
//!   `blackwhite`, `monochrome`; в `colors` — только отличия от неё.
//! * Цвет — `текст/фон`, каждый: имя цвета BIOS (`white`, `lightblue`…),
//!   номер цвета xterm-256 (`214`) или `#rrggbb` (берётся ближайший из 256).
//!
//! Подключение в программе:
//!
//! ```no_run
//! # fn f(app: &mut tvision::Program) -> Result<(), String> {
//! app.load_theme("themes/sea.tvtheme")?; // или переменная окружения TVISION_THEME
//! # Ok(()) }
//! ```

use std::path::{Path, PathBuf};

use crate::color::*;
use crate::json::{parse_json, Json};
use crate::widgets::colorsel::ColorGroup;

/// Версия формата файла.
pub const FORMAT_VERSION: i64 = 1;

/// Сколько цветов в палитре приложения.
pub const PALETTE_LEN: usize = 135;

/// Переменная окружения с путём к теме, которую `Program::with_terminal`
/// подключает сама.
pub const THEME_ENV: &str = "TVISION_THEME";

/// Встроенная палитра, от которой отсчитывается тема.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Base {
    /// Классическая цветная палитра Turbo Vision (16 цветов).
    Classic,
    /// Палитра gruvbox (256 цветов) — по умолчанию в tvision-rs.
    Gruvbox,
    /// Чёрно-белая.
    BlackWhite,
    /// Монохромная.
    Monochrome,
}

impl Base {
    pub const ALL: [Base; 4] = [Base::Classic, Base::Gruvbox, Base::BlackWhite, Base::Monochrome];

    pub fn key(self) -> &'static str {
        match self {
            Base::Classic => "classic",
            Base::Gruvbox => "gruvbox",
            Base::BlackWhite => "blackwhite",
            Base::Monochrome => "monochrome",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Base::Classic => "Классическая",
            Base::Gruvbox => "Gruvbox",
            Base::BlackWhite => "Чёрно-белая",
            Base::Monochrome => "Монохромная",
        }
    }
    pub fn from_key(s: &str) -> Option<Base> {
        Base::ALL.into_iter().find(|b| b.key() == s)
    }
    /// Сама палитра.
    pub fn palette(self) -> &'static [Attr] {
        match self {
            Base::Classic => palettes::APP_COLOR,
            Base::Gruvbox => palettes::GRUVBOX,
            Base::BlackWhite => palettes::APP_BLACK_WHITE,
            Base::Monochrome => palettes::APP_MONOCHROME,
        }
    }
}

// ----------------------------------------------------------------------
// Группы цветов
// ----------------------------------------------------------------------

/// Все цвета палитры по группам, с русскими названиями (для редактора
/// тем и `ColorDialog`). Номера — позиции в палитре приложения (с 1).
pub fn color_groups() -> Vec<ColorGroup> {
    let window = |base: u8| -> Vec<(&'static str, u8)> {
        vec![
            ("Рамка неактивная", base),
            ("Рамка активная", base + 1),
            ("Значки рамки", base + 2),
            ("Полоса прокрутки", base + 3),
            ("Стрелки прокрутки", base + 4),
            ("Текст", base + 5),
            ("Выделенный текст", base + 6),
        ]
    };
    let dialog = |base: u8| -> Vec<(&'static str, u8)> {
        let names = [
            "Рамка неактивная",
            "Рамка и фон",
            "Значки рамки",
            "Полоса прокрутки",
            "Стрелки прокрутки",
            "Текст",
            "Подпись",
            "Подпись выбранного поля",
            "Горячая буква подписи",
            "Кнопка",
            "Кнопка по умолчанию",
            "Кнопка выбранная",
            "Кнопка недоступная",
            "Горячая буква кнопки",
            "Тень кнопки",
            "Флажки",
            "Флажки выбранные",
            "Горячая буква флажков",
            "Строка ввода",
            "Строка ввода выбранная",
            "Стрелки строки ввода",
            "Кнопка истории",
            "Края кнопки истории",
            "История: полоса прокрутки",
            "История: стрелки",
            "Список",
            "Список: курсор",
            "Список: отмеченное",
            "Список: разделитель",
            "Информационная строка",
            "Флажки недоступные",
        ];
        names.iter().enumerate().map(|(i, n)| (*n, base + i as u8)).collect()
    };
    vec![
        ColorGroup::new("Рабочий стол", &[("Фон", 1)]),
        ColorGroup::new(
            "Меню и строка состояния",
            &[
                ("Пункт", 2),
                ("Недоступный пункт", 3),
                ("Горячая буква", 4),
                ("Выбранный пункт", 5),
                ("Выбранный недоступный", 6),
                ("Горячая буква выбранного", 7),
            ],
        ),
        ColorGroup::new("Синее окно", &window(8)),
        ColorGroup::new("Бирюзовое окно", &window(16)),
        ColorGroup::new("Серое окно", &window(24)),
        ColorGroup::new("Серый диалог", &dialog(32)),
        ColorGroup::new("Синий диалог", &dialog(64)),
        ColorGroup::new("Бирюзовый диалог", &dialog(96)),
        ColorGroup::new(
            "Окно справки",
            &[
                ("Рамка неактивная", 128),
                ("Рамка активная", 129),
                ("Значки рамки", 130),
                ("Полоса прокрутки", 131),
                ("Стрелки прокрутки", 132),
                ("Текст", 133),
                ("Ссылка", 134),
                ("Выбранная ссылка", 135),
            ],
        ),
    ]
}

/// Название цвета палитры номер `index`: «Серый диалог: кнопка».
pub fn item_name(index: u8) -> Option<String> {
    for g in color_groups() {
        if let Some(it) = g.items.iter().find(|i| i.index == index) {
            return Some(format!("{}: {}", g.name, it.name.to_lowercase()));
        }
    }
    None
}

// ----------------------------------------------------------------------
// Запись цвета текстом
// ----------------------------------------------------------------------

/// Имена цветов BIOS (0–15).
pub const COLOR_NAMES: [&str; 16] = [
    "black",
    "blue",
    "green",
    "cyan",
    "red",
    "magenta",
    "brown",
    "lightgray",
    "darkgray",
    "lightblue",
    "lightgreen",
    "lightcyan",
    "lightred",
    "lightmagenta",
    "yellow",
    "white",
];

/// Русские названия цветов BIOS — для подсказок.
pub const COLOR_TITLES: [&str; 16] = [
    "чёрный",
    "синий",
    "зелёный",
    "бирюзовый",
    "красный",
    "фиолетовый",
    "коричневый",
    "светло-серый",
    "тёмно-серый",
    "голубой",
    "светло-зелёный",
    "светло-бирюзовый",
    "светло-красный",
    "розовый",
    "жёлтый",
    "белый",
];

/// Цвет одной стороны атрибута.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    /// Цвет BIOS 0–15 (виден в любом терминале).
    Bios(u8),
    /// Цвет xterm-256.
    Xterm(u8),
}

impl Color {
    pub fn parse(s: &str) -> Result<Color, String> {
        let s = s.trim().to_lowercase();
        if let Some(i) = COLOR_NAMES.iter().position(|n| *n == s.replace(['-', '_', ' '], "")) {
            return Ok(Color::Bios(i as u8));
        }
        if let Some(hex) = s.strip_prefix('#') {
            let v = u32::from_str_radix(hex, 16).ok().filter(|_| hex.len() == 6).ok_or_else(|| format!("«{}»: нужно #rrggbb.", s))?;
            return Ok(Color::Xterm(rgb_to_xterm(v)));
        }
        match s.parse::<u16>() {
            Ok(n) if n <= 255 => Ok(Color::Xterm(n as u8)),
            _ => Err(format!("Неизвестный цвет «{}»: нужно имя (white, lightblue…), номер 0–255 или #rrggbb.", s)),
        }
    }

    pub fn to_text(self) -> String {
        match self {
            Color::Bios(c) => COLOR_NAMES[(c & 15) as usize].to_string(),
            Color::Xterm(n) => n.to_string(),
        }
    }

    /// Цвет текста атрибута.
    pub fn fg_of(a: Attr) -> Color {
        if is_fg256(a) {
            Color::Xterm(fg256(a))
        } else {
            Color::Bios(fore(a))
        }
    }

    /// Цвет фона атрибута.
    pub fn bg_of(a: Attr) -> Color {
        if is_bg256(a) {
            Color::Xterm(bg256(a))
        } else {
            Color::Bios(back(a))
        }
    }
}

/// Атрибут из цветов текста и фона.
pub fn make_attr(fg: Color, bg: Color) -> Attr {
    let a = match fg {
        Color::Bios(c) => with_fore(0, c),
        Color::Xterm(n) => with_fg256(0, n),
    };
    match bg {
        Color::Bios(c) => with_back(a, c),
        Color::Xterm(n) => with_bg256(a, n),
    }
}

/// Атрибут → текст `текст/фон`.
pub fn attr_to_text(a: Attr) -> String {
    format!("{}/{}", Color::fg_of(a).to_text(), Color::bg_of(a).to_text())
}

/// Текст `текст/фон` → атрибут.
pub fn attr_from_text(s: &str) -> Result<Attr, String> {
    let (f, b) = s.split_once('/').ok_or_else(|| format!("«{}»: цвет пишется как текст/фон, например white/blue.", s))?;
    Ok(make_attr(Color::parse(f)?, Color::parse(b)?))
}

// ----------------------------------------------------------------------
// Тема
// ----------------------------------------------------------------------

/// Тема: имя, описание и палитра приложения.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub name: String,
    pub description: String,
    /// От какой встроенной палитры записываются отличия.
    pub base: Base,
    /// Полная палитра (135 цветов).
    pub palette: Vec<Attr>,
}

impl Theme {
    /// Тема, совпадающая со встроенной палитрой.
    pub fn builtin(base: Base) -> Theme {
        Theme { name: base.title().to_string(), description: String::new(), base, palette: base.palette().to_vec() }
    }

    /// Тема из готовой палитры (например, `app.palette`).
    pub fn from_palette(name: &str, base: Base, palette: &[Attr]) -> Theme {
        let mut p = base.palette().to_vec();
        for (i, a) in palette.iter().take(PALETTE_LEN).enumerate() {
            p[i] = *a;
        }
        Theme { name: name.to_string(), description: String::new(), base, palette: p }
    }

    /// Цвет номер `index` (с 1).
    pub fn get(&self, index: u8) -> Attr {
        self.palette.get(index as usize - 1).copied().unwrap_or(ERROR_ATTR)
    }

    pub fn set(&mut self, index: u8, a: Attr) {
        if let Some(x) = self.palette.get_mut(index as usize - 1) {
            *x = a;
        }
    }

    /// Номера цветов, отличающихся от основы.
    pub fn changed(&self) -> Vec<u8> {
        let base = self.base.palette();
        (1..=PALETTE_LEN as u8).filter(|&i| self.palette[i as usize - 1] != base[i as usize - 1]).collect()
    }

    pub fn to_json(&self) -> Json {
        let colors = self
            .changed()
            .into_iter()
            .map(|i| {
                Json::object(vec![
                    ("index", Json::from_i64(i as i64)),
                    ("item", Json::from_str(&item_name(i).unwrap_or_else(|| format!("цвет {}", i)))),
                    ("color", Json::from_str(&attr_to_text(self.get(i)))),
                ])
            })
            .collect();
        Json::object(vec![
            ("tvtheme", Json::from_i64(FORMAT_VERSION)),
            ("name", Json::from_str(&self.name)),
            ("description", Json::from_str(&self.description)),
            ("base", Json::from_str(self.base.key())),
            ("colors", Json::Arr(colors)),
        ])
    }

    /// Текст файла `.tvtheme`: по цвету на строку — удобно читать и
    /// сравнивать в git.
    pub fn to_text(&self) -> String {
        let j = self.to_json();
        let field = |k: &str| j.get(k).map(Json::compact).unwrap_or_default();
        let mut s = String::from("{\n");
        for k in ["tvtheme", "name", "description", "base"] {
            s += &format!("  \"{}\": {},\n", k, field(k));
        }
        let colors = j.get("colors").and_then(Json::as_array).unwrap_or(&[]);
        if colors.is_empty() {
            s += "  \"colors\": []\n}\n";
            return s;
        }
        s += "  \"colors\": [\n";
        for (i, c) in colors.iter().enumerate() {
            s += &format!("    {}{}\n", c.compact(), if i + 1 < colors.len() { "," } else { "" });
        }
        s += "  ]\n}\n";
        s
    }

    pub fn from_json(j: &Json) -> Result<Theme, String> {
        if j.get("tvtheme").and_then(Json::as_i64).is_none() {
            return Err("Это не файл темы (нет ключа \"tvtheme\").".into());
        }
        let base_key = j.get("base").and_then(Json::as_str).unwrap_or("classic");
        let base = Base::from_key(base_key).ok_or_else(|| format!("Неизвестная основа «{}»: classic, gruvbox, blackwhite, monochrome.", base_key))?;
        let mut t = Theme::builtin(base);
        t.name = j.get("name").and_then(Json::as_str).unwrap_or("").to_string();
        t.description = j.get("description").and_then(Json::as_str).unwrap_or("").to_string();
        for (k, c) in j.get("colors").and_then(Json::as_array).unwrap_or(&[]).iter().enumerate() {
            let idx = c
                .get("index")
                .and_then(Json::as_i64)
                .filter(|i| (1..=PALETTE_LEN as i64).contains(i))
                .ok_or_else(|| format!("Цвет {}: нужен \"index\" от 1 до {}.", k + 1, PALETTE_LEN))?;
            let text = c.get("color").and_then(Json::as_str).ok_or_else(|| format!("Цвет {}: нет \"color\".", idx))?;
            let a = attr_from_text(text).map_err(|e| format!("Цвет {}: {}", idx, e))?;
            t.set(idx as u8, a);
        }
        Ok(t)
    }

    pub fn parse(text: &str) -> Result<Theme, String> {
        let j = parse_json(text).map_err(|e| e.to_string())?;
        Theme::from_json(&j)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Theme, String> {
        let p = path.as_ref();
        let text = std::fs::read_to_string(p).map_err(|e| format!("{}: {}", p.display(), e))?;
        Theme::parse(&text).map_err(|e| format!("{}: {}", p.display(), e))
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let p = path.as_ref();
        std::fs::write(p, self.to_text()).map_err(|e| format!("Не удалось записать {}:\n{}", p.display(), e))
    }

    /// Тема как Rust-код: константа с палитрой — чтобы встроить тему в
    /// программу без файла.
    pub fn to_rust(&self, const_name: &str) -> String {
        let mut s = format!(
            "/// Тема «{}»{} — сгенерировано редактором тем.\n///\n/// Подключение: `app.palette = {}.to_vec();`\npub const {}: &[tvision::color::Attr] = &[\n",
            self.name,
            if self.description.is_empty() { String::new() } else { format!(" ({})", self.description) },
            const_name,
            const_name
        );
        for (i, chunk) in self.palette.chunks(8).enumerate() {
            let vals: Vec<String> = chunk.iter().map(|a| format!("0x{:08X}", a)).collect();
            s += &format!("    {}, // {}–{}\n", vals.join(", "), i * 8 + 1, i * 8 + chunk.len());
        }
        s += "];\n";
        s
    }
}

/// Все темы `*.tvtheme` в каталоге (по имени файла). Нечитаемые файлы
/// пропускаются — их ошибки во втором списке.
pub fn find_themes(dir: impl AsRef<Path>) -> (Vec<(PathBuf, Theme)>, Vec<String>) {
    let mut ok = Vec::new();
    let mut errors = Vec::new();
    let mut paths: Vec<PathBuf> = match std::fs::read_dir(dir.as_ref()) {
        Ok(rd) => rd.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|e| e == "tvtheme")).collect(),
        Err(_) => Vec::new(),
    };
    paths.sort();
    for p in paths {
        match Theme::load(&p) {
            Ok(t) => ok.push((p, t)),
            Err(e) => errors.push(e),
        }
    }
    (ok, errors)
}

// ----------------------------------------------------------------------
// Выбор темы в программе
// ----------------------------------------------------------------------

/// Диалог выбора темы с живым предпросмотром: пока курсор ходит по
/// списку, вся программа перекрашивается. OK — оставить тему (вернёт её
/// номер), Отмена — вернуть прежние цвета.
///
/// ```no_run
/// # fn f(app: &mut tvision::Program) {
/// let (themes, _errors) = tvision::theme::find_themes("themes");
/// let list: Vec<_> = themes.into_iter().map(|(_, t)| t).collect();
/// if let Some(i) = tvision::theme::choose_theme(app, &list) {
///     // list[i] уже подключена; можно запомнить выбор в настройках
/// }
/// # }
/// ```
pub fn choose_theme(app: &mut crate::Program, themes: &[Theme]) -> Option<usize> {
    use crate::layout::{item, HBox, VBox};
    use crate::*;
    if themes.is_empty() {
        return None;
    }
    let before = app.palette.clone();
    let mut d = Dialog::centered(50, 16, "Тема");
    let mut list = ListBox::new(Rect::default(), 1, None);
    list.set_items(themes.iter().map(|t| t.name.clone()).collect());
    // Сразу встать на текущую тему, если она есть в списке.
    if let Some(i) = themes.iter().position(|t| t.palette == before) {
        list.focused = i;
    }
    let list_id = d.add(list);
    let about = d.add(StaticText::new(Rect::default(), ""));
    let ok = d.add(Button::new(Rect::default(), "O~K~", cm::OK, bf::DEFAULT));
    let cancel = d.add(Button::new(Rect::default(), "Отмена", cm::CANCEL, bf::NORMAL));
    d.set_layout(
        VBox::new()
            .margins(2, 1, 2, 0)
            .add(item(list_id).min(30, 6).stretch(1))
            .space(1)
            .add(item(about).min_height(2))
            .space(1)
            .add(HBox::new().stretch(1).add(ok).add(cancel)),
    );
    d.select(list_id);
    let mut shown = usize::MAX;
    let (cmd, d) = app.exec_dialog_idle(d, |app, _| {
        let i = app.find::<ListBox>(list_id).map(|l| l.focused).unwrap_or(0);
        if i != shown && i < themes.len() {
            shown = i;
            app.palette = themes[i].palette.clone();
            if let Some(t) = app.find_mut::<StaticText>(about) {
                t.text = themes[i].description.clone();
            }
        }
        None
    });
    let i = d.win.group.find::<ListBox>(list_id).map(|l| l.focused).unwrap_or(0);
    if cmd == cm::OK && i < themes.len() {
        app.palette = themes[i].palette.clone();
        Some(i)
    } else {
        app.palette = before;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_as_text() {
        assert_eq!(attr_to_text(0x1F), "white/blue");
        assert_eq!(attr_from_text("white/blue").unwrap(), 0x1F);
        assert_eq!(attr_from_text("Light-Blue / black").unwrap(), 0x09);
        let a = attr_from_text("214/235").unwrap();
        assert_eq!((fg256(a), bg256(a)), (214, 235));
        assert_eq!(attr_to_text(a), "214/235");
        let m = attr_from_text("yellow/236").unwrap();
        assert_eq!(attr_to_text(m), "yellow/236");
        assert_eq!(Color::parse("#ff0000").unwrap(), Color::Xterm(196));
        assert!(attr_from_text("white").is_err());
        assert!(attr_from_text("white/фиолетовый").is_err());
        // Все цвета обеих встроенных палитр переживают запись текстом.
        for base in Base::ALL {
            for &a in base.palette() {
                assert_eq!(attr_from_text(&attr_to_text(a)).unwrap(), a & 0x03FF_FFFF, "{:#x}", a);
            }
        }
    }

    #[test]
    fn groups_cover_palette() {
        let mut seen = vec![false; PALETTE_LEN + 1];
        for g in color_groups() {
            for it in &g.items {
                assert!(!seen[it.index as usize], "{} дважды", it.index);
                seen[it.index as usize] = true;
            }
        }
        // Не описаны только служебные (резерв): 15, 23, 31, 63, 95, 127.
        let missing: Vec<usize> = (1..=PALETTE_LEN).filter(|&i| !seen[i]).collect();
        assert_eq!(missing, [15, 23, 31, 63, 95, 127]);
        assert_eq!(item_name(42).unwrap(), "Серый диалог: кнопка по умолчанию");
    }

    #[test]
    fn theme_file_roundtrip() {
        let mut t = Theme::builtin(Base::Classic);
        t.name = "Морская".into();
        t.set(1, attr_from_text("cyan/blue").unwrap());
        t.set(42, attr256(221, 24));
        let text = t.to_text();
        assert!(text.contains("\"item\": \"Рабочий стол: фон\"") && text.contains("\"color\": \"221/24\""), "{}", text);
        assert_eq!(t.changed(), vec![1, 42]);
        assert_eq!(Theme::parse(&text).unwrap(), t);
        assert!(Theme::parse("{\"name\": \"x\"}").is_err());
        assert!(Theme::parse("{\"tvtheme\": 1, \"colors\": [{\"index\": 200, \"color\": \"white/blue\"}]}").is_err());
        assert!(Theme::parse("{\"tvtheme\": 1, \"base\": \"neon\"}").is_err());
        let code = t.to_rust("SEA");
        assert!(code.contains("pub const SEA: &[tvision::color::Attr] = &[") && code.contains("// 129–135"), "{}", code);
    }
}

#[cfg(test)]
mod program_tests {
    use super::*;
    use crate::platform::HeadlessBackend;
    use crate::*;

    fn themes() -> Vec<Theme> {
        let mut sea = Theme::builtin(Base::Classic);
        sea.name = "Морская".into();
        sea.set(1, attr_from_text("cyan/blue").unwrap());
        vec![Theme::builtin(Base::Gruvbox), Theme::builtin(Base::Classic), sea]
    }

    #[test]
    fn choose_keeps_or_restores() {
        // ↓ ↓ Enter — третья тема.
        let mut b = HeadlessBackend::new(80, 25);
        b.push_key(kb::key(Key::Down));
        b.push_key(kb::key(Key::Down));
        b.push_key(kb::ENTER);
        let mut app = Program::new(Box::new(b), None, None);
        let t = themes();
        assert_eq!(choose_theme(&mut app, &t), Some(2));
        assert_eq!(app.palette, t[2].palette);
        // ↓ и Esc — цвета прежние.
        let mut b = HeadlessBackend::new(80, 25);
        b.push_key(kb::key(Key::Down));
        b.push_key(kb::ESC);
        let mut app = Program::new(Box::new(b), None, None);
        let before = app.palette.clone();
        assert_eq!(choose_theme(&mut app, &t), None);
        assert_eq!(app.palette, before);
    }

    #[test]
    fn load_theme_from_file_and_dir() {
        let dir = std::env::temp_dir().join(format!("tv-themes-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let t = themes();
        t[2].save(dir.join("sea.tvtheme")).unwrap();
        std::fs::write(dir.join("bad.tvtheme"), "{").unwrap();
        std::fs::write(dir.join("readme.txt"), "не тема").unwrap();
        let (found, errors) = find_themes(&dir);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].1, t[2]);
        assert_eq!(errors.len(), 1);
        let mut app = Program::new(Box::new(HeadlessBackend::new(80, 25)), None, None);
        app.load_theme(dir.join("sea.tvtheme")).unwrap();
        assert_eq!(app.palette[0], attr_from_text("cyan/blue").unwrap());
        assert!(app.load_theme(dir.join("bad.tvtheme")).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
