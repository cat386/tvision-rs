//! Меню и строка состояния среды.

use super::*;

/// Меню с темами по умолчанию (встроенными).
pub fn menu() -> Menu {
    menu_with(&load_themes(&[]))
}

/// Меню «Тема»: тема среды и тема формы — по пункту на тему.
fn themes_menu(themes: &[ThemeEntry]) -> Menu {
    let mut ide = Menu::new();
    let mut form = Menu::new();
    for (i, e) in themes.iter().enumerate().take(MENU_THEMES) {
        ide = ide.item(&e.theme.name, CM_IDE_THEME + i as u16);
        form = form.item(&e.theme.name, CM_FORM_THEME + i as u16);
    }
    ide = ide
        .separator()
        .item_key("~В~ыбрать с предпросмотром...", CM_THEME_CHOOSE, kb::ctrl('t'), "Ctrl-T")
        .item("~О~ткрыть файл темы...", CM_THEME_OPEN)
        .item("По ~у~молчанию", CM_THEME_DEFAULT);
    form = form.separator().item("~В~ыбрать с предпросмотром...", CM_FORM_THEME_CHOOSE).item("~К~ак у среды", CM_FORM_THEME_SAME);
    Menu::new().submenu("~С~реда", ide).submenu("~Ф~орма на холсте", form)
}

pub fn menu_with(themes: &[ThemeEntry]) -> Menu {
    Menu::new()
        .submenu(
            "~Ф~айл",
            Menu::new()
                .item_key("~Н~овая форма", CM_NEW, kb::ctrl('n'), "Ctrl-N")
                .item_key("~О~ткрыть...", CM_OPEN, kb::F3, "F3")
                .item_key("~С~охранить", CM_SAVE, kb::F2, "F2")
                .item("Сохранить ~к~ак...", CM_SAVE_AS)
                .separator()
                .item_key("Сгенерировать ~к~од", CM_GENERATE, kb::f(9), "F9")
                .separator()
                .item("Нас~т~ройки...", CM_SETTINGS)
                .separator()
                .item_key("~В~ыход", cm::QUIT, kb::ALT_X, "Alt-X"),
        )
        .submenu(
            "~П~равка",
            Menu::new()
                .item_key("~О~тменить", CM_UNDO, kb::ctrl('z'), "Ctrl-Z")
                .item_key("~П~овторить", CM_REDO, kb::ctrl('y'), "Ctrl-Y")
                .separator()
                .item_key("~Д~обавить из палитры", CM_ADD, kb::key(Key::Ins), "Ins")
                .item_key("~У~далить", CM_DELETE, kb::key(Key::Del), "Del")
                .item_key("~В~ыше", CM_UP, KeyEvent::new(Key::Up, Mods::CTRL), "Ctrl-↑")
                .item_key("~Н~иже", CM_DOWN, KeyEvent::new(Key::Down, Mods::CTRL), "Ctrl-↓")
                .item_key("~С~войство...", CM_PROP_EDIT, kb::f(4), "F4"),
        )
        .submenu(
            "~Р~аскладка",
            Menu::new()
                .item("Обернуть в ~с~троку (HBox)", CM_WRAP_H)
                .item("Обернуть в с~т~олбец (VBox)", CM_WRAP_V)
                .item("Обернуть в се~т~ку (Grid)", CM_WRAP_GRID)
                .item("Обернуть в ~ф~орму (Form)", CM_WRAP_FORM),
        )
        .submenu(
            "Ф~о~рма",
            Menu::new()
                .item_key("~П~редпросмотр", CM_PREVIEW, kb::ctrl('r'), "Ctrl-R")
                .item_key("Показать ~к~од", CM_CODE, kb::f(7), "F7")
                .item("П~р~оверить", CM_CHECK)
                .separator()
                .item("О ~п~рограмме", CM_ABOUT),
        )
        .submenu("~Т~ема", themes_menu(themes))
        .submenu("~О~кна", Menu::new().item_key("~С~ледующее", cm::NEXT, kb::F6, "F6").item_key("~З~акрыть", cm::CLOSE, kb::ALT_F3, "Alt-F3"))
}

/// Строка состояния — только F-клавиши: она видит клавиши даже в
/// диалогах, а Ins/Del там нужны строкам ввода.
pub fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![
        StatusItem::new("~F2~ Сохранить", kb::F2, CM_SAVE),
        StatusItem::new("~F3~ Открыть", kb::F3, CM_OPEN),
        StatusItem::new("~F4~ Свойство", kb::f(4), CM_PROP_EDIT),
        StatusItem::new("~F7~ Код", kb::f(7), CM_CODE),
        StatusItem::new("~F9~ Генерировать", kb::f(9), CM_GENERATE),
        StatusItem::new("~Ctrl-R~ Предпросмотр", kb::ctrl('r'), CM_PREVIEW),
        StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT),
        StatusItem::new("", kb::F10, cm::MENU),
    ])]
}
