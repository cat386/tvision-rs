//! Каталог книг: своё окно над моделью — без готовых `ModelWindow` и
//! `CrudController`, из «кирпичиков» модуля `tvision::mvc`.
//!
//! Запуск:
//! ```text
//! cargo run --example mvc_books                    # файл во временном каталоге
//! cargo run --example mvc_books -- книги.json
//! cargo run --example mvc_books --features sqlite -- книги.db
//! ```
//!
//! Что показывает (урок 12, рецепты 4–6):
//! * своё окно: таблица `model_table` + строка итогов внизу, которая
//!   пересчитывается, когда растёт `Model::revision()`;
//! * правка без контроллера: `edit_record` + методы модели;
//! * поиск (Ctrl+F, F3 — дальше) и статистика по всем записям.
//!
//! Клавиши: Ins — добавить, Enter — изменить, Del — удалить,
//! Ctrl+F — найти, F3 — найти дальше, F9 — статистика, Alt-X — выход.

use std::path::PathBuf;

use tvision::mvc::*;
use tvision::*;

const CM_ADD: u16 = 130;
const CM_EDIT: u16 = 131;
const CM_DELETE: u16 = 132;
const CM_FIND: u16 = 133;
const CM_FIND_NEXT: u16 = 134;
const CM_STATS: u16 = 135;

const F_TITLE: usize = 0;
const F_AUTHOR: usize = 1;
const F_YEAR: usize = 2;
const F_PRICE: usize = 4;
const F_READ: usize = 5;

pub fn schema() -> Schema {
    Schema::new(
        "books",
        vec![
            Field::text("title", "~Н~азвание").required().max_len(80).width(28),
            Field::text("author", "~А~втор").max_len(60).width(20),
            Field::int("year", "~Г~од").width(6),
            Field::choice("genre", "~Ж~анр", &["Роман", "Фантастика", "Детектив", "Научная"]),
            Field::real("price", "~Ц~ена").width(9),
            Field::bool("read", "~П~рочитана").width(4),
        ],
    )
}

fn sample() -> Vec<Record> {
    let b = |t: &str, a: &str, y: i64, g: &str, p: f64, r: bool| Record::new(vec![t.into(), a.into(), y.into(), g.into(), p.into(), r.into()]);
    vec![
        b("Мастер и Маргарита", "М. Булгаков", 1967, "Роман", 450.0, true),
        b("Пикник на обочине", "А. и Б. Стругацкие", 1972, "Фантастика", 380.0, true),
        b("Дюна", "Ф. Герберт", 1965, "Фантастика", 690.0, false),
        b("Убийство в «Восточном экспрессе»", "А. Кристи", 1934, "Детектив", 320.0, false),
        b("Краткая история времени", "С. Хокинг", 1988, "Научная", 540.5, true),
    ]
}

pub fn open_model(path: &std::path::Path, seed: bool) -> Result<ModelRef, String> {
    let fresh = !path.exists();
    let mut m = Model::open(schema(), open_storage(path)?)?;
    if fresh && seed {
        for r in sample() {
            m.add(r)?;
        }
    }
    Ok(m.into_ref())
}

/// Итоги по записям — для строки внизу окна и окна статистики.
#[derive(Debug, PartialEq)]
pub struct Stats {
    pub count: usize,
    pub read: usize,
    pub total_price: f64,
    pub oldest: Option<i64>,
}

pub fn stats(m: &Model) -> Stats {
    let r = m.records();
    Stats {
        count: r.len(),
        read: r.iter().filter(|x| x.get(F_READ).as_bool()).count(),
        total_price: r.iter().filter_map(|x| x.get(F_PRICE).as_f64()).sum(),
        oldest: r.iter().filter_map(|x| x.get(F_YEAR).as_i64()).min(),
    }
}

