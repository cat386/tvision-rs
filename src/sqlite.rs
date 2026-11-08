//! SQLite: минимальная обёртка над системной библиотекой `libsqlite3`.
//!
//! Модуль включается признаком `sqlite`:
//!
//! ```toml
//! tvision-rs = { git = "https://github.com/cat386/tvision-rs", features = ["sqlite"] }
//! ```
//!
//! Внешних крейтов нет: функции SQLite вызываются напрямую (FFI). Нужна
//! установленная библиотека: в Linux — пакет `libsqlite3-dev` (Debian,
//! Ubuntu) или `sqlite-devel` (Fedora); в macOS она есть всегда; в Windows
//! используется системная `winsqlite3.dll` (Windows 10 и новее).
//!
//! ```
//! use tvision::sqlite::{Connection, SqlValue};
//!
//! let db = Connection::open_in_memory().unwrap();
//! db.execute("CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT)", &[]).unwrap();
//! db.execute("INSERT INTO t (name) VALUES (?)", &[SqlValue::from("Аня")]).unwrap();
//! assert_eq!(db.last_insert_rowid(), 1);
//! let rows = db.query("SELECT id, name FROM t", &[]).unwrap();
//! assert_eq!(rows, vec![vec![SqlValue::Int(1), SqlValue::Text("Аня".into())]]);
//! ```

use std::ffi::{c_char, c_int, CStr, CString};
use std::fmt;
use std::ptr;

#[repr(C)]
struct Sqlite3 {
    _p: [u8; 0],
}
#[repr(C)]
struct Stmt {
    _p: [u8; 0],
}

const SQLITE_OK: c_int = 0;
const SQLITE_ROW: c_int = 100;
const SQLITE_DONE: c_int = 101;
const SQLITE_INTEGER: c_int = 1;
const SQLITE_FLOAT: c_int = 2;
const SQLITE_TEXT: c_int = 3;
const SQLITE_BLOB: c_int = 4;
const SQLITE_OPEN_READWRITE: c_int = 0x02;
const SQLITE_OPEN_CREATE: c_int = 0x04;
const SQLITE_OPEN_URI: c_int = 0x40;
/// «Скопируй строку себе» — для `bind_text`.
const SQLITE_TRANSIENT: isize = -1;

#[cfg_attr(not(windows), link(name = "sqlite3"))]
#[cfg_attr(windows, link(name = "winsqlite3"))]
extern "C" {
    fn sqlite3_open_v2(name: *const c_char, db: *mut *mut Sqlite3, flags: c_int, vfs: *const c_char) -> c_int;
    fn sqlite3_close_v2(db: *mut Sqlite3) -> c_int;
    fn sqlite3_errmsg(db: *mut Sqlite3) -> *const c_char;
    fn sqlite3_prepare_v2(db: *mut Sqlite3, sql: *const c_char, n: c_int, stmt: *mut *mut Stmt, tail: *mut *const c_char) -> c_int;
    fn sqlite3_finalize(stmt: *mut Stmt) -> c_int;
    fn sqlite3_step(stmt: *mut Stmt) -> c_int;
    fn sqlite3_bind_null(stmt: *mut Stmt, i: c_int) -> c_int;
    fn sqlite3_bind_int64(stmt: *mut Stmt, i: c_int, v: i64) -> c_int;
    fn sqlite3_bind_double(stmt: *mut Stmt, i: c_int, v: f64) -> c_int;
    fn sqlite3_bind_text(stmt: *mut Stmt, i: c_int, v: *const c_char, n: c_int, free: isize) -> c_int;
    fn sqlite3_column_count(stmt: *mut Stmt) -> c_int;
    fn sqlite3_column_type(stmt: *mut Stmt, i: c_int) -> c_int;
    fn sqlite3_column_int64(stmt: *mut Stmt, i: c_int) -> i64;
    fn sqlite3_column_double(stmt: *mut Stmt, i: c_int) -> f64;
    fn sqlite3_column_text(stmt: *mut Stmt, i: c_int) -> *const u8;
    fn sqlite3_column_bytes(stmt: *mut Stmt, i: c_int) -> c_int;
    fn sqlite3_column_name(stmt: *mut Stmt, i: c_int) -> *const c_char;
    fn sqlite3_changes(db: *mut Sqlite3) -> c_int;
    fn sqlite3_last_insert_rowid(db: *mut Sqlite3) -> i64;
}

