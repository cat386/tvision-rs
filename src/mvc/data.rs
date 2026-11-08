//! Описание данных: значение [`Data`], поле [`Field`], схема [`Schema`],
//! запись [`Record`].

use std::cmp::Ordering;

use crate::json::Json;
use crate::text::{cstr_width, str_width};

/// Значение одного поля записи.
#[derive(Clone, Debug, PartialEq)]
pub enum Data {
    /// Пусто (необязательное число, которое не заполнили).
    Null,
    Int(i64),
    Real(f64),
    Text(String),
    Bool(bool),
}

impl Data {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Data::Text(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Data::Int(n) => Some(*n),
            _ => None,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Data::Int(n) => Some(*n as f64),
            Data::Real(x) => Some(*x),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> bool {
        matches!(self, Data::Bool(true))
    }
    pub fn is_null(&self) -> bool {
        matches!(self, Data::Null)
    }

    /// Текст для строки ввода: число — цифрами, пусто — пустая строка.
    pub fn to_text(&self) -> String {
        match self {
            Data::Null => String::new(),
            Data::Int(n) => n.to_string(),
            Data::Real(x) => x.to_string(),
            Data::Text(s) => s.clone(),
            Data::Bool(b) => if *b { "1" } else { "0" }.to_string(),
        }
    }

    /// Значение JSON.
    pub fn to_json(&self) -> Json {
        match self {
            Data::Null => Json::Null,
            Data::Int(n) => Json::from_i64(*n),
            Data::Real(x) => Json::from_f64(*x),
            Data::Text(s) => Json::Str(s.clone()),
            Data::Bool(b) => Json::Bool(*b),
        }
    }

    /// Сравнение для сортировки: пустые — первыми, числа — как числа,
    /// текст — без учёта регистра.
    pub fn compare(&self, other: &Data) -> Ordering {
        match (self, other) {
            (Data::Null, Data::Null) => Ordering::Equal,
            (Data::Null, _) => Ordering::Less,
            (_, Data::Null) => Ordering::Greater,
            (Data::Bool(a), Data::Bool(b)) => a.cmp(b),
            (Data::Text(a), Data::Text(b)) => a.to_lowercase().cmp(&b.to_lowercase()).then_with(|| a.cmp(b)),
            (a, b) => match (a.as_f64(), b.as_f64()) {
                (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
                _ => a.to_text().cmp(&b.to_text()),
            },
        }
    }
}

impl From<&str> for Data {
    fn from(s: &str) -> Self {
        Data::Text(s.to_string())
    }
}
impl From<String> for Data {
    fn from(s: String) -> Self {
        Data::Text(s)
    }
}
impl From<i64> for Data {
    fn from(n: i64) -> Self {
        Data::Int(n)
    }
}
impl From<f64> for Data {
    fn from(x: f64) -> Self {
        Data::Real(x)
    }
}
impl From<bool> for Data {
    fn from(b: bool) -> Self {
        Data::Bool(b)
    }
}

/// Вид поля — от него зависят элемент диалога, проверка и тип столбца
/// в базе.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldKind {
    /// Строка. В диалоге — строка ввода.
    Text,
    /// Целое число. Строка ввода, в которую можно набрать только цифры и `-`.
    Int,
    /// Дробное число (запятая или точка). Строка ввода.
    Real,
    /// Да/нет. Флажок; в таблице — `✓`.
    Bool,
    /// Один из вариантов. Переключатели; хранится текст варианта.
    Choice(Vec<String>),
}

/// Своя проверка поля: функция и сообщение об ошибке.
pub type FieldCheck = (fn(&str) -> bool, String);

/// Поле записи: имя (ключ в JSON, столбец в базе), подпись и вид.
///
/// ```
/// use tvision::mvc::Field;
///
/// let name = Field::text("name", "~И~мя").required().max_len(60);
/// let age = Field::int("age", "~В~озраст");
/// assert!(name.parse("").is_err());          // обязательное
/// assert!(age.parse("12a").is_err());       // не число
/// assert_eq!(age.parse("").unwrap(), tvision::mvc::Data::Null);
/// ```
#[derive(Clone, Debug)]
pub struct Field {
    /// Имя для хранения: ключ JSON, имя столбца SQLite.
    pub name: String,
    /// Подпись в диалоге и заголовок столбца; `~буква~` — горячая клавиша.
    pub title: String,
    pub kind: FieldKind,
    /// Нельзя оставить пустым.
    pub required: bool,
    /// Ширина столбца таблицы.
    pub width: i32,
    /// Наибольшая длина строки ввода.
    pub max_len: usize,
    /// Своя проверка введённого текста.
    pub check: Option<FieldCheck>,
}

impl Field {
    fn new(name: &str, title: &str, kind: FieldKind, width: usize) -> Field {
        let width = width.max(cstr_width(title) + 1) as i32;
        Field { name: name.to_string(), title: title.to_string(), kind, required: false, width, max_len: 100, check: None }
    }

