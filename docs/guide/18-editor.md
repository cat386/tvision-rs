# Справочник: текстовый редактор

Модуль `widgets::editor`. Порт `TEditor`, `TMemo`, `TFileEditor`,
`TEditWindow`, `TIndicator`; текст хранится массивом строк, поэтому
Unicode работает полностью, а отмена многоуровневая.

* [`Editor`](#editor) · [`Pos`](#pos) · [`ef`, `ec`](#ef-и-ec)
* [`Memo`](#memo) · [`FileEditor`](#fileeditor) · [`EditWindow`](#editwindow) · [`Indicator`](#indicator)
* [`find_dialog`, `replace_dialog`, `display_width`, `absolute`](#функции-модуля)

---

## `Editor`

**Редактор текста** (`TEditor`): курсор, выделение, многоуровневая
отмена и повтор, поиск и замена, буфер обмена, автоотступ, мышь.

```rust
use tvision::*;

fn notes_window(app: &mut Program) -> ViewId {
    let mut w = Window::new(Rect::new(5, 2, 65, 18), "Заметки", 1);
    let h = w.standard_scroll_bar(false);
    let v = w.standard_scroll_bar(true);
    let mut ed = Editor::new(w.interior(), Some(h), Some(v), None);
    ed.set_text("Пишите здесь…");
    let id = w.add(ed);
    app.insert_window(w);
    id
}

fn editing() {
    let mut ed = Editor::new(Rect::new(0, 0, 40, 10), None, None, None);
    ed.set_text("первая строка\nвторая строка"); // сбрасывает отмену и «изменён»
    ed.set_select(Pos::new(1, 0), Pos::new(1, 6)); // выделить «вторая»
    assert_eq!(ed.selected_text(), "вторая");
    ed.insert_text("третья", false); // заменить выделение
    assert_eq!(ed.line(1), "третья строка");
    assert!(ed.modified);
    ed.undo(); // вставка поверх выделения отменяется одним шагом
    assert_eq!(ed.line(1), "вторая строка");
    ed.set_select(Pos::new(0, 0), Pos::new(0, 0));
    assert!(ed.search("строка", 0));
    assert_eq!(ed.selection(), Some((Pos::new(0, 7), Pos::new(0, 13))));
}
```

| Метод | Что |
|-------|-----|
| `Editor::new(bounds, h_scroll, v_scroll, indicator)` | редактор; полосы и индикатор — id соседей |
| `text()`, `set_text(s)` | весь текст (строки через `\n`); `set_text` сбрасывает отмену |
| `line_count()`, `line(i)` | строки |
| `cursor() -> Pos`, `selection()`, `selected_text()` | курсор и выделение |
| `set_select(from, to)` | выделить и поставить курсор в конец (`set_select(p, p)` — просто курсор) |
| `insert_text(s, select)` | вставить на место курсора, заменив выделение |
| `delete_select()` | удалить выделение |
| `undo()`, `redo()` | отмена, повтор |
| `search(what, flags) -> bool` | найти от курсора вперёд и выделить |
| `track_cursor(center)`, `scroll_to(x, y)` | прокрутка к курсору / в точку |
| `col_to_x(line, col)`, `x_to_col(line, x)` | номер символа ↔ экранный столбец (табуляции) |

| Поле | Что |
|------|-----|
| `modified`, `overwrite`, `auto_indent`, `can_undo` | флаги |
| `delta` | прокрутка (x — столбец, y — строка) |
| `find_str`, `replace_str`, `editor_flags` | строки и флаги поиска (`ef::*`) |
| `h_scroll`, `v_scroll`, `indicator` | id связанных представлений |
| `palette` | палитра (`palettes::EDITOR`; у `Memo` — `MEMO`) |

Редактор в фокусе сам обрабатывает и **разрешает** команды `cm::UNDO`,
`REDO`, `CUT`, `COPY`, `PASTE`, `CLEAR`, `SELECT_ALL`, `FIND`, `REPLACE`,
`SEARCH_AGAIN` — достаточно поставить их в меню, и пункты будут серыми,
когда действие невозможно. Клавиши — как в оригинале (WordStar:
Ctrl+K B, Ctrl+Q F…) и привычные (Shift+стрелки, Ctrl+Ins/Shift+Ins);
полная таблица — [глава 9](../09-editor.md#клавиши). Данные — `Value::Str`.

## `Pos`

**Позиция в тексте**: `line` и `col` (номер **символа** в строке, не
экранный столбец). `Pos::new(line, col)`; сравнивается (`<`, `==`) — так
удобно упорядочивать концы выделения.

## `ef` и `ec`

Флаги поиска и замены `ef`: `CASE_SENSITIVE`, `WHOLE_WORDS_ONLY`,
`PROMPT_ON_REPLACE`, `REPLACE_ALL`, `DO_REPLACE`, `BACKUP_FILES` (при
сохранении оставлять `.bak`).

`ec` — внутренние команды редактора (`CHAR_LEFT` … `DEL_WORD_LEFT`,
номера 500–525), как `cmCharLeft` в оригинале. Их можно послать
редактору, но обычно не нужно.

## `Memo`

**Многострочное поле в диалоге** (`TMemo`): Tab переходит к следующему
элементу, данные — весь текст (`Value::Str`).

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

`Memo::new(bounds, h_scroll, v_scroll)`; внутренний редактор — поле
`editor`.

## `FileEditor`

**Редактор файла** (`TFileEditor`): загрузка, сохранение, «Сохранить
как», вопрос о сохранении при закрытии.

| Метод / поле | Что |
|--------------|-----|
| `FileEditor::new(bounds, h, v, indicator, path)` | существующий файл загружается сразу |
| `load_file()`, `save_file()` | `true` — успех; иначе текст в `last_error` |
| `file_name` | путь; пустой — «Untitled» (`editor::UNTITLED`) |
| `title()` | имя для заголовка окна |
| `editor` | внутренний `Editor` |

Обрабатывает `cm::SAVE`, `cm::SAVE_AS` (диалог выбора имени). Перед
закрытием изменённого текста `valid` спрашивает «Сохранить?» (Да / Нет /
Отмена) через `ctx.exec` — это образец асинхронного вопроса из
представления.

## `EditWindow`

**Окно редактора файла** (`TEditWindow`): `FileEditor`, полосы прокрутки,
индикатор в нижней рамке, имя файла в заголовке.

```rust
use std::path::Path;
use tvision::*;

fn open_file(app: &mut Program, file: &Path, number: i16) -> ViewId {
    let r = app.desktop().group.vs.extent();
    app.insert_window(EditWindow::new(r, file, number)) // пустой путь — новый файл
}

fn text_of(app: &Program, id: ViewId) -> Option<String> {
    Some(app.find::<EditWindow>(id)?.editor().editor.text())
}
```

`EditWindow::new(bounds, path, number)`; `editor()`, `editor_mut()`
(`&FileEditor`), поля `win`, `editor_id`. Полный пример — `examples/tvedit.rs`.

## `Indicator`

**Индикатор `строка:столбец`** и признак изменения `*` в нижней рамке
(`TIndicator`). `Indicator::new(bounds)`; поля `location`, `modified`.
Редактор обновляет его сам сообщением `cm::INDICATOR_SET`, если получил
его id в конструкторе.

## Функции модуля

| Функция | Что |
|---------|-----|
| `editor::find_dialog(find, flags) -> Dialog` | диалог «Поиск»; данные `[текст, флажки]` |
| `editor::replace_dialog(find, replace, flags) -> Dialog` | диалог «Замена»; данные `[что, на что, флажки]` |
| `editor::display_width(s)` | ширина строки на экране с табуляциями |
| `editor::absolute(path)` | абсолютный путь без обращения к ФС (`fexpand`) |

`editor::TAB_SIZE` — ширина табуляции (8).
