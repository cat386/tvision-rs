//! Пример: открыть файл JSON и показать данные в таблице.
//!
//! Запуск:
//! ```text
//! cargo run --example json_table                              # пустое окно, F3 — открыть
//! cargo run --example json_table -- examples/data/books.json  # сразу открыть файл
//! ```
//!
//! * F3 — диалог «Открыть JSON» (`FileDialog` с шаблоном `*.json`).
//! * Файл загружается в отдельном потоке; на экране — диалог «Загрузка
//!   файла» с полосой выполнения (`ProgressBar`) и этапами: чтение, разбор
//!   JSON, построение таблицы. «Отмена» (или Esc) прерывает загрузку.
//! * F2 — создать большой тестовый файл (по умолчанию 100 000 записей) и
//!   открыть его: на маленьких файлах полосу почти не видно.
//! * Каждый файл открывается в своём окне с таблицей: номера строк,
//!   чередование цветов, сортировка щелчком по заголовку, ширина столбцов
//!   подобрана по данным, числа выровнены вправо.
//! * Enter, F4 или двойной щелчок по строке — окно «Запись» со всеми полями
//!   записи, вложенные объекты и массивы показаны целиком.
//! * Ошибка в файле — сообщение с номером строки и столбца.
//!
//! Какие JSON понимаются (как они превращаются в таблицу — см. [`to_table`]):
//! * массив объектов `[{...}, {...}]` — строка на объект, столбец на ключ;
//!   вложенные объекты разворачиваются в столбцы `адрес.город`;
//! * объект с массивом внутри `{"items": [...]}` — берётся первый массив;
//! * объект объектов `{"id1": {...}, "id2": {...}}` — первый столбец «ключ»;
//! * массив массивов `[[1, 2], [3, 4]]` — столбцы A, B, …;
//! * массив значений `[1, 2, 3]` и одиночный объект — тоже показываются.
//!
//! Внешних зависимостей у библиотеки нет, поэтому разбор JSON здесь свой —
//! маленький, но полный (строки с `\uXXXX`, числа, вложенность, ошибки с
//! позицией). В своей программе можно взять `serde_json`.

use std::path::{Path, PathBuf};

use tvision::*;

const CM_OPEN_JSON: u16 = 100;
/// Создать большой тестовый файл и открыть его.
const CM_GENERATE: u16 = 102;
/// Показать запись под курсором (F4; Enter и двойной щелчок делают то же).
const CM_SHOW_RECORD: u16 = 101;

// ======================================================================
// Разбор JSON
// ======================================================================

/// Значение JSON. Порядок ключей объекта сохраняется (как в файле) —
/// от него зависит порядок столбцов.
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    /// Число хранится текстом, как в файле: `1e3`, `0.10` и большие целые
    /// показываются без искажений.
    Num(String),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

/// Ошибка разбора с позицией (строка и столбец — с 1).
#[derive(Clone, Debug, PartialEq)]
pub struct JsonError {
    pub line: usize,
    pub col: usize,
    pub msg: String,
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "строка {}, столбец {}: {}", self.line, self.col, self.msg)
    }
}

/// Функция хода разбора: получает (разобрано символов, всего символов),
/// возвращает `false`, чтобы прервать разбор.
pub type ParseHook<'a> = &'a mut dyn FnMut(usize, usize) -> bool;

/// Разборщик: позиция в массиве символов. Время жизни текста (`'a`) и
/// функции хода (`'h`) разные: текст создаётся внутри `parse_json_with`.
struct Parser<'a, 'h> {
    s: &'a [char],
    i: usize,
    /// Сообщать о ходе разбора (для полосы выполнения).
    hook: Option<&'h mut dyn FnMut(usize, usize) -> bool>,
    /// Позиция, на которой в следующий раз позвать `hook`.
    next_report: usize,
    /// Разбор прерван через `hook`.
    cancelled: bool,
}

/// Как часто сообщать о ходе разбора (в символах).
const REPORT_STEP: usize = 64 * 1024;

/// Разобрать текст JSON. Допускается метка порядка байт (BOM) в начале.
pub fn parse_json(text: &str) -> Result<Json, JsonError> {
    parse_json_with(text, None).map_err(|e| e.unwrap_or_else(|| unreachable!("без hook разбор не прерывается")))
}

/// Разбор с сообщениями о ходе работы. `Err(None)` — разбор прерван.
pub fn parse_json_with(text: &str, hook: Option<ParseHook<'_>>) -> Result<Json, Option<JsonError>> {
    let chars: Vec<char> = text.trim_start_matches('\u{feff}').chars().collect();
    let mut p = Parser { s: &chars, i: 0, hook, next_report: 0, cancelled: false };
    p.ws();
    let v = p.value(0);
    if p.cancelled {
        return Err(None);
    }
    let v = v.map_err(Some)?;
    p.ws();
    if p.i < p.s.len() {
        return Err(Some(p.err("лишние символы после конца данных")));
    }
    if let Some(h) = p.hook.as_mut() {
        h(p.s.len(), p.s.len());
    }
    Ok(v)
}

impl<'a, 'h> Parser<'a, 'h> {
    /// Ошибка в текущей позиции.
    fn err(&self, msg: &str) -> JsonError {
        let mut line = 1;
        let mut col = 1;
        for &c in &self.s[..self.i.min(self.s.len())] {
            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        JsonError { line, col, msg: msg.into() }
    }

    fn peek(&self) -> Option<char> {
        self.s.get(self.i).copied()
    }

    /// Пропустить пробелы, табуляции и переводы строк.
    fn ws(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.i += 1;
        }
    }

    /// Ожидаемое слово (`true`, `false`, `null`).
    fn word(&mut self, w: &str, v: Json) -> Result<Json, JsonError> {
        for c in w.chars() {
            if self.peek() != Some(c) {
                return Err(self.err(&format!("ожидалось «{}»", w)));
            }
            self.i += 1;
        }
        Ok(v)
    }

