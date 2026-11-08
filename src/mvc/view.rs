//! Представление: таблица над моделью, окно с ней и диалог записи.

use super::data::{FieldKind, Record, Schema};
use super::model::ModelRef;
use super::CM_RECORD_EDIT;
use crate::view::{gf, Ctx, DrawCtx, Value, View, ViewId};
use crate::widgets::*;
use crate::{cm, Event, Info, Key, Mods, Rect};

// ----------------------------------------------------------------------
// Таблица
// ----------------------------------------------------------------------

/// [`TableModel`] над [`ModelRef`]: таблица читает ячейки прямо из модели.
///
/// Щелчок по заголовку сортирует саму модель — поэтому номер строки
/// таблицы всегда совпадает с номером записи в модели.
pub struct ModelTableModel {
    pub model: ModelRef,
}

impl TableModel for ModelTableModel {
    fn row_count(&self) -> usize {
        self.model.borrow().len()
    }
    fn col_count(&self) -> usize {
        self.model.borrow().schema().fields.len()
    }
    fn cell(&self, row: usize, col: usize) -> String {
        let m = self.model.borrow();
        match (m.get(row), m.schema().fields.get(col)) {
            (Some(r), Some(f)) => f.display(r.get(col)),
            _ => String::new(),
        }
    }
    fn header(&self, col: usize) -> String {
        self.model.borrow().schema().fields.get(col).map(|f| f.plain_title()).unwrap_or_default()
    }
    fn sort(&mut self, col: usize, ascending: bool) -> bool {
        self.model.borrow_mut().sort_by(col, ascending);
        true
    }
}

/// Таблица над моделью: столбцы — поля схемы (ширина из [`Field::width`](super::Field::width)),
/// выбор строкой, растягивается вместе с окном.
pub fn model_table(bounds: Rect, model: ModelRef, v: Option<ViewId>, h: Option<ViewId>) -> Table {
    let cols: Vec<TableColumn> = model
        .borrow()
        .schema()
        .fields
        .iter()
        .map(|f| {
            let c = TableColumn::new(f.width);
            match f.kind {
                FieldKind::Bool => c.align(Align::Center),
                FieldKind::Int | FieldKind::Real => c.align(Align::Right),
                _ => c,
            }
        })
        .collect();
    let mut t = Table::new(bounds, Box::new(ModelTableModel { model }), v, h).with_columns(cols).selection(SelectionMode::Row).striped(true);
    t.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
    t
}

// ----------------------------------------------------------------------
// Окно
// ----------------------------------------------------------------------

/// Окно с таблицей записей — «V» в MVC.
///
/// Само ничего не меняет: Enter и двойной щелчок посылают команду
/// [`CM_RECORD_EDIT`], а правку делает контроллер. Если модель изменилась
/// (растёт `revision`), окно обновляет таблицу само.
pub struct ModelWindow {
    pub win: Window,
    pub table: ViewId,
    pub model: ModelRef,
    /// Заголовок без имени файла.
    pub caption: String,
    seen: u64,
}

impl ModelWindow {
    pub fn new(bounds: Rect, caption: &str, model: ModelRef) -> Self {
        let mut win = Window::new(bounds, "", 1);
        let v = win.standard_scroll_bar(true);
        let h = win.standard_scroll_bar(false);
        let table = win.add(model_table(win.interior(), model.clone(), Some(v), Some(h)));
        let seen = model.borrow().revision();
        let mut w = ModelWindow { win, table, model, caption: caption.to_string(), seen };
        w.update_title();
        w
    }

    fn update_title(&mut self) {
        let m = self.model.borrow();
        self.win.title = format!("{} — {} ({})", self.caption, m.storage_name(), m.len());
    }

    pub fn table(&self) -> &Table {
        self.win.group.find::<Table>(self.table).expect("таблица")
    }
    pub fn table_mut(&mut self) -> &mut Table {
        let id = self.table;
        self.win.group.find_mut::<Table>(id).expect("таблица")
    }

    /// Номер записи под курсором (`None` — записей нет).
    pub fn current(&self) -> Option<usize> {
        let (row, _) = self.table().cursor();
        (row < self.model.borrow().len()).then_some(row)
    }

    /// Поставить курсор на запись `i`.
    pub fn select(&mut self, i: usize) {
        self.table_mut().set_cursor(i, 0);
    }

    /// Обновить таблицу и заголовок после изменения модели.
    pub fn refresh(&mut self) {
        self.seen = self.model.borrow().revision();
        self.table_mut().refresh();
        self.update_title();
    }

    fn sync(&mut self) {
        if self.model.borrow().revision() != self.seen {
            self.refresh();
        }
    }
}

impl View for ModelWindow {
    crate::impl_group_basics!(win.group);

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
        self.sync();
        let double = matches!(ev, Event::MouseDown(m) if m.double);
        self.win.handle_window_event(ev, ctx);
        if matches!(ev, Event::KeyDown(k) if k.key == Key::Enter && k.mods == Mods::NONE) || double {
            ctx.post_command(CM_RECORD_EDIT, Info::None);
            ev.clear();
        }
        self.sync(); // заголовок сортировал модель
    }
}

// ----------------------------------------------------------------------
// Диалог записи
// ----------------------------------------------------------------------

/// Проверка строки ввода по полю схемы. Текст ошибки — от
/// [`Field::parse`](super::Field::parse), поэтому он точный.
struct FieldValidator {
    field: super::Field,
    last_error: std::cell::RefCell<String>,
}

