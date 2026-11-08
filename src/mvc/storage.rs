//! Хранилища записей: в памяти, в файле JSON, в базе SQLite.

use std::path::{Path, PathBuf};

use super::data::{Record, Schema};
use crate::json::{parse_json, Json};

/// Где лежат записи. Модель вызывает эти методы и ничего не знает о
/// формате: поменять JSON на SQLite — значит передать другое хранилище.
///
/// Каждый метод сразу сохраняет изменение (файл переписывается, строка
/// базы обновляется): отдельной команды «Сохранить» не нужно.
pub trait Storage {
    /// Прочитать все записи.
    fn load(&mut self, schema: &Schema) -> Result<Vec<Record>, String>;
    /// Добавить запись и записать в `rec.id` её новый номер.
    fn insert(&mut self, schema: &Schema, rec: &mut Record) -> Result<(), String>;
    /// Сохранить изменённую запись (по `rec.id`).
    fn update(&mut self, schema: &Schema, rec: &Record) -> Result<(), String>;
    /// Удалить запись по номеру.
    fn delete(&mut self, schema: &Schema, id: i64) -> Result<(), String>;
    /// Короткое описание для заголовка окна: имя файла.
    fn name(&self) -> String;
}

// ----------------------------------------------------------------------
// В памяти
// ----------------------------------------------------------------------

/// Записи только в памяти — для тестов и черновиков.
#[derive(Default)]
pub struct MemoryStorage {
    records: Vec<Record>,
    next_id: i64,
}

impl MemoryStorage {
    pub fn new() -> Self {
        MemoryStorage { records: Vec::new(), next_id: 1 }
    }

    /// Хранилище с готовыми записями (номера раздаются по порядку).
    pub fn with_records(records: Vec<Record>) -> Self {
        let mut s = MemoryStorage::new();
        for mut r in records {
            r.id = Some(s.next_id);
            s.next_id += 1;
            s.records.push(r);
        }
        s
    }
}

impl Storage for MemoryStorage {
    fn load(&mut self, _schema: &Schema) -> Result<Vec<Record>, String> {
        Ok(self.records.clone())
    }
    fn insert(&mut self, _schema: &Schema, rec: &mut Record) -> Result<(), String> {
        self.next_id = self.next_id.max(1);
        rec.id = Some(self.next_id);
        self.next_id += 1;
        self.records.push(rec.clone());
        Ok(())
    }
    fn update(&mut self, _schema: &Schema, rec: &Record) -> Result<(), String> {
        let r = self.records.iter_mut().find(|r| r.id == rec.id).ok_or("Запись не найдена.")?;
        *r = rec.clone();
        Ok(())
    }
    fn delete(&mut self, _schema: &Schema, id: i64) -> Result<(), String> {
        self.records.retain(|r| r.id != Some(id));
        Ok(())
    }
    fn name(&self) -> String {
        "в памяти".into()
    }
}

// ----------------------------------------------------------------------
// JSON
// ----------------------------------------------------------------------

/// Записи в файле JSON: массив объектов.
///
/// ```json
/// [
///   {"id": 1, "name": "Анна", "phone": "+7 900 111-22-33"},
///   {"id": 2, "name": "Борис", "phone": ""}
/// ]
/// ```
///
/// * Файла нет — записей нет; файл появится при первом изменении.
/// * У объекта нет `"id"` — номер даётся при чтении.
/// * Ключи, которых нет в схеме, сохраняются как были.
/// * После каждого изменения файл переписывается целиком (сначала во
///   временный файл, потом переименованием — сбой не испортит данные).
pub struct JsonStorage {
    path: PathBuf,
    /// Объекты файла с их номерами — чтобы не терять лишние ключи.
    items: Vec<(i64, Json)>,
    next_id: i64,
}

impl JsonStorage {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        JsonStorage { path: path.into(), items: Vec::new(), next_id: 1 }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn save(&self) -> Result<(), String> {
        let arr = Json::Arr(self.items.iter().map(|(_, j)| j.clone()).collect());
        let text = arr.pretty() + "\n";
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, text)
            .and_then(|_| std::fs::rename(&tmp, &self.path))
            .map_err(|e| format!("Не удалось записать {}:\n{}", self.path.display(), e))
    }

    fn item_mut(&mut self, id: i64) -> Result<&mut Json, String> {
        self.items.iter_mut().find(|(i, _)| *i == id).map(|(_, j)| j).ok_or_else(|| "Запись не найдена.".to_string())
    }
}