    fn value(&mut self, depth: usize) -> Result<Json, JsonError> {
        // Защита от переполнения стека на «бесконечной» вложенности.
        if depth > 200 {
            return Err(self.err("слишком глубокая вложенность"));
        }
        // Сообщить о ходе разбора; `false` — прервать.
        if self.i >= self.next_report {
            self.next_report = self.i + REPORT_STEP;
            let (i, n) = (self.i, self.s.len());
            if let Some(h) = self.hook.as_mut() {
                if !h(i, n) {
                    self.cancelled = true;
                    return Err(self.err("прервано"));
                }
            }
        }
        match self.peek() {
            Some('{') => self.object(depth),
            Some('[') => self.array(depth),
            Some('"') => Ok(Json::Str(self.string()?)),
            Some('t') => self.word("true", Json::Bool(true)),
            Some('f') => self.word("false", Json::Bool(false)),
            Some('n') => self.word("null", Json::Null),
            Some(c) if c == '-' || c.is_ascii_digit() => self.number(),
            Some(c) => Err(self.err(&format!("неожиданный символ «{}»", c))),
            None => Err(self.err("неожиданный конец файла")),
        }
    }

    fn object(&mut self, depth: usize) -> Result<Json, JsonError> {
        self.i += 1; // '{'
        let mut fields = Vec::new();
        self.ws();
        if self.peek() == Some('}') {
            self.i += 1;
            return Ok(Json::Obj(fields));
        }
        loop {
            self.ws();
            if self.peek() != Some('"') {
                return Err(self.err("ожидалось имя поля в кавычках"));
            }
            let key = self.string()?;
            self.ws();
            if self.peek() != Some(':') {
                return Err(self.err("ожидалось «:»"));
            }
            self.i += 1;
            self.ws();
            let v = self.value(depth + 1)?;
            fields.push((key, v));
            self.ws();
            match self.peek() {
                Some(',') => self.i += 1,
                Some('}') => {
                    self.i += 1;
                    return Ok(Json::Obj(fields));
                }
                _ => return Err(self.err("ожидалось «,» или «}»")),
            }
        }
    }

    fn array(&mut self, depth: usize) -> Result<Json, JsonError> {
        self.i += 1; // '['
        let mut items = Vec::new();
        self.ws();
        if self.peek() == Some(']') {
            self.i += 1;
            return Ok(Json::Arr(items));
        }
        loop {
            self.ws();
            items.push(self.value(depth + 1)?);
            self.ws();
            match self.peek() {
                Some(',') => self.i += 1,
                Some(']') => {
                    self.i += 1;
                    return Ok(Json::Arr(items));
                }
                _ => return Err(self.err("ожидалось «,» или «]»")),
            }
        }
    }

    /// Четыре шестнадцатеричные цифры после `\u`.
    fn hex4(&mut self) -> Result<u32, JsonError> {
        let mut v = 0;
        for _ in 0..4 {
            let d = self.peek().and_then(|c| c.to_digit(16)).ok_or_else(|| self.err("ожидались 4 шестнадцатеричные цифры после \\u"))?;
            v = v * 16 + d;
            self.i += 1;
        }
        Ok(v)
    }

    fn string(&mut self) -> Result<String, JsonError> {
        self.i += 1; // '"'
        let mut out = String::new();
        loop {
            match self.peek() {
                None => return Err(self.err("незакрытая строка")),
                Some('"') => {
                    self.i += 1;
                    return Ok(out);
                }
                Some('\\') => {
                    self.i += 1;
                    let c = self.peek().ok_or_else(|| self.err("незакрытая строка"))?;
                    self.i += 1;
                    match c {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        '/' => out.push('/'),
                        'b' => out.push('\u{8}'),
                        'f' => out.push('\u{c}'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => {
                            let hi = self.hex4()?;
                            // Суррогатная пара: 😀 → 😀
                            let code = if (0xD800..0xDC00).contains(&hi) && self.peek() == Some('\\') && self.s.get(self.i + 1) == Some(&'u') {
                                self.i += 2;
                                let lo = self.hex4()?;
                                0x10000 + ((hi - 0xD800) << 10) + (lo.wrapping_sub(0xDC00) & 0x3FF)
                            } else {
                                hi
                            };
                            out.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
                        }
                        _ => {
                            self.i -= 1;
                            return Err(self.err(&format!("неизвестная escape-последовательность «\\{}»", c)));
                        }
                    }
                }
                Some(c) if (c as u32) < 0x20 => return Err(self.err("управляющий символ внутри строки")),
                Some(c) => {
                    out.push(c);
                    self.i += 1;
                }
            }
        }
    }

    fn number(&mut self) -> Result<Json, JsonError> {
        let start = self.i;
        if self.peek() == Some('-') {
            self.i += 1;
        }
        let digits = |p: &mut Parser<'_, '_>| {
            let s = p.i;
            while matches!(p.peek(), Some(c) if c.is_ascii_digit()) {
                p.i += 1;
            }
            p.i > s
        };
        if !digits(self) {
            return Err(self.err("ожидалась цифра"));
        }
        if self.peek() == Some('.') {
            self.i += 1;
            if !digits(self) {
                return Err(self.err("ожидалась цифра после точки"));
            }
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.i += 1;
            if matches!(self.peek(), Some('+' | '-')) {
                self.i += 1;
            }
            if !digits(self) {
                return Err(self.err("ожидалась цифра в показателе степени"));
            }
        }
        Ok(Json::Num(self.s[start..self.i].iter().collect()))
    }
}

impl Json {
    /// Компактная запись JSON (для вложенных значений в ячейке).
    pub fn compact(&self) -> String {
        let mut s = String::new();
        self.write(&mut s, None, 0);
        s
    }

    /// Запись JSON с отступами (для окна «Запись»).
    pub fn pretty(&self) -> String {
        let mut s = String::new();
        self.write(&mut s, Some(2), 0);
        s
    }

