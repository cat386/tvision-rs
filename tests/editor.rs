//! Тесты редактора и файловых диалогов на «невидимом» терминале.

use std::path::PathBuf;

use tvision::platform::{HeadlessBackend, RawEvent};
use tvision::*;

const CM_OPEN_DLG: u16 = 200;

/// Временный каталог для теста (уникальный по имени теста).
fn tmp_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("tvision-rs-test-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn key(k: KeyEvent) -> RawEvent {
    RawEvent::Key(k)
}
fn ch(c: char) -> RawEvent {
    RawEvent::Key(KeyEvent::new(Key::Char(c), Mods::NONE))
}
fn text(s: &str) -> Vec<RawEvent> {
    s.chars().map(ch).collect()
}
fn plain(k: Key) -> RawEvent {
    RawEvent::Key(KeyEvent::new(k, Mods::NONE))
}
fn ctrl(c: char) -> RawEvent {
    RawEvent::Key(kb::ctrl(c))
}

struct EdApp {
    file: PathBuf,
    dir: PathBuf,
    dialog_result: Option<(u16, PathBuf)>,
}

impl AppHandler for EdApp {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        match ev.command() {
            Some(cm::NEW) => {
                app.insert_window(EditWindow::new(Rect::new(0, 0, 60, 15), &self.file, 1));
                ev.clear();
            }
            Some(CM_OPEN_DLG) => {
                let mut d = FileDialog::new("*.txt", "Open", "~N~ame", fd::OPEN_BUTTON, 100);
                d.directory = self.dir.clone();
                d.read_directory();
                let (cmd, d) = app.exec_dialog(d);
                self.dialog_result = Some((cmd, d.file_name()));
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
        StatusItem::new("~F2~ Save", kb::F2, cm::SAVE),
        StatusItem::new("~F3~ Open", kb::F3, CM_OPEN_DLG),
        StatusItem::new("", kb::ALT_F3, cm::CLOSE),
        StatusItem::new("", kb::F5, cm::NEW),
    ])];
    Program::new(Box::new(b), None, Some(status))
}

fn editor_text(app: &Program) -> Option<String> {
    let w = app.desktop().group.children.last()?;
    let ew = w.as_any().downcast_ref::<EditWindow>()?;
    Some(ew.editor().editor.text())
}

#[test]
fn edit_and_save() {
    let dir = tmp_dir("save");
    let file = dir.join("a.txt");
    std::fs::write(&file, "hello").unwrap();
    let mut input = vec![key(kb::F5), plain(Key::End)];
    input.extend(text(" world"));
    input.push(key(kb::F2));
    let mut app = program(input);
    let mut h = EdApp { file: file.clone(), dir, dialog_result: None };
    app.pump(&mut h);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "hello world");
    let screen = app.screen_text();
    let lines: Vec<&str> = screen.lines().collect();
    assert!(lines[0].contains("a.txt"), "заголовок окна: {:?}", lines[0]);
    assert!(lines[1].starts_with("║hello world"), "текст: {:?}", lines[1]);
    // Индикатор «строка:столбец» в нижней рамке.
    assert!(lines[14].contains(" 1:12 "), "индикатор: {:?}", lines[14]);
}

#[test]
fn close_modified_asks() {
    let dir = tmp_dir("close");
    let file = dir.join("a.txt");
    std::fs::write(&file, "keep").unwrap();
    let mut input = vec![key(kb::F5)];
    input.extend(text("XX"));
    input.push(key(kb::ALT_F3)); // закрыть → «Сохранить?»
    input.push(ch('n')); // «No»
    let mut app = program(input);
    let mut h = EdApp { file: file.clone(), dir, dialog_result: None };
    app.pump(&mut h);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "keep");
    assert_eq!(app.desktop().group.children.len(), 1, "окно закрыто");
}

#[test]
fn close_modified_yes_saves() {
    let dir = tmp_dir("closeyes");
    let file = dir.join("a.txt");
    std::fs::write(&file, "x").unwrap();
    let mut input = vec![key(kb::F5), plain(Key::End), ch('y')];
    input.push(key(kb::ALT_F3));
    input.push(ch('y')); // «Yes»
    let mut app = program(input);
    let mut h = EdApp { file: file.clone(), dir, dialog_result: None };
    app.pump(&mut h);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "xy");
    assert_eq!(app.desktop().group.children.len(), 1, "окно закрыто после сохранения");
}

#[test]
fn find_and_search_again() {
    let dir = tmp_dir("find");
    let file = dir.join("a.txt");
    std::fs::write(&file, "alpha beta gamma beta").unwrap();
    let mut input = vec![key(kb::F5), ctrl('q'), ch('f')];
    input.extend(text("beta"));
    input.push(key(kb::ENTER));
    input.push(ctrl('l')); // искать дальше
    let mut app = program(input);
    let mut h = EdApp { file, dir, dialog_result: None };
    app.pump(&mut h);
    let w = app.desktop().group.children.last().unwrap();
    let ed = &w.as_any().downcast_ref::<EditWindow>().unwrap().editor().editor;
    assert_eq!(ed.selection(), Some((Pos::new(0, 17), Pos::new(0, 21))));
    assert_eq!(ed.selected_text(), "beta");
}

