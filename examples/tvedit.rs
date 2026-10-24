//! Порт примера `examples/tvedit` из Turbo Vision: многооконный текстовый
//! редактор с меню, файловыми диалогами, поиском и заменой.
//!
//! Запуск: `cargo run --example tvedit [файлы...]`
//!
//! Отличие от оригинала: «Выход» — Alt+X, а не Ctrl+Q, потому что Ctrl+Q
//! в редакторе — префикс команд WordStar (Ctrl+Q F — поиск, Ctrl+Q A — замена).

use std::path::{Path, PathBuf};

use tvision::*;

// Команды приложения (как в `tvedit.h`).
const CM_CHANGE_DRCT: u16 = 102;
const CM_SHOW_CLIP: u16 = 105;

struct EditorApp {
    /// Счётчик для номеров окон.
    windows: i16,
}

impl EditorApp {
    /// Открыть окно редактора (`openEditor`).
    fn open_editor(&mut self, app: &mut Program, file: &Path) {
        self.windows += 1;
        let r = app.desktop().group.vs.extent();
        let w = EditWindow::new(r, file, if self.windows < 10 { self.windows } else { 0 });
        app.insert_window(w);
    }

    /// «Открыть файл» (`fileOpen`).
    fn file_open(&mut self, app: &mut Program) {
        let d = FileDialog::new("*", "Open file", "~N~ame", fd::OPEN_BUTTON, 100);
        let (cmd, d) = app.exec_dialog(d);
        if cmd != cm::CANCEL {
            let name = d.file_name();
            self.open_editor(app, &name);
        }
    }

    /// «Сменить каталог» (`changeDir`).
    fn change_dir(&mut self, app: &mut Program) {
        let d = ChDirDialog::new(cd::NORMAL, 0);
        app.exec_dialog(d);
    }
}

impl AppHandler for EditorApp {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        let c = match ev.command() {
            Some(c) => c,
            None => return,
        };
        match c {
            cm::OPEN => self.file_open(app),
            cm::NEW => self.open_editor(app, Path::new("")),
            CM_CHANGE_DRCT => self.change_dir(app),
            CM_SHOW_CLIP => {
                let t = app.ctx.clipboard.get();
                let text = if t.is_empty() { "(пусто)".to_string() } else { t };
                message_box_rect(app, Rect::new(0, 0, 60, 12), &format!("Буфер обмена:\n{}", text), mf::INFORMATION | mf::OK_BUTTON, true);
            }
            cm::DOS_SHELL => {
                message_box(app, "\x03Выход в оболочку не поддерживается в этом примере.", mf::INFORMATION | mf::OK_BUTTON);
            }
            _ => return,
        }
        ev.clear();
    }
}

fn main() -> std::io::Result<()> {
    let menu = Menu::new()
        .submenu(
            "~F~ile",
            Menu::new()
                .item_key("~O~pen", cm::OPEN, kb::F3, "F3")
                .item_key("~N~ew", cm::NEW, kb::ctrl('N'), "Ctrl-N")
                .item_key("~S~ave", cm::SAVE, kb::F2, "F2")
                .item("S~a~ve as...", cm::SAVE_AS)
                .separator()
                .item("~C~hange dir...", CM_CHANGE_DRCT)
                .item("~D~OS shell", cm::DOS_SHELL)
                .item_key("E~x~it", cm::QUIT, kb::ALT_X, "Alt-X"),
        )
        .submenu(
            "~E~dit",
            Menu::new()
                .item_key("~U~ndo", cm::UNDO, kb::ctrl('U'), "Ctrl-U")
                .item("~R~edo", cm::REDO)
                .separator()
                .item_key("Cu~t~", cm::CUT, KeyEvent::new(Key::Del, Mods::SHIFT), "Shift-Del")
                .item_key("~C~opy", cm::COPY, KeyEvent::new(Key::Ins, Mods::CTRL), "Ctrl-Ins")
                .item_key("~P~aste", cm::PASTE, KeyEvent::new(Key::Ins, Mods::SHIFT), "Shift-Ins")
                .item("~S~how clipboard", CM_SHOW_CLIP)
                .separator()
                .item_key("~C~lear", cm::CLEAR, KeyEvent::new(Key::Del, Mods::CTRL), "Ctrl-Del"),
        )
        .submenu("~S~earch", Menu::new().item("~F~ind...", cm::FIND).item("~R~eplace...", cm::REPLACE).item("~S~earch again", cm::SEARCH_AGAIN))
        .submenu(
            "~W~indows",
            Menu::new()
                .item_key("~S~ize/move", cm::RESIZE, kb::CTRL_F5, "Ctrl-F5")
                .item_key("~Z~oom", cm::ZOOM, kb::F5, "F5")
                .item("~T~ile", cm::TILE)
                .item("C~a~scade", cm::CASCADE)
                .item_key("~N~ext", cm::NEXT, kb::F6, "F6")
                .item_key("~P~revious", cm::PREV, kb::SHIFT_F6, "Shift-F6")
                .item_key("~C~lose", cm::CLOSE, kb::ctrl('W'), "Ctrl-W"),
        );

    let status = vec![StatusDef::all(vec![
        StatusItem::new("", kb::ALT_X, cm::QUIT),
        StatusItem::new("~F2~ Save", kb::F2, cm::SAVE),
        StatusItem::new("~F3~ Open", kb::F3, cm::OPEN),
        StatusItem::new("~Ctrl-W~ Close", kb::ALT_F3, cm::CLOSE),
        StatusItem::new("~F5~ Zoom", kb::F5, cm::ZOOM),
        StatusItem::new("~F6~ Next", kb::F6, cm::NEXT),
        StatusItem::new("~F10~ Menu", kb::F10, cm::MENU),
        StatusItem::new("", kb::SHIFT_F6, cm::PREV),
        StatusItem::new("", kb::CTRL_F5, cm::RESIZE),
    ])];

    let mut app = Program::with_terminal(Some(menu), Some(status))?;
    let mut ed = EditorApp { windows: 0 };
    // Файлы из командной строки (как в оригинале).
    for arg in std::env::args().skip(1) {
        ed.open_editor(&mut app, &PathBuf::from(arg));
    }
    app.run(&mut ed);
    Ok(())
}
