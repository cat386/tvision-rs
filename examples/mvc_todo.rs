//! Список задач на каркасе MVC: свои команды поверх модели.
//!
//! Запуск:
//! ```text
//! cargo run --example mvc_todo                     # файл во временном каталоге
//! cargo run --example mvc_todo -- дела.json
//! cargo run --example mvc_todo --features sqlite -- дела.db
//! ```
//!
//! Что показывает (урок 12, рецепты 1–3):
//! * поля разных видов: текст с проверкой даты, выбор, флажок;
//! * своя команда, меняющая запись: F5 — отметить «сделано» / снять;
//! * своя команда над всеми записями: «Удалить сделанные»;
//! * сортировка модели при запуске и из меню, сводка в заголовке окна.
//!
//! Клавиши: Ins — новая задача, Enter — изменить, F5 — сделано,
//! Del — удалить, Alt-X — выход.

use std::path::PathBuf;

use tvision::mvc::*;
use tvision::*;

const CM_TOGGLE_DONE: u16 = 120;
const CM_PURGE_DONE: u16 = 121;
const CM_SORT_DUE: u16 = 122;
const CM_SORT_PRIORITY: u16 = 123;

/// Номера полей схемы — чтобы не писать «магические» 0, 1, 2…
#[allow(dead_code)] // нужно тестам
const F_TITLE: usize = 0;
const F_PRIORITY: usize = 1;
const F_DUE: usize = 2;
const F_DONE: usize = 3;

/// Дата в виде ГГГГ-ММ-ДД (так строки сортируются как даты).
fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && s.chars().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
        && matches!(s[5..7].parse::<u32>(), Ok(1..=12))
        && matches!(s[8..10].parse::<u32>(), Ok(1..=31))
}

pub fn schema() -> Schema {
    Schema::new(
        "tasks",
        vec![
            Field::text("title", "~З~адача").required().max_len(80).width(36),
            Field::choice("priority", "~П~риоритет", &["1 высокий", "2 средний", "3 низкий"]),
            Field::text("due", "~С~рок").max_len(10).width(11).check(is_date, "Срок пишется как ГГГГ-ММ-ДД, например 2026-12-31."),
            Field::bool("done", "Сдела~н~о").width(4),
        ],
    )
}

fn sample() -> Vec<Record> {
    let t = |s: &str, p: &str, d: &str, done: bool| Record::new(vec![s.into(), p.into(), d.into(), done.into()]);
    vec![
        t("Купить хлеб", "2 средний", "", false),
        t("Сдать отчёт", "1 высокий", "2026-11-15", false),
        t("Позвонить маме", "1 высокий", "", true),
        t("Разобрать почту", "3 низкий", "2026-11-30", false),
    ]
}

/// Модель над файлом; новый файл — с образцом. Сортировка: по приоритету.
pub fn open_model(path: &std::path::Path, seed: bool) -> Result<ModelRef, String> {
    let fresh = !path.exists();
    let mut m = Model::open(schema(), open_storage(path)?)?;
    if fresh && seed {
        for r in sample() {
            m.add(r)?;
        }
    }
    m.sort_by(F_PRIORITY, true);
    Ok(m.into_ref())
}

struct App {
    crud: CrudController,
}

impl App {
    /// Рецепт 1: своя команда меняет одну запись — через модель.
    fn toggle_done(&mut self, app: &mut Program) {
        let Some(i) = self.crud.window(app).and_then(|w| w.current()) else { return };
        let res = {
            let mut m = self.crud.model.borrow_mut();
            let mut r = m.get(i).unwrap().clone();
            r.values[F_DONE] = Data::Bool(!r.get(F_DONE).as_bool());
            m.update(i, r)
        };
        match res {
            // Окно обновится само (модель изменилась); курсор — за записью.
            Ok(j) => self.crud.window(app).unwrap().select(j),
            Err(e) => {
                message_box(app, &e, mf::ERROR | mf::OK_BUTTON);
            }
        }
    }

    /// Рецепт 2: команда над всеми записями.
    fn purge_done(&mut self, app: &mut Program) {
        let n = self.crud.model.borrow().records().iter().filter(|r| r.get(F_DONE).as_bool()).count();
        if n == 0 {
            message_box(app, "\x03Сделанных задач нет.", mf::INFORMATION | mf::OK_BUTTON);
            return;
        }
        if message_box(app, &format!("\x03Удалить сделанные задачи ({})?", n), mf::CONFIRMATION | mf::YES_BUTTON | mf::NO_BUTTON) != cm::YES
        {
            return;
        }
        let mut m = self.crud.model.borrow_mut();
        // С конца — чтобы номера ещё не просмотренных записей не сдвигались.
        for i in (0..m.len()).rev() {
            if m.get(i).unwrap().get(F_DONE).as_bool() {
                if let Err(e) = m.remove(i) {
                    drop(m);
                    message_box(app, &e, mf::ERROR | mf::OK_BUTTON);
                    return;
                }
            }
        }
    }
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if self.crud.handle(app, ev) {
            return;
        }
        match ev.command() {
            Some(CM_TOGGLE_DONE) => self.toggle_done(app),
            Some(CM_PURGE_DONE) => self.purge_done(app),
            Some(CM_SORT_DUE) => self.crud.model.borrow_mut().sort_by(F_DUE, true),
            Some(CM_SORT_PRIORITY) => self.crud.model.borrow_mut().sort_by(F_PRIORITY, true),
            _ => return,
        }
        ev.clear();
    }

    fn idle(&mut self, app: &mut Program) {
        self.crud.update_commands(app);
        let has_rows = app.command_enabled(CM_RECORD_EDIT);
        for c in [CM_TOGGLE_DONE, CM_PURGE_DONE] {
            if has_rows {
                app.enable_command(c)
            } else {
                app.disable_command(c)
            }
        }
        // Рецепт 3: сводка по модели — в заголовке окна.
        let (left, total) = {
            let m = self.crud.model.borrow();
            (m.records().iter().filter(|r| !r.get(F_DONE).as_bool()).count(), m.len())
        };
        if let Some(w) = self.crud.window(app) {
            let t = format!("Задачи: осталось {} из {}", left, total);
            if w.caption != t {
                w.caption = t;
                w.refresh();
            }
        }
    }
}

