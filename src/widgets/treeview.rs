//! Дерево `TreeView` — современная замена `Outline` (в оригинале Turbo Vision
//! такого виджета нет; `TOutline` перенесён отдельно как [`crate::Outline`]).
//!
//! Что умеет:
//! * узлы с текстом, значком, своим цветом и пользовательскими данными;
//! * **ленивая загрузка**: у узла с `lazy = true` дети появляются только при
//!   первом раскрытии — их возвращает функция-загрузчик (так строится дерево
//!   файловой системы, где обход всего диска занял бы минуты);
//! * линии ветвей (`├`, `└`, `│`) или простые отступы, свои символы;
//! * **флажки** с тремя состояниями: у родителя «частично», если отмечена
//!   только часть детей; отметка родителя отмечает всё поддерево;
//! * быстрый поиск набором букв, горизонтальная и вертикальная прокрутка,
//!   мышь (щелчок по стрелке — раскрыть, по флажку — отметить, двойной
//!   щелчок — раскрыть или выбрать), колесо;
//! * правка дерева из программы: вставка, удаление, поиск узла, переход к
//!   узлу по пути с раскрытием предков;
//! * цвета берутся у владельца (как у [`crate::Table`]): синее окно, серый
//!   диалог, рабочий стол — без настройки.
//!
//! Сообщения владельцу (`Info::Id` дерева):
//! * `cm::TREE_CURSOR_MOVED` — курсор встал на другой узел;
//! * `cm::TREE_NODE_TOGGLED` — узел раскрыт или свёрнут;
//! * `cm::TREE_CHECK_CHANGED` — изменились флажки;
//! * `cm::LIST_ITEM_SELECTED` — Enter или двойной щелчок по листу (или по
//!   любому узлу, если `enter_toggles = false`).
//!
//! Узел задаётся **путём** — индексами детей от корня: `[0]` — первый узел
//! верхнего уровня, `[0, 2]` — его третий ребёнок.

use std::time::{Duration, Instant};

use crate::color::Attr;
use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Info, Key, Mods};
use crate::geometry::Rect;
use crate::text::{char_width, str_width};
use crate::view::{default_handle_event, gf, of, Ctx, DrawCtx, View, ViewId, ViewState};

/// Путь к узлу: индексы детей от корня.
pub type TreePath = Vec<usize>;

/// Функция ленивой загрузки: получает путь и сам узел, возвращает детей.
pub type TreeLoader = Box<dyn FnMut(&[usize], &TreeNode) -> Vec<TreeNode>>;

// ======================================================================
// Узел
// ======================================================================

/// Узел дерева.
///
/// ```
/// use tvision::TreeNode;
/// let n = TreeNode::new("src")
///     .icon('▣')
///     .expanded(true)
///     .child(TreeNode::new("main.rs").data("src/main.rs"))
///     .child(TreeNode::new("lib.rs"));
/// assert_eq!(n.children.len(), 2);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TreeNode {
    pub text: String,
    pub children: Vec<TreeNode>,
    /// Узел раскрыт (у листа не важно).
    pub expanded: bool,
    /// Дети ещё не загружены: узел показывается со стрелкой «▸», а при
    /// первом раскрытии дети берутся у загрузчика ([`TreeView::with_loader`]).
    pub lazy: bool,
    /// Отметка флажка. У узла с детьми показываемое состояние вычисляется
    /// по детям ([`CheckState`]).
    pub checked: bool,
    /// Значок перед текстом.
    pub icon: Option<char>,
    /// Свой цвет текста (кроме строки с курсором).
    pub color: Option<Attr>,
    /// Данные программы (путь файла, ключ записи…). Дерево их не трогает.
    pub data: String,
}

impl TreeNode {
    /// Узел с текстом (без детей).
    pub fn new(text: &str) -> Self {
        TreeNode { text: text.into(), ..Default::default() }
    }

    /// Добавить ребёнка.
    pub fn child(mut self, n: TreeNode) -> Self {
        self.children.push(n);
        self
    }

    /// Задать всех детей.
    pub fn children(mut self, c: Vec<TreeNode>) -> Self {
        self.children = c;
        self
    }

    pub fn expanded(mut self, on: bool) -> Self {
        self.expanded = on;
        self
    }

    /// Дети будут загружены при первом раскрытии.
    pub fn lazy(mut self) -> Self {
        self.lazy = true;
        self
    }

    pub fn checked(mut self, on: bool) -> Self {
        self.checked = on;
        self
    }

    pub fn icon(mut self, c: char) -> Self {
        self.icon = Some(c);
        self
    }

    pub fn color(mut self, a: Attr) -> Self {
        self.color = Some(a);
        self
    }

    pub fn data(mut self, d: &str) -> Self {
        self.data = d.into();
        self
    }

    /// Есть ли (или могут ли появиться) дети — показывать ли стрелку.
    pub fn has_children(&self) -> bool {
        !self.children.is_empty() || self.lazy
    }

    /// Состояние флажка с учётом детей.
    pub fn check_state(&self) -> CheckState {
        if self.children.is_empty() {
            return if self.checked { CheckState::Checked } else { CheckState::Unchecked };
        }
        let mut any_on = false;
        let mut any_off = false;
        for c in &self.children {
            match c.check_state() {
                CheckState::Checked => any_on = true,
                CheckState::Unchecked => any_off = true,
                CheckState::Partial => return CheckState::Partial,
            }
            if any_on && any_off {
                return CheckState::Partial;
            }
        }
        if any_on {
            CheckState::Checked
        } else {
            CheckState::Unchecked
        }
    }

    /// Отметить узел и всё загруженное поддерево.
    pub fn set_checked_deep(&mut self, on: bool) {
        self.checked = on;
        for c in &mut self.children {
            c.set_checked_deep(on);
        }
    }

    /// Число узлов в поддереве (вместе с самим узлом, только загруженные).
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(|c| c.count()).sum::<usize>()
    }
}

