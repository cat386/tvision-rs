# 7. Диалоги и элементы управления

## Диалог `Dialog`

Диалог — окно с серой палитрой, которое обычно выполняется модально.
Внутри — элементы управления; порядок их вставки задаёт порядок перехода
по Tab.

```rust
use tvision::*;

fn settings_dialog() -> Dialog {
    let mut d = Dialog::centered(56, 17, "Настройки");

    let name = d.add(InputLine::new(Rect::new(14, 2, 52, 3), 64));
    d.add(Label::new(Rect::new(2, 2, 13, 3), "~И~мя:", Some(name)));

    let style = d.add(Cluster::check_boxes(Rect::new(3, 5, 26, 8), &["~Ж~ирный", "~К~урсив", "~П~одчёркнутый"]));
    d.add(Label::new(Rect::new(2, 4, 12, 5), "Стиль", Some(style)));

    let color = d.add(Cluster::radio_buttons(Rect::new(29, 5, 52, 8), &["~С~иний", "~З~елёный", "К~р~асный"]));
    d.add(Label::new(Rect::new(28, 4, 38, 5), "Цвет", Some(color)));

    let sb = d.add(ScrollBar::new(Rect::new(52, 10, 53, 13)));
    let mut lb = ListBox::new(Rect::new(3, 10, 52, 13), 2, Some(sb));
    lb.set_items(["Rust", "C++", "Pascal", "Go"].iter().map(|s| s.to_string()).collect());
    let list = d.add(lb);
    d.add(Label::new(Rect::new(2, 9, 12, 10), "~Я~зык", Some(list)));

    d.add(Button::new(Rect::new(14, 14, 26, 16), "O~K~", cm::OK, bf::DEFAULT));
    d.add(Button::new(Rect::new(30, 14, 42, 16), "Отмена", cm::CANCEL, bf::NORMAL));
    d
}
```

| Метод / поле | Назначение |
|--------------|-----------|
| `Dialog::new(bounds, title)` | диалог в заданных границах (`TDialog`) |
| `Dialog::centered(w, h, title)` | диалог размера `w`×`h`, центрируется при вставке |
| `add(v)`, `insert(box)` | вставить элемент, вернуть его id |
| `select(id)` | какой элемент получит фокус первым (по умолчанию — первый вставленный выбираемый) |
| `set_palette(DialogPalette::Gray / Blue / Cyan)` | цветовая схема |
| `win` | окно диалога: `win.title`, `win.flags`, `win.group` (элементы) |
| `id()` | id диалога |

Поведение диалога:

* **Esc** посылает `cm::CANCEL`, **Enter** нажимает кнопку по умолчанию;
* команды `cm::OK`, `cm::CANCEL`, `cm::YES`, `cm::NO` в модальном режиме
  завершают диалог с этим кодом (после проверки `valid`; «Отмена»
  допустима всегда);
* все остальные команды кнопок проходят дальше — к `Program::modal_hook`
  (в модальном режиме) или к `AppHandler`;
* окно диалога по умолчанию можно двигать и закрывать (`wf::MOVE | wf::CLOSE`),
  но не растягивать.

Координаты элементов отсчитываются от левого верхнего угла диалога;
строка 0 и столбец 0 — рамка, поэтому элементы обычно начинаются с `x ≥ 2`,
`y ≥ 1`.

## Обмен данными

У диалога (как у любой группы) есть `get_data()` и `set_data(&Value)`:
данные — это `Value::List` со значениями **всех элементов с данными** в
порядке вставки. Метки, кнопки, статический текст и полосы прокрутки
данных не имеют и в список не попадают.

Для диалога выше список такой:

| № | Элемент | Значение |
|---|---------|----------|
| 0 | `InputLine` «Имя» | `Value::Str(текст)` |
| 1 | флажки «Стиль» | `Value::U32(маска)`: бит 0 — «Жирный», бит 1 — «Курсив»… |
| 2 | переключатели «Цвет» | `Value::U32(номер выбранного)` |
| 3 | `ListBox` «Язык» | `Value::Int(номер выбранной строки)` |

