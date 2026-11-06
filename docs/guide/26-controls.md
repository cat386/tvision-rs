# Справочник: элементы управления

Всё, что кладут в диалоги и окна. Координаты элементов считаются от
левого верхнего угла окна или диалога; строка 0 и столбец 0 — рамка
([урок 4](04-coordinates.md)).

На этой странице:
[`Button`](#button--кнопка) ·
[`StaticText`](#statictext--текст) ·
[`Label`](#label--подпись) ·
[`InputLine`](#inputline--строка-ввода) ·
[`InputLong`](#inputlong--поле-для-числа) ·
[`Cluster`: флажки и переключатели](#cluster--флажки-и-переключатели) ·
[`ScrollBar`](#scrollbar--полоса-прокрутки) ·
[`ListBox`](#listbox--список) ·
[`History`](#history--история-ввода) ·
[`Combo`](#combo--выпадающий-список) ·
[`ProgressBar`](#progressbar--полоса-выполнения) ·
[окна сообщений](#окна-сообщений)

---

## `Button` — кнопка

**Что это.** Кнопка с надписью. Нажатие (щелчком, пробелом, горячей
буквой или Enter для кнопки по умолчанию) посылает команду.

**Аналог:** `TButton`.

```rust
use tvision::*;

const CM_APPLY: u16 = 150;

fn buttons(d: &mut Dialog) {
    d.add(Button::new(Rect::new(4, 10, 16, 12), "O~K~", cm::OK, bf::DEFAULT)); // по Enter
    d.add(Button::new(Rect::new(18, 10, 32, 12), "~П~рименить", CM_APPLY, bf::NORMAL));
    d.add(Button::new(Rect::new(34, 10, 46, 12), "Отмена", cm::CANCEL, bf::NORMAL)); // и по Esc
}
```

**Как создать.** `Button::new(прямоугольник, надпись, команда, флаги)`.
Высота прямоугольника — **2**: вторая строка — тень кнопки.

**Что важно знать.**

* `cm::OK`, `cm::CANCEL`, `cm::YES`, `cm::NO` **закрывают** диалог.
  Другие команды идут дальше — в `AppHandler` (или в `modal_hook`, если
  диалог модальный).
* `bf::DEFAULT` — кнопка «по умолчанию», её нажимает Enter. Такая кнопка
  в диалоге одна.
* Если команда кнопки запрещена (`disable_command`), кнопка серая и не
  нажимается.

**Флаги `bf`:** `NORMAL`, `DEFAULT`, `LEFT_JUST` (надпись слева, а не по
центру), `BROADCAST` (вместо команды разослать сообщение соседям —
чтобы кнопка «поговорила» с другими элементами, не закрывая диалог),
`GRAB_FOCUS` (забирать фокус при нажатии мышью).

**Поля и методы:** `title`, `command`, `flags`, `am_default`,
`press(ctx)` — нажать из программы.

## `StaticText` — текст

**Что это.** Надпись. Длинный текст сам переносится по словам; `\n` —
новая строка; `\x03` в начале — по центру.

**Аналог:** `TStaticText` (и `TColoredText` из Dialog Design).

```rust
use tvision::*;

fn texts(d: &mut Dialog) {
    d.add(StaticText::new(Rect::new(2, 2, 40, 4), "\x03Длинный текст переносится по словам в пределах прямоугольника."));
    d.add(StaticText::colored(Rect::new(2, 5, 40, 6), "Внимание!", color::attr(14, 4))); // жёлтый на красном
}
```

* `StaticText::new(прямоугольник, текст)` — цвет как у текста диалога;
* `StaticText::colored(прямоугольник, текст, цвет)` — свой цвет;
* поля `text` и `attr` можно менять на ходу:
  `app.find_mut::<StaticText>(id).unwrap().text = "…".into()`.

`ColoredText` — другое имя того же типа.

## `Label` — подпись

**Что это.** Подпись к полю. Щелчок по ней или Alt+горячая буква
переводят курсор в поле, а пока поле в фокусе, подпись подсвечена.

**Аналог:** `TLabel`.

```rust
use tvision::*;

fn labeled(d: &mut Dialog) {
    let name = d.add(InputLine::new(Rect::new(12, 2, 40, 3), 60));
    d.add(Label::new(Rect::new(2, 2, 11, 3), "~И~мя:", Some(name))); // Alt+И → в поле
}
```

`Label::new(прямоугольник, текст, id_поля)`. Поля: `text`, `link`,
`light`.

---

## `InputLine` — строка ввода

**Что это.** Поле для одной строки текста. Умеет выделение мышью и
Shift+стрелками, копирование и вставку (в том числе из системного
буфера обмена), режим замены (Ins), историю и проверку.

**Аналог:** `TInputLine`.

```rust
use tvision::*;

fn input(d: &mut Dialog) -> ViewId {
    let mut il = InputLine::new(Rect::new(3, 2, 30, 3), 80); // не длиннее 80 символов
    il.set_text("42"); // начальный текст (выделен — набор заменит его)
    d.add(il)
}

fn read(app: &Program, id: ViewId) -> String {
    app.find::<InputLine>(id).map(|il| il.text()).unwrap_or_default()
}
```

| Метод / поле | Что |
|--------------|-----|
| `InputLine::new(прямоугольник, макс_длина)` | высота — 1; длинный текст прокручивается |
| `text()`, `set_text(s)` | текст |
| `with_validator(v)` | [проверка ввода](25-validators.md) |
| `history_id` | номер списка [истории](#history--история-ввода) |
| данные для диалога | `Value::Str(текст)` |

**Реже:** `cursor_pos()`, `set_cursor_pos(i)`, `select_all(вкл)`,
`validator`.

**Клавиши:** ←/→, Home/End, Ctrl+←/→ (по словам), Shift+стрелки
(выделить), Backspace, Del, Ins (вставка/замена), Ctrl+Y (очистить),
Ctrl+Ins (копировать), Shift+Ins (вставить), Shift+Del (вырезать).

## `InputLong` — поле для числа

**Что это.** Строка ввода для целого числа в заданных пределах. Значение
— сразу число (`Value::Int`), а не строка.

**Аналог:** `TInputLong` из Dialog Design.

```rust
use tvision::*;

fn number(d: &mut Dialog) -> ViewId {
    // макс. длина 8, от −100 до 1000, можно вводить «$1F»; "Счётчик" — для сообщения об ошибке
    let n = d.add(InputLong::new(Rect::new(14, 2, 24, 3), 8, -100, 1000, il::HEX, "~С~чётчик"));
    d.add(Label::new(Rect::new(2, 2, 13, 3), "~С~чётчик", Some(n)));
    n
}

fn values() {
    let mut f = InputLong::new(Rect::new(0, 0, 10, 1), 8, 0, 100, 0, "N");
    f.set_value(500); // больше предела — обрежется
    assert_eq!(f.value(), 100);
    assert_eq!(parse_long("$1F"), Some(31)); // «$» — шестнадцатеричное
}
```

`InputLong::new(прямоугольник, макс_длина, мин, макс, флаги, название)`.
Методы `value()`, `set_value(v)`, `range_check()`. Флаги `il`: `HEX`
(можно `$1F`), `BLANK_EQ_ZERO` (пусто = 0), `DISPLAY_HEX` (показывать
в шестнадцатеричном виде). Функция `parse_long(s)` разбирает число.

---

## `Cluster` — флажки и переключатели

**Что это.** Группа вариантов. Один тип — три вида:

| Как создать | Что это | Значение |
|-------------|---------|----------|
| `Cluster::check_boxes(прямоугольник, &надписи)` | флажки `[X]` — сколько угодно | биты: бит `i` — отмечен вариант `i` |
| `Cluster::radio_buttons(прямоугольник, &надписи)` | переключатели `(•)` — ровно один | номер выбранного |
| `Cluster::multi_check_boxes(…)` | флажки с несколькими состояниями (`[ ]`, `[+]`, `[-]`) | по несколько бит на флажок |

**Аналоги:** `TCheckBoxes`, `TRadioButtons`, `TMultiCheckBoxes`. Имена
`CheckBoxes`, `RadioButtons`, `MultiCheckBoxes` тоже есть — это тот же
`Cluster`.

```rust
use tvision::*;

fn clusters(d: &mut Dialog) {
    // 4 флажка в прямоугольнике высотой 2 — два столбца по 2.
    d.add(Cluster::check_boxes(Rect::new(3, 2, 30, 4), &["~К~ниги", "~С~порт", "~М~узыка", "~И~гры"]));
    d.add(Cluster::radio_buttons(Rect::new(32, 2, 50, 4), &["~Д~а", "~Н~ет"]));
}

fn marks() {
    let mut c = Cluster::check_boxes(Rect::new(0, 0, 20, 2), &["a", "b"]);
    c.press(1); // отметить второй
    assert!(c.mark(1) && !c.mark(0));
    assert_eq!(c.value, 0b10);
}
```

**Что важно знать.**

* Высота прямоугольника — сколько вариантов в одном столбце. Если
  вариантов больше, они переходят в следующий столбец.
* Клавиши: стрелки — между вариантами, пробел — отметить, горячая буква
  — сразу нужный вариант.
* Проверить флажок `i` по значению: `value & (1 << i) != 0`.

**Поля и методы:** `value`, `sel` (вариант под курсором), `strings`,
`mark(i)`, `press(i)`, `set_button_state(маска, вкл)` (сделать варианты
недоступными), `button_state(i)`. **Реже:** `kind` (`ClusterKind`),
`enable_mask`, `multi_mark(i)`;
`Cluster::multi_check_boxes(прямоугольник, &надписи, состояний, флаги,
символы)` — флаги `cf::ONE_BIT`, `TWO_BITS`, `FOUR_BITS`, `EIGHT_BITS`
(сколько бит на флажок), символы — по одному на состояние, например
`" +-"`.

---

## `ScrollBar` — полоса прокрутки

**Что это.** Полоса с бегунком: показывает, какая часть списка видна, и
позволяет прокручивать мышью.

**Аналог:** `TScrollBar`.

**Как использовать.** Полосу создают и кладут **в то же окно или
диалог**, что и список, а её id передают списку:

```rust,ignore
let sb = d.add(ScrollBar::new(Rect::new(30, 2, 31, 8)));   // шириной 1 — вертикальная
let list = ListBox::new(Rect::new(3, 2, 30, 8), 1, Some(sb));
```

В окне удобнее `window.standard_scroll_bar(true)` — полоса сразу на
правой рамке.

Поля `value`, `min`, `max`, `pg_step`, `ar_step` и метод
`set_params(…)` — для своих прокручиваемых представлений; списки,
таблицы и редактор управляют полосой сами.

## `ListBox` — список

**Что это.** Список строк с выбором. Может быть в несколько столбцов.

**Аналог:** `TListBox`.

```rust
use tvision::*;

fn languages(d: &mut Dialog) -> ViewId {
    let sb = d.add(ScrollBar::new(Rect::new(30, 2, 31, 8)));
    let mut lb = ListBox::new(Rect::new(3, 2, 30, 8), 2, Some(sb)); // 2 столбца
    lb.set_items(["Rust", "C++", "Pascal", "Go", "Zig"].iter().map(|s| s.to_string()).collect());
    lb.focus_item(2); // выбрать «Pascal»
    d.add(lb)
}

fn chosen(d: &Dialog, id: ViewId) -> Option<String> {
    d.win.group.find::<ListBox>(id)?.selected_text().map(String::from)
}
```

| Метод / поле | Что |
|--------------|-----|
| `ListBox::new(прямоугольник, столбцов, полоса)` | |
| `set_items(строки)` | содержимое |
| `focused`, `focus_item(i)` | номер выбранной строки |
| `selected_text()` | текст выбранной строки |
| данные для диалога | `Value::Int(номер)` |

**Реже:** `items`, `top_item`, `range()`, `palette`. Пустой список
показывает `<empty>` (`listbox::EMPTY_TEXT`, с русским языком —
«<пусто>»).

**Что важно знать.** Enter и двойной щелчок рассылают сообщение
`cm::LIST_ITEM_SELECTED` соседям; буква — переход к строке на эту
букву. **Список сам не растягивается** с окном — задайте
`grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y`.

---

## `History` — история ввода

**Что это.** Маленький значок `▐↓▌` справа от строки ввода. Щелчок по
нему (или ↓ в строке) открывает список того, что вводили раньше.

**Аналог:** `THistory`.

```rust
use tvision::*;

fn with_history(d: &mut Dialog) {
    let mut il = InputLine::new(Rect::new(3, 2, 30, 3), 80);
    il.history_id = Some(5); // номер списка истории — любой, но одинаковый у значка
    let id = d.add(il);
    d.add(History::new(Rect::new(30, 2, 33, 3), id, 5)); // значок 3×1 справа
}

fn store() {
    history_add(9, "первый");
    history_add(9, "второй");
    assert_eq!(history_str(9, 0).as_deref(), Some("второй")); // 0 — самое свежее
    assert_eq!(history_count(9), 2);
}
```

**Что важно знать.** Строки запоминаются, когда нажата кнопка диалога или
фокус ушёл из поля. В каждом списке не больше 32 строк
(`history::HISTORY_LIMIT`), повтор переносится в начало.

**Функции:** `history_add(номер, строка)`, `history_count(номер)`,
`history_str(номер, i)`, `history_list(номер)`, `clear_history()`.
**Реже:** `HistoryWindow` — само всплывающее окно со списком
(`HistoryWindow::new(границы, номер)`, `selection()`); значок открывает
его сам.

## `Combo` — выпадающий список

**Что это.** Значок рядом со строкой ввода, который открывает список
готовых вариантов. Выбранный вариант попадает в строку.

**Аналог:** `TCombo` из Dialog Design.

```rust
use tvision::*;

fn city(d: &mut Dialog) {
    let il = d.add(InputLine::new(Rect::new(3, 2, 25, 3), 30));
    let mut cb = Combo::new(Rect::new(25, 2, 26, 3), il, cbx::ONLY_LIST, &["Москва", "Тверь", "Казань"]);
    cb.activate_char('▼'); // свой значок
    d.add(cb);
}
```

`Combo::new(прямоугольник, id_строки, флаги, &варианты)`. Флаги `cbx`:
`ONLY_LIST` — ввести можно **только** вариант из списка (буквы ищут по
списку); `NO_TRANSFER` — не участвовать в обмене данными диалога;
`DISPOSES_LIST` — для совместимости. Методы: `new_list(варианты)`,
`activate_char(c)`; поля `list`, `show_char`, `activate`. **Реже:**
`ComboWindow` — открывающееся окно списка (`selection()`).

---

## `ProgressBar` — полоса выполнения

**Что это.** Полоса, которая показывает, сколько работы сделано. В
оригинальном Turbo Vision её нет.

```rust
use tvision::*;

fn bars(d: &mut Dialog) {
    d.add(ProgressBar::new(Rect::new(3, 2, 43, 3), 200)); // от 0 до 200, по центру «25%»
    d.add(ProgressBar::new(Rect::new(3, 4, 43, 5), 10).with_style(ProgressStyle::Blocks).with_caption("{value} из {max} файлов"));
    d.add(ProgressBar::new(Rect::new(3, 6, 43, 7), 0).indeterminate()); // «бегающий» блок
}

fn values() {
    let mut p = ProgressBar::new(Rect::new(0, 0, 20, 1), 200);
    p.set_value(50);
    assert_eq!(p.percent(), 25);
    p.step(1000); // больше максимума — обрежется
    assert!(p.is_done());
}
```

| Метод | Что |
|-------|-----|
| `ProgressBar::new(прямоугольник, максимум)` | полоса от 0 до максимума |
| `set_value(v)`, `step(шаг)`, `value()` | значение |
| `percent()`, `fraction()`, `is_done()` | сколько сделано |
| `with_caption("{value} из {max}")` | своя надпись: `{value}`, `{min}`, `{max}`, `{percent}` |
| `with_style(ProgressStyle::Smooth / Blocks)` | плавная или блоками `█░` |
| `indeterminate()` | когда сколько осталось — неизвестно |

**Реже:** `with_range(мин, макс)`, `set_range`, `with_colors(пустая,
заполненная)` (нужно, если полоса в обычном окне, а не в диалоге),
`text()`, поля `style`, `caption`, `show_text`, `indeterminate`, `colors`;
палитра `progress::PROGRESS_PALETTE`.

**Как обновлять во время долгой работы.** В модальном диалоге —
функцией `exec_dialog_idle` ([пример](23-program.md#окна-и-диалоги)).
Если работа идёт в отдельном потоке — см. главу 7 и примеры
`examples/progress.rs`, `examples/json_load.rs`. Ещё один способ —
сообщение `ctx.message(cm::PROGRESS_SET, Info::Progress { id, value })`.

---

## Окна сообщений

**Что это.** Готовые маленькие диалоги: сообщение с кнопками и окно для
ввода одной строки.

**Аналоги:** `messageBox`, `inputBox`.

```rust
use tvision::*;

fn dialogs(app: &mut Program) {
    // Вопрос: функция возвращает команду нажатой кнопки.
    if message_box(app, "\x03Удалить файл?", mf::CONFIRMATION | mf::YES_BUTTON | mf::NO_BUTTON) == cm::YES {
        // …удалить…
    }
    // Сообщение побольше — своего размера.
    message_box_rect(app, Rect::new(0, 0, 60, 12), "Длинное сообщение\nв несколько строк", mf::ERROR | mf::OK_BUTTON, true);
    // Ввод строки: Some(текст) — нажали OK, None — отмена.
    if let Some(name) = input_box(app, "Новый файл", "~И~мя:", "без имени.txt", 100) {
        let _ = name;
    }
}
```

| Функция | Что делает |
|---------|-----------|
| `message_box(app, текст, флаги) -> команда` | сообщение 40×9 по центру |
| `message_box_rect(app, прямоугольник, текст, флаги, по_центру)` | сообщение своего размера |
| `input_box(app, заголовок, подпись, начальный_текст, макс_длина)` | строка ввода; `Some(текст)` или `None` |
| `build_message_box(прямоугольник, текст, флаги) -> Dialog` | собрать, не показывая (например, для `ctx.exec` из представления) |

**Флаги `mf`.** Заголовок: `INFORMATION` («Информация»), `WARNING`,
`ERROR`, `CONFIRMATION`. Кнопки: `OK_BUTTON`, `YES_BUTTON`, `NO_BUTTON`,
`CANCEL_BUTTON`; сочетания `YES_NO_CANCEL`, `OK_CANCEL`. Флаги
складываются знаком `|`. Первая кнопка — «по умолчанию» (Enter).

**Что важно знать.** Текст переносится по словам; `\n` — новая строка;
`\x03` в начале — по центру. Надписи кнопок и заголовков по-русски —
после `locale::set_language(Language::Ru)`; сами тексты хранятся в
`MsgBoxText`.
