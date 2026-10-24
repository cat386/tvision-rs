//! Тесты таблицы на «невидимом» терминале.

use tvision::platform::{HeadlessBackend, RawEvent};
use tvision::*;

const CM_DIALOG: u16 = 200;

fn plain(k: Key) -> RawEvent {
    RawEvent::Key(KeyEvent::new(k, Mods::NONE))
}
fn ctrl_key(k: Key) -> RawEvent {
    RawEvent::Key(KeyEvent::new(k, Mods::CTRL))
}
fn ch(c: char) -> RawEvent {
    RawEvent::Key(KeyEvent::new(Key::Char(c), Mods::NONE))
}
fn click(x: i32, y: i32) -> Vec<RawEvent> {
    let m = |buttons| RawEvent::Mouse { pos: Point::new(x, y), buttons, mods: Mods::NONE, wheel: 0 };
    vec![m(1), m(0)]
}

fn people() -> VecTableModel {
    VecTableModel::new(
        &["Имя", "Город", "Возраст"],
        vec![
            vec!["Иван".into(), "Тверь".into(), "30".into()],
            vec!["Анна".into(), "Москва".into(), "25".into()],
            vec!["Пётр".into(), "Казань".into(), "100".into()],
        ],
    )
    .editable(true)
}

/// Окно с таблицей и полосами прокрутки во весь рабочий стол 60×16.
fn window_with(model: Box<dyn TableModel>, setup: impl FnOnce(Table) -> Table) -> (Window, ViewId) {
    let mut w = Window::new(Rect::new(0, 0, 60, 16), "Таблица", 1);
    let v = w.standard_scroll_bar(true);
    let h = w.standard_scroll_bar(false);
    let t = setup(Table::new(w.interior(), model, Some(v), Some(h)));
    let id = w.add(t);
    (w, id)
}

fn run(input: Vec<RawEvent>, win: Window) -> (Program, ViewId) {
    let mut b = HeadlessBackend::new(60, 16);
    b.input.extend(input);
    let mut app = Program::new(Box::new(b), None, None);
    let wid = app.insert_window(win);
    assert!(wid != 0);
    app.pump(&mut program::NoHandler);
    app.redraw();
    (app, wid)
}

fn no_error_colors(app: &Program, w: usize, h: usize) {
    for y in 0..h {
        for x in 0..w {
            assert_ne!(app.screen().get(x, y).attr, color::ERROR_ATTR, "ошибка палитры в ({}, {})\n{}", x, y, app.screen_text());
        }
    }
}

#[test]
fn draws_in_window() {
    let (w, id) = window_with(Box::new(people()), |t| t.row_numbers(true));
    let (app, _) = run(vec![], w);
    let s = app.screen_text();
    assert!(s.contains("Имя"), "{}", s);
    assert!(s.contains("Москва"));
    assert!(s.contains("─┼─") || s.contains("┼"));
    no_error_colors(&app, 60, 16);
    let t = app.find::<Table>(id).unwrap();
    assert_eq!(t.cursor(), (0, 0));
}

#[test]
fn keyboard_navigation() {
    let input = vec![plain(Key::Down), plain(Key::Down), plain(Key::Right), plain(Key::Up)];
    let (w, id) = window_with(Box::new(people()), |t| t);
    let (app, _) = run(input, w);
    assert_eq!(app.find::<Table>(id).unwrap().cursor(), (1, 1));
}

#[test]
fn huge_table_end() {
    let m = FnTableModel::new(1_000_000, 500, |r, c| format!("R{}C{}", r, c));
    let input = vec![ctrl_key(Key::End), plain(Key::End)];
    let (w, id) = window_with(Box::new(m), |t| t.row_numbers(true).frozen(1).with_columns((0..500).map(|_| TableColumn::new(12)).collect()));
    let (app, _) = run(input, w);
    let t = app.find::<Table>(id).unwrap();
    assert_eq!(t.cursor(), (999_999, 499));
    let s = app.screen_text();
    assert!(s.contains("1000000"), "{}", s); // номер последней строки
    assert!(s.contains("R999999C0"), "закреплённый столбец виден: {}", s);
    assert!(s.contains("R999999C499"), "{}", s);
}

