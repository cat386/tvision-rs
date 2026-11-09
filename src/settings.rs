//! Настройки программы: тема, цвета терминала, узор фона, язык,
//! расположение окон, мышь — в файле рядом с исполнимым файлом, и
//! стандартный диалог «Настройки».
//!
//! Обычный порядок работы:
//!
//! ```ignore
//! use tvision::settings::{settings_dialog, Settings};
//!
//! fn main() -> std::io::Result<()> {
//!     // 1. Прочитать настройки (нет файла — настройки по умолчанию).
//!     let path = Settings::default_path();          // моя_программа.settings.json рядом с .exe
//!     let mut settings = Settings::load_or_default(&path);
//!     settings.apply_language();                    // до создания меню: от языка зависят надписи
//!     let mut app = Program::with_terminal(Some(menu()), Some(status()))?;
//!     settings.apply(&mut app);                     // тема, цвета, фон, мышь
//!     open_my_windows(&mut app);
//!     settings.restore_windows(&mut app);           // окна — туда, где были
//!     app.run(&mut handler);
//!     // 2. При выходе запомнить, где стояли окна.
//!     settings.capture_windows(&app);
//!     let _ = settings.save(&path);
//!     Ok(())
//! }
//!
//! // 3. Пункт меню «Настройки…»:
//! if settings_dialog(app, &mut settings) {         // OK — всё уже применено
//!     let _ = settings.save(&path);
//! }
//! ```
//!
//! Файл — JSON, его можно править руками:
//!
//! ```json
//! {
//!   "tvsettings": 1,
//!   "theme": "themes/sea.tvtheme",
//!   "colors": "auto",
//!   "pattern": "░",
//!   "language": "ru",
//!   "windows_at_start": "restore",
//!   "tile_columns_first": false,
//!   "reverse_buttons": false,
//!   "double_click_ms": 400,
//!   "windows": [ {"key": "1", "bounds": [0, 0, 60, 18], "zoomed": false} ],
//!   "extra": { "свои": "настройки программы" }
//! }
//! ```
//!
//! Тема: `""` — стандартные цвета, `"builtin:classic"` (`gruvbox`,
//! `blackwhite`, `monochrome`) — встроенная, иначе путь к `.tvtheme`;
//! относительный путь отсчитывается от каталога программы.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::color::{palettes, set_color_mode, ColorMode};
use crate::geometry::{Point, Rect};
use crate::json::{parse_json, Json};
use crate::locale::{self, tr, Language};
use crate::theme::{find_themes, Base, Theme};
use crate::view::View;
use crate::widgets::Window;
use crate::Program;

/// Переменная окружения: путь к файлу настроек вместо файла рядом с
/// программой (для тестов и переносных установок).
pub const SETTINGS_ENV: &str = "TVISION_SETTINGS";

/// Версия формата файла.
pub const FORMAT_VERSION: i64 = 1;

/// Узоры фона, которые предлагает диалог.
pub const PATTERNS: [char; 4] = ['░', '▒', '▓', ' '];

/// Язык надписей.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LanguageChoice {
    /// По переменным окружения `LC_ALL`, `LC_MESSAGES`, `LANG`.
    #[default]
    System,
    En,
    Ru,
}

impl LanguageChoice {
    pub const ALL: [LanguageChoice; 3] = [LanguageChoice::System, LanguageChoice::En, LanguageChoice::Ru];

    pub fn key(self) -> &'static str {
        match self {
            LanguageChoice::System => "system",
            LanguageChoice::En => "en",
            LanguageChoice::Ru => "ru",
        }
    }

    pub fn from_key(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|l| l.key() == s)
    }

    /// Какой язык получится.
    pub fn language(self) -> Language {
        match self {
            LanguageChoice::En => Language::En,
            LanguageChoice::Ru => Language::Ru,
            LanguageChoice::System => system_language(),
        }
    }
}

/// Язык системы по переменным окружения (первая непустая из `LC_ALL`,
/// `LC_MESSAGES`, `LANG`): `ru…` — русский, иначе английский.
pub fn system_language() -> Language {
    for v in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(s) = std::env::var(v) {
            if !s.is_empty() {
                return if s.to_ascii_lowercase().starts_with("ru") { Language::Ru } else { Language::En };
            }
        }
    }
    Language::En
}

/// Сколько цветов у терминала.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ColorChoice {
    /// Определить по терминалу ([`crate::color::detect_color_mode`]).
    #[default]
    Auto,
    Colors16,
    Colors256,
}

impl ColorChoice {
    pub const ALL: [ColorChoice; 3] = [ColorChoice::Auto, ColorChoice::Colors16, ColorChoice::Colors256];

    pub fn key(self) -> &'static str {
        match self {
            ColorChoice::Auto => "auto",
            ColorChoice::Colors16 => "16",
            ColorChoice::Colors256 => "256",
        }
    }

    pub fn from_key(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.key() == s)
    }

    fn mode(self) -> Option<ColorMode> {
        match self {
            ColorChoice::Auto => None,
            ColorChoice::Colors16 => Some(ColorMode::Ansi16),
            ColorChoice::Colors256 => Some(ColorMode::Xterm256),
        }
    }
}

/// Как расставить окна при запуске.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WindowsAtStart {
    /// Как стояли при выходе.
    #[default]
    Restore,
    /// Каскадом.
    Cascade,
    /// Мозаикой.
    Tile,
    /// Как их открывает программа (ничего не менять).
    AsProgram,
}

impl WindowsAtStart {
    pub const ALL: [WindowsAtStart; 4] = [WindowsAtStart::Restore, WindowsAtStart::Cascade, WindowsAtStart::Tile, WindowsAtStart::AsProgram];

