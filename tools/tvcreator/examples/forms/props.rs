//! Форма «Свойства» — сгенерировано TVCreator из «props.tvui».
//!
//! Не правьте этот файл вручную: изменения пропадут при следующей
//! генерации. Меняйте форму в TVCreator и генерируйте код заново.
//!
//! ```text
//! let (view, ui) = PropsDialog::create();
//! let (cmd, view) = app.exec_dialog(view);
//! ```

#![allow(dead_code)]

use tvision::layout::*;
use tvision::*;

/// Элементы формы «Свойства»: id для `find` / `find_mut`.
pub struct PropsDialog {
    /// Подпись «Имя:».
    pub label_name: ViewId,
    /// Строка ввода.
    pub file_name: ViewId,
    /// Линия.
    pub line1: ViewId,
    /// Текст «Размер:».
    pub size_title: ViewId,
    /// Текст «12 КБ».
    pub size: ViewId,
    /// Линия.
    pub line2: ViewId,
    /// Текст «Изменён:».
    pub date_title: ViewId,
    /// Текст «сегодня, 12:00».
    pub date: ViewId,
    /// Линия.
    pub line3: ViewId,
    /// Кнопка «OK».
    pub ok: ViewId,
    /// Кнопка «Отмена».
    pub cancel: ViewId,
}

impl PropsDialog {
    /// Создать диалог «Свойства».
    pub fn create() -> (Dialog, PropsDialog) {
        let mut d = Dialog::centered(40, 12, "Свойства");
        let file_name = d.add(InputLine::new(Rect::default(), 30));
        let line1 = d.add(Line::horizontal(Rect::default()).to_frame());
        let size_title = d.add(StaticText::new(Rect::default(), "Размер:"));
        let size = d.add(StaticText::new(Rect::default(), "12 КБ"));
        let line2 = d.add(Line::vertical(Rect::default()));
        let date_title = d.add(StaticText::new(Rect::default(), "Изменён:"));
        let date = d.add(StaticText::new(Rect::default(), "сегодня, 12:00"));
        let line3 = d.add(Line::horizontal(Rect::default()).to_frame());
        let ok = d.add(Button::new(Rect::default(), "O~K~", cm::OK, bf::DEFAULT));
        let cancel = d.add(Button::new(Rect::default(), "Отмена", cm::CANCEL, bf::NORMAL));
        let label_name = d.add(Label::new(Rect::default(), "~И~мя:", Some(file_name)));
        let form1 = Form::new()
            .row(label_name, file_name);
        let vbox1 = VBox::new()
            .add(size_title)
            .add(size);
        let vbox2 = VBox::new()
            .add(date_title)
            .add(date);
        let hbox1 = HBox::new()
            .add(vbox1)
            .add(line2)
            .add(vbox2);
        let hbox2 = HBox::new()
            .stretch(1)
            .add(ok)
            .add(cancel);
        let vbox3 = VBox::new()
            .margins(2, 1, 2, 0)
            .add(form1)
            .add(line1)
            .add(hbox1)
            .add(line3)
            .add(hbox2);
        d.set_layout(vbox3);
        d.adjust_size();
        d.select(file_name);
        (d, PropsDialog { label_name, file_name, line1, size_title, size, line2, date_title, date, line3, ok, cancel })
    }
}
