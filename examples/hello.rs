//! Порт классического примера `hello.cpp` из Turbo Vision.
//!
//! Запуск: `cargo run --example hello`
//! Меню «Hello» → «Greeting...» (или Alt+G) открывает диалог;
//! Alt+X — выход.

use tvision::*;

/// Своя команда приложения (`GreetThemCmd`).
const GREET_THEM_CMD: u16 = 100;

/// Аналог класса `THelloApp`: обрабатывает команды, которые не обработали окна.
struct HelloApp;

impl HelloApp {
    /// Аналог `THelloApp::greetingBox()`.
    fn greeting_box(&self, app: &mut Program) {
        let mut d = Dialog::new(Rect::new(25, 5, 55, 16), "Hello, World!");
        d.add(StaticText::new(Rect::new(3, 5, 15, 6), "How are you?"));
        d.add(Button::new(Rect::new(16, 2, 28, 4), "Terrific", cm::CANCEL, bf::NORMAL));
        d.add(Button::new(Rect::new(16, 4, 28, 6), "Ok", cm::CANCEL, bf::NORMAL));
        d.add(Button::new(Rect::new(16, 6, 28, 8), "Lousy", cm::CANCEL, bf::NORMAL));
        d.add(Button::new(Rect::new(16, 8, 28, 10), "Cancel", cm::CANCEL, bf::NORMAL));
        // `deskTop->execView(d)` — модальный запуск диалога.
        app.exec_dialog(d);
    }
}

impl AppHandler for HelloApp {
    /// Аналог `THelloApp::handleEvent`.
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(GREET_THEM_CMD) {
            self.greeting_box(app);
            ev.clear();
        }
    }
}

fn main() -> std::io::Result<()> {
    // Аналог `initMenuBar`.
    let menu = Menu::new().submenu(
        "~H~ello",
        Menu::new().item_key("~G~reeting...", GREET_THEM_CMD, kb::alt('G'), "Alt-G").separator().item_key("E~x~it", cm::QUIT, kb::ALT_X, "Alt-X"),
    );
    // Аналог `initStatusLine`.
    let status = vec![StatusDef::all(vec![StatusItem::new("~Alt-X~ Exit", kb::ALT_X, cm::QUIT), StatusItem::new("", kb::F10, cm::MENU)])];

    let mut app = Program::with_terminal(Some(menu), Some(status))?;
    app.run(&mut HelloApp);
    Ok(())
}