fn menu() -> Menu {
    Menu::new().submenu("~Ф~айл", Menu::new().item_key("~В~ыход", cm::QUIT, kb::ALT_X, "Alt-X")).submenu(
        "~З~адача",
        Menu::new()
            .item_key("~Н~овая...", CM_RECORD_ADD, kb::key(Key::Ins), "Ins")
            .item_key("~И~зменить...", CM_RECORD_EDIT, kb::f(4), "F4")
            .item_key("~С~делано / не сделано", CM_TOGGLE_DONE, kb::f(5), "F5")
            .item_key("~У~далить", CM_RECORD_DELETE, kb::key(Key::Del), "Del")
            .separator()
            .item("Сортировать по ~п~риоритету", CM_SORT_PRIORITY)
            .item("Сортировать по с~р~оку", CM_SORT_DUE)
            .separator()
            .item("Удалить с~д~еланные...", CM_PURGE_DONE),
    )
}

fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![
        StatusItem::new("~Ins~ Новая", kb::key(Key::Ins), CM_RECORD_ADD),
        StatusItem::new("~F5~ Сделано", kb::f(5), CM_TOGGLE_DONE),
        StatusItem::new("~Del~ Удалить", kb::key(Key::Del), CM_RECORD_DELETE),
        StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT),
        StatusItem::new("", kb::F10, cm::MENU),
    ])]
}

fn start(app: &mut Program, model: ModelRef) -> App {
    let r = app.desktop().group.vs.extent();
    let mut crud = CrudController::open(app, r, "Задачи", model);
    crud.add_title = "Новая задача".into();
    crud.edit_title = "Задача".into();
    App { crud }
}

fn main() -> std::io::Result<()> {
    locale::set_language(locale::Language::Ru);
    let (path, seed) = match std::env::args().nth(1) {
        Some(p) => (PathBuf::from(p), false),
        None => (std::env::temp_dir().join("tvision-mvc-todo.json"), true),
    };
    let model = open_model(&path, seed).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1)
    });
    let mut app = Program::with_terminal(Some(menu()), Some(status()))?;
    let mut h = start(&mut app, model);
    app.run(&mut h);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tvision::platform::{HeadlessBackend, RawEvent};

    fn key(k: Key) -> RawEvent {
        RawEvent::Key(KeyEvent::new(k, Mods::NONE))
    }

    fn temp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("mvc-todo-{}-{}", std::process::id(), name));
        let _ = std::fs::remove_file(&p);
        p
    }

    fn run(path: &std::path::Path, input: Vec<RawEvent>) -> (Program, App) {
        let mut b = HeadlessBackend::new(90, 25);
        b.input.extend(input);
        let mut app = Program::new(Box::new(b), Some(menu()), Some(status()));
        let mut h = start(&mut app, open_model(path, true).unwrap());
        app.pump(&mut h);
        h.idle(&mut app);
        app.redraw();
        (app, h)
    }

    #[test]
    fn date_check() {
        assert!(is_date("2026-01-31"));
        assert!(!is_date("2026-13-01") && !is_date("31.01.2026") && !is_date("2026-1-1"));
    }

    #[test]
    fn sorted_by_priority_and_counted() {
        let path = temp("a.json");
        let (app, h) = run(&path, vec![]);
        let m = h.crud.model.borrow();
        assert_eq!(m.get(0).unwrap().text(F_PRIORITY), "1 высокий");
        assert_eq!(m.get(0).unwrap().text(F_DUE), "2026-11-15", "сортировка устойчивая");
        assert_eq!(m.get(3).unwrap().text(F_TITLE), "Разобрать почту");
        assert!(app.screen_text().contains("Задачи: осталось 3 из 4"), "{}", app.screen_text());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn toggle_and_purge() {
        let path = temp("b.json");
        // F5 на первой задаче («Сдать отчёт»).
        let (_app, h) = run(&path, vec![RawEvent::Key(kb::f(5))]);
        assert!(h.crud.model.borrow().get(0).unwrap().get(F_DONE).as_bool());
        drop(h);
        // Заново из файла: «Удалить сделанные» → Enter («Да»).
        let mut b = HeadlessBackend::new(90, 25);
        b.push_key(kb::ENTER);
        let mut app = Program::new(Box::new(b), Some(menu()), Some(status()));
        let mut h = start(&mut app, open_model(&path, false).unwrap());
        app.put_event(Event::command_event(CM_PURGE_DONE));
        app.pump(&mut h);
        let titles: Vec<String> = h.crud.model.borrow().records().iter().map(|r| r.text(F_TITLE)).collect();
        assert_eq!(titles, ["Купить хлеб", "Разобрать почту"]);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn bad_date_rejected() {
        let path = temp("c.json");
        let mut input = vec![key(Key::Ins)];
        input.extend("Тест".chars().map(|c| key(Key::Char(c))));
        input.extend([key(Key::Tab), key(Key::Tab)]);
        input.extend("завтра".chars().map(|c| key(Key::Char(c))));
        input.push(key(Key::Enter));
        let (_app, h) = run(&path, input);
        assert_eq!(h.crud.model.borrow().len(), 4, "задача с неверным сроком не добавлена");
        let _ = std::fs::remove_file(&path);
    }
}
