//! Контакты на каркасе MVC (`tvision::mvc`): данные в файле JSON или в
//! базе SQLite — программа одна и та же, хранилище выбирается по
//! расширению файла. Урок: `docs/guide/11-mvc.md`.
//!
//! Запуск:
//! ```text
//! cargo run --example mvc_contacts                          # образец (JSON во временном каталоге)
//! cargo run --example mvc_contacts -- мои.json              # файл JSON
//! cargo run --example mvc_contacts --features sqlite -- мои.db   # база SQLite
//! ```
//!
//! Клавиши: Ins — добавить, Enter или F4 — изменить, Del — удалить,
//! щелчок по заголовку столбца — сортировка, F3 — открыть ещё один файл
//! (в новом окне), Alt-X — выход. Каждое изменение сразу записывается в
//! файл — команды «Сохранить» нет.
//!
//! Сравните с `guide_contacts` (урок 9): там окно, данные и чтение файла
//! написаны вручную, здесь — только описание полей.

use std::path::{Path, PathBuf};

use tvision::mvc::*;
use tvision::*;

const CM_SORT_NAME: u16 = 110;
const CM_RELOAD: u16 = 111;

// ----------------------------------------------------------------------
// Модель: описание полей — единственное, что нужно для данных
// ----------------------------------------------------------------------

fn is_phone(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit() || "+-() ".contains(c))
}

fn is_email(s: &str) -> bool {
    s.contains('@')
}

/// Схема контакта. Порядок полей — порядок столбцов таблицы и элементов
/// диалога; `name` — ключ в JSON и столбец в SQLite.
pub fn schema() -> Schema {
    Schema::new(
        "contacts",
        vec![
            Field::text("name", "~И~мя").required().max_len(60).width(22),
            Field::text("phone", "~Т~елефон").max_len(30).width(18).check(is_phone, "В телефоне — только цифры и + - ( )."),
            Field::text("email", "~П~очта").max_len(60).width(22).check(is_email, "В адресе почты нет «@»."),
            Field::choice("group", "~Г~руппа", &["Семья", "Работа", "Друзья"]),
            Field::bool("favorite", "Из~б~ранный").width(3),
        ],
    )
}

fn sample() -> Vec<Record> {
    let c = |n: &str, p: &str, e: &str, g: &str, f: bool| Record::new(vec![n.into(), p.into(), e.into(), g.into(), f.into()]);
    vec![
        c("Анна Петрова", "+7 900 111-22-33", "anna@example.ru", "Семья", true),
        c("Борис Смирнов", "+7 900 222-33-44", "boris@example.ru", "Работа", false),
        c("Вера Иванова", "+7 900 333-44-55", "", "Друзья", true),
        c("Глеб Орлов", "+7 900 444-55-66", "gleb@work.example", "Работа", false),
    ]
}

/// Открыть модель над файлом; новый файл образца заполняется примером.
pub fn open_model(path: &Path, seed: bool) -> Result<ModelRef, String> {
    let fresh = !path.exists();
    let mut m = Model::open(schema(), open_storage(path)?)?;
    if fresh && seed {
        for r in sample() {
            m.add(r)?;
        }
    }
    Ok(m.into_ref())
}

fn default_file() -> PathBuf {
    std::env::temp_dir().join("tvision-mvc-contacts.json")
}

// ----------------------------------------------------------------------
// Приложение: контроллеры окон + свои команды
// ----------------------------------------------------------------------

struct App {
    /// По контроллеру на окно; закрытые окна убираются в `idle`.
    windows: Vec<CrudController>,
}

impl App {
    fn open_window(&mut self, app: &mut Program, model: ModelRef) {
        let r = app.desktop().group.vs.extent();
        // Каждое следующее окно — чуть ниже и правее.
        let k = self.windows.len() as i32 % 5;
        let r = Rect::new(r.a.x + k * 2, r.a.y + k, r.b.x, r.b.y);
        let mut c = CrudController::open(app, r, "Контакты", model);
        c.add_title = "Новый контакт".into();
        c.edit_title = "Контакт".into();
        self.windows.push(c);
    }

    fn open_file(&mut self, app: &mut Program, path: &Path) {
        match open_model(path, false) {
            Ok(m) => self.open_window(app, m),
            Err(e) => {
                message_box(app, &e, mf::ERROR | mf::OK_BUTTON);
            }
        }
    }

