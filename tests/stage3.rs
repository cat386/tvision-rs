//! Тесты этапа 3: дерево, диалог цветов, справка.

use std::rc::Rc;

use tvision::platform::{HeadlessBackend, RawEvent};
use tvision::*;

const CM_TREE: u16 = 300;
const CM_COLORS: u16 = 301;

const HELP: &str = ".topic NoContext=0
 Start
See {Second} topic.
.topic Second=7
 Second topic
Back to {start:NoContext}.
";

fn key(k: KeyEvent) -> RawEvent {
    RawEvent::Key(k)
}
fn plain(k: Key) -> RawEvent {
    RawEvent::Key(KeyEvent::new(k, Mods::NONE))
}
fn ch(c: char) -> RawEvent {
    RawEvent::Key(KeyEvent::new(Key::Char(c), Mods::NONE))
}

#[derive(Default)]
struct App {
    help: Option<Rc<HelpFile>>,
    help_seen: Vec<String>,
    applied: bool,
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        match ev.command() {
            Some(CM_TREE) => {
                let mut w = Window::new(Rect::new(0, 0, 40, 12), "Tree", 1);
                w.group.vs.help_ctx = 7;
                let tree = vec![Node::with_children(
                    "root",
                    vec![Node::with_children("a", vec![Node::new("a1"), Node::new("a2")], false), Node::new("b")],
                    true,
                )];
                w.add(Outline::new(w.interior(), None, None, tree));
                app.insert_window(w);
                ev.clear();
            }
            Some(CM_COLORS) => {
                let d = ColorDialog::new(&app.palette, standard_color_groups());
                let (cmd, d) = app.exec_dialog(d);
                if cmd == cm::OK {
                    app.palette = d.palette.clone();
                    self.applied = true;
                }
                ev.clear();
            }
            Some(cm::HELP) => {
                let ctx = app.help_ctx();
                let w = HelpWindow::new(self.help.clone().unwrap(), ctx);
                let (_, w) = app.exec_dialog(w);
                self.help_seen = w.viewer().text_lines();
                ev.clear();
            }
            _ => {}
        }
    }
}

fn program(input: Vec<RawEvent>) -> Program {
    let mut b = HeadlessBackend::new(80, 25);
    b.input.extend(input);
    let status = vec![StatusDef::all(vec![
        StatusItem::new("~F1~ Help", kb::F1, cm::HELP),
        StatusItem::new("", kb::F2, CM_TREE),
        StatusItem::new("", kb::F3, CM_COLORS),
    ])];
    Program::new(Box::new(b), None, Some(status))
}

fn outline(app: &Program) -> &Outline {
    let w = app.desktop().group.children.last().unwrap();
    w.as_group().unwrap().children.iter().find_map(|c| c.as_any().downcast_ref::<Outline>()).unwrap()
}

#[test]
fn outline_keyboard() {
    // root, a (свёрнут), b → вниз на «a», «+» раскрывает, «-» на корне сворачивает.
    let input = vec![key(kb::F2), plain(Key::Down), ch('+')];
    let mut app = program(input);
    app.pump(&mut App::default());
    let o = outline(&app);
    assert_eq!(o.row_count(), 5);
    assert_eq!(o.focused_node().unwrap().text, "a");
    let text = app.screen_text();
    let lines: Vec<&str> = text.lines().collect();
    // Строки меню нет: окно начинается со строки 0, дерево — со строки 1.
    assert!(lines[1].contains("║└──root"), "{:?}", lines[1]);
    assert!(lines[2].contains("║   ├──a"), "{:?}", lines[2]);
    assert!(lines[3].contains("║   │  ├──a1"), "{:?}", lines[3]);
    assert!(lines[4].contains("║   │  └──a2"), "{:?}", lines[4]);
    assert!(lines[5].contains("║   └──b"), "{:?}", lines[5]);

    let mut app = program(vec![key(kb::F2), ch('-')]);
    app.pump(&mut App::default());
    assert_eq!(outline(&app).row_count(), 1);
}

#[test]
fn outline_mouse_toggle() {
    // Щелчок по ветке «a» (строка 2 экрана, столбец графики) раскрывает узел.
    let input = vec![
        key(kb::F2),
        RawEvent::Mouse { pos: Point::new(4, 2), buttons: 1, mods: Mods::NONE, wheel: 0 },
        RawEvent::Mouse { pos: Point::new(4, 2), buttons: 0, mods: Mods::NONE, wheel: 0 },
    ];
    let mut app = program(input);
    app.pump(&mut App::default());
    assert_eq!(outline(&app).row_count(), 5);
}

#[test]
fn color_dialog_changes_palette() {
    // Группа «Desktop», элемент «Color»: Tab, Tab → сетка текста, стрелка вправо (синий → зелёный).
    let input = vec![key(kb::F3), key(kb::TAB), key(kb::TAB), plain(Key::Right), key(kb::ENTER)];
    let mut app = program(input);
    app.palette = color::palettes::APP_COLOR.to_vec(); // классическая палитра
    let mut h = App::default();
    app.pump(&mut h);
    assert!(h.applied);
    assert_eq!(app.palette[0], 0x72);
    // Фон рабочего стола перерисован новым цветом.
    assert_eq!(app.screen().get(5, 5).attr, 0x72);
}

#[test]
fn help_by_context_and_links() {
    let hf = Rc::new(HelpFile::parse(HELP).unwrap());
    // Окно с контекстом 7 → тема «Second»; Tab выбирает ссылку, Enter — переход к началу.
    let input = vec![key(kb::F2), key(kb::F1), key(kb::TAB), key(kb::ENTER), key(kb::ESC)];
    let mut app = program(input);
    let mut h = App { help: Some(hf.clone()), ..Default::default() };
    app.pump(&mut h);
    assert_eq!(h.help_seen, vec![" Start", "See Second topic."]);

    // Без перехода: просто тема по контексту.
    let mut app = program(vec![key(kb::F2), key(kb::F1), key(kb::ESC)]);
    let mut h = App { help: Some(hf), ..Default::default() };
    app.pump(&mut h);
    assert_eq!(h.help_seen, vec![" Second topic", "Back to start."]);
}
