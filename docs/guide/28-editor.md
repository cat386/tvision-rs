# Справочник: текстовый редактор

Многострочный редактор текста: курсор, выделение, отмена, поиск и замена,
буфер обмена. Есть в трёх видах — сам редактор, поле для диалога и окно
для файла.

На этой странице:
[`EditWindow`](#editwindow--окно-для-файла) ·
[`Editor`](#editor--редактор) ·
[`Pos`](#pos--место-в-тексте) ·
[`Memo`](#memo--многострочное-поле-в-диалоге) ·
[`FileEditor`](#fileeditor--редактор-файла) ·
[`Indicator`](#indicator--строка-и-столбец) ·
[`ef`, `ec`](#ef-и-ec) ·
[функции](#функции-модуля)

Какой выбрать:

| Нужно | Берите |
|-------|--------|
| открыть и править файл в окне | [`EditWindow`](#editwindow--окно-для-файла) — всё готово |
| несколько строк текста в диалоге (комментарий, адрес) | [`Memo`](#memo--многострочное-поле-в-диалоге) |
| текст в своём окне, без файла | [`Editor`](#editor--редактор) |

---

## `EditWindow` — окно для файла

**Что это.** Готовое окно редактора файла: текст, полосы прокрутки,
строка и столбец курсора в нижней рамке, имя файла в заголовке. Само
сохраняет (F2 / `cm::SAVE`, «Сохранить как» — `cm::SAVE_AS`) и при
закрытии изменённого файла спрашивает «Сохранить?».

**Аналог:** `TEditWindow`.

```rust
use std::path::Path;
use tvision::*;

fn open_file(app: &mut Program, file: &Path) -> ViewId {
    let r = app.desktop().group.vs.extent(); // во весь рабочий стол
    app.insert_window(EditWindow::new(r, file, 1)) // пустой путь — новый файл
}

fn text_of(app: &Program, id: ViewId) -> Option<String> {
    Some(app.find::<EditWindow>(id)?.editor().editor.text())
}
```

`EditWindow::new(прямоугольник, путь, номер)`. Методы `editor()` и
`editor_mut()` — внутренний [`FileEditor`](#fileeditor--редактор-файла);
поля `win`, `editor_id`. Пример — `cargo run --example tvedit`.

## `Editor` — редактор

**Что это.** Сам редактор текста. Остальные — `Memo`, `FileEditor`,
`EditWindow` — построены на нём.

**Аналог:** `TEditor`.

```rust
use tvision::*;

fn notes_window(app: &mut Program) {
    let mut w = Window::new(Rect::new(5, 2, 65, 18), "Заметки", 1);
    let h = w.standard_scroll_bar(false);
    let v = w.standard_scroll_bar(true);
    let mut ed = Editor::new(w.interior(), Some(h), Some(v), None);
    ed.set_text("Пишите здесь…");
    w.add(ed);
    app.insert_window(w);
}

fn editing() {
    let mut ed = Editor::new(Rect::new(0, 0, 40, 10), None, None, None);
    ed.set_text("первая строка\nвторая строка");
    ed.set_select(Pos::new(1, 0), Pos::new(1, 6)); // выделить «вторая»
    assert_eq!(ed.selected_text(), "вторая");
    ed.insert_text("третья", false); // заменить выделенное
    assert_eq!(ed.line(1), "третья строка");
    ed.undo(); // отменить
    assert_eq!(ed.line(1), "вторая строка");
}
```

| Метод | Что делает |
|-------|-----------|
| `Editor::new(прямоугольник, горизонтальная, вертикальная, индикатор)` | редактор; полосы и индикатор — id (или `None`) |
| `text()`, `set_text(s)` | весь текст (строки через `\n`); `set_text` очищает историю отмены |
| `line_count()`, `line(i)` | число строк, строка |
| `cursor()` | где курсор ([`Pos`](#pos--место-в-тексте)) |
| `set_select(от, до)` | выделить (`set_select(p, p)` — просто поставить курсор) |
| `selected_text()`, `selection()` | выделенный текст и его границы |
| `insert_text(s, выделить)` | вставить на место курсора (выделенное заменится) |
| `undo()`, `redo()` | отменить, повторить |
| `search(что, флаги) -> bool` | найти от курсора и выделить |
| `modified` | изменён ли текст |

**Реже:** `delete_select()`, `track_cursor(по_центру)` (прокрутить к
курсору), `scroll_to(x, y)`, `col_to_x`, `x_to_col` (номер символа ↔
экранный столбец), поля `overwrite`, `auto_indent`, `can_undo`, `delta`,
`find_str`, `replace_str`, `editor_flags`, `h_scroll`, `v_scroll`,
`indicator`, `palette`.

**Что важно знать.**

* Редактор в фокусе сам выполняет **и разрешает** команды `cm::UNDO`,
  `REDO`, `CUT`, `COPY`, `PASTE`, `CLEAR`, `SELECT_ALL`, `FIND`,
  `REPLACE`, `SEARCH_AGAIN` — поставьте их в меню, и пункты будут серыми,
  когда действие невозможно.
* Отмена многоуровневая; вставка поверх выделения отменяется одним
  шагом.
* Клавиши — привычные (стрелки, Shift+стрелки, Ctrl+Ins/Shift+Ins,
  Ctrl+Z) и из Turbo Pascal (Ctrl+K B, Ctrl+Q F…). Полная таблица —
  [глава 9](../09-editor.md#клавиши).
* Данные для диалога — `Value::Str(весь текст)`.

## `Pos` — место в тексте

Строка `line` и номер символа в ней `col` (оба с нуля).
`Pos::new(строка, символ)`. Это **не экранные координаты**: табуляция
занимает несколько клеток, а считается одним символом.

## `Memo` — многострочное поле в диалоге

**Что это.** Редактор для диалога: Tab в нём переходит к следующему
полю (а не вставляет табуляцию), данные — весь текст.

**Аналог:** `TMemo`.

```rust
use tvision::*;

fn comment_dialog() -> Dialog {
    let mut d = Dialog::centered(50, 14, "Комментарий");
    let v = d.add(ScrollBar::new(Rect::new(46, 2, 47, 10)));
    let memo = d.add(Memo::new(Rect::new(3, 2, 46, 10), None, Some(v)));
    d.add(Label::new(Rect::new(2, 1, 12, 2), "~Т~екст", Some(memo)));
    d.add(Button::new(Rect::new(13, 11, 25, 13), "O~K~", cm::OK, bf::DEFAULT));
    d.set_data(&Value::List(vec![Value::Str("Первая строка\nвторая".into())]));
    d
}
```

`Memo::new(прямоугольник, горизонтальная, вертикальная)`; внутренний
редактор — поле `editor`.

## `FileEditor` — редактор файла

**Что это.** Редактор, привязанный к файлу: загружает, сохраняет,
спрашивает имя для «Сохранить как», а при закрытии изменённого текста —
«Сохранить?». Обычно его не создают сами, а берут готовым в
`EditWindow`.

**Аналог:** `TFileEditor`.

| Метод / поле | Что |
|--------------|-----|
| `FileEditor::new(прямоугольник, гор., верт., индикатор, путь)` | существующий файл загружается сразу |
| `load_file()`, `save_file()` | `true` — получилось, иначе текст ошибки в `last_error` |
| `file_name` | путь (пустой — «Без имени») |
| `title()` | имя для заголовка окна |
| `editor` | сам редактор |

## `Indicator` — строка и столбец

**Что это.** Маленький индикатор `12:5` (строка:столбец) и `*`, если
текст изменён, — обычно в нижней рамке окна редактора. `EditWindow`
создаёт его сам. `Indicator::new(прямоугольник)`, поля `location`,
`modified`; редактор обновляет его сам, если получил его id.

**Аналог:** `TIndicator`.

## `ef` и `ec`

**`ef`** — флаги поиска и замены: `CASE_SENSITIVE` (учитывать регистр),
`WHOLE_WORDS_ONLY` (только слова целиком), `PROMPT_ON_REPLACE`
(спрашивать перед каждой заменой), `REPLACE_ALL`, `DO_REPLACE`,
`BACKUP_FILES` (при сохранении оставлять копию `.bak`).

**`ec`** — внутренние команды редактора (`ec::CHAR_LEFT` и т. п.), как
в оригинале. Обычно не нужны.

## Функции модуля

* `editor::find_dialog(текст, флаги)`, `editor::replace_dialog(что,
  на_что, флаги)` — диалоги поиска и замены (редактор открывает их сам по
  `cm::FIND` и `cm::REPLACE`);
* `editor::display_width(s)` — ширина строки на экране с табуляциями;
* `editor::absolute(путь)` — полный путь;
* `editor::TAB_SIZE` — ширина табуляции (8), `editor::UNTITLED` — имя
  нового файла.
