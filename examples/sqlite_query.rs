//! Консоль SQL: строка запроса сверху, результат — в таблице.
//! Нужен признак `sqlite`.
//!
//! Запуск:
//! ```text
//! cargo run --example sqlite_query --features sqlite               # учебная база в памяти
//! cargo run --example sqlite_query --features sqlite -- мои.db     # свой файл
//! ```
//!
//! Что показывает (урок 12, рецепты 8–9):
//! * `tvision::sqlite` напрямую, без MVC: `query_rows` с именами
//!   столбцов, `changes()` для `INSERT`/`UPDATE`/`DELETE`;
//! * результат произвольной формы в `Table` через `VecTableModel`;
//! * окно из двух частей (строка ввода + таблица), Enter в строке —
//!   выполнить запрос.
//!
//! Клавиши: Enter (в строке запроса) или F9 — выполнить, Tab — перейти между строкой и таблицей,
//! F2 — список таблиц, Alt-X — выход.

use tvision::sqlite::{Connection, Rows};
use tvision::*;

const CM_RUN: u16 = 140;
const CM_TABLES: u16 = 141;

/// Учебная база: страны и города.
pub fn demo_db() -> Connection {
    let db = Connection::open_in_memory().expect("SQLite в памяти");
    db.execute_batch(
        "CREATE TABLE countries (id INTEGER PRIMARY KEY, name TEXT, capital TEXT);
         CREATE TABLE cities (id INTEGER PRIMARY KEY, name TEXT, country_id INTEGER, population INTEGER);
         INSERT INTO countries (name, capital) VALUES ('Россия', 'Москва'), ('Франция', 'Париж'), ('Япония', 'Токио');
         INSERT INTO cities (name, country_id, population) VALUES
           ('Москва', 1, 13100000), ('Санкт-Петербург', 1, 5600000), ('Казань', 1, 1300000),
           ('Париж', 2, 2100000), ('Лион', 2, 520000), ('Токио', 3, 14000000), ('Осака', 3, 2700000);",
    )
    .expect("учебные таблицы");
    db
}

/// Что показать после запроса.
#[derive(Debug, PartialEq)]
pub enum Outcome {
    /// Запрос вернул строки (возможно, ноль).
    Table(Rows),
    /// Команда без результата: сколько строк изменено.
    Changed(usize),
}

/// Выполнить один запрос.
pub fn run_sql(db: &Connection, sql: &str) -> Result<Outcome, String> {
    let rows = db.query_rows(sql, &[]).map_err(|e| e.to_string())?;
    if rows.columns.is_empty() {
        Ok(Outcome::Changed(db.changes()))
    } else {
        Ok(Outcome::Table(rows))
    }
}

/// Строки результата → модель таблицы (`NULL` показываем как «·»).
fn to_model(rows: &Rows) -> VecTableModel {
    let headers: Vec<&str> = rows.columns.iter().map(String::as_str).collect();
    let cells = rows.rows.iter().map(|r| r.iter().map(|v| if v.is_null() { "·".to_string() } else { v.to_text() }).collect()).collect();
    VecTableModel::new(&headers, cells)
}

// ----------------------------------------------------------------------
// Рецепт 9: окно из строки ввода и таблицы
// ----------------------------------------------------------------------

pub struct QueryWindow {
    win: Window,
    input: ViewId,
    table: ViewId,
}

impl QueryWindow {
    pub fn new(bounds: Rect) -> Self {
        let mut win = Window::new(bounds, "SQL", 1);
        let i = win.interior();
        let mut line = InputLine::new(Rect::new(i.a.x, i.a.y, i.b.x, i.a.y + 1), 1000);
        line.vs.grow_mode = gf::GROW_HI_X;
        line.set_text(
            "SELECT c.name, c.population, k.name AS country FROM cities c JOIN countries k ON k.id = c.country_id ORDER BY population DESC",
        );
        let v = win.standard_scroll_bar(true);
        let h = win.standard_scroll_bar(false);
        let mut t = Table::new(Rect::new(i.a.x, i.a.y + 1, i.b.x, i.b.y), Box::new(VecTableModel::new(&[], vec![])), Some(v), Some(h))
            .selection(SelectionMode::Cell)
            .striped(true);
        t.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
        let table = win.add(t);
        let input = win.add(line);
        win.select(input); // фокус — строке ввода
        QueryWindow { win, input, table }
    }

    pub fn sql(&self) -> String {
        self.win.group.find::<InputLine>(self.input).map(|l| l.text()).unwrap_or_default()
    }

    pub fn set_sql(&mut self, s: &str) {
        let id = self.input;
        if let Some(l) = self.win.group.find_mut::<InputLine>(id) {
            l.set_text(s);
        }
    }

    pub fn table(&self) -> &Table {
        self.win.group.find::<Table>(self.table).unwrap()
    }

    /// Показать результат.
    pub fn show(&mut self, rows: &Rows) {
        let id = self.table;
        let t = self.win.group.find_mut::<Table>(id).unwrap();
        t.set_model(Box::new(to_model(rows)));
        t.auto_width(200); // ширина столбцов — по первым 200 строкам
        t.refresh();
        self.win.title = format!("SQL — строк: {}", rows.rows.len());
    }

    pub fn set_title(&mut self, s: &str) {
        self.win.title = s.to_string();
    }
}

impl View for QueryWindow {
    tvision::impl_group_basics!(win.group);