impl Validator for FieldValidator {
    fn is_valid_input(&self, s: &mut String, _suppress_fill: bool) -> bool {
        match self.field.kind {
            FieldKind::Int => s.chars().all(|c| c.is_ascii_digit() || c == '-' || c == '+' || c == ' '),
            FieldKind::Real => s.chars().all(|c| c.is_ascii_digit() || "-+.,eE ".contains(c)),
            _ => true,
        }
    }
    fn is_valid(&self, s: &str) -> bool {
        match self.field.parse(s) {
            Ok(_) => true,
            Err(e) => {
                *self.last_error.borrow_mut() = e;
                false
            }
        }
    }
    fn error_message(&self) -> String {
        self.last_error.borrow().clone()
    }
}

/// Диалог правки записи, собранный по схеме:
///
/// | вид поля | элемент |
/// |----------|---------|
/// | `Text`, `Int`, `Real` | строка ввода с подписью и проверкой |
/// | `Bool` | флажок |
/// | `Choice` | переключатели |
///
/// Данные: [`record_value`] / [`record_from_value`]. Обычно вызывать не
/// нужно — это делает [`edit_record`].
pub fn record_dialog(schema: &Schema, rec: &Record, title: &str) -> Dialog {
    let label_w = schema.fields.iter().filter(|f| !matches!(f.kind, FieldKind::Bool)).map(|f| f.title_width()).max().unwrap_or(0) as i32 + 1;
    let x0 = 3 + label_w;
    let input_w = schema.fields.iter().map(|f| f.input_width()).max().unwrap_or(10);
    let width = (x0 + input_w + 3).max(36);
    let mut height = 3;
    for f in &schema.fields {
        height += match &f.kind {
            FieldKind::Choice(o) => o.len().max(1) as i32 + 1,
            _ => 2,
        };
    }
    height += 3;
    let mut d = Dialog::centered(width, height, title);
    let mut y = 2;
    let mut first = None;
    for f in &schema.fields {
        let id = match &f.kind {
            FieldKind::Bool => d.add(Cluster::check_boxes(Rect::new(x0, y, x0 + f.input_width(), y + 1), &[&f.title])),
            FieldKind::Choice(o) => {
                let items: Vec<&str> = o.iter().map(|s| s.as_str()).collect();
                let h = items.len().max(1) as i32;
                let id = d.add(Cluster::radio_buttons(Rect::new(x0, y, x0 + f.input_width(), y + h), &items));
                d.add(Label::new(Rect::new(2, y, x0 - 1, y + 1), &f.title, Some(id)));
                y += h - 1;
                id
            }
            _ => {
                let w = f.input_width().min(width - x0 - 3);
                let input = InputLine::new(Rect::new(x0, y, x0 + w, y + 1), f.max_len)
                    .with_validator(FieldValidator { field: f.clone(), last_error: Default::default() });
                let id = d.add(input);
                d.add(Label::new(Rect::new(2, y, x0 - 1, y + 1), &f.title, Some(id)));
                id
            }
        };
        first.get_or_insert(id);
        y += 2;
    }
    let bx = width / 2 - 13;
    d.add(Button::new(Rect::new(bx, y, bx + 12, y + 2), "O~K~", cm::OK, bf::DEFAULT));
    d.add(Button::new(Rect::new(bx + 14, y, bx + 26, y + 2), "Отмена", cm::CANCEL, bf::NORMAL));
    d.set_data(&record_value(schema, rec));
    if let Some(id) = first {
        d.select(id);
    }
    d
}

/// Данные диалога для записи: по значению на поле.
pub fn record_value(schema: &Schema, rec: &Record) -> Value {
    Value::List(
        schema
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let v = rec.get(i);
                match &f.kind {
                    FieldKind::Bool => Value::U32(v.as_bool() as u32),
                    FieldKind::Choice(o) => Value::U32(o.iter().position(|x| Some(x.as_str()) == v.as_str()).unwrap_or(0) as u32),
                    _ => Value::Str(v.to_text()),
                }
            })
            .collect(),
    )
}

/// Запись из данных диалога (id не задаётся). Ошибка — сообщение.
pub fn record_from_value(schema: &Schema, v: &Value) -> Result<Record, String> {
    let l = v.as_list().unwrap_or(&[]);
    let mut values = Vec::with_capacity(schema.fields.len());
    for (i, f) in schema.fields.iter().enumerate() {
        let x = l.get(i);
        let d = match &f.kind {
            FieldKind::Bool => super::Data::Bool(x.and_then(Value::as_u32).unwrap_or(0) & 1 != 0),
            FieldKind::Choice(o) => {
                let k = x.and_then(Value::as_u32).unwrap_or(0) as usize;
                super::Data::Text(o.get(k).cloned().unwrap_or_default())
            }
            _ => f.parse(x.and_then(Value::as_str).unwrap_or(""))?,
        };
        values.push(d);
    }
    Ok(Record::new(values))
}

/// Показать диалог записи. `Some(запись)` — нажали OK (id взят из `rec`).
pub fn edit_record(app: &mut crate::Program, schema: &Schema, rec: &Record, title: &str) -> Option<Record> {
    let (cmd, d) = app.exec_dialog(record_dialog(schema, rec, title));
    if cmd != cm::OK {
        return None;
    }
    match record_from_value(schema, &d.get_data()) {
        Ok(mut r) => {
            r.id = rec.id;
            Some(r)
        }
        Err(e) => {
            message_box(app, &e, mf::ERROR | mf::OK_BUTTON);
            None
        }
    }
}
