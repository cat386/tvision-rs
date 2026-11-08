//! Темы в программе: файлы `.tvtheme` из каталога `themes/`.
//!
//! Запуск:
//! ```text
//! cargo run --example themes                    # темы из themes/ репозитория
//! cargo run --example themes -- мой_каталог     # темы из своего каталога
//! TVISION_THEME=themes/sea.tvtheme cargo run --example themes   # сразу в теме
//! ```
//!
//! * меню «Тема» — по пункту на каждый файл темы;
//! * F3 — «Выбрать тему…»: список с живым предпросмотром (`choose_theme`);
//! * «Тема → Открыть файл…» — любой `.tvtheme` (`app.load_theme`).
//!
//! Темы создаются в редакторе тем: `cargo run -p tvtheme`.

use std::path::{Path, PathBuf};

use tvision::layout::*;
use tvision::theme::{choose_theme, find_themes, Theme};
use tvision::*;

const CM_CHOOSE: u16 = 150;
const CM_OPEN_THEME: u16 = 151;
const CM_DIALOG: u16 = 152;
/// Пункты меню тем: 200 + номер темы.
const CM_THEME: u16 = 200;

pub fn themes_dir() -> PathBuf {
    std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("themes"))
}

pub fn load_themes(dir: &Path) -> Vec<Theme> {
    find_themes(dir).0.into_iter().map(|(_, t)| t).collect()
}

fn menu(themes: &[Theme]) -> Menu {
    let mut m = Menu::new();
    for (i, t) in themes.iter().enumerate().take(50) {
        m = m.item(&t.name, CM_THEME + i as u16);
    }
    m = m.separator().item_key("~В~ыбрать...", CM_CHOOSE, kb::F3, "F3").item("~О~ткрыть файл...", CM_OPEN_THEME);
    Menu::new()
        .submenu("~Ф~айл", Menu::new().item_key("~Д~иалог", CM_DIALOG, kb::F2, "F2").separator().item_key("~В~ыход", cm::QUIT, kb::ALT_X, "Alt-X"))
        .submenu("~Т~ема", m)
}

fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![
        StatusItem::new("~F2~ Диалог", kb::F2, CM_DIALOG),
        StatusItem::new("~F3~ Тема", kb::F3, CM_CHOOSE),
        StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT),
        StatusItem::new("", kb::F10, cm::MENU),
    ])]
}

fn demo_dialog() -> Dialog {
    let mut d = Dialog::centered(40, 12, "Образец");
    let name = d.add(InputLine::new(Rect::default(), 30));
    let ln = d.add(Label::new(Rect::default(), "~И~мя:", Some(name)));
    let opts = d.add(Cluster::check_boxes(Rect::default(), &["~З~апомнить", "~У~ведомлять"]));
    let ok = d.add(Button::new(Rect::default(), "O~K~", cm::OK, bf::DEFAULT));
    let cancel = d.add(Button::new(Rect::default(), "Отмена", cm::CANCEL, bf::NORMAL));
    d.set_layout(
        VBox::new().margins(2, 1, 2, 0).add(Form::new().row(ln, name).row_wide(opts)).space(1).add(HBox::new().stretch(1).add(ok).add(cancel)),
    );
    d.adjust_size();
    d
}

fn demo_window(r: Rect) -> Window {
    let mut w = Window::new(r, "Текст", 1);
    let v = w.standard_scroll_bar(true);
    let h = w.standard_scroll_bar(false);
    let mut e = Editor::new(w.interior(), Some(h), Some(v), None);
    e.set_text("Меню «Тема» — выбрать тему из каталога themes/.\nF3 — выбрать с предпросмотром.\nF2 — диалог, чтобы посмотреть кнопки и поля.\n\nСвои темы — в редакторе: cargo run -p tvtheme");
    e.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
    w.add(e);
    w
}

struct App {
    themes: Vec<Theme>,
    /// Имя текущей темы — в заголовке окна.
    current: String,
    window: ViewId,
}

