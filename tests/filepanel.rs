//! Тесты файловой панели и двух панелей на «невидимом» терминале.

use std::path::{Path, PathBuf};

use tvision::platform::{HeadlessBackend, RawEvent};
use tvision::*;

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("tv-filepanel-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("sub/deep")).unwrap();
    std::fs::create_dir_all(d.join("other")).unwrap();
    for (f, n) in [("a.txt", 10u64), ("b.rs", 300), ("c.png", 2000), (".hidden", 1), ("zz.o", 5)] {
        std::fs::File::create(d.join(f)).unwrap().set_len(n).unwrap();
    }
    std::fs::write(d.join("sub/inner.txt"), "x").unwrap();
    d
}

fn key(k: Key) -> RawEvent {
    RawEvent::Key(KeyEvent::new(k, Mods::NONE))
}

fn ckey(k: Key, m: Mods) -> RawEvent {
    RawEvent::Key(KeyEvent::new(k, m))
}

fn click(x: i32, y: i32, buttons: u8) -> Vec<RawEvent> {
    let m = |b| RawEvent::Mouse { pos: Point::new(x, y), buttons: b, mods: Mods::NONE, wheel: 0 };
    vec![m(buttons), m(0)]
}

/// Программа без меню и строки состояния: две панели во весь экран 80×25.
fn run(left: &Path, right: &Path, input: Vec<RawEvent>, h: &mut dyn AppHandler) -> (Program, ViewId) {
    let mut b = HeadlessBackend::new(80, 25);
    b.input.extend(input);
    let mut app = Program::new(Box::new(b), None, None);
    let r = app.desktop().group.vs.extent();
    let id = app.insert_window(DualPanel::new(r, left, right));
    app.pump(h);
    app.redraw();
    (app, id)
}

#[derive(Default)]
struct Exec {
    executed: Vec<PathBuf>,
    panels: ViewId,
}

impl AppHandler for Exec {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(cm::FILE_PANEL_EXECUTE) {
            let p = app.find::<DualPanel>(self.panels).or_else(|| app.find::<DualPanel>(2)).map(|d| d.active().current().unwrap().path.clone());
            if let Some(p) = p {
                self.executed.push(p);
            }
            ev.clear();
        }
    }
}

