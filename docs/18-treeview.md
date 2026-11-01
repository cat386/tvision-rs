# 18. Дерево `TreeView`

`TreeView` показывает иерархию: каталоги, разделы документа, структуру
JSON, параметры с флажками. В оригинальном Turbo Vision для этого есть
только `TOutline` (он перенесён как [`Outline`](10-std-dialogs.md#дерево-outline)).
`TreeView` — более полный виджет, которого в оригинале нет:

* **ленивая загрузка** — дети узла появляются только при первом раскрытии;
* **флажки** с тремя состояниями;
* линии ветвей или простые отступы, значки, свой цвет узла;
* быстрый поиск набором букв, прокрутка по вертикали и горизонтали, мышь;
* правка из программы: вставка, удаление, переименование, поиск, переход к
  узлу с раскрытием предков;
* цвета берутся у владельца — дерево одинаково хорошо смотрится в синем
  окне, сером диалоге и прямо на рабочем столе.

Полный пример — `cargo run --example treeview`: дерево файловой системы,
дерево для правки, просмотр JSON и диалог с флажками.

```text
└▾ Переезд в новый офис
  ├▾ Подготовка
  │ ├─ Выбрать помещение
  │ └─ Составить смету
  ├▸ Переезд
  └▸ После переезда
```

## Простое дерево в окне

```rust
use tvision::*;

fn tree_window(r: Rect) -> Window {
    let mut w = Window::new(r, "Проект", 1);
    let v = w.standard_scroll_bar(true);
    let h = w.standard_scroll_bar(false);
    let roots = vec![
        TreeNode::new("src")
            .expanded(true)
            .child(TreeNode::new("main.rs"))
            .child(TreeNode::new("widgets").child(TreeNode::new("tree.rs"))),
        TreeNode::new("Cargo.toml"),
    ];
    let tree = TreeView::new(w.interior(), roots, Some(v), Some(h));
    w.add(tree);
    w
}
```

Полосы прокрутки должны лежать в той же группе, что и дерево (как у
списка и таблицы). По умолчанию `grow_mode` дерева — `GROW_HI_X | GROW_HI_Y`:
оно растягивается вместе с окном.

## Узлы: `TreeNode`

Узел собирается цепочкой вызовов:

| Метод | Что задаёт |
|-------|-----------|
| `TreeNode::new(текст)` | узел без детей |
| `.child(n)`, `.children(vec)` | дети |
| `.expanded(true)` | раскрыт сразу |
| `.lazy()` | дети будут загружены при раскрытии (см. ниже) |
| `.checked(true)` | отметка флажка |
| `.icon('{')` | значок перед текстом |
| `.color(attr)` | свой цвет текста (кроме строки с курсором) |
| `.data("…")` | данные программы: путь файла, ключ записи. Дерево их не трогает |

Поля узла открыты (`text`, `children`, `expanded`, `lazy`, `checked`,
`icon`, `color`, `data`). Помощники: `has_children()` (есть дети или узел
ленивый), `check_state()`, `set_checked_deep(on)`, `count()` — число
узлов в поддереве.

### Путь к узлу

Узел задаётся **путём** `TreePath = Vec<usize>` — индексами детей от
корня. `[0]` — первый узел верхнего уровня, `[0, 2]` — третий ребёнок
первого. Путь меняется при вставке и удалении соседей, поэтому для
постоянной связи с данными храните ключ в `data`, а путь ищите через
`find`.

## Клавиши и мышь

| Клавиша | Действие |
|---------|----------|
| ↑ ↓ PgUp PgDn Home End | движение (Ctrl+PgUp/PgDn — в начало / в конец) |
| → | раскрыть узел; у раскрытого — к первому ребёнку |
| ← | свернуть узел; у свёрнутого — к родителю |
| `+` / `-` / `*` | раскрыть / свернуть / раскрыть всю ветку (до 8 уровней) |
| Enter | раскрыть или свернуть узел с детьми; на листе — `cm::LIST_ITEM_SELECTED` |
| пробел | флажок (если флажки включены), иначе раскрыть / свернуть |
| буквы | быстрый поиск: курсор на узел, текст которого начинается с набранного; та же буква ещё раз — следующее совпадение; пауза больше 1,2 с начинает поиск заново |
| Backspace | убрать букву поиска; без поиска — к родителю |
| Ctrl+← / Ctrl+→ | прокрутка вбок |
| Ctrl+C, Ctrl+Ins | текст узла — в буфер обмена |

Мышь: щелчок по строке ставит курсор, по стрелке `▸`/`▾` — раскрывает
или сворачивает, по флажку — отмечает. Двойной щелчок по тексту — как
Enter. Колесо прокручивает, с Shift — вбок.

Если `enter_toggles(false)`, Enter и двойной щелчок всегда посылают
`cm::LIST_ITEM_SELECTED`, а раскрывать узлы можно стрелками, `+`/`-` и
щелчком по стрелке. Enter на листе дерево не поглощает: в диалоге он ещё
нажмёт кнопку по умолчанию.

## Настройки

```rust
use tvision::*;

fn settings(r: Rect, roots: Vec<TreeNode>) -> TreeView {
    TreeView::new(r, roots, None, None)
        .lines(false)               // только отступы, без │ ├ └
        .checkboxes(true)           // флажки [X] [-] [ ]
        .enter_toggles(false)       // Enter всегда «выбирает»
        .glyphs(TreeGlyphs::ASCII)  // | + ` - вместо псевдографики
}
```

| Поле / метод | По умолчанию | Смысл |
|--------------|--------------|-------|
| `show_lines`, `lines(on)` | `true` | линии ветвей |
| `checkboxes`, `checkboxes(on)` | `false` | флажки перед текстом |
| `enter_toggles`, `enter_toggles(on)` | `true` | Enter / двойной щелчок раскрывают узел с детьми |
| `glyphs`, `glyphs(g)` | `TreeGlyphs::UNICODE` | символы линий, стрелок и флажков |
| `colors`, `with_colors(c)` | `None` | свои цвета вместо палитры владельца |

`TreeGlyphs` — символы `vert` (`│`), `tee` (`├`), `corner` (`└`),
`horiz` (`─`), `collapsed` (`▸`), `expanded` (`▾`) и содержимое флажка
`check_on` (`X`), `check_partial` (`-`), `check_off` (пробел). Готовые
наборы — `TreeGlyphs::UNICODE` и `TreeGlyphs::ASCII`.

## Ленивая загрузка

Узел с `.lazy()` показывается со стрелкой `▸`, хотя детей у него пока
нет. При первом раскрытии дерево вызывает загрузчик — функцию
`(путь, &узел) -> Vec<TreeNode>` — и запоминает результат (второй раз
загрузчик не вызывается). Пустой результат превращает узел в лист.

```rust
use std::path::Path;
use tvision::*;

/// Каталог — ленивый узел, файл — лист; полный путь — в `data`.
fn fs_node(path: &Path, name: &str) -> TreeNode {
    let n = TreeNode::new(name).data(&path.to_string_lossy());
    if path.is_dir() { n.lazy() } else { n }
}

fn fs_tree(r: Rect, root: &Path) -> TreeView {
    let mut t = TreeView::new(r, vec![fs_node(root, &root.to_string_lossy())], None, None)
        .with_loader(|_path, node| {
            let mut kids: Vec<TreeNode> = std::fs::read_dir(&node.data)
                .map(|rd| rd.filter_map(|e| e.ok()).map(|e| fs_node(&e.path(), &e.file_name().to_string_lossy())).collect())
                .unwrap_or_default();
            kids.sort_by(|a, b| b.lazy.cmp(&a.lazy).then(a.text.cmp(&b.text))); // каталоги первыми
            kids
        });
    t.expand(&[0]);
    t
}
```

Так дерево `/` открывается мгновенно: читается только то, что
пользователь раскрыл. Загрузчик выполняется в основном потоке, поэтому
очень медленный источник (сеть) лучше читать заранее в потоке, а в
загрузчике отдавать готовое.

`expand_subtree(path, max_depth)` (клавиша `*`) загружает ленивые узлы
на заданную глубину; `expand_all()` раскрывает только уже загруженное.

## Флажки

С `checkboxes(true)` перед каждым узлом рисуется флажок. Отметка
хранится в поле `checked`, но у узла с детьми **показываемое** состояние
вычисляется по детям: `[X]` — отмечены все, `[ ]` — ни одного, `[-]` —
часть. Отметка родителя отмечает или снимает всё его поддерево;
частично отмеченный узел при переключении становится отмеченным.
Ленивые дети, загруженные позже, наследуют отметку родителя.

```rust
use tvision::*;

/// Диалог «Компоненты»: что выбрал пользователь.
fn choose(app: &mut Program) -> Vec<String> {
    let roots = vec![
        TreeNode::new("Библиотека").expanded(true).child(TreeNode::new("Ядро").checked(true)).child(TreeNode::new("Справка")),
        TreeNode::new("Примеры").child(TreeNode::new("hello")).child(TreeNode::new("tvdemo")),
    ];
    let mut d = Dialog::centered(44, 14, "Компоненты");
    let sb = d.add(ScrollBar::new(Rect::new(39, 2, 40, 9)));
    let tid = d.add(TreeView::new(Rect::new(3, 2, 39, 9), roots, Some(sb), None).checkboxes(true));
    d.add(Button::new(Rect::new(16, 10, 28, 12), "O~K~", cm::OK, bf::DEFAULT));
    d.select(tid);
    let (cmd, d) = app.exec_dialog(d);
    if cmd != cm::OK {
        return Vec::new();
    }
    let t = d.win.group.find::<TreeView>(tid).unwrap();
    // «Библиотека / Ядро» для каждого отмеченного листа.
    t.checked_paths().iter().map(|p| t.path_texts(p).join(" / ")).collect()
}
```

| Метод | Что делает |
|-------|-----------|
| `check_state(path) -> CheckState` | `Checked`, `Unchecked` или `Partial` |
| `set_checked(path, on)` | отметить узел со всем поддеревом |
| `toggle_check(path)` | переключить (пробел, щелчок по флажку) |
| `checked_paths()` | пути отмеченных **листьев** — обычно именно они и нужны |

## Работа с деревом из программы

| Метод | Что делает |
|-------|-----------|
| `roots()`, `node(path)` | узлы верхнего уровня, узел по пути |
| `roots_mut()`, `node_mut(path)` + `refresh()` | правка узлов на месте; после неё — `refresh()` |
| `set_roots(vec)` | заменить всё дерево |
| `insert(parent, index, node) -> Option<TreePath>` | вставить ребёнком `parent` (`None` — верхний уровень) |
| `push(parent, node)` | добавить последним ребёнком |
| `remove(path) -> Option<TreeNode>` | удалить узел с поддеревом |
| `find(\|n\| …)`, `find_text(текст)` | первый подходящий узел (обход загруженных, включая свёрнутые) |
| `for_each(\|path, node\| …)` | обойти все загруженные узлы |
| `path_texts(path)` | тексты от корня: `["src", "widgets", "tree.rs"]` |
| `cursor_path()`, `current()` | путь и узел под курсором |
| `select_path(path)` | поставить курсор, раскрыв (и загрузив) предков |
| `cursor_row()`, `set_cursor_row(i)`, `row_count()`, `row_path(i)` | видимые строки |
| `expand(path)`, `collapse(path)`, `toggle(path)`, `set_expanded(path, on)` | раскрыть / свернуть |
| `expand_subtree(path, глубина)`, `expand_all()`, `collapse_all()` | раскрыть или свернуть много сразу |

Курсор при правке остаётся на том же узле: вставка перед ним сдвигает
его путь, а при удалении узла под курсором курсор встаёт на следующего
брата, если его нет — на предыдущего, если нет и его — на родителя.
Если курсор оказался в свёрнутой ветке, он поднимается к ближайшему
видимому предку.

Пример правки — «новый вложенный узел» из `examples/treeview.rs`:

```rust
use tvision::*;

fn add_child(app: &mut Program, tree_id: ViewId) {
    // Путь читаем до диалога: пока он открыт, дерево «взаймы» не держим.
    let Some(path) = app.find_mut::<TreeView>(tree_id).and_then(|t| t.cursor_path()) else { return };
    let Some(name) = input_box(app, "Новый узел", "~Н~азвание:", "", 60) else { return };
    let tree = app.find_mut::<TreeView>(tree_id).unwrap();
    if let Some(p) = tree.push(Some(&path), TreeNode::new(&name)) {
        tree.select_path(&p); // раскроет родителя и поставит курсор на новый узел
    }
}
```

## Сообщения

Дерево рассылает соседям по группе (`Info::Id` дерева):

| Сообщение | Когда |
|-----------|-------|
| `cm::TREE_CURSOR_MOVED` (64) | курсор встал на другой узел |
| `cm::TREE_NODE_TOGGLED` (65) | узел раскрыт или свёрнут |
| `cm::TREE_CHECK_CHANGED` (66) | изменились флажки |
| `cm::LIST_ITEM_SELECTED` (56) | Enter или двойной щелчок по листу |

Их ловит **соседнее** представление в том же окне или диалоге (как у
таблицы). Само окно сообщений своих детей не получает; если сведения об
узле показывает окно, проще сверять курсор после каждого события — так
сделано в окне «Файлы» примера:

```rust
use tvision::*;

struct TreeWindow {
    win: Window,
    tree: ViewId,
    info: ViewId,
    last: Option<TreePath>,
}

impl TreeWindow {
    fn handle(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        self.win.handle_window_event(ev, ctx);
        let t = self.win.group.find::<TreeView>(self.tree).unwrap();
        let path = t.cursor_path();
        if path != self.last {
            let text = t.current().map(|n| n.data.clone()).unwrap_or_default();
            self.win.group.find_mut::<StaticText>(self.info).unwrap().text = text;
            self.last = path;
        }
    }
}
```

## Цвета

У дерева пустая палитра: номера цветов идут прямо к владельцу, и дерево
понимает, где находится (как [таблица](07-dialogs.md)):

| Где | обычный | курсор | курсор без фокуса | линии |
|-----|---------|--------|-------------------|-------|
| в окне | 6 | 7 | 4 | 1 |
| в диалоге | 26 | 27 | 28 | 29 |
| прямо на рабочем столе | 13 | 14 | 11 | 8 |

Свои цвета — `TreeColors { normal, cursor, cursor_inactive, lines }` и
`with_colors(c)`; запасной набор — `DEFAULT_TREE_COLORS`. Цвет
отдельного узла — `TreeNode::color(attr)`.

## `TreeView` или `Outline`

| | `Outline` | `TreeView` |
|-|-----------|------------|
| происхождение | порт `TOutline` | нет в оригинале |
| ленивая загрузка | нет | да |
| флажки | нет | три состояния |
| значки, свой цвет узла, данные | нет | да |
| быстрый поиск | нет | да |
| правка из программы | только через `roots` | `insert`, `remove`, `select_path`… |
| ← / → | вверх / вниз по строкам | свернуть / раскрыть, к родителю / ребёнку |

`Outline` оставлен для совместимости с программами, перенесёнными с C++;
для новых программ удобнее `TreeView`.
