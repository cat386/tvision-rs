use super::*;
use crate::json::parse_json;
use crate::platform::{HeadlessBackend, RawEvent};
use crate::{kb, Key, KeyEvent, Mods, Program, View};

fn schema() -> Schema {
    Schema::new(
        "people",
        vec![
            Field::text("name", "~И~мя").required().max_len(30),
            Field::int("age", "~В~озраст"),
            Field::real("score", "~Б~аллы"),
            Field::bool("vip", "~V~IP"),
            Field::choice("group", "~Г~руппа", &["Семья", "Работа"]),
        ],
    )
}

fn rec(name: &str, age: Option<i64>) -> Record {
    Record::new(vec![name.into(), age.map_or(Data::Null, Data::Int), Data::Real(1.5), Data::Bool(true), "Работа".into()])
}

fn temp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("tv-mvc-{}-{}", std::process::id(), name))
}

#[test]
fn field_parse_and_display() {
    let s = schema();
    assert!(s.fields[0].parse("  ").is_err());
    assert_eq!(s.fields[1].parse(" 42 ").unwrap(), Data::Int(42));
    assert_eq!(s.fields[2].parse("2,5").unwrap(), Data::Real(2.5));
    assert!(s.fields[4].parse("Друзья").is_err());
    assert_eq!(s.fields[3].display(&Data::Bool(true)), "✓");
    assert_eq!(s.fields[0].plain_title(), "Имя");
    let checked = Field::text("mail", "Почта").check(|s| s.contains('@'), "Нет «@».");
    assert_eq!(checked.parse("x").unwrap_err(), "Нет «@».");
    assert!(checked.parse("").is_ok(), "пустое необязательное поле проверку не проходит");
    assert_eq!(s.new_record().values, vec!["".into(), Data::Null, Data::Null, Data::Bool(false), "Семья".into()]);
}

