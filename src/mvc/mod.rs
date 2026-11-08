//! Model–View–Controller: готовый каркас «таблица записей + диалог правки»
//! над файлом JSON или базой SQLite.
//!
//! * **Модель** ([`Model`]) — записи и правила: проверка по схеме,
//!   сохранение в хранилище ([`Storage`]: [`JsonStorage`],
//!   `SqliteStorage`, [`MemoryStorage`]), сортировка.
//! * **Представление** ([`ModelWindow`]) — окно с таблицей; только
//!   показывает модель и посылает команды.
//! * **Контроллер** ([`CrudController`]) — принимает команды «Добавить»,
//!   «Изменить», «Удалить», показывает диалог ([`record_dialog`]) и
//!   меняет модель.
//!
//! Чтобы получить программу, достаточно описать поля:
//!
//! ```
//! use tvision::mvc::*;
//! use tvision::*;
//!
//! fn schema() -> Schema {
//!     Schema::new("contacts", vec![
//!         Field::text("name", "~И~мя").required(),
//!         Field::text("phone", "~Т~елефон"),
//!         Field::bool("favorite", "Из~б~ранный"),
//!     ])
//! }
//!
//! struct App {
//!     crud: CrudController,
//! }
//!
//! impl AppHandler for App {
//!     fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
//!         self.crud.handle(app, ev); // Ins, Enter, Del — всё здесь
//!     }
//!     fn idle(&mut self, app: &mut Program) {
//!         self.crud.update_commands(app); // «Изменить» без записей — серая
//!     }
//! }
//!
//! fn start(app: &mut Program, file: &str) -> Result<App, String> {
//!     let model = Model::open(schema(), open_storage(file)?)?.into_ref();
//!     let r = app.desktop().group.vs.extent();
//!     Ok(App { crud: CrudController::open(app, r, "Контакты", model) })
//! }
//! # let _ = start;
//! ```
//!
//! Пример целиком — `cargo run --example mvc_contacts`, урок —
//! `docs/guide/11-mvc.md`.

mod controller;
mod data;
mod model;
mod storage;
mod view;

pub use controller::CrudController;
pub use data::{Data, Field, FieldCheck, FieldKind, Record, Schema};
pub use model::{Model, ModelRef};
#[cfg(feature = "sqlite")]
pub use storage::SqliteStorage;
pub use storage::{open_storage, JsonStorage, MemoryStorage, Storage};
pub use view::{edit_record, model_table, record_dialog, record_from_value, record_value, ModelTableModel, ModelWindow};

/// Команда «добавить запись» (обычно — Ins).
pub const CM_RECORD_ADD: u16 = 180;
/// Команда «изменить запись» (Enter, двойной щелчок, F4).
pub const CM_RECORD_EDIT: u16 = 181;
/// Команда «удалить запись» (Del).
pub const CM_RECORD_DELETE: u16 = 182;

#[cfg(test)]
mod tests;