#[test]
fn replace_all_without_prompt() {
    let dir = tmp_dir("replace");
    let file = dir.join("a.txt");
    std::fs::write(&file, "alpha beta gamma beta").unwrap();
    let mut input = vec![key(kb::F5), ctrl('q'), ch('a')];
    input.extend(text("beta"));
    input.push(key(kb::TAB));
    input.extend(text("B"));
    input.push(key(kb::TAB)); // флажки
    let down = plain(Key::Down);
    input.extend([down.clone(), down.clone(), ch(' '), down, ch(' ')]); // снять «Prompt», поставить «Replace all»
    input.push(key(kb::ENTER));
    let mut app = program(input);
    let mut h = EdApp { file, dir, dialog_result: None };
    app.pump(&mut h);
    assert_eq!(editor_text(&app).unwrap(), "alpha B gamma B");
}

#[test]
fn replace_with_prompt() {
    let dir = tmp_dir("prompt");
    let file = dir.join("a.txt");
    std::fs::write(&file, "x x x").unwrap();
    let mut input = vec![key(kb::F5), ctrl('q'), ch('a')];
    input.extend(text("x"));
    input.push(key(kb::TAB));
    input.extend(text("y"));
    input.push(key(kb::TAB));
    let down = plain(Key::Down);
    input.extend([down.clone(), down.clone(), down, ch(' ')]); // «Replace all» (с вопросом)
    input.push(key(kb::ENTER));
    input.extend([ch('y'), ch('n'), ch('y')]); // заменить, пропустить, заменить
    let mut app = program(input);
    let mut h = EdApp { file, dir, dialog_result: None };
    app.pump(&mut h);
    assert_eq!(editor_text(&app).unwrap(), "y x y");
}

#[test]
fn undo_and_clipboard() {
    let dir = tmp_dir("undo");
    let file = dir.join("a.txt");
    std::fs::write(&file, "one").unwrap();
    let shift_end = key(KeyEvent::new(Key::End, Mods::SHIFT));
    let mut input = vec![key(kb::F5), shift_end];
    input.push(key(KeyEvent::new(Key::Ins, Mods::CTRL))); // копировать
    input.push(plain(Key::End));
    input.push(key(KeyEvent::new(Key::Ins, Mods::SHIFT))); // вставить
    input.extend(text("!"));
    input.push(ctrl('u')); // отменить «!»
    let mut app = program(input);
    let mut h = EdApp { file, dir, dialog_result: None };
    app.pump(&mut h);
    assert_eq!(editor_text(&app).unwrap(), "oneone");
}

#[test]
fn file_dialog_open() {
    let dir = tmp_dir("filedlg");
    std::fs::write(dir.join("a.txt"), "1").unwrap();
    std::fs::write(dir.join("b.txt"), "2").unwrap();
    std::fs::write(dir.join("c.md"), "3").unwrap();
    std::fs::create_dir(dir.join("sub")).unwrap();
    std::fs::write(dir.join("sub").join("d.txt"), "4").unwrap();

    // Сначала проверяем содержимое списка.
    let mut app = program(vec![key(kb::F3)]);
    let mut h = EdApp { file: PathBuf::new(), dir: dir.clone(), dialog_result: None };
    app.pump(&mut h);
    // Esc закрыл диалог (защита тестового терминала).
    assert_eq!(h.dialog_result.as_ref().unwrap().0, cm::CANCEL);

    // Переход в подкаталог и выбор файла.
    let mut input = vec![key(kb::F3)];
    input.extend(text("sub"));
    input.push(key(kb::ENTER)); // каталог → остаёмся в диалоге
    input.push(key(kb::alt('n'))); // фокус в строку имени
    input.extend(text("d.txt"));
    input.push(key(kb::ENTER));
    let mut app = program(input);
    let mut h = EdApp { file: PathBuf::new(), dir: dir.clone(), dialog_result: None };
    app.pump(&mut h);
    let (cmd, name) = h.dialog_result.unwrap();
    assert_eq!(cmd, cm::FILE_OPEN);
    assert_eq!(name, dir.join("sub").join("d.txt"));
}

#[test]
fn file_dialog_lists_files() {
    let dir = tmp_dir("filelist");
    std::fs::write(dir.join("a.txt"), "1").unwrap();
    std::fs::write(dir.join("c.md"), "3").unwrap();
    std::fs::create_dir(dir.join("zdir")).unwrap();
    let mut d = FileDialog::new("*.txt", "Open", "~N~ame", fd::OPEN_BUTTON, 100);
    d.directory = dir.clone();
    d.read_directory();
    let list = d.dlg.win.group.children.iter().find_map(|c| c.as_any().downcast_ref::<FileList>()).unwrap();
    let names: Vec<String> = list.entries.iter().map(|e| e.display()).collect();
    let sep = std::path::MAIN_SEPARATOR;
    assert_eq!(names, vec!["a.txt".to_string(), format!("zdir{}", sep), format!("..{}", sep)]);
}

#[test]
fn terminal_autoscroll() {
    let mut t = Terminal::new(Rect::new(0, 0, 20, 3), None, None, 100);
    for i in 0..10 {
        t.write_line(&format!("line {}", i));
    }
    // Последняя строка пустая (после перевода строки), видны строки 8, 9 и пустая.
    assert_eq!(t.scroller.delta.y, 8);
    assert_eq!(t.lines().count(), 11);
}
