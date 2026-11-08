# Справочник: данные — JSON, SQLite, MVC

Чтение и запись JSON, работа с базой SQLite и каркас «модель —
представление — контроллер» для программ, которые показывают и правят
записи. Уроки — [11. MVC](11-mvc.md) и [12. Рецепты](12-data-recipes.md),
примеры — `mvc_contacts`, `mvc_todo`, `mvc_books`, `mvc_csv`,
`sqlite_query`, `json_settings`.

На этой странице:
[`Json`](#json--значение-json) ·
[`parse_json`](#parse_json-и-parse_json_with) ·
[`JsonError`](#jsonerror) ·
[`Connection`](#connection--база-sqlite) ·
[`SqlValue`](#sqlvalue) ·
[`SqlError`, `Rows`](#sqlerror-и-rows) ·
[`Schema`](#schema--схема) ·
[`Field`](#field--поле) ·
[`FieldKind`](#fieldkind) ·
[`Data`](#data--значение-поля) ·
[`Record`](#record--запись) ·
[`Storage`](#storage--хранилище) ·
[`JsonStorage`](#jsonstorage) ·
[`SqliteStorage`](#sqlitestorage) ·
[`MemoryStorage`](#memorystorage) ·
[`open_storage`](#open_storage) ·
[`Model`](#model--модель) ·
[`ModelWindow`](#modelwindow--окно-с-таблицей) ·
[`ModelTableModel`](#modeltablemodel-и-model_table) ·
[диалог записи](#диалог-записи) ·
[`CrudController`](#crudcontroller--контроллер)

---

## `Json` — значение JSON

**Что это.** Разобранный JSON: модуль `tvision::json`. Внешних
библиотек не нужно.

```rust
use tvision::json::{parse_json, Json};

fn read_settings() {
    let j = parse_json(r#"{"имя": "Аня", "размер": [80, 25], "тёмная": true}"#).unwrap();
    assert_eq!(j.get("имя").and_then(Json::as_str), Some("Аня"));
    assert_eq!(j.get("размер").and_then(|r| r.at(1)).and_then(Json::as_i64), Some(25));
    assert_eq!(j.get("тёмная").and_then(Json::as_bool), Some(true));
    assert!(j.get("нет такого").is_none());
}

fn write_settings() {
    let mut j = Json::object(vec![("имя", Json::from_str("Аня"))]);
    j.set("размер", Json::Arr(vec![Json::from_i64(80), Json::from_i64(25)]));
    assert_eq!(j.compact(), r#"{"имя": "Аня", "размер": [80, 25]}"#);
    let text = j.pretty(); // с отступами — для файла
    assert!(text.contains("\n  \"имя\""));
}
```

Варианты: `Null`, `Bool(bool)`, `Num(String)` (число хранится текстом —
без потери точности), `Str(String)`, `Arr(Vec<Json>)`,
`Obj(Vec<(String, Json)>)` (порядок ключей сохраняется).

| Метод | Что делает |
|-------|-----------|
| `get(ключ)`, `at(номер)` | поле объекта, элемент массива (`Option`) |
| `as_str()`, `as_i64()`, `as_f64()`, `as_bool()` | значение нужного вида (`None` — другой вид) |
| `as_array()`, `as_object()`, `is_null()` | |
| `Json::object(пары)`, `from_str`, `from_i64`, `from_f64` | построить |
| `set(ключ, значение)` | задать поле объекта |
| `compact()`, `pretty()` | текст в одну строку / с отступами |

**Реже:** `write(…)` (в свой буфер), `cell_text()` (коротко для ячейки
таблицы).

## `parse_json` и `parse_json_with`

`parse_json(текст) -> Result<Json, JsonError>` — разобрать.
`parse_json_with(текст, Some(&mut ход))` — то же, но функция
`ход(сделано, всего) -> bool` вызывается по мере разбора (для полосы
выполнения); вернула `false` — разбор прерывается (`Err(None)`).
Пример с полосой выполнения — `examples/json_load.rs`.

## `JsonError`

Ошибка разбора: поля `line`, `col` (с единицы) и `msg`; печатается как
«строка 3, столбец 7: ожидалось …».

---

## `Connection` — база SQLite

**Что это.** Открытая база SQLite: модуль `tvision::sqlite`, только с
признаком `sqlite` (`features = ["sqlite"]`). Работает через системную
`libsqlite3` (в Windows — `winsqlite3.dll`); внешних крейтов нет.

```rust
use tvision::sqlite::{Connection, SqlValue};

fn books() {
    let db = Connection::open_in_memory().unwrap(); // файл: Connection::open("books.db")
    db.execute_batch(
        "CREATE TABLE books (id INTEGER PRIMARY KEY, title TEXT, year INTEGER);
         INSERT INTO books (title, year) VALUES ('Мастер и Маргарита', 1967);",
    )
    .unwrap();
    // Значения — отдельно от текста запроса, на месте «?».
    db.execute("INSERT INTO books (title, year) VALUES (?, ?)", &["Дюна".into(), 1965i64.into()]).unwrap();
    assert_eq!(db.last_insert_rowid(), 2);
    let rows = db.query("SELECT title FROM books WHERE year < ? ORDER BY year", &[SqlValue::Int(1966)]).unwrap();
    assert_eq!(rows, vec![vec![SqlValue::Text("Дюна".into())]]);
}
```

| Метод | Что делает |
|-------|-----------|
| `Connection::open(путь)` | открыть или создать файл базы |
| `Connection::open_in_memory()` | база в памяти (для тестов) |
| `execute(sql, &[параметры])` | команда без результата; вернёт число изменённых строк |
| `execute_batch(sql)` | несколько команд через `;`, без параметров |
| `query(sql, &[параметры])` | строки результата `Vec<Vec<SqlValue>>` |
| `query_rows(sql, &[…])` | то же с именами столбцов ([`Rows`](#sqlerror-и-rows)) |
| `last_insert_rowid()` | номер последней вставленной строки |
| `changes()` | сколько строк изменила последняя команда (даже выполненная через `query`) |
| `transaction(\|db\| { … })` | всё внутри — или ничего: при ошибке изменения отменяются |

**Что важно знать.**

* **Никогда не вклеивайте данные в текст запроса** через `format!` —
  кавычка в имени сломает запрос (а злоумышленник сможет выполнить свой
  SQL). Пишите `?` и передавайте значения вторым аргументом.
* Имя таблицы или столбца параметром передать нельзя; если оно не
  ваше, экранируйте функцией `sqlite::quote_ident(имя)`.
* База закрывается, когда `Connection` удаляется.

## `SqlValue`

Значение ячейки: `Null`, `Int(i64)`, `Real(f64)`, `Text(String)`.
Создаётся через `.into()` из `&str`, `String`, `i64`, `f64`, `bool`
(→ 0/1), `Option<…>` (`None` → `Null`). Методы `to_text()`, `as_i64()`,
`as_f64()`, `is_null()`.

## `SqlError` и `Rows`

`SqlError` — ошибка SQLite: поля `code`, `msg`; печатается как
«SQLite (1): near "SELEKT": syntax error». `Rows` — результат
`query_rows`: поля `columns` (имена) и `rows`.

---

## `Schema` — схема

**Что это.** Описание записи для модуля `tvision::mvc`: имя таблицы
(для SQLite) и поля. Из схемы получаются столбцы таблицы, диалог правки
и проверка ввода.

```rust
use tvision::mvc::*;

fn schema() -> Schema {
    Schema::new(
        "books",
        vec![
            Field::text("title", "~Н~азвание").required().width(30),
            Field::int("year", "~Г~од"),
            Field::real("price", "~Ц~ена"),
            Field::bool("read", "~П~рочитана"),
            Field::choice("shelf", "~П~олка", &["Дома", "На работе"]),
        ],
    )
}

fn schema_basics() {
    let s = schema();
    assert_eq!(s.index_of("year"), Some(1));
    let r = s.new_record(); // пустая запись: "", пусто, пусто, нет, «Дома»
    assert!(s.validate(&r).is_err()); // название обязательно
    let j = s.to_json(&Record::new(vec!["Дюна".into(), Data::Int(1965), Data::Null, true.into(), "Дома".into()]));
    assert_eq!(j.compact(), r#"{"title": "Дюна", "year": 1965, "price": null, "read": true, "shelf": "Дома"}"#);
}
```

| Метод | Что делает |
|-------|-----------|
| `Schema::new(таблица, поля)` | |
| `index_of(имя)` | номер поля |
| `new_record()` | пустая запись со значениями по умолчанию |
| `validate(&запись)` | проверить все поля |
| `to_json(&запись)`, `from_json(&json)` | запись ↔ объект JSON |

Поля `table` и `fields` открыты.

## `Field` — поле

**Что это.** Одно поле записи: имя для хранения (`name`), подпись
(`title`, с `~горячей~` буквой), вид и правила.

```rust
use tvision::mvc::*;

fn fields() {
    let age = Field::int("age", "~В~озраст");
    assert_eq!(age.parse(" 42 ").unwrap(), Data::Int(42));
    assert_eq!(age.parse("").unwrap(), Data::Null); // необязательное — можно пусто
    assert_eq!(age.parse("сорок").unwrap_err(), "В поле «Возраст» нужно целое число.");

    let mail = Field::text("email", "Почта").check(|s| s.contains('@'), "В адресе нет «@».");
    assert!(mail.parse("a@b").is_ok() && mail.parse("ab").is_err());

    let vip = Field::bool("vip", "VIP");
    assert_eq!(vip.display(&Data::Bool(true)), "✓");
}
```

| Создать | Вид |
|---------|-----|
| `Field::text(имя, подпись)` | строка |
| `Field::int(…)` | целое число |
| `Field::real(…)` | дробное (запятая или точка) |
| `Field::bool(…)` | да/нет |
| `Field::choice(…, &[варианты])` | один из вариантов (всегда обязателен) |

| Уточнение | Что делает |
|-----------|-----------|
| `.required()` | нельзя оставить пустым |
| `.max_len(n)` | длина строки ввода (по умолчанию 100) |
| `.width(n)` | ширина столбца таблицы |
| `.check(функция, сообщение)` | своя проверка текста; для пустого необязательного поля не вызывается |

**Реже:** `parse(текст)` (текст → значение или сообщение об ошибке),
`validate(&значение)`, `display(&значение)` (текст ячейки),
`default_value()`, `plain_title()` (подпись без тильд), `options()`,
`from_json(…)`; тип `FieldCheck`.

## `FieldKind`

Вид поля: `Text`, `Int`, `Real`, `Bool`, `Choice(Vec<String>)`. Обычно
задаётся конструктором `Field::…`.

## `Data` — значение поля

`Null` (пусто), `Int(i64)`, `Real(f64)`, `Text(String)`, `Bool(bool)`.
Создаётся через `.into()` из `&str`, `String`, `i64`, `f64`, `bool`.
Методы: `as_str()`, `as_i64()`, `as_f64()`, `as_bool()`, `is_null()`,
`to_text()` (для строки ввода), `to_json()`, `compare(&другое)` (для
сортировки: пустые первыми, текст без учёта регистра).

## `Record` — запись

Поля `id: Option<i64>` (номер в хранилище; `None` — ещё не сохранена) и
`values: Vec<Data>` (по порядку полей схемы). `Record::new(значения)`,
`get(i)` (значение), `text(i)` (значение текстом).

---

## `Storage` — хранилище

**Что это.** Трейт «где лежат записи». Модель вызывает его методы и не
знает формата. Каждый метод **сразу** сохраняет изменение.

| Метод | Что делает |
|-------|-----------|
| `load(&схема)` | прочитать все записи |
| `insert(&схема, &mut запись)` | добавить, записать новый номер в `запись.id` |
| `update(&схема, &запись)` | сохранить по `запись.id` |
| `delete(&схема, id)` | удалить |
| `name()` | имя для заголовка окна |

Своё хранилище (CSV, сервер по сети) — реализуйте эти пять методов.

## `JsonStorage`

Файл JSON — массив объектов с ключом `"id"`. `JsonStorage::new(путь)`,
`path()`. Нет файла — нет записей; объект без `"id"` (или с
повторяющимся) получает номер при чтении; ключи не из схемы
сохраняются; файл переписывается целиком через временный файл.

## `SqliteStorage`

Таблица `схема.table` в базе SQLite (признак `sqlite`).
`SqliteStorage::open(путь)`, `SqliteStorage::in_memory()`,
`connection()` — для своих запросов. Таблица создаётся сама
(`id INTEGER PRIMARY KEY` + столбцы `TEXT`, `INTEGER`, `REAL`);
недостающие столбцы добавляются. `Bool` хранится как 0/1.

## `MemoryStorage`

Записи только в памяти — для тестов: `MemoryStorage::new()`,
`MemoryStorage::with_records(записи)`.

## `open_storage`

`open_storage(путь) -> Result<Box<dyn Storage>, String>` — хранилище по
расширению: `.json` → `JsonStorage`; `.db`, `.sqlite`, `.sqlite3` →
`SqliteStorage` (без признака `sqlite` — ошибка с подсказкой).

---

## `Model` — модель

**Что это.** Записи в памяти + хранилище. Каждое изменение: проверка
по схеме → сохранение → изменение в памяти. Если что-то не удалось,
ничего не меняется.

```rust
use tvision::mvc::*;

fn model() {
    let schema = Schema::new("t", vec![Field::text("name", "Имя").required(), Field::int("n", "Число")]);
    let mut m = Model::open(schema, Box::new(MemoryStorage::new())).unwrap();
    m.add(Record::new(vec!["Вера".into(), Data::Int(3)])).unwrap();
    m.add(Record::new(vec!["Анна".into(), Data::Int(1)])).unwrap();
    m.sort_by(0, true); // по имени
    let i = m.add(Record::new(vec!["Борис".into(), Data::Null])).unwrap();
    assert_eq!(i, 1); // встала между Анной и Верой
    assert_eq!(m.get(0).unwrap().id, Some(2)); // id — от хранилища
    assert!(m.add(Record::new(vec!["".into(), Data::Null])).is_err()); // имя обязательно
    m.remove(2).unwrap(); // Вера
    assert_eq!(m.len(), 2);
}
```

| Метод | Что делает |
|-------|-----------|
| `Model::open(схема, хранилище)` | создать и прочитать записи |
| `into_ref()` | завернуть в `ModelRef` (= `Rc<RefCell<Model>>`) |
| `len()`, `is_empty()`, `get(i)`, `records()` | записи |
| `add(запись)` | добавить; вернёт номер строки |
| `update(i, запись)` | заменить (id сохраняется); вернёт новый номер строки |
| `remove(i)` | удалить; вернёт удалённую запись |
| `sort_by(поле, по_возрастанию)` | сортировать (и потом держать порядок) |
| `reload()` | перечитать из хранилища |

**Реже:** `schema()`, `index_of_id(id)`, `revision()` (растёт при каждом
изменении), `storage_name()`, `sort_state()`, `unsort()`.

## `ModelWindow` — окно с таблицей

**Что это.** Окно с таблицей записей модели. Таблица растягивается с
окном, щелчок по заголовку сортирует модель, Enter и двойной щелчок
посылают `CM_RECORD_EDIT`. Если модель изменилась (её изменили не
через окно), окно обновляется само.

| Метод / поле | Что |
|--------------|-----|
| `ModelWindow::new(прямоугольник, заголовок, модель)` | окно; в заголовке ещё имя файла и число записей |
| `current()` | номер записи под курсором |
| `select(i)` | поставить курсор |
| `refresh()` | обновить таблицу и заголовок |
| `table()`, `table_mut()` | сама [`Table`](27-data-views.md#table--таблица) |
| поля `win`, `table`, `model`, `caption` | |

## `ModelTableModel` и `model_table`

`ModelTableModel { model }` — [`TableModel`](27-data-views.md#tablemodel--откуда-таблица-берёт-данные)
над моделью: для своего окна. `model_table(прямоугольник, модель,
полоса_в, полоса_г)` — готовая [`Table`](27-data-views.md#table--таблица)
со столбцами по схеме (ширина из `Field::width`, числа справа, `✓` по
центру).

## Диалог записи

| Функция | Что делает |
|---------|-----------|
| `edit_record(app, &схема, &запись, заголовок)` | показать диалог; `Some(запись)` — нажали OK |
| `record_dialog(&схема, &запись, заголовок)` | только собрать `Dialog` (строки ввода, флажки, переключатели по видам полей) |
| `record_value(&схема, &запись)` | данные для `Dialog::set_data` |
| `record_from_value(&схема, &value)` | запись из `Dialog::get_data` (или сообщение об ошибке) |

Строки ввода проверяются при OK по правилам поля; ошибка — окно с
сообщением, диалог остаётся открытым.

## `CrudController` — контроллер

**Что это.** Команды «Добавить», «Изменить», «Удалить» для одного окна
`ModelWindow`: показывает диалог, меняет модель, ставит курсор на
изменённую запись, сообщает об ошибках хранилища.

```rust
use tvision::mvc::*;
use tvision::*;

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

fn start(app: &mut Program) -> App {
    let schema = Schema::new("notes", vec![Field::text("text", "~Т~екст").required().max_len(200)]);
    let model = Model::open(schema, Box::new(MemoryStorage::new())).unwrap().into_ref();
    let r = app.desktop().group.vs.extent();
    let mut crud = CrudController::open(app, r, "Заметки", model);
    crud.add_title = "Новая заметка".into();
    crud.confirm_delete = false; // удалять без вопроса
    App { crud }
}
```

| Метод / поле | Что |
|--------------|-----|
| `CrudController::open(app, прямоугольник, заголовок, модель)` | вставить окно и создать контроллер |
| `CrudController::new(модель, id_окна)` | для окна, вставленного вами |
| `handle(app, ev) -> bool` | выполнить команду записи, если окно активно |
| `update_commands(app)` | разрешить/запретить команды по состоянию |
| `add(app)`, `edit(app)`, `delete(app)` | то же, что команды, — вызвать напрямую |
| `window(app)`, `is_active(app)` | окно контроллера; активно ли оно |
| поля `model`, `window`, `add_title`, `edit_title`, `confirm_delete` | |

Команды: `CM_RECORD_ADD` (180), `CM_RECORD_EDIT` (181),
`CM_RECORD_DELETE` (182). Свои команды программы нумеруйте так, чтобы
не пересечься с ними.