/// Номер первой записи после `from`, где название или автор содержат
/// `what` (без учёта регистра); поиск идёт по кругу.
pub fn find(m: &Model, what: &str, from: usize) -> Option<usize> {
    let what = what.to_lowercase();
    let n = m.len();
    (1..=n).map(|k| (from + k) % n).find(|&i| {
        let r = m.get(i).unwrap();
        r.text(F_TITLE).to_lowercase().contains(&what) || r.text(F_AUTHOR).to_lowercase().contains(&what)
    })
}

// ----------------------------------------------------------------------
// Рецепт 4: своё окно над моделью
// ----------------------------------------------------------------------

pub struct BooksWindow {
    win: Window,
    table: ViewId,
    summary: ViewId,
    model: ModelRef,
    seen: u64,
}

impl BooksWindow {
    pub fn new(bounds: Rect, model: ModelRef) -> Self {
        let mut win = Window::new(bounds, "Книги", 1);
        let v = win.standard_scroll_bar(true);
        let h = win.standard_scroll_bar(false);
        // Таблица — всё внутри рамки, кроме последней строки.
        let mut r = win.interior();
        r.b.y -= 1;
        let table = win.add(model_table(r, model.clone(), Some(v), Some(h)));
        // Строка итогов: прилипает к низу окна и растягивается по ширине.
        let i = win.interior();
        let mut s = StaticText::new(Rect::new(i.a.x, i.b.y - 1, i.b.x, i.b.y), "");
        s.vs.grow_mode = gf::GROW_LO_Y | gf::GROW_HI_X | gf::GROW_HI_Y;
        let summary = win.add(s);
        let mut w = BooksWindow { win, table, summary, model, seen: u64::MAX };
        w.sync();
        w
    }

    fn table_mut(&mut self) -> &mut Table {
        let id = self.table;
        self.win.group.find_mut::<Table>(id).unwrap()
    }

    pub fn current(&self) -> Option<usize> {
        let (row, _) = self.win.group.find::<Table>(self.table)?.cursor();
        (row < self.model.borrow().len()).then_some(row)
    }

    pub fn select(&mut self, i: usize) {
        self.sync();
        self.table_mut().set_cursor(i, 0);
    }

    pub fn summary_text(&self) -> String {
        self.win.group.find::<StaticText>(self.summary).map(|s| s.text.clone()).unwrap_or_default()
    }

    /// Модель изменилась — обновить таблицу и итоги.
    pub fn sync(&mut self) {
        let rev = self.model.borrow().revision();
        if rev == self.seen {
            return;
        }
        self.seen = rev;
        let s = stats(&self.model.borrow());
        let text = format!(" Книг: {}   прочитано: {}   на сумму: {:.2} ₽", s.count, s.read, s.total_price);
        let id = self.summary;
        if let Some(t) = self.win.group.find_mut::<StaticText>(id) {
            t.text = text;
        }
        self.table_mut().refresh();
    }
}

impl View for BooksWindow {
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
        self.sync();
        let double = matches!(ev, Event::MouseDown(m) if m.double);
        self.win.handle_window_event(ev, ctx);
        if matches!(ev, Event::KeyDown(k) if k.key == Key::Enter && k.mods == Mods::NONE) || double {
            ctx.post_command(CM_EDIT, Info::None);
            ev.clear();
        }
        self.sync();
    }
}

// ----------------------------------------------------------------------
// Рецепты 5–6: правка без контроллера, поиск, статистика
// ----------------------------------------------------------------------

struct App {
    model: ModelRef,
    window: ViewId,
    last_find: String,
}

