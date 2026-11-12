//! Тесты среды.

use super::*;
use tvision::platform::HeadlessBackend;

fn ide_with(keys: &[KeyEvent]) -> (Program, Ide) {
    let mut b = HeadlessBackend::new(120, 40);
    for k in keys {
        b.push_key(*k);
    }
    let mut app = Program::new(Box::new(b), Some(menu()), Some(status()));
    let mut ide = Ide::new(&mut app, FormDoc::new("Form1"), None);
    app.pump(&mut ide);
    (app, ide)
}

fn temp(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("tvcreator-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_file(&p);
    p
}

/// Индекс пункта палитры для виджета.
fn pal(k: WidgetKind) -> usize {
    6 + WidgetKind::ALL.iter().position(|x| *x == k).unwrap()
}

#[test]
fn screen_has_all_panes() {
    let (mut app, _ide) = ide_with(&[]);
    app.redraw();
    let s = app.screen_text();
    for w in ["Виджеты", "Объекты", "Форма", "Свойства", "TVCreator — без имени.tvui", "Строка (HBox)", "Form1 — Dialog", "Пустая форма"]
    {
        assert!(s.contains(w), "нет «{}»:\n{}", w, s);
    }
}

#[test]
fn enter_in_palette_adds() {
    // Фокус — на палитре (первый выбираемый); Enter — добавить «Строку».
    let (_app, ide) = ide_with(&[kb::ENTER]);
    assert_eq!(ide.doc.root.children.len(), 1);
    assert_eq!(ide.doc.root.children[0].layout_kind(), Some(LayoutKind::HBox));
    assert_eq!(ide.sel, Sel::Node(vec![0]));
    assert!(ide.is_dirty());
}

#[test]
fn build_form_save_generate() {
    let (mut app, mut ide) = ide_with(&[]);
    // Форма «подпись — поле» и строка кнопок.
    ide.add(&mut app, 3); // Form
    ide.add(&mut app, pal(WidgetKind::Label)); // внутрь формы
    ide.add(&mut app, pal(WidgetKind::InputLine)); // после подписи — подпись свяжется
    assert_eq!(ide.doc.node(&[0, 0]).unwrap().as_widget().unwrap().link, "input1");
    ide.apply_prop(&mut app, "w.name", "user").unwrap();
    ide.sel = Sel::Node(vec![0, 0]);
    ide.apply_prop(&mut app, "w.text", "~И~мя:").unwrap();
    assert_eq!(ide.doc.node(&[0, 0]).unwrap().as_widget().unwrap().link, "user", "ссылка переименована");
    ide.sel = Sel::Node(vec![]);
    ide.add(&mut app, 0); // HBox в корень
    ide.add(&mut app, 4); // пружина внутрь
    ide.add(&mut app, pal(WidgetKind::Button)); // после пружины
    ide.apply_prop(&mut app, "w.default", "да").unwrap();
    assert!(ide.doc.problems().is_empty(), "{:?}", ide.doc.problems());
    // Свойства формы.
    ide.sel = Sel::Form;
    ide.apply_prop(&mut app, "form.name", "LoginDialog").unwrap();
    assert!(ide.apply_prop(&mut app, "form.name", "плохое имя").is_err());
    // Сохранить и прочитать.
    let path = temp("login.tvui");
    assert!(ide.save_to(&mut app, &path));
    assert!(!ide.is_dirty());
    assert_eq!(FormDoc::load(&path).unwrap(), ide.doc);
    // Код — рядом с формой (окно «Код записан» закроет Esc терминала).
    let out = ide.generate(&mut app).unwrap();
    assert_eq!(out, path.with_file_name("login_dialog.rs"));
    let code = std::fs::read_to_string(&out).unwrap();
    assert!(code.contains("pub struct LoginDialog") && code.contains("Label::new(Rect::default(), \"~И~мя:\", Some(user))"), "{}", code);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn undo_redo_and_delete() {
    let (mut app, mut ide) = ide_with(&[]);
    ide.add(&mut app, pal(WidgetKind::Button));
    ide.add(&mut app, pal(WidgetKind::Button));
    assert_eq!(ide.doc.widgets().len(), 2);
    ide.delete(&mut app);
    assert_eq!(ide.doc.widgets().len(), 1);
    ide.undo(&mut app);
    assert_eq!(ide.doc.widgets().len(), 2);
    ide.redo(&mut app);
    assert_eq!(ide.doc.widgets().len(), 1);
    // Вверх/вниз и обёртка.
    ide.add(&mut app, pal(WidgetKind::InputLine));
    ide.shift(&mut app, -1);
    assert_eq!(ide.sel, Sel::Node(vec![0]));
    assert_eq!(ide.doc.widgets()[0].kind, WidgetKind::InputLine);
    ide.wrap(&mut app, LayoutKind::HBox);
    assert_eq!(ide.doc.node(&[0]).unwrap().layout_kind(), Some(LayoutKind::HBox));
}

#[test]
fn selection_follows_canvas_and_tree() {
    let (mut app, mut ide) = ide_with(&[]);
    ide.add(&mut app, pal(WidgetKind::Button));
    ide.add(&mut app, pal(WidgetKind::InputLine));
    // Щелчок на холсте (как его запоминает Canvas).
    app.find_mut::<Canvas>(ide.ids.2).unwrap().picked = Some(Sel::Node(vec![0]));
    ide.idle(&mut app);
    assert_eq!(ide.sel, Sel::Node(vec![0]));
    let t = app.find::<TreeView>(ide.ids.1).unwrap();
    assert_eq!(t.cursor_path().map(|p| p.to_vec()), Some(vec![0, 0, 0]), "дерево — на том же узле");
    // Курсор в дереве.
    app.find_mut::<TreeView>(ide.ids.1).unwrap().select_path(&[0, 0, 1]);
    ide.idle(&mut app);
    assert_eq!(ide.sel, Sel::Node(vec![1]));
    app.redraw();
    assert!(app.screen_text().contains("input1"), "свойства выбранного:\n{}", app.screen_text());
    // Команды по состоянию.
    ide.sel = Sel::Form;
    ide.idle(&mut app);
    assert!(!app.command_enabled(CM_DELETE) && app.command_enabled(CM_UNDO));
}

#[test]
fn quit_asks_when_dirty() {
    let (mut app, mut ide) = ide_with(&[]);
    ide.add(&mut app, 0);
    // Alt+X: окно не даёт выйти и спрашивает; Esc терминала — «Отмена».
    app.put_event(Event::command_event(cm::QUIT));
    app.pump(&mut ide);
    assert!(app.is_running(), "изменения не потеряны");
    // Без изменений — выход сразу.
    ide.set_dirty(false);
    app.put_event(Event::command_event(cm::QUIT));
    app.pump(&mut ide);
    assert!(!app.is_running());
}

#[test]
fn preview_and_code_window() {
    let (mut app, mut ide) = ide_with(&[]);
    ide.add(&mut app, pal(WidgetKind::Button));
    app.put_event(Event::command_event(CM_PREVIEW)); // модальный — закроет Esc
    app.pump(&mut ide);
    app.put_event(Event::command_event(CM_CODE));
    app.pump(&mut ide);
    app.redraw();
    let s = app.screen_text();
    assert!(s.contains("Код: form1.rs") && s.contains("pub struct Form1"), "{}", s);
}

// ---------- темы ----------

fn canvas_theme(app: &Program, ide: &Ide) -> Option<String> {
    app.find::<Canvas>(ide.ids.2).and_then(|c| c.form_theme.as_ref().map(|(n, _)| n.clone()))
}

#[test]
fn menu_has_theme_items() {
    let (mut app, _ide) = ide_with(&[KeyEvent::new(Key::Char('т'), Mods::ALT)]);
    app.redraw();
    let s = app.screen_text();
    assert!(s.contains("Среда") && s.contains("Форма на холсте"), "{}", s);
}

#[test]
fn ide_theme_from_menu_and_default() {
    let (mut app, mut ide) = ide_with(&[]);
    let classic = ide.themes.iter().position(|e| e.theme.base == tvision::theme::Base::Classic).unwrap();
    app.put_event(Event::command_event(CM_IDE_THEME + classic as u16));
    app.pump(&mut ide);
    assert_eq!(app.palette, ide.themes[classic].theme.palette);
    assert_eq!(ide.settings.theme, "builtin:classic");
    app.put_event(Event::command_event(CM_THEME_DEFAULT));
    app.pump(&mut ide);
    assert_eq!(app.palette, standard_palette(), "стандартные цвета для этого терминала");
    assert!(ide.settings.theme.is_empty());
}

#[test]
fn form_theme_paints_only_canvas() {
    let (mut app, mut ide) = ide_with(&[]);
    let ide_pal = app.palette.clone();
    let classic = ide.themes.iter().position(|e| e.theme.base == tvision::theme::Base::Classic).unwrap();
    app.put_event(Event::command_event(CM_FORM_THEME + classic as u16));
    app.pump(&mut ide);
    assert_eq!(app.palette, ide_pal, "цвета среды не меняются");
    assert_eq!(canvas_theme(&app, &ide).as_deref(), Some("Классическая"));
    app.redraw();
    let s = app.screen_text();
    assert!(s.contains("Тема формы: Классическая"), "{}", s);
    // Рамка диалога формы — в цвете классического серого диалога.
    let t = &ide.themes[classic].theme;
    let frame = [t.get(32), t.get(33)];
    assert!(app.screen().cells.iter().any(|c| c.ch == '┌' && frame.contains(&c.attr)));
    // Предпросмотр в теме формы возвращает цвета среды.
    app.put_event(Event::command_event(CM_PREVIEW));
    app.pump(&mut ide);
    assert_eq!(app.palette, ide_pal);
    app.put_event(Event::command_event(CM_FORM_THEME_SAME));
    app.pump(&mut ide);
    assert_eq!(canvas_theme(&app, &ide), None);
}

#[test]
fn choose_form_theme_dialog() {
    let down = KeyEvent::new(Key::Down, Mods::NONE);
    let chosen = |last: KeyEvent| {
        let mut b = HeadlessBackend::new(120, 40);
        b.push_key(down);
        b.push_key(last);
        let mut app = Program::new(Box::new(b), Some(menu()), Some(status()));
        let mut ide = Ide::new(&mut app, FormDoc::new("Form1"), None);
        app.put_event(Event::command_event(CM_FORM_THEME_CHOOSE));
        app.pump(&mut ide);
        (canvas_theme(&app, &ide), ide.themes[0].theme.name.clone())
    };
    let (got, first) = chosen(kb::ENTER);
    assert_eq!(got, Some(first));
    // Отмена возвращает прежнюю тему формы.
    assert_eq!(chosen(kb::ESC).0, None);
}

#[test]
fn themes_remembered_between_runs() {
    let cfg = std::env::temp_dir().join(format!("tvcreator-theme-cfg-{}", std::process::id())).join("settings.json");
    let _ = std::fs::remove_file(&cfg);
    let run = |cmds: &[u16]| {
        let mut app = Program::new(Box::new(HeadlessBackend::new(120, 40)), Some(menu()), Some(status()));
        let mut ide = Ide::with_themes(&mut app, FormDoc::new("Form1"), None, load_themes(&[]), Some(cfg.clone()));
        for c in cmds {
            app.put_event(Event::command_event(*c));
        }
        app.pump(&mut ide);
        let ft = canvas_theme(&app, &ide);
        (app.palette.clone(), ft)
    };
    let themes = load_themes(&[]);
    let mono = themes.iter().position(|e| e.theme.base == tvision::theme::Base::Monochrome).unwrap();
    let classic = themes.iter().position(|e| e.theme.base == tvision::theme::Base::Classic).unwrap();
    run(&[CM_IDE_THEME + mono as u16, CM_FORM_THEME + classic as u16]);
    let (pal, ft) = run(&[]);
    assert_eq!(pal, themes[mono].theme.palette);
    assert_eq!(ft.as_deref(), Some("Классическая"));
    let _ = std::fs::remove_dir_all(cfg.parent().unwrap());
}

#[test]
fn settings_dialog_saves_file_and_window() {
    let cfg = std::env::temp_dir().join(format!("tvcreator-std-settings-{}", std::process::id())).join("tvcreator.settings.json");
    let _ = std::fs::remove_file(&cfg);
    // Диалог: узор фона (Alt+Ф) → ↓ «▒», Enter — OK.
    let mut b = HeadlessBackend::new(120, 40);
    for k in [KeyEvent::new(Key::Char('ф'), Mods::ALT), KeyEvent::new(Key::Down, Mods::NONE), kb::ENTER] {
        b.push_key(k);
    }
    let mut app = Program::new(Box::new(b), Some(menu()), Some(status()));
    let mut ide = Ide::with_themes(&mut app, FormDoc::new("Form1"), None, load_themes(&[]), Some(cfg.clone()));
    app.put_event(Event::command_event(CM_SETTINGS));
    app.pump(&mut ide);
    assert_eq!(app.desktop().pattern(), '▒');
    let saved = Settings::load(&cfg).unwrap();
    assert_eq!(saved.pattern, '▒');
    assert_eq!(saved.language, LanguageChoice::Ru, "по умолчанию — русский");
    // При выходе запоминается окно среды.
    ide.save_on_exit(&app);
    assert!(Settings::load(&cfg).unwrap().window("tvcreator").is_some());
    // Следующий запуск применяет настройки сам.
    let mut app2 = Program::new(Box::new(HeadlessBackend::new(120, 40)), Some(menu()), Some(status()));
    let _ide2 = Ide::with_themes(&mut app2, FormDoc::new("Form1"), None, load_themes(&[]), Some(cfg.clone()));
    assert_eq!(app2.desktop().pattern(), '▒');
    let _ = std::fs::remove_dir_all(cfg.parent().unwrap());
}

/// В палитре виджетов подсвечен ровно один пункт — текущий (раньше
/// весь список был цвета ошибки: список диалога в обычном окне).
#[test]
fn widget_palette_highlights_one_item() {
    for keys in [vec![], vec![KeyEvent::new(Key::Down, Mods::NONE), KeyEvent::new(Key::Down, Mods::NONE)]] {
        let (mut app, ide) = ide_with(&keys);
        app.redraw();
        let pal = app.find::<ListBox>(ide.ids.0).unwrap();
        let (b, focused) = (pal.vs.bounds(), pal.focused);
        let win = app.desktop().group.children.iter().find_map(|v| v.as_window().filter(|w| w.title.starts_with("TVCreator"))).unwrap();
        // Экранные координаты: стол начинается под строкой меню.
        let o = win.group.vs.origin + app.desktop().group.vs.origin;
        let sc = app.screen();
        let rows: Vec<u32> = (0..b.height()).map(|k| sc.cells[(o.y + b.a.y + k) as usize * sc.width + (o.x + b.a.x + 1) as usize].attr).collect();
        assert!(rows.iter().all(|&a| a != tvision::color::ERROR_ATTR), "{:x?}", rows);
        let hi: Vec<usize> = (0..rows.len()).filter(|&k| rows[k] != rows[if focused == 0 { 1 } else { 0 }]).collect();
        assert_eq!(hi, vec![focused], "подсвечен только текущий пункт: {:x?}", rows);
    }
}

/// Линии формы на холсте соединяются с рамкой и между собой, а в
/// палитре есть «Линия».
#[test]
fn lines_on_canvas() {
    assert!(palette_items().iter().any(|t| t.contains("Линия (Line)")));
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("forms/props.tvui");
    let doc = FormDoc::load(&path).unwrap();
    let mut app = Program::new(Box::new(HeadlessBackend::new(120, 40)), Some(menu()), Some(status()));
    let _ide = Ide::new(&mut app, doc, Some(path));
    app.redraw();
    let s = app.screen_text();
    // Форма на холсте неактивна — рамка одинарная: ├───┬───┤.
    assert!(s.contains("├────") && s.contains("─┬─") && s.contains("─┴─") && s.contains("────┤"), "{}", s);
}