    fn as_window(&self) -> Option<&Window> {
        Some(&self.win)
    }
    fn draw(&self, dc: &mut DrawCtx) {
        self.win.draw_window(dc);
    }
    fn palette(&self) -> &'static [u8] {
        self.win.palette
    }
    fn set_state(&mut self, flag: u16, enable: bool) {
        self.win.set_window_state(flag, enable);
    }
    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        self.win.handle_window_event(ev, ctx);
        // Enter, который не забрала строка ввода, — «выполнить».
        let in_input = self.win.group.current_view().map(|v| v.vs().id) == Some(self.input);
        if in_input && matches!(ev, Event::KeyDown(k) if k.key == Key::Enter && k.mods == Mods::NONE) {
            ctx.post_command(CM_RUN, Info::None);
            ev.clear();
        }
    }
}

// ----------------------------------------------------------------------
// Приложение
// ----------------------------------------------------------------------

struct App {
    db: Connection,
    window: ViewId,
    /// Выполненные запросы (для тестов и истории).
    history: Vec<String>,
}

impl App {
    fn run_text(&mut self, app: &mut Program, sql: &str) {
        let sql = sql.trim();
        if sql.is_empty() {
            return;
        }
        self.history.push(sql.to_string());
        match run_sql(&self.db, sql) {
            Ok(Outcome::Table(rows)) => {
                if let Some(w) = app.find_mut::<QueryWindow>(self.window) {
                    w.show(&rows);
                }
            }
            Ok(Outcome::Changed(n)) => {
                if let Some(w) = app.find_mut::<QueryWindow>(self.window) {
                    w.set_title(&format!("SQL — изменено строк: {}", n));
                }
            }
            Err(e) => {
                message_box(app, &e, mf::ERROR | mf::OK_BUTTON);
            }
        }
    }
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        match ev.command() {
            Some(CM_RUN) => {
                let sql = app.find::<QueryWindow>(self.window).map(|w| w.sql()).unwrap_or_default();
                self.run_text(app, &sql);
            }
            Some(CM_TABLES) => {
                let sql = "SELECT name, sql FROM sqlite_master WHERE type = 'table' ORDER BY name";
                if let Some(w) = app.find_mut::<QueryWindow>(self.window) {
                    w.set_sql(sql);
                }
                self.run_text(app, sql);
            }
            _ => return,
        }
        ev.clear();
    }
}

fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![
        StatusItem::new("~F9~ Выполнить", kb::f(9), CM_RUN),
        StatusItem::new("~F2~ Таблицы", kb::F2, CM_TABLES),
        StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT),
    ])]
}

fn start(app: &mut Program, db: Connection) -> App {
    let r = app.desktop().group.vs.extent();
    let window = app.insert_window(QueryWindow::new(r));
    App { db, window, history: Vec::new() }
}

fn main() -> std::io::Result<()> {
    locale::set_language(locale::Language::Ru);
    let db = match std::env::args().nth(1) {
        Some(p) => Connection::open(&p).unwrap_or_else(|e| {
            eprintln!("{}: {}", p, e);
            std::process::exit(1)
        }),
        None => demo_db(),
    };
    let mut app = Program::with_terminal(None, Some(status()))?;
    let mut h = start(&mut app, db);
    app.put_event(Event::command_event(CM_RUN)); // сразу показать пример запроса
    app.run(&mut h);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tvision::platform::{HeadlessBackend, RawEvent};
    use tvision::sqlite::SqlValue;

    #[test]
    fn select_and_change() {
        let db = demo_db();
        let Outcome::Table(r) = run_sql(&db, "SELECT name FROM countries ORDER BY name").unwrap() else { panic!() };
        assert_eq!(r.columns, ["name"]);
        assert_eq!(r.rows[0], [SqlValue::Text("Россия".into())]);
        assert_eq!(run_sql(&db, "UPDATE cities SET population = 0 WHERE country_id = 1").unwrap(), Outcome::Changed(3));
        assert!(run_sql(&db, "SELECT * FROM нет").unwrap_err().contains("no such table"));
    }

    fn program(input: Vec<RawEvent>) -> (Program, App) {
        let mut b = HeadlessBackend::new(100, 25);
        b.input.extend(input);
        let mut app = Program::new(Box::new(b), None, Some(status()));
        let mut h = start(&mut app, demo_db());
        app.put_event(Event::command_event(CM_RUN));
        app.pump(&mut h);
        app.redraw();
        (app, h)
    }

    #[test]
    fn first_query_fills_table() {
        let (app, _h) = program(vec![]);
        let s = app.screen_text();
        assert!(s.contains("SQL — строк: 7") && s.contains("Токио") && s.contains("country"), "{}", s);
    }

    #[test]
    fn typed_query_runs_on_enter() {
        // Ctrl+Y — очистить строку, набрать запрос, Enter.
        let mut input = vec![RawEvent::Key(KeyEvent::new(Key::Char('y'), Mods::CTRL))];
        input.extend("DELETE FROM cities WHERE population < 1000000".chars().map(|c| RawEvent::Key(KeyEvent::new(Key::Char(c), Mods::NONE))));
        input.push(RawEvent::Key(kb::ENTER));
        let (app, h) = program(input);
        assert!(app.screen_text().contains("изменено строк: 1"), "{}", app.screen_text());
        assert_eq!(h.history.len(), 2);
    }

    #[test]
    fn tables_list() {
        let (app, _h) = program(vec![RawEvent::Key(kb::F2)]);
        let w = app.find::<QueryWindow>(_h.window).unwrap();
        assert_eq!(w.table().model().row_count(), 2);
        assert_eq!(w.table().model().cell(0, 0), "cities");
    }
}
