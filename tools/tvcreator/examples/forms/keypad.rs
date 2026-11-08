//! Форма «Код доступа» — сгенерировано TVCreator из «keypad.tvui».
//!
//! Не правьте этот файл вручную: изменения пропадут при следующей
//! генерации. Меняйте форму в TVCreator и генерируйте код заново.
//!
//! ```text
//! let (view, ui) = KeypadDialog::create();
//! let (cmd, view) = app.exec_dialog(view);
//! ```

#![allow(dead_code)]

use tvision::layout::*;
use tvision::*;

/// Команда кнопки «1».
pub const CM_KEY1: u16 = 120;
/// Команда кнопки «2».
pub const CM_KEY2: u16 = 121;
/// Команда кнопки «3».
pub const CM_KEY3: u16 = 122;
/// Команда кнопки «4».
pub const CM_KEY4: u16 = 123;
/// Команда кнопки «5».
pub const CM_KEY5: u16 = 124;
/// Команда кнопки «6».
pub const CM_KEY6: u16 = 125;
/// Команда кнопки «0».
pub const CM_KEY0: u16 = 126;

/// Элементы формы «Код доступа»: id для `find` / `find_mut`.
pub struct KeypadDialog {
    /// Строка ввода.
    pub code: ViewId,
    /// Кнопка «1».
    pub k1: ViewId,
    /// Кнопка «2».
    pub k2: ViewId,
    /// Кнопка «3».
    pub k3: ViewId,
    /// Кнопка «4».
    pub k4: ViewId,
    /// Кнопка «5».
    pub k5: ViewId,
    /// Кнопка «6».
    pub k6: ViewId,
    /// Кнопка «0».
    pub k0: ViewId,
    /// Кнопка «OK».
    pub enter: ViewId,
}

impl KeypadDialog {
    /// Создать диалог «Код доступа».
    pub fn create() -> (Dialog, KeypadDialog) {
        let mut d = Dialog::centered(30, 16, "Код доступа");
        let code = d.add(InputLine::new(Rect::default(), 8));
        let k1 = d.add(Button::new(Rect::default(), "1", CM_KEY1, bf::NORMAL));
        let k2 = d.add(Button::new(Rect::default(), "2", CM_KEY2, bf::NORMAL));
        let k3 = d.add(Button::new(Rect::default(), "3", CM_KEY3, bf::NORMAL));
        let k4 = d.add(Button::new(Rect::default(), "4", CM_KEY4, bf::NORMAL));
        let k5 = d.add(Button::new(Rect::default(), "5", CM_KEY5, bf::NORMAL));
        let k6 = d.add(Button::new(Rect::default(), "6", CM_KEY6, bf::NORMAL));
        let k0 = d.add(Button::new(Rect::default(), "0", CM_KEY0, bf::NORMAL));
        let enter = d.add(Button::new(Rect::default(), "O~K~", cm::OK, bf::DEFAULT));
        let grid1 = Grid::new()
            .margins(1, 1, 1, 0)
            .span(0, 0, 1, 3, code)
            .cell(1, 0, item(k1).expand())
            .cell(1, 1, item(k2).expand())
            .cell(1, 2, item(k3).expand())
            .cell(2, 0, item(k4).expand())
            .cell(2, 1, item(k5).expand())
            .cell(2, 2, item(k6).expand())
            .span(3, 0, 1, 2, item(k0).expand())
            .cell(3, 2, item(enter).expand());
        d.set_layout(grid1);
        d.adjust_size();
        d.select(code);
        (d, KeypadDialog { code, k1, k2, k3, k4, k5, k6, k0, enter })
    }
}
