# Справочник: настройки

Модуль `tvision::settings`: настройки программы в файле рядом с
исполнимым файлом и стандартный диалог «Настройки». Урок —
[16. Настройки программы](16-settings.md), пример —
`cargo run --example app_settings`.

На этой странице:
[`Settings`](#settings--настройки) ·
[файл](#файл) ·
[применить](#применить) ·
[окна](#окна) ·
[диалог](#диалог) ·
[темы](#темы) ·
[перечисления](#перечисления)

---

## `Settings` — настройки

| Поле | Что | По умолчанию |
|------|-----|--------------|
| `theme` | `""` — стандартные цвета, `"builtin:classic"` (`gruvbox`, `blackwhite`, `monochrome`) или путь к `.tvtheme` (относительный — от каталога программы) | `""` |
| `colors` | [`ColorChoice`](#перечисления) | `Auto` |
| `pattern` | узор рабочего стола | `'░'` |
| `language` | [`LanguageChoice`](#перечисления) | `System` |
| `windows_at_start` | [`WindowsAtStart`](#перечисления) | `Restore` |
| `tile_columns_first` | мозаика сначала по столбцам | `false` |
| `reverse_buttons` | поменять кнопки мыши | `false` |
| `double_click_ms` | интервал двойного щелчка | `400` |
| `windows` | `Vec<WindowPlace>` — где стояли окна | пусто |
| `extra` | свои настройки программы: `get(ключ)`, `set(ключ, Json)` | пусто |

```rust
use tvision::settings::{ColorChoice, Settings};

fn settings_text() {
    let s = Settings { colors: ColorChoice::Colors256, pattern: '▒', ..Settings::default() };
    let text = s.to_text();                // содержимое файла
    assert!(text.contains("\"colors\": \"256\""));
    assert_eq!(Settings::parse(&text).unwrap(), s);
    // Неизвестные значения — по умолчанию, файл не «ломается».
    let odd = Settings::parse(r#"{"colors": "много", "double_click_ms": 1}"#).unwrap();
    assert_eq!(odd.colors, ColorChoice::Auto);
    assert_eq!(odd.double_click_ms, 50);   // не меньше 50 мс
}
```

## Файл

| Функция | Что делает |
|---------|-----------|
| `Settings::default_path()` | `<имя программы>.settings.json` в каталоге исполнимого файла; или путь из `TVISION_SETTINGS` |
| `Settings::load(путь)` | прочитать; ошибка — текстом с путём |
| `Settings::load_or_default(путь)` | прочитать, а нет файла или он испорчен — по умолчанию |
| `Settings::load_or(путь, запасные)` | то же, но вместо умолчаний — свои (например, с русским языком) |
| `save(путь)` | записать (каталог создаётся) |
| `parse(текст)`, `to_text()`, `from_json`, `to_json` | текст файла |
| `exe_dir()` | каталог исполнимого файла |
| `exe_relative(путь)` | путь относительно каталога программы |

Константы: `SETTINGS_ENV` (`"TVISION_SETTINGS"`), `FORMAT_VERSION`,
`PATTERNS` (узоры, которые предлагает диалог).

## Применить

| Метод | Что делает |
|-------|-----------|
| `apply_language()` | `locale::set_language` по `language`; вызывать до создания меню |
| `apply(&mut app)` | язык, режим цвета, тема, узор, мышь, мозаика. Тема не читается — стандартные цвета и `Err(текст)` |
| `system_language()` | язык по `LC_ALL`, `LC_MESSAGES`, `LANG` |
| `standard_palette()` | стандартная палитра для текущего режима цвета |

## Окна

`WindowPlace { key, bounds, zoomed }` — где стояло окно. `bounds` — в
обычном виде: у распахнутого окна — те границы, к которым оно вернётся.

| Метод | Что делает |
|-------|-----------|
| `capture_windows(&app)` | запомнить окна рабочего стола; закрытые остаются с прежним местом |
| `restore_windows(&mut app)` | расставить по `windows_at_start`; границы подгоняются под рабочий стол |
| `place_window(&app, &mut окно)` | поставить окно на его место до `insert_window`; `true` — место нашлось |
| `window(ключ)` | запомненное место |
| `default_window_key(&Window)` | ключ по умолчанию: номер окна, а без номера — заголовок |
| `capture_windows_with`, `restore_windows_with`, `place_window_with` | то же со своим ключом: `Fn(&Window) -> Option<String>` |

Свои окна (структура с полем `Window`) должны отдавать окно через
`as_window` и `as_window_mut` — см. [«Своё окно»](23-program.md#своё-окно).
В каскаде и мозаике участвуют окна с флагом `of::TILEABLE`.

## Диалог

* `settings_dialog(app, &mut settings) -> bool` — темы из
  [`theme_dirs()`](#темы);
* `settings_dialog_with(app, &mut settings, &темы) -> bool` — свой
  список тем: `&[(значение для theme, Theme)]`.

Пока диалог открыт, тема, режим цвета и узор видны сразу. OK — настройки
записаны в `settings` и применены (`true`); Отмена — всё как было
(`false`); «По умолчанию» — стандартные значения (места окон и `extra`
не сбрасываются). Окна диалог не переставляет: порядок «при запуске»
действует со следующего запуска. Файл диалог не пишет.

Надписи диалога — через `locale::tr`, по-английски и по-русски.

## Темы

| Функция | Что |
|---------|-----|
| `theme_dirs()` | `themes/` рядом с программой и в текущем каталоге |
| `available_themes(&каталоги)` | темы из каталогов, затем встроенные, которых среди них нет; пути внутри каталога программы — относительные |
| `resolve_theme(значение)` | `Ok(None)` — стандартные цвета, `Ok(Some(тема))`, `Err` — не читается |

## Перечисления

| Тип | Значения (в файле) |
|-----|-------------------|
| `LanguageChoice` | `System` (`system`), `En` (`en`), `Ru` (`ru`); `language()` — какой язык получится |
| `ColorChoice` | `Auto` (`auto`), `Colors16` (`16`), `Colors256` (`256`) |
| `WindowsAtStart` | `Restore` (`restore`), `Cascade` (`cascade`), `Tile` (`tile`), `AsProgram` (`program`) |

У всех: `ALL`, `key()`, `from_key(текст)`.
