# 9. Текстовый редактор

Редактор — порт `TEditor`, `TMemo`, `TFileEditor`, `TEditWindow` и
`TIndicator`. Внутри текст хранится не «буфером с разрывом», как в
оригинале, а массивом строк из символов, поэтому:

* полностью поддерживается Unicode (кириллица, широкие символы);
* отмена **многоуровневая** (в оригинале — только последняя правка), есть
  повтор;
* размер текста ограничен только памятью.

## Типы

| Тип | Для чего |
|-----|----------|
| `Editor` | сам редактор: текст, курсор, выделение, отмена, поиск |
| `Memo` | многострочное поле в диалоге: Tab переходит к следующему элементу, данные — весь текст |
| `FileEditor` | редактор файла: загрузка, сохранение, «Сохранить как», вопрос о сохранении при закрытии |
| `EditWindow` | окно с `FileEditor`, полосами прокрутки и индикатором строки:столбца |
| `Indicator` | индикатор `строка:столбец` и признак изменения в нижней рамке |

## Окно редактора файла

Проще всего открыть файл в готовом окне:

```rust
use std::path::Path;
use tvision::*;

fn open_file(app: &mut Program, file: &Path, number: i16) {
    let r = app.desktop().group.vs.extent(); // на весь рабочий стол
    let w = EditWindow::new(r, file, number); // пустой путь — новый файл «Untitled»
    app.insert_window(w);
}
```

`EditWindow` сам:

* показывает имя файла в заголовке (и обновляет его после «Сохранить как»);
* обрабатывает `cm::SAVE` и `cm::SAVE_AS` (`FileDialog` для нового имени);
* при закрытии изменённого файла спрашивает «Сохранить?» (Да / Нет / Отмена);
* держит индикатор строки:столбца в нижней рамке.

Доступ к редактору: `w.editor()`, `w.editor_mut()` (`FileEditor`), поле
`editor_id`.

## Клавиши

Раскладка — как в оригинале (WordStar):

| Клавиши | Действие |
|---------|----------|
| ←/→, Ctrl+S/Ctrl+D | символ влево/вправо |
| Ctrl+←/→, Ctrl+A/Ctrl+F | слово влево/вправо |
| ↑/↓, Ctrl+E/Ctrl+X | строка вверх/вниз |
| Home/End | начало/конец строки |
| PgUp/PgDn, Ctrl+R/Ctrl+C | страница вверх/вниз |
| Ctrl+Home/Ctrl+End | начало/конец текста |
| Shift + движение | выделение |
| Enter | новая строка (с автоотступом, если включён) |
| Backspace, Ctrl+H | удалить символ слева |
| Del, Ctrl+G | удалить символ справа |
| Ctrl+Backspace | удалить слово слева |
| Ctrl+T | удалить слово справа |
| Ctrl+Y | удалить строку |
| Ctrl+Q H / Ctrl+Q Y | удалить до начала / до конца строки |
| Ins, Ctrl+V | вставка/замена |
| Ctrl+O | автоотступ вкл/выкл |
| Ctrl+U, Ctrl+Z | отменить |
| Ctrl+Ins, Ctrl+K K | копировать |
| Shift+Del, Ctrl+K Y | вырезать |
| Shift+Ins, Ctrl+K C | вставить |
| Ctrl+Del | удалить выделенное |
| Ctrl+K B / Ctrl+K H | начать выделение / снять выделение |
| Ctrl+Q F | найти |
| Ctrl+Q A | заменить |
| Ctrl+L | искать дальше |
| Ctrl+Q S/D/R/C | начало строки / конец строки / начало текста / конец текста |

Мышь: щелчок ставит курсор, протяжка выделяет (с автопрокруткой),
двойной щелчок выделяет слово, колесо прокручивает.

## Команды

Редактор в фокусе обрабатывает стандартные команды — их достаточно
поставить в меню:

