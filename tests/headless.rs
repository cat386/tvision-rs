//! Интеграционные тесты на «невидимом» терминале (`HeadlessBackend`):
//! программа получает заранее подготовленные нажатия клавиш и щелчки,
//! а результат проверяется по тексту на экране и по данным представлений.

use std::cell::RefCell;
use std::rc::Rc;

use tvision::platform::{HeadlessBackend, RawEvent};
use tvision::*;

const CM_DIALOG: u16 = 100;

fn menu() -> Menu {
    Menu::new()
        .submenu(
            "~F~ile",
            Menu::new()
                .item_key("~N~ew", cm::NEW, kb::F2, "F2")
                .item("~D~ialog...", CM_DIALOG)
                .separator()
                .item_key("E~x~it", cm::QUIT, kb::ALT_X, "Alt-X"),
        )
        .submenu("~W~indow", Menu::new().item("~T~ile", cm::TILE).item("C~a~scade", cm::CASCADE))
}

fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![
        StatusItem::new("~Alt-X~ Exit", kb::ALT_X, cm::QUIT),
        StatusItem::new("~F2~ New", kb::F2, cm::NEW),
        StatusItem::new("", kb::F10, cm::MENU),
        StatusItem::new("", kb::ALT_F3, cm::CLOSE),
        StatusItem::new("", kb::F6, cm::NEXT),
    ])]
}

/// Создаёт программу 80×25 на тестовом терминале; события добавляются через `input`.
fn app_with(input: Vec<RawEvent>) -> Program {
    let mut b = HeadlessBackend::new(80, 25);
    b.input.extend(input);
    Program::new(Box::new(b), Some(menu()), Some(status()))
}

fn key(k: KeyEvent) -> RawEvent {
    RawEvent::Key(k)
}

fn ch(c: char) -> RawEvent {
    RawEvent::Key(KeyEvent::new(Key::Char(c), Mods::NONE))
}

fn mouse(x: i32, y: i32, buttons: u8) -> RawEvent {
    RawEvent::Mouse { pos: Point::new(x, y), buttons, mods: Mods::NONE, wheel: 0 }
}

/// Обработчик, запоминающий команды и умеющий создавать окна/диалоги.
#[derive(Default)]
struct TestApp {
    commands: Vec<u16>,
    windows: u16,
    dialog_result: Rc<RefCell<Option<(u16, Value)>>>,
}

impl AppHandler for TestApp {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if let Some(c) = ev.command() {
            self.commands.push(c);
            match c {
                cm::NEW => {
                    self.windows += 1;
                    let mut w = Window::new(Rect::new(2, 2, 40, 14), &format!("Win {}", self.windows), self.windows as i16);
                    w.group.vs.options |= of::TILEABLE;
                    app.insert_window(w);
                    ev.clear();
                }
                CM_DIALOG => {
                    let mut d = Dialog::centered(50, 14, "Test dialog");
                    let il = d.add(InputLine::new(Rect::new(12, 2, 46, 3), 40));
                    d.add(Label::new(Rect::new(2, 2, 11, 3), "~N~ame:", Some(il)));
                    d.add(Cluster::check_boxes(Rect::new(3, 5, 22, 7), &["~O~ne", "~T~wo"]));
                    d.add(Cluster::radio_buttons(Rect::new(25, 5, 45, 8), &["~R~ed", "~G~reen", "~B~lue"]));
                    d.add(Button::new(Rect::new(14, 10, 24, 12), "O~K~", cm::OK, bf::DEFAULT));
                    d.add(Button::new(Rect::new(26, 10, 36, 12), "Cancel", cm::CANCEL, bf::NORMAL));
                    let (cmd, d) = app.exec_dialog(d);
                    *self.dialog_result.borrow_mut() = Some((cmd, d.get_data()));
                    ev.clear();
                }
                _ => {}
            }
        }
    }
}

#[test]
fn initial_screen() {
    let mut app = app_with(vec![]);
    app.redraw();
    let text = app.screen_text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[0].starts_with("  File  Window "), "строка меню: {:?}", lines[0]);
    assert!(lines[1].chars().all(|c| c == '░'), "фон рабочего стола");
    assert!(lines[24].starts_with(" Alt-X Exit  F2 New "), "строка состояния: {:?}", lines[24]);
    // Цвета: меню — чёрный на сером (0x70), горячая буква — красная (0x74).
    assert_eq!(app.screen().get(1, 0).attr, 0x70);
    assert_eq!(app.screen().get(2, 0).attr, 0x74);
    assert_eq!(app.screen().get(2, 0).ch, 'F');
}