    fn write(&self, out: &mut String, indent: Option<usize>, level: usize) {
        let nl = |out: &mut String, l: usize| {
            if let Some(n) = indent {
                out.push('\n');
                out.push_str(&" ".repeat(n * l));
            }
        };
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Json::Num(n) => out.push_str(n),
            Json::Str(s) => {
                out.push('"');
                for c in s.chars() {
                    match c {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        '\n' => out.push_str("\\n"),
                        '\t' => out.push_str("\\t"),
                        c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                        c => out.push(c),
                    }
                }
                out.push('"');
            }
            Json::Arr(a) => {
                if a.is_empty() {
                    out.push_str("[]");
                    return;
                }
                out.push('[');
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                        if indent.is_none() {
                            out.push(' ');
                        }
                    }
                    nl(out, level + 1);
                    v.write(out, indent, level + 1);
                }
                nl(out, level);
                out.push(']');
            }
            Json::Obj(o) => {
                if o.is_empty() {
                    out.push_str("{}");
                    return;
                }
                out.push('{');
                for (i, (k, v)) in o.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                        if indent.is_none() {
                            out.push(' ');
                        }
                    }
                    nl(out, level + 1);
                    Json::Str(k.clone()).write(out, None, 0);
                    out.push_str(": ");
                    v.write(out, indent, level + 1);
                }
                nl(out, level);
                out.push('}');
            }
        }
    }

    /// Текст для ячейки таблицы: строки — без кавычек, массивы значений —
    /// через запятую, остальное вложенное — компактным JSON.
    pub fn cell_text(&self) -> String {
        match self {
            Json::Null => "null".into(),
            Json::Bool(b) => b.to_string(),
            Json::Num(n) => n.clone(),
            // Переводы строк в ячейке заменяем, чтобы строка таблицы не «ломалась».
            Json::Str(s) => s.replace(['\n', '\r', '\t'], " "),
            Json::Arr(a) if a.iter().all(|v| !matches!(v, Json::Arr(_) | Json::Obj(_))) => {
                a.iter().map(|v| v.cell_text()).collect::<Vec<_>>().join(", ")
            }
            other => other.compact(),
        }
    }
}

// ======================================================================
// JSON → таблица
// ======================================================================

/// Тип данных столбца — от него зависит выравнивание.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColKind {
    Number,
    Bool,
    Text,
}

/// Данные для таблицы: заголовки, строки, тип столбцов и исходные записи
/// (для окна «Запись»).
#[derive(Clone, Debug, PartialEq)]
pub struct JsonTable {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub kinds: Vec<ColKind>,
    pub records: Vec<Json>,
    /// Откуда взяты строки: `""` — весь файл, `"items"` — поле объекта.
    pub source: String,
}

/// Развернуть объект в пары «столбец — значение»: вложенные объекты дают
/// столбцы через точку (`адрес.город`), не глубже `depth` уровней.
fn flatten(prefix: &str, obj: &[(String, Json)], depth: usize, out: &mut Vec<(String, Json)>) {
    for (k, v) in obj {
        let name = if prefix.is_empty() { k.clone() } else { format!("{}.{}", prefix, k) };
        match v {
            Json::Obj(inner) if depth > 0 && !inner.is_empty() => flatten(&name, inner, depth - 1, out),
            _ => out.push((name, v.clone())),
        }
    }
}

/// Найти массив, который стоит показывать: сам корень или первое поле-массив
/// объекта (`{"data": [...]}`), если оно непустое.
fn find_rows(root: &Json) -> (String, Option<&Vec<Json>>) {
    match root {
        Json::Arr(a) => (String::new(), Some(a)),
        Json::Obj(fields) => {
            for (k, v) in fields {
                if let Json::Arr(a) = v {
                    if !a.is_empty() {
                        return (k.clone(), Some(a));
                    }
                }
            }
            (String::new(), None)
        }
        _ => (String::new(), None),
    }
}

/// Превратить JSON в таблицу.
pub fn to_table(root: &Json) -> JsonTable {
    to_table_with(root, &mut |_, _| true).expect("без прерывания")
}

/// То же с сообщениями о ходе: `hook(готово строк, всего строк)` (строки
/// проходятся дважды — сбор столбцов и заполнение ячеек, поэтому «всего» —
/// удвоенное число записей). `None` — прервано.
pub fn to_table_with(root: &Json, hook: &mut dyn FnMut(usize, usize) -> bool) -> Option<JsonTable> {
    let (source, arr) = find_rows(root);
    // Строки таблицы: список (ключ, запись). Ключ есть у «объекта объектов».
    let mut items: Vec<(Option<String>, Json)> = match (arr, root) {
        (Some(a), _) => a.iter().map(|v| (None, v.clone())).collect(),
        // Объект, все значения которого — объекты: строка на значение.
        (None, Json::Obj(f)) if !f.is_empty() && f.iter().all(|(_, v)| matches!(v, Json::Obj(_))) => {
            f.iter().map(|(k, v)| (Some(k.clone()), v.clone())).collect()
        }
        // Одиночный объект или значение — одна строка.
        (None, other) => vec![(None, other.clone())],
    };
    let keyed = items.iter().any(|(k, _)| k.is_some());

    // Столбцы в порядке первого появления ключа.
    let mut headers: Vec<String> = Vec::new();
    let mut rows_kv: Vec<Vec<(String, Json)>> = Vec::new();
    let all_arrays = items.iter().all(|(_, v)| matches!(v, Json::Arr(_)));
    let total = items.len() * 2;
    for (n, (key, v)) in items.iter_mut().enumerate() {
        if n % 1000 == 0 && !hook(n, total) {
            return None;
        }
        let mut kv = Vec::new();
        if let Some(k) = key {
            kv.push(("ключ".to_string(), Json::Str(k.clone())));
        }
        match v {
            Json::Obj(o) => flatten("", o, 3, &mut kv),
            // Массив массивов: столбцы A, B, C…
            Json::Arr(a) if all_arrays => {
                for (i, x) in a.iter().enumerate() {
                    kv.push((column_letters(i), x.clone()));
                }
            }
            other => kv.push(("значение".to_string(), other.clone())),
        }
        for (k, _) in &kv {
            if !headers.contains(k) {
                headers.push(k.clone());
            }
        }
        rows_kv.push(kv);
    }
    if keyed {
        // «ключ» — всегда первый столбец.
        headers.retain(|h| h != "ключ");
        headers.insert(0, "ключ".into());
    }

    // Ячейки и типы столбцов. Отсутствующее поле — пустая ячейка.
    let mut kinds = vec![None::<ColKind>; headers.len()];
    let mut rows: Vec<Vec<String>> = Vec::with_capacity(rows_kv.len());
    let half = rows_kv.len();
    for (n, kv) in rows_kv.iter().enumerate() {
        if n % 1000 == 0 && !hook(half + n, total) {
            return None;
        }
        let row: Vec<String> = headers
            .iter()
            .enumerate()
            .map(|(ci, h)| match kv.iter().find(|(k, _)| k == h) {
                Some((_, v)) => {
                    let k = match v {
                        Json::Num(_) => Some(ColKind::Number),
                        Json::Bool(_) => Some(ColKind::Bool),
                        Json::Null => None,
                        _ => Some(ColKind::Text),
                    };
                    if let Some(k) = k {
                        kinds[ci] = match kinds[ci] {
                            None => Some(k),
                            Some(old) if old == k => Some(k),
                            _ => Some(ColKind::Text),
                        };
                    }
                    v.cell_text()
                }
                None => String::new(),
            })
            .collect();
        rows.push(row);
    }
    hook(total, total);
    let records = items.into_iter().map(|(_, v)| v).collect();
    Some(JsonTable { headers, rows, kinds: kinds.into_iter().map(|k| k.unwrap_or(ColKind::Text)).collect(), records, source })
}