Удобно описать данные своей структурой и переводить её в `Value` и
обратно — так же, как в C++ описывали запись `struct DialogData`:

```rust
use tvision::*;

#[derive(Clone, Debug, Default)]
struct Settings {
    name: String,
    style: u32,
    color: u32,
    language: i64,
}

impl Settings {
    fn to_value(&self) -> Value {
        Value::List(vec![
            Value::Str(self.name.clone()),
            Value::U32(self.style),
            Value::U32(self.color),
            Value::Int(self.language),
        ])
    }

    fn from_value(v: &Value) -> Self {
        let l = v.as_list().unwrap_or(&[]);
        let get = |i: usize| l.get(i).cloned().unwrap_or_default();
        Settings {
            name: get(0).as_str().unwrap_or("").to_string(),
            style: get(1).as_u32().unwrap_or(0),
            color: get(2).as_u32().unwrap_or(0),
            language: get(3).as_int().unwrap_or(0),
        }
    }
}

fn edit_settings(app: &mut Program, s: &mut Settings, d: Dialog) {
    let mut d = d;
    d.set_data(&s.to_value()); // заполнить поля
    let (cmd, d) = app.exec_dialog(d);
    if cmd == cm::OK {
        *s = Settings::from_value(&d.get_data()); // прочитать
    }
}
```

> Генератор кода программы dlgdsn (редактор диалогов) создаёт такую
> структуру и функцию построения диалога автоматически.

Если нужен один элемент, его можно прочитать напрямую:
`d.win.group.find::<InputLine>(name).map(|il| il.text())`.

## Элементы управления

### Кнопка `Button`

```rust
use tvision::*;

fn buttons(d: &mut Dialog) {
    d.add(Button::new(Rect::new(10, 12, 22, 14), "O~K~", cm::OK, bf::DEFAULT));
    d.add(Button::new(Rect::new(24, 12, 36, 14), "~П~рименить", 200, bf::NORMAL));
}
```

`Button::new(bounds, title, command, flags)` — высота кнопки обычно 2:
строка надписи и тень. Нажатие (щелчок, пробел в фокусе, Alt+горячая
буква, просто горячая буква, если элемент в фокусе её не взял, Enter для
кнопки по умолчанию) посылает команду `command` с `Info::Id(id кнопки)`.

Флаги `bf`:

| Флаг | Смысл |
|------|-------|
| `NORMAL` | обычная кнопка |
| `DEFAULT` | кнопка по умолчанию: срабатывает по Enter, выделена цветом |
| `LEFT_JUST` | надпись прижата влево |
| `BROADCAST` | вместо команды рассылает **сообщение** с этим номером соседям (`ctx.message`) — удобно для кнопок, которые обрабатывает другой элемент того же диалога |
| `GRAB_FOCUS` | забирает фокус при нажатии мышью |

Кнопка становится серой и не нажимается, если её команда запрещена
(`disable_command`). Перед нажатием кнопка рассылает
`cm::RECORD_HISTORY` — строки ввода с историей запоминают текст.

Поля: `title`, `command`, `flags`, `am_default`; метод `press(ctx)` —
нажать программно.

### Строка ввода `InputLine`

```rust
use tvision::*;

fn inputs(d: &mut Dialog) {
    let mut il = InputLine::new(Rect::new(3, 2, 33, 3), 40); // не больше 40 символов
    il.set_text("начальное значение");
    let id = d.add(il);
    d.add(Label::new(Rect::new(2, 1, 12, 2), "~Т~екст", Some(id)));
}
```

* `InputLine::new(bounds, max_len)` — однострочное поле; текст длиннее
  поля прокручивается (стрелки `◄` `►` по краям).
* `text()`, `set_text(s)` (выделяет весь текст, как `setData`),
  `select_all(enable)`, `cursor_pos()`, `set_cursor_pos(n)`.