#[test]
fn listing_sorted_dirs_first() {
    let d = tmp("list");
    let p = FilePanel::new(Rect::new(0, 0, 40, 20), &d);
    let names: Vec<String> = p.entries().iter().map(|e| e.name.clone()).collect();
    assert_eq!(names, ["..", "other", "sub", ".hidden", "a.txt", "b.rs", "c.png", "zz.o"]);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn sort_filter_hidden() {
    let d = tmp("sort");
    let mut p = FilePanel::new(Rect::new(0, 0, 40, 20), &d);
    p.set_sort(SortMode::Size, true);
    let names: Vec<String> = p.entries().iter().map(|e| e.name.clone()).collect();
    assert_eq!(&names[3..], ["c.png", "b.rs", "a.txt", "zz.o", ".hidden"]);
    p.set_filter("*.o,*.png");
    p.toggle_hidden();
    let names: Vec<String> = p.entries().iter().map(|e| e.name.clone()).collect();
    assert_eq!(names, ["..", "sub", "other", "b.rs", "a.txt"]); // обратный порядок — и у каталогов
    p.set_filter("*,!*.rs");
    assert_eq!(p.entries().len(), 4); // .., other, sub, b.rs
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn marks_and_targets() {
    let d = tmp("marks");
    let mut p = FilePanel::new(Rect::new(0, 0, 40, 20), &d);
    assert_eq!(p.mark_glob("*.txt,*.rs", true), 2);
    let t: Vec<String> = p.targets().iter().map(|x| x.file_name().unwrap().to_string_lossy().into_owned()).collect();
    assert_eq!(t, ["a.txt", "b.rs"]);
    p.invert_marks();
    assert_eq!(p.marked().len(), 3); // .hidden, c.png, zz.o
    p.clear_marks();
    p.select_name("c.png");
    assert_eq!(p.targets(), vec![d.join("c.png")]);
    p.set_cursor(0);
    assert!(p.targets().is_empty()); // «..» не цель
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn keyboard_navigation() {
    let d = tmp("keys");
    // ↓ ↓ → (в sub), End, ← (обратно, курсор на sub), Ins, Ctrl+S «b» Esc.
    let input = vec![
        key(Key::Down),
        key(Key::Down),
        key(Key::Right),
        key(Key::End),
        key(Key::Left),
        key(Key::Ins),
        ckey(Key::Char('s'), Mods::CTRL),
        key(Key::Char('b')),
        key(Key::Esc),
    ];
    let mut h = Exec::default();
    let (app, id) = run(&d, &d.join("other"), input, &mut h);
    let dp = app.find::<DualPanel>(id).unwrap();
    let p = dp.active();
    assert_eq!(p.path(), d.as_path());
    assert_eq!(p.current().unwrap().name, "b.rs");
    assert_eq!(p.marked().len(), 1);
    assert_eq!(p.marked()[0].name, "sub");
    assert_eq!(p.history(), &[d.clone(), d.join("sub")]); // посещённые каталоги
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn enter_on_file_posts_execute() {
    let d = tmp("exec");
    let input = vec![ckey(Key::Char('a'), Mods::ALT), key(Key::Enter)];
    let mut h = Exec { panels: 0, ..Default::default() };
    let mut b = HeadlessBackend::new(80, 25);
    b.input.extend(input);
    let mut app = Program::new(Box::new(b), None, None);
    let r = app.desktop().group.vs.extent();
    h.panels = app.insert_window(DualPanel::new(r, &d, &d));
    app.pump(&mut h);
    assert_eq!(h.executed, vec![d.join("a.txt")]);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn tab_switches_panels_and_single_mode() {
    let d = tmp("dual");
    let mut h = Exec::default();
    let (mut app, id) = run(&d, &d.join("other"), vec![key(Key::Tab)], &mut h);
    {
        let dp = app.find_mut::<DualPanel>(id).unwrap();
        assert_eq!(dp.active().path(), d.join("other"));
        dp.sync_other();
        assert_eq!(dp.other().path(), d.join("other"));
        dp.toggle_single();
    }
    app.redraw();
    let s = app.screen_text();
    // Одна панель во весь экран: рамка от края до края.
    assert!(s.lines().next().unwrap().starts_with('╔') && s.lines().next().unwrap().ends_with('╗'), "{}", s);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn tabs_in_frame_and_mouse() {
    let d = tmp("tabs");
    let mut b = HeadlessBackend::new(80, 25);
    // Щелчок по строке «sub» (строка 4: рамка, заголовки, «..», other, sub),
    // двойной щелчок — войти; правая кнопка по «inner.txt» — выделить.
    b.input.extend(click(5, 4, 1));
    b.input.extend(click(5, 4, 1));
    b.input.extend(click(5, 3, 2));
    let mut app = Program::new(Box::new(b), None, None);
    let r = app.desktop().group.vs.extent();
    let mut dp = DualPanel::new(r, &d, &d);
    dp.active_mut().new_tab();
    let id = app.insert_window(dp);
    app.pump(&mut Exec::default());
    app.redraw();
    let dp = app.find::<DualPanel>(id).unwrap();
    let p = dp.active();
    assert_eq!(p.tab_count(), 2);
    assert_eq!(p.path(), d.join("sub"), "{}", app.screen_text());
    assert_eq!(p.marked().iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), ["deep"]);
    let first = app.screen_text().lines().next().unwrap().to_string();
    assert!(first.contains("1:") && first.contains("2:sub"), "{}", first);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn virtual_listing() {
    let d = tmp("listing");
    let mut p = FilePanel::new(Rect::new(0, 0, 60, 20), &d);
    p.set_listing("*.txt", &d, vec![d.join("a.txt"), d.join("sub/inner.txt")]);
    let names: Vec<String> = p.entries().iter().map(|e| e.name.clone()).collect();
    let inner = Path::new("sub").join("inner.txt").to_string_lossy().into_owned();
    assert_eq!(names, vec!["..".to_string(), "a.txt".to_string(), inner]);
    assert_eq!(p.listing_title(), Some("*.txt"));
    assert!(p.go_up());
    assert!(p.listing_title().is_none());
    assert_eq!(p.path(), d.as_path());
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn no_palette_errors() {
    let d = tmp("colors");
    for classic in [false, true] {
        let mut b = HeadlessBackend::new(80, 25);
        b.input.push_back(key(Key::Down));
        let mut app = Program::new(Box::new(b), None, None);
        if classic {
            app.palette = color::palettes::APP_COLOR.to_vec();
        }
        let r = app.desktop().group.vs.extent();
        app.insert_window(DualPanel::new(r, &d, &d));
        // И панель в окне и в диалоге.
        let mut w = Window::new(Rect::new(5, 5, 60, 20), "Окно", 1);
        w.add(FilePanel::new(w.interior(), &d));
        app.insert_window(w);
        let mut dl = Dialog::new(Rect::new(20, 2, 78, 22), "Диалог");
        dl.add(FilePanel::new(Rect::new(2, 2, 56, 18), &d));
        app.insert_window(dl);
        app.pump(&mut Exec::default());
        app.redraw();
        for y in 0..25 {
            for x in 0..80 {
                assert_ne!(app.screen().get(x, y).attr, color::ERROR_ATTR, "({}, {})\n{}", x, y, app.screen_text());
            }
        }
    }
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn dir_sizes() {
    let d = tmp("sizes");
    let mut p = FilePanel::new(Rect::new(0, 0, 60, 20), &d);
    let total = p.calc_dir_sizes();
    assert_eq!(total, 1);
    assert_eq!(p.dir_size(&d.join("sub")), Some(1));
    let _ = std::fs::remove_dir_all(&d);
}
