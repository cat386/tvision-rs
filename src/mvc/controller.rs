//! Контроллер: команды «Добавить», «Изменить», «Удалить».

use super::model::ModelRef;
use super::view::{edit_record, ModelWindow};
use super::{CM_RECORD_ADD, CM_RECORD_DELETE, CM_RECORD_EDIT};
use crate::view::ViewId;
use crate::widgets::{message_box, mf};
use crate::{cm, Event, Program, Rect};

/// Контроллер — «C» в MVC: связывает команды пользователя с моделью.
///
/// Вызовите [`handle`](Self::handle) из `AppHandler::handle_event` и
/// [`update_commands`](Self::update_commands) из `AppHandler::idle`.
/// Команды берутся, только когда окно контроллера активно, — поэтому
/// можно открыть несколько окон, у каждого свой контроллер.
pub struct CrudController {
    pub model: ModelRef,
    /// Окно ([`ModelWindow`]) на рабочем столе.
    pub window: ViewId,
    /// Заголовок диалога новой записи.
    pub add_title: String,
    /// Заголовок диалога правки.
    pub edit_title: String,
    /// Спрашивать «Удалить?».
    pub confirm_delete: bool,
}

impl CrudController {
    /// Контроллер для уже вставленного окна.
    pub fn new(model: ModelRef, window: ViewId) -> Self {
        CrudController { model, window, add_title: "Новая запись".into(), edit_title: "Запись".into(), confirm_delete: true }
    }

    /// Создать окно [`ModelWindow`] на рабочем столе и контроллер к нему.
    pub fn open(app: &mut Program, bounds: Rect, caption: &str, model: ModelRef) -> Self {
        let id = app.insert_window(ModelWindow::new(bounds, caption, model.clone()));
        CrudController::new(model, id)
    }

    /// Окно контроллера (если не закрыто).
    pub fn window<'a>(&self, app: &'a mut Program) -> Option<&'a mut ModelWindow> {
        app.find_mut::<ModelWindow>(self.window)
    }

    /// Окно открыто и активно (верхнее на рабочем столе).
    pub fn is_active(&self, app: &Program) -> bool {
        app.desktop().group.current_view().map(|v| v.vs().id) == Some(self.window)
    }

    /// Обработать команду. `true` — команда наша и выполнена.
    pub fn handle(&mut self, app: &mut Program, ev: &mut Event) -> bool {
        let Some(cmd) = ev.command() else { return false };
        if !matches!(cmd, CM_RECORD_ADD | CM_RECORD_EDIT | CM_RECORD_DELETE) || !self.is_active(app) {
            return false;
        }
        ev.clear();
        match cmd {
            CM_RECORD_ADD => self.add(app),
            CM_RECORD_EDIT => self.edit(app),
            _ => self.delete(app),
        }
        true
    }

    fn after_change(&self, app: &mut Program, row: Option<usize>) {
        if let Some(w) = self.window(app) {
            w.refresh();
            if let Some(i) = row {
                w.select(i);
            }
        }
    }

    fn error(app: &mut Program, e: &str) {
        message_box(app, e, mf::ERROR | mf::OK_BUTTON);
    }

    /// Новая запись через диалог.
    pub fn add(&mut self, app: &mut Program) {
        let (schema, blank) = {
            let m = self.model.borrow();
            (m.schema().clone(), m.schema().new_record())
        };
        let Some(rec) = edit_record(app, &schema, &blank, &self.add_title) else { return };
        let r = self.model.borrow_mut().add(rec);
        match r {
            Ok(i) => self.after_change(app, Some(i)),
            Err(e) => Self::error(app, &e),
        }
    }

    /// Изменить запись под курсором.
    pub fn edit(&mut self, app: &mut Program) {
        let Some(i) = self.window(app).and_then(|w| w.current()) else { return };
        // Копию берём до диалога: пока он открыт, модель не заимствуем.
        let (schema, rec) = {
            let m = self.model.borrow();
            (m.schema().clone(), m.get(i).cloned())
        };
        let Some(rec) = rec else { return };
        let Some(new) = edit_record(app, &schema, &rec, &self.edit_title) else { return };
        let r = self.model.borrow_mut().update(i, new);
        match r {
            Ok(j) => self.after_change(app, Some(j)),
            Err(e) => Self::error(app, &e),
        }
    }

    /// Удалить запись под курсором (с вопросом, если `confirm_delete`).
    pub fn delete(&mut self, app: &mut Program) {
        let Some(i) = self.window(app).and_then(|w| w.current()) else { return };
        if self.confirm_delete {
            let what = self.model.borrow().get(i).map(|r| r.text(0)).unwrap_or_default();
            let q = format!("\x03Удалить запись «{}»?", what);
            if message_box(app, &q, mf::CONFIRMATION | mf::YES_BUTTON | mf::NO_BUTTON) != cm::YES {
                return;
            }
        }
        let r = self.model.borrow_mut().remove(i);
        match r {
            Ok(_) => self.after_change(app, None),
            Err(e) => Self::error(app, &e),
        }
    }

    /// Разрешить или запретить команды записей по состоянию: окна нет —
    /// всё запрещено; записей нет — нельзя изменить и удалить. Для
    /// неактивного окна ничего не делает (решает активное).
    pub fn update_commands(&self, app: &mut Program) {
        let exists = app.find::<ModelWindow>(self.window).is_some();
        if exists && !self.is_active(app) {
            return;
        }
        let rows = exists && !self.model.borrow().is_empty();
        for (c, on) in [(CM_RECORD_ADD, exists), (CM_RECORD_EDIT, rows), (CM_RECORD_DELETE, rows)] {
            if on {
                app.enable_command(c);
            } else {
                app.disable_command(c);
            }
        }
    }
}
