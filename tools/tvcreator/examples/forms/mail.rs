//! Форма «Почта» — сгенерировано TVCreator из «mail.tvui».
//!
//! Не правьте этот файл вручную: изменения пропадут при следующей
//! генерации. Меняйте форму в TVCreator и генерируйте код заново.
//!
//! ```text
//! let (view, ui) = MailWindow::create();
//! app.insert_window(view);
//! ```

#![allow(dead_code)]

use tvision::layout::*;
use tvision::*;

/// Команда кнопки «Ответить».
pub const CM_REPLY: u16 = 110;
/// Команда кнопки «Удалить».
pub const CM_DELETE: u16 = 111;

/// Элементы формы «Почта»: id для `find` / `find_mut`.
pub struct MailWindow {
    /// Список.
    pub folders: ViewId,
    /// Таблица.
    pub letters: ViewId,
    /// Текст «Выберите письмо.».
    pub preview: ViewId,
    /// Полоса выполнения.
    pub sync: ViewId,
    /// Кнопка «Ответить».
    pub reply: ViewId,
    /// Кнопка «Удалить».
    pub delete: ViewId,
}

impl MailWindow {
    /// Создать окно «Почта».
    pub fn create() -> (Window, MailWindow) {
        let mut w = Window::new(Rect::new(0, 0, 64, 20), "Почта", 0);
        let folders = w.add({
            let mut l = ListBox::new(Rect::default(), 1, None);
            l.set_items(vec!["Входящие".into(), "Отправленные".into(), "Черновики".into(), "Корзина".into()]);
            l
        });
        let letters = w.add(Table::new(Rect::default(), Box::new(VecTableModel::new(&["От", "Тема", "Дата"], vec![])), None, None));
        let preview = w.add(StaticText::new(Rect::default(), "Выберите письмо."));
        let sync = w.add(ProgressBar::new(Rect::default(), 100));
        let reply = w.add(Button::new(Rect::default(), "~О~тветить", CM_REPLY, bf::NORMAL));
        let delete = w.add(Button::new(Rect::default(), "~У~далить", CM_DELETE, bf::NORMAL));
        let vbox1 = VBox::new()
            .spacing(1)
            .add(item(letters).stretch(2))
            .add(item(preview).stretch(1).expand());
        let hbox1 = HBox::new()
            .add(item(folders).stretch(1).min_width(12))
            .add(item(vbox1).stretch(3));
        let hbox2 = HBox::new()
            .add(item(sync).align_v(Alignment::Center))
            .stretch(1)
            .add(reply)
            .add(delete);
        let vbox2 = VBox::new()
            .add(hbox1)
            .space(1)
            .add(hbox2);
        w.set_layout(vbox2);
        w.select(folders);
        (w, MailWindow { folders, letters, preview, sync, reply, delete })
    }
}
