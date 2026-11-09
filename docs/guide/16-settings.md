# Урок 16. Настройки программы

У большинства программ есть пункт «Настройки…»: цвета, язык, как
открывать окна. Выбор должен сохраняться, а при следующем запуске —
применяться сам. В `tvision-rs` для этого есть модуль
**`tvision::settings`**: готовый диалог и файл настроек.

```text
cargo run --example app_settings
```

## Что в диалоге

| Раздел | Что выбирает |
|--------|--------------|
| **Тема** | стандартные цвета, встроенные темы и файлы `.tvtheme` из каталога `themes/` рядом с программой; пока курсор ходит по списку, программа перекрашивается |
| **Цвета терминала** | определить сами, 16 или 256 цветов |
| **Узор фона** | `░`, `▒`, `▓` или без узора — тоже сразу видно |
| **Язык** | как в системе, English, русский |
| **Окна при запуске** | где были при выходе, каскадом, мозаикой, как откроет программа |
| **Параметры** | мозаика по столбцам, поменять кнопки мыши |
| **Двойной щелчок, мс** | интервал двойного щелчка |

**OK** — применить, **Отмена** — вернуть всё как было, **По умолчанию**
— заполнить стандартными значениями. Диалог помещается в обычный
терминал 80 × 25 и сам переводится на выбранный язык.

## Где файл

Рядом с исполнимым файлом: `D:\prog\editor.exe` →
`D:\prog\editor.settings.json`. Так настройки переезжают вместе с
программой (её можно носить на флешке). Путь даёт
`Settings::default_path()`; переменная окружения `TVISION_SETTINGS`
задаёт другой — например, для тестов.

Файл — JSON, его можно поправить руками:

```json
{
  "tvsettings": 1,
  "theme": "themes/sea.tvtheme",
  "language": "ru",
  "windows_at_start": "restore",
  "windows": [ {"key": "1", "bounds": [0, 0, 60, 18], "zoomed": false} ]
}
```

Чего в файле нет или что не читается — берётся по умолчанию; нет
самого файла — все настройки по умолчанию.

## Программа с настройками

Четыре места в программе:

```rust,ignore
use tvision::settings::{settings_dialog, Settings};

fn main() -> std::io::Result<()> {
    // 1. При запуске: прочитать и применить.
    let path = Settings::default_path();
    let mut settings = Settings::load_or_default(&path);
    settings.apply_language();          // до меню: надписи зависят от языка
    let mut app = Program::with_terminal(Some(menu()), Some(status()))?;
    if let Err(e) = settings.apply(&mut app) {   // тема, цвета, фон, мышь
        message_box(&mut app, &e, mf::WARNING | mf::OK_BUTTON);
    }
    open_windows(&mut app);
    settings.restore_windows(&mut app); // 2. окна — на свои места
    let mut h = App { settings, path };
    app.run(&mut h);
    // 3. При выходе: запомнить, где стоят окна.
    h.settings.capture_windows(&app);
    let _ = h.settings.save(&h.path);
    Ok(())
}

// 4. Пункт меню «Настройки…».
fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
    if ev.command() == Some(CM_SETTINGS) {
        ev.clear();
        if settings_dialog(app, &mut self.settings) {   // OK: уже применено
            if let Err(e) = self.settings.save(&self.path) {
                message_box(app, &e, mf::ERROR | mf::OK_BUTTON);
            }
        }
    }
}
```

`settings_dialog` файл не пишет — сохраняете вы, когда удобно (сразу,
как здесь, или при выходе).

## Окна на своих местах

Окно узнаётся по **ключу**: по умолчанию это номер окна (`"1"`, `"2"`…),
а у окон без номера — заголовок. Запоминаются границы и то, было ли окно
распахнуто (F5); у распахнутого — и место, куда оно вернётся.

* `capture_windows(&app)` — запомнить все окна рабочего стола; окна,
  которые уже закрыты, остаются в списке с прежним местом.
* `restore_windows(&mut app)` — расставить по выбору в диалоге: на
  прежние места, каскадом, мозаикой или не трогать. Если терминал стал
  меньше, окна подгоняются под рабочий стол.
* `place_window(&app, &mut окно)` — окно, которое открывается позже
  (например, «Файл → Открыть»), — на своё место, до `insert_window`.

Ключ можно выбрать свой — например, имя файла у окон редактора:

```rust,ignore
settings.capture_windows_with(&app, |w| Some(w.title.clone()));
settings.restore_windows_with(&mut app, |w| Some(w.title.clone()));
```

Чтобы окно участвовало в каскаде и мозаике, у него должен быть флаг
`of::TILEABLE`: `w.group.vs.options |= of::TILEABLE;`.

## Язык на ходу

Надписи библиотеки (кнопки окон сообщений, диалоги файлов, сам диалог
настроек) переключаются сразу. Свои надписи программа выбирает по
`locale::language()`; после смены языка их собирают заново — пример
`app_settings` перестраивает меню, строку состояния и заголовки окон:

```rust,ignore
if let Some(m) = app.find_mut::<MenuBar>(app.menu_bar_id.unwrap()) {
    m.menu = Rc::new(menu());           // menu() смотрит на locale::language()
}
if let Some(s) = app.status_line_mut() {
    s.defs = status();
}
```

## Свои настройки

Всё, что программа хочет помнить сама (последний файл, размер шрифта),
хранится в том же файле:

```rust
use tvision::json::Json;
use tvision::settings::Settings;

fn own_settings() {
    let mut s = Settings::default();
    s.set("last_file", Json::from_str("notes.txt"));
    assert_eq!(s.get("last_file").and_then(Json::as_str), Some("notes.txt"));
    let back = Settings::parse(&s.to_text()).unwrap();
    assert_eq!(back, s);
}
```

## Попробуйте

1. Запустите `cargo run --example app_settings`, нажмите F9, выберите
   тему «Матрица» и русский язык. Распахните окно (F5), выйдите (Alt-X) и
   запустите снова.
2. Откройте файл `target/debug/examples/app_settings.settings.json` и
   поменяйте `"windows_at_start"` на `"tile"`.
3. Добавьте в программу из урока 9 пункт «Настройки…» и запоминание окон.

## Важно запомнить

* `Settings::default_path()` — файл рядом с программой;
  `load_or_default` — прочитать, `save` — записать.
* При запуске: `apply_language()` до меню, `apply(&mut app)` после
  создания программы, `restore_windows` после открытия окон.
* При выходе: `capture_windows(&app)` и `save`.
* `settings_dialog(app, &mut settings)` — стандартный диалог; `true` —
  выбрано OK и уже применено.

Справочник — «[Настройки](35-settings.md)».
