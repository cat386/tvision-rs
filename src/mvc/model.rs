//! Модель: записи в памяти + хранилище, в которое уходит каждое изменение.

use std::cell::RefCell;
use std::rc::Rc;

use super::data::{Record, Schema};
use super::storage::Storage;

/// Общая ссылка на модель: её держат и окно (чтобы рисовать), и
/// контроллер (чтобы менять).
pub type ModelRef = Rc<RefCell<Model>>;

/// Модель — «М» в MVC: данные и правила работы с ними.
///
/// * держит записи в памяти (для быстрого показа в таблице);
/// * каждое изменение сначала проверяет по схеме, потом сохраняет в
///   хранилище и только при успехе меняет записи в памяти;
/// * помнит выбранную сортировку и соблюдает её после изменений;
/// * счётчик `revision()` растёт при каждом изменении — по нему
///   представления узнают, что пора обновиться.
///
/// ```
/// use tvision::mvc::*;
///
/// let schema = Schema::new("people", vec![Field::text("name", "Имя").required(), Field::int("age", "Возраст")]);
/// let mut m = Model::open(schema, Box::new(MemoryStorage::new())).unwrap();
/// m.add(Record::new(vec!["Борис".into(), Data::Int(40)])).unwrap();
/// m.add(Record::new(vec!["Анна".into(), Data::Null])).unwrap();
/// m.sort_by(0, true);
/// assert_eq!(m.get(0).unwrap().text(0), "Анна");
/// assert!(m.add(Record::new(vec!["".into(), Data::Null])).is_err()); // имя обязательно
/// ```
pub struct Model {
    schema: Schema,
    records: Vec<Record>,
    storage: Box<dyn Storage>,
    revision: u64,
    sort: Option<(usize, bool)>,
}

impl Model {
    /// Модель над хранилищем: сразу читает все записи.
    pub fn open(schema: Schema, mut storage: Box<dyn Storage>) -> Result<Model, String> {
        let records = storage.load(&schema)?;
        Ok(Model { schema, records, storage, revision: 0, sort: None })
    }

    /// Обернуть в [`ModelRef`].
    pub fn into_ref(self) -> ModelRef {
        Rc::new(RefCell::new(self))
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
    /// Запись по номеру строки.
    pub fn get(&self, i: usize) -> Option<&Record> {
        self.records.get(i)
    }
    pub fn records(&self) -> &[Record] {
        &self.records
    }
    /// Номер строки записи с данным id.
    pub fn index_of_id(&self, id: i64) -> Option<usize> {
        self.records.iter().position(|r| r.id == Some(id))
    }
    /// Растёт при каждом изменении.
    pub fn revision(&self) -> u64 {
        self.revision
    }
    /// Описание хранилища (имя файла).
    pub fn storage_name(&self) -> String {
        self.storage.name()
    }
    /// Текущая сортировка: (номер поля, по возрастанию).
    pub fn sort_state(&self) -> Option<(usize, bool)> {
        self.sort
    }

    fn changed(&mut self) {
        self.revision += 1;
    }

    fn resort(&mut self) {
        if let Some((col, asc)) = self.sort {
            self.records.sort_by(|a, b| {
                let o = a.get(col).compare(b.get(col));
                if asc {
                    o
                } else {
                    o.reverse()
                }
            });
        }
    }

    /// Добавить запись. Возвращает её номер строки (с учётом сортировки).
    pub fn add(&mut self, mut rec: Record) -> Result<usize, String> {
        self.schema.validate(&rec)?;
        rec.id = None;
        self.storage.insert(&self.schema, &mut rec)?;
        let id = rec.id;
        self.records.push(rec);
        self.resort();
        self.changed();
        Ok(self.records.iter().position(|r| r.id == id).unwrap_or(self.records.len() - 1))
    }

    /// Заменить запись в строке `i` (id сохраняется). Возвращает новый
    /// номер строки — после сортировки запись могла переместиться.
    pub fn update(&mut self, i: usize, mut rec: Record) -> Result<usize, String> {
        let old = self.records.get(i).ok_or("Нет такой записи.")?;
        self.schema.validate(&rec)?;
        rec.id = old.id;
        if *old == rec {
            return Ok(i); // ничего не изменилось
        }
        self.storage.update(&self.schema, &rec)?;
        let id = rec.id;
        self.records[i] = rec;
        self.resort();
        self.changed();
        Ok(self.records.iter().position(|r| r.id == id).unwrap_or(i))
    }

    /// Удалить запись в строке `i`.
    pub fn remove(&mut self, i: usize) -> Result<Record, String> {
        let id = self.records.get(i).ok_or("Нет такой записи.")?.id;
        if let Some(id) = id {
            self.storage.delete(&self.schema, id)?;
        }
        let r = self.records.remove(i);
        self.changed();
        Ok(r)
    }

    /// Перечитать записи из хранилища.
    pub fn reload(&mut self) -> Result<(), String> {
        self.records = self.storage.load(&self.schema)?;
        self.resort();
        self.changed();
        Ok(())
    }

    /// Сортировать по полю `col`; порядок сохраняется и после изменений.
    pub fn sort_by(&mut self, col: usize, ascending: bool) {
        if col < self.schema.fields.len() {
            self.sort = Some((col, ascending));
            self.resort();
            self.changed();
        }
    }

    /// Отменить сортировку (новые записи — в конец).
    pub fn unsort(&mut self) {
        self.sort = None;
    }
}
