# Справочник: таблица, дерево, прокрутка

Представления для больших объёмов данных. Модули `widgets::table`,
`widgets::treeview`, `widgets::outline`, `widgets::scroller`.

* [`Table`](#table) · [`TableModel`](#tablemodel) · [`VecTableModel`](#vectablemodel) · [`FnTableModel`](#fntablemodel) · [`TableColumn`, `Align`](#tablecolumn-и-align) · [`SelectionMode`](#selectionmode) · [`TableColors`](#tablecolors)
* [`TreeView`](#treeview) · [`TreeNode`](#treenode) · [`TreePath`, `TreeLoader`](#treepath-и-treeloader) · [`CheckState`](#checkstate) · [`TreeGlyphs`](#treeglyphs) · [`TreeColors`](#treecolors)
* [`Outline`, `Node`](#outline-и-node)
* [`Scroller`](#scroller) · [`Terminal`](#terminal)

---

## `Table`

**Таблица** с заголовками, сеткой, прокруткой, закреплёнными столбцами,
сортировкой, правкой ячеек и выделением строк. В оригинале её нет.
Данные таблица берёт из **модели** (`TableModel`) и запрашивает только
видимые ячейки — миллион строк не проблема.

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
    let model = VecTableModel::new(&["Имя", "Возраст", "Город"], rows).editable(true);
    let mut t = Table::new(w.interior(), Box::new(model), Some(v), Some(h))
        .with_columns(vec![TableColumn::new(16), TableColumn::new(8).align(Align::Right), TableColumn::new(16)])
        .row_numbers(true)
        .striped(true)
        .editable(true); // F2, Enter или буква — правка ячейки
    t.auto_width(100); // ширина по первым 100 строкам
    let id = w.add(t);
    app.insert_window(w);
    id
}

fn read_cell(app: &Program, id: ViewId) -> Option<String> {
    let t = app.find::<Table>(id)?;
    let (row, col) = t.cursor();
    Some(t.model().cell(row, col))
}
```

### Настройка (цепочкой и полями)

| Метод → поле | По умолчанию | Что |
|--------------|--------------|-----|
| `header(on)` → `show_header` | да | строка заголовков |
| `grid(on)` → `show_grid` | да | линии между столбцами |
| `row_numbers(on)` → `show_row_numbers`, `row_number_base` | нет, с 1 | столбец номеров строк |
| `frozen(n)` → `frozen_cols` | 0 | закрепить первые `n` столбцов слева |
| `selection(m)` → `selection_mode` | `Cell` | курсор на ячейке или на строке |
| `multi_select(on)` → `multi_select` | нет | выделение строк (Shift+стрелки, Ctrl+щелчок, пробел, Ctrl+A) |
| `editable(on)` → `allow_edit` | нет | правка ячеек (если модель разрешает) |
| `sortable(on)` → `allow_sort` | да | сортировка щелчком по заголовку |
| `resizable(on)` → `allow_resize` | да | ширина столбца мышью и Ctrl+←/→ |
| `striped(on)` → `zebra` | нет | чередование цвета строк |
| `with_colors(c)` → `colors` | по владельцу | свои цвета |
| `with_columns(v)` → `columns` | ширина 12 | описания столбцов |

### Методы

| Метод | Что |
|-------|-----|
| `Table::new(bounds, Box<dyn TableModel>, v_scroll, h_scroll)` | таблица; `h_scroll` прокручивает столбцы |
| `model()`, `model_mut()`, `set_model(m)` | модель (замена пересоздаёт столбцы) |
| `set_columns(v)`, `auto_width(n)`, `resize_column(col, d)` | столбцы |
| `refresh()` | после изменения данных модели: поправить курсор и прокрутку |
| `cursor() -> (строка, столбец)`, `set_cursor(r, c)` | курсор |
| `selected_rows()`, `select_rows(&rows)`, `clear_selection()` | выделение |
| `sort_by(col, asc) -> bool`, `sort_state()` | сортировка |
| `is_editing()`, `copy_text()` | идёт ли правка; текст для буфера обмена |

Сообщения соседям: `cm::TABLE_CURSOR_MOVED`, `cm::TABLE_CELL_EDITED`,
`cm::LIST_ITEM_SELECTED` (Enter или двойной щелчок по нередактируемой
ячейке; Enter таблица не поглощает — в диалоге он нажмёт кнопку по
умолчанию). Ctrl+C / Ctrl+Ins — копировать.

Цвета без настройки берутся у владельца: синее окно, диалог, рабочий
стол — по длине его палитры. Подробно — [глава 7](../07-dialogs.md#таблица-table),
пример — `examples/table.rs`.

## `TableModel`

**Источник данных таблицы** — трейт. Обязательны три метода.

| Метод | Что | По умолчанию |
|-------|-----|--------------|
| `row_count()`, `col_count()` | размеры | — |
| `cell(row, col) -> String` | текст ячейки (вызывается только для видимых) | — |
| `header(col)` | заголовок | `A`, `B`, … `AA` |
| `editable(row, col)` | можно ли править | нет |
| `set_cell(row, col, value) -> bool` | записать; `false` — не принято | `false` |
| `sort(col, asc) -> bool` | отсортировать строки | не умеет |

```rust
use tvision::*;

/// Таблица умножения 1000×1000 без хранения данных.
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
    let m = Multiply;
    assert_eq!(m.cell(2, 3), "12");
    let _t = Table::new(Rect::new(0, 0, 40, 10), Box::new(m), None, None);
}
```

## `VecTableModel`

**Данные в памяти**: `VecTableModel::new(&["Имя", …], rows)`, где `rows:
Vec<Vec<String>>`. `.editable(true)` разрешает правку; сортирует сама
(числа — как числа, см. `natural_cmp`). Поля `headers`, `rows`, `editable`.
После изменения `rows` через `model_mut` вызовите `table.refresh()`.

## `FnTableModel`

**Ячейки вычисляются функцией**: `FnTableModel::new(строк, столбцов,
|r, c| …).headers(&["A", "B"])`. Поля `rows`, `cols`, `headers`, `f`.
Самый короткий путь к «виртуальной» таблице.

```rust
use tvision::*;

fn squares() -> Table {
    let m = FnTableModel::new(1_000_000, 2, |r, c| if c == 0 { r.to_string() } else { (r * r).to_string() }).headers(&["n", "n²"]);
    Table::new(Rect::new(0, 0, 30, 10), Box::new(m), None, None)
}
```

Помощники модуля: `column_letters(i)` (0 → `A`, 26 → `AA`) и
`natural_cmp(a, b)` (если обе строки — числа, сравнивает как числа,
иначе — текст без учёта регистра).

## `TableColumn` и `Align`

**Описание столбца**: `TableColumn::new(ширина)` и цепочка `title(s)`,
`align(Align::Left / Center / Right)`, `limits(min, max)`,
`visible(false)` (скрытый, курсор пропускает), `resizable(false)`,
`sortable(false)`. Поля — те же (`title`, `width`, `min_width`,
`max_width`, `align`, `visible`, `resizable`, `sortable`).

## `SelectionMode`

`Cell` — курсор на ячейке, ←/→ ходят по столбцам; `Row` — курсор на
строке, ←/→ прокручивают столбцы.

## `TableColors`

**Свои цвета таблицы** (готовые атрибуты): `normal`, `zebra`, `header`,
`header_sorted`, `cursor`, `cursor_inactive`, `selected`, `grid`,
`row_number`. Запасной набор — `table::DEFAULT_TABLE_COLORS`.

---

## `TreeView`

**Дерево** с ленивой загрузкой, флажками с тремя состояниями, линиями
ветвей, быстрым поиском и правкой из программы. В оригинале его нет
(есть `TOutline` — см. [`Outline`](#outline-и-node)). Полное описание —
[глава 18](../18-treeview.md), пример — `examples/treeview.rs`.

```rust
use tvision::*;

fn project_tree(r: Rect) -> TreeView {
    let roots = vec![
        TreeNode::new("src")
            .expanded(true)
            .child(TreeNode::new("main.rs").data("src/main.rs"))
            .child(TreeNode::new("widgets").child(TreeNode::new("tree.rs"))),
        TreeNode::new("Cargo.toml"),
    ];
    TreeView::new(r, roots, None, None)
}

fn tree_ops() {
    let mut t = project_tree(Rect::new(0, 0, 30, 10));
    assert_eq!(t.row_count(), 4); // src, main.rs, widgets (свёрнут), Cargo.toml
    assert!(t.select_path(&[0, 1, 0])); // раскроет «widgets» и встанет на tree.rs
    assert_eq!(t.path_texts(&t.cursor_path().unwrap()), ["src", "widgets", "tree.rs"]);
    let p = t.push(Some(&[0]), TreeNode::new("lib.rs")).unwrap();
    assert_eq!(p, vec![0, 2]);
    t.remove(&p);
    assert_eq!(t.find_text("lib.rs"), None);
}
```

| Группа | Методы |
|--------|--------|
| создание | `TreeView::new(bounds, roots, v_scroll, h_scroll)`; цепочкой `lines(on)`, `checkboxes(on)`, `enter_toggles(on)`, `glyphs(g)`, `with_colors(c)`, `with_loader(f)` |
| узлы | `roots()`, `roots_mut()`, `set_roots(v)`, `node(path)`, `node_mut(path)` + `refresh()`, `path_texts(path)`, `find(pred)`, `find_text(s)`, `for_each(f)` |
| курсор | `cursor_path()`, `current()`, `select_path(path)`, `cursor_row()`, `set_cursor_row(i)`, `row_count()`, `row_path(i)` |
| раскрытие | `expand`, `collapse`, `toggle`, `set_expanded(path, on)`, `expand_subtree(path, глубина)`, `expand_all()`, `collapse_all()` |
| флажки | `check_state(path)`, `set_checked(path, on)`, `toggle_check(path)`, `checked_paths()` |
| правка | `insert(parent, index, node)`, `push(parent, node)`, `remove(path)` |
| поля | `show_lines`, `checkboxes`, `enter_toggles`, `glyphs`, `colors`, `v_scroll`, `h_scroll` |

Сообщения соседям: `cm::TREE_CURSOR_MOVED`, `TREE_NODE_TOGGLED`,
`TREE_CHECK_CHANGED`, `LIST_ITEM_SELECTED`.

## `TreeNode`

**Узел дерева**: собирается цепочкой `TreeNode::new(текст)` →
`.child(n)`, `.children(v)`, `.expanded(on)`, `.lazy()`, `.checked(on)`,
`.icon(c)`, `.color(attr)`, `.data(s)`. Поля открыты: `text`, `children`,
`expanded`, `lazy`, `checked`, `icon`, `color`, `data`. Помощники:
`has_children()`, `check_state()`, `set_checked_deep(on)`, `count()`.

## `TreePath` и `TreeLoader`

`TreePath` = `Vec<usize>` — путь к узлу, индексы детей от корня (`[0, 2]`
— третий ребёнок первого узла).

`TreeLoader` = `Box<dyn FnMut(&[usize], &TreeNode) -> Vec<TreeNode>>` —
функция ленивой загрузки: получает путь и узел с `lazy = true`, возвращает
детей. Задаётся `with_loader`; вызывается один раз, при первом раскрытии.

```rust
use tvision::*;

/// Бесконечное дерево: у каждого узла по три ребёнка, загружаемых по требованию.
fn infinite(r: Rect) -> TreeView {
    TreeView::new(r, vec![TreeNode::new("корень").lazy()], None, None)
        .with_loader(|path, node| (1..=3).map(|i| TreeNode::new(&format!("{}.{} (уровень {})", node.text, i, path.len()))).map(|n| n.lazy()).collect())
}
```

## `CheckState`

Состояние флажка: `Checked`, `Unchecked`, `Partial` (отмечена часть
детей). У узла с детьми вычисляется по детям.

## `TreeGlyphs`

Символы дерева: `vert` (`│`), `tee` (`├`), `corner` (`└`), `horiz` (`─`),
`collapsed` (`▸`), `expanded` (`▾`), `check_on`, `check_partial`,
`check_off`. Готовые наборы — `TreeGlyphs::UNICODE` (по умолчанию) и
`TreeGlyphs::ASCII`.

## `TreeColors`

`normal`, `cursor`, `cursor_inactive`, `lines`. Без настройки — по
палитре владельца; запасной набор — `treeview::DEFAULT_TREE_COLORS`.

---

## `Outline` и `Node`

**Дерево из оригинала** (`TOutline`, `TNode`) — для программ,
перенесённых с C++. Для новых удобнее `TreeView`.

```rust
use tvision::*;

fn outline(r: Rect) -> Outline {
    let root = Node::with_children("Проект", vec![Node::new("main.rs"), Node::with_children("docs", vec![Node::new("guide.md")], false)], true);
    Outline::new(r, None, None, vec![root])
}
```

| Поле / метод | Что |
|--------------|-----|
| `Node::new(text)`, `Node::with_children(text, children, expanded)` | узлы; поля `text`, `children`, `expanded` |
| `Outline::new(bounds, h_scroll, v_scroll, roots)` | дерево (обратите внимание: сначала горизонтальная полоса) |
| `roots`, `foc` | узлы и строка с фокусом |
| `update()` | пересчитать после правки `roots` |
| `row_count()`, `node_at(i)`, `focused_node()`, `focused_path()` | строки |
| `set_expanded(i, on)`, `expand_all(i)` | раскрытие по номеру строки |

Клавиши: стрелки, `+` `-` `*`, Enter — `cm::LIST_ITEM_SELECTED`.
Флаги рисования `outline::ov::*` и функция `create_graph` — для своей
отрисовки веток.

---

## `Scroller`

**Движок прокрутки** (`TScroller` был базовым классом, здесь — компонент,
который встраивают в своё представление): хранит смещение `delta` и
размер содержимого `limit`, связан с полосами прокрутки.

```rust
use tvision::*;

/// Просмотр длинного текста с прокруткой.
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
        scroller.set_limit(w, lines.len() as i32, r.size());
        TextView { vs, scroller, lines }
    }
}