    /// Контроллер активного окна.
    fn active(&self, app: &Program) -> Option<&CrudController> {
        self.windows.iter().find(|c| c.is_active(app))
    }
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        // Команды записей: каждый контроллер берёт их, только если его окно активно.
        for c in &mut self.windows {
            if c.handle(app, ev) {
                return;
            }
        }
        let Some(cmd) = ev.command() else { return };
        match cmd {
            cm::OPEN => {
                let d = FileDialog::new("*.*", "Открыть (.json, .db)", "~И~мя", fd::OPEN_BUTTON, 100);
                let (cmd, d) = app.exec_dialog(d);
                if cmd != cm::CANCEL {
                    self.open_file(app, &d.file_name());
                }
            }
            CM_SORT_NAME => {
                if let Some(c) = self.active(app) {
                    c.model.borrow_mut().sort_by(0, true); // окно обновится само
                }
            }
            CM_RELOAD => {
                if let Some(c) = self.active(app) {
                    let r = c.model.borrow_mut().reload();
                    if let Err(e) = r {
                        message_box(app, &e, mf::ERROR | mf::OK_BUTTON);
                    }
                }
            }
            _ => return,
        }
        ev.clear();
    }

    fn idle(&mut self, app: &mut Program) {
        self.windows.retain(|c| app.find::<ModelWindow>(c.window).is_some());
        match self.active(app) {
            Some(c) => {
                c.update_commands(app);
                app.enable_command(CM_SORT_NAME);
                app.enable_command(CM_RELOAD);
            }
            None => {
                for c in [CM_RECORD_ADD, CM_RECORD_EDIT, CM_RECORD_DELETE, CM_SORT_NAME, CM_RELOAD] {
                    app.disable_command(c);
                }
            }
        }
    }
}

fn menu() -> Menu {
    Menu::new()
        .submenu(
            "~Ф~айл",
            Menu::new()
                .item_key("~О~ткрыть...", cm::OPEN, kb::F3, "F3")
                .item_key("~П~еречитать", CM_RELOAD, kb::ctrl('r'), "Ctrl-R")
                .separator()
                .item_key("~В~ыход", cm::QUIT, kb::ALT_X, "Alt-X"),
        )
        .submenu(
            "~К~онтакт",
            Menu::new()
                .item_key("~Д~обавить...", CM_RECORD_ADD, kb::key(Key::Ins), "Ins")
                .item_key("~И~зменить...", CM_RECORD_EDIT, kb::f(4), "F4")
                .item_key("~У~далить", CM_RECORD_DELETE, kb::key(Key::Del), "Del")
                .separator()
                .item("~С~ортировать по имени", CM_SORT_NAME),
        )
        .submenu("~О~кна", Menu::new().item_key("~С~ледующее", cm::NEXT, kb::F6, "F6").item_key("~З~акрыть", cm::CLOSE, kb::ALT_F3, "Alt-F3"))
}

fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![
        StatusItem::new("~Ins~ Добавить", kb::key(Key::Ins), CM_RECORD_ADD),
        StatusItem::new("~F4~ Изменить", kb::f(4), CM_RECORD_EDIT),
        StatusItem::new("~Del~ Удалить", kb::key(Key::Del), CM_RECORD_DELETE),
        StatusItem::new("~F3~ Открыть", kb::F3, cm::OPEN),
        StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT),
        StatusItem::new("", kb::F10, cm::MENU),
    ])]
}

fn main() -> std::io::Result<()> {
    locale::set_language(locale::Language::Ru);
    let (path, seed) = match std::env::args().nth(1) {
        Some(p) => (PathBuf::from(p), false),
        None => (default_file(), true),
    };
    // Ошибку (например, .db без признака sqlite) покажем до запуска интерфейса.
    let model = match open_model(&path, seed) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
    let mut app = Program::with_terminal(Some(menu()), Some(status()))?;
    let mut h = App { windows: Vec::new() };
    h.open_window(&mut app, model);
    app.run(&mut h);
    Ok(())
}

