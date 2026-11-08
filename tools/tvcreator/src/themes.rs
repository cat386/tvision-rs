//! Темы в TVCreator: где искать файлы `.tvtheme` и как запомнить выбор.
//!
//! Темы ищутся (по порядку, без повторов):
//! 1. в каталоге из переменной окружения `TVCREATOR_THEMES`;
//! 2. в `themes/` текущего каталога;
//! 3. в `themes/` рядом с программой `tvcreator`;
//! 4. в `themes/` репозитория tvision-rs (если TVCreator собран из него);
//! 5. всегда есть встроенные: «Классическая», «Gruvbox», «Чёрно-белая»,
//!    «Монохромная».
//!
//! Выбор хранится в файле настроек (`settings.json`): в каталоге из
//! `TVCREATOR_CONFIG`, иначе `$XDG_CONFIG_HOME/tvcreator`,
//! `~/.config/tvcreator`, на Windows — `%APPDATA%\tvcreator`.

use std::path::{Path, PathBuf};

use tvision::json::{parse_json, Json};
use tvision::theme::{find_themes, Base, Theme};

/// Тема и откуда она (`None` — встроенная).
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeEntry {
    pub theme: Theme,
    pub path: Option<PathBuf>,
}

impl ThemeEntry {
    /// Как запомнить в настройках: путь к файлу или `builtin:основа`.
    pub fn key(&self) -> String {
        match &self.path {
            Some(p) => p.display().to_string(),
            None => format!("builtin:{}", self.theme.base.key()),
        }
    }
}

/// Каталоги, где искать темы (существующие, без повторов).
pub fn theme_dirs() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = Vec::new();
    if let Ok(d) = std::env::var("TVCREATOR_THEMES") {
        v.push(PathBuf::from(d));
    }
    v.push(PathBuf::from("themes"));
    if let Some(d) = std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.join("themes"))) {
        v.push(d);
    }
    v.push(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes"));
    let mut out: Vec<PathBuf> = Vec::new();
    for d in v {
        if let Ok(c) = d.canonicalize() {
            if c.is_dir() && !out.contains(&c) {
                out.push(c);
            }
        }
    }
    out
}

/// Все темы из каталогов и встроенные; одинаковые палитры — один раз.
pub fn load_themes(dirs: &[PathBuf]) -> Vec<ThemeEntry> {
    let mut out: Vec<ThemeEntry> = Vec::new();
    for d in dirs {
        for (p, t) in find_themes(d).0 {
            if !out.iter().any(|e| e.theme.palette == t.palette && e.theme.name == t.name) {
                out.push(ThemeEntry { theme: t, path: Some(p) });
            }
        }
    }
    for b in Base::ALL {
        let t = Theme::builtin(b);
        if !out.iter().any(|e| e.theme.palette == t.palette) {
            out.push(ThemeEntry { theme: t, path: None });
        }
    }
    out
}

/// Найти тему по ключу из настроек.
pub fn find_key(themes: &[ThemeEntry], key: &str) -> Option<usize> {
    themes.iter().position(|e| e.key() == key).or_else(|| {
        // Файл мог переехать — тогда по встроенной основе.
        key.strip_prefix("builtin:").and_then(Base::from_key).and_then(|b| themes.iter().position(|e| e.theme.palette == b.palette()))
    })
}

/// Тема по ключу: из списка, а если файла в списке нет — прочитать.
pub fn resolve(themes: &[ThemeEntry], key: &str) -> Option<ThemeEntry> {
    if let Some(i) = find_key(themes, key) {
        return Some(themes[i].clone());
    }
    let p = PathBuf::from(key);
    Theme::load(&p).ok().map(|theme| ThemeEntry { theme, path: Some(p) })
}

// ----------------------------------------------------------------------
// Настройки
// ----------------------------------------------------------------------

/// Что запоминает TVCreator между запусками.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Settings {
    /// Тема среды (ключ [`ThemeEntry::key`]); пусто — цвета по умолчанию.
    pub theme: String,
    /// Тема формы на холсте и в предпросмотре; пусто — как у среды.
    pub form_theme: String,
}

/// Файл настроек.
pub fn settings_path() -> Option<PathBuf> {
    let dir = if let Ok(d) = std::env::var("TVCREATOR_CONFIG") {
        PathBuf::from(d)
    } else if let Ok(d) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(d).join("tvcreator")
    } else if let Ok(d) = std::env::var("APPDATA") {
        PathBuf::from(d).join("tvcreator")
    } else {
        PathBuf::from(std::env::var("HOME").ok()?).join(".config").join("tvcreator")
    };
    Some(dir.join("settings.json"))
}

impl Settings {
    /// Прочитать; нет файла или он испорчен — настройки по умолчанию.
    pub fn load(path: &Path) -> Settings {
        let Ok(text) = std::fs::read_to_string(path) else { return Settings::default() };
        let Ok(j) = parse_json(&text) else { return Settings::default() };
        let s = |k: &str| j.get(k).and_then(Json::as_str).unwrap_or("").to_string();
        Settings { theme: s("theme"), form_theme: s("form_theme") }
    }

    /// Записать (каталог создаётся). Ошибка не страшна — просто не запомнится.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(d) = path.parent() {
            std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
        }
        let j = Json::object(vec![("theme", Json::from_str(&self.theme)), ("form_theme", Json::from_str(&self.form_theme))]);
        std::fs::write(path, j.pretty() + "\n").map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_always_present() {
        let t = load_themes(&[]);
        assert_eq!(t.len(), 4);
        assert!(t.iter().all(|e| e.path.is_none()));
        assert_eq!(find_key(&t, "builtin:gruvbox"), Some(1));
    }

    #[test]
    fn repo_themes_found_without_duplicates() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes").canonicalize().unwrap();
        let t = load_themes(&[dir.clone(), dir]);
        let names: Vec<&str> = t.iter().map(|e| e.theme.name.as_str()).collect();
        assert!(names.contains(&"Морская") && names.contains(&"Матрица"), "{:?}", names);
        // Классическая из файла совпадает со встроенной — встроенной второй раз нет.
        assert_eq!(names.iter().filter(|n| **n == "Классическая").count(), 1);
        let sea = t.iter().find(|e| e.theme.name == "Морская").unwrap();
        assert_eq!(find_key(&t, &sea.key()).map(|i| &t[i].theme.name), Some(&sea.theme.name));
    }

    #[test]
    fn settings_roundtrip() {
        let p = std::env::temp_dir().join(format!("tvcreator-cfg-{}", std::process::id())).join("settings.json");
        let s = Settings { theme: "builtin:classic".into(), form_theme: "/x/sea.tvtheme".into() };
        s.save(&p).unwrap();
        assert_eq!(Settings::load(&p), s);
        std::fs::write(&p, "{").unwrap();
        assert_eq!(Settings::load(&p), Settings::default());
        let _ = std::fs::remove_dir_all(p.parent().unwrap());
    }
}
