# Урок 12. Рецепты: данные

Урок 11 показал готовый каркас: схема, модель, окно и контроллер. Здесь
— десять рецептов «как сделать, если нужно чуть больше». Каждый взят из
готовой программы-примера, которую можно запустить и прочитать целиком.

| Пример | Что в нём | Рецепты |
|--------|-----------|---------|
| `mvc_todo` | список задач: приоритет, срок, «сделано» | [1](#рецепт-1-своя-команда-меняет-запись), [2](#рецепт-2-команда-над-всеми-записями), [3](#рецепт-3-сводка-в-заголовке-окна) |
| `mvc_books` | каталог книг со своим окном, поиском и статистикой | [4](#рецепт-4-своё-окно-над-моделью), [5](#рецепт-5-правка-без-контроллера), [6](#рецепт-6-поиск-по-записям) |
| `mvc_csv` | склад в файле CSV | [7](#рецепт-7-своё-хранилище) |
| `sqlite_query` | консоль SQL: запрос сверху, результат в таблице | [8](#рецепт-8-запросы-sql-без-mvc), [9](#рецепт-9-окно-из-строки-ввода-и-таблицы) |
| `json_settings` | настройки программы в JSON | [10](#рецепт-10-настройки-в-json) |

Запуск: `cargo run --example mvc_todo` (и так далее). Примеры с MVC
открывают файл во временном каталоге, а с аргументом — ваш:
`cargo run --example mvc_books -- книги.json`, или базу:
`cargo run --example mvc_books --features sqlite -- книги.db`.
`sqlite_query` работает только с признаком `sqlite`.

---

## Рецепт 1. Своя команда меняет запись

**Задача.** F5 отмечает задачу «сделано» или снимает отметку.

**Как.** Взять запись из модели, поменять поле и вернуть через
`update`. Таблицу трогать не нужно: модель изменилась — окно
обновится само.

```rust,ignore
const F_DONE: usize = 3;   // номер поля в схеме — константой, а не «магическим» числом

fn toggle_done(&mut self, app: &mut Program) {
    let Some(i) = self.crud.window(app).and_then(|w| w.current()) else { return };
    let res = {
        let mut m = self.crud.model.borrow_mut();
        let mut r = m.get(i).unwrap().clone();
        r.values[F_DONE] = Data::Bool(!r.get(F_DONE).as_bool());
        m.update(i, r)                                  // проверка + сохранение
    };                                                  // ← borrow_mut закончился здесь
    match res {
        Ok(j) => self.crud.window(app).unwrap().select(j), // курсор — за записью
        Err(e) => { message_box(app, &e, mf::ERROR | mf::OK_BUTTON); }
    }
}
```

**Важно.** Фигурные скобки вокруг `borrow_mut` — не украшение: модель
нужно «отпустить» до `message_box`, потому что окно сообщения
перерисовывает экран, а окну для рисования нужен `borrow`. Если
запись после изменения переехала (модель отсортирована), `update`
вернёт её новый номер `j`.

## Рецепт 2. Команда над всеми записями

**Задача.** «Удалить сделанные».

```rust,ignore
let mut m = self.crud.model.borrow_mut();
for i in (0..m.len()).rev() {                   // с конца!
    if m.get(i).unwrap().get(F_DONE).as_bool() {
        m.remove(i)?;
    }
}
```

**Важно.** Удаляйте **с конца**: после `remove(i)` записи правее
сдвигаются на одну позицию влево, и при проходе с начала каждая вторая
подходящая запись была бы пропущена.

## Рецепт 3. Сводка в заголовке окна

**Задача.** «Задачи: осталось 3 из 4» в рамке окна.

**Как.** В `idle` посчитать по записям и записать в `caption` окна —
это заголовок без имени файла; `refresh()` соберёт полный.

```rust,ignore
fn idle(&mut self, app: &mut Program) {
    self.crud.update_commands(app);
    let (left, total) = {
        let m = self.crud.model.borrow();
        (m.records().iter().filter(|r| !r.get(F_DONE).as_bool()).count(), m.len())
    };
    if let Some(w) = self.crud.window(app) {
        let t = format!("Задачи: осталось {} из {}", left, total);
        if w.caption != t {          // не обновлять зря: idle вызывается очень часто
            w.caption = t;
            w.refresh();
        }
    }
}
```

---

## Рецепт 4. Своё окно над моделью

**Задача.** Под таблицей — строка итогов «Книг: 5, прочитано: 3, на
сумму …», которая обновляется при каждом изменении.

`ModelWindow` такого не умеет, но его «кирпичики» доступны по
отдельности: `model_table` строит таблицу над моделью, а
`Model::revision()` подсказывает, когда пересчитать итоги.

```rust,ignore
pub struct BooksWindow {
    win: Window,
    table: ViewId,
    summary: ViewId,
    model: ModelRef,
    seen: u64,                       // revision, которую мы уже показали
}

impl BooksWindow {
    pub fn new(bounds: Rect, model: ModelRef) -> Self {
        let mut win = Window::new(bounds, "Книги", 1);
        let v = win.standard_scroll_bar(true);
        let h = win.standard_scroll_bar(false);
        let mut r = win.interior();
        r.b.y -= 1;                                          // последняя строка — для итогов
        let table = win.add(model_table(r, model.clone(), Some(v), Some(h)));
        let i = win.interior();
        let mut s = StaticText::new(Rect::new(i.a.x, i.b.y - 1, i.b.x, i.b.y), "");
        s.vs.grow_mode = gf::GROW_LO_Y | gf::GROW_HI_X | gf::GROW_HI_Y;   // прилипнуть к низу
        let summary = win.add(s);
        let mut w = BooksWindow { win, table, summary, model, seen: u64::MAX };
        w.sync();
        w
    }

    /// Модель изменилась — обновить итоги и таблицу.
    pub fn sync(&mut self) {
        let rev = self.model.borrow().revision();
        if rev == self.seen { return; }
        self.seen = rev;
        let s = stats(&self.model.borrow());
        let id = self.summary;
        if let Some(t) = self.win.group.find_mut::<StaticText>(id) {
            t.text = format!(" Книг: {}   прочитано: {}   на сумму: {:.2} ₽", s.count, s.read, s.total_price);
        }
        let id = self.table;
        self.win.group.find_mut::<Table>(id).unwrap().refresh();
    }
}
```

В `handle_event` окна `sync()` вызывается до и после обработки
события, а остальное (`draw`, `palette`, `set_state`, Enter →
команда) — как в [уроке 8](08-own-window.md).

**Про `grow_mode`.** `GROW_LO_Y | GROW_HI_Y` — «верхний и нижний края
двигаются вместе с нижним краем окна»: строка остаётся внизу, когда
окно растягивают. `GROW_HI_X` — растягиваться по ширине.

## Рецепт 5. Правка без контроллера

Без `CrudController` команды «Добавить», «Изменить», «Удалить» пишутся
сами — это три коротких метода. Диалог по схеме даёт `edit_record`.

```rust,ignore
fn edit(&mut self, app: &mut Program) {
    let Some(i) = self.win(app).and_then(|w| w.current()) else { return };
    // Копия схемы и записи — до диалога: пока он открыт, модель не заимствуем.
    let (schema, rec) = {
        let m = self.model.borrow();
        (m.schema().clone(), m.get(i).unwrap().clone())
    };
    if let Some(new) = edit_record(app, &schema, &rec, "Книга") {
        let r = self.model.borrow_mut().update(i, new);
        match r {
            Ok(j) => self.win(app).unwrap().select(j),
            Err(e) => { message_box(app, &e, mf::ERROR | mf::OK_BUTTON); }
        }
    }
}
```

`add` — то же с `schema.new_record()` и `model.add`, `delete` — вопрос
и `model.remove`. Полностью — в `examples/mvc_books.rs`.

Когда писать самому, а когда взять контроллер? Контроллер — когда
хватает трёх стандартных команд и окна-таблицы. Сами — когда окно своё
(как здесь) или правка нестандартная (например, «изменить» открывает
не диалог, а отдельное окно).

## Рецепт 6. Поиск по записям

**Задача.** Ctrl+F — спросить текст и перейти к первой книге, где он
есть в названии или авторе; F3 — к следующей.

Поиск — обычная функция над моделью, её легко проверить тестом:

```rust,ignore
/// Номер первой записи после `from`, где есть `what`; по кругу.
pub fn find(m: &Model, what: &str, from: usize) -> Option<usize> {
    let what = what.to_lowercase();
    let n = m.len();
    (1..=n).map(|k| (from + k) % n).find(|&i| {
        let r = m.get(i).unwrap();
        r.text(F_TITLE).to_lowercase().contains(&what) || r.text(F_AUTHOR).to_lowercase().contains(&what)
    })
}
```

Спросить текст — готовой функцией `input_box(app, заголовок, подпись,
начальный_текст, длина)`: она вернёт `Some(текст)` после OK.

---

## Рецепт 7. Своё хранилище

**Задача.** Хранить записи в файле CSV (его открывают Excel и
LibreOffice), а окно, модель и контроллер оставить как есть.

**Как.** Реализовать трейт `Storage` — пять методов. Удобный способ:
держать копию записей в памяти и после каждого изменения переписывать
файл целиком.

```rust,ignore
pub struct CsvStorage {
    path: PathBuf,
    records: Vec<Record>,
    next_id: i64,
}

impl Storage for CsvStorage {
    fn load(&mut self, schema: &Schema) -> Result<Vec<Record>, String> {
        // прочитать файл (нет файла — Ok(пусто)), найти столбцы по именам
        // из первой строки, каждое значение разобрать `field.parse(текст)`
        // — так проверка та же, что в диалоге
    }
    fn insert(&mut self, schema: &Schema, rec: &mut Record) -> Result<(), String> {
        rec.id = Some(self.next_id);
        self.records.push(rec.clone());
        if let Err(e) = self.save(schema) {      // не записалось —
            self.records.pop();                  // вернуть как было
            rec.id = None;
            return Err(e);
        }
        self.next_id += 1;
        Ok(())
    }
    fn update(&mut self, schema: &Schema, rec: &Record) -> Result<(), String> { /* заменить и save */ }
    fn delete(&mut self, schema: &Schema, id: i64) -> Result<(), String> { /* убрать и save */ }
    fn name(&self) -> String { /* имя файла для заголовка */ }
}

// И всё: дальше как с JSON.
let model = Model::open(schema(), Box::new(CsvStorage::new("склад.csv")))?.into_ref();
```

**Правило хорошего хранилища:** если сохранить не удалось, верните
ошибку и **оставьте всё как было** — модель тогда тоже ничего не
изменит, и пользователь увидит сообщение, а не потерянные данные.

Полный код с разбором кавычек CSV — `examples/mvc_csv.rs`.

---

## Рецепт 8. Запросы SQL без MVC

**Задача.** Выполнить любой запрос, который ввёл пользователь, и
показать результат — заранее неизвестно, какие будут столбцы.

`query_rows` возвращает и строки, и имена столбцов. Если столбцов нет,
это была команда (`INSERT`, `UPDATE`, `DELETE`), и `changes()` скажет,
сколько строк она изменила.

```rust
use tvision::sqlite::{Connection, Rows};
use tvision::VecTableModel;

pub enum Outcome {
    Table(Rows),
    Changed(usize),
}

pub fn run_sql(db: &Connection, sql: &str) -> Result<Outcome, String> {
    let rows = db.query_rows(sql, &[]).map_err(|e| e.to_string())?;
    if rows.columns.is_empty() {
        Ok(Outcome::Changed(db.changes()))
    } else {
        Ok(Outcome::Table(rows))
    }
}

/// Результат → модель для `Table` (NULL показываем точкой).
fn to_model(rows: &Rows) -> VecTableModel {
    let headers: Vec<&str> = rows.columns.iter().map(String::as_str).collect();
    let cells = rows.rows.iter().map(|r| r.iter().map(|v| if v.is_null() { "·".into() } else { v.to_text() }).collect()).collect();
    VecTableModel::new(&headers, cells)
}

fn sql_console() {
    let db = Connection::open_in_memory().unwrap();
    db.execute_batch("CREATE TABLE t (a INTEGER, b TEXT); INSERT INTO t VALUES (1, 'x'), (2, NULL);").unwrap();
    let Ok(Outcome::Table(r)) = run_sql(&db, "SELECT * FROM t") else { panic!() };
    let m = to_model(&r);
    assert_eq!((m.rows.len(), m.rows[1][1].as_str()), (2, "·"));
    assert!(matches!(run_sql(&db, "DELETE FROM t"), Ok(Outcome::Changed(2))));
}
```

Показать новую модель в таблице: `table.set_model(Box::new(модель))`,
затем `table.auto_width(200)` — ширина столбцов по содержимому первых
200 строк.

> Здесь запрос пишет сам пользователь — это консоль. В обычной
> программе данные **никогда** не вклеиваются в текст запроса: только
> `?` и параметры (см. [справочник](32-data.md#connection--база-sqlite)).

## Рецепт 9. Окно из строки ввода и таблицы

**Задача.** Сверху строка запроса, под ней таблица; Enter в строке —
выполнить.

```rust,ignore
let mut win = Window::new(bounds, "SQL", 1);
let i = win.interior();
let mut line = InputLine::new(Rect::new(i.a.x, i.a.y, i.b.x, i.a.y + 1), 1000);
line.vs.grow_mode = gf::GROW_HI_X;                          // растягиваться по ширине
let mut t = Table::new(Rect::new(i.a.x, i.a.y + 1, i.b.x, i.b.y), Box::new(VecTableModel::new(&[], vec![])), None, None);
t.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
let table = win.add(t);
let input = win.add(line);
win.select(input);                                         // фокус — строке ввода
```

Строка ввода не забирает Enter себе — он возвращается окну. Окно
проверяет, что фокус в строке, и посылает команду:

```rust,ignore
fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
    self.win.handle_window_event(ev, ctx);
    let in_input = self.win.group.current_view().map(|v| v.vs().id) == Some(self.input);
    if in_input && matches!(ev, Event::KeyDown(k) if k.key == Key::Enter && k.mods == Mods::NONE) {
        ctx.post_command(CM_RUN, Info::None);
        ev.clear();
    }
}
```

**Не привязывайте Enter в строке состояния.** Строка состояния видит
клавиши раньше всех — даже в диалогах, — и Enter перестал бы нажимать
кнопку OK. Для «Выполнить» в строке состояния взята F9.

---

## Рецепт 10. Настройки в JSON

**Задача.** Хранить настройки программы в файле: прочитать при
запуске, записать после изменения. Файл может быть старым (без новых
ключей), испорченным руками или содержать чужие ключи — со всем этим
нужно обойтись аккуратно.

```rust
use tvision::json::{parse_json, Json};

#[derive(Debug, PartialEq)]
pub struct Settings {
    pub user: String,
    pub tab_size: i64,
    pub recent: Vec<String>,
}

impl Settings {
    /// Нет ключа или он не того вида — значение по умолчанию.
    pub fn from_json(j: &Json) -> Settings {
        Settings {
            user: j.get("user").and_then(Json::as_str).unwrap_or("").to_string(),
            tab_size: j.get("tab_size").and_then(Json::as_i64).filter(|n| (1..=16).contains(n)).unwrap_or(4),
            recent: j
                .get("recent")
                .and_then(Json::as_array)
                .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                .unwrap_or_default(),
        }
    }

    /// Записать свои поля в объект; чужие ключи остаются.
    pub fn write_json(&self, into: &mut Json) {
        into.set("user", Json::from_str(&self.user));
        into.set("tab_size", Json::from_i64(self.tab_size));
        into.set("recent", Json::Arr(self.recent.iter().map(|s| Json::from_str(s)).collect()));
    }
}

fn settings_roundtrip() {
    let mut raw = parse_json(r#"{"plugin": {"on": true}, "tab_size": 99}"#).unwrap();
    let mut s = Settings::from_json(&raw);
    assert_eq!(s, Settings { user: String::new(), tab_size: 4, recent: vec![] });
    s.recent.push("a.txt".into());
    s.write_json(&mut raw);
    let text = raw.pretty(); // → в файл
    assert!(text.contains("\"plugin\"") && text.contains("\"a.txt\""));
}
```

Три правила, которые здесь соблюдены:

1. **Отсутствующий или неверный ключ — значение по умолчанию**, а не
   ошибка: старый файл настроек продолжит работать.
2. **Чужие ключи не теряются**: мы храним разобранный `Json` целиком и
   меняем в нём только свои поля (`set`), а не собираем объект заново.
3. **Испорченный файл — ошибка при запуске** (с номером строки из
   `JsonError`), а не молчаливая перезапись настройками по умолчанию.

Диалог настроек и список недавних файлов — в `examples/json_settings.rs`.

---

## Попробуйте

1. В `mvc_todo` добавьте команду «Просроченные»: перейти к первой
   несделанной задаче со сроком раньше сегодняшнего. Дата сегодня —
   `tvision::platform::time::now()`; сроки — строки `ГГГГ-ММ-ДД`, их
   можно сравнивать как строки.
2. В `mvc_books` добавьте в строку итогов среднюю цену.
3. Сделайте `TsvStorage` (значения через табуляцию) на основе
   `CsvStorage` и подключите его в `open_model` по расширению `.tsv`.
4. В `sqlite_query` сохраняйте историю запросов в JSON-файл через
   рецепт 10 и подставляйте предыдущий запрос по Ctrl+↑.

Справочник по всем типам — «[Данные: JSON, SQLite, MVC](32-data.md)».
Следующий урок — [раскладки](13-layouts.md): окна без единой
координаты, как в Qt.