#[test]
fn edit_cell() {
    // F2 → стереть «Иван» → «Олег» → Enter; потом набрать «7» в возрасте.
    let mut input = vec![plain(Key::F(2))];
    input.extend((0..4).map(|_| plain(Key::Backspace)));
    input.extend("Олег".chars().map(ch));
    input.push(plain(Key::Enter));
    input.push(plain(Key::End));
    input.push(ch('7'));
    input.push(plain(Key::Enter));
    // Начатая и отменённая правка не меняет данные.
    input.push(plain(Key::Down));
    input.push(ch('9'));
    input.push(plain(Key::Esc));
    let (w, id) = window_with(Box::new(people()), |t| t.editable(true));
    let (app, _) = run(input, w);
    let t = app.find::<Table>(id).unwrap();
    assert_eq!(t.model().cell(0, 0), "Олег");
    assert_eq!(t.model().cell(0, 2), "7");
    assert_eq!(t.model().cell(1, 2), "25");
    assert!(!t.is_editing());
}

#[test]
fn sort_by_header_click() {
    // Окно в (0,0): рамка 1, заголовок таблицы — строка 1.
    // Столбец «Возраст» начинается с x = 1 + 12 + 1 + 12 + 1 = 27.
    let mut input = click(30, 1);
    input.extend(click(30, 1)); // второй щелчок — по убыванию
    let (w, id) = window_with(Box::new(people()), |t| t);
    let (app, _) = run(input, w);
    let t = app.find::<Table>(id).unwrap();
    assert_eq!(t.sort_state(), Some((2, false)));
    assert_eq!(t.model().cell(0, 2), "100");
    assert!(app.screen_text().contains("Возраст▼"));
}

#[test]
fn resize_and_multiselect() {
    let input = vec![ctrl_key(Key::Right), ctrl_key(Key::Right), ch(' '), RawEvent::Key(KeyEvent::new(Key::Down, Mods::SHIFT))];
    let (w, id) = window_with(Box::new(people()), |t| t.multi_select(true).selection(SelectionMode::Row));
    let (app, _) = run(input, w);
    let t = app.find::<Table>(id).unwrap();
    assert_eq!(t.columns[0].width, 14);
    assert_eq!(t.selected_rows(), vec![1, 2]);
}

struct DlgApp {
    result: Option<(u16, i64)>,
}

impl AppHandler for DlgApp {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(CM_DIALOG) {
            let mut d = Dialog::new(Rect::new(0, 0, 50, 14), "Выбор");
            let sb = d.add(ScrollBar::new(Rect::new(46, 2, 47, 10)));
            d.add(Table::new(Rect::new(2, 2, 46, 10), Box::new(people()), Some(sb), None).selection(SelectionMode::Row));
            d.add(Button::new(Rect::new(18, 11, 30, 13), "O~K~", cm::OK, bf::DEFAULT));
            let (c, d) = app.exec_dialog(d);
            // Данные диалога: [полоса прокрутки?, таблица, кнопка] — берём число.
            let row = match d.get_data() {
                Value::List(v) => v.iter().find_map(|x| if let Value::Int(i) = x { Some(*i) } else { None }).unwrap_or(-1),
                _ => -1,
            };
            self.result = Some((c, row));
            ev.clear();
        }
    }
}

#[test]
fn table_in_dialog() {
    let mut b = HeadlessBackend::new(60, 16);
    b.input.push_back(plain(Key::Down));
    b.input.push_back(plain(Key::Down));
    b.input.push_back(plain(Key::Enter)); // Enter в таблице нажимает OK
    let mut app = Program::new(Box::new(b), None, None);
    app.put_event(Event::command_event(CM_DIALOG));
    let mut h = DlgApp { result: None };
    app.pump(&mut h);
    assert_eq!(h.result, Some((cm::OK, 2)));
}

#[test]
fn dialog_colors() {
    // Отрисовка в диалоге без модального цикла: диалог как обычное окно.
    let mut b = HeadlessBackend::new(60, 16);
    b.input.push_back(plain(Key::Down));
    let mut app = Program::new(Box::new(b), None, None);
    let mut d = Dialog::new(Rect::new(0, 0, 50, 14), "Выбор");
    d.add(Table::new(Rect::new(2, 2, 46, 10), Box::new(people()), None, None).row_numbers(true).striped(true));
    app.insert_window(d);
    app.pump(&mut program::NoHandler);
    app.redraw();
    no_error_colors(&app, 50, 14);
    assert!(app.screen_text().contains("Казань"));
}
