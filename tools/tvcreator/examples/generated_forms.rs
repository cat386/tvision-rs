//! Формы, сгенерированные TVCreator, в обычной программе.
//!
//! Формы лежат в `tools/tvcreator/forms/*.tvui`, код к ним — в
//! `examples/forms/*.rs` (получен командой
//! `cargo run -p tvcreator -- --gen forms/login.tvui -o examples/forms/login.rs`).
//!
//! Запуск: `cargo run -p tvcreator --example generated_forms`
//! F2 — «Вход», F3 — «Код доступа», окно «Почта» открыто сразу.

// Сгенерированный код не форматируем — как вывод uic.
#[rustfmt::skip]
#[path = "forms/keypad.rs"]
mod keypad;
// Сгенерированный код не форматируем — как вывод uic.
#[rustfmt::skip]
#[path = "forms/login.rs"]
mod login;
// Сгенерированный код не форматируем — как вывод uic.
#[rustfmt::skip]
#[path = "forms/mail.rs"]
mod mail;

use tvision::*;

const CM_LOGIN: u16 = 200;
const CM_KEYPAD: u16 = 201;

struct App;

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        match ev.command() {
            Some(CM_LOGIN) => {
                let (d, ui) = login::LoginDialog::create();
                let (cmd, d) = app.exec_dialog(d);
                if cmd == cm::OK {
                    // Данные — через id из сгенерированной структуры.
                    let name = d.win.group.find::<InputLine>(ui.user).map(|l| l.text()).unwrap_or_default();
                    message_box(app, &format!("\x03Здравствуйте, {}!", name), mf::INFORMATION | mf::OK_BUTTON);
                }
            }
            Some(CM_KEYPAD) => {
                let (d, ui) = keypad::KeypadDialog::create();
                // Кнопки цифр посылают свои команды (CM_KEY0…): дописываем цифру.
                let keys = [keypad::CM_KEY0, keypad::CM_KEY1, keypad::CM_KEY2, keypad::CM_KEY3, keypad::CM_KEY4, keypad::CM_KEY5, keypad::CM_KEY6];
                let code = ui.code;
                app.modal_hook = Some(Box::new(move |app: &mut Program, ev: &mut Event| {
                    let Some(digit) = ev.command().and_then(|c| keys.iter().position(|k| *k == c)) else { return };
                    if let Some(l) = app.find_mut::<InputLine>(code) {
                        let t = l.text() + &digit.to_string();
                        l.set_text(&t);
                    }
                    ev.clear();
                }));
                let (cmd, d) = app.exec_dialog(d);
                app.modal_hook = None;
                if cmd == cm::OK {
                    let code = d.win.group.find::<InputLine>(ui.code).map(|l| l.text()).unwrap_or_default();
                    message_box(app, &format!("\x03Код: {}", code), mf::INFORMATION | mf::OK_BUTTON);
                }
            }
            _ => return,
        }
        ev.clear();
    }
}

fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![
        StatusItem::new("~F2~ Вход", kb::F2, CM_LOGIN),
        StatusItem::new("~F3~ Код доступа", kb::F3, CM_KEYPAD),
        StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT),
    ])]
}

fn main() -> std::io::Result<()> {
    locale::set_language(locale::Language::Ru);
    let mut app = Program::with_terminal(None, Some(status()))?;
    let (w, _ui) = mail::MailWindow::create();
    app.insert_window(w);
    app.run(&mut App);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use tvcreator::{build, codegen, model::FormDoc};
    use tvision::*;

    fn form(name: &str) -> FormDoc {
        FormDoc::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("forms").join(format!("{}.tvui", name))).unwrap()
    }

    /// Границы всех элементов окна по порядку.
    fn geometry(g: &Group) -> Vec<(Rect, bool)> {
        g.children.iter().map(|c| (c.vs().bounds(), c.vs().is_selected())).collect()
    }

    fn same(name: &str, generated: &dyn View) {
        let built = build::build(&form(name));
        let a = built.group();
        let b = generated.as_group().unwrap();
        assert_eq!(a.vs.size, b.vs.size, "{}: размер окна", name);
        assert_eq!(geometry(a), geometry(b), "{}: расположение элементов", name);
    }

    #[test]
    fn generated_matches_runtime() {
        same("login", &super::login::LoginDialog::create().0);
        same("mail", &super::mail::MailWindow::create().0);
        same("keypad", &super::keypad::KeypadDialog::create().0);
    }

    #[test]
    fn generated_code_is_up_to_date() {
        for name in ["login", "mail", "keypad"] {
            let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/forms").join(format!("{}.rs", name));
            let on_disk = std::fs::read_to_string(&path).unwrap();
            let fresh = codegen::generate(&form(name), &format!("{}.tvui", name));
            assert!(
                on_disk == fresh,
                "{} устарел: cargo run -p tvcreator -- --gen forms/{}.tvui -o examples/forms/{}.rs",
                path.display(),
                name,
                name
            );
        }
    }

    /// Код для `include!` (как из build.rs) тоже собирается и совпадает.
    #[rustfmt::skip]
    mod included {
        include!("forms/keypad_include.rs");
    }

    #[test]
    fn include_variant_works() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/forms/keypad_include.rs");
        let fresh = codegen::generate_for_include(&form("keypad"), "keypad.tvui");
        assert!(std::fs::read_to_string(&path).unwrap() == fresh, "{} устарел", path.display());
        assert!(!fresh.contains("#!") && !fresh.contains("//!"));
        same("keypad", &included::KeypadDialog::create().0);
    }

    #[test]
    fn layouts_look_right() {
        let (d, ui) = super::login::LoginDialog::create();
        let g = &d.win.group;
        let b = |id| g.view(id).unwrap().vs().bounds();
        // Кнопка «Отмена» — у правого края, «Регистрация» — у левого.
        assert_eq!(b(ui.cancel).b.x, g.vs.size.x - 3);
        assert_eq!(b(ui.register).a.x, 3);
        // Строки ввода — после столбца подписей.
        assert_eq!(b(ui.user).a.x, b(ui.pass).a.x);
        let (w, ui) = super::mail::MailWindow::create();
        let g = &w.group;
        let folders = g.view(ui.folders).unwrap().vs().bounds();
        let letters = g.view(ui.letters).unwrap().vs().bounds();
        // 1 : 3 по ширине.
        let (fw, lw) = (folders.b.x - folders.a.x, letters.b.x - letters.a.x);
        assert!((lw as f64 / fw as f64 - 3.0).abs() < 0.3, "{} : {}", fw, lw);
    }
}