impl View for TextView {
    tvision::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let a = dc.color(1);
        let d = self.scroller.delta;
        let w = self.vs.size.x.max(0) as usize;
        for y in 0..self.vs.size.y {
            let mut b = DrawBuffer::new(w);
            b.move_char(0, ' ', a, w);
            if let Some(l) = self.lines.get((d.y + y) as usize) {
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
        self.scroller.handle(ev, size, self.vs.is_focused()); // полосы, колесо, стрелки
        self.scroller.sync(size, ctx); // сообщить полосам новые значения
    }
}
```

| Метод / поле | Что |
|--------------|-----|
| `Scroller::new(h, v)` | движок с id полос (`None` — без полосы) |
| `delta`, `limit` | смещение и размер содержимого |
| `set_limit(x, y, size)`, `scroll_to(x, y, size)` | задать размер / прокрутить (в пределах) |
| `handle(ev, size, focused) -> bool` | обработать полосы, колесо, стрелки и PgUp/PgDn |
| `sync(size, ctx)` | отправить полосам параметры (только при изменениях) |

## `Terminal`

**Окно вывода** (`TTerminal`): журнал с кольцевым буфером строк;
прокрутка следует за концом, пока пользователь не прокрутил вверх.

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

Методы: `write(text)` (`\n` — новая строка), `write_line(text)`,
`clear()`, `lines()`; поля `max_lines`, `scroller`.