    pub fn key(self) -> &'static str {
        match self {
            WindowsAtStart::Restore => "restore",
            WindowsAtStart::Cascade => "cascade",
            WindowsAtStart::Tile => "tile",
            WindowsAtStart::AsProgram => "program",
        }
    }

    pub fn from_key(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|w| w.key() == s)
    }
}

/// Где стояло окно.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowPlace {
    /// Ключ окна (см. [`Settings::capture_windows_with`]); по умолчанию —
    /// номер окна, а у окон без номера — заголовок.
    pub key: String,
    /// Границы в обычном виде (у распахнутого — те, к которым оно вернётся).
    pub bounds: Rect,
    /// Окно было распахнуто.
    pub zoomed: bool,
}

/// Все настройки.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    /// Тема: `""` — стандартные цвета, `builtin:…` или путь к `.tvtheme`.
    pub theme: String,
    pub colors: ColorChoice,
    /// Узор рабочего стола.
    pub pattern: char,
    pub language: LanguageChoice,
    pub windows_at_start: WindowsAtStart,
    /// Мозаика сначала по столбцам ([`crate::widgets::desktop::Desktop::tile_columns_first`]).
    pub tile_columns_first: bool,
    /// Поменять местами кнопки мыши.
    pub reverse_buttons: bool,
    /// Интервал двойного щелчка, мс.
    pub double_click_ms: u32,
    /// Где стояли окна.
    pub windows: Vec<WindowPlace>,
    /// Свои настройки программы: [`Settings::get`], [`Settings::set`].
    pub extra: Vec<(String, Json)>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: String::new(),
            colors: ColorChoice::Auto,
            pattern: '░',
            language: LanguageChoice::System,
            windows_at_start: WindowsAtStart::Restore,
            tile_columns_first: false,
            reverse_buttons: false,
            double_click_ms: crate::program::DOUBLE_CLICK.as_millis() as u32,
            windows: Vec::new(),
            extra: Vec::new(),
        }
    }
}

// ----------------------------------------------------------------------
// Файл
// ----------------------------------------------------------------------

/// Каталог исполнимого файла.
pub fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf))
}

/// Путь относительно каталога программы (абсолютный — как есть).
pub fn exe_relative(p: &str) -> PathBuf {
    let p = PathBuf::from(p);
    if p.is_absolute() {
        return p;
    }
    exe_dir().map(|d| d.join(&p)).unwrap_or(p)
}

