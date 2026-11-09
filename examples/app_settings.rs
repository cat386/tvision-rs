//! Стандартный диалог «Настройки» и файл настроек рядом с программой.
//!
//! Запуск:
//! ```text
//! cargo run --example app_settings
//! ```
//!
//! * **F9** (или «Файл → Настройки…») — диалог: тема, цвета терминала,
//!   узор фона, язык, окна при запуске, мышь;
//! * окна можно двигать, растягивать, распахивать (F5) — при выходе
//!   (Alt-X) их места запоминаются, и при следующем запуске окна встанут
//!   туда же;
//! * настройки лежат рядом с исполнимым файлом:
//!   `target/debug/examples/app_settings.settings.json` (путь показан в
//!   окне «О настройках»). Файл можно править руками.
//!
//! Язык переключается сразу: меню, строка состояния и заголовки окон
//! собираются заново по [`locale::language`].

use std::path::PathBuf;
use std::rc::Rc;

use tvision::locale::Language;
use tvision::settings::{settings_dialog, Settings};
use tvision::*;

const CM_SETTINGS: u16 = 150;

/// Надпись на текущем языке.
fn t(en: &'static str, ru: &'static str) -> &'static str {
    if locale::language() == Language::Ru {
        ru
    } else {
        en
    }
}

fn menu() -> Menu {
    Menu::new()
        .submenu(
            t("~F~ile", "~Ф~айл"),
            Menu::new().item_key(t("~S~ettings...", "~Н~астройки..."), CM_SETTINGS, kb::f(9), "F9").separator().item_key(
                t("E~x~it", "~В~ыход"),
                cm::QUIT,
                kb::ALT_X,
                "Alt-X",
            ),
        )
        .submenu(
            t("~W~indow", "~О~кно"),
            Menu::new()
                .item(t("~T~ile", "~М~озаика"), cm::TILE)
                .item(t("C~a~scade", "~К~аскад"), cm::CASCADE)
                .separator()
                .item_key(t("~Z~oom", "~Р~аспахнуть"), cm::ZOOM, kb::F5, "F5")
                .item_key(t("~N~ext", "~С~ледующее"), cm::NEXT, kb::F6, "F6"),
        )
}

fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![
        StatusItem::new(t("~F9~ Settings", "~F9~ Настройки"), kb::f(9), CM_SETTINGS),
        StatusItem::new(t("~F5~ Zoom", "~F5~ Распахнуть"), kb::F5, cm::ZOOM),
        StatusItem::new(t("~F6~ Next", "~F6~ Следующее"), kb::F6, cm::NEXT),
        StatusItem::new(t("~Alt-X~ Exit", "~Alt-X~ Выход"), kb::ALT_X, cm::QUIT),
        StatusItem::new("", kb::F10, cm::MENU),
    ])]
}

/// Заголовки окон: номер окна → (английский, русский).
const TITLES: [(&str, &str); 2] = [("Notes", "Заметки"), ("About settings", "О настройках")];

fn title(n: i16) -> &'static str {
    let (en, ru) = TITLES[(n - 1) as usize];
    t(en, ru)
}

fn text_window(r: Rect, n: i16, text: &str) -> Window {
    let mut w = Window::new(r, title(n), n);
    // Окна участвуют в мозаике и каскаде.
    w.group.vs.options |= of::TILEABLE;
    let v = w.standard_scroll_bar(true);
    let mut e = Editor::new(w.interior(), None, Some(v), None);
    e.set_text(text);
    e.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
    w.add(e);
    w
}

fn about_text(path: &std::path::Path) -> String {
    format!(
        "{}\n\n{}\n\n{}",
        t("Settings file (next to the executable):", "Файл настроек (рядом с исполнимым файлом):"),
        path.display(),
        t(
            "F9 — settings. Move or zoom the windows,\nexit with Alt-X and start again: the windows\nwill be where you left them.",
            "F9 — настройки. Передвиньте или распахните окна,\nвыйдите (Alt-X) и запустите снова: окна\nвстанут на свои места."
        )
    )
}

struct App {
    settings: Settings,
    path: PathBuf,
    windows: Vec<ViewId>,
}

impl App {
    /// Язык сменился — надписи программы заново.
    fn retranslate(&self, app: &mut Program) {
        if let Some(id) = app.menu_bar_id {
            if let Some(m) = app.find_mut::<MenuBar>(id) {
                m.menu = Rc::new(menu());
            }
        }
        if let Some(s) = app.status_line_mut() {
            s.defs = status();
        }
        for &id in &self.windows {
            if let Some(w) = app.find_mut::<Window>(id) {
                w.title = title(w.number).to_string();
            }
        }
    }
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() != Some(CM_SETTINGS) {
            return;
        }
        ev.clear();
        let lang = locale::language();
        if settings_dialog(app, &mut self.settings) {
            if let Err(e) = self.settings.save(&self.path) {
                message_box(app, &e, mf::ERROR | mf::OK_BUTTON);
            }
            if locale::language() != lang {
                self.retranslate(app);
            }
        }
    }
}