* `with_validator(v)` — проверка ввода ([глава 8](08-validators.md)).
* `history_id: Option<u16>` — номер списка истории (см. `History`).
* Данные: `Value::Str`.

Клавиши: ←/→, Ctrl+←/→ (по словам), Home/End, Backspace, Del, Ins
(вставка/замена), Shift+движение — выделение, Ctrl+Y — очистить,
Ctrl+C/Ctrl+Ins — копировать, Ctrl+X/Shift+Del — вырезать,
Ctrl+V/Shift+Ins — вставить (системный буфер обмена). Мышь: щелчок ставит
курсор, протяжка выделяет, двойной щелчок выделяет всё. При получении
фокуса текст выделяется целиком.

### Метка `Label`

`Label::new(bounds, text, Some(link))` — подпись к элементу `link`.
Щелчок по метке или её горячая буква (Alt+буква всегда, просто буква —
если элемент в фокусе её не взял) передаёт фокус связанному элементу.
Пока связанный элемент в фокусе, метка подсвечена.

Метку вставляют **после** элемента: ей нужен его id. Порядок Tab это не
меняет — метки не выбираются.

### Статический текст `StaticText`

```rust
use tvision::*;

fn texts(d: &mut Dialog) {
    d.add(StaticText::new(Rect::new(2, 1, 40, 3), "Длинный текст переносится по словам\n\x03по центру"));
    d.add(StaticText::colored(Rect::new(2, 4, 40, 5), "Жёлтым по синему", 0x1E));
}
```

* Текст переносится по словам; `\n` начинает новый абзац; абзац,
  начинающийся с `\x03`, выравнивается по центру.
* `StaticText::colored(bounds, text, attr)` — текст своим цветом
  (`TColoredText` из пакета Dialog Design); `attr` — байт «фон<<4 | цвет»
  ([глава 11](11-colors.md)). Тип-синоним — `ColoredText`.
* Поле `text` можно менять на ходу.

### Флажки и переключатели `Cluster`

```rust
use tvision::*;

fn clusters(d: &mut Dialog) {
    // 3 строки → 3 пункта в одном столбце; 6 пунктов в 3 строках — два столбца.
    let mut cb = Cluster::check_boxes(Rect::new(3, 2, 30, 5), &["~А~", "~Б~", "~В~", "~Г~", "~Д~", "~Е~"]);
    cb.set_button_state(0b100, false); // пункт 2 («В») недоступен
    d.add(cb);
    d.add(Cluster::radio_buttons(Rect::new(32, 2, 50, 4), &["~Д~а", "~Н~ет"]));
}
```

| Конструктор | Значение (`Value::U32`) |
|-------------|------------------------|
| `Cluster::check_boxes(bounds, items)` | маска отмеченных: бит i — пункт i |
| `Cluster::radio_buttons(bounds, items)` | номер выбранного пункта |
| `Cluster::multi_check_boxes(bounds, items, sel_range, flags, states)` | состояния пунктов, по `flags >> 8` бит на пункт |

Пункты раскладываются по столбцам: высота группы — число строк в столбце,
лишние пункты переходят в следующий столбец. Ширина столбца — самый
длинный пункт плюс 6 ячеек (на `[ ] ` и промежуток).

Методы: `mark(i)`, `press(i)`, `button_state(i)`,
`set_button_state(mask, enable)` — разрешить/запретить пункты по маске
(если запрещены все, группа перестаёт выбираться); поля `value`, `sel`,
`strings`, `enable_mask`.

Клавиши: стрелки — по пунктам (←/→ — между столбцами), пробел — отметить,
горячая буква пункта — выбрать и отметить.

**Многопозиционные флажки** (`TMultiCheckBoxes` из Turbo Vision 2.0): у
каждого пункта несколько состояний, символ в скобках берётся из строки
`states`, щелчок переключает состояние по кругу.

