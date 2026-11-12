//! Темы в TVCreator: где искать файлы `.tvtheme` и как запомнить выбор.
//!
//! Темы ищутся (по порядку, без повторов):
//! 1. в каталоге из переменной окружения `TVCREATOR_THEMES`;
//! 2. в `themes/` рядом с программой `tvcreator`;
//! 3. в `themes/` текущего каталога;
//! 4. в `themes/` репозитория tvision-rs (если TVCreator собран из него);
//! 5. всегда есть встроенные: «Классическая», «Gruvbox», «Чёрно-белая»,
//!    «Монохромная».
//!
//! Выбор хранится в стандартных настройках `tvision::settings` — в файле
//! `tvcreator.settings.json` рядом с программой.

use std::path::{Path, PathBuf};

use tvision::settings;
use tvision::theme::{Base, Theme};

/// Тема и её значение для настроек: путь к файлу (внутри каталога
/// программы — относительный) или `builtin:основа`.
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeEntry {
    pub key: String,
    pub theme: Theme,
}

impl ThemeEntry {
    /// Тема из файла (путь запоминается полным).
    pub fn from_file(p: &Path, theme: Theme) -> ThemeEntry {
        let p = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
        ThemeEntry { key: p.display().to_string(), theme }
    }
}

/// Каталоги, где искать темы (существующие, без повторов).
pub fn theme_dirs() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = Vec::new();
    if let Ok(d) = std::env::var("TVCREATOR_THEMES") {
        v.push(PathBuf::from(d));
    }
    v.extend(settings::theme_dirs()); // рядом с программой и в текущем каталоге
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

/// Все темы из каталогов и встроенные; одинаковые — один раз
/// ([`settings::available_themes`]).
pub fn load_themes(dirs: &[PathBuf]) -> Vec<ThemeEntry> {
    settings::available_themes(dirs).into_iter().map(|(key, theme)| ThemeEntry { key, theme }).collect()
}

/// Найти тему по ключу из настроек.
pub fn find_key(themes: &[ThemeEntry], key: &str) -> Option<usize> {
    themes.iter().position(|e| e.key == key).or_else(|| {
        // Встроенная может быть в списке под именем файла с той же палитрой.
        key.strip_prefix("builtin:").and_then(Base::from_key).and_then(|b| themes.iter().position(|e| e.theme.palette == b.palette()))
    })
}

/// Тема по ключу: из списка, а если её там нет — прочитать
/// ([`settings::resolve_theme`]).
pub fn resolve(themes: &[ThemeEntry], key: &str) -> Option<ThemeEntry> {
    if let Some(i) = find_key(themes, key) {
        return Some(themes[i].clone());
    }
    settings::resolve_theme(key).ok().flatten().map(|theme| ThemeEntry { key: key.to_string(), theme })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_always_present() {
        let t = load_themes(&[]);
        assert_eq!(t.len(), 4);
        assert!(t.iter().all(|e| e.key.starts_with("builtin:")));
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
        assert_eq!(find_key(&t, &sea.key).map(|i| &t[i].theme.name), Some(&sea.theme.name));
    }
}