impl Settings {
    /// Файл настроек: `<имя программы>.settings.json` в каталоге
    /// исполнимого файла (`D:\prog\app.exe` → `D:\prog\app.settings.json`).
    /// Переменная окружения [`SETTINGS_ENV`] задаёт другой путь.
    pub fn default_path() -> PathBuf {
        if let Ok(p) = std::env::var(SETTINGS_ENV) {
            if !p.is_empty() {
                return PathBuf::from(p);
            }
        }
        let exe = std::env::current_exe().ok();
        let stem = exe.as_ref().and_then(|e| e.file_stem()).map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "tvision".into());
        let name = format!("{}.settings.json", stem);
        match exe_dir() {
            Some(d) => d.join(name),
            None => PathBuf::from(name),
        }
    }

    /// Прочитать файл. Неизвестные и испорченные поля — по умолчанию.
    pub fn load(path: impl AsRef<Path>) -> Result<Settings, String> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
        Settings::parse(&text).map_err(|e| format!("{}: {}", path.display(), e))
    }

    /// Прочитать; нет файла или он испорчен — настройки по умолчанию.
    pub fn load_or_default(path: impl AsRef<Path>) -> Settings {
        Settings::load(path).unwrap_or_default()
    }

    /// Записать (каталог создаётся).
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref();
        if let Some(d) = path.parent() {
            if !d.as_os_str().is_empty() {
                std::fs::create_dir_all(d).map_err(|e| format!("{}: {}", d.display(), e))?;
            }
        }
        std::fs::write(path, self.to_text()).map_err(|e| format!("{}: {}", path.display(), e))
    }

    /// Текст файла.
    pub fn to_text(&self) -> String {
        self.to_json().pretty() + "\n"
    }

    pub fn parse(text: &str) -> Result<Settings, String> {
        let j = parse_json(text).map_err(|e| e.to_string())?;
        if j.as_object().is_none() {
            return Err("ожидался объект JSON".into());
        }
        Ok(Settings::from_json(&j))
    }

    pub fn to_json(&self) -> Json {
        let windows = self
            .windows
            .iter()
            .map(|w| {
                let b = w.bounds;
                Json::object(vec![
                    ("key", Json::from_str(&w.key)),
                    (
                        "bounds",
                        Json::Arr(vec![
                            Json::from_i64(b.a.x as i64),
                            Json::from_i64(b.a.y as i64),
                            Json::from_i64(b.b.x as i64),
                            Json::from_i64(b.b.y as i64),
                        ]),
                    ),
                    ("zoomed", Json::Bool(w.zoomed)),
                ])
            })
            .collect();
        Json::object(vec![
            ("tvsettings", Json::from_i64(FORMAT_VERSION)),
            ("theme", Json::from_str(&self.theme)),
            ("colors", Json::from_str(self.colors.key())),
            ("pattern", Json::from_str(&self.pattern.to_string())),
            ("language", Json::from_str(self.language.key())),
            ("windows_at_start", Json::from_str(self.windows_at_start.key())),
            ("tile_columns_first", Json::Bool(self.tile_columns_first)),
            ("reverse_buttons", Json::Bool(self.reverse_buttons)),
            ("double_click_ms", Json::from_i64(self.double_click_ms as i64)),
            ("windows", Json::Arr(windows)),
            ("extra", Json::Obj(self.extra.clone())),
        ])
    }

    /// Из JSON; чего нет или что не читается — по умолчанию.
    pub fn from_json(j: &Json) -> Settings {
        let d = Settings::default();
        let s = |k: &str| j.get(k).and_then(Json::as_str);
        let b = |k: &str, def: bool| j.get(k).and_then(Json::as_bool).unwrap_or(def);
        let windows = j
            .get("windows")
            .and_then(Json::as_array)
            .unwrap_or(&[])
            .iter()
            .filter_map(|w| {
                let key = w.get("key")?.as_str()?.to_string();
                let r: Vec<i32> = w.get("bounds")?.as_array()?.iter().filter_map(|n| n.as_i64()).map(|n| n as i32).collect();
                if r.len() != 4 || r[2] <= r[0] || r[3] <= r[1] {
                    return None;
                }
                Some(WindowPlace { key, bounds: Rect::new(r[0], r[1], r[2], r[3]), zoomed: w.get("zoomed").and_then(Json::as_bool).unwrap_or(false) })
            })
            .collect();
        Settings {
            theme: s("theme").unwrap_or("").to_string(),
            colors: s("colors").and_then(ColorChoice::from_key).unwrap_or(d.colors),
            pattern: s("pattern").and_then(|p| p.chars().next()).unwrap_or(d.pattern),
            language: s("language").and_then(LanguageChoice::from_key).unwrap_or(d.language),
            windows_at_start: s("windows_at_start").and_then(WindowsAtStart::from_key).unwrap_or(d.windows_at_start),
            tile_columns_first: b("tile_columns_first", d.tile_columns_first),
            reverse_buttons: b("reverse_buttons", d.reverse_buttons),
            double_click_ms: j.get("double_click_ms").and_then(Json::as_i64).map(|n| n.clamp(50, 5000) as u32).unwrap_or(d.double_click_ms),
            windows,
            extra: j.get("extra").and_then(Json::as_object).map(|o| o.to_vec()).unwrap_or_default(),
        }
    }

    // ---------- свои настройки программы ----------

    /// Своя настройка программы.
    pub fn get(&self, key: &str) -> Option<&Json> {
        self.extra.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Записать свою настройку (заменяет прежнее значение).
    pub fn set(&mut self, key: &str, value: Json) {
        match self.extra.iter_mut().find(|(k, _)| k == key) {
            Some(e) => e.1 = value,
            None => self.extra.push((key.to_string(), value)),
        }
    }

    // ---------- применить ----------

    /// Включить язык. Вызывайте до создания меню и окон: их надписи
    /// обычно выбираются по [`locale::language`].
    pub fn apply_language(&self) {
        locale::set_language(self.language.language());
    }

    /// Применить всё, кроме расположения окон: язык, цвета терминала,
    /// тему, узор фона, мышь, мозаику. Тему, которая не читается,
    /// пропускает (цвета стандартные) и возвращает ошибку текстом.
    pub fn apply(&self, app: &mut Program) -> Result<(), String> {
        self.apply_language();
        set_color_mode(self.colors.mode());
        let res = match resolve_theme(&self.theme) {
            Ok(Some(t)) => {
                app.set_theme(&t);
                Ok(())
            }
            Ok(None) => {
                app.palette = standard_palette();
                Ok(())
            }
            Err(e) => {
                app.palette = standard_palette();
                Err(e)
            }
        };
        app.desktop_mut().set_pattern(self.pattern);
        app.desktop_mut().tile_columns_first = self.tile_columns_first;
        app.reverse_buttons = self.reverse_buttons;
        app.double_delay = Duration::from_millis(self.double_click_ms as u64);
        res
    }

    // ---------- окна ----------

    /// Ключ окна по умолчанию: номер (`"1"`…), а у окон без номера —
    /// заголовок.
    pub fn default_window_key(w: &Window) -> Option<String> {
        if w.number > 0 {
            Some(w.number.to_string())
        } else if !w.title.is_empty() {
            Some(w.title.clone())
        } else {
            None
        }
    }

    /// Запомнить, где стоят окна рабочего стола (ключ —
    /// [`Settings::default_window_key`]). Окна, которых сейчас нет,
    /// остаются в списке с прежним местом.
    pub fn capture_windows(&mut self, app: &Program) {
        self.capture_windows_with(app, Settings::default_window_key);
    }

    /// То же со своим ключом: например, по имени файла у окон редактора.
    /// `None` — окно не запоминать.
    pub fn capture_windows_with(&mut self, app: &Program, key: impl Fn(&Window) -> Option<String>) {
        let mut now: Vec<WindowPlace> = Vec::new();
        for v in &app.desktop().group.children {
            let Some(w) = v.as_window() else { continue };
            if !w.group.vs.is_visible() {
                continue;
            }
            let Some(k) = key(w) else { continue };
            if let Some(i) = now.iter().position(|p| p.key == k) {
                now.remove(i); // одинаковые ключи — запоминается верхнее окно
            }
            now.push(WindowPlace { key: k, bounds: w.normal_bounds(), zoomed: w.is_zoomed() });
        }
        // Сначала открытые окна (снизу вверх, как лежат на столе), потом прежние.
        for old in self.windows.drain(..) {
            if !now.iter().any(|p| p.key == old.key) {
                now.push(old);
            }
        }
        self.windows = now;
    }

    /// Место окна по ключу.
    pub fn window(&self, key: &str) -> Option<&WindowPlace> {
        self.windows.iter().find(|w| w.key == key)
    }

    /// Расставить окна рабочего стола по [`Settings::windows_at_start`].
    pub fn restore_windows(&self, app: &mut Program) {
        self.restore_windows_with(app, Settings::default_window_key);
    }

    pub fn restore_windows_with(&self, app: &mut Program, key: impl Fn(&Window) -> Option<String>) {
        let ext = app.desktop().group.vs.extent();
        match self.windows_at_start {
            WindowsAtStart::AsProgram => {}
            WindowsAtStart::Cascade => app.desktop_mut().cascade(ext),
            WindowsAtStart::Tile => app.desktop_mut().tile(ext),
            WindowsAtStart::Restore => {
                for v in app.desktop_mut().group.children.iter_mut() {
                    let Some(k) = v.as_window().and_then(&key) else { continue };
                    if let Some(p) = self.window(&k) {
                        place(v.as_mut(), p, ext.b);
                    }
                }
            }
        }
    }

    /// Поставить окно на запомненное место перед `insert_window` (для
    /// окон, которые открываются по ходу работы). `true` — место нашлось.
    pub fn place_window(&self, app: &Program, w: &mut dyn View) -> bool {
        self.place_window_with(app, w, Settings::default_window_key)
    }

    pub fn place_window_with(&self, app: &Program, w: &mut dyn View, key: impl Fn(&Window) -> Option<String>) -> bool {
        let size = app.desktop().group.vs.size;
        match w.as_window().and_then(key).and_then(|k| self.window(&k).cloned()) {
            Some(p) => {
                place(w, &p, size);
                true
            }
            None => false,
        }
    }
}

