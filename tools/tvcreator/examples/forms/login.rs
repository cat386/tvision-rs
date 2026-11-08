//! Форма «Вход» — сгенерировано TVCreator из «login.tvui».
//!
//! Не правьте этот файл вручную: изменения пропадут при следующей
//! генерации. Меняйте форму в TVCreator и генерируйте код заново.
//!
//! ```text
//! let (view, ui) = LoginDialog::create();
//! let (cmd, view) = app.exec_dialog(view);
//! ```

#![allow(dead_code)]

use tvision::layout::*;
use tvision::*;

/// Команда кнопки «Регистрация».
pub const CM_REGISTER: u16 = 100;

/// Элементы формы «Вход»: id для `find` / `find_mut`.
pub struct LoginDialog {
    /// Подпись «Имя:».
    pub label_user: ViewId,
    /// Строка ввода.
    pub user: ViewId,
    /// Подпись «Пароль:».
    pub label_pass: ViewId,
    /// Строка ввода.
    pub pass: ViewId,
    /// Подпись «Сервер:».
    pub label_server: ViewId,
    /// Переключатели.
    pub server: ViewId,
    /// Флажки.
    pub remember: ViewId,
    /// Кнопка «Регистрация».
    pub register: ViewId,
    /// Кнопка «OK».
    pub ok: ViewId,
    /// Кнопка «Отмена».
    pub cancel: ViewId,
}

impl LoginDialog {
    /// Создать диалог «Вход».
    pub fn create() -> (Dialog, LoginDialog) {
        let mut d = Dialog::centered(40, 12, "Вход");
        let user = d.add(InputLine::new(Rect::default(), 30));
        let pass = d.add(InputLine::new(Rect::default(), 30));
        let server = d.add(Cluster::radio_buttons(Rect::default(), &["~Р~абочий", "~Д~омашний"]));
        let remember = d.add(Cluster::check_boxes(Rect::default(), &["~З~апомнить пароль"]));
        let register = d.add(Button::new(Rect::default(), "~Р~егистрация", CM_REGISTER, bf::NORMAL));
        let ok = d.add(Button::new(Rect::default(), "O~K~", cm::OK, bf::DEFAULT));
        let cancel = d.add(Button::new(Rect::default(), "Отмена", cm::CANCEL, bf::NORMAL));
        let label_user = d.add(Label::new(Rect::default(), "~И~мя:", Some(user)));
        let label_pass = d.add(Label::new(Rect::default(), "~П~ароль:", Some(pass)));
        let label_server = d.add(Label::new(Rect::default(), "~С~ервер:", Some(server)));
        let form1 = Form::new()
            .row(label_user, user)
            .row(label_pass, pass)
            .row(label_server, server)
            .row_wide(remember);
        let hbox1 = HBox::new()
            .add(register)
            .stretch(1)
            .add(ok)
            .add(cancel);
        let vbox1 = VBox::new()
            .margins(2, 1, 2, 0)
            .add(form1)
            .space(1)
            .add(hbox1);
        d.set_layout(vbox1);
        d.adjust_size();
        d.select(user);
        (d, LoginDialog { label_user, user, label_pass, pass, label_server, server, remember, register, ok, cancel })
    }
}