#[test]
fn new_window_via_hotkey() {
    let mut app = app_with(vec![key(kb::F2)]);
    let mut h = TestApp::default();
    app.pump(&mut h);
    assert_eq!(h.windows, 1);
    let text = app.screen_text();
    let lines: Vec<&str> = text.lines().collect();
    // Окно в (2,2) на рабочем столе, который начинается со строки 1.
    let top = lines[3];
    assert!(top.contains("╔"), "активная рамка двойная: {:?}", top);
    assert!(top.contains(" Win 1 "), "заголовок: {:?}", top);
    assert!(top.contains("[■]"), "кнопка закрытия: {:?}", top);
    assert!(top.contains("[↑]"), "кнопка распахивания: {:?}", top);
    assert!(top.contains('1'), "номер окна");
    // Тень справа от окна.
    assert_eq!(app.screen().get(40, 5).attr, 0x08);
}

#[test]
fn menu_keyboard() {
    // Alt+F открывает «File», стрелка вниз, Enter → «Dialog...» → Esc закрывает диалог.
    let mut app = app_with(vec![key(kb::alt('f')), key(KeyEvent::new(Key::Down, Mods::NONE)), key(kb::ENTER), key(kb::ESC)]);
    let mut h = TestApp::default();
    app.pump(&mut h);
    assert!(h.commands.contains(&CM_DIALOG), "команды: {:?}", h.commands);
    let r = h.dialog_result.borrow().clone().unwrap();
    assert_eq!(r.0, cm::CANCEL);
}

#[test]
fn menu_drawing() {
    let mut app = app_with(vec![key(kb::alt('f'))]);
    let mut h = TestApp::default();
    app.pump(&mut h);
    let text = app.screen_text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[1].starts_with(" ┌"), "рамка меню: {:?}", lines[1]);
    assert!(lines[2].contains("New") && lines[2].contains("F2"), "пункт New: {:?}", lines[2]);
    assert!(lines[4].contains("├"), "разделитель: {:?}", lines[4]);
    // Выбранный пункт — чёрный на зелёном (0x20).
    assert_eq!(app.screen().get(4, 2).attr, 0x20);
}

#[test]
fn dialog_data() {
    // Открываем диалог из меню, вводим имя, Tab → флажки (пробел), Tab → переключатели (стрелка вниз), Enter.
    let mut input = vec![key(kb::alt('f')), ch('d')];
    input.extend("Ivan".chars().map(ch));
    input.push(key(kb::TAB));
    input.push(ch(' '));
    input.push(key(kb::TAB));
    input.push(key(KeyEvent::new(Key::Down, Mods::NONE)));
    input.push(key(kb::ENTER));
    let mut app = app_with(input);
    let mut h = TestApp::default();
    app.pump(&mut h);
    let (cmd, data) = h.dialog_result.borrow().clone().unwrap();
    assert_eq!(cmd, cm::OK);
    assert_eq!(data, Value::List(vec![Value::Str("Ivan".into()), Value::U32(1), Value::U32(1)]));
}

#[test]
fn dialog_hotkeys_and_label() {
    // Alt+T отмечает «Two», Alt+B выбирает «Blue», Alt+N переводит фокус в строку ввода.
    let mut input = vec![key(kb::alt('f')), ch('d'), key(kb::alt('t')), key(kb::alt('b')), key(kb::alt('n'))];
    input.extend("X".chars().map(ch));
    input.push(key(kb::alt('k'))); // Alt+K — кнопка «OK»
    let mut app = app_with(input);
    let mut h = TestApp::default();
    app.pump(&mut h);
    let (cmd, data) = h.dialog_result.borrow().clone().unwrap();
    assert_eq!(cmd, cm::OK);
    assert_eq!(data, Value::List(vec![Value::Str("X".into()), Value::U32(2), Value::U32(2)]));
}

#[test]
fn mouse_close_window() {
    // Создаём окно, щёлкаем по [■] (окно в (2,2) рабочего стола → экран (4..6, 3)).
    let mut app = app_with(vec![key(kb::F2), mouse(5, 3, 1), mouse(5, 3, 0)]);
    let mut h = TestApp::default();
    app.pump(&mut h);
    assert!(h.commands.contains(&cm::CLOSE) || app.desktop().group.children.len() == 1);
    assert_eq!(app.desktop().group.children.len(), 1, "окно должно закрыться");
}

#[test]
fn mouse_drag_window() {
    // Тянем окно за заголовок на 5 вправо и 2 вниз.
    let mut app = app_with(vec![key(kb::F2), mouse(20, 3, 1), mouse(25, 5, 1), mouse(25, 5, 0)]);
    let mut h = TestApp::default();
    app.pump(&mut h);
    let w = app.desktop().group.children.last().unwrap();
    assert_eq!(w.vs().origin, Point::new(7, 4));
}

#[test]
fn tile_and_next() {
    let mut app = app_with(vec![key(kb::F2), key(kb::F2), key(kb::alt('w')), ch('t')]);
    let mut h = TestApp::default();
    app.pump(&mut h);
    let desk = &app.desktop().group;
    assert_eq!(desk.children.len(), 3);
    // Рабочий стол 80×23: два окна делят его по вертикали.
    let mut bounds: Vec<Rect> = desk.children[1..].iter().map(|c| c.vs().bounds()).collect();
    bounds.sort_by_key(|r| r.a.y);
    assert_eq!(bounds[0], Rect::new(0, 0, 80, 11));
    assert_eq!(bounds[1], Rect::new(0, 11, 80, 23));
}