| Команда | Действие |
|---------|----------|
| `cm::UNDO`, `cm::REDO` | отменить / повторить |
| `cm::CUT`, `cm::COPY`, `cm::PASTE`, `cm::CLEAR` | буфер обмена (системный — [глава 12](12-platform.md#буфер-обмена)) |
| `cm::SELECT_ALL` | выделить всё |
| `cm::FIND`, `cm::REPLACE`, `cm::SEARCH_AGAIN` | поиск и замена |
| `cm::SAVE`, `cm::SAVE_AS` | сохранение (`FileEditor`) |

Редактор сам **разрешает** эти команды, когда он в фокусе и действие
возможно (есть что отменить, есть выделение), и запрещает в остальное
время, поэтому пункты меню автоматически становятся серыми.

```rust
use tvision::*;

fn edit_menu() -> Menu {
    Menu::new()
        .item_key("~О~тменить", cm::UNDO, kb::ctrl('Z'), "Ctrl-Z")
        .separator()
        .item_key("~В~ырезать", cm::CUT, KeyEvent::new(Key::Del, Mods::SHIFT), "Shift-Del")
        .item_key("~К~опировать", cm::COPY, KeyEvent::new(Key::Ins, Mods::CTRL), "Ctrl-Ins")
        .item_key("Вст~а~вить", cm::PASTE, KeyEvent::new(Key::Ins, Mods::SHIFT), "Shift-Ins")
        .separator()
        .item("~Н~айти...", cm::FIND)
        .item("~З~аменить...", cm::REPLACE)
        .item_key("Искать ~д~альше", cm::SEARCH_AGAIN, kb::ctrl('L'), "Ctrl-L")
}
```

## Поиск и замена

`cm::FIND` и `cm::REPLACE` открывают диалоги «Поиск» / «Замена» (функции
`find_dialog` и `replace_dialog` — их можно вызвать и самому). Строки
поиска и флаги запоминаются в полях редактора `find_str`, `replace_str`,
`editor_flags`:

| Флаг `ef` | Смысл |
|-----------|-------|
| `CASE_SENSITIVE` | учитывать регистр |
| `WHOLE_WORDS_ONLY` | только слова целиком |
| `PROMPT_ON_REPLACE` | спрашивать перед каждой заменой |
| `REPLACE_ALL` | заменить все |
| `BACKUP_FILES` | при сохранении сохранять старую версию как `.bak` (`FileEditor`) |

Программно: `editor.search(what, flags) -> bool` ищет от курсора вперёд и
выделяет найденное.

## Работа с текстом из программы

```rust
use tvision::*;

fn fill(ed: &mut Editor) {
    ed.set_text("первая строка\nвторая строка"); // сбрасывает отмену и «изменён»
    ed.set_select(Pos::new(1, 0), Pos::new(1, 6)); // выделить «вторая»
    ed.insert_text("третья", false); // заменить выделение
    assert!(ed.modified);
    let _all = ed.text();
    let _n = ed.line_count();
    let _line = ed.line(0);
    let _cur = ed.cursor();
    ed.undo();
}
```

| Метод / поле | Что делает |
|--------------|-----------|
| `text()`, `set_text(s)` | весь текст (строки через `\n`) |
| `line_count()`, `line(i)` | число строк, строка `i` |
| `cursor() -> Pos` | позиция курсора (`Pos { line, col }`, `col` — номер символа) |
| `selection()`, `selected_text()` | выделение |
| `set_select(from, to)` | выделить и поставить курсор в конец |
| `insert_text(s, select)` | вставить на место курсора (заменив выделение) |
| `delete_select()` | удалить выделение |
| `undo()`, `redo()` | отмена, повтор |
| `search(what, flags)` | поиск |
| `track_cursor(center)`, `scroll_to(x, y)` | прокрутка |
| `col_to_x`, `x_to_col` | перевод между номером символа и экранным столбцом (табуляции) |
| `modified`, `overwrite`, `auto_indent`, `can_undo` | флаги |
| `delta` | прокрутка (x — столбец, y — строка) |
| `h_scroll`, `v_scroll`, `indicator` | id связанных полос и индикатора |

Ширина табуляции — `editor::TAB_SIZE` (8); `display_width(s)` считает
ширину строки на экране с табуляциями.

## `Editor` в своём окне

```rust
use tvision::*;

fn notes_window(app: &mut Program) {
    let mut w = Window::new(Rect::new(5, 2, 65, 18), "Заметки", 1);
    let h = w.standard_scroll_bar(false);
    let v = w.standard_scroll_bar(true);
    let mut ed = Editor::new(w.interior(), Some(h), Some(v), None);
    ed.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
    ed.set_text("Пишите здесь…");
    w.add(ed);
    app.insert_window(w);
}
```

## `Memo` в диалоге

```rust
use tvision::*;

fn comment_dialog() -> Dialog {
    let mut d = Dialog::centered(50, 14, "Комментарий");
    let v = d.add(ScrollBar::new(Rect::new(46, 2, 47, 10)));
    let memo = d.add(Memo::new(Rect::new(3, 2, 46, 10), None, Some(v)));
    d.add(Label::new(Rect::new(2, 1, 12, 2), "~Т~екст", Some(memo)));
    d.add(Button::new(Rect::new(13, 11, 25, 13), "O~K~", cm::OK, bf::DEFAULT));
    d.add(Button::new(Rect::new(27, 11, 39, 13), "Отмена", cm::CANCEL, bf::NORMAL));
    d
}
```

Данные `Memo` — `Value::Str` со всем текстом. Внутренний редактор —
поле `editor`.

## `FileEditor`

| Метод / поле | Что делает |
|--------------|-----------|
| `FileEditor::new(bounds, h, v, indicator, path)` | редактор; существующий файл загружается |
| `load_file()`, `save_file()` | загрузить / записать (`true` — успех) |
| `file_name` | путь (пустой — «Untitled» / «Без имени») |
| `title()` | имя для заголовка окна |
| `last_error` | текст ошибки последней операции с файлом |
| `editor` | внутренний `Editor` |

Перед закрытием окна `FileEditor::valid(cm::CLOSE)` спрашивает о
сохранении изменённого текста (окно сообщения запускается через
`ctx.exec`, ответ приходит в `exec_result`).
