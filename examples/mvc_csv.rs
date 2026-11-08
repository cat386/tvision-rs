//! Своё хранилище для модели: записи в файле CSV (таблица, которую
//! открывают Excel и LibreOffice).
//!
//! Запуск:
//! ```text
//! cargo run --example mvc_csv                  # файл во временном каталоге
//! cargo run --example mvc_csv -- склад.csv
//! ```
//!
//! Что показывает (урок 12, рецепт 7): трейт `Storage` — пять методов, и
//! модель, окно и контроллер работают с новым форматом без изменений.
//!
//! Формат файла: первая строка — имена столбцов (`id,name,qty,price`),
//! дальше строка на запись; значения с запятой, кавычкой или переводом
//! строки берутся в кавычки, кавычка внутри удваивается.

use std::path::{Path, PathBuf};

use tvision::mvc::*;
use tvision::*;

// ----------------------------------------------------------------------
// CSV: разбор и запись строк
// ----------------------------------------------------------------------

/// Значение для CSV: в кавычках, если нужно.
pub fn csv_quote(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) || s.starts_with(' ') || s.ends_with(' ') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Разобрать текст CSV в строки из значений (кавычки, `""`, переводы
/// строк внутри кавычек).
pub fn csv_parse(text: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match (quoted, c) {
            (true, '"') if chars.peek() == Some(&'"') => {
                cur.push('"');
                chars.next();
            }
            (true, '"') => quoted = false,
            (true, c) => cur.push(c),
            (false, '"') => quoted = true,
            (false, ',') => row.push(std::mem::take(&mut cur)),
            (false, '\r') => {}
            (false, '\n') => {
                row.push(std::mem::take(&mut cur));
                rows.push(std::mem::take(&mut row));
            }
            (false, c) => cur.push(c),
        }
    }
    if !cur.is_empty() || !row.is_empty() {
        row.push(cur);
        rows.push(row);
    }
    rows
}

// ----------------------------------------------------------------------
// Рецепт 7: своё хранилище
// ----------------------------------------------------------------------

pub struct CsvStorage {
    path: PathBuf,
    records: Vec<Record>,
    next_id: i64,
}

impl CsvStorage {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        CsvStorage { path: path.into(), records: Vec::new(), next_id: 1 }
    }

    /// Переписать файл целиком.
    fn save(&self, schema: &Schema) -> Result<(), String> {
        let mut out = String::from("id");
        for f in &schema.fields {
            out.push(',');
            out.push_str(&csv_quote(&f.name));
        }
        out.push('\n');
        for r in &self.records {
            out.push_str(&r.id.unwrap_or(0).to_string());
            for (f, v) in schema.fields.iter().zip(&r.values) {
                out.push(',');
                // Флажок — «1»/«0», остальное — как в строке ввода.
                let text = match f.kind {
                    FieldKind::Bool => if v.as_bool() { "1" } else { "0" }.to_string(),
                    _ => v.to_text(),
                };
                out.push_str(&csv_quote(&text));
            }
            out.push('\n');
        }
        std::fs::write(&self.path, out).map_err(|e| format!("Не удалось записать {}:\n{}", self.path.display(), e))
    }
}

impl Storage for CsvStorage {
    fn load(&mut self, schema: &Schema) -> Result<Vec<Record>, String> {
        self.records.clear();
        self.next_id = 1;
        let text = match std::fs::read_to_string(&self.path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(format!("{}: {}", self.path.display(), e)),
        };
        let mut rows = csv_parse(&text).into_iter();
        let Some(header) = rows.next() else { return Ok(Vec::new()) };
        // Столбцы ищем по имени — их порядок в файле может быть любым.
        let col = |name: &str| header.iter().position(|h| h.trim() == name);
        let id_col = col("id");
        let cols: Vec<Option<usize>> = schema.fields.iter().map(|f| col(&f.name)).collect();
        for (n, row) in rows.enumerate() {
            let mut values = Vec::new();
            for (f, c) in schema.fields.iter().zip(&cols) {
                let cell = c.and_then(|c| row.get(c)).map(String::as_str).unwrap_or("");
                let v = match f.kind {
                    FieldKind::Bool => Data::Bool(cell.trim() == "1"),
                    _ => f.parse(cell).map_err(|e| format!("{}, строка {}: {}", self.path.display(), n + 2, e))?,
                };
                values.push(v);
            }
            let id = id_col.and_then(|c| row.get(c)).and_then(|s| s.trim().parse::<i64>().ok()).filter(|&id| id > 0);
            let id = id.unwrap_or(self.next_id);
            self.next_id = self.next_id.max(id + 1);
            self.records.push(Record { id: Some(id), values });
        }
        Ok(self.records.clone())
    }

    fn insert(&mut self, schema: &Schema, rec: &mut Record) -> Result<(), String> {
        rec.id = Some(self.next_id);
        self.records.push(rec.clone());
        if let Err(e) = self.save(schema) {
            self.records.pop();
            rec.id = None;
            return Err(e);
        }
        self.next_id += 1;
        Ok(())
    }

    fn update(&mut self, schema: &Schema, rec: &Record) -> Result<(), String> {
        let i = self.records.iter().position(|r| r.id == rec.id).ok_or("Запись не найдена.")?;
        let old = std::mem::replace(&mut self.records[i], rec.clone());
        self.save(schema).map_err(|e| {
            self.records[i] = old;
            e
        })
    }