impl Storage for JsonStorage {
    fn load(&mut self, schema: &Schema) -> Result<Vec<Record>, String> {
        self.items.clear();
        self.next_id = 1;
        let text = match std::fs::read_to_string(&self.path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(format!("Не удалось прочитать {}:\n{}", self.path.display(), e)),
        };
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }
        let j = parse_json(&text).map_err(|e| format!("{}: {}", self.path.display(), e))?;
        let Json::Arr(arr) = j else {
            return Err(format!("{}: ожидался массив объектов [ … ].", self.path.display()));
        };
        let mut parsed = Vec::with_capacity(arr.len());
        for (k, item) in arr.into_iter().enumerate() {
            if !matches!(item, Json::Obj(_)) {
                return Err(format!("{}: элемент {} — не объект.", self.path.display(), k + 1));
            }
            let rec = schema.from_json(&item);
            parsed.push((item, rec));
        }
        // Номера: сначала занятые, потом новые — тем, у кого номера нет
        // или он повторяется.
        let mut used = std::collections::HashSet::new();
        for (_, rec) in &mut parsed {
            match rec.id {
                Some(id) if used.insert(id) => self.next_id = self.next_id.max(id + 1),
                _ => rec.id = None,
            }
        }
        let mut out = Vec::with_capacity(parsed.len());
        for (mut item, mut rec) in parsed {
            let id = match rec.id {
                Some(id) => id,
                None => {
                    let id = self.next_id;
                    self.next_id += 1;
                    rec.id = Some(id);
                    item.set("id", Json::from_i64(id));
                    id
                }
            };
            self.items.push((id, item));
            out.push(rec);
        }
        Ok(out)
    }

    fn insert(&mut self, schema: &Schema, rec: &mut Record) -> Result<(), String> {
        let id = self.next_id;
        rec.id = Some(id);
        self.items.push((id, schema.to_json(rec)));
        if let Err(e) = self.save() {
            self.items.pop();
            rec.id = None;
            return Err(e);
        }
        self.next_id += 1;
        Ok(())
    }

    fn update(&mut self, schema: &Schema, rec: &Record) -> Result<(), String> {
        let id = rec.id.ok_or("У записи нет номера.")?;
        let item = self.item_mut(id)?;
        let old = item.clone();
        for (f, v) in schema.fields.iter().zip(&rec.values) {
            item.set(&f.name, v.to_json());
        }
        if let Err(e) = self.save() {
            *self.item_mut(id)? = old;
            return Err(e);
        }
        Ok(())
    }

    fn delete(&mut self, _schema: &Schema, id: i64) -> Result<(), String> {
        let Some(pos) = self.items.iter().position(|(i, _)| *i == id) else { return Ok(()) };
        let old = self.items.remove(pos);
        if let Err(e) = self.save() {
            self.items.insert(pos, old);
            return Err(e);
        }
        Ok(())
    }

    fn name(&self) -> String {
        file_name(&self.path)
    }
}

fn file_name(p: &Path) -> String {
    p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| p.display().to_string())
}

// ----------------------------------------------------------------------
// SQLite
// ----------------------------------------------------------------------

#[cfg(feature = "sqlite")]
pub use sqlite_storage::SqliteStorage;

#[cfg(feature = "sqlite")]
mod sqlite_storage {
    use super::*;
    use crate::mvc::data::{Data, FieldKind};
    use crate::sqlite::{quote_ident, Connection, SqlValue};

    /// Записи в таблице базы SQLite (нужен признак `sqlite`).
    ///
    /// Таблица `schema.table` создаётся сама:
    /// `id INTEGER PRIMARY KEY` и по столбцу на поле (`TEXT`, `INTEGER`,
    /// `REAL`). Если таблица уже есть, недостающие столбцы добавляются.
    pub struct SqliteStorage {
        conn: Connection,
        name: String,
        ready: bool,
    }

    impl SqliteStorage {
        /// Открыть (или создать) файл базы.
        pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
            let p = path.as_ref();
            let conn = Connection::open(p).map_err(|e| format!("{}: {}", p.display(), e))?;
            Ok(SqliteStorage { conn, name: file_name(p), ready: false })
        }

        /// База в памяти — для тестов.
        pub fn in_memory() -> Result<Self, String> {
            let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
            Ok(SqliteStorage { conn, name: "SQLite в памяти".into(), ready: false })
        }

        /// Соединение — для своих запросов.
        pub fn connection(&self) -> &Connection {
            &self.conn
        }

