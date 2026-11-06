# Справочник: таблица, дерево, прокрутка

Представления для большого количества данных.

На этой странице:
[`Table`](#table--таблица) ·
[`TableModel`](#tablemodel--откуда-таблица-берёт-данные) ·
[`VecTableModel`](#vectablemodel--данные-в-памяти) ·
[`FnTableModel`](#fntablemodel--ячейки-из-функции) ·
[`TableColumn`, `Align`](#tablecolumn-и-align--столбцы) ·
[`SelectionMode`](#selectionmode) ·
[`TableColors`](#tablecolors) ·
[`TreeView`](#treeview--дерево) ·
[`TreeNode`](#treenode--узел-дерева) ·
[`TreePath`, `TreeLoader`](#treepath-и-treeloader) ·
[`CheckState`](#checkstate) ·
[`TreeGlyphs`](#treeglyphs) ·
[`TreeColors`](#treecolors) ·
[`Outline`, `Node`](#outline-и-node--дерево-из-оригинала) ·
[`Scroller`](#scroller--прокрутка-для-своих-представлений) ·
[`Terminal`](#terminal--окно-журнала)

---

## `Table` — таблица

**Что это.** Таблица со строкой заголовков, линиями между столбцами,
прокруткой, сортировкой щелчком по заголовку, правкой ячеек и выделением
строк. В оригинальном Turbo Vision её нет.

**Когда нужна.** Когда у данных несколько столбцов: контакты, файлы,
записи из базы. Для одного столбца хватит `ListBox`.

Таблица не хранит данные сама — она берёт их у **модели**
([`TableModel`](#tablemodel--откуда-таблица-берёт-данные)) и спрашивает
только видимые ячейки. Поэтому даже миллион строк показывается быстро.

```rust
use tvision::*;

fn people_window(app: &mut Program) -> ViewId {
    let mut w = Window::new(Rect::new(0, 0, 60, 16), "Люди", 1);
    let v = w.standard_scroll_bar(true);
    let h = w.standard_scroll_bar(false);
    let rows = vec![
        vec!["Анна".to_string(), "34".to_string(), "Тула".to_string()],
        vec!["Борис".to_string(), "27".to_string(), "Казань".to_string()],
    ];
    let model = VecTableModel::new(&["Имя", "Возраст", "Город"], rows);
    let mut t = Table::new(w.interior(), Box::new(model), Some(v), Some(h))
        .with_columns(vec![TableColumn::new(16), TableColumn::new(8).align(Align::Right), TableColumn::new(16)])
        .row_numbers(true) // столбец с номерами строк
        .striped(true); // строки через одну другого цвета
    t.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y; // растягиваться вместе с окном
    let id = w.add(t);
    app.insert_window(w);
    id
}

/// Текст ячейки под курсором.
fn current_cell(app: &Program, id: ViewId) -> Option<String> {
    let t = app.find::<Table>(id)?;
    let (row, col) = t.cursor();
    Some(t.model().cell(row, col))
}
```

**Как создать.** `Table::new(прямоугольник, Box::new(модель),
вертикальная_полоса, горизонтальная_полоса)`, затем настройки цепочкой.

**Настройки** (цепочкой при создании или полями потом):

| Метод (поле) | Что включает | По умолчанию |
|--------------|-------------|--------------|
| `with_columns(v)` (`columns`) | ширина и выравнивание столбцов | ширина 12 |
| `header(вкл)` (`show_header`) | строка заголовков | да |
| `grid(вкл)` (`show_grid`) | линии между столбцами | да |
| `row_numbers(вкл)` (`show_row_numbers`) | номера строк слева | нет |
| `striped(вкл)` (`zebra`) | «зебра» | нет |
| `selection(SelectionMode::Row)` (`selection_mode`) | курсор на всю строку | на ячейку |
| `sortable(вкл)` (`allow_sort`) | сортировка щелчком по заголовку | да |
| `editable(вкл)` (`allow_edit`) | правка ячеек (F2, Enter или буква), если модель разрешает | нет |
| `multi_select(вкл)` (`multi_select`) | выделение нескольких строк (Shift+стрелки, Ctrl+щелчок, пробел) | нет |
| `frozen(n)` (`frozen_cols`) | первые `n` столбцов не уезжают при прокрутке вбок | 0 |
| `resizable(вкл)` (`allow_resize`) | менять ширину столбцов мышью | да |
| `with_colors(c)` (`colors`) | свои цвета | по окну |

**Главные методы:**

| Метод | Что делает |
|-------|-----------|
| `cursor() -> (строка, столбец)`, `set_cursor(строка, столбец)` | курсор |
| `model()`, `model_mut()` | модель — чтобы прочитать или изменить данные |
| `set_model(m)` | заменить модель целиком |
| `refresh()` | **после изменения данных**: поправить курсор и прокрутку |
| `auto_width(n)` | подобрать ширину столбцов по первым `n` строкам |
| `selected_rows()` | выделенные строки (при `multi_select`) |

**Реже:** `set_columns(v)`, `resize_column(столбец, изменение)`,
`sort_by(столбец, по_возрастанию)`, `sort_state()`, `select_rows(&строки)`,
`clear_selection()`, `is_editing()`, `copy_text()`, `row_number_base`.

**Что важно знать.**

* **Таблица сама не растягивается** с окном — задайте `grow_mode`, как в
  примере.
* При сортировке щелчком таблица переставляет строки **модели**. Если вы
  храните данные ещё и отдельно (как в [уроке 9](09-contacts.md)), номера
  строк разойдутся — либо выключите сортировку (`sortable(false)`), либо
  держите данные только в модели.
* Enter таблица не забирает: в диалоге он нажмёт кнопку по умолчанию, в
  окне — вернётся к окну ([урок 9](09-contacts.md#enter-в-таблице)).
* Сообщения соседям: `cm::TABLE_CURSOR_MOVED` (курсор сдвинулся),
  `cm::TABLE_CELL_EDITED` (ячейку изменили), `cm::LIST_ITEM_SELECTED`
  (Enter или двойной щелчок).
* Ctrl+C и Ctrl+Ins копируют ячейку или выделенные строки.

Подробнее — [глава 7](../07-dialogs.md#таблица-table), пример —
`cargo run --example table`.

## `TableModel` — откуда таблица берёт данные

**Что это.** Трейт: таблица спрашивает у модели число строк и столбцов и
текст каждой видимой ячейки. Свою модель пишут, когда данные удобнее
вычислять или брать из своего хранилища, а не копировать в
`Vec<Vec<String>>`.

| Метод | Обязателен | Что вернуть |
|-------|-----------|-------------|
| `row_count()`, `col_count()` | да | размеры |
| `cell(строка, столбец)` | да | текст ячейки |
| `header(столбец)` | нет | заголовок (по умолчанию `A`, `B`, …) |
| `editable(строка, столбец)` | нет | можно ли править (по умолчанию нет) |
| `set_cell(строка, столбец, текст)` | нет | записать; `false` — не принято |
| `sort(столбец, по_возрастанию)` | нет | отсортировать строки; `false` — не умею |

```rust
use tvision::*;

/// Таблица умножения 1000×1000 — данные не хранятся, а вычисляются.
struct Multiply;

impl TableModel for Multiply {
    fn row_count(&self) -> usize {
        1000
    }
    fn col_count(&self) -> usize {
        1000
    }
    fn cell(&self, r: usize, c: usize) -> String {
        ((r + 1) * (c + 1)).to_string()
    }
    fn header(&self, c: usize) -> String {
        (c + 1).to_string()
    }
}

fn model() {
    assert_eq!(Multiply.cell(2, 3), "12");
    let _t = Table::new(Rect::new(0, 0, 40, 10), Box::new(Multiply), None, None);
}
```

## `VecTableModel` — данные в памяти

**Что это.** Готовая модель: заголовки и строки как `Vec<Vec<String>>`.
`VecTableModel::new(&["Имя", "Возраст"], строки)`; `.editable(true)` —
разрешить правку. Сортирует сама (числа — как числа). Поля `headers`,
`rows`, `editable`: после изменения `rows` через `model_mut()` вызовите
`table.refresh()`.

## `FnTableModel` — ячейки из функции

**Что это.** Самый короткий способ сделать «вычисляемую» таблицу без
своего типа: `FnTableModel::new(строк, столбцов, |строка, столбец| текст)`
и `.headers(&["…"])`.

```rust
use tvision::*;

fn squares() -> Table {
    let m = FnTableModel::new(1_000_000, 2, |r, c| if c == 0 { r.to_string() } else { (r * r).to_string() }).headers(&["n", "n²"]);
    Table::new(Rect::new(0, 0, 30, 10), Box::new(m), None, None)
}
```

**Функции модуля:** `column_letters(i)` — имя столбца как в Excel (0 →
`A`, 26 → `AA`); `natural_cmp(a, b)` — сравнение «как числа, если обе
строки — числа».

## `TableColumn` и `Align` — столбцы

`TableColumn::new(ширина)` и цепочкой: `.title("…")`,
`.align(Align::Left / Center / Right)`, `.limits(мин, макс)` (пределы
ширины при растягивании мышью), `.visible(false)` (скрыть),
`.resizable(false)`, `.sortable(false)`.

## `SelectionMode`

`Cell` — курсор на одной ячейке, ←/→ ходят по столбцам; `Row` — курсор на
всю строку, ←/→ прокручивают вбок.

## `TableColors`

Свои цвета таблицы: `normal`, `zebra`, `header`, `header_sorted`,
`cursor`, `cursor_inactive`, `selected`, `grid`, `row_number`. Обычно не
нужны — таблица берёт цвета у окна. Запасные —
`table::DEFAULT_TABLE_COLORS`.

---

## `TreeView` — дерево

**Что это.** Дерево с раскрывающимися ветками: `▸` — свёрнуто, `▾` —
раскрыто. Умеет загружать ветки только при раскрытии, флажки,
быстрый поиск набором букв. В оригинале его нет.

**Когда нужно.** Каталоги, разделы документа, структура JSON, настройки
по группам. Полное описание — [глава 18](../18-treeview.md), пример —
`cargo run --example treeview`.

```rust
use tvision::*;

fn project_tree(r: Rect) -> TreeView {
    let roots = vec![
        TreeNode::new("src")
            .expanded(true) // раскрыта сразу
            .child(TreeNode::new("main.rs"))
            .child(TreeNode::new("widgets").child(TreeNode::new("tree.rs"))),
        TreeNode::new("Cargo.toml"),
    ];
    TreeView::new(r, roots, None, None)
}

fn tree_ops() {
    let mut t = project_tree(Rect::new(0, 0, 30, 10));
    assert_eq!(t.row_count(), 4); // src, main.rs, widgets (свёрнута), Cargo.toml
    assert!(t.select_path(&[0, 1, 0])); // встать на tree.rs (widgets раскроется)
    assert_eq!(t.current().unwrap().text, "tree.rs");
    t.push(Some(&[0]), TreeNode::new("lib.rs")); // добавить в src
    assert!(t.find_text("lib.rs").is_some());
}
```

**Клавиши:** ↑↓ — по строкам; → — раскрыть или к первому вложенному; ← —
свернуть или к родителю; `+` `-` `*` — раскрыть, свернуть, раскрыть всё
внутри; Enter — раскрыть или «выбрать» (на листе); пробел — флажок;
буквы — поиск.

**Главные методы:**

| Метод | Что делает |
|-------|-----------|
| `TreeView::new(прямоугольник, узлы, вертикальная_полоса, горизонтальная_полоса)` | дерево |
| `current()`, `cursor_path()` | узел под курсором и путь к нему |
| `select_path(путь)` | поставить курсор на узел (раскрыв всё выше) |
| `node(путь)`, `node_mut(путь)` | узел по пути |
| `push(родитель, узел)`, `insert(родитель, место, узел)`, `remove(путь)` | добавить, вставить, удалить |
| `expand(путь)`, `collapse(путь)`, `expand_all()`, `collapse_all()` | раскрыть, свернуть |
| `find_text(текст)`, `find(\|узел\| …)` | найти узел |
| `refresh()` | после правки узлов через `node_mut` |

**Настройки цепочкой:** `.checkboxes(true)` — флажки; `.lines(false)` —
без линий веток; `.with_loader(функция)` — загрузка веток при раскрытии;
`.enter_toggles(false)` — Enter всегда «выбирает»; `.glyphs(…)`,
`.with_colors(…)`.

**Реже:** `roots()`, `roots_mut()`, `set_roots(v)`, `path_texts(путь)`
(тексты от корня), `for_each(f)`, `toggle`, `set_expanded`,
`expand_subtree(путь, глубина)`, `row_count()`, `row_path(i)`,
`cursor_row()`, `set_cursor_row(i)`, `check_state(путь)`,
`set_checked(путь, вкл)`, `toggle_check(путь)`, `checked_paths()`
(отмеченные листья); поля `show_lines`, `checkboxes`, `enter_toggles`,
`glyphs`, `colors`, `v_scroll`, `h_scroll`.

**Сообщения соседям:** `cm::TREE_CURSOR_MOVED`, `cm::TREE_NODE_TOGGLED`,
`cm::TREE_CHECK_CHANGED`, `cm::LIST_ITEM_SELECTED`.

## `TreeNode` — узел дерева

**Что это.** Один узел: текст, вложенные узлы и признаки. Собирается
цепочкой:

| Метод | Что задаёт |
|-------|-----------|
| `TreeNode::new(текст)` | узел |
| `.child(узел)`, `.children(vec)` | вложенные |
| `.expanded(true)` | раскрыт сразу |
| `.lazy()` | вложенные загрузит функция [`TreeLoader`](#treepath-и-treeloader) при раскрытии |
| `.checked(true)` | флажок отмечен |
| `.icon('{')` | значок перед текстом |
| `.color(цвет)` | свой цвет текста |
| `.data("…")` | ваши данные: путь к файлу, ключ записи |

Поля открыты: `text`, `children`, `expanded`, `lazy`, `checked`, `icon`,
`color`, `data`. Помощники: `has_children()`, `check_state()`,
`set_checked_deep(вкл)`, `count()`.

## `TreePath` и `TreeLoader`

**`TreePath`** — путь к узлу: номера по порядку от корня
(`Vec<usize>`). `[0]` — первый узел верхнего уровня, `[0, 2]` — его
третий вложенный.

**`TreeLoader`** — функция, которая возвращает вложенные узлы для узла с
`.lazy()`. Вызывается один раз — при первом раскрытии. Так дерево
каталогов открывается мгновенно: читается только то, что раскрыли.

```rust
use tvision::*;

/// Бесконечное дерево: у каждого узла по три вложенных, создаются по требованию.
fn infinite(r: Rect) -> TreeView {
    TreeView::new(r, vec![TreeNode::new("корень").lazy()], None, None)
        .with_loader(|_path, node| (1..=3).map(|i| TreeNode::new(&format!("{}.{}", node.text, i)).lazy()).collect())
}
```

## `CheckState`

Состояние флажка: `Checked` (отмечен), `Unchecked`, `Partial` (отмечена
только часть вложенных — показывается `[-]`).

## `TreeGlyphs`

Символы, которыми рисуется дерево: линии `│ ├ └ ─`, стрелки `▸ ▾`,
содержимое флажков. Готовые наборы: `TreeGlyphs::UNICODE` (по умолчанию)
и `TreeGlyphs::ASCII` — для терминалов без псевдографики.

## `TreeColors`

Свои цвета дерева: `normal`, `cursor`, `cursor_inactive`, `lines`.
Обычно не нужны — дерево берёт цвета у окна. Запасные —
`treeview::DEFAULT_TREE_COLORS`.

---

## `Outline` и `Node` — дерево из оригинала

**Что это.** Перевод `TOutline` из Turbo Vision — для программ,
переносимых с C++ или Pascal. Для новых программ удобнее `TreeView`.

```rust
use tvision::*;

fn outline(r: Rect) -> Outline {
    let root = Node::with_children("Проект", vec![Node::new("main.rs"), Node::with_children("docs", vec![Node::new("guide.md")], false)], true);
    Outline::new(r, None, None, vec![root])
}
```

`Node::new(текст)`, `Node::with_children(текст, вложенные, раскрыт)`;
`Outline::new(прямоугольник, горизонтальная_полоса, вертикальная_полоса,
узлы)` — обратите внимание: сначала **горизонтальная**. Поля `roots`,
`foc`; методы `update()` (после правки `roots`), `row_count()`,
`node_at(i)`, `focused_node()`, `focused_path()`, `set_expanded(i, вкл)`,
`expand_all(i)`. Служебное: флаги `outline::ov::*`, функция
`create_graph`.

---

## `Scroller` — прокрутка для своих представлений

**Что это.** Помощник для своего представления, которое больше экрана
(длинный текст, большая картинка): хранит, насколько прокручено
(`delta`), и связывает представление с полосами прокрутки.

**Аналог:** `TScroller` (там от него наследовались, здесь его кладут
полем в свою структуру).

```rust
use tvision::*;

/// Просмотр длинного текста.
pub struct TextView {
    pub vs: ViewState,
    pub scroller: Scroller,
    pub lines: Vec<String>,
}

impl TextView {
    pub fn new(r: Rect, h: Option<ViewId>, v: Option<ViewId>, lines: Vec<String>) -> Self {
        let mut vs = ViewState::new(r);
        vs.options |= of::SELECTABLE;
        let mut scroller = Scroller::new(h, v);
        let w = lines.iter().map(|l| text::str_width(l)).max().unwrap_or(0) as i32;
        scroller.set_limit(w, lines.len() as i32, r.size()); // размер содержимого
        TextView { vs, scroller, lines }
    }
}

impl View for TextView {
    tvision::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let a = dc.color(1);
        let d = self.scroller.delta; // насколько прокручено
        let w = self.vs.size.x.max(0) as usize;
        for y in 0..self.vs.size.y {
            let mut b = DrawBuffer::new(w);
            b.move_char(0, ' ', a, w);
            if let Some(l) = self.lines.get((d.y + y) as usize) {
                // строка данных = прокрутка + строка на экране
                let s: String = l.chars().skip(d.x.max(0) as usize).collect();
                b.move_str_limited(0, &s, a, w);
            }
            dc.write_line(0, y, self.vs.size.x, 1, &b);
        }
    }

    fn palette(&self) -> &'static [u8] {
        color::palettes::SCROLLER
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        view::default_handle_event(&self.vs, ev, ctx);
        let size = self.vs.size;
        self.scroller.handle(ev, size, self.vs.is_focused()); // стрелки, колесо, полосы
        self.scroller.sync(size, ctx); // передать полосам новое положение
    }
}
```

| Метод / поле | Что |
|--------------|-----|
| `Scroller::new(горизонтальная, вертикальная)` | id полос (или `None`) |
| `set_limit(ширина, высота, размер_окна)` | размер содержимого |
| `delta` | насколько прокручено (столбцов, строк) |
| `handle(ev, размер, в_фокусе)` | обработать прокрутку |
| `sync(размер, ctx)` | обновить полосы |
| `scroll_to(x, y, размер)`, `limit` | прокрутить в точку; размер содержимого |

## `Terminal` — окно журнала

**Что это.** Окно для вывода строк одна за другой, как в консоли: новые
строки добавляются снизу, старые удаляются, когда их больше заданного
числа. Удобно для журнала событий.

**Аналог:** `TTerminal`.

```rust
use tvision::*;

fn log_window(app: &mut Program) -> ViewId {
    let mut w = Window::new(Rect::new(0, 0, 60, 15), "Журнал", 0);
    let v = w.standard_scroll_bar(true);
    let log = w.add(Terminal::new(w.interior(), None, Some(v), 1000)); // хранить 1000 строк
    app.insert_window(w);
    log
}

fn log(app: &mut Program, id: ViewId, msg: &str) {
    if let Some(t) = app.find_mut::<Terminal>(id) {
        t.write_line(msg);
    }
}
```

Методы: `write_line(текст)`, `write(текст)` (`\n` — новая строка),
`clear()`, `lines()`; поля `max_lines`, `scroller`.