#[test]
fn listbox_with_scrollbar() {
    struct ListApp {
        result: Option<(u16, i32)>,
    }
    impl AppHandler for ListApp {
        fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
            if ev.command() == Some(cm::NEW) {
                let mut d = Dialog::centered(30, 12, "List");
                let sb = d.add(ScrollBar::new(Rect::new(26, 2, 27, 8)));
                let mut lb = ListBox::new(Rect::new(2, 2, 26, 8), 1, Some(sb));
                lb.set_items((0..20).map(|i| format!("Item {}", i)).collect());
                let lid = d.add(lb);
                d.select(lid);
                let (cmd, d) = app.exec_dialog(d);
                let v = d.win.group.find::<ScrollBar>(sb).unwrap().value;
                self.result = Some((cmd, v));
                let _ = d.win.group.find::<ListBox>(lid).unwrap();
                ev.clear();
            }
        }
    }
    let down = key(KeyEvent::new(Key::Down, Mods::NONE));
    let mut app = app_with(vec![key(kb::F2), down.clone(), down.clone(), down, key(kb::ESC)]);
    let mut h = ListApp { result: None };
    app.pump(&mut h);
    assert_eq!(h.result, Some((cm::CANCEL, 3)), "полоса прокрутки следует за списком");
}

#[test]
fn message_box_buttons() {
    struct MbApp {
        r: Option<u16>,
    }
    impl AppHandler for MbApp {
        fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
            if ev.command() == Some(cm::NEW) {
                self.r = Some(message_box(app, "\x03Сохранить изменения?", mf::CONFIRMATION | mf::YES_NO_CANCEL));
                ev.clear();
            }
        }
    }
    // «N» в фазе пост-обработки нажимает кнопку «No».
    let mut app = app_with(vec![key(kb::F2), ch('n')]);
    let mut h = MbApp { r: None };
    app.pump(&mut h);
    assert_eq!(h.r, Some(cm::NO));
}

#[test]
fn quit_with_alt_x() {
    let mut app = app_with(vec![key(kb::ALT_X)]);
    let mut h = TestApp::default();
    app.pump(&mut h);
    assert!(!app.is_running());
}

#[test]
fn resize_terminal() {
    let mut app = app_with(vec![RawEvent::Resize(Point::new(100, 30))]);
    let mut h = TestApp::default();
    app.pump(&mut h);
    let text = app.screen_text();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 30);
    assert_eq!(app.desktop().group.vs.size, Point::new(100, 28));
    assert!(lines[29].starts_with(" Alt-X Exit"));
}

/// Диалог с одной строкой ввода (с валидатором и историей) и кнопками.
struct FormApp {
    results: Vec<(u16, Value)>,
}

impl AppHandler for FormApp {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(cm::NEW) {
            let mut d = Dialog::centered(40, 10, "Form");
            let mut il = InputLine::new(Rect::new(3, 2, 30, 3), 20).with_validator(RangeValidator::new(1, 10));
            il.history_id = Some(7);
            let il = d.add(il);
            d.add(History::new(Rect::new(30, 2, 33, 3), il, 7));
            d.add(Button::new(Rect::new(3, 6, 13, 8), "O~K~", cm::OK, bf::DEFAULT));
            d.add(Button::new(Rect::new(15, 6, 25, 8), "Cancel", cm::CANCEL, bf::NORMAL));
            let (cmd, d) = app.exec_dialog(d);
            self.results.push((cmd, d.get_data()));
            ev.clear();
        }
    }
}

#[test]
fn validator_rejects_and_reports() {
    let mut input = vec![key(kb::F2)];
    // Буквы фильтр не пропускает; «50» вне диапазона → сообщение об ошибке.
    input.extend("a5b0".chars().map(ch));
    input.push(key(kb::ENTER)); // OK → ошибка валидатора
    input.push(key(kb::ENTER)); // закрыть сообщение
    input.push(key(KeyEvent::new(Key::Backspace, Mods::NONE)));
    input.push(key(kb::ENTER)); // теперь «5» — OK
    let mut app = app_with(input);
    let mut h = FormApp { results: vec![] };
    app.pump(&mut h);
    assert_eq!(h.results, vec![(cm::OK, Value::List(vec![Value::Str("5".into())]))]);
}

#[test]
fn history_window() {
    let down = key(KeyEvent::new(Key::Down, Mods::NONE));
    let mut input = vec![key(kb::F2), ch('7'), key(kb::ENTER)];
    // Второй раз: стрелка вниз открывает историю, Enter выбирает «7», Enter — OK.
    input.extend([key(kb::F2), down, key(kb::ENTER), key(kb::ENTER)]);
    let mut app = app_with(input);
    let mut h = FormApp { results: vec![] };
    app.pump(&mut h);
    assert_eq!(h.results.len(), 2);
    assert_eq!(h.results[1], (cm::OK, Value::List(vec![Value::Str("7".into())])));
}