#[test]
fn json_roundtrip_of_record() {
    let s = schema();
    let mut r = rec("Аня", Some(30));
    r.id = Some(7);
    let j = s.to_json(&r);
    assert_eq!(j.compact(), r#"{"id": 7, "name": "Аня", "age": 30, "score": 1.5, "vip": true, "group": "Работа"}"#);
    assert_eq!(s.from_json(&j), r);
    // Нестрогие значения из чужого файла.
    let j = parse_json(r#"{"name": 5, "age": "12", "vip": 1}"#).unwrap();
    let r = s.from_json(&j);
    assert_eq!(r.values[..4], ["5".into(), Data::Int(12), Data::Null, Data::Bool(true)]);
}

#[test]
fn model_crud_and_sort() {
    let mut m = Model::open(schema(), Box::new(MemoryStorage::new())).unwrap();
    m.add(rec("Вера", Some(20))).unwrap();
    m.add(rec("Анна", None)).unwrap();
    m.sort_by(0, true);
    assert_eq!(m.get(0).unwrap().text(0), "Анна");
    // Новая запись встаёт на место по сортировке.
    let i = m.add(rec("Борис", Some(1))).unwrap();
    assert_eq!(i, 1);
    // Переименование двигает запись.
    let j = m.update(0, rec("Яна", None)).unwrap();
    assert_eq!(j, 2);
    assert_eq!(m.get(2).unwrap().id, Some(2), "id сохраняется");
    // По возрасту по убыванию: пустые — в конце.
    m.sort_by(1, false);
    assert_eq!(m.records().iter().map(|r| r.text(0)).collect::<Vec<_>>(), ["Вера", "Борис", "Яна"]);
    let rev = m.revision();
    assert!(m.add(rec("", None)).is_err());
    assert_eq!(m.revision(), rev, "неверная запись ничего не меняет");
    m.remove(0).unwrap();
    assert_eq!(m.len(), 2);
}

#[test]
fn json_storage_file() {
    let path = temp("a.json");
    let _ = std::fs::remove_file(&path);
    {
        let mut m = Model::open(schema(), Box::new(JsonStorage::new(&path))).unwrap();
        assert!(m.is_empty(), "нет файла — нет записей");
        m.add(rec("Аня", Some(1))).unwrap();
        m.add(rec("Боря", None)).unwrap();
        m.update(0, rec("Анна", Some(2))).unwrap();
        m.remove(1).unwrap();
        m.add(rec("Вера", None)).unwrap();
    }
    let m = Model::open(schema(), Box::new(JsonStorage::new(&path))).unwrap();
    assert_eq!(m.records().iter().map(|r| (r.id, r.text(0))).collect::<Vec<_>>(), [(Some(1), "Анна".into()), (Some(3), "Вера".into())]);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn json_storage_keeps_unknown_keys_and_assigns_ids() {
    let path = temp("b.json");
    std::fs::write(&path, r#"[{"name": "А", "note": "лишнее"}, {"id": 5, "name": "Б"}, {"id": 5, "name": "В"}]"#).unwrap();
    let mut m = Model::open(schema(), Box::new(JsonStorage::new(&path))).unwrap();
    let ids: Vec<_> = m.records().iter().map(|r| r.id.unwrap()).collect();
    assert_eq!(ids, [6, 5, 7], "без номера и с повтором — новые номера");
    m.update(0, rec("А2", None)).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("\"note\": \"лишнее\"") && text.contains("А2"), "{}", text);
    std::fs::write(&path, "{\"не\": \"массив\"}").unwrap();
    assert!(Model::open(schema(), Box::new(JsonStorage::new(&path))).is_err());
    std::fs::write(&path, "[1,").unwrap();
    let e = Model::open(schema(), Box::new(JsonStorage::new(&path))).err().unwrap();
    assert!(e.contains("строка 1"), "{}", e);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn open_storage_by_extension() {
    assert!(open_storage("x.json").is_ok());
    assert!(open_storage("x.txt").is_err());
    #[cfg(not(feature = "sqlite"))]
    assert!(open_storage("x.db").err().unwrap().contains("sqlite"));
}

#[cfg(feature = "sqlite")]
#[test]
fn sqlite_storage_file() {
    let path = temp("c.db");
    let _ = std::fs::remove_file(&path);
    {
        let mut m = Model::open(schema(), open_storage(&path).unwrap()).unwrap();
        m.add(rec("Аня", Some(1))).unwrap();
        m.add(rec("Боря", None)).unwrap();
        m.update(1, rec("Борис", Some(3))).unwrap();
        m.remove(0).unwrap();
    }
    // Схема с новым полем: столбец добавится сам.
    let mut s = schema();
    s.fields.push(Field::text("city", "Город"));
    let m = Model::open(s, Box::new(SqliteStorage::open(&path).unwrap())).unwrap();
    assert_eq!(m.len(), 1);
    let r = m.get(0).unwrap();
    assert_eq!(
        (r.id, r.text(0), r.get(1).clone(), r.get(3).clone(), r.text(5)),
        (Some(2), "Борис".into(), Data::Int(3), Data::Bool(true), "".into())
    );
    drop(m);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn dialog_layout_and_data() {
    let s = schema();
    let r = rec("Аня", Some(5));
    let d = record_dialog(&s, &r, "Запись");
    let size = d.win.group.vs.size;
    let inside = crate::Rect::new(1, 1, size.x - 1, size.y - 1);
    for c in &d.win.group.children {
        let b = c.vs().bounds();
        assert_eq!(inside.intersect(&b), b, "{:?} залезает на рамку", b);
    }
    let back = record_from_value(&s, &d.get_data()).unwrap();
    assert_eq!(back.values, r.values);
}

fn key(k: Key) -> RawEvent {
    RawEvent::Key(KeyEvent::new(k, Mods::NONE))
}

struct App {
    crud: CrudController,
}

impl crate::AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut crate::Event) {
        self.crud.handle(app, ev);
    }
    fn idle(&mut self, app: &mut Program) {
        self.crud.update_commands(app);
    }
}

fn status() -> Vec<crate::StatusDef> {
    use crate::StatusItem;
    vec![crate::StatusDef::all(vec![
        StatusItem::new("~Ins~ Добавить", kb::key(Key::Ins), CM_RECORD_ADD),
        StatusItem::new("~Del~ Удалить", kb::key(Key::Del), CM_RECORD_DELETE),
    ])]
}

fn program(input: Vec<RawEvent>, model: ModelRef) -> (Program, App) {
    let mut b = HeadlessBackend::new(90, 30);
    b.input.extend(input);
    let mut app = Program::new(Box::new(b), None, Some(status()));
    let r = app.desktop().group.vs.extent();
    let crud = CrudController::open(&mut app, r, "Люди", model);
    (app, App { crud })
}

#[test]
fn controller_add_edit_delete() {
    let model = Model::open(schema(), Box::new(MemoryStorage::with_records(vec![rec("Аня", Some(1))]))).unwrap().into_ref();
    // Ins → диалог: имя «Боря», Tab, возраст «4x2» (буква не наберётся), Enter.
    let mut input = vec![key(Key::Ins)];
    input.extend("Боря".chars().map(|c| key(Key::Char(c))));
    input.push(key(Key::Tab));
    input.extend("4x2".chars().map(|c| key(Key::Char(c))));
    input.push(key(Key::Enter));
    // Enter на новой записи → диалог правки; стереть имя и набрать «Борис».
    input.push(key(Key::Enter));
    input.push(RawEvent::Key(KeyEvent::new(Key::Char('y'), Mods::CTRL)));
    input.extend("Борис".chars().map(|c| key(Key::Char(c))));
    input.push(key(Key::Enter));
    let (mut app, mut h) = program(input, model.clone());
    app.pump(&mut h);
    {
        let m = model.borrow();
        assert_eq!(m.len(), 2);
        assert_eq!((m.get(1).unwrap().text(0), m.get(1).unwrap().get(1).clone()), ("Борис".into(), Data::Int(42)));
    }
    app.redraw();
    let screen = app.screen_text();
    assert!(screen.contains("Люди — в памяти (2)") && screen.contains("Борис"), "{}", screen);
}

#[test]
fn controller_delete_asks() {
    let model = Model::open(schema(), Box::new(MemoryStorage::with_records(vec![rec("Аня", Some(1)), rec("Боря", None)]))).unwrap().into_ref();
    // ↓ на вторую запись, Del → вопрос, Enter — «Да».
    let (mut app, mut h) = program(vec![key(Key::Down), key(Key::Del), key(Key::Enter)], model.clone());
    app.pump(&mut h);
    assert_eq!(model.borrow().len(), 1);
    assert_eq!(model.borrow().get(0).unwrap().text(0), "Аня");
}

#[test]
fn controller_commands_follow_state() {
    let model = Model::open(schema(), Box::new(MemoryStorage::new())).unwrap().into_ref();
    let (mut app, mut h) = program(vec![], model);
    crate::AppHandler::idle(&mut h, &mut app);
    assert!(app.command_enabled(CM_RECORD_ADD));
    assert!(!app.command_enabled(CM_RECORD_EDIT) && !app.command_enabled(CM_RECORD_DELETE));
    // Пустое имя: OK не закрывает диалог, сообщение и диалог закроет Esc.
    let (cmd, _) = app.exec_dialog(record_dialog(&schema(), &schema().new_record(), "Новая"));
    assert_eq!(cmd, crate::cm::CANCEL);
}