/// Состояние флажка.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckState {
    Unchecked,
    Checked,
    /// Отмечена часть детей.
    Partial,
}

// ======================================================================
// Внешний вид
// ======================================================================

/// Символы для рисования дерева.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeGlyphs {
    /// Вертикальная линия предка: `│`.
    pub vert: char,
    /// Ответвление к не последнему ребёнку: `├`.
    pub tee: char,
    /// Ответвление к последнему ребёнку: `└`.
    pub corner: char,
    /// Горизонтальная линия у листа: `─`.
    pub horiz: char,
    /// Свёрнутый узел: `▸`.
    pub collapsed: char,
    /// Раскрытый узел: `▾`.
    pub expanded: char,
    /// Внутри флажка: отмечен, частично, не отмечен.
    pub check_on: char,
    pub check_partial: char,
    pub check_off: char,
}

impl TreeGlyphs {
    /// Символы по умолчанию (псевдографика Unicode).
    pub const UNICODE: TreeGlyphs =
        TreeGlyphs {
            vert: '│', tee: '├', corner: '└', horiz: '─', collapsed: '▸', expanded: '▾', check_on: 'X', check_partial: '-', check_off: ' '
        };
    /// Только ASCII — для терминалов без псевдографики.
    pub const ASCII: TreeGlyphs =
        TreeGlyphs { vert: '|', tee: '+', corner: '`', horiz: '-', collapsed: '+', expanded: '-', check_on: 'X', check_partial: '-', check_off: ' ' };
}

impl Default for TreeGlyphs {
    fn default() -> Self {
        TreeGlyphs::UNICODE
    }
}

/// Цвета дерева.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeColors {
    pub normal: Attr,
    /// Курсор, когда дерево в фокусе.
    pub cursor: Attr,
    /// Курсор, когда дерево не в фокусе.
    pub cursor_inactive: Attr,
    /// Линии ветвей и стрелки.
    pub lines: Attr,
}

/// Цвета на случай, если у владельца нет подходящей палитры.
pub const DEFAULT_TREE_COLORS: TreeColors = TreeColors { normal: 0x1E, cursor: 0x2F, cursor_inactive: 0x30, lines: 0x17 };

/// Сколько ждать следующей буквы быстрого поиска, прежде чем начать заново.
const SEARCH_TIMEOUT: Duration = Duration::from_millis(1200);

/// Видимая строка дерева.
#[derive(Clone, Debug, PartialEq)]
struct Row {
    path: TreePath,
    /// `guides[l]` — рисовать вертикаль на уровне `l` (предок не последний).
    guides: Vec<bool>,
    last: bool,
}

impl Row {
    fn level(&self) -> usize {
        self.path.len() - 1
    }
}

// ======================================================================
// Представление
// ======================================================================

/// Дерево с прокруткой, ленивой загрузкой и флажками.
pub struct TreeView {
    pub vs: ViewState,
    roots: Vec<TreeNode>,
    pub v_scroll: Option<ViewId>,
    pub h_scroll: Option<ViewId>,

    // --- настройки ---
    /// Рисовать линии ветвей (иначе — только отступы).
    pub show_lines: bool,
    /// Флажки перед текстом.
    pub checkboxes: bool,
    /// Enter и двойной щелчок по узлу с детьми раскрывают/сворачивают его,
    /// а `cm::LIST_ITEM_SELECTED` посылается только для листьев. Если
    /// `false` — сообщение посылается всегда, раскрытие — стрелками, `+`/`-`.
    pub enter_toggles: bool,
    pub glyphs: TreeGlyphs,
    /// Свои цвета (`None` — по палитре владельца).
    pub colors: Option<TreeColors>,
    loader: Option<TreeLoader>,

    // --- состояние ---
    rows: Vec<Row>,
    cursor: usize,
    top: usize,
    /// Горизонтальный сдвиг в ячейках.
    left: usize,
    /// Ширина самой длинной строки.
    max_width: usize,
    search: String,
    search_at: Option<Instant>,
    /// Сообщения, накопленные методами без `Ctx` (раскрытие, флажки).
    pending: Vec<u16>,
    last_sent: Option<(usize, usize, usize, usize, i32, i32)>,
    last_cursor: Option<TreePath>,
}