// ======================================================================
// Загрузка файла с ходом работы
// ======================================================================

/// Этап загрузки.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Чтение файла: (прочитано байт, всего байт).
    Read,
    /// Разбор JSON: (разобрано символов, всего символов).
    Parse,
    /// Построение таблицы: (готово строк, всего строк × 2).
    Table,
}

impl Stage {
    /// Доля этапа в общей полосе: (начало, конец) в тысячных.
    fn span(self) -> (u64, u64) {
        match self {
            Stage::Read => (0, 300),
            Stage::Parse => (300, 850),
            Stage::Table => (850, 1000),
        }
    }

    /// Общий ход загрузки в тысячных (0–1000).
    pub fn permille(self, done: u64, total: u64) -> u64 {
        let (a, b) = self.span();
        let f = if total == 0 { 1.0 } else { (done as f64 / total as f64).min(1.0) };
        a + ((b - a) as f64 * f) as u64
    }
}

/// Результат неудачной загрузки.
#[derive(Clone, Debug, PartialEq)]
pub enum LoadError {
    /// Пользователь нажал «Отмена».
    Cancelled,
    /// Ошибка чтения или разбора — текст для сообщения.
    Failed(String),
}

/// Прочитать и разобрать файл, сообщая о ходе работы:
/// `report(этап, сделано, всего)` → `false` прерывает загрузку.
pub fn load_with(path: &Path, report: &mut dyn FnMut(Stage, u64, u64) -> bool) -> Result<JsonTable, LoadError> {
    use std::io::Read;
    let fail = |e: std::io::Error| LoadError::Failed(format!("Не удалось прочитать файл\n{}\n{}", path.display(), e));

    // 1. Чтение кусками по 256 КБ — чтобы было что показывать на полосе.
    let mut f = std::fs::File::open(path).map_err(fail)?;
    let total = f.metadata().map(|m| m.len()).unwrap_or(0);
    let mut bytes = Vec::with_capacity(total as usize);
    let mut buf = vec![0u8; 256 * 1024];
    loop {
        if !report(Stage::Read, bytes.len() as u64, total) {
            return Err(LoadError::Cancelled);
        }
        let n = f.read(&mut buf).map_err(fail)?;
        if n == 0 {
            break;
        }
        bytes.extend_from_slice(&buf[..n]);
    }
    let text = String::from_utf8(bytes).map_err(|_| LoadError::Failed("Файл не в кодировке UTF-8.".into()))?;

    // 2. Разбор JSON.
    let mut hook = |done: usize, all: usize| report(Stage::Parse, done as u64, all as u64);
    let json = match parse_json_with(&text, Some(&mut hook)) {
        Ok(j) => j,
        Err(None) => return Err(LoadError::Cancelled),
        Err(Some(e)) => return Err(LoadError::Failed(format!("Ошибка в JSON:\n{}", e))),
    };
    drop(text);

    // 3. Построение таблицы.
    to_table_with(&json, &mut |done, all| report(Stage::Table, done as u64, all as u64)).ok_or(LoadError::Cancelled)
}

/// Прочитать и разобрать файл без полосы выполнения.
pub fn load(path: &Path) -> Result<JsonTable, String> {
    load_with(path, &mut |_, _, _| true).map_err(|e| match e {
        LoadError::Failed(m) => m,
        LoadError::Cancelled => "Прервано".into(),
    })
}

/// Сообщения от потока загрузки.
enum LoadMsg {
    Progress { stage: Stage, done: u64, total: u64 },
    Done(Result<JsonTable, LoadError>),
}

/// «12,3 МБ»
fn megabytes(b: u64) -> String {
    format!("{:.1} МБ", b as f64 / (1024.0 * 1024.0)).replace('.', ",")
}

/// «123 456» — разряды через пробел.
fn grouped(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

/// Текст этапа под полосой.
fn stage_text(stage: Stage, done: u64, total: u64) -> String {
    match stage {
        Stage::Read => format!("Чтение файла: {} из {}", megabytes(done), megabytes(total)),
        Stage::Parse => format!("Разбор JSON: {}%", if total == 0 { 100 } else { done * 100 / total }),
        Stage::Table => format!("Построение таблицы: {} из {} записей", grouped(done.min(total) / 2), grouped(total / 2)),
    }
}

/// Загрузить файл в отдельном потоке, показывая модальный диалог с полосой
/// выполнения. Кнопка «Отмена» (или Esc) останавливает поток.
pub fn load_with_progress(app: &mut Program, path: &Path) -> Result<JsonTable, LoadError> {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc};

    let name = path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let mut d = Dialog::centered(58, 11, "Загрузка файла");
    d.add(StaticText::new(Rect::new(3, 2, 55, 3), &name));
    let stage = d.add(StaticText::new(Rect::new(3, 3, 55, 4), "Открытие…"));
    let bar = d.add(ProgressBar::new(Rect::new(3, 5, 55, 6), 1000));
    let cancel = d.add(Button::new(Rect::new(23, 7, 35, 9), "Отмена", cm::CANCEL, bf::DEFAULT));
    d.select(cancel);

    // Поток: загрузка с отчётами не чаще, чем раз в 1/1000 общего хода.
    let (tx, rx) = mpsc::channel();
    let stop = Arc::new(AtomicBool::new(false));
    let worker = {
        let (path, stop, tx) = (path.to_path_buf(), stop.clone(), tx.clone());
        std::thread::spawn(move || {
            let mut last = (None, u64::MAX);
            let r = load_with(&path, &mut |st, done, total| {
                let pm = st.permille(done, total);
                // Отправляем при смене этапа или заметном продвижении.
                if last.0 != Some(st) || pm != last.1 {
                    last = (Some(st), pm);
                    let _ = tx.send(LoadMsg::Progress { stage: st, done, total });
                }
                !stop.load(Ordering::Relaxed)
            });
            let _ = tx.send(LoadMsg::Done(r));
        })
    };
    drop(tx);

    let mut result: Option<Result<JsonTable, LoadError>> = None;
    let (cmd, _) = app.exec_dialog_idle(d, |app, _| {
        // Забираем всё, что поток прислал с прошлого раза.
        let mut last = None;
        loop {
            match rx.try_recv() {
                Ok(LoadMsg::Progress { stage, done, total }) => last = Some((stage, done, total)),
                Ok(LoadMsg::Done(r)) => {
                    result = Some(r);
                    return Some(cm::OK);
                }
                Err(mpsc::TryRecvError::Empty) => break,
                // Поток завершился, не прислав результат (например, паника).
                Err(mpsc::TryRecvError::Disconnected) => {
                    result = Some(Err(LoadError::Failed("Загрузка прервалась из-за внутренней ошибки.".into())));
                    return Some(cm::OK);
                }
            }
        }
        if let Some((st, done, total)) = last {
            if let Some(b) = app.find_mut::<ProgressBar>(bar) {
                b.set_value(st.permille(done, total) as i64);
            }
            if let Some(t) = app.find_mut::<StaticText>(stage) {
                t.text = stage_text(st, done, total);
            }
        }
        None
    });
    stop.store(true, Ordering::Relaxed);
    let _ = worker.join();
    if cmd != cm::OK {
        return Err(LoadError::Cancelled);
    }
    result.unwrap_or(Err(LoadError::Cancelled))
}