        fn sql_type(k: &FieldKind) -> &'static str {
            match k {
                FieldKind::Int | FieldKind::Bool => "INTEGER",
                FieldKind::Real => "REAL",
                FieldKind::Text | FieldKind::Choice(_) => "TEXT",
            }
        }

        fn ensure_table(&mut self, schema: &Schema) -> Result<(), String> {
            if self.ready {
                return Ok(());
            }
            let t = quote_ident(&schema.table);
            let cols: Vec<String> = schema.fields.iter().map(|f| format!("{} {}", quote_ident(&f.name), Self::sql_type(&f.kind))).collect();
            let sql = format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, {})", t, cols.join(", "));
            self.conn.execute(&sql, &[]).map_err(|e| e.to_string())?;
            // Старая таблица без новых полей — добавить столбцы.
            let have: Vec<String> =
                self.conn.query(&format!("PRAGMA table_info({})", t), &[]).map_err(|e| e.to_string())?.iter().map(|r| r[1].to_text()).collect();
            for f in &schema.fields {
                if !have.iter().any(|c| c.eq_ignore_ascii_case(&f.name)) {
                    let sql = format!("ALTER TABLE {} ADD COLUMN {} {}", t, quote_ident(&f.name), Self::sql_type(&f.kind));
                    self.conn.execute(&sql, &[]).map_err(|e| e.to_string())?;
                }
            }
            self.ready = true;
            Ok(())
        }

        fn to_sql(v: &Data) -> SqlValue {
            match v {
                Data::Null => SqlValue::Null,
                Data::Int(n) => SqlValue::Int(*n),
                Data::Real(x) => SqlValue::Real(*x),
                Data::Text(s) => SqlValue::Text(s.clone()),
                Data::Bool(b) => SqlValue::Int(*b as i64),
            }
        }

        fn from_sql(k: &FieldKind, v: &SqlValue) -> Data {
            match (k, v) {
                (FieldKind::Bool, v) => Data::Bool(v.as_i64().unwrap_or(0) != 0),
                (FieldKind::Text | FieldKind::Choice(_), SqlValue::Null) => Data::Text(String::new()),
                (_, SqlValue::Null) => Data::Null,
                (FieldKind::Int, v) => v.as_i64().map_or(Data::Null, Data::Int),
                (FieldKind::Real, v) => v.as_f64().map_or(Data::Null, Data::Real),
                (FieldKind::Text | FieldKind::Choice(_), v) => Data::Text(v.to_text()),
            }
        }

        fn columns(schema: &Schema) -> Vec<String> {
            schema.fields.iter().map(|f| quote_ident(&f.name)).collect()
        }
    }

    impl Storage for SqliteStorage {
        fn load(&mut self, schema: &Schema) -> Result<Vec<Record>, String> {
            self.ensure_table(schema)?;
            let sql = format!("SELECT id, {} FROM {} ORDER BY id", Self::columns(schema).join(", "), quote_ident(&schema.table));
            let rows = self.conn.query(&sql, &[]).map_err(|e| e.to_string())?;
            Ok(rows
                .into_iter()
                .map(|r| Record {
                    id: r[0].as_i64(),
                    values: schema.fields.iter().enumerate().map(|(i, f)| Self::from_sql(&f.kind, &r[i + 1])).collect(),
                })
                .collect())
        }

        fn insert(&mut self, schema: &Schema, rec: &mut Record) -> Result<(), String> {
            self.ensure_table(schema)?;
            let cols = Self::columns(schema);
            let marks = vec!["?"; cols.len()].join(", ");
            let sql = format!("INSERT INTO {} ({}) VALUES ({})", quote_ident(&schema.table), cols.join(", "), marks);
            let params: Vec<SqlValue> = rec.values.iter().map(Self::to_sql).collect();
            self.conn.execute(&sql, &params).map_err(|e| e.to_string())?;
            rec.id = Some(self.conn.last_insert_rowid());
            Ok(())
        }

        fn update(&mut self, schema: &Schema, rec: &Record) -> Result<(), String> {
            self.ensure_table(schema)?;
            let id = rec.id.ok_or("У записи нет номера.")?;
            let sets: Vec<String> = Self::columns(schema).iter().map(|c| format!("{} = ?", c)).collect();
            let sql = format!("UPDATE {} SET {} WHERE id = ?", quote_ident(&schema.table), sets.join(", "));
            let mut params: Vec<SqlValue> = rec.values.iter().map(Self::to_sql).collect();
            params.push(SqlValue::Int(id));
            match self.conn.execute(&sql, &params).map_err(|e| e.to_string())? {
                0 => Err("Запись не найдена (её удалили?).".into()),
                _ => Ok(()),
            }
        }

        fn delete(&mut self, schema: &Schema, id: i64) -> Result<(), String> {
            self.ensure_table(schema)?;
            let sql = format!("DELETE FROM {} WHERE id = ?", quote_ident(&schema.table));
            self.conn.execute(&sql, &[SqlValue::Int(id)]).map(|_| ()).map_err(|e| e.to_string())
        }

        fn name(&self) -> String {
            self.name.clone()
        }
    }
}

/// Хранилище по расширению файла: `.json` — [`JsonStorage`]; `.db`,
/// `.sqlite`, `.sqlite3` — `SqliteStorage` (с признаком `sqlite`).
pub fn open_storage(path: impl AsRef<Path>) -> Result<Box<dyn Storage>, String> {
    let p = path.as_ref();
    let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    match ext.as_str() {
        "json" => Ok(Box::new(JsonStorage::new(p))),
        "db" | "sqlite" | "sqlite3" => {
            #[cfg(feature = "sqlite")]
            {
                Ok(Box::new(SqliteStorage::open(p)?))
            }
            #[cfg(not(feature = "sqlite"))]
            {
                Err(format!("{}: для баз SQLite соберите программу с признаком «sqlite» (--features sqlite).", p.display()))
            }
        }
        _ => Err(format!("{}: неизвестный формат; нужен .json, .db или .sqlite.", p.display())),
    }
}