/// Поставить окно: границы подгоняются к рабочему столу (терминал мог
/// стать меньше), распахнутое распахивается.
fn place(v: &mut dyn View, p: &WindowPlace, desk: Point) {
    let (min, max) = v.size_limits(desk);
    let w = p.bounds.width().clamp(min.x, max.x.max(min.x)).min(desk.x.max(min.x));
    let h = p.bounds.height().clamp(min.y, max.y.max(min.y)).min(desk.y.max(min.y));
    let x = p.bounds.a.x.clamp(0, (desk.x - w).max(0));
    let y = p.bounds.a.y.clamp(0, (desk.y - h).max(0));
    v.change_bounds(Rect::new(x, y, x + w, y + h));
    if let Some(win) = v.as_window_mut() {
        if p.zoomed != win.is_zoomed() {
            win.zoom(desk);
        }
    }
}

// ----------------------------------------------------------------------
// Темы
// ----------------------------------------------------------------------

/// Стандартная палитра — как у [`Program::with_terminal`]: gruvbox, а в 16
/// цветах — классическая.
pub fn standard_palette() -> Vec<crate::color::Attr> {
    if crate::color::color_mode() == ColorMode::Ansi16 {
        palettes::APP_COLOR.to_vec()
    } else {
        palettes::GRUVBOX.to_vec()
    }
}

/// Тема по значению из настроек: `Ok(None)` — стандартные цвета.
pub fn resolve_theme(key: &str) -> Result<Option<Theme>, String> {
    if key.is_empty() {
        return Ok(None);
    }
    if let Some(b) = key.strip_prefix("builtin:") {
        return Base::from_key(b).map(|b| Some(Theme::builtin(b))).ok_or_else(|| format!("нет встроенной темы «{}»", b));
    }
    Theme::load(exe_relative(key)).map(Some)
}

/// Каталоги с темами: `themes/` рядом с программой и `themes/` текущего
/// каталога (существующие, без повторов).
pub fn theme_dirs() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    for d in [exe_dir().map(|d| d.join("themes")), Some(PathBuf::from("themes"))].into_iter().flatten() {
        if let Ok(c) = d.canonicalize() {
            if c.is_dir() && !out.contains(&c) {
                out.push(c);
            }
        }
    }
    out
}

/// Темы для диалога: (значение для [`Settings::theme`], тема). Сначала
/// темы из каталогов, потом встроенные, которых среди них нет. Пути
/// внутри каталога программы записываются относительными — настройки
/// переезжают вместе с программой.
pub fn available_themes(dirs: &[PathBuf]) -> Vec<(String, Theme)> {
    let exe = exe_dir().and_then(|d| d.canonicalize().ok());
    let mut out: Vec<(String, Theme)> = Vec::new();
    for d in dirs {
        for (p, t) in find_themes(d).0 {
            if out.iter().any(|(_, x)| x.palette == t.palette && x.name == t.name) {
                continue;
            }
            let key = match exe.as_ref().and_then(|e| p.strip_prefix(e).ok()) {
                Some(rel) => rel.to_string_lossy().replace('\\', "/"),
                None => p.to_string_lossy().into_owned(),
            };
            out.push((key, t));
        }
    }
    for b in Base::ALL {
        let t = Theme::builtin(b);
        if !out.iter().any(|(_, x)| x.palette == t.palette) {
            out.push((format!("builtin:{}", b.key()), t));
        }
    }
    out
}

// ----------------------------------------------------------------------
// Диалог
// ----------------------------------------------------------------------

/// Стандартный диалог «Настройки» с темами из [`theme_dirs`].
/// См. [`settings_dialog_with`].
pub fn settings_dialog(app: &mut Program, settings: &mut Settings) -> bool {
    let themes = available_themes(&theme_dirs());
    settings_dialog_with(app, settings, &themes)
}