// ======================================================================
// Тестовый файл
// ======================================================================

/// Записать большой файл JSON с `count` записями — чтобы увидеть полосу
/// выполнения на настоящем объёме (маленькие файлы грузятся мгновенно).
pub fn generate_sample(path: &Path, count: usize) -> std::io::Result<()> {
    use std::io::Write;
    const CITIES: [&str; 8] = ["Москва", "Санкт-Петербург", "Казань", "Новосибирск", "Екатеринбург", "Тула", "Самара", "Пермь"];
    const NAMES: [&str; 10] = ["Анна", "Борис", "Вера", "Глеб", "Дарья", "Егор", "Жанна", "Захар", "Ирина", "Кирилл"];
    const GOODS: [&str; 6] = ["чайник", "лампа", "книга", "кружка", "плед", "часы"];
    let mut w = std::io::BufWriter::new(std::fs::File::create(path)?);
    writeln!(w, "{{\n  \"описание\": \"Тестовые заказы\",\n  \"заказы\": [")?;
    // Простой генератор псевдослучайных чисел — данные одинаковые при каждом запуске.
    let mut x: u64 = 0x2545F4914F6CDD1D;
    let mut rnd = |n: u64| {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x % n
    };
    for i in 0..count {
        let name = NAMES[rnd(10) as usize];
        let city = CITIES[rnd(8) as usize];
        let qty = 1 + rnd(9);
        let price = 100 + rnd(9900);
        let paid = rnd(3) != 0;
        let g1 = GOODS[rnd(6) as usize];
        let g2 = GOODS[rnd(6) as usize];
        let comma = if i + 1 < count { "," } else { "" };
        writeln!(
            w,
            "    {{\"номер\": {}, \"клиент\": \"{} №{}\", \"адрес\": {{\"город\": \"{}\", \"индекс\": \"{:06}\"}}, \"товары\": [\"{}\", \"{}\"], \"количество\": {}, \"сумма\": {}.{:02}, \"оплачен\": {}}}{}",
            i + 1,
            name,
            i + 1,
            city,
            100000 + rnd(900000),
            g1,
            g2,
            qty,
            price * qty,
            rnd(100),
            paid,
            comma
        )?;
    }
    writeln!(w, "  ]\n}}")?;
    w.flush()
}

// ======================================================================
// Окна
// ======================================================================

/// Окно с таблицей данных одного файла.
pub struct JsonWindow {
    pub win: Window,
    pub table: ViewId,
    pub data: JsonTable,
}

impl JsonWindow {
    pub fn new(bounds: Rect, path: &Path, data: JsonTable, number: i16) -> Self {
        let name = path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let title = if data.source.is_empty() { name } else { format!("{} → {}", name, data.source) };
        let mut win = Window::new(bounds, &title, number);
        win.group.vs.options |= of::TILEABLE;
        let v = win.standard_scroll_bar(true);
        let h = win.standard_scroll_bar(false);
        let headers: Vec<&str> = data.headers.iter().map(|s| s.as_str()).collect();
        let model = VecTableModel::new(&headers, data.rows.clone());
        // Столбцы: числа — вправо, true/false — по центру.
        let cols = data
            .kinds
            .iter()
            .map(|k| {
                TableColumn::new(10).limits(3, 40).align(match k {
                    ColKind::Number => Align::Right,
                    ColKind::Bool => Align::Center,
                    ColKind::Text => Align::Left,
                })
            })
            .collect();
        let mut t = Table::new(win.interior(), Box::new(model), Some(v), Some(h))
            .with_columns(cols)
            .row_numbers(true)
            .striped(true)
            .sortable(true)
            .selection(SelectionMode::Row);
        // Ширина по первым 200 строкам.
        t.auto_width(200);
        let table = win.add(t);
        JsonWindow { win, table, data }
    }

    /// Номер записи под курсором (с учётом сортировки — по содержимому строки).
    fn current_record(&self) -> Option<usize> {
        let t = self.win.group.find::<Table>(self.table)?;
        let (row, _) = t.cursor();
        let m = t.model();
        if row >= m.row_count() {
            return None;
        }
        // После сортировки порядок строк в модели меняется — ищем исходную
        // запись по совпадению всех ячеек.
        let cells: Vec<String> = (0..m.col_count()).map(|c| m.cell(row, c)).collect();
        self.data.rows.iter().position(|r| *r == cells)
    }

    /// Окно «Запись» со всеми полями.
    fn show_record(&self, ctx: &mut Ctx) {
        let i = match self.current_record() {
            Some(i) => i,
            None => return,
        };
        let d = record_dialog(&self.data.records[i], i, ctx.screen_size);
        ctx.exec(Box::new(d), None, 0);
    }
}

