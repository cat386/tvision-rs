//! Язык встроенных надписей библиотеки (кнопки окон сообщений, диалоги
//! файлов и смены каталога, справка, сообщения валидаторов, поиск в
//! редакторе).
//!
//! По умолчанию — английский, как в оригинальном Turbo Vision. Программа
//! может включить русский одной строкой в начале `main`:
//!
//! ```ignore
//! tvision::locale::set_language(tvision::locale::Language::Ru);
//! ```
//!
//! Внутри библиотеки все такие строки проходят через [`tr`]: она получает
//! английский текст и возвращает перевод для текущего языка (или сам текст,
//! если перевода нет). Константы вроде `MsgBoxText::OK` остались прежними —
//! это английские «ключи» перевода.

use std::cell::Cell;

/// Язык надписей.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    En,
    Ru,
}

thread_local! {
    static LANG: Cell<Language> = const { Cell::new(Language::En) };
}

/// Выбрать язык надписей библиотеки.
pub fn set_language(l: Language) {
    LANG.with(|c| c.set(l));
}

/// Текущий язык.
pub fn language() -> Language {
    LANG.with(|c| c.get())
}

/// Русские переводы: (английский текст, перевод). Горячие буквы `~X~`
/// подобраны так, чтобы не совпадать внутри одного диалога.
const RU: &[(&str, &str)] = &[
    // Окна сообщений.
    ("~Y~es", "~Д~а"),
    ("~N~o", "~Н~ет"),
    ("O~K~", "O~K~"),
    ("Cancel", "Отмена"),
    ("Warning", "Предупреждение"),
    ("Error", "Ошибка"),
    ("Information", "Информация"),
    ("Confirm", "Подтверждение"),
    // Диалог файлов.
    ("~F~iles", "~Ф~айлы"),
    ("~O~pen", "~О~ткрыть"),
    ("~R~eplace", "~З~аменить"),
    ("~C~lear", "О~ч~истить"),
    ("~H~elp", "~С~правка"),
    ("Help", "Справка"),
    ("Invalid drive or directory", "Неверный диск или каталог"),
    ("Invalid file name", "Неверное имя файла"),
    ("~N~ame", "~И~мя"),
    // Смена каталога.
    ("Change Directory", "Смена каталога"),
    ("Directory ~n~ame", "~И~мя каталога"),
    ("Directory ~t~ree", "~Д~ерево каталогов"),
    ("~C~hdir", "~П~ерейти"),
    ("~R~evert", "~В~ернуть"),
    ("Invalid directory:", "Неверный каталог:"),
    // Справка.
    ("\n No help available in this context.", "\n Справка для этого места отсутствует."),
    // Валидаторы.
    ("Input does not conform to picture:", "Ввод не соответствует шаблону:"),
    ("Invalid character in input", "Недопустимый символ"),
    ("Value not in the range", "Значение вне диапазона"),
    ("to", "до"),
    ("Input is not in list of valid strings", "Такого значения нет в списке допустимых"),
    ("Value not within range", "Значение вне диапазона"),
    // Цвета.
    ("Colors", "Цвета"),
    ("~G~roup", "~Г~руппа"),
    ("~I~tem", "~Э~лемент"),
    ("~F~oreground", "~Т~екст"),
    ("~B~ackground", "~Ф~он"),
    ("Text ", "Текст "),
    // Список.
    ("<empty>", "<пусто>"),
    // Редактор.
    ("Untitled", "Без имени"),
    ("\x03Search string not found.", "\x03Строка не найдена."),
    ("\x03Replace this occurrence?", "\x03Заменить это вхождение?"),
    ("\x03Save untitled file?", "\x03Сохранить файл без имени?"),
    ("has been modified. Save?", "изменён. Сохранить?"),
    ("Error reading file", "Ошибка чтения файла"),
    ("Error writing file", "Ошибка записи файла"),
    ("Save file as", "Сохранить файл как"),
    ("Find", "Поиск"),
    ("~T~ext to find", "~И~скать"),
    ("~C~ase sensitive", "~У~читывать регистр"),
    ("~W~hole words only", "~Т~олько слова целиком"),
    ("Replace", "Замена"),
    ("~N~ew text", "~Н~овый текст"),
    ("~P~rompt on replace", "~С~прашивать при замене"),
    ("~R~eplace all", "Заменить ~в~се"),
];

/// Перевести встроенную надпись на текущий язык.
pub fn tr(s: &'static str) -> &'static str {
    match language() {
        Language::En => s,
        Language::Ru => RU.iter().find(|(e, _)| *e == s).map(|(_, r)| *r).unwrap_or(s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switch() {
        assert_eq!(tr("Cancel"), "Cancel");
        set_language(Language::Ru);
        assert_eq!(tr("Cancel"), "Отмена");
        assert_eq!(tr("нет перевода"), "нет перевода");
        set_language(Language::En);
        assert_eq!(tr("Cancel"), "Cancel");
    }
}