/// Стандартный диалог «Настройки»: тема (с живым предпросмотром), цвета
/// терминала, узор фона, язык, окна при запуске, мозаика, мышь.
///
/// Пока диалог открыт, тема, цвета и узор видны сразу. OK — настройки
/// записываются в `settings` и применяются (`true`); окна при этом не
/// переставляются — порядок «при запуске» действует со следующего
/// запуска. Отмена — всё как было (`false`). «По умолчанию» заполняет
/// диалог стандартными значениями.
///
/// Файл диалог не пишет: сохраните `settings.save(путь)` сами.
pub fn settings_dialog_with(app: &mut Program, settings: &mut Settings, themes: &[(String, Theme)]) -> bool {
    // Что вернуть при отмене.
    let saved = (app.palette.clone(), app.desktop().pattern());
    let saved_mode = settings.colors;
    let mut current = settings.clone();
    loop {
        let (cmd, values) = run_dialog(app, &current, themes);
        match cmd {
            crate::cm::OK => {
                values.store(&mut current, themes);
                let _ = current.apply(app);
                *settings = current;
                return true;
            }
            crate::cm::YES => {
                // «По умолчанию»: те же окна и свои настройки, остальное — стандартное.
                let d = Settings::default();
                current = Settings { windows: current.windows.clone(), extra: current.extra.clone(), ..d };
            }
            _ => {
                set_color_mode(saved_mode.mode());
                app.palette = saved.0;
                app.desktop_mut().set_pattern(saved.1);
                return false;
            }
        }
    }
}

/// Что выбрано в диалоге.
#[derive(Clone, Debug)]
struct Values {
    /// Номер темы в списке (0 — стандартные цвета), `None` — тема из
    /// настроек не нашлась в списке и не трогалась.
    theme: Option<usize>,
    colors: usize,
    pattern: Option<usize>,
    language: usize,
    windows: usize,
    options: u32,
    double_click: String,
}

impl Values {
    fn store(&self, s: &mut Settings, themes: &[(String, Theme)]) {
        if let Some(i) = self.theme {
            s.theme = if i == 0 { String::new() } else { themes[i - 1].0.clone() };
        }
        s.colors = ColorChoice::ALL[self.colors.min(2)];
        if let Some(p) = self.pattern {
            s.pattern = PATTERNS[p.min(PATTERNS.len() - 1)];
        }
        s.language = LanguageChoice::ALL[self.language.min(2)];
        s.windows_at_start = WindowsAtStart::ALL[self.windows.min(3)];
        s.tile_columns_first = self.options & 1 != 0;
        s.reverse_buttons = self.options & 2 != 0;
        if let Ok(n) = self.double_click.trim().parse::<u32>() {
            s.double_click_ms = n.clamp(50, 5000);
        }
    }
}

/// Палитра для пункта списка тем (0 — стандартная).
fn theme_palette(i: usize, themes: &[(String, Theme)]) -> Vec<crate::color::Attr> {
    match i.checked_sub(1).and_then(|k| themes.get(k)) {
        Some((_, t)) => t.palette.clone(),
        None => standard_palette(),
    }
}