/// Значение ячейки SQLite.
///
/// Двоичные данные (BLOB) читаются как `Text` (с заменой неверных байтов
/// UTF-8) — для учебных программ этого достаточно.
#[derive(Clone, Debug, PartialEq)]
pub enum SqlValue {
    Null,
    Int(i64),
    Real(f64),
    Text(String),
}

impl SqlValue {
    /// Текст для показа: `NULL` — пустая строка.
    pub fn to_text(&self) -> String {
        match self {
            SqlValue::Null => String::new(),
            SqlValue::Int(n) => n.to_string(),
            SqlValue::Real(x) => x.to_string(),
            SqlValue::Text(s) => s.clone(),
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            SqlValue::Int(n) => Some(*n),
            SqlValue::Real(x) if x.fract() == 0.0 => Some(*x as i64),
            SqlValue::Text(s) => s.trim().parse().ok(),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            SqlValue::Int(n) => Some(*n as f64),
            SqlValue::Real(x) => Some(*x),
            SqlValue::Text(s) => s.trim().parse().ok(),
            SqlValue::Null => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, SqlValue::Null)
    }
}

impl From<&str> for SqlValue {
    fn from(s: &str) -> Self {
        SqlValue::Text(s.to_string())
    }
}
impl From<String> for SqlValue {
    fn from(s: String) -> Self {
        SqlValue::Text(s)
    }
}
impl From<i64> for SqlValue {
    fn from(n: i64) -> Self {
        SqlValue::Int(n)
    }
}
impl From<f64> for SqlValue {
    fn from(x: f64) -> Self {
        SqlValue::Real(x)
    }
}
impl From<bool> for SqlValue {
    fn from(b: bool) -> Self {
        SqlValue::Int(b as i64)
    }
}
impl<T: Into<SqlValue>> From<Option<T>> for SqlValue {
    fn from(v: Option<T>) -> Self {
        v.map_or(SqlValue::Null, Into::into)
    }
}

/// Ошибка SQLite: код и текст сообщения.
#[derive(Clone, Debug, PartialEq)]
pub struct SqlError {
    pub code: i32,
    pub msg: String,
}

impl fmt::Display for SqlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SQLite ({}): {}", self.code, self.msg)
    }
}

impl std::error::Error for SqlError {}

/// Результат запроса: имена столбцов и строки.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rows {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<SqlValue>>,
}

/// Открытая база данных. Закрывается при удалении.
///
/// Все запросы — с параметрами `?` (значения передаются отдельно, а не
/// вклеиваются в текст SQL), поэтому кавычки в данных ничего не ломают.
pub struct Connection {
    db: *mut Sqlite3,
}

impl Connection {
    /// Открыть (или создать) файл базы.
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Connection, SqlError> {
        let p = path.as_ref().to_string_lossy().into_owned();
        Self::open_flags(&p, SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE)
    }

    /// База в памяти: исчезает при закрытии. Удобно для тестов.
    pub fn open_in_memory() -> Result<Connection, SqlError> {
        Self::open_flags(":memory:", SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_URI)
    }

    fn open_flags(name: &str, flags: c_int) -> Result<Connection, SqlError> {
        let c = CString::new(name).map_err(|_| SqlError { code: -1, msg: "нулевой байт в имени файла".into() })?;
        let mut db = ptr::null_mut();
        // SAFETY: аргументы — корректные указатели; db заполняет SQLite.
        let rc = unsafe { sqlite3_open_v2(c.as_ptr(), &mut db, flags, ptr::null()) };
        let conn = Connection { db };
        if rc != SQLITE_OK {
            let msg = if db.is_null() { "не хватает памяти".to_string() } else { conn.errmsg() };
            return Err(SqlError { code: rc, msg });
        }
        Ok(conn)
    }

    fn errmsg(&self) -> String {
        // SAFETY: db открыта; строка принадлежит SQLite и живёт до следующего вызова.
        unsafe { CStr::from_ptr(sqlite3_errmsg(self.db)).to_string_lossy().into_owned() }
    }

    fn err(&self, code: c_int) -> SqlError {
        SqlError { code, msg: self.errmsg() }
    }