    /// Текстовое поле.
    pub fn text(name: &str, title: &str) -> Field {
        Field::new(name, title, FieldKind::Text, 20)
    }
    /// Целое число.
    pub fn int(name: &str, title: &str) -> Field {
        let mut f = Field::new(name, title, FieldKind::Int, 8);
        f.max_len = 20;
        f
    }
    /// Дробное число.
    pub fn real(name: &str, title: &str) -> Field {
        let mut f = Field::new(name, title, FieldKind::Real, 10);
        f.max_len = 30;
        f
    }
    /// Да/нет.
    pub fn bool(name: &str, title: &str) -> Field {
        Field::new(name, title, FieldKind::Bool, 3)
    }
    /// Один из вариантов.
    pub fn choice(name: &str, title: &str, options: &[&str]) -> Field {
        let w = options.iter().map(|s| cstr_width(s)).max().unwrap_or(1);
        let mut f = Field::new(name, title, FieldKind::Choice(options.iter().map(|s| s.to_string()).collect()), w);
        f.required = true; // какой-то вариант выбран всегда
        f
    }

    /// Сделать обязательным.
    pub fn required(mut self) -> Field {
        self.required = true;
        self
    }
    /// Ширина столбца таблицы.
    pub fn width(mut self, w: i32) -> Field {
        self.width = w.max(1);
        self
    }
    /// Наибольшая длина строки ввода.
    pub fn max_len(mut self, n: usize) -> Field {
        self.max_len = n.max(1);
        self
    }
    /// Своя проверка: `f(текст)` должна вернуть `true`, иначе — сообщение.
    pub fn check(mut self, f: fn(&str) -> bool, message: &str) -> Field {
        self.check = Some((f, message.to_string()));
        self
    }

    /// Подпись без тильд — для заголовков и сообщений.
    pub fn plain_title(&self) -> String {
        self.title.replace('~', "").trim_end_matches(':').to_string()
    }

    /// Варианты (для `Choice`), иначе пусто.
    pub fn options(&self) -> &[String] {
        match &self.kind {
            FieldKind::Choice(o) => o,
            _ => &[],
        }
    }

    /// Значение нового поля: пустая строка, пустое число, «нет», первый
    /// вариант.
    pub fn default_value(&self) -> Data {
        match &self.kind {
            FieldKind::Text => Data::Text(String::new()),
            FieldKind::Int | FieldKind::Real => Data::Null,
            FieldKind::Bool => Data::Bool(false),
            FieldKind::Choice(o) => Data::Text(o.first().cloned().unwrap_or_default()),
        }
    }

    /// Разобрать введённый текст. Ошибка — готовое сообщение для человека.
    pub fn parse(&self, input: &str) -> Result<Data, String> {
        let s = input.trim();
        let t = self.plain_title();
        if s.is_empty() && !matches!(self.kind, FieldKind::Bool) {
            if self.required {
                return Err(format!("Поле «{}» нужно заполнить.", t));
            }
            return Ok(match self.kind {
                FieldKind::Text => Data::Text(String::new()),
                _ => Data::Null,
            });
        }
        let v = match &self.kind {
            FieldKind::Text => Data::Text(s.to_string()),
            FieldKind::Int => Data::Int(s.parse().map_err(|_| format!("В поле «{}» нужно целое число.", t))?),
            FieldKind::Real => {
                Data::Real(s.replace(',', ".").parse::<f64>().ok().filter(|x| x.is_finite()).ok_or_else(|| format!("В поле «{}» нужно число.", t))?)
            }
            FieldKind::Bool => Data::Bool(matches!(s.to_lowercase().as_str(), "1" | "true" | "да" | "yes" | "✓")),
            FieldKind::Choice(o) => {
                if !o.iter().any(|x| x == s) {
                    return Err(format!("В поле «{}» нет варианта «{}».", t, s));
                }
                Data::Text(s.to_string())
            }
        };
        if let Some((f, msg)) = &self.check {
            if !f(s) {
                return Err(msg.clone());
            }
        }
        Ok(v)
    }

    /// Проверить уже готовое значение (например, прочитанное из файла).
    pub fn validate(&self, v: &Data) -> Result<(), String> {
        let ok_kind = match (&self.kind, v) {
            (_, Data::Null) => true,
            (FieldKind::Text | FieldKind::Choice(_), Data::Text(_)) => true,
            (FieldKind::Int, Data::Int(_)) => true,
            (FieldKind::Real, Data::Real(_) | Data::Int(_)) => true,
            (FieldKind::Bool, Data::Bool(_)) => true,
            _ => false,
        };
        if !ok_kind {
            return Err(format!("Поле «{}»: значение не того вида ({:?}).", self.plain_title(), v));
        }
        if matches!(self.kind, FieldKind::Bool) {
            return Ok(());
        }
        self.parse(&v.to_text()).map(|_| ())
    }

