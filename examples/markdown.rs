//! Просмотр Markdown: cargo run --example markdown -- README.md
//! F3 — открыть UTF-8 файл, стрелки/PgUp/PgDn/Home/End — прокрутка.
use std::io::Read;
use std::path::Path;
use tvision::*;

const DEMO: &str = "# MarkdownViewer\n\nВиджет для просмотра **Markdown** в *Turbo Vision*.\n\n## Возможности\n- Заголовки и списки\n- Кириллица и перенос строк\n- `inline code` и блоки кода\n\n> F3 — открыть файл, Alt+X — выйти.\n\n```rust\nlet viewer = MarkdownViewer::new(bounds, markdown, scroll);\n```\n\n[Исходники](https://github.com/cat386/tvision-rs)\n\n---\nHTML, изображения и ссылки не выполняются.\n";

fn open_document(app: &mut Program, title: &str, source: &str) {
    let bounds = app.desktop().group.vs.extent();
    let mut window = Window::new(bounds, title, 0);
    let scroll = window.standard_scroll_bar(true);
    window.add(MarkdownViewer::new(Rect::new(1, 1, bounds.width() - 1, bounds.height() - 1), source, Some(scroll)));
    app.insert_window(window);
}

fn read_document(path: &Path) -> std::io::Result<String> {
    // Ошибки чтения не заменяются пустым документом; ограничиваем размер примера.
    let mut bytes = Vec::new();
    std::fs::File::open(path)?.take(4 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Файл больше 4 МиБ"));
    }
    String::from_utf8(bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

fn open_file(app: &mut Program, path: &Path) {
    match read_document(path) {
        Ok(source) => open_document(app, &path.to_string_lossy(), source.trim_start_matches('\u{feff}')),
        Err(error) => {
            message_box(app, &format!("Не удалось прочитать {}: {error}", path.display()), mf::ERROR | mf::OK_BUTTON);
        }
    }
}

struct ViewerApp;
impl AppHandler for ViewerApp {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(cm::OPEN) {
            let dialog = FileDialog::new("*.md", "Открыть Markdown", "~И~мя", fd::OPEN_BUTTON, 101);
            let (command, dialog) = app.exec_dialog(dialog);
            if command != cm::CANCEL {
                open_file(app, &dialog.file_name());
            }
            ev.clear();
        }
    }
}

fn main() -> std::io::Result<()> {
    locale::set_language(locale::Language::Ru);
    let menu =
        Menu::new().submenu("~Ф~айл", Menu::new().item_key("~О~ткрыть", cm::OPEN, kb::F3, "F3").item_key("В~ы~ход", cm::QUIT, kb::ALT_X, "Alt+X"));
    let status = vec![StatusDef::all(vec![StatusItem::new("~F3~ Открыть", kb::F3, cm::OPEN), StatusItem::new("~Alt+X~ Выход", kb::ALT_X, cm::QUIT)])];
    let mut app = Program::with_terminal(Some(menu), Some(status))?;
    if let Some(path) = std::env::args_os().nth(1) {
        open_file(&mut app, Path::new(&path));
    } else {
        open_document(&mut app, "Markdown — пример", DEMO);
    }
    app.run(&mut ViewerApp);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn demo_renders_in_window() {
        let backend = platform::HeadlessBackend::new(80, 25);
        let mut app = Program::new(Box::new(backend), None, None);
        open_document(&mut app, "Документ", DEMO);
        app.pump(&mut ViewerApp);
        let screen = app.screen_text();
        assert!(screen.contains("MarkdownViewer"));
        assert!(screen.contains("Кириллица"));
        assert!(!screen.contains("**Markdown**"));
    }
}