    fn prepare(&self, sql: &str, params: &[SqlValue]) -> Result<Statement<'_>, SqlError> {
        let c = CString::new(sql).map_err(|_| SqlError { code: -1, msg: "нулевой байт в тексте SQL".into() })?;
        let mut st = ptr::null_mut();
        // SAFETY: строка с нулём на конце, n = -1.
        let rc = unsafe { sqlite3_prepare_v2(self.db, c.as_ptr(), -1, &mut st, ptr::null_mut()) };
        if rc != SQLITE_OK {
            return Err(self.err(rc));
        }
        if st.is_null() {
            return Err(SqlError { code: -1, msg: "пустой запрос".into() });
        }
        let stmt = Statement { conn: self, st };
        for (i, p) in params.iter().enumerate() {
            let i = (i + 1) as c_int;
            // SAFETY: st подготовлен; текст копируется (SQLITE_TRANSIENT).
            let rc = unsafe {
                match p {
                    SqlValue::Null => sqlite3_bind_null(st, i),
                    SqlValue::Int(n) => sqlite3_bind_int64(st, i, *n),
                    SqlValue::Real(x) => sqlite3_bind_double(st, i, *x),
                    SqlValue::Text(s) => sqlite3_bind_text(st, i, s.as_ptr() as *const c_char, s.len() as c_int, SQLITE_TRANSIENT),
                }
            };
            if rc != SQLITE_OK {
                return Err(self.err(rc));
            }
        }
        Ok(stmt)
    }

    /// Выполнить команду без результата (`CREATE`, `INSERT`, `UPDATE`,
    /// `DELETE`…). Возвращает число изменённых строк.
    pub fn execute(&self, sql: &str, params: &[SqlValue]) -> Result<usize, SqlError> {
        let st = self.prepare(sql, params)?;
        loop {
            match st.step()? {
                true => continue, // строки результата не нужны
                false => break,
            }
        }
        drop(st);
        // SAFETY: db открыта.
        Ok(unsafe { sqlite3_changes(self.db) } as usize)
    }

    /// Выполнить несколько команд, разделённых `;` (без параметров).
    pub fn execute_batch(&self, sql: &str) -> Result<(), SqlError> {
        for part in split_statements(sql) {
            self.execute(&part, &[])?;
        }
        Ok(())
    }

    /// Запрос `SELECT`: все строки результата.
    pub fn query(&self, sql: &str, params: &[SqlValue]) -> Result<Vec<Vec<SqlValue>>, SqlError> {
        Ok(self.query_rows(sql, params)?.rows)
    }

    /// Запрос `SELECT` вместе с именами столбцов.
    pub fn query_rows(&self, sql: &str, params: &[SqlValue]) -> Result<Rows, SqlError> {
        let st = self.prepare(sql, params)?;
        // SAFETY: st подготовлен.
        let n = unsafe { sqlite3_column_count(st.st) };
        let columns = (0..n)
            .map(|i| unsafe {
                let p = sqlite3_column_name(st.st, i);
                if p.is_null() {
                    String::new()
                } else {
                    CStr::from_ptr(p).to_string_lossy().into_owned()
                }
            })
            .collect();
        let mut rows = Vec::new();
        while st.step()? {
            rows.push((0..n).map(|i| st.column(i)).collect());
        }
        Ok(Rows { columns, rows })
    }

    /// Номер (`rowid`) последней вставленной строки.
    pub fn last_insert_rowid(&self) -> i64 {
        // SAFETY: db открыта.
        unsafe { sqlite3_last_insert_rowid(self.db) }
    }

    /// Выполнить `f` в транзакции: при ошибке все изменения отменяются.
    pub fn transaction<T>(&self, f: impl FnOnce(&Connection) -> Result<T, SqlError>) -> Result<T, SqlError> {
        self.execute("BEGIN", &[])?;
        match f(self) {
            Ok(v) => {
                self.execute("COMMIT", &[])?;
                Ok(v)
            }
            Err(e) => {
                let _ = self.execute("ROLLBACK", &[]);
                Err(e)
            }
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        if !self.db.is_null() {
            // SAFETY: все Statement уже удалены (они заимствуют Connection).
            unsafe {
                sqlite3_close_v2(self.db);
            }
        }
    }
}