// ----------------------------------------------------------------------
// Тесты: cargo test --example mvc_contacts [--features sqlite]
// ----------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tvision::platform::{HeadlessBackend, RawEvent};

    fn key(k: Key) -> RawEvent {
        RawEvent::Key(KeyEvent::new(k, Mods::NONE))
    }

    fn text(s: &str) -> Vec<RawEvent> {
        s.chars().map(|c| key(Key::Char(c))).collect()
    }

    fn temp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("mvc-contacts-{}-{}", std::process::id(), name));
        let _ = std::fs::remove_file(&p);
        p
    }

    fn run(path: &Path, input: Vec<RawEvent>) -> Program {
        let mut b = HeadlessBackend::new(100, 30);
        b.input.extend(input);
        let mut app = Program::new(Box::new(b), Some(menu()), Some(status()));
        let mut h = App { windows: Vec::new() };
        h.open_window(&mut app, open_model(path, true).unwrap());
        app.pump(&mut h);
        app.redraw();
        app
    }

    /// Ins, имя, Tab, телефон (буквы — ошибка при OK), Enter.
    fn add_daria() -> Vec<RawEvent> {
        let mut v = vec![key(Key::Ins)];
        v.extend(text("Дарья"));
        v.push(key(Key::Tab));
        v.extend(text("+7 999"));
        v.push(key(Key::Enter));
        v
    }

    #[test]
    fn json_file_is_written_at_once() {
        let path = temp("a.json");
        let app = run(&path, add_daria());
        let s = app.screen_text();
        assert!(s.contains("Контакты — ") && s.contains("(5)") && s.contains("Дарья"), "{}", s);
        let file = std::fs::read_to_string(&path).unwrap();
        assert!(file.contains("\"name\": \"Дарья\"") && file.contains("\"phone\": \"+7 999\""), "{}", file);
        // Открыть заново — те же 5 записей.
        assert_eq!(open_model(&path, false).unwrap().borrow().len(), 5);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn bad_email_keeps_dialog_open() {
        let path = temp("b.json");
        // Почта без @: OK не принимается (сообщение и диалог закроет Esc).
        let mut input = vec![key(Key::Ins)];
        input.extend(text("Ева"));
        input.extend([key(Key::Tab), key(Key::Tab)]);
        input.extend(text("ева.ру"));
        input.push(key(Key::Enter));
        let _app = run(&path, input);
        assert_eq!(open_model(&path, false).unwrap().borrow().len(), 4);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn sort_and_delete() {
        let path = temp("c.json");
        // По имени по убыванию (дважды по заголовку не щёлкаем — сортируем модель),
        // затем Del на первой строке и «Да».
        let mut b = HeadlessBackend::new(100, 30);
        b.input.extend([key(Key::Del), key(Key::Enter)]);
        let mut app = Program::new(Box::new(b), Some(menu()), Some(status()));
        let mut h = App { windows: Vec::new() };
        let model = open_model(&path, true).unwrap();
        model.borrow_mut().sort_by(0, false);
        h.open_window(&mut app, model.clone());
        app.pump(&mut h);
        let names: Vec<String> = model.borrow().records().iter().map(|r| r.text(0)).collect();
        assert_eq!(names, ["Вера Иванова", "Борис Смирнов", "Анна Петрова"], "удалён Глеб — первый по убыванию");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn commands_disabled_without_window() {
        let path = temp("d.json");
        let mut app = run(&path, vec![RawEvent::Key(kb::ALT_F3)]); // закрыть окно
        let mut h = App { windows: Vec::new() };
        h.idle(&mut app);
        assert!(!app.command_enabled(CM_RECORD_ADD) && !app.command_enabled(CM_SORT_NAME));
        let _ = std::fs::remove_file(&path);
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn sqlite_database() {
        let path = temp("e.db");
        let _app = run(&path, add_daria());
        let db = tvision::sqlite::Connection::open(&path).unwrap();
        let rows = db.query("SELECT name, phone, favorite FROM contacts ORDER BY id", &[]).unwrap();
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[4][0].to_text(), "Дарья");
        assert_eq!(rows[0][2], tvision::sqlite::SqlValue::Int(1));
        drop(db);
        let _ = std::fs::remove_file(&path);
    }

    #[cfg(not(feature = "sqlite"))]
    #[test]
    fn sqlite_needs_feature() {
        assert!(open_model(Path::new("x.db"), false).err().unwrap().contains("--features sqlite"));
    }
}