impl App {
    fn show_name(&self, app: &mut Program) {
        if let Some(w) = app.find_mut::<Window>(self.window) {
            w.title = format!("Тема: {}", self.current);
        }
    }
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        let Some(cmd) = ev.command() else { return };
        match cmd {
            c if (CM_THEME..CM_THEME + self.themes.len() as u16).contains(&c) => {
                let t = &self.themes[(c - CM_THEME) as usize];
                app.set_theme(t);
                self.current = t.name.clone();
                self.show_name(app);
            }
            CM_CHOOSE => {
                if let Some(i) = choose_theme(app, &self.themes) {
                    self.current = self.themes[i].name.clone();
                    self.show_name(app);
                }
            }
            CM_OPEN_THEME => {
                let (c, d) = app.exec_dialog(FileDialog::new("*.tvtheme", "Открыть тему", "~И~мя", fd::OPEN_BUTTON, 120));
                if c != cm::CANCEL {
                    match app.load_theme(d.file_name()) {
                        Ok(t) => {
                            self.current = t.name;
                            self.show_name(app);
                        }
                        Err(e) => {
                            message_box(app, &e, mf::ERROR | mf::OK_BUTTON);
                        }
                    }
                }
            }
            CM_DIALOG => {
                app.exec_dialog(demo_dialog());
            }
            _ => return,
        }
        ev.clear();
    }
}

fn start(app: &mut Program, themes: Vec<Theme>) -> App {
    let current = themes.iter().find(|t| t.palette == app.palette).map(|t| t.name.clone()).unwrap_or_else(|| "своя".into());
    let window = app.insert_window(demo_window(Rect::new(4, 2, 70, 14)));
    let a = App { themes, current, window };
    a.show_name(app);
    a
}

fn main() -> std::io::Result<()> {
    locale::set_language(locale::Language::Ru);
    let themes = load_themes(&themes_dir());
    // with_terminal сам подключает тему из TVISION_THEME, если она задана.
    let mut app = Program::with_terminal(Some(menu(&themes)), Some(status()))?;
    let mut h = start(&mut app, themes);
    app.run(&mut h);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tvision::platform::HeadlessBackend;

    fn repo_themes() -> Vec<Theme> {
        load_themes(&Path::new(env!("CARGO_MANIFEST_DIR")).join("themes"))
    }

    #[test]
    fn all_sample_themes_load() {
        let (ok, errors) = find_themes(Path::new(env!("CARGO_MANIFEST_DIR")).join("themes"));
        assert!(errors.is_empty(), "{:?}", errors);
        let names: Vec<&str> = ok.iter().map(|(_, t)| t.name.as_str()).collect();
        for n in ["Классическая", "Gruvbox", "Морская", "Матрица", "Полночь"] {
            assert!(names.contains(&n), "нет темы «{}»: {:?}", n, names);
        }
        // Темы различаются.
        let pals: Vec<&Vec<u32>> = ok.iter().map(|(_, t)| &t.palette).collect();
        for i in 0..pals.len() {
            for j in i + 1..pals.len() {
                assert_ne!(pals[i], pals[j], "{} и {}", names[i], names[j]);
            }
        }
    }

    #[test]
    fn menu_switches_theme() {
        let themes = repo_themes();
        let sea = themes.iter().position(|t| t.name == "Морская").unwrap();
        let mut app = Program::new(Box::new(HeadlessBackend::new(80, 25)), Some(menu(&themes)), Some(status()));
        let mut h = start(&mut app, themes.clone());
        app.put_event(Event::command_event(CM_THEME + sea as u16));
        app.pump(&mut h);
        assert_eq!(app.palette, themes[sea].palette);
        app.redraw();
        assert!(app.screen_text().contains("Тема: Морская"));
        // Рабочий стол — в цветах темы.
        assert!(app.screen().cells.iter().any(|c| c.ch == '░' && c.attr == themes[sea].get(1)));
    }
}