fn run_dialog(app: &mut Program, s: &Settings, themes: &[(String, Theme)]) -> (u16, Values) {
    use crate::layout::{item, HBox, VBox};
    use crate::widgets::validate::RangeValidator;
    use crate::*;

    let mut d = Dialog::centered(72, 22, tr("Settings"));

    // Тема.
    let mut names = vec![tr("Standard colors").to_string()];
    names.extend(themes.iter().map(|(_, t)| t.name.clone()));
    let mut list = ListBox::new(Rect::default(), 1, None);
    list.set_items(names);
    // Текущая тема: по значению из настроек, иначе по палитре.
    let theme_idx = if s.theme.is_empty() {
        Some(0)
    } else {
        themes
            .iter()
            .position(|(k, _)| *k == s.theme)
            .or_else(|| resolve_theme(&s.theme).ok().flatten().and_then(|t| themes.iter().position(|(_, x)| x.palette == t.palette)))
            .map(|i| i + 1)
    };
    list.focused = theme_idx.unwrap_or(0);
    let list_id = d.add(list);
    let theme_label = d.add(Label::new(Rect::default(), tr("~T~heme"), Some(list_id)));
    let about = d.add(StaticText::new(Rect::default(), ""));

    // Цвета терминала.
    let mut colors = Cluster::radio_buttons(Rect::default(), &[tr("Detect automatically"), tr("16 colors"), tr("256 colors")]);
    colors.value = ColorChoice::ALL.iter().position(|c| *c == s.colors).unwrap_or(0) as u32;
    let colors_id = d.add(colors);
    let colors_label = d.add(Label::new(Rect::default(), tr("Terminal ~c~olors"), Some(colors_id)));

    // Узор фона.
    let pattern_idx = PATTERNS.iter().position(|c| *c == s.pattern);
    let mut pattern = Cluster::radio_buttons(Rect::default(), &[tr("░ light"), tr("▒ medium"), tr("▓ dark"), tr("none")]);
    pattern.value = pattern_idx.unwrap_or(0) as u32;
    let pattern_id = d.add(pattern);
    let pattern_label = d.add(Label::new(Rect::default(), tr("Desktop ~p~attern"), Some(pattern_id)));

    // Язык.
    let mut lang = Cluster::radio_buttons(Rect::default(), &[tr("As in the system"), "English", "Русский"]);
    lang.value = LanguageChoice::ALL.iter().position(|l| *l == s.language).unwrap_or(0) as u32;
    let lang_id = d.add(lang);
    let lang_label = d.add(Label::new(Rect::default(), tr("~L~anguage"), Some(lang_id)));

    // Окна при запуске.
    let mut wins = Cluster::radio_buttons(Rect::default(), &[tr("Where they were"), tr("Cascade"), tr("Tile"), tr("As the program opens them")]);
    wins.value = WindowsAtStart::ALL.iter().position(|w| *w == s.windows_at_start).unwrap_or(0) as u32;
    let wins_id = d.add(wins);
    let wins_label = d.add(Label::new(Rect::default(), tr("~W~indows at start"), Some(wins_id)));

    // Флажки.
    let mut opts = Cluster::check_boxes(Rect::default(), &[tr("Tile columns first"), tr("Swap mouse buttons")]);
    opts.value = (s.tile_columns_first as u32) | ((s.reverse_buttons as u32) << 1);
    let opts_id = d.add(opts);
    let opts_label = d.add(Label::new(Rect::default(), tr("~O~ptions"), Some(opts_id)));

    // Двойной щелчок.
    let mut dbl = InputLine::new(Rect::default(), 4);
    dbl.validator = Some(Box::new(RangeValidator::new(50, 5000)));
    let dbl_id = d.add(dbl);
    let dbl_label = d.add(Label::new(Rect::default(), tr("Dou~b~le click, ms:"), Some(dbl_id)));

    let ok = d.add(Button::new(Rect::default(), "O~K~", cm::OK, bf::DEFAULT));
    let cancel = d.add(Button::new(Rect::default(), tr("Cancel"), cm::CANCEL, bf::NORMAL));
    let defaults = d.add(Button::new(Rect::default(), tr("~D~efaults"), cm::YES, bf::NORMAL));

    let left = VBox::new()
        .add(theme_label)
        .add(item(list_id).min(30, 5).stretch(1))
        .add(item(about).min(30, 2))
        .space(1)
        .add(colors_label)
        .add(colors_id)
        .space(1)
        .add(pattern_label)
        // Четыре узора — в два столбца.
        .add(item(pattern_id).fixed_height(2));
    let right = VBox::new()
        .add(lang_label)
        .add(lang_id)
        .space(1)
        .add(wins_label)
        .add(wins_id)
        .space(1)
        .add(opts_label)
        .add(opts_id)
        .space(1)
        .add(HBox::new().add(dbl_label).add(item(dbl_id).fixed_width(6)).stretch(1))
        .stretch(1);
    d.set_layout(
        VBox::new()
            .margins(2, 1, 2, 0)
            .add(HBox::new().spacing(3).add(left).add(right))
            .space(1)
            .add(HBox::new().stretch(1).add(ok).add(cancel).add(defaults)),
    );
    d.adjust_size();
    // Текст — после раскладки: до неё у строки нулевая ширина, и она
    // прокрутилась бы в конец.
    if let Some(l) = d.win.group.find_mut::<InputLine>(dbl_id) {
        l.set_text(&s.double_click_ms.to_string());
    }
    d.select(list_id);

    // Живой предпросмотр: тема, цвета терминала, узор.
    let mut shown = (usize::MAX, u32::MAX, u32::MAX);
    let (cmd, d) = app.exec_dialog_idle(d, |app, _| {
        let t = app.find::<ListBox>(list_id).map(|l| l.focused).unwrap_or(0);
        let c = app.find::<Cluster>(colors_id).map(|c| c.value).unwrap_or(0);
        let p = app.find::<Cluster>(pattern_id).map(|c| c.value).unwrap_or(0);
        if (t, c, p) != shown {
            if c != shown.1 {
                set_color_mode(ColorChoice::ALL[(c as usize).min(2)].mode());
            }
            if t != shown.0 || c != shown.1 {
                app.palette = theme_palette(t, themes);
                let text = match t.checked_sub(1).and_then(|k| themes.get(k)) {
                    Some((_, th)) => th.description.clone(),
                    None => tr("The program's own colors.").to_string(),
                };
                if let Some(a) = app.find_mut::<StaticText>(about) {
                    a.text = text;
                }
            }
            if p != shown.2 {
                app.desktop_mut().set_pattern(PATTERNS[(p as usize).min(PATTERNS.len() - 1)]);
            }
            shown = (t, c, p);
        }
        None
    });

    let g = &d.win.group;
    let list_focused = g.find::<ListBox>(list_id).map(|l| l.focused).unwrap_or(0);
    let pattern_value = g.find::<Cluster>(pattern_id).map(|c| c.value as usize).unwrap_or(0);
    let values = Values {
        // Тема из настроек, которой нет в списке, остаётся, пока курсор не сдвинули.
        theme: if theme_idx.is_none() && list_focused == 0 { None } else { Some(list_focused) },
        colors: g.find::<Cluster>(colors_id).map(|c| c.value as usize).unwrap_or(0),
        // Свой узор (не из списка) остаётся, пока не выбран другой.
        pattern: if pattern_idx.is_none() && pattern_value == 0 { None } else { Some(pattern_value) },
        language: g.find::<Cluster>(lang_id).map(|c| c.value as usize).unwrap_or(0),
        windows: g.find::<Cluster>(wins_id).map(|c| c.value as usize).unwrap_or(0),
        options: g.find::<Cluster>(opts_id).map(|c| c.value).unwrap_or(0),
        double_click: g.find::<InputLine>(dbl_id).map(|l| l.text()).unwrap_or_default(),
    };
    (cmd, values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::HeadlessBackend;

    fn tmp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("tvision-settings-{}-{}", std::process::id(), name))
    }

    #[test]
    fn default_path_is_next_to_exe() {
        let p = Settings::default_path();
        if std::env::var(SETTINGS_ENV).is_err() {
            assert_eq!(p.parent(), exe_dir().as_deref());
            assert!(p.file_name().unwrap().to_string_lossy().ends_with(".settings.json"));
        }
    }

    #[test]
    fn roundtrip_file() {
        let mut s = Settings {
            theme: "builtin:classic".into(),
            colors: ColorChoice::Colors16,
            pattern: '▒',
            language: LanguageChoice::Ru,
            windows_at_start: WindowsAtStart::Tile,
            tile_columns_first: true,
            reverse_buttons: true,
            double_click_ms: 600,
            windows: vec![WindowPlace { key: "1".into(), bounds: Rect::new(1, 2, 40, 12), zoomed: true }],
            extra: Vec::new(),
        };
        s.set("последний файл", Json::from_str("a.txt"));
        let p = tmp("rt").join("app.settings.json");
        s.save(&p).unwrap();
        assert_eq!(Settings::load(&p).unwrap(), s);
        assert_eq!(Settings::load(&p).unwrap().get("последний файл").and_then(Json::as_str), Some("a.txt"));
        // Испорченный файл и неизвестные значения — по умолчанию.
        std::fs::write(&p, "{").unwrap();
        assert!(Settings::load(&p).is_err());
        assert_eq!(Settings::load_or_default(&p), Settings::default());
        let odd =
            Settings::parse(r#"{"colors": "1024", "pattern": "", "double_click_ms": 1, "windows": [{"key": "x", "bounds": [5, 5, 1, 1]}]}"#).unwrap();
        assert_eq!(odd.colors, ColorChoice::Auto);
        assert_eq!(odd.pattern, '░');
        assert_eq!(odd.double_click_ms, 50);
        assert!(odd.windows.is_empty());
        let _ = std::fs::remove_dir_all(p.parent().unwrap());
    }

    #[test]
    fn apply_sets_everything() {
        let mut app = Program::new(Box::new(HeadlessBackend::new(80, 25)), None, None);
        let s = Settings {
            theme: "builtin:classic".into(),
            pattern: '▓',
            reverse_buttons: true,
            double_click_ms: 700,
            tile_columns_first: true,
            ..Settings::default()
        };
        s.apply(&mut app).unwrap();
        assert_eq!(app.palette, palettes::APP_COLOR.to_vec());
        assert_eq!(app.desktop().pattern(), '▓');
        assert!(app.reverse_buttons && app.desktop().tile_columns_first);
        assert_eq!(app.double_delay, Duration::from_millis(700));
        // Нечитаемая тема — стандартные цвета и ошибка.
        let bad = Settings { theme: "нет/такой.tvtheme".into(), ..Settings::default() };
        assert!(bad.apply(&mut app).is_err());
        assert_eq!(app.palette, standard_palette());
        locale::set_language(Language::En);
    }

    fn app_with_windows() -> Program {
        let mut app = Program::new(Box::new(HeadlessBackend::new(80, 25)), None, None);
        for (r, t, n) in [(Rect::new(0, 0, 30, 10), "Один", 1), (Rect::new(5, 5, 40, 15), "Два", 2), (Rect::new(10, 2, 50, 12), "Без номера", 0)]
        {
            let mut w = Window::new(r, t, n);
            w.group.vs.options |= crate::of::TILEABLE;
            app.insert_window(w);
        }
        app
    }

    #[test]
    fn windows_remembered_and_restored() {
        let mut app = app_with_windows();
        // Переставим окна и распахнём второе.
        for v in app.desktop_mut().group.children.iter_mut() {
            if let Some(w) = v.as_window_mut() {
                if w.number == 1 {
                    w.group.change_bounds(Rect::new(20, 3, 60, 13));
                } else if w.number == 2 {
                    w.zoom(Point::new(80, 25));
                }
            }
        }
        let mut s = Settings::default();
        s.windows.push(WindowPlace { key: "закрытое".into(), bounds: Rect::new(0, 0, 20, 8), zoomed: false });
        s.capture_windows(&app);
        let keys: Vec<&str> = s.windows.iter().map(|w| w.key.as_str()).collect();
        assert_eq!(keys, vec!["1", "2", "Без номера", "закрытое"]);
        assert_eq!(s.window("1").unwrap().bounds, Rect::new(20, 3, 60, 13));
        assert_eq!(s.window("2").unwrap(), &WindowPlace { key: "2".into(), bounds: Rect::new(5, 5, 40, 15), zoomed: true });

        // Новый запуск: окна на местах по умолчанию → на запомненные.
        let text = s.to_text();
        let s = Settings::parse(&text).unwrap();
        let mut app = app_with_windows();
        s.restore_windows(&mut app);
        let find = |app: &Program, n: i16| {
            app.desktop()
                .group
                .children
                .iter()
                .filter_map(|v| v.as_window())
                .find(|w| w.number == n)
                .map(|w| (w.group.vs.bounds(), w.is_zoomed(), w.normal_bounds()))
                .unwrap()
        };
        assert_eq!(find(&app, 1).0, Rect::new(20, 3, 60, 13));
        let (b, z, n) = find(&app, 2);
        assert!(z && b == Rect::new(0, 0, 80, 25) && n == Rect::new(5, 5, 40, 15), "{:?}", (b, z, n));

        // Окно, которое открывается позже, — тоже на своё место.
        let mut w = Window::new(Rect::new(0, 0, 30, 10), "Один", 1);
        assert!(s.place_window(&app, &mut w));
        assert_eq!(w.group.vs.bounds(), Rect::new(20, 3, 60, 13));
    }

    #[test]
    fn restore_fits_smaller_screen() {
        let s =
            Settings { windows: vec![WindowPlace { key: "1".into(), bounds: Rect::new(100, 40, 200, 80), zoomed: false }], ..Settings::default() };
        let mut app = Program::new(Box::new(HeadlessBackend::new(80, 25)), None, None);
        let id = app.insert_window(Window::new(Rect::new(0, 0, 30, 10), "Один", 1));
        s.restore_windows(&mut app);
        let b = app.find::<Window>(id).unwrap().group.vs.bounds();
        assert_eq!(b, Rect::new(0, 0, 80, 25), "стол без меню и строки состояния — весь экран");
    }

    #[test]
    fn cascade_and_tile_at_start() {
        let mut app = app_with_windows();
        Settings { windows_at_start: WindowsAtStart::Tile, ..Settings::default() }.restore_windows(&mut app);
        let total: i32 = app.desktop().group.children.iter().filter_map(|v| v.as_window()).map(|w| w.group.vs.size.x * w.group.vs.size.y).sum();
        assert_eq!(total, 80 * 25, "мозаика занимает весь стол");
    }

    #[test]
    fn available_themes_has_builtins() {
        let t = available_themes(&[]);
        assert_eq!(t.len(), 4);
        assert_eq!(t[0].0, "builtin:classic");
        assert!(resolve_theme(&t[1].0).unwrap().is_some());
        assert!(resolve_theme("").unwrap().is_none());
        assert!(resolve_theme("builtin:нет").is_err());
    }

    fn dialog_app(keys: &[crate::KeyEvent]) -> Program {
        // Диалог должен помещаться в обычный терминал 80 × 25.
        let mut b = HeadlessBackend::new(80, 25);
        for k in keys {
            b.push_key(*k);
        }
        Program::new(Box::new(b), None, None)
    }

    #[test]
    fn dialog_ok_applies_theme() {
        locale::set_language(Language::En);
        let down = crate::KeyEvent::new(crate::Key::Down, crate::Mods::NONE);
        let mut app = dialog_app(&[down, crate::kb::ENTER]);
        let mut s = Settings::default();
        let themes = available_themes(&[]);
        assert!(settings_dialog_with(&mut app, &mut s, &themes));
        assert_eq!(s.theme, "builtin:classic");
        assert_eq!(app.palette, palettes::APP_COLOR.to_vec());
        locale::set_language(Language::En);
    }

    #[test]
    fn dialog_cancel_restores() {
        locale::set_language(Language::En);
        let down = crate::KeyEvent::new(crate::Key::Down, crate::Mods::NONE);
        let mut app = dialog_app(&[down, down, crate::kb::ESC]);
        let mut s = Settings { pattern: '▓', ..Settings::default() };
        s.apply(&mut app).unwrap();
        let before = app.palette.clone();
        let s0 = s.clone();
        assert!(!settings_dialog_with(&mut app, &mut s, &available_themes(&[])));
        assert_eq!(s, s0);
        assert_eq!(app.palette, before);
        assert_eq!(app.desktop().pattern(), '▓');
        locale::set_language(Language::En);
    }

    #[test]
    fn dialog_defaults_button() {
        locale::set_language(Language::En);
        let alt_d = crate::KeyEvent::new(crate::Key::Char('d'), crate::Mods::ALT);
        let mut app = dialog_app(&[alt_d, crate::kb::ENTER]);
        let mut s = Settings { pattern: '▓', reverse_buttons: true, language: LanguageChoice::En, ..Settings::default() };
        s.set("x", Json::from_i64(1));
        assert!(settings_dialog_with(&mut app, &mut s, &available_themes(&[])));
        assert_eq!(s.pattern, '░');
        assert!(!s.reverse_buttons);
        assert_eq!(s.get("x").and_then(Json::as_i64), Some(1), "свои настройки не сбрасываются");
        locale::set_language(Language::En);
    }

    #[test]
    fn dialog_looks_right() {
        use std::cell::RefCell;
        use std::rc::Rc;
        let f12 = crate::KeyEvent::new(crate::Key::F(12), crate::Mods::NONE);
        for lang in [Language::Ru, Language::En] {
            locale::set_language(lang);
            let mut app = dialog_app(&[f12, crate::kb::ESC]);
            // Снимок экрана, пока диалог открыт: F12 никто не обрабатывает,
            // и событие попадает в modal_hook.
            let shot = Rc::new(RefCell::new(String::new()));
            let sh = shot.clone();
            app.modal_hook = Some(Box::new(move |app, _| {
                app.redraw();
                *sh.borrow_mut() = app.screen_text();
            }));
            let mut s = Settings::default();
            settings_dialog_with(&mut app, &mut s, &available_themes(&[]));
            let text = shot.borrow().clone();
            if std::env::var("SHOW_DIALOG").is_ok() {
                println!("{}", text);
            }
            let want: &[&str] = if lang == Language::Ru {
                &[
                    "Настройки",
                    "Тема",
                    "Стандартные цвета",
                    "Gruvbox",
                    "Цвета терминала",
                    "Узор фона",
                    "Язык",
                    "Русский",
                    "Окна при запуске",
                    "Мозаикой",
                    "Поменять кнопки мыши",
                    "Двойной щелчок",
                    "400",
                    "По умолчанию",
                ]
            } else {
                &[
                    "Settings",
                    "Theme",
                    "Terminal colors",
                    "Desktop pattern",
                    "Language",
                    "Windows at start",
                    "Swap mouse buttons",
                    "Double click",
                    "Defaults",
                ]
            };
            for w in want {
                assert!(text.contains(w), "нет «{}»:\n{}", w, text);
            }
        }
        locale::set_language(Language::En);
    }
}