/// Программа с настройками: всё, что делается при запуске.
fn start(app: &mut Program, settings: Settings, path: PathBuf) -> App {
    let theme_error = settings.apply(app).err();
    let w1 = app.insert_window(text_window(Rect::new(1, 1, 45, 14), 1, t("Write anything here.", "Пишите здесь что угодно.")));
    let w2 = app.insert_window(text_window(Rect::new(20, 6, 76, 20), 2, &about_text(&path)));
    settings.restore_windows(app);
    if let Some(e) = theme_error {
        message_box(app, &e, mf::WARNING | mf::OK_BUTTON);
    }
    App { settings, path, windows: vec![w1, w2] }
}

fn main() -> std::io::Result<()> {
    let path = Settings::default_path();
    let settings = Settings::load_or_default(&path);
    // Язык — до создания меню: от него зависят надписи.
    settings.apply_language();
    let mut app = Program::with_terminal(Some(menu()), Some(status()))?;
    let mut h = start(&mut app, settings, path);
    app.run(&mut h);
    // При выходе — запомнить, где стояли окна.
    h.settings.capture_windows(&app);
    let _ = h.settings.save(&h.path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tvision::platform::HeadlessBackend;
    use tvision::settings::{WindowPlace, WindowsAtStart};

    fn run(settings: Settings, path: PathBuf, keys: &[KeyEvent], cmds: &[u16]) -> (Program, App) {
        let mut b = HeadlessBackend::new(80, 25);
        for k in keys {
            b.push_key(*k);
        }
        settings.apply_language();
        let mut app = Program::new(Box::new(b), Some(menu()), Some(status()));
        let mut h = start(&mut app, settings, path);
        for c in cmds {
            app.put_event(Event::command_event(*c));
        }
        app.pump(&mut h);
        (app, h)
    }

    #[test]
    fn settings_survive_restart() {
        let path = std::env::temp_dir().join(format!("app-settings-example-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);
        // Первый запуск: русский язык, окно 1 — в другом месте.
        let first = Settings { language: settings::LanguageChoice::Ru, theme: "builtin:classic".into(), ..Settings::default() };
        let (mut app, mut h) = run(first, path.clone(), &[], &[]);
        let id = h.windows[0];
        app.find_mut::<Window>(id).unwrap().group.change_bounds(Rect::new(30, 2, 70, 12));
        h.settings.capture_windows(&app);
        h.settings.save(&path).unwrap();

        // Второй запуск — из файла.
        let s = Settings::load(&path).unwrap();
        assert_eq!(s.window("1"), Some(&WindowPlace { key: "1".into(), bounds: Rect::new(30, 2, 70, 12), zoomed: false }));
        let (mut app, h) = run(s, path.clone(), &[], &[]);
        assert_eq!(app.find::<Window>(h.windows[0]).unwrap().group.vs.bounds(), Rect::new(30, 2, 70, 12));
        app.redraw();
        assert!(app.screen_text().contains("Заметки") && app.screen_text().contains("Настройки"));
        assert_eq!(app.palette, tvision::color::palettes::APP_COLOR.to_vec());
        let _ = std::fs::remove_file(&path);
        locale::set_language(Language::En);
    }

    #[test]
    fn dialog_switches_language_and_saves() {
        let path = std::env::temp_dir().join(format!("app-settings-example-lang-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);
        // В диалоге: Alt+L — «Язык», ↓↓ — «Русский», Enter — OK.
        let alt_l = KeyEvent::new(Key::Char('l'), Mods::ALT);
        let down = KeyEvent::new(Key::Down, Mods::NONE);
        let s = Settings { language: settings::LanguageChoice::En, windows_at_start: WindowsAtStart::AsProgram, ..Settings::default() };
        let (mut app, _h) = run(s, path.clone(), &[alt_l, down, down, kb::ENTER], &[CM_SETTINGS]);
        assert_eq!(locale::language(), Language::Ru);
        app.redraw();
        let text = app.screen_text();
        assert!(text.contains("Файл") && text.contains("Заметки"), "{}", text);
        assert_eq!(Settings::load(&path).unwrap().language, settings::LanguageChoice::Ru);
        let _ = std::fs::remove_file(&path);
        locale::set_language(Language::En);
    }
}