```rust
use tvision::*;

fn tri_state(d: &mut Dialog) {
    // 3 состояния: « » (0), «+» (1), «-» (2); по 2 бита на пункт.
    d.add(Cluster::multi_check_boxes(Rect::new(3, 2, 20, 4), &["~Ч~тение", "~З~апись"], 3, cf::TWO_BITS, " +-"));
}
```

Флаги `cf`: `ONE_BIT` (2 состояния), `TWO_BITS` (до 4), `FOUR_BITS` (до 16),
`EIGHT_BITS` (до 256). Состояние пункта — `multi_mark(i)`.

### Список `ListBox`

```rust
use tvision::*;

fn list(d: &mut Dialog) -> ViewId {
    let sb = d.add(ScrollBar::new(Rect::new(30, 2, 31, 10)));
    let mut lb = ListBox::new(Rect::new(2, 2, 30, 10), 1, Some(sb));
    lb.set_items(vec!["один".into(), "два".into(), "три".into()]);
    lb.focus_item(1);
    d.add(lb)
}
```

* `ListBox::new(bounds, num_cols, v_scroll)` — список в `num_cols`
  столбцов, связанный с вертикальной полосой прокрутки (её id);
  горизонтальная — поле `h_scroll`.
* `set_items(v)`, `items`, `focused`, `focus_item(i)`, `selected_text()`,
  `range()`, `top_item`.
* Данные: `Value::Int(номер выбранного)`; строки списка задаются
  `set_items` (в C++ `TListBoxRec` передавал и коллекцию — здесь только номер).
* Двойной щелчок или пробел рассылают `cm::LIST_ITEM_SELECTED` с
  `Info::Id(id списка)`.

Клавиши: ↑/↓, ←/→ (между столбцами), PgUp/PgDn, Home/End (видимая
страница), Ctrl+PgUp/PgDn или Ctrl+Home/End (весь список), колесо мыши.
Пустой список показывает `<empty>` (или `<пусто>` при русском языке).

### Таблица `Table`

Таблицы в оригинальном Turbo Vision не было: `Table` написана для
tvision-rs. Она показывает данные в строках и столбцах, с заголовками,
сеткой, прокруткой в обе стороны, правкой ячеек и сортировкой. Таблица
работает и в диалоге, и в обычном окне, и прямо на рабочем столе.

**Данные хранит не таблица, а модель** — любой тип с трейтом
`TableModel`. Таблица спрашивает у модели только видимые ячейки, поэтому
миллион строк × тысяча столбцов показываются так же быстро, как десять
строк.

```rust
use tvision::*;

/// Окно с таблицей на миллион строк: ячейки вычисляются по запросу.
fn big_table(app: &mut Program) {
    let model = FnTableModel::new(1_000_000, 1_000, |row, col| format!("{}:{}", row, col));
    let mut w = Window::new(Rect::new(0, 0, 70, 20), "Данные", 1);
    let v = w.standard_scroll_bar(true); // вертикальная — по строкам
    let h = w.standard_scroll_bar(false); // горизонтальная — по столбцам
    let mut t = Table::new(w.interior(), Box::new(model), Some(v), Some(h))
        .row_numbers(true) // столбец с номерами строк
        .frozen(1); // первый столбец не уезжает при прокрутке вправо
    t.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
    w.add(t);
    app.insert_window(w);
}
```

#### Модели данных

| Модель | Когда брать |
|--------|-------------|
| `VecTableModel::new(&["Имя", …], rows)` | данные в памяти (`Vec<Vec<String>>`); `.editable(true)` разрешает правку; сортировка уже есть |
| `FnTableModel::new(rows, cols, \|r, c\| …)` | «виртуальная» таблица: текст ячейки вычисляется функцией; `.headers(&[…])` — заголовки |
| свой тип с `TableModel` | данные в своих структурах, вычисляемые столбцы, проверка ввода |

Методы трейта `TableModel` (обязательны первые три):

