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
//! Выбор хранится в стандартных настройках `tvision::settings` — в файле
//! `tvcreator.settings.json` рядом с программой.

use std::path::{Path, PathBuf};

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
}