    /// Текст ячейки таблицы.
    pub fn display(&self, v: &Data) -> String {
        match (&self.kind, v) {
            (FieldKind::Bool, Data::Bool(true)) => "✓".into(),
            (FieldKind::Bool, _) => String::new(),
            (_, v) => v.to_text(),
        }
    }

    /// Значение из JSON (неподходящее — значение по умолчанию).
    pub fn from_json(&self, j: Option<&Json>) -> Data {
        let Some(j) = j.filter(|j| !j.is_null()) else {
            return match self.kind {
                FieldKind::Bool => Data::Bool(false),
                FieldKind::Text => Data::Text(String::new()),
                _ => Data::Null,
            };
        };
        match &self.kind {
            FieldKind::Text | FieldKind::Choice(_) => match j {
                Json::Str(s) => Data::Text(s.clone()),
                other => Data::Text(other.cell_text()),
            },
            FieldKind::Int => j.as_i64().or_else(|| j.as_str().and_then(|s| s.trim().parse().ok())).map_or(Data::Null, Data::Int),
            FieldKind::Real => j.as_f64().or_else(|| j.as_str().and_then(|s| s.trim().replace(',', ".").parse().ok())).map_or(Data::Null, Data::Real),
            FieldKind::Bool => Data::Bool(j.as_bool().unwrap_or_else(|| j.as_f64().map_or(false, |x| x != 0.0))),
        }
    }

    /// Ширина подписи в диалоге (без тильд).
    pub(crate) fn title_width(&self) -> usize {
        cstr_width(&self.title)
    }

    /// Ширина элемента диалога.
    pub(crate) fn input_width(&self) -> i32 {
        match &self.kind {
            FieldKind::Bool => cstr_width(&self.title) as i32 + 6,
            FieldKind::Choice(o) => o.iter().map(|s| str_width(s)).max().unwrap_or(1) as i32 + 6,
            FieldKind::Int => 12,
            FieldKind::Real => 16,
            FieldKind::Text => (self.max_len as i32 + 2).clamp(10, 40),
        }
    }
}

/// Схема данных: имя таблицы и поля.
#[derive(Clone, Debug)]
pub struct Schema {
    /// Имя таблицы SQLite (для JSON не используется).
    pub table: String,
    pub fields: Vec<Field>,
}

impl Schema {
    pub fn new(table: &str, fields: Vec<Field>) -> Schema {
        Schema { table: table.to_string(), fields }
    }

    /// Номер поля по имени.
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.fields.iter().position(|f| f.name == name)
    }

    /// Пустая новая запись (без id).
    pub fn new_record(&self) -> Record {
        Record { id: None, values: self.fields.iter().map(Field::default_value).collect() }
    }

    /// Проверить все поля записи.
    pub fn validate(&self, r: &Record) -> Result<(), String> {
        if r.values.len() != self.fields.len() {
            return Err(format!("В записи {} значений, а полей {}.", r.values.len(), self.fields.len()));
        }
        for (f, v) in self.fields.iter().zip(&r.values) {
            f.validate(v)?;
        }
        Ok(())
    }

    /// Запись → объект JSON (`id` первым, потом поля по порядку).
    pub fn to_json(&self, r: &Record) -> Json {
        let mut o = Vec::with_capacity(self.fields.len() + 1);
        if let Some(id) = r.id {
            o.push(("id".to_string(), Json::from_i64(id)));
        }
        for (f, v) in self.fields.iter().zip(&r.values) {
            o.push((f.name.clone(), v.to_json()));
        }
        Json::Obj(o)
    }

    /// Объект JSON → запись (лишние ключи пропускаются).
    pub fn from_json(&self, j: &Json) -> Record {
        Record { id: j.get("id").and_then(Json::as_i64), values: self.fields.iter().map(|f| f.from_json(j.get(&f.name))).collect() }
    }
}

/// Одна запись: номер в хранилище и значения полей (по порядку схемы).
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    /// Номер в хранилище; `None` — запись ещё не сохранена.
    pub id: Option<i64>,
    pub values: Vec<Data>,
}

impl Record {
    /// Запись из значений (без id).
    pub fn new(values: Vec<Data>) -> Record {
        Record { id: None, values }
    }

    /// Значение поля по номеру.
    pub fn get(&self, i: usize) -> &Data {
        self.values.get(i).unwrap_or(&Data::Null)
    }

    /// Текст поля по номеру (пусто — пустая строка).
    pub fn text(&self, i: usize) -> String {
        self.get(i).to_text()
    }
}