struct Statement<'c> {
    conn: &'c Connection,
    st: *mut Stmt,
}

impl Statement<'_> {
    /// Следующая строка: `true` — есть, `false` — запрос закончен.
    fn step(&self) -> Result<bool, SqlError> {
        // SAFETY: st подготовлен.
        match unsafe { sqlite3_step(self.st) } {
            SQLITE_ROW => Ok(true),
            SQLITE_DONE => Ok(false),
            rc => Err(self.conn.err(rc)),
        }
    }

    fn column(&self, i: c_int) -> SqlValue {
        // SAFETY: st стоит на строке; указатель текста действителен до следующего step.
        unsafe {
            match sqlite3_column_type(self.st, i) {
                SQLITE_INTEGER => SqlValue::Int(sqlite3_column_int64(self.st, i)),
                SQLITE_FLOAT => SqlValue::Real(sqlite3_column_double(self.st, i)),
                SQLITE_TEXT | SQLITE_BLOB => {
                    let p = sqlite3_column_text(self.st, i);
                    let n = sqlite3_column_bytes(self.st, i) as usize;
                    if p.is_null() {
                        SqlValue::Text(String::new())
                    } else {
                        SqlValue::Text(String::from_utf8_lossy(std::slice::from_raw_parts(p, n)).into_owned())
                    }
                }
                _ => SqlValue::Null,
            }
        }
    }
}

impl Drop for Statement<'_> {
    fn drop(&mut self) {
        // SAFETY: st подготовлен и больше не используется.
        unsafe {
            sqlite3_finalize(self.st);
        }
    }
}

/// Разбить текст на команды по `;` вне строк в кавычках.
fn split_statements(sql: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    for c in sql.chars() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if c == '\'' || c == '"' => quote = Some(c),
            None if c == ';' => {
                if !cur.trim().is_empty() {
                    out.push(cur.trim().to_string());
                }
                cur.clear();
                continue;
            }
            None => {}
        }
        cur.push(c);
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

/// Экранировать имя таблицы или столбца для текста SQL: `"имя"`.
pub fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_types() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE t (a INTEGER, b REAL, c TEXT, d TEXT); ").unwrap();
        let n = db
            .execute("INSERT INTO t VALUES (?, ?, ?, ?)", &[SqlValue::Int(-5), SqlValue::Real(2.5), "в ' кавычках".into(), SqlValue::Null])
            .unwrap();
        assert_eq!(n, 1);
        let r = db.query_rows("SELECT a, b, c, d FROM t", &[]).unwrap();
        assert_eq!(r.columns, vec!["a", "b", "c", "d"]);
        assert_eq!(r.rows[0], vec![SqlValue::Int(-5), SqlValue::Real(2.5), SqlValue::Text("в ' кавычках".into()), SqlValue::Null]);
    }

    #[test]
    fn errors_and_transactions() {
        let db = Connection::open_in_memory().unwrap();
        let e = db.execute("SELEKT 1", &[]).unwrap_err();
        assert!(e.msg.contains("syntax"), "{}", e);
        db.execute("CREATE TABLE t (x INTEGER NOT NULL)", &[]).unwrap();
        let r: Result<(), SqlError> = db.transaction(|db| {
            db.execute("INSERT INTO t VALUES (1)", &[])?;
            db.execute("INSERT INTO t VALUES (NULL)", &[])?; // ошибка NOT NULL
            Ok(())
        });
        assert!(r.is_err());
        assert!(db.query("SELECT * FROM t", &[]).unwrap().is_empty()); // откатилось
    }

    #[test]
    fn file_database() {
        let path = std::env::temp_dir().join(format!("tv-sqlite-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        {
            let db = Connection::open(&path).unwrap();
            db.execute("CREATE TABLE t (s TEXT)", &[]).unwrap();
            db.execute("INSERT INTO t VALUES (?)", &["сохранено".into()]).unwrap();
        }
        let db = Connection::open(&path).unwrap();
        assert_eq!(db.query("SELECT s FROM t", &[]).unwrap()[0][0].to_text(), "сохранено");
        drop(db);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn split_and_quote() {
        assert_eq!(split_statements("a; b ';' c ;; d"), vec!["a", "b ';' c", "d"]);
        assert_eq!(quote_ident("a\"b"), "\"a\"\"b\"");
    }
}