| Метод | По умолчанию | Смысл |
|-------|--------------|-------|
| `row_count()`, `col_count()` | — | размеры |
| `cell(row, col) -> String` | — | текст ячейки |
| `header(col) -> String` | «A», «B», … «AA» | заголовок столбца |
| `editable(row, col) -> bool` | `false` | можно ли править ячейку |
| `set_cell(row, col, &str) -> bool` | `false` | записать значение; `false` — значение отвергнуто (ячейка не меняется) |
| `sort(col, ascending) -> bool` | `false` | отсортировать строки; `false` — модель не умеет |

Своя модель с вычисляемым столбцом и проверкой ввода:

```rust
use tvision::*;

struct Price {
    items: Vec<(String, u32, f64)>, // наименование, количество, цена
}

impl TableModel for Price {
    fn row_count(&self) -> usize {
        self.items.len()
    }
    fn col_count(&self) -> usize {
        4
    }
    fn cell(&self, row: usize, col: usize) -> String {
        let (name, qty, price) = &self.items[row];
        match col {
            0 => name.clone(),
            1 => qty.to_string(),
            2 => format!("{:.2}", price),
            _ => format!("{:.2}", *qty as f64 * price), // сумма не хранится
        }
    }
    fn header(&self, col: usize) -> String {
        ["Товар", "Кол-во", "Цена", "Сумма"][col].to_string()
    }
    fn editable(&self, _row: usize, col: usize) -> bool {
        col == 1 // править можно только количество
    }
    fn set_cell(&mut self, row: usize, _col: usize, value: &str) -> bool {
        match value.trim().parse() {
            Ok(q) => {
                self.items[row].1 = q;
                true
            }
            Err(_) => false, // не число — значение не принимается
        }
    }
}
```

Сравнение для сортировки «как у людей» (числа как числа, текст без
учёта регистра) — функция `natural_cmp(a, b)`; ею пользуется
`VecTableModel`.

#### Столбцы

Описание столбца — `TableColumn`; список передаётся `with_columns`
(недостающие столбцы получают описание по умолчанию: ширина 12, текст
влево).

```rust
use tvision::*;

fn columns() -> Vec<TableColumn> {
    vec![
        TableColumn::new(20).title("Товар").limits(8, 40), // ширина 20, от 8 до 40
        TableColumn::new(8).align(Align::Right),
        TableColumn::new(10).align(Align::Right).sortable(false),
        TableColumn::new(10).align(Align::Center).resizable(false),
        TableColumn::new(6).visible(false), // скрытый столбец
    ]
}
```

| Поле / метод | Смысл |
|--------------|-------|
| `new(width)`, `width` | ширина в ячейках экрана |
| `title(s)` | свой заголовок (иначе — `model.header(col)`) |
| `align(Align::Left / Center / Right)` | выравнивание текста |
| `limits(min, max)` | пределы ширины при изменении мышью и клавишами |
| `visible(false)` | скрыть столбец (курсор его пропускает) |
| `resizable(false)`, `sortable(false)` | запретить изменение ширины / сортировку этого столбца |

Поле `columns` у таблицы открыто: столбцы можно менять и после
создания. `auto_width(n)` подбирает ширину по заголовку и первым `n`
строкам.

#### Настройки таблицы

Методы-построители (у каждого есть одноимённое поле):