impl App {
    fn win<'a>(&self, app: &'a mut Program) -> Option<&'a mut BooksWindow> {
        app.find_mut::<BooksWindow>(self.window)
    }

    fn report(app: &mut Program, r: Result<usize, String>, w: ViewId) {
        match r {
            Ok(i) => {
                if let Some(w) = app.find_mut::<BooksWindow>(w) {
                    w.select(i);
                }
            }
            Err(e) => {
                message_box(app, &e, mf::ERROR | mf::OK_BUTTON);
            }
        }
    }

    fn add(&mut self, app: &mut Program) {
        let (schema, blank) = {
            let m = self.model.borrow();
            (m.schema().clone(), m.schema().new_record())
        };
        if let Some(rec) = edit_record(app, &schema, &blank, "Новая книга") {
            let r = self.model.borrow_mut().add(rec);
            Self::report(app, r, self.window);
        }
    }

    fn edit(&mut self, app: &mut Program) {
        let Some(i) = self.win(app).and_then(|w| w.current()) else { return };
        let (schema, rec) = {
            let m = self.model.borrow();
            (m.schema().clone(), m.get(i).unwrap().clone())
        };
        if let Some(new) = edit_record(app, &schema, &rec, "Книга") {
            let r = self.model.borrow_mut().update(i, new);
            Self::report(app, r, self.window);
        }
    }

    fn delete(&mut self, app: &mut Program) {
        let Some(i) = self.win(app).and_then(|w| w.current()) else { return };
        let title = self.model.borrow().get(i).unwrap().text(F_TITLE);
        if message_box(app, &format!("\x03Удалить «{}»?", title), mf::CONFIRMATION | mf::YES_BUTTON | mf::NO_BUTTON) == cm::YES {
            let r = self.model.borrow_mut().remove(i).map(|_| i.saturating_sub(1));
            Self::report(app, r, self.window);
        }
    }

    fn find(&mut self, app: &mut Program, ask: bool) {
        if ask || self.last_find.is_empty() {
            match input_box(app, "Найти", "~Т~екст:", &self.last_find, 60) {
                Some(s) if !s.trim().is_empty() => self.last_find = s.trim().to_string(),
                _ => return,
            }
        }
        let from = self.win(app).and_then(|w| w.current()).unwrap_or(usize::MAX - 1);
        // При ask ищем и в текущей строке: начинаем «с предыдущей».
        let from = if ask { from.wrapping_add(self.model.borrow().len()).wrapping_sub(1) } else { from };
        let found = find(&self.model.borrow(), &self.last_find, from);
        match found {
            Some(i) => self.win(app).unwrap().select(i),
            None => {
                message_box(app, &format!("\x03«{}» не найдено.", self.last_find), mf::INFORMATION | mf::OK_BUTTON);
            }
        }
    }

    fn show_stats(&mut self, app: &mut Program) {
        let s = stats(&self.model.borrow());
        let pct = if s.count > 0 { s.read * 100 / s.count } else { 0 };
        let oldest = s.oldest.map_or("—".to_string(), |y| y.to_string());
        let text = format!("Книг: {}\nПрочитано: {} ({}%)\nОбщая цена: {:.2} ₽\nСамая старая: {} г.", s.count, s.read, pct, s.total_price, oldest);
        message_box(app, &text, mf::INFORMATION | mf::OK_BUTTON);
    }
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        match ev.command() {
            Some(CM_ADD) => self.add(app),
            Some(CM_EDIT) => self.edit(app),
            Some(CM_DELETE) => self.delete(app),
            Some(CM_FIND) => self.find(app, true),
            Some(CM_FIND_NEXT) => self.find(app, false),
            Some(CM_STATS) => self.show_stats(app),
            _ => return,
        }
        ev.clear();
    }

    fn idle(&mut self, app: &mut Program) {
        let rows = !self.model.borrow().is_empty();
        for c in [CM_EDIT, CM_DELETE, CM_FIND, CM_FIND_NEXT] {
            if rows {
                app.enable_command(c)
            } else {
                app.disable_command(c)
            }
        }
    }
}

fn menu() -> Menu {
    Menu::new()
        .submenu(
            "~Ф~айл",
            Menu::new().item_key("~С~татистика", CM_STATS, kb::f(9), "F9").separator().item_key("~В~ыход", cm::QUIT, kb::ALT_X, "Alt-X"),
        )
        .submenu(
            "~К~нига",
            Menu::new()
                .item_key("~Д~обавить...", CM_ADD, kb::key(Key::Ins), "Ins")
                .item_key("~И~зменить...", CM_EDIT, kb::f(4), "F4")
                .item_key("~У~далить", CM_DELETE, kb::key(Key::Del), "Del")
                .separator()
                .item_key("~Н~айти...", CM_FIND, kb::ctrl('f'), "Ctrl-F")
                .item_key("Найти ~д~альше", CM_FIND_NEXT, kb::F3, "F3"),
        )
}

fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![
        StatusItem::new("~Ins~ Добавить", kb::key(Key::Ins), CM_ADD),
        StatusItem::new("~Del~ Удалить", kb::key(Key::Del), CM_DELETE),
        StatusItem::new("~Ctrl-F~ Найти", kb::ctrl('f'), CM_FIND),
        StatusItem::new("~F9~ Статистика", kb::f(9), CM_STATS),
        StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT),
        StatusItem::new("", kb::F10, cm::MENU),
    ])]
}

fn start(app: &mut Program, model: ModelRef) -> App {
    let r = app.desktop().group.vs.extent();
    let window = app.insert_window(BooksWindow::new(r, model.clone()));
    App { model, window, last_find: String::new() }
}

fn main() -> std::io::Result<()> {
    locale::set_language(locale::Language::Ru);
    let (path, seed) = match std::env::args().nth(1) {
        Some(p) => (PathBuf::from(p), false),
        None => (std::env::temp_dir().join("tvision-mvc-books.json"), true),
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

    fn memory() -> ModelRef {
        Model::open(schema(), Box::new(MemoryStorage::with_records(sample()))).unwrap().into_ref()
    }

    fn run(model: ModelRef, input: Vec<RawEvent>) -> (Program, App) {
        let mut b = HeadlessBackend::new(100, 25);
        b.input.extend(input);
        let mut app = Program::new(Box::new(b), Some(menu()), Some(status()));
        let mut h = start(&mut app, model);
        app.pump(&mut h);
        app.redraw();
        (app, h)
    }

    #[test]
    fn stats_and_find() {
        let m = memory();
        let m = m.borrow();
        assert_eq!(stats(&m), Stats { count: 5, read: 3, total_price: 2380.5, oldest: Some(1934) });
        assert_eq!(find(&m, "стругацк", 0), Some(1));
        assert_eq!(find(&m, "ДЮНА", 2), Some(2), "по кругу находит и саму строку");
        assert_eq!(find(&m, "нет такой", 0), None);
    }

    #[test]
    fn summary_follows_model() {
        let model = memory();
        let (mut app, mut h) = run(model.clone(), vec![]);
        assert!(app.screen_text().contains("Книг: 5   прочитано: 3"), "{}", app.screen_text());
        // Изменение мимо окна — строка итогов обновится сама.
        model.borrow_mut().remove(0).unwrap();
        app.put_event(Event::command_event(999)); // любое событие: окно сверится с моделью
        app.pump(&mut h);
        app.redraw();
        assert!(app.screen_text().contains("Книг: 4   прочитано: 2"), "{}", app.screen_text());
    }

    #[test]
    fn find_selects_row() {
        // Ctrl+F, «хокинг», Enter → курсор на пятой книге.
        let mut input = vec![RawEvent::Key(kb::ctrl('f'))];
        input.extend("хокинг".chars().map(|c| key(Key::Char(c))));
        input.push(key(Key::Enter));
        let (app, h) = run(memory(), input);
        assert_eq!(app.find::<BooksWindow>(h.window).unwrap().current(), Some(4));
    }

    #[test]
    fn edit_price_through_dialog() {
        // Enter на первой книге; Tab до цены (название, автор, год, жанр, цена).
        let mut input = vec![key(Key::Enter)];
        for _ in 0..4 {
            input.push(key(Key::Tab));
        }
        input.push(RawEvent::Key(KeyEvent::new(Key::Char('y'), Mods::CTRL)));
        input.extend("99,9".chars().map(|c| key(Key::Char(c))));
        input.push(key(Key::Enter));
        let model = memory();
        let (_app, _h) = run(model.clone(), input);
        assert_eq!(model.borrow().get(0).unwrap().get(F_PRICE), &Data::Real(99.9));
    }
}