impl View for JsonWindow {
    tvision::impl_group_basics!(win.group);

    fn as_window(&self) -> Option<&Window> {
        Some(&self.win)
    }

    fn draw(&self, dc: &mut DrawCtx) {
        self.win.draw_window(dc);
    }

    fn palette(&self) -> &'static [u8] {
        self.win.palette
    }

    fn set_state(&mut self, flag: u16, enable: bool) {
        self.win.set_window_state(flag, enable);
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        // Enter — показать запись (таблица без правки Enter не использует).
        if let Event::KeyDown(k) = ev {
            if k.key == Key::Enter && k.mods == Mods::NONE && self.win.group.vs.is_focused() {
                self.show_record(ctx);
                ev.clear();
                return;
            }
        }
        // Команда из строки состояния / меню (F4).
        if ev.command() == Some(CM_SHOW_RECORD) {
            self.show_record(ctx);
            ev.clear();
            return;
        }
        // Двойной щелчок по строке: сначала таблица ставит курсор, потом — окно записи.
        let double = matches!(ev, Event::MouseDown(m) if m.double);
        self.win.handle_window_event(ev, ctx);
        if double {
            self.show_record(ctx);
        }
    }
}

/// Диалог «Запись N»: таблица «поле — значение», вложенное — JSON с отступами
/// (каждая строка JSON — отдельная строка таблицы).
pub fn record_dialog(rec: &Json, index: usize, screen: Point) -> Dialog {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut fields = Vec::new();
    match rec {
        Json::Obj(o) => flatten("", o, 3, &mut fields),
        other => fields.push(("значение".to_string(), other.clone())),
    }
    for (k, v) in fields {
        // Массив простых значений — одной строкой через запятую.
        let simple_arr = matches!(&v, Json::Arr(a) if a.iter().all(|x| !matches!(x, Json::Arr(_) | Json::Obj(_))));
        match v {
            Json::Arr(_) | Json::Obj(_) if !simple_arr => {
                for (i, line) in v.pretty().lines().enumerate() {
                    rows.push(vec![if i == 0 { k.clone() } else { String::new() }, line.to_string()]);
                }
            }
            other => rows.push(vec![k, other.cell_text()]),
        }
    }
    let w = (screen.x - 4).clamp(40, 76);
    // Строки таблицы + заголовок таблицы (2) + рамка и кнопка (7).
    let h = (rows.len() as i32 + 9).clamp(10, screen.y - 3);
    let mut d = Dialog::centered(w, h, &format!("Запись {}", index + 1));
    let sb = d.add(ScrollBar::new(Rect::new(w - 3, 2, w - 2, h - 4)));
    let model = VecTableModel::new(&["Поле", "Значение"], rows);
    let mut t = Table::new(Rect::new(2, 2, w - 3, h - 4), Box::new(model), Some(sb), None)
        .with_columns(vec![TableColumn::new(16).limits(4, 30), TableColumn::new(w - 24).limits(10, 200)])
        .grid(true)
        .selection(SelectionMode::Row);
    t.auto_width(500);
    // «Значение» занимает всё оставшееся место (граница между столбцами — 1 клетка).
    let table_w = w - 5;
    t.columns[1].width = (table_w - t.columns[0].width - 2).max(t.columns[1].width);
    let tid = d.add(t);
    d.add(Button::new(Rect::new(w / 2 - 6, h - 3, w / 2 + 6, h - 1), "O~K~", cm::OK, bf::DEFAULT));
    d.select(tid);
    d
}

// ======================================================================
// Приложение
// ======================================================================

struct App {
    windows: i16,
}

impl App {
    /// Открыть файл в новом окне (или показать ошибку). Загрузка идёт в
    /// потоке, пока на экране диалог с полосой выполнения.
    fn open(&mut self, app: &mut Program, path: &Path) -> bool {
        match load_with_progress(app, path) {
            Ok(data) => {
                self.windows += 1;
                let r = app.desktop().group.vs.extent();
                let n = if self.windows < 10 { self.windows } else { 0 };
                app.insert_window(JsonWindow::new(r, path, data, n));
                true
            }
            Err(LoadError::Cancelled) => false,
            Err(LoadError::Failed(e)) => {
                message_box_rect(app, Rect::new(0, 0, 60, 11), &e, mf::ERROR | mf::OK_BUTTON, true);
                false
            }
        }
    }

    /// Создать тестовый файл заданного размера во временном каталоге и открыть.
    fn generate(&mut self, app: &mut Program) {
        let n = match input_box(app, "Тестовый файл", "~З~аписей:", "100000", 8) {
            Some(t) => match t.trim().replace(' ', "").parse::<usize>() {
                Ok(n) if n > 0 => n.min(2_000_000),
                _ => {
                    message_box(app, "\x03Нужно целое число больше нуля.", mf::ERROR | mf::OK_BUTTON);
                    return;
                }
            },
            None => return,
        };
        let path = std::env::temp_dir().join(format!("tvision-заказы-{}.json", n));
        if let Err(e) = generate_sample(&path, n) {
            message_box(app, &format!("Не удалось создать файл:\n{}", e), mf::ERROR | mf::OK_BUTTON);
            return;
        }
        self.open(app, &path);
    }

    /// Диалог выбора файла.
    fn open_dialog(&mut self, app: &mut Program) {
        let d = FileDialog::new("*.json", "Открыть JSON", "~И~мя", fd::OPEN_BUTTON, 100);
        let (cmd, d) = app.exec_dialog(d);
        if cmd != cm::CANCEL {
            let p: PathBuf = d.file_name();
            self.open(app, &p);
        }
    }
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        match ev.command() {
            Some(CM_OPEN_JSON) => self.open_dialog(app),
            Some(CM_GENERATE) => self.generate(app),
            _ => return,
        }
        ev.clear();
    }
}

fn menu() -> Menu {
    Menu::new()
        .submenu(
            "~Ф~айл",
            Menu::new()
                .item_key("~О~ткрыть JSON...", CM_OPEN_JSON, kb::F3, "F3")
                .item_key("~С~оздать тестовый файл...", CM_GENERATE, kb::F2, "F2")
                .item_key("~З~апись...", CM_SHOW_RECORD, kb::f(4), "F4")
                .separator()
                .item_key("~В~ыход", cm::QUIT, kb::ALT_X, "Alt-X"),
        )
        .submenu(
            "~О~кна",
            Menu::new()
                .item("~М~озаика", cm::TILE)
                .item("~К~аскад", cm::CASCADE)
                .separator()
                .item_key("Р~а~спахнуть", cm::ZOOM, kb::F5, "F5")
                .item_key("~С~ледующее", cm::NEXT, kb::F6, "F6")
                .item_key("~З~акрыть", cm::CLOSE, kb::ALT_F3, "Alt-F3"),
        )
}

fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![
        StatusItem::new("~F3~ Открыть", kb::F3, CM_OPEN_JSON),
        StatusItem::new("~F2~ Тестовый файл", kb::F2, CM_GENERATE),
        // Не вешаем на Enter: строка состояния видит клавиши первой и
        // забрала бы Enter и у диалогов.
        StatusItem::new("~F4~ Запись", kb::f(4), CM_SHOW_RECORD),
        StatusItem::new("~Alt-F3~ Закрыть", kb::ALT_F3, cm::CLOSE),
        StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT),
        StatusItem::new("", kb::F10, cm::MENU),
        StatusItem::new("", kb::F5, cm::ZOOM),
        StatusItem::new("", kb::F6, cm::NEXT),
    ])]
}

fn main() -> std::io::Result<()> {
    let mut app = Program::with_terminal(Some(menu()), Some(status()))?;
    let mut h = App { windows: 0 };
    let files: Vec<String> = std::env::args().skip(1).collect();
    for f in &files {
        h.open(&mut app, Path::new(f));
    }
    if files.is_empty() {
        // Сразу предложить выбрать файл.
        app.put_event(Event::command_event(CM_OPEN_JSON));
    }
    app.run(&mut h);
    Ok(())
}

// ======================================================================
// Тесты: cargo test --example json_table
// ======================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tvision::platform::{HeadlessBackend, RawEvent};

    fn s(x: &str) -> Json {
        Json::Str(x.into())
    }
    fn n(x: &str) -> Json {
        Json::Num(x.into())
    }

    #[test]
    fn parse_values() {
        assert_eq!(parse_json(" null ").unwrap(), Json::Null);
        assert_eq!(parse_json("[true, false, -1.5e3, 0]").unwrap(), Json::Arr(vec![Json::Bool(true), Json::Bool(false), n("-1.5e3"), n("0")]));
        assert_eq!(
            parse_json(r#"{"b": 1, "a": {"x": []}}"#).unwrap(),
            Json::Obj(vec![("b".into(), n("1")), ("a".into(), Json::Obj(vec![("x".into(), Json::Arr(vec![]))]))])
        );
        // Экранирование и суррогатные пары.
        assert_eq!(parse_json(r#""\"Привет\"\nЖ 😀""#).unwrap(), s("\"Привет\"\nЖ 😀"));
        assert_eq!(parse_json("\u{feff}[1]").unwrap(), Json::Arr(vec![n("1")]));
    }

    #[test]
    fn parse_errors_with_position() {
        let e = parse_json("[1, 2,\n  ]").unwrap_err();
        assert_eq!((e.line, e.col), (2, 3), "{}", e);
        let e = parse_json(r#"{"a" 1}"#).unwrap_err();
        assert!(e.msg.contains("«:»"), "{}", e);
        assert!(parse_json("[1] x").is_err());
        assert!(parse_json(r#""abc"#).is_err());
        assert!(parse_json("01x").is_err());
        assert!(parse_json(&"[".repeat(1000)).unwrap_err().msg.contains("вложенность"));
    }

    #[test]
    fn roundtrip_compact() {
        let src = r#"{"a": [1, {"b": "x\ty"}], "c": null}"#;
        let j = parse_json(src).unwrap();
        assert_eq!(j.compact(), r#"{"a": [1, {"b": "x\ty"}], "c": null}"#);
        assert_eq!(parse_json(&j.pretty()).unwrap(), j);
    }

    #[test]
    fn array_of_objects() {
        let j = parse_json(r#"[{"id": 1, "имя": "Аня", "адрес": {"город": "Тула"}}, {"id": 2, "имя": "Борис", "тэги": ["a", "b"], "ok": true}]"#)
            .unwrap();
        let t = to_table(&j);
        assert_eq!(t.headers, vec!["id", "имя", "адрес.город", "тэги", "ok"]);
        assert_eq!(t.rows[0], vec!["1", "Аня", "Тула", "", ""]);
        assert_eq!(t.rows[1], vec!["2", "Борис", "", "a, b", "true"]);
        assert_eq!(t.kinds, vec![ColKind::Number, ColKind::Text, ColKind::Text, ColKind::Text, ColKind::Bool]);
    }

    #[test]
    fn other_shapes() {
        // Объект с массивом внутри.
        let t = to_table(&parse_json(r#"{"total": 2, "items": [{"a": 1}, {"a": 2}]}"#).unwrap());
        assert_eq!((t.source.as_str(), t.rows.len()), ("items", 2));
        // Объект объектов: столбец «ключ».
        let t = to_table(&parse_json(r#"{"x": {"v": 1}, "y": {"v": 2}}"#).unwrap());
        assert_eq!(t.headers, vec!["ключ", "v"]);
        assert_eq!(t.rows[1], vec!["y", "2"]);
        // Массив массивов.
        let t = to_table(&parse_json("[[1, 2], [3, 4, 5]]").unwrap());
        assert_eq!(t.headers, vec!["A", "B", "C"]);
        assert_eq!(t.rows[0], vec!["1", "2", ""]);
        // Массив значений и смешанный тип столбца.
        let t = to_table(&parse_json(r#"[1, "два", null]"#).unwrap());
        assert_eq!(t.headers, vec!["значение"]);
        assert_eq!(t.kinds, vec![ColKind::Text]);
        assert_eq!(t.rows[2], vec!["null"]);
    }

    /// Тестовая программа 100×30 с заданным вводом.
    /// Тестовый терминал сам не нажимает Esc (`esc_budget = 0`): иначе он
    /// отменил бы загрузку, пока поток ещё читает файл.
    fn program(input: Vec<RawEvent>) -> Program {
        let mut b = HeadlessBackend::new(100, 30);
        b.input.extend(input);
        b.esc_budget = 0;
        Program::new(Box::new(b), Some(menu()), Some(status()))
    }

    /// Временный каталог для файлов теста.
    fn tmp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("json-table-{}-{}", name, std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/data/books.json")
    }

    #[test]
    fn sample_file_in_window() {
        let mut app = program(vec![]);
        let mut h = App { windows: 0 };
        assert!(h.open(&mut app, &sample_path()));
        app.redraw();
        let screen = app.screen_text();
        assert!(screen.contains("books.json → книги"), "заголовок окна:\n{}", screen);
        assert!(screen.contains("название"), "{}", screen);
        assert!(screen.contains("Мастер и Маргарита"), "{}", screen);
        assert!(screen.contains(" 1967│"), "числа выровнены вправо:\n{}", screen);
        // Вложенный объект развёрнут в столбцы (они правее, за краем окна).
        let w = app.desktop().group.children.last().unwrap().as_any().downcast_ref::<JsonWindow>().unwrap();
        assert!(w.data.headers.contains(&"издательство.город".to_string()));
        assert_eq!(w.data.rows.len(), 8);
    }

    #[test]
    fn record_window_on_enter() {
        // Окно вставляем напрямую (без диалога загрузки), а тестовый терминал
        // сам «нажмёт» Esc, когда ввод кончится, — и закроет окно записи.
        let mut b = HeadlessBackend::new(100, 30);
        b.input.push_back(RawEvent::Key(kb::ENTER));
        let mut app = Program::new(Box::new(b), Some(menu()), Some(status()));
        let mut h = App { windows: 0 };
        let data = load(&sample_path()).unwrap();
        let r = app.desktop().group.vs.extent();
        app.insert_window(JsonWindow::new(r, &sample_path(), data, 1));
        // Шагаем, пока окно записи не откроется (оно модальное).
        let mut seen = String::new();
        for _ in 0..10 {
            app.step(&mut h);
            let t = app.screen_text();
            if t.contains("Запись 1") {
                seen = t;
                break;
            }
        }
        assert!(seen.contains("Запись 1"), "{}", app.screen_text());
        assert!(seen.contains("жанры"), "{}", seen);
    }

    #[test]
    fn record_after_sort() {
        // После сортировки по году первая строка — «Евгений Онегин» (1833),
        // и окно записи должно показать именно её, а не первую в файле.
        let mut app = program(vec![]);
        let mut h = App { windows: 0 };
        h.open(&mut app, &sample_path());
        let id = app.desktop().group.children.last().unwrap().vs().id;
        let w = app.find_mut::<JsonWindow>(id).unwrap();
        let year = w.data.headers.iter().position(|h| h == "год").unwrap();
        let tid = w.table;
        let t = w.win.group.find_mut::<Table>(tid).unwrap();
        assert!(t.sort_by(year, true));
        t.set_cursor(0, 0);
        let rec = w.current_record().unwrap();
        assert_eq!(w.data.rows[rec][1], "Евгений Онегин");
    }

    #[test]
    fn bad_file_shows_error() {
        let p = tmp("bad").join("bad.json");
        std::fs::write(&p, "{\n  \"a\": [1, 2,\n}").unwrap();
        let mut app = program(vec![]);
        match load_with_progress(&mut app, &p) {
            Err(LoadError::Failed(m)) => assert!(m.contains("строка 3, столбец 1"), "{}", m),
            other => panic!("ожидалась ошибка разбора: {:?}", other.map(|t| t.rows.len())),
        }
    }

    #[test]
    fn stages_in_order() {
        // Этапы идут по порядку, общий ход растёт и доходит до 1000.
        let p = tmp("stages").join("orders.json");
        generate_sample(&p, 3000).unwrap();
        let mut seen: Vec<(Stage, u64)> = Vec::new();
        let t = load_with(&p, &mut |st, done, total| {
            seen.push((st, st.permille(done, total)));
            true
        })
        .unwrap();
        assert_eq!(t.rows.len(), 3000);
        assert!(t.headers.contains(&"адрес.город".to_string()));
        let stages: Vec<Stage> = seen.iter().map(|x| x.0).collect();
        assert_eq!(stages.first(), Some(&Stage::Read));
        assert_eq!(stages.last(), Some(&Stage::Table));
        assert!(stages.contains(&Stage::Parse));
        assert!(seen.windows(2).all(|w| w[0].1 <= w[1].1), "ход не убывает");
        assert_eq!(seen.last().unwrap().1, 1000);
    }

    #[test]
    fn load_can_be_cancelled() {
        // Esc в диалоге загрузки = «Отмена»: поток останавливается, окна нет.
        let p = tmp("cancel").join("big.json");
        generate_sample(&p, 50_000).unwrap();
        let mut app = program(vec![RawEvent::Key(kb::ESC)]);
        let mut h = App { windows: 0 };
        let t0 = std::time::Instant::now();
        assert!(!h.open(&mut app, &p));
        assert_eq!(app.desktop().group.children.len(), 1, "окно не открыто");
        assert!(t0.elapsed().as_secs() < 5, "поток остановился быстро");
    }

    #[test]
    fn cancel_inside_parse() {
        // Прерывание посреди разбора JSON (а не только между этапами).
        let p = tmp("cancel2").join("big.json");
        generate_sample(&p, 20_000).unwrap();
        let r = load_with(&p, &mut |st, done, total| !(st == Stage::Parse && done * 2 > total));
        assert_eq!(r.unwrap_err(), LoadError::Cancelled);
    }

    #[test]
    fn progress_dialog_shows_stage() {
        // Кадр диалога загрузки: заголовок, имя файла, полоса и «Отмена».
        let p = tmp("frame").join("orders.json");
        generate_sample(&p, 200).unwrap();
        let mut app = program(vec![]);
        load_with_progress(&mut app, &p).unwrap();
        // Последний нарисованный кадр — ещё с диалогом (окно таблицы не вставлялось).
        let frame = app.screen_text();
        assert!(frame.contains("Загрузка файла"), "{}", frame);
        assert!(frame.contains("orders.json"), "{}", frame);
        assert!(frame.contains("Отмена"), "{}", frame);
    }

    #[test]
    fn number_formatting() {
        assert_eq!(grouped(1234567), "1 234 567");
        assert_eq!(grouped(999), "999");
        assert_eq!(megabytes(1536 * 1024), "1,5 МБ");
        assert_eq!(stage_text(Stage::Table, 4000, 10000), "Построение таблицы: 2 000 из 5 000 записей");
    }
}