| Метод | По умолчанию | Смысл |
|-------|--------------|-------|
| `header(bool)` | да | строка заголовков |
| `grid(bool)` | да | линии `│` между столбцами и `─┼─` под заголовком |
| `row_numbers(bool)` | нет | столбец номеров строк (с `row_number_base`, обычно 1) |
| `frozen(n)` | 0 | закрепить слева первые `n` видимых столбцов |
| `selection(SelectionMode::Cell / Row)` | `Cell` | курсор — ячейка или целая строка (в режиме строки ←/→ прокручивают столбцы) |
| `multi_select(bool)` | нет | выделение нескольких строк |
| `editable(bool)` | нет | правка ячеек (если модель разрешает) |
| `sortable(bool)` | да | сортировка щелчком по заголовку |
| `resizable(bool)` | да | изменение ширины столбцов |
| `striped(bool)` | нет | «зебра»: чётные строки другим фоном |
| `with_colors(TableColors {…})` | по палитре | свои цвета ([глава 11](11-colors.md#цвета-таблицы)) |

Методы для программы:

| Метод | Что делает |
|-------|-----------|
| `cursor()` → `(строка, столбец модели)`, `set_cursor(row, col)` | курсор |
| `selected_rows()`, `select_rows(&[…])`, `clear_selection()` | выделенные строки |
| `sort_by(col, asc)`, `sort_state()` | сортировка из программы, текущая сортировка |
| `model()`, `model_mut()`, `set_model(box)` | доступ к модели, замена модели |
| `refresh()` | вызвать после изменения данных модели снаружи (поправит курсор и полосы) |
| `resize_column(col, d)` | изменить ширину столбца |
| `copy_text()` | текст для копирования (ячейка или выделенные строки через табуляцию) |
| `is_editing()` | идёт ли правка ячейки |

Данные таблицы для диалога — `Value::Int(строка с курсором)`, как у
списка.

#### Клавиши и мышь

| Клавиши | Действие |
|---------|----------|
| ↑ ↓ ← →, PgUp, PgDn | движение курсора |
| Home / End | первый / последний столбец |
| Ctrl+Home / Ctrl+End, Ctrl+PgUp / Ctrl+PgDn | первая / последняя строка |
| Shift+↑ ↓ PgUp PgDn | выделить диапазон строк (при `multi_select`) |
| Пробел | выделить / снять строку (при `multi_select`) |
| Ctrl+A | выделить всё (при `multi_select`) |
| Ctrl+← / Ctrl+→ | ширина текущего столбца |
| Ctrl+C, Ctrl+Ins | копировать в буфер обмена |
| F2 или печатный символ | правка ячейки (символ заменяет текст, F2 — правит имеющийся) |
| Enter | «выбрать»: `cm::LIST_ITEM_SELECTED`; клавиша идёт дальше, поэтому в диалоге Enter нажимает кнопку по умолчанию |

Во время правки: ←/→/Home/End/Backspace/Del, Ctrl+V и Shift+Ins —
вставка; Enter — сохранить, Esc — отменить, ↑/↓ — сохранить и перейти на
соседнюю строку, Tab — сохранить и перейти к следующему элементу.
Щелчок в другом месте и потеря фокуса тоже сохраняют правку.

Мышь: щелчок — ячейка (с Ctrl — выделить строку, с Shift — диапазон);
двойной щелчок — правка или «выбрать»; щелчок по заголовку — сортировка
(повторный — в обратном порядке, в заголовке появляется ▲ или ▼);
протяжка границы столбца в строке заголовков — ширина; колесо —
прокрутка по 3 строки, Shift+колесо — по столбцам.

#### Сообщения

Таблица рассылает соседям (`Broadcast`, `Info::Id(id таблицы)`):

| Команда | Когда |
|---------|-------|
| `cm::TABLE_CURSOR_MOVED` (57) | курсор перешёл в другую ячейку |
| `cm::TABLE_CELL_EDITED` (58) | значение ячейки принято моделью |
| `cm::LIST_ITEM_SELECTED` (56) | Enter или двойной щелчок по нередактируемой ячейке |

Своё представление в том же окне или диалоге (например, строка «Итого»)
ловит их в `handle_event` и перерисовывается. Полосы прокрутки, как и у
списка, должны лежать в той же группе, что и таблица.

#### Таблица в диалоге

```rust
use tvision::*;

/// Выбрать строку из таблицы. Возвращает номер строки или `None`.
fn choose(app: &mut Program) -> Option<usize> {
    let model = VecTableModel::new(
        &["Город", "Население"],
        vec![vec!["Москва".into(), "13 104 177".into()], vec!["Казань".into(), "1 318 604".into()]],
    );
    let mut d = Dialog::centered(44, 14, "Выбор");
    let sb = d.add(ScrollBar::new(Rect::new(40, 3, 41, 10)));
    let cols = vec![TableColumn::new(20), TableColumn::new(16).align(Align::Right)];
    let t = Table::new(Rect::new(3, 3, 40, 10), Box::new(model), Some(sb), None).with_columns(cols).selection(SelectionMode::Row);
    let tid = d.add(t);
    d.add(Label::new(Rect::new(2, 2, 20, 3), "~Г~орода:", Some(tid)));
    d.add(Button::new(Rect::new(16, 11, 28, 13), "O~K~", cm::OK, bf::DEFAULT));
    let (c, d) = app.exec_dialog(d);
    if c != cm::OK {
        return None;
    }
    d.win.group.find::<Table>(tid).map(|t| t.cursor().0)
}
```

Полный пример — `cargo run --example table`: большая вычисляемая
таблица, склад со своей моделью (правка, сортировка, «зебра»,
выделение строк) и диалог выбора города.

### Полоса прокрутки `ScrollBar`

`ScrollBar::new(bounds)` — вертикальная, если ширина 1, иначе
горизонтальная. Поля `value`, `min`, `max`, `pg_step`, `ar_step`; метод
`set_params(value, min, max, pg_step, ar_step)`.

Полоса и её «хозяин» (список, редактор, `Scroller`) общаются
сообщениями внутри общего владельца: полоса рассылает
`cm::SCROLL_BAR_CHANGED`, хозяин присылает ей `cm::SCROLL_BAR_SET` с
новыми параметрами. Поэтому полоса должна лежать **в той же группе**, что
и список, а список должен знать её id.

### История ввода `History`

```rust
use tvision::*;

fn with_history(d: &mut Dialog) {
    let mut il = InputLine::new(Rect::new(3, 2, 30, 3), 80);
    il.history_id = Some(5); // номер списка истории
    let id = d.add(il);
    d.add(History::new(Rect::new(30, 2, 33, 3), id, 5)); // значок ▐↓▌ справа
}
```

Значок истории открывает окно со списком прошлых значений этой строки;
выбор подставляется в строку. Открывается щелчком или ↓, когда строка в
фокусе. Строки запоминаются при нажатии любой кнопки диалога и при уходе
фокуса со строки.

Функции общего хранилища истории: `history_add(id, s)`,
`history_count(id)`, `history_str(id, i)`, `history_list(id)`,
`clear_history()`. В одном списке не больше `HISTORY_LIMIT` (32) строк;
повтор переносится в начало.

### Выпадающий список `Combo`

`Combo` (`TCombo` из пакета Dialog Design) — значок рядом со строкой
ввода, открывающий список готовых значений:

```rust
use tvision::*;

fn city(d: &mut Dialog) {
    let il = d.add(InputLine::new(Rect::new(3, 2, 25, 3), 30));
    let mut cb = Combo::new(Rect::new(25, 2, 26, 3), il, cbx::ONLY_LIST, &["Москва", "Тверь", "Казань"]);
    cb.activate_char('▼'); // другой значок (по умолчанию ↓)
    d.add(cb);
}
```

* `Combo::new(bounds, link, flags, items)` — обычно 1×1 или 3×1 справа от
  строки ввода; строка может быть `InputLine` или `InputLong`.
* Список сортируется без учёта регистра; `new_list(v)` — заменить.
* `activate_char(c)` — значок и клавиша вызова: для стрелок (`↓`, `▼`)
  список открывает ↓, для другого символа — сам символ.
* Флаги `cbx`:
  * `ONLY_LIST` — в строку можно ввести только значение из списка: буквы
    ищут по первым символам, пробел (Shift+пробел) листает вперёд (назад);
  * `DISPOSES_LIST` — оставлен для совместимости (память освобождается сама);
  * `NO_TRANSFER` — список не участвует в обмене данными. Без этого флага
    данные `Combo` — список строк (`Value::List` из `Value::Str`).

### Числовое поле `InputLong`

`InputLong` (`TInputLong` из пакета Dialog Design) — строка ввода для
целого числа в диапазоне:

```rust
use tvision::*;

fn number(d: &mut Dialog) {
    let n = d.add(InputLong::new(Rect::new(14, 2, 24, 3), 8, -100, 1000, il::HEX, "~С~чётчик"));
    d.add(Label::new(Rect::new(2, 2, 13, 3), "~С~чётчик", Some(n)));
}
```

* `InputLong::new(bounds, max_len, low, high, flags, label)`; `label` —
  текст для сообщения об ошибке.
* Принимает только цифры, `-` (если `low < 0`, в начале), `$` и `A–F`
  для шестнадцатеричного ввода (с `il::HEX`).
* Tab/Shift+Tab и закрытие диалога проверяют диапазон; при ошибке —
  сообщение «Значение вне диапазона…», фокус остаётся в поле.
* Флаги `il`: `HEX` — разрешён ввод `$1F`; `BLANK_EQ_ZERO` — пустое поле
  = 0; `DISPLAY_HEX` — показывать число в шестнадцатеричном виде.
* `value()`, `set_value(v)` (с подрезкой по диапазону), `range_check()`;
  данные — `Value::Int`. Функция `parse_long(s)` разбирает число так же,
  как `Val` в Turbo Pascal.

### Многострочное поле `Memo`

`Memo::new(bounds, h_scroll, v_scroll)` — текстовый редактор внутри
диалога ([глава 9](09-editor.md)); Tab в нём переходит к следующему
элементу. Данные — весь текст (`Value::Str`, строки через `\n`).

## Окна сообщений

```rust
use tvision::*;

fn messages(app: &mut Program) {
    message_box(app, "Файл сохранён.", mf::INFORMATION | mf::OK_BUTTON);

    match message_box(app, "\x03Сохранить изменения?", mf::CONFIRMATION | mf::YES_NO_CANCEL) {
        c if c == cm::YES => { /* сохранить */ }
        c if c == cm::NO => { /* не сохранять */ }
        _ => { /* отмена */ }
    }

    // Окно нужного размера, не по центру:
    message_box_rect(app, Rect::new(5, 3, 65, 13), "Длинный текст…", mf::WARNING | mf::OK_BUTTON, false);
}
```

| Функция | Что делает |
|---------|-----------|
| `message_box(app, msg, flags)` | окно 40×9 по центру; возвращает `cm::YES`/`NO`/`OK`/`CANCEL` |
| `message_box_rect(app, r, msg, flags, centered)` | окно заданного размера |
| `build_message_box(r, msg, flags) -> Dialog` | только построить (чтобы показать из представления через `ctx.exec`) |
| `input_box(app, title, label, initial, limit) -> Option<String>` | запросить строку |

Флаги `mf`: вид — `WARNING`, `ERROR`, `INFORMATION`, `CONFIRMATION`
(определяет заголовок); кнопки — `YES_BUTTON`, `NO_BUTTON`, `OK_BUTTON`,
`CANCEL_BUTTON` и сочетания `YES_NO_CANCEL`, `OK_CANCEL`. Строка,
начинающаяся с `\x03`, центрируется. Заголовки и кнопки переводятся на
русский через `locale` ([глава 13](13-locale.md)).

## Советы по раскладке

* Высота кнопки — 2 (вторая строка — тень), ширина — надпись плюс 3–4.
* Метку ставьте над полем (`y - 1`) или слева в той же строке; рамка метки
  должна вмещать текст плюс 1 ячейку.
* Элементы не должны перекрываться и залезать на рамку диалога (строку 0,
  столбец 0, последнюю строку и столбец).
* Горячие буквы в одном диалоге не должны повторяться — и с учётом
  раскладки: «~К~» (клавиша R) и «~R~» — одна и та же клавиша.
* Значок истории — 3×1 справа от строки ввода; полоса прокрутки списка —
  столбец справа от него, той же высоты.