    fn delete(&mut self, schema: &Schema, id: i64) -> Result<(), String> {
        let Some(i) = self.records.iter().position(|r| r.id == Some(id)) else { return Ok(()) };
        let old = self.records.remove(i);
        self.save(schema).map_err(|e| {
            self.records.insert(i, old);
            e
        })
    }

    fn name(&self) -> String {
        self.path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
    }
}

// ----------------------------------------------------------------------
// Программа — такая же, как с JSON: меняется только хранилище
// ----------------------------------------------------------------------

pub fn schema() -> Schema {
    Schema::new(
        "stock",
        vec![
            Field::text("name", "~Т~овар").required().max_len(60).width(30),
            Field::int("qty", "~К~оличество").required().width(11),
            Field::real("price", "~Ц~ена").width(10),
            Field::bool("order", "~З~аказать").width(9),
        ],
    )
}

pub fn open_model(path: &Path, seed: bool) -> Result<ModelRef, String> {
    let fresh = !path.exists();
    let mut m = Model::open(schema(), Box::new(CsvStorage::new(path)))?;
    if fresh && seed {
        let p = |n: &str, q: i64, c: f64, o: bool| Record::new(vec![n.into(), q.into(), c.into(), o.into()]);
        for r in [p("Бумага А4, 500 л.", 12, 390.0, false), p("Ручка \"Шарик\"", 3, 25.5, true), p("Скрепки", 40, 60.0, false)]
        {
            m.add(r)?;
        }
    }
    Ok(m.into_ref())
}

struct App {
    crud: CrudController,
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        self.crud.handle(app, ev);
    }
    fn idle(&mut self, app: &mut Program) {
        self.crud.update_commands(app);
    }
}

fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![
        StatusItem::new("~Ins~ Добавить", kb::key(Key::Ins), CM_RECORD_ADD),
        StatusItem::new("~F4~ Изменить", kb::f(4), CM_RECORD_EDIT),
        StatusItem::new("~Del~ Удалить", kb::key(Key::Del), CM_RECORD_DELETE),
        StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT),
    ])]
}

fn start(app: &mut Program, model: ModelRef) -> App {
    let r = app.desktop().group.vs.extent();
    let mut crud = CrudController::open(app, r, "Склад", model);
    crud.add_title = "Новый товар".into();
    crud.edit_title = "Товар".into();
    App { crud }
}

fn main() -> std::io::Result<()> {
    locale::set_language(locale::Language::Ru);
    let (path, seed) = match std::env::args().nth(1) {
        Some(p) => (PathBuf::from(p), false),
        None => (std::env::temp_dir().join("tvision-mvc-stock.csv"), true),
    };
    let model = open_model(&path, seed).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1)
    });
    let mut app = Program::with_terminal(None, Some(status()))?;
    let mut h = start(&mut app, model);
    app.run(&mut h);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tvision::platform::{HeadlessBackend, RawEvent};

    fn temp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("mvc-csv-{}-{}", std::process::id(), name));
        let _ = std::fs::remove_file(&p);
        p
    }

    #[test]
    fn csv_roundtrip() {
        let vals = ["просто", "с, запятой", "с \"кавычками\"", "две\nстроки", ""];
        let line: Vec<String> = vals.iter().map(|s| csv_quote(s)).collect();
        let text = line.join(",") + "\n";
        assert_eq!(csv_parse(&text), vec![vals.iter().map(|s| s.to_string()).collect::<Vec<_>>()]);
        assert_eq!(csv_parse("a,b\r\n1,2"), vec![vec!["a", "b"], vec!["1", "2"]]);
    }

    #[test]
    fn file_written_and_read_back() {
        let path = temp("a.csv");
        let m = open_model(&path, true).unwrap();
        m.borrow_mut().update(1, Record::new(vec!["Ручка \"Шарик\"".into(), Data::Int(10), Data::Null, false.into()])).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("id,name,qty,price,order\n1,\"Бумага А4, 500 л.\",12,390,0\n2,\"Ручка \"\"Шарик\"\"\",10,,0\n"), "{}", text);
        // Столбцы в другом порядке и без id — тоже читается.
        std::fs::write(&path, "qty,name\n5,Клей\n").unwrap();
        let m = open_model(&path, false).unwrap();
        let r = m.borrow().get(0).unwrap().clone();
        assert_eq!((r.id, r.text(0), r.get(1).clone()), (Some(1), "Клей".into(), Data::Int(5)));
        // Неверное число — понятная ошибка с номером строки.
        std::fs::write(&path, "name,qty\nКлей,много\n").unwrap();
        assert!(open_model(&path, false).err().unwrap().contains("строка 2"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn crud_through_interface() {
        let path = temp("b.csv");
        let mut b = HeadlessBackend::new(90, 25);
        b.push_key(kb::key(Key::Ins));
        for c in "Клей".chars() {
            b.push_key(KeyEvent::new(Key::Char(c), Mods::NONE));
        }
        b.push_key(kb::key(Key::Tab));
        b.input.push_back(RawEvent::Key(KeyEvent::new(Key::Char('7'), Mods::NONE)));
        b.push_key(kb::ENTER);
        let mut app = Program::new(Box::new(b), None, Some(status()));
        let mut h = start(&mut app, open_model(&path, true).unwrap());
        app.pump(&mut h);
        assert!(std::fs::read_to_string(&path).unwrap().ends_with("4,Клей,7,,0\n"));
        let _ = std::fs::remove_file(&path);
    }
}