impl TreeView {
    /// Дерево из узлов верхнего уровня. `v_scroll`, `h_scroll` — id полос
    /// прокрутки в той же группе (или `None`).
    pub fn new(bounds: Rect, roots: Vec<TreeNode>, v_scroll: Option<ViewId>, h_scroll: Option<ViewId>) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.options |= of::SELECTABLE | of::FIRST_CLICK;
        vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
        let mut t = TreeView {
            vs,
            roots,
            v_scroll,
            h_scroll,
            show_lines: true,
            checkboxes: false,
            enter_toggles: true,
            glyphs: TreeGlyphs::UNICODE,
            colors: None,
            loader: None,
            rows: Vec::new(),
            cursor: 0,
            top: 0,
            left: 0,
            max_width: 0,
            search: String::new(),
            search_at: None,
            pending: Vec::new(),
            last_sent: None,
            last_cursor: None,
        };
        t.rebuild();
        t
    }

    // --- настройка цепочкой вызовов ---

    pub fn lines(mut self, on: bool) -> Self {
        self.show_lines = on;
        self.rebuild();
        self
    }

    pub fn checkboxes(mut self, on: bool) -> Self {
        self.checkboxes = on;
        self.rebuild();
        self
    }

    pub fn enter_toggles(mut self, on: bool) -> Self {
        self.enter_toggles = on;
        self
    }

    pub fn glyphs(mut self, g: TreeGlyphs) -> Self {
        self.glyphs = g;
        self
    }

    pub fn with_colors(mut self, c: TreeColors) -> Self {
        self.colors = Some(c);
        self
    }

    /// Функция ленивой загрузки детей (для узлов с `lazy = true`).
    pub fn with_loader(mut self, f: impl FnMut(&[usize], &TreeNode) -> Vec<TreeNode> + 'static) -> Self {
        self.loader = Some(Box::new(f));
        self
    }

    // ------------------------------------------------------------------
    // Доступ к узлам
    // ------------------------------------------------------------------

    /// Узлы верхнего уровня.
    pub fn roots(&self) -> &[TreeNode] {
        &self.roots
    }

    /// Узлы для правки «на месте». После изменений вызовите [`refresh`](Self::refresh).
    pub fn roots_mut(&mut self) -> &mut Vec<TreeNode> {
        &mut self.roots
    }

    /// Заменить всё дерево (курсор — на первый узел).
    pub fn set_roots(&mut self, roots: Vec<TreeNode>) {
        self.roots = roots;
        self.cursor = 0;
        self.top = 0;
        self.left = 0;
        self.rebuild();
    }

    /// Узел по пути.
    pub fn node(&self, path: &[usize]) -> Option<&TreeNode> {
        let (first, rest) = path.split_first()?;
        let mut n = self.roots.get(*first)?;
        for &i in rest {
            n = n.children.get(i)?;
        }
        Some(n)
    }

    /// Узел по пути для правки. После изменений вызовите [`refresh`](Self::refresh).
    pub fn node_mut(&mut self, path: &[usize]) -> Option<&mut TreeNode> {
        let (first, rest) = path.split_first()?;
        let mut n = self.roots.get_mut(*first)?;
        for &i in rest {
            n = n.children.get_mut(i)?;
        }
        Some(n)
    }

    /// Тексты узлов от корня до `path`: `["src", "widgets", "tree.rs"]`.
    pub fn path_texts(&self, path: &[usize]) -> Vec<String> {
        (1..=path.len()).filter_map(|n| self.node(&path[..n]).map(|x| x.text.clone())).collect()
    }

    /// Первый узел (обход в глубину по загруженным узлам, включая свёрнутые),
    /// для которого `pred` истинно.
    pub fn find(&self, pred: impl Fn(&TreeNode) -> bool) -> Option<TreePath> {
        fn rec(nodes: &[TreeNode], prefix: &mut TreePath, pred: &dyn Fn(&TreeNode) -> bool) -> Option<TreePath> {
            for (i, n) in nodes.iter().enumerate() {
                prefix.push(i);
                if pred(n) {
                    return Some(prefix.clone());
                }
                if let Some(p) = rec(&n.children, prefix, pred) {
                    return Some(p);
                }
                prefix.pop();
            }
            None
        }
        rec(&self.roots, &mut Vec::new(), &pred)
    }

    /// Найти узел по точному тексту.
    pub fn find_text(&self, text: &str) -> Option<TreePath> {
        self.find(|n| n.text == text)
    }

    /// Обойти все загруженные узлы: `f(путь, узел)`.
    pub fn for_each(&self, mut f: impl FnMut(&[usize], &TreeNode)) {
        fn rec(nodes: &[TreeNode], prefix: &mut TreePath, f: &mut dyn FnMut(&[usize], &TreeNode)) {
            for (i, n) in nodes.iter().enumerate() {
                prefix.push(i);
                f(prefix, n);
                rec(&n.children, prefix, f);
                prefix.pop();
            }
        }
        rec(&self.roots, &mut Vec::new(), &mut f);
    }

    // ------------------------------------------------------------------
    // Курсор
    // ------------------------------------------------------------------

    /// Число видимых строк.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Путь узла в видимой строке `i`.
    pub fn row_path(&self, i: usize) -> Option<&[usize]> {
        self.rows.get(i).map(|r| r.path.as_slice())
    }

    /// Номер строки с курсором.
    pub fn cursor_row(&self) -> usize {
        self.cursor
    }

    /// Путь узла под курсором.
    pub fn cursor_path(&self) -> Option<TreePath> {
        self.rows.get(self.cursor).map(|r| r.path.clone())
    }

    /// Узел под курсором.
    pub fn current(&self) -> Option<&TreeNode> {
        self.rows.get(self.cursor).and_then(|r| self.node(&r.path))
    }

    /// Поставить курсор на строку `i`.
    pub fn set_cursor_row(&mut self, i: usize) {
        self.cursor = i.min(self.rows.len().saturating_sub(1));
        self.ensure_visible();
    }

    /// Поставить курсор на узел, раскрыв (и при необходимости загрузив) всех
    /// его предков. `false` — такого узла нет.
    pub fn select_path(&mut self, path: &[usize]) -> bool {
        if path.is_empty() {
            return false;
        }
        for n in 1..path.len() {
            let p = path[..n].to_vec();
            if self.node(&p).is_none() {
                return false;
            }
            self.load_if_needed(&p);
            match self.node_mut(&p) {
                Some(x) if !x.expanded => {
                    x.expanded = true;
                    self.pending.push(cm::TREE_NODE_TOGGLED);
                }
                _ => {}
            }
        }
        if self.node(path).is_none() {
            self.rebuild();
            return false;
        }
        self.rebuild();
        match self.rows.iter().position(|r| r.path == path) {
            Some(i) => {
                self.cursor = i;
                self.ensure_visible();
                true
            }
            None => false,
        }
    }

    // ------------------------------------------------------------------
    // Раскрытие
    // ------------------------------------------------------------------

    /// Загрузить детей ленивого узла (если ещё не загружены).
    fn load_if_needed(&mut self, path: &[usize]) {
        let lazy = matches!(self.node(path), Some(n) if n.lazy);
        if !lazy {
            return;
        }
        let kids = match (self.loader.take(), self.node(path)) {
            (Some(mut f), Some(n)) => {
                let k = f(path, n);
                self.loader = Some(f);
                k
            }
            (f, _) => {
                self.loader = f;
                Vec::new()
            }
        };
        if let Some(n) = self.node_mut(path) {
            n.lazy = false;
            // Новые дети наследуют отметку родителя.
            let on = n.checked;
            n.children = kids;
            if on {
                n.set_checked_deep(true);
            }
        }
    }

    /// Раскрыть или свернуть узел.
    pub fn set_expanded(&mut self, path: &[usize], on: bool) {
        if on {
            self.load_if_needed(path);
        }
        if let Some(n) = self.node_mut(path) {
            if n.expanded != on && n.has_children() {
                n.expanded = on;
                self.pending.push(cm::TREE_NODE_TOGGLED);
            }
        }
        self.rebuild();
    }

    pub fn expand(&mut self, path: &[usize]) {
        self.set_expanded(path, true);
    }

    pub fn collapse(&mut self, path: &[usize]) {
        self.set_expanded(path, false);
    }

    /// Раскрыть, если свёрнут, и наоборот.
    pub fn toggle(&mut self, path: &[usize]) {
        let on = self.node(path).map(|n| !n.expanded).unwrap_or(false);
        self.set_expanded(path, on);
    }

    /// Раскрыть узел и всех его потомков. Ленивые узлы загружаются — для
    /// дерева файловой системы это может быть долго, поэтому глубина
    /// ограничена `max_depth` уровнями.
    pub fn expand_subtree(&mut self, path: &[usize], max_depth: usize) {
        fn rec(t: &mut TreeView, p: &mut TreePath, depth: usize) {
            if depth == 0 {
                return;
            }
            t.load_if_needed(p);
            let n = match t.node_mut(p) {
                Some(n) => n,
                None => return,
            };
            if !n.children.is_empty() {
                n.expanded = true;
            }
            let count = n.children.len();
            for i in 0..count {
                p.push(i);
                rec(t, p, depth - 1);
                p.pop();
            }
        }
        rec(self, &mut path.to_vec(), max_depth);
        self.pending.push(cm::TREE_NODE_TOGGLED);
        self.rebuild();
    }

    /// Раскрыть всё загруженное дерево (ленивые узлы не загружаются).
    pub fn expand_all(&mut self) {
        fn rec(nodes: &mut [TreeNode]) {
            for n in nodes {
                if !n.children.is_empty() {
                    n.expanded = true;
                    rec(&mut n.children);
                }
            }
        }
        rec(&mut self.roots);
        self.pending.push(cm::TREE_NODE_TOGGLED);
        self.rebuild();
    }

    /// Свернуть всё дерево.
    pub fn collapse_all(&mut self) {
        fn rec(nodes: &mut [TreeNode]) {
            for n in nodes {
                n.expanded = false;
                rec(&mut n.children);
            }
        }
        rec(&mut self.roots);
        self.pending.push(cm::TREE_NODE_TOGGLED);
        self.rebuild();
    }

    // ------------------------------------------------------------------
    // Флажки
    // ------------------------------------------------------------------

    /// Состояние флажка узла.
    pub fn check_state(&self, path: &[usize]) -> CheckState {
        self.node(path).map(|n| n.check_state()).unwrap_or(CheckState::Unchecked)
    }

    /// Отметить узел вместе со всем загруженным поддеревом.
    pub fn set_checked(&mut self, path: &[usize], on: bool) {
        if let Some(n) = self.node_mut(path) {
            n.set_checked_deep(on);
            self.pending.push(cm::TREE_CHECK_CHANGED);
        }
        self.sync_parent_checks();
    }

    /// Переключить флажок: частично отмеченный узел становится отмеченным.
    pub fn toggle_check(&mut self, path: &[usize]) {
        let on = self.check_state(path) != CheckState::Checked;
        self.set_checked(path, on);
    }

    /// Пути отмеченных листьев (узлов без загруженных детей) — то, что
    /// обычно нужно программе: файлы для копирования, выбранные компоненты.
    pub fn checked_paths(&self) -> Vec<TreePath> {
        let mut out = Vec::new();
        self.for_each(|p, n| {
            if n.children.is_empty() && n.checked {
                out.push(p.to_vec());
            }
        });
        out
    }

    /// Поле `checked` у родителей приводится в соответствие с детьми, чтобы
    /// ленивые дети, загруженные позже, унаследовали правильную отметку.
    fn sync_parent_checks(&mut self) {
        fn rec(nodes: &mut [TreeNode]) {
            for n in nodes {
                if !n.children.is_empty() {
                    rec(&mut n.children);
                    n.checked = n.check_state() == CheckState::Checked;
                }
            }
        }
        rec(&mut self.roots);
    }

    // ------------------------------------------------------------------
    // Правка дерева
    // ------------------------------------------------------------------

    /// Вставить узел ребёнком `parent` (`None` — на верхний уровень) на место
    /// `index` (больше числа детей — в конец). Возвращает путь нового узла.
    pub fn insert(&mut self, parent: Option<&[usize]>, index: usize, node: TreeNode) -> Option<TreePath> {
        let cur = self.cursor_path();
        let list = match parent {
            None => &mut self.roots,
            Some(p) => &mut self.node_mut(p)?.children,
        };
        let i = index.min(list.len());
        list.insert(i, node);
        let mut path = parent.map(|p| p.to_vec()).unwrap_or_default();
        path.push(i);
        // Курсор остаётся на том же узле: сдвигаем его путь, если вставка была перед ним.
        let cur = cur.map(|mut c| {
            let d = path.len() - 1;
            if c.len() > d && c[..d] == path[..d] && c[d] >= i {
                c[d] += 1;
            }
            c
        });
        self.rebuild_keep(cur);
        Some(path)
    }

    /// Добавить узел последним ребёнком `parent` (`None` — на верхний уровень).
    pub fn push(&mut self, parent: Option<&[usize]>, node: TreeNode) -> Option<TreePath> {
        self.insert(parent, usize::MAX, node)
    }

    /// Удалить узел вместе с поддеревом. Курсор переходит на следующего брата,
    /// а если его нет — на предыдущего или на родителя.
    pub fn remove(&mut self, path: &[usize]) -> Option<TreeNode> {
        let (last, parent) = path.split_last()?;
        let cur = self.cursor_path();
        let list = if parent.is_empty() { &mut self.roots } else { &mut self.node_mut(parent)?.children };
        if *last >= list.len() {
            return None;
        }
        let removed = list.remove(*last);
        let left = list.len();
        let cur = cur.map(|mut c| {
            let d = parent.len();
            if c.len() > d && c[..d] == *parent {
                if c[d] == *last {
                    // Удалён узел курсора (или его предок).
                    c.truncate(d + 1);
                    if left == 0 {
                        c.truncate(d);
                    } else if c[d] >= left {
                        c[d] = left - 1;
                    }
                } else if c[d] > *last {
                    c[d] -= 1;
                }
            }
            c
        });
        self.rebuild_keep(cur);
        Some(removed)
    }

    /// Пересобрать видимые строки после правки узлов (курсор остаётся на
    /// том же пути, а если узел скрыт — встаёт на ближайшего видимого предка).
    pub fn refresh(&mut self) {
        self.sync_parent_checks();
        self.rebuild();
    }

    // ------------------------------------------------------------------
    // Внутреннее: строки и прокрутка
    // ------------------------------------------------------------------

    fn rebuild(&mut self) {
        let cur = self.cursor_path();
        self.rebuild_keep(cur);
    }

    fn rebuild_keep(&mut self, cur: Option<TreePath>) {
        fn collect(nodes: &[TreeNode], prefix: &mut TreePath, guides: &mut Vec<bool>, out: &mut Vec<Row>) {
            for (i, n) in nodes.iter().enumerate() {
                let last = i + 1 == nodes.len();
                prefix.push(i);
                out.push(Row { path: prefix.clone(), guides: guides.clone(), last });
                if n.expanded && !n.children.is_empty() {
                    guides.push(!last);
                    collect(&n.children, prefix, guides, out);
                    guides.pop();
                }
                prefix.pop();
            }
        }
        let mut rows = Vec::new();
        collect(&self.roots, &mut Vec::new(), &mut Vec::new(), &mut rows);
        self.rows = rows;
        self.max_width = (0..self.rows.len()).map(|i| self.row_width(i)).max().unwrap_or(0);
        // Курсор: тот же путь или ближайший видимый предок.
        if let Some(mut p) = cur {
            while !p.is_empty() {
                if let Some(i) = self.rows.iter().position(|r| r.path == p) {
                    self.cursor = i;
                    break;
                }
                p.pop();
            }
        }
        self.cursor = self.cursor.min(self.rows.len().saturating_sub(1));
        self.left = self.left.min(self.max_left());
        self.ensure_visible();
    }

    /// Ширина отступа и стрелки строки: 2 ячейки на уровень + ответвление,
    /// стрелка и пробел.
    fn prefix_width(r: &Row) -> usize {
        r.level() * 2 + 3
    }

    /// Ширина строки целиком.
    fn row_width(&self, i: usize) -> usize {
        let r = &self.rows[i];
        let n = match self.node(&r.path) {
            Some(n) => n,
            None => return 0,
        };
        let mut w = Self::prefix_width(r) + str_width(&n.text);
        if self.checkboxes {
            w += 4;
        }
        if let Some(c) = n.icon {
            w += char_width(c) + 1;
        }
        w
    }

    fn page(&self) -> usize {
        self.vs.size.y.max(1) as usize
    }

    fn max_left(&self) -> usize {
        self.max_width.saturating_sub(self.vs.size.x.max(0) as usize)
    }

    fn ensure_visible(&mut self) {
        let page = self.page();
        if self.cursor < self.top {
            self.top = self.cursor;
        } else if self.cursor >= self.top + page {
            self.top = self.cursor + 1 - page;
        }
        let max_top = self.rows.len().saturating_sub(page);
        self.top = self.top.min(max_top);
    }

    fn move_cursor(&mut self, to: i64) {
        let n = self.rows.len() as i64;
        if n == 0 {
            return;
        }
        self.cursor = to.clamp(0, n - 1) as usize;
        self.ensure_visible();
    }

    fn scroll_h(&mut self, d: i64) {
        self.left = (self.left as i64 + d).clamp(0, self.max_left() as i64) as usize;
    }

    /// Цвета: свои или по палитре владельца (как у таблицы).
    fn palette_colors(&self, dc: &DrawCtx) -> TreeColors {
        if let Some(c) = self.colors {
            return c;
        }
        let pick = |n: [u8; 4]| TreeColors { normal: dc.color(n[0]), cursor: dc.color(n[1]), cursor_inactive: dc.color(n[2]), lines: dc.color(n[3]) };
        // [обычный, курсор, курсор без фокуса, линии]
        match dc.owner_palette_len() {
            // Окно: 6 — текст, 7 — выделенный текст, 4 — полоса прокрутки, 1 — пассивная рамка.
            8 => pick([6, 7, 4, 1]),
            // Диалог: 26–29 — цвета списка.
            32 => pick([26, 27, 28, 29]),
            // Прямо на рабочем столе — номера синего окна в палитре приложения.
            n if n >= 100 => pick([13, 14, 11, 8]),
            _ => DEFAULT_TREE_COLORS,
        }
    }

    /// Куски строки `i`: (текст, атрибут). Отступ и стрелка — цветом линий,
    /// остальное — `text_attr`.
    fn segments(&self, i: usize, text_attr: Attr, lines_attr: Attr) -> Vec<(String, Attr)> {
        let r = &self.rows[i];
        let n = match self.node(&r.path) {
            Some(n) => n,
            None => return Vec::new(),
        };
        let g = &self.glyphs;
        let mut prefix = String::new();
        for &v in &r.guides {
            prefix.push(if self.show_lines && v { g.vert } else { ' ' });
            prefix.push(' ');
        }
        prefix.push(if self.show_lines {
            if r.last {
                g.corner
            } else {
                g.tee
            }
        } else {
            ' '
        });
        prefix.push(if n.has_children() {
            if n.expanded && !n.children.is_empty() {
                g.expanded
            } else {
                g.collapsed
            }
        } else if self.show_lines {
            g.horiz
        } else {
            ' '
        });
        prefix.push(' ');
        let mut out = vec![(prefix, lines_attr)];
        let mut rest = String::new();
        if self.checkboxes {
            let c = match n.check_state() {
                CheckState::Checked => g.check_on,
                CheckState::Partial => g.check_partial,
                CheckState::Unchecked => g.check_off,
            };
            rest.push('[');
            rest.push(c);
            rest.push_str("] ");
        }
        if let Some(ic) = n.icon {
            rest.push(ic);
            rest.push(' ');
        }
        rest.push_str(&n.text);
        out.push((rest, text_attr));
        out
    }

    /// Что под точкой строки: стрелка, флажок или текст.
    fn hit(&self, row: usize, x: usize) -> Hit {
        let r = &self.rows[row];
        let marker = r.level() * 2 + 1;
        let has = self.node(&r.path).map(|n| n.has_children()).unwrap_or(false);
        if has && (x == marker || x == marker - 1) {
            return Hit::Marker;
        }
        let pw = Self::prefix_width(r);
        if self.checkboxes && x >= pw && x < pw + 3 {
            return Hit::Check;
        }
        Hit::Text
    }

    /// Действие Enter / двойного щелчка.
    fn activate(&mut self, ctx: &mut Ctx) -> bool {
        let path = match self.cursor_path() {
            Some(p) => p,
            None => return false,
        };
        let has = self.node(&path).map(|n| n.has_children()).unwrap_or(false);
        if self.enter_toggles && has {
            self.toggle(&path);
            true
        } else {
            ctx.message(cm::LIST_ITEM_SELECTED, Info::Id(self.vs.id));
            false
        }
    }

    /// Быстрый поиск: перейти к строке, текст которой начинается с набранного.
    fn quick_search(&mut self, c: char) -> bool {
        let now = Instant::now();
        if self.search_at.map(|t| now.duration_since(t) > SEARCH_TIMEOUT).unwrap_or(true) {
            self.search.clear();
        }
        self.search_at = Some(now);
        self.search.push(c);
        let want = self.search.to_lowercase();
        let n = self.rows.len();
        // С первой буквы ищем со следующей строки (повтор буквы — следующее
        // совпадение), дальше — с текущей.
        let start = if self.search.chars().count() == 1 { self.cursor + 1 } else { self.cursor };
        for k in 0..n {
            let i = (start + k) % n;
            if let Some(node) = self.node(&self.rows[i].path) {
                if node.text.to_lowercase().starts_with(&want) {
                    self.cursor = i;
                    self.ensure_visible();
                    return true;
                }
            }
        }
        // Нет совпадения — буква не добавляется.
        self.search.pop();
        false
    }

    /// Сообщить полосам прокрутки и владельцу об изменениях.
    fn sync(&mut self, ctx: &mut Ctx) {
        let key = (self.cursor, self.rows.len(), self.left, self.max_width, self.vs.size.x, self.vs.size.y);
        if self.last_sent != Some(key) {
            self.last_sent = Some(key);
            if let Some(id) = self.v_scroll {
                let page = (self.page() as i32 - 1).max(1);
                let max = self.rows.len().saturating_sub(1) as i32;
                ctx.message(cm::SCROLL_BAR_SET, Info::Scroll { id, value: self.cursor as i32, min: 0, max, page, arrow: 1 });
            }
            if let Some(id) = self.h_scroll {
                let page = (self.vs.size.x - 1).max(1);
                ctx.message(cm::SCROLL_BAR_SET, Info::Scroll { id, value: self.left as i32, min: 0, max: self.max_left() as i32, page, arrow: 1 });
            }
        }
        let id = self.vs.id;
        let mut pending = std::mem::take(&mut self.pending);
        pending.dedup();
        for c in pending {
            ctx.message(c, Info::Id(id));
        }
        let cur = self.cursor_path();
        if cur != self.last_cursor {
            self.last_cursor = cur;
            ctx.message(cm::TREE_CURSOR_MOVED, Info::Id(id));
        }
    }
}

