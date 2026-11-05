# Справочник: элементы управления

Всё, что кладут в диалоги и окна: кнопки, надписи, поля ввода, флажки,
списки, полосы, окна сообщений. Модули `widgets::*`.

* [`Button`](#button) · [`StaticText`, `ColoredText`](#statictext-и-coloredtext) · [`Label`](#label)
* [`InputLine`](#inputline) · [`InputLong`](#inputlong)
* [`Cluster`: `CheckBoxes`, `RadioButtons`, `MultiCheckBoxes`](#cluster)
* [`ScrollBar`](#scrollbar) · [`ListBox`](#listbox)
* [`History`, `HistoryWindow`](#history-и-historywindow) · [`Combo`, `ComboWindow`](#combo-и-combowindow)
* [`ProgressBar`](#progressbar)
* [Окна сообщений: `message_box`, `input_box`, `MsgBoxText`](#окна-сообщений)

Координаты элементов — от левого верхнего угла окна или диалога; строка
0 и столбец 0 заняты рамкой.

---

## `Button`

**Кнопка** (`TButton`): нажатие посылает команду. Тень и «нажатие» рисует
сама.

```rust
use tvision::*;

const CM_APPLY: u16 = 150;

fn buttons(d: &mut Dialog) {
    d.add(Button::new(Rect::new(4, 10, 16, 12), "O~K~", cm::OK, bf::DEFAULT)); // Enter
    d.add(Button::new(Rect::new(18, 10, 32, 12), "~П~рименить", CM_APPLY, bf::NORMAL));
    d.add(Button::new(Rect::new(34, 10, 46, 12), "Отмена", cm::CANCEL, bf::NORMAL)); // и Esc
}
```

| Поле / метод | Что |
|--------------|-----|
| `Button::new(bounds, title, cmd, flags)` | высота 2 (вторая строка — тень) |
| `title` | C-строка: Alt+буква нажимает кнопку, в диалоге — и просто буква |
| `command`, `flags`, `am_default` | команда, флаги, «по умолчанию сейчас» |
| `press(ctx)` | нажать программно |

Флаги `bf`: `NORMAL`, `DEFAULT` (по Enter), `LEFT_JUST` (текст влево),
`BROADCAST` (вместо команды — сообщение соседям с `Info::Id` кнопки;
так кнопка общается с соседними элементами, не выходя из диалога),
`GRAB_FOCUS` (забирать фокус при нажатии мышью).

Нажатие с `bf::NORMAL` ставит команду в очередь (`Info::Id` кнопки);
`OK`/`CANCEL`/`YES`/`NO` закрывают модальный диалог. Кнопка с
запрещённой командой рисуется серой и не нажимается.

## `StaticText` и `ColoredText`

**Текст** с переносом по словам (`TStaticText`). Строка, начинающаяся с
`\x03`, центрируется; `\n` — новая строка.

```rust
use tvision::*;

fn texts(d: &mut Dialog) {
    d.add(StaticText::new(Rect::new(2, 2, 40, 4), "\x03Длинный текст переносится по словам в пределах прямоугольника."));
    d.add(StaticText::colored(Rect::new(2, 5, 40, 6), "Внимание!", color::attr(14, 4))); // жёлтый на красном
}
```

| Поле / метод | Что |
|--------------|-----|
| `StaticText::new(bounds, text)` | текст цветом палитры |
| `StaticText::colored(bounds, text, attr)` | свой цвет (`TColoredText` из Dialog Design) |
| `text`, `attr` | можно менять на ходу (`app.find_mut::<StaticText>(id)`) |

`ColoredText` — псевдоним `StaticText`.

## `Label`

**Подпись** элемента (`TLabel`): щелчок по ней или Alt+буква передают
фокус элементу, а сама подпись подсвечивается, пока элемент в фокусе.

```rust
use tvision::*;

fn labeled(d: &mut Dialog) {
    let name = d.add(InputLine::new(Rect::new(12, 2, 40, 3), 60));
    d.add(Label::new(Rect::new(2, 2, 11, 3), "~И~мя:", Some(name)));
}
```

Поля: `text`, `link: Option<ViewId>` (связанный элемент), `light`
(подсвечена ли).

---

## `InputLine`

**Строка ввода** (`TInputLine`): прокрутка, выделение, буфер обмена,
вставка/замена, история, проверка.

```rust
use tvision::*;

fn inputs(d: &mut Dialog) -> ViewId {
    let mut il = InputLine::new(Rect::new(3, 2, 30, 3), 80).with_validator(FilterValidator::new("0123456789"));
    il.set_text("42"); // курсор в конец, текст выделен — набор заменит его
    il.history_id = Some(7); // запоминать введённое (вместе со значком History)
    d.add(il)
}

fn read(app: &Program, id: ViewId) -> String {
    app.find::<InputLine>(id).map(|il| il.text()).unwrap_or_default()
}
```

| Поле / метод | Что |
|--------------|-----|
| `InputLine::new(bounds, max_len)` | `max_len` — наибольшая длина в символах |
| `with_validator(v)`, `validator` | [проверка ввода](15-validators.md) |
| `text()`, `set_text(s)` | текст (`set_text` выделяет всё, как `setData`) |
| `cursor_pos()`, `set_cursor_pos(i)` | позиция курсора (номер символа) |
| `select_all(on)` | выделить всё / снять |
| `history_id: Option<u16>` | номер списка истории |
| данные | `Value::Str` |

Клавиши: ←/→, Home/End, Ctrl+←/→ (по словам), Shift+стрелки
(выделение), Backspace, Del, Ins (вставка/замена), Ctrl+Y (очистить),
Ctrl+Ins / Shift+Ins / Shift+Del (буфер обмена, в том числе системный).

## `InputLong`

**Поле для целого числа** в диапазоне (`TInputLong` из Dialog Design).

```rust
use tvision::*;

fn number(d: &mut Dialog) -> ViewId {
    let n = d.add(InputLong::new(Rect::new(14, 2, 24, 3), 8, -100, 1000, il::HEX, "~С~чётчик"));
    d.add(Label::new(Rect::new(2, 2, 13, 3), "~С~чётчик", Some(n)));
    n
}

fn parse() {
    assert_eq!(parse_long("$1F"), Some(31)); // как Val в Turbo Pascal
    assert_eq!(parse_long("-12"), Some(-12));
    let mut f = InputLong::new(Rect::new(0, 0, 10, 1), 8, 0, 100, 0, "N");
    f.set_value(500); // подрезается по диапазону
    assert_eq!(f.value(), 100);
}
```

| Поле / метод | Что |
|--------------|-----|
| `InputLong::new(bounds, max_len, low, high, flags, label)` | `label` — для сообщения об ошибке |
| `value()`, `set_value(v)`, `range_check()` | значение (с подрезкой) и проверка |
| `il`, `il_options`, `low`, `high` | вложенная `InputLine`, флаги, диапазон |
| данные | `Value::Int` |

Флаги `il`: `HEX` — можно вводить `$1F`, `BLANK_EQ_ZERO` — пустое = 0,
`DISPLAY_HEX` — показывать в шестнадцатеричном виде. Выход из поля и
закрытие диалога проверяют диапазон.

---

## `Cluster`

**Группа флажков или переключателей** (`TCluster`). Один тип на три вида;
`CheckBoxes`, `RadioButtons`, `MultiCheckBoxes` — псевдонимы `Cluster`.

```rust
use tvision::*;

fn clusters(d: &mut Dialog) {
    // 6 пунктов в 3 строках — два столбца.
    let mut cb = Cluster::check_boxes(Rect::new(3, 2, 30, 5), &["~А~", "~Б~", "~В~", "~Г~", "~Д~", "~Е~"]);
    cb.set_button_state(0b100, false); // пункт 2 («В») недоступен
    d.add(cb);
    d.add(Cluster::radio_buttons(Rect::new(32, 2, 50, 4), &["~Д~а", "~Н~ет"]));
    // Три состояния: « », «+», «-»; 2 бита на пункт.
    d.add(Cluster::multi_check_boxes(Rect::new(3, 6, 20, 8), &["~Ч~тение", "~З~апись"], 3, cf::TWO_BITS, " +-"));
}

fn marks() {
    let mut c = Cluster::check_boxes(Rect::new(0, 0, 20, 2), &["a", "b"]);
    c.press(1);
    assert!(c.mark(1) && !c.mark(0));
    assert_eq!(c.value, 0b10);
}
```

| Конструктор | Значение (`Value::U32`) |
|-------------|------------------------|
| `Cluster::check_boxes(bounds, items)` | маска: бит `i` — пункт `i` отмечен |
| `Cluster::radio_buttons(bounds, items)` | номер выбранного |
| `Cluster::multi_check_boxes(bounds, items, sel_range, flags, states)` | по `flags >> 8` бит на пункт; `states` — символ каждого состояния |

| Поле / метод | Что |
|--------------|-----|
| `value`, `sel`, `strings`, `kind: ClusterKind` | значение, пункт под курсором, надписи, вид |
| `mark(i)`, `multi_mark(i)` | отмечен ли / состояние пункта |
| `press(i)` | нажать пункт |
| `button_state(i)`, `set_button_state(mask, on)`, `enable_mask` | доступность пунктов |

Пункты раскладываются по столбцам: высота группы — число строк в столбце.
Клавиши: стрелки, пробел, горячая буква. Флаги `cf` (для
многосостоятельных): `ONE_BIT`, `TWO_BITS`, `FOUR_BITS`, `EIGHT_BITS`.

`ClusterKind` — `CheckBoxes`, `RadioButtons`, `MultiCheckBoxes`.

---

## `ScrollBar`

**Полоса прокрутки** (`TScrollBar`). `ScrollBar::new(bounds)` —
вертикальная, если ширина 1. Поля `value`, `min`, `max`, `pg_step`,
`ar_step`; метод `set_params(value, min, max, pg_step, ar_step)`.

Полоса и её хозяин (список, таблица, редактор) обмениваются сообщениями
внутри **общего владельца**: полоса рассылает `cm::SCROLL_BAR_CHANGED`,
хозяин присылает `cm::SCROLL_BAR_SET`. Поэтому полоса лежит в той же
группе, а хозяин получает её id в конструкторе. В окне полосы на рамке
создаёт `window.standard_scroll_bar(vertical)`.

## `ListBox`

**Список строк** (`TListBox`), в один или несколько столбцов.

```rust
use tvision::*;

fn languages(d: &mut Dialog) -> ViewId {
    let sb = d.add(ScrollBar::new(Rect::new(30, 2, 31, 8)));
    let mut lb = ListBox::new(Rect::new(3, 2, 30, 8), 2, Some(sb)); // 2 столбца
    lb.set_items(["Rust", "C++", "Pascal", "Go", "Zig"].iter().map(|s| s.to_string()).collect());
    lb.focus_item(2);
    d.add(lb)
}

fn chosen(d: &Dialog, id: ViewId) -> Option<String> {
    d.win.group.find::<ListBox>(id)?.selected_text().map(String::from)
}
```

| Поле / метод | Что |
|--------------|-----|
| `ListBox::new(bounds, num_cols, v_scroll)` | |
| `set_items(v)`, `items` | строки |
| `focused`, `focus_item(i)`, `top_item` | выбранная строка, первая видимая |
| `selected_text()`, `range()` | текст выбранной, число строк |
| `palette` | своя палитра (у списка истории — другая) |
| данные | `Value::Int` — номер выбранной |

Enter и двойной щелчок рассылают `cm::LIST_ITEM_SELECTED` с
`Info::Id(список)`; буква — поиск по первым буквам. Пустой список
показывает `listbox::EMPTY_TEXT` (`<empty>`).

---

## `History` и `HistoryWindow`

**История ввода** (`THistory`): значок `▐↓▌` справа от строки ввода
открывает окно с прошлыми значениями.

```rust
use tvision::*;

fn with_history(d: &mut Dialog) {
    let mut il = InputLine::new(Rect::new(3, 2, 30, 3), 80);
    il.history_id = Some(5);
    let id = d.add(il);
    d.add(History::new(Rect::new(30, 2, 33, 3), id, 5));
}

fn store() {
    history_add(9, "первый");
    history_add(9, "второй");
    assert_eq!(history_str(9, 0).as_deref(), Some("второй")); // 0 — самая свежая
    assert_eq!(history_count(9), 2);
}
```

* `History::new(bounds, link, history_id)` — значок 3×1; поля `link`,
  `history_id`. Окно открывается щелчком или ↓ в строке.
* Строки запоминаются при нажатии кнопки диалога и уходе фокуса.
* Общее хранилище: `history_add(id, s)`, `history_count(id)`,
  `history_str(id, i)`, `history_list(id)`, `clear_history()`; в списке не
  больше `history::HISTORY_LIMIT` (32), повтор переносится в начало.
* `HistoryWindow::new(bounds, id)` — само окно со списком;
  `selection()` — выбранная строка. Открывается значком само.

## `Combo` и `ComboWindow`

**Выпадающий список** (`TCombo` из Dialog Design) — значок рядом со
строкой ввода со списком готовых значений.

```rust
use tvision::*;

fn city(d: &mut Dialog) {
    let il = d.add(InputLine::new(Rect::new(3, 2, 25, 3), 30));
    let mut cb = Combo::new(Rect::new(25, 2, 26, 3), il, cbx::ONLY_LIST, &["Москва", "Тверь", "Казань"]);
    cb.activate_char('▼'); // свой значок
    d.add(cb);
}
```

| Поле / метод | Что |
|--------------|-----|
| `Combo::new(bounds, link, flags, items)` | обычно 1×1 или 3×1 справа от строки (`InputLine` или `InputLong`) |
| `new_list(v)`, `list` | список (сортируется без учёта регистра) |
| `activate_char(c)`, `show_char`, `activate` | значок и клавиша вызова (для стрелок — ↓) |

Флаги `cbx`: `ONLY_LIST` — ввести можно только значение из списка (буквы
ищут, пробел листает); `NO_TRANSFER` — не участвовать в обмене данными
(иначе данные — `Value::List` строк списка); `DISPOSES_LIST` — для
совместимости. `ComboWindow` — открывающееся окно списка;
`selection()` — выбранная строка.

---

## `ProgressBar`

**Полоса выполнения** — в оригинале её нет, сделана в том же стиле.

```rust
use tvision::*;

fn bars(d: &mut Dialog) {
    d.add(ProgressBar::new(Rect::new(3, 2, 43, 3), 200)); // «25%» по центру
    d.add(ProgressBar::new(Rect::new(3, 4, 43, 5), 10).with_style(ProgressStyle::Blocks).with_caption("{value} из {max} файлов"));
    d.add(ProgressBar::new(Rect::new(3, 6, 43, 7), 0).indeterminate()); // неизвестная длительность
}

fn values() {
    let mut p = ProgressBar::new(Rect::new(0, 0, 20, 1), 200);
    p.set_value(50);
    assert_eq!((p.percent(), p.fraction()), (25, 0.25));
    p.step(1000); // подрезается
    assert!(p.is_done());
}
```

| Поле / метод | Что |
|--------------|-----|
| `ProgressBar::new(bounds, max)` | диапазон `0..=max` |
| `with_range(min, max)`, `set_range` | другой диапазон |
| `with_style(ProgressStyle::Smooth / Blocks)`, `style` | плавно (1/8 клетки) или блоками `█░` |
| `with_caption("… {percent}% …")`, `caption`, `show_text` | надпись: `{value}`, `{min}`, `{max}`, `{percent}` |
| `indeterminate()`, `indeterminate` | бегающий блок |
| `with_colors(empty, filled)`, `colors` | свои цвета (нужны в окне — палитра рассчитана на диалог) |
| `set_value`, `step`, `value`, `fraction`, `percent`, `is_done`, `text` | значение |
| данные | `Value::Int` |

Обновлять — напрямую (`app.find_mut::<ProgressBar>(id)`) или сообщением
`ctx.message(cm::PROGRESS_SET, Info::Progress { id, value })`. В
модальном диалоге — функцией фоновой работы `exec_dialog_idle`
([пример](13-program.md#окна-и-модальность)); долгую работу ведите в
потоке ([глава 7](../07-dialogs.md#полоса-выполнения-progressbar),
`examples/progress.rs`, `examples/json_load.rs`). Палитра —
`progress::PROGRESS_PALETTE`.

---

## Окна сообщений

Готовые модальные окна (`messageBox`, `inputBox`).

```rust
use tvision::*;

fn dialogs(app: &mut Program) {
    // Возвращает команду нажатой кнопки.
    if message_box(app, "\x03Удалить файл?", mf::CONFIRMATION | mf::YES_BUTTON | mf::NO_BUTTON) == cm::YES {
        // …
    }
    message_box_rect(app, Rect::new(0, 0, 60, 12), "Длинное сообщение\nв несколько строк", mf::ERROR | mf::OK_BUTTON, true);
    if let Some(name) = input_box(app, "Новый файл", "~И~мя:", "без имени.txt", 100) {
        let _ = name;
    }
}
```

| Функция | Что |
|---------|-----|
| `message_box(app, msg, options) -> cmd` | окно 40×9 в центре |
| `message_box_rect(app, r, msg, options, centered) -> cmd` | своего размера |
| `build_message_box(r, msg, options) -> Dialog` | построить, не запуская (для `ctx.exec` из представления) |
| `input_box(app, title, label, initial, limit) -> Option<String>` | строка ввода; `Some` — нажали OK |

Флаги `mf` — вид (заголовок): `WARNING`, `ERROR`, `INFORMATION`,
`CONFIRMATION`; кнопки: `YES_BUTTON`, `NO_BUTTON`, `OK_BUTTON`,
`CANCEL_BUTTON`, `YES_NO_CANCEL`, `OK_CANCEL`. Первая кнопка — по
умолчанию. Текст переносится по словам; `\x03` в начале — по центру.

`MsgBoxText` — тексты кнопок и заголовков (`YES`, `NO`, `OK`, `CANCEL`,
`WARNING`, `ERROR`, `INFORMATION`, `CONFIRM`); на русский их переводит
`locale::set_language(Language::Ru)`.