/// Часть строки под мышью.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Hit {
    Marker,
    Check,
    Text,
}

impl View for TreeView {
    crate::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let colors = self.palette_colors(dc);
        let w = self.vs.size.x.max(0) as usize;
        let focused = self.vs.is_focused();
        for y in 0..self.vs.size.y.max(0) as usize {
            let i = self.top + y;
            let mut b = DrawBuffer::new(w);
            b.move_char(0, ' ', colors.normal, w);
            if i < self.rows.len() {
                let is_cur = i == self.cursor;
                let node_color = self.node(&self.rows[i].path).and_then(|n| n.color);
                let text_attr = match (is_cur, focused) {
                    (true, true) => colors.cursor,
                    (true, false) => colors.cursor_inactive,
                    _ => node_color.unwrap_or(colors.normal),
                };
                let segs = self.segments(i, text_attr, colors.lines);
                // Полоса курсора — от начала текста до правого края.
                if is_cur {
                    let pw = Self::prefix_width(&self.rows[i]).saturating_sub(self.left);
                    if pw < w {
                        b.move_char(pw, ' ', text_attr, w - pw);
                    }
                }
                // Горизонтальный сдвиг: пропускаем `left` ячеек.
                let mut skip = self.left;
                let mut x = 0;
                for (s, a) in segs {
                    let mut vis = String::new();
                    for c in s.chars() {
                        let cw = char_width(c);
                        if skip > 0 {
                            skip = skip.saturating_sub(cw);
                            continue;
                        }
                        vis.push(c);
                    }
                    if x < w {
                        x += b.move_str_limited(x, &vis, a, w - x);
                    }
                }
            }
            dc.write_line(0, y as i32, self.vs.size.x, 1, &b);
        }
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        default_handle_event(&self.vs, ev, ctx);
        let id = self.vs.id;
        let focused = self.vs.is_focused();
        match ev {
            Event::MouseDown(m) => {
                let m = *m;
                let p = self.vs.make_local(m.pos);
                if p.y >= 0 {
                    let row = self.top + p.y as usize;
                    if row < self.rows.len() {
                        self.cursor = row;
                        self.ensure_visible();
                        let x = (p.x.max(0) as usize) + self.left;
                        let path = self.rows[row].path.clone();
                        match self.hit(row, x) {
                            Hit::Marker => self.toggle(&path),
                            Hit::Check => self.toggle_check(&path),
                            Hit::Text if m.double => {
                                self.activate(ctx);
                            }
                            Hit::Text => {}
                        }
                    }
                }
                ev.clear();
            }
            Event::MouseWheel(m) => {
                if m.mods.shift {
                    self.scroll_h(m.wheel as i64 * 4);
                } else {
                    // Колесо прокручивает список, курсор остаётся в видимой части.
                    let max_top = self.rows.len().saturating_sub(self.page()) as i64;
                    self.top = (self.top as i64 + m.wheel as i64 * 3).clamp(0, max_top) as usize;
                    let page = self.page();
                    if self.cursor < self.top {
                        self.cursor = self.top;
                    } else if self.cursor >= self.top + page {
                        self.cursor = self.top + page - 1;
                    }
                }
                ev.clear();
            }
            Event::KeyDown(k) if focused => {
                let k = *k;
                let cur = self.cursor as i64;
                let page = self.page() as i64;
                let path = self.cursor_path();
                let node = path.as_ref().and_then(|p| self.node(p));
                let has = node.map(|n| n.has_children()).unwrap_or(false);
                let open = node.map(|n| n.expanded && !n.children.is_empty()).unwrap_or(false);
                let text = node.map(|n| n.text.clone());
                // Любая клавиша, кроме букв, начинает быстрый поиск заново.
                if k.text_char().is_none() && k.key != Key::Backspace {
                    self.search.clear();
                }
                let handled = match (k.key, k.mods) {
                    (Key::Up, Mods::NONE) => {
                        self.move_cursor(cur - 1);
                        true
                    }
                    (Key::Down, Mods::NONE) => {
                        self.move_cursor(cur + 1);
                        true
                    }
                    (Key::PgUp, Mods::NONE) => {
                        self.move_cursor(cur - (page - 1).max(1));
                        true
                    }
                    (Key::PgDn, Mods::NONE) => {
                        self.move_cursor(cur + (page - 1).max(1));
                        true
                    }
                    (Key::Home, _) | (Key::PgUp, Mods::CTRL) => {
                        self.move_cursor(0);
                        true
                    }
                    (Key::End, _) | (Key::PgDn, Mods::CTRL) => {
                        self.move_cursor(self.rows.len() as i64 - 1);
                        true
                    }
                    (Key::Right, Mods::NONE) => {
                        if let Some(p) = &path {
                            if open {
                                self.move_cursor(cur + 1);
                            } else if has {
                                self.expand(p);
                            }
                        }
                        true
                    }
                    (Key::Left, Mods::NONE) => {
                        if let Some(p) = &path {
                            if open {
                                self.collapse(p);
                            } else if p.len() > 1 {
                                self.select_path(&p[..p.len() - 1]);
                            }
                        }
                        true
                    }
                    (Key::Left, Mods::CTRL) => {
                        self.scroll_h(-4);
                        true
                    }
                    (Key::Right, Mods::CTRL) => {
                        self.scroll_h(4);
                        true
                    }
                    (Key::Char('+'), _) => {
                        if let Some(p) = &path {
                            self.expand(p);
                        }
                        true
                    }
                    (Key::Char('-'), _) => {
                        if let Some(p) = &path {
                            self.collapse(p);
                        }
                        true
                    }
                    (Key::Char('*'), _) => {
                        if let Some(p) = &path {
                            self.expand_subtree(p, 8);
                        }
                        true
                    }
                    (Key::Char(' '), Mods::NONE) => {
                        if let Some(p) = &path {
                            if self.checkboxes {
                                self.toggle_check(p);
                            } else if has {
                                self.toggle(p);
                            }
                        }
                        true
                    }
                    (Key::Enter, Mods::NONE) => self.activate(ctx),
                    (Key::Backspace, Mods::NONE) => {
                        if self.search.pop().is_none() {
                            // Без поиска Backspace — к родителю.
                            if let Some(p) = &path {
                                if p.len() > 1 {
                                    self.select_path(&p[..p.len() - 1]);
                                }
                            }
                        }
                        true
                    }
                    (Key::Ins, Mods::CTRL) => {
                        if let Some(t) = &text {
                            ctx.clipboard.set(t);
                        }
                        true
                    }
                    (Key::Char(c), Mods::CTRL) if c.eq_ignore_ascii_case(&'c') => {
                        if let Some(t) = &text {
                            ctx.clipboard.set(t);
                        }
                        true
                    }
                    _ => match k.text_char() {
                        Some(c) if !c.is_control() => {
                            self.quick_search(c);
                            true
                        }
                        _ => false,
                    },
                };
                if handled {
                    ev.clear();
                }
            }
            Event::Broadcast(m) => match (&m.command, &m.info) {
                (&cm::SCROLL_BAR_CHANGED, Info::Scroll { id: sb, value, .. }) if Some(*sb) == self.v_scroll => {
                    self.move_cursor((*value).max(0) as i64);
                    self.last_sent = None;
                }
                (&cm::SCROLL_BAR_CHANGED, Info::Scroll { id: sb, value, .. }) if Some(*sb) == self.h_scroll => {
                    self.left = ((*value).max(0) as usize).min(self.max_left());
                    self.last_sent = None;
                }
                (&cm::SCROLL_BAR_CLICKED, Info::Id(sb)) if Some(*sb) == self.v_scroll || Some(*sb) == self.h_scroll => {
                    if !self.vs.is_selected() {
                        ctx.select(id);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        self.sync(ctx);
    }

    fn change_bounds(&mut self, bounds: Rect) {
        self.vs.set_bounds(bounds);
        self.left = self.left.min(self.max_left());
        self.ensure_visible();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<TreeNode> {
        vec![
            TreeNode::new("src")
                .expanded(true)
                .child(TreeNode::new("main.rs"))
                .child(TreeNode::new("widgets").child(TreeNode::new("tree.rs")).child(TreeNode::new("table.rs"))),
            TreeNode::new("Cargo.toml"),
        ]
    }

    fn texts(t: &TreeView) -> Vec<String> {
        (0..t.row_count()).map(|i| t.node(t.row_path(i).unwrap()).unwrap().text.clone()).collect()
    }

    #[test]
    fn rows_expand_collapse() {
        let mut t = TreeView::new(Rect::new(0, 0, 30, 10), sample(), None, None);
        assert_eq!(texts(&t), ["src", "main.rs", "widgets", "Cargo.toml"]);
        t.expand(&[0, 1]);
        assert_eq!(t.row_count(), 6);
        assert_eq!(t.path_texts(&[0, 1, 1]), ["src", "widgets", "table.rs"]);
        // Курсор внутри сворачиваемой ветки встаёт на её корень.
        assert!(t.select_path(&[0, 1, 0]));
        t.collapse(&[0]);
        assert_eq!(t.cursor_path(), Some(vec![0]));
        assert_eq!(t.row_count(), 2);
    }

    #[test]
    fn select_path_expands_ancestors() {
        let mut t = TreeView::new(Rect::new(0, 0, 30, 10), sample(), None, None);
        t.collapse_all();
        assert_eq!(t.row_count(), 2);
        assert!(t.select_path(&[0, 1, 1]));
        assert_eq!(t.current().unwrap().text, "table.rs");
        assert!(!t.select_path(&[5]));
    }

    #[test]
    fn lazy_loading() {
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let c = calls.clone();
        let mut t = TreeView::new(Rect::new(0, 0, 30, 10), vec![TreeNode::new("корень").lazy()], None, None).with_loader(move |path, n| {
            c.set(c.get() + 1);
            if path.len() < 3 {
                (0..2).map(|i| TreeNode::new(&format!("{}.{}", n.text, i)).lazy()).collect()
            } else {
                Vec::new()
            }
        });
        assert_eq!(t.row_count(), 1);
        t.expand(&[0]);
        assert_eq!(texts(&t), ["корень", "корень.0", "корень.1"]);
        t.collapse(&[0]);
        t.expand(&[0]);
        assert_eq!(calls.get(), 1, "дети загружаются один раз");
        // Лист: загрузчик вернул пусто — стрелки больше нет.
        t.expand_subtree(&[0], 5);
        assert_eq!(t.find_text("корень.0.1"), Some(vec![0, 0, 1]));
        assert!(!t.node(&[0, 0, 1]).unwrap().has_children());
        assert_eq!(t.row_count(), 7);
    }

    #[test]
    fn checks_tristate() {
        let mut t = TreeView::new(Rect::new(0, 0, 30, 10), sample(), None, None).checkboxes(true);
        t.set_checked(&[0, 1, 0], true);
        assert_eq!(t.check_state(&[0, 1]), CheckState::Partial);
        assert_eq!(t.check_state(&[0]), CheckState::Partial);
        t.toggle_check(&[0]);
        assert_eq!(t.check_state(&[0]), CheckState::Checked);
        assert_eq!(t.checked_paths(), vec![vec![0, 0], vec![0, 1, 0], vec![0, 1, 1]]);
        t.toggle_check(&[0]);
        assert!(t.checked_paths().is_empty());
    }

    #[test]
    fn insert_and_remove_keep_cursor() {
        let mut t = TreeView::new(Rect::new(0, 0, 30, 10), sample(), None, None);
        t.select_path(&[1]); // Cargo.toml
        t.insert(None, 0, TreeNode::new("README.md"));
        assert_eq!(t.current().unwrap().text, "Cargo.toml");
        assert_eq!(t.cursor_path(), Some(vec![2]));
        // Удаление узла курсора — курсор на предыдущего брата (следующего нет).
        t.remove(&[2]);
        assert_eq!(t.current().unwrap().text, "src");
        // Удаление единственного ребёнка — курсор на родителя.
        let p = t.push(Some(&[0]), TreeNode::new("x")).unwrap(); // у README.md детей не было
        assert!(t.select_path(&p));
        t.remove(&p);
        assert_eq!(t.cursor_path(), Some(vec![0]));
    }

    #[test]
    fn segments_and_hit() {
        let mut t = TreeView::new(Rect::new(0, 0, 30, 10), sample(), None, None);
        t.expand(&[0, 1]);
        let line = |t: &TreeView, i: usize| t.segments(i, 0, 0).into_iter().map(|s| s.0).collect::<String>();
        assert_eq!(line(&t, 0), "├▾ src");
        assert_eq!(line(&t, 1), "│ ├─ main.rs");
        assert_eq!(line(&t, 3), "│   ├─ tree.rs"); // `widgets` — последний у `src`: вертикали нет
        assert_eq!(line(&t, 4), "│   └─ table.rs");
        assert_eq!(line(&t, 5), "└─ Cargo.toml");
        assert_eq!(t.hit(2, 3), Hit::Marker);
        assert_eq!(t.hit(2, 6), Hit::Text);
        let t = TreeView::new(Rect::new(0, 0, 30, 10), sample(), None, None).lines(false).checkboxes(true);
        assert_eq!(line(&t, 0), " ▾ [ ] src");
        assert_eq!(line(&t, 1), "   ─ [ ] main.rs".replace('─', " "));
        assert_eq!(t.hit(0, 3), Hit::Check);
    }

    #[test]
    fn quick_search_finds_visible() {
        let mut t = TreeView::new(Rect::new(0, 0, 30, 10), sample(), None, None);
        assert!(t.quick_search('c'));
        assert_eq!(t.current().unwrap().text, "Cargo.toml");
        assert!(!t.quick_search('z'));
        assert_eq!(t.current().unwrap().text, "Cargo.toml");
    }
}
