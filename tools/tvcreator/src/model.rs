//! Документ формы: дерево раскладок и виджетов, операции правки и файл
//! `.tvui` (JSON — как `.ui` у Qt Designer, только читается глазами).
//!
//! ```json
//! {
//!   "tvcreator": 1,
//!   "name": "LoginDialog",
//!   "title": "Вход",
//!   "kind": "dialog",
//!   "size": [40, 12],
//!   "auto_size": true,
//!   "command_base": 100,
//!   "layout": {
//!     "type": "vbox", "margins": [2, 1, 2, 0],
//!     "items": [
//!       {"type": "form", "items": [
//!         {"type": "label", "name": "label1", "text": "~И~мя:", "link": "user"},
//!         {"type": "input", "name": "user", "max_len": 30}
//!       ]},
//!       {"type": "stretch", "factor": 1},
//!       {"type": "hbox", "items": [
//!         {"type": "stretch", "factor": 1},
//!         {"type": "button", "name": "ok", "text": "O~K~", "command": "OK", "default": true}
//!       ]}
//!     ]
//!   }
//! }
//! ```

use tvision::json::{parse_json, Json};
use tvision::layout::Alignment;

/// Версия формата файла.
pub const FORMAT_VERSION: i64 = 1;

// ----------------------------------------------------------------------
// Типы
// ----------------------------------------------------------------------

/// Что за окно описывает форма.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormKind {
    /// Модальный диалог (`Dialog`), открывается по центру.
    Dialog,
    /// Обычное окно (`Window`): двигается, меняет размер.
    Window,
}

/// Вид раскладки.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutKind {
    HBox,
    VBox,
    Grid,
    Form,
}

/// Вид виджета.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidgetKind {
    Label,
    StaticText,
    InputLine,
    Button,
    CheckBoxes,
    RadioButtons,
    ListBox,
    Table,
    ProgressBar,
    Line,
}

impl LayoutKind {
    pub const ALL: [LayoutKind; 4] = [LayoutKind::HBox, LayoutKind::VBox, LayoutKind::Grid, LayoutKind::Form];

    /// Имя в файле.
    pub fn key(self) -> &'static str {
        match self {
            LayoutKind::HBox => "hbox",
            LayoutKind::VBox => "vbox",
            LayoutKind::Grid => "grid",
            LayoutKind::Form => "form",
        }
    }
    /// Имя типа в коде (`HBox`…).
    pub fn rust(self) -> &'static str {
        match self {
            LayoutKind::HBox => "HBox",
            LayoutKind::VBox => "VBox",
            LayoutKind::Grid => "Grid",
            LayoutKind::Form => "Form",
        }
    }
    pub fn from_key(s: &str) -> Option<LayoutKind> {
        LayoutKind::ALL.into_iter().find(|k| k.key() == s || k.rust().eq_ignore_ascii_case(s))
    }
}

impl WidgetKind {
    pub const ALL: [WidgetKind; 10] = [
        WidgetKind::Label,
        WidgetKind::StaticText,
        WidgetKind::InputLine,
        WidgetKind::Button,
        WidgetKind::CheckBoxes,
        WidgetKind::RadioButtons,
        WidgetKind::ListBox,
        WidgetKind::Table,
        WidgetKind::ProgressBar,
        WidgetKind::Line,
    ];

    /// Имя в файле.
    pub fn key(self) -> &'static str {
        match self {
            WidgetKind::Label => "label",
            WidgetKind::StaticText => "text",
            WidgetKind::InputLine => "input",
            WidgetKind::Button => "button",
            WidgetKind::CheckBoxes => "checkboxes",
            WidgetKind::RadioButtons => "radiobuttons",
            WidgetKind::ListBox => "listbox",
            WidgetKind::Table => "table",
            WidgetKind::ProgressBar => "progress",
            WidgetKind::Line => "line",
        }
    }
    /// Название для человека (палитра, дерево объектов).
    pub fn title(self) -> &'static str {
        match self {
            WidgetKind::Label => "Подпись (Label)",
            WidgetKind::StaticText => "Текст (StaticText)",
            WidgetKind::InputLine => "Строка ввода (InputLine)",
            WidgetKind::Button => "Кнопка (Button)",
            WidgetKind::CheckBoxes => "Флажки (CheckBoxes)",
            WidgetKind::RadioButtons => "Переключатели (RadioButtons)",
            WidgetKind::ListBox => "Список (ListBox)",
            WidgetKind::Table => "Таблица (Table)",
            WidgetKind::ProgressBar => "Полоса выполнения (ProgressBar)",
            WidgetKind::Line => "Линия (Line)",
        }
    }
    /// Приставка для имён новых виджетов: `button1`, `input2`…
    pub fn prefix(self) -> &'static str {
        match self {
            WidgetKind::Label => "label",
            WidgetKind::StaticText => "text",
            WidgetKind::InputLine => "input",
            WidgetKind::Button => "button",
            WidgetKind::CheckBoxes => "checks",
            WidgetKind::RadioButtons => "radios",
            WidgetKind::ListBox => "list",
            WidgetKind::Table => "table",
            WidgetKind::ProgressBar => "progress",
            WidgetKind::Line => "line",
        }
    }
    pub fn from_key(s: &str) -> Option<WidgetKind> {
        WidgetKind::ALL.into_iter().find(|k| k.key() == s)
    }
    /// Можно ли сделать текущим (получает фокус по Tab).
    pub fn selectable(self) -> bool {
        !matches!(self, WidgetKind::Label | WidgetKind::StaticText | WidgetKind::ProgressBar | WidgetKind::Line)
    }
    /// Есть ли у виджета данные (`get_data`/`set_data`).
    pub fn has_data(self) -> bool {
        matches!(self, WidgetKind::InputLine | WidgetKind::CheckBoxes | WidgetKind::RadioButtons | WidgetKind::ListBox)
    }
}

/// Свойства виджета. Используются не все: у каждого вида — свои (см.
/// [`crate::props`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Widget {
    pub kind: WidgetKind,
    /// Имя переменной и поля в сгенерированной структуре.
    pub name: String,
    /// Надпись (подпись, текст, кнопка).
    pub text: String,
    /// Команда кнопки: `OK`, `CANCEL`… или своя `CM_ИМЯ`.
    pub command: String,
    /// Кнопка по умолчанию (Enter).
    pub default: bool,
    /// Длина строки ввода.
    pub max_len: usize,
    /// Пункты флажков, переключателей, списка; заголовки столбцов таблицы.
    pub items: Vec<String>,
    /// Подпись: имя виджета, который она подписывает.
    pub link: String,
    /// Полоса выполнения: максимум.
    pub max: i64,
    /// Линия: вертикальная (иначе горизонтальная), двойная, до рамки окна.
    pub vertical: bool,
    pub double: bool,
    pub to_frame: bool,
}

impl Widget {
    /// Новый виджет со значениями по умолчанию.
    pub fn new(kind: WidgetKind, name: &str) -> Widget {
        let (text, items): (&str, Vec<&str>) = match kind {
            WidgetKind::Label => ("~П~одпись:", vec![]),
            WidgetKind::StaticText => ("Текст", vec![]),
            WidgetKind::Button => ("~К~нопка", vec![]),
            WidgetKind::CheckBoxes => ("", vec!["~П~ервый", "~В~торой"]),
            WidgetKind::RadioButtons => ("", vec!["~О~дин", "~Д~ва"]),
            WidgetKind::ListBox => ("", vec!["Строка 1", "Строка 2"]),
            WidgetKind::Table => ("", vec!["Столбец 1", "Столбец 2"]),
            _ => ("", vec![]),
        };
        Widget {
            kind,
            name: name.to_string(),
            text: text.to_string(),
            command: if kind == WidgetKind::Button { "OK".into() } else { String::new() },
            default: false,
            max_len: 30,
            items: items.into_iter().map(String::from).collect(),
            link: String::new(),
            max: 100,
            vertical: false,
            double: false,
            to_frame: false,
        }
    }
}

/// Уточнения размера и места элемента (то, что в коде пишется через
/// `item(…)`) и его клетка в сетке.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemProps {
    pub stretch: u16,
    pub expand: bool,
    pub align_h: Alignment,
    pub align_v: Alignment,
    pub min_w: Option<i32>,
    pub min_h: Option<i32>,
    pub fixed_w: Option<i32>,
    pub fixed_h: Option<i32>,
    /// Сетка: явная клетка (строка, столбец); `None` — по порядку.
    pub cell: Option<(usize, usize)>,
    /// Сетка: сколько строк и столбцов занимает.
    pub span: (usize, usize),
    /// Форма: на всю ширину (обе колонки).
    pub wide: bool,
}

impl Default for ItemProps {
    fn default() -> Self {
        ItemProps {
            stretch: 0,
            expand: false,
            align_h: Alignment::Fill,
            align_v: Alignment::Fill,
            min_w: None,
            min_h: None,
            fixed_w: None,
            fixed_h: None,
            cell: None,
            span: (1, 1),
            wide: false,
        }
    }
}

impl ItemProps {
    /// Нет ли уточнений (тогда в коде — просто `id`).
    pub fn is_plain(&self) -> bool {
        self.stretch == 0
            && !self.expand
            && self.align_h == Alignment::Fill
            && self.align_v == Alignment::Fill
            && self.min_w.is_none()
            && self.min_h.is_none()
            && self.fixed_w.is_none()
            && self.fixed_h.is_none()
    }
}

/// Узел дерева формы.
#[derive(Clone, Debug, PartialEq)]
pub enum NodeKind {
    Layout {
        kind: LayoutKind,
        /// Промежуток; `None` — по умолчанию для вида раскладки.
        spacing: Option<i32>,
        /// Поля: слева, сверху, справа, снизу.
        margins: [i32; 4],
        /// Сетка: столбцов при расстановке по порядку.
        columns: usize,
    },
    Widget(Widget),
    /// Пружина с коэффициентом.
    Stretch(u16),
    /// Пустое место в клетках.
    Space(i32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    pub item: ItemProps,
    /// Дети (только у раскладок).
    pub children: Vec<Node>,
}

impl Node {
    pub fn layout(kind: LayoutKind) -> Node {
        Node { kind: NodeKind::Layout { kind, spacing: None, margins: [0; 4], columns: 2 }, item: ItemProps::default(), children: Vec::new() }
    }
    pub fn widget(w: Widget) -> Node {
        let mut item = ItemProps::default();
        if matches!(w.kind, WidgetKind::ListBox | WidgetKind::Table) {
            item.min_w = Some(20);
            item.min_h = Some(5);
        }
        Node { kind: NodeKind::Widget(w), item, children: Vec::new() }
    }
    pub fn stretch(f: u16) -> Node {
        Node { kind: NodeKind::Stretch(f), item: ItemProps::default(), children: Vec::new() }
    }
    pub fn space(n: i32) -> Node {
        Node { kind: NodeKind::Space(n), item: ItemProps::default(), children: Vec::new() }
    }
    pub fn is_layout(&self) -> bool {
        matches!(self.kind, NodeKind::Layout { .. })
    }
    pub fn as_widget(&self) -> Option<&Widget> {
        match &self.kind {
            NodeKind::Widget(w) => Some(w),
            _ => None,
        }
    }
    pub fn as_widget_mut(&mut self) -> Option<&mut Widget> {
        match &mut self.kind {
            NodeKind::Widget(w) => Some(w),
            _ => None,
        }
    }
    pub fn layout_kind(&self) -> Option<LayoutKind> {
        match self.kind {
            NodeKind::Layout { kind, .. } => Some(kind),
            _ => None,
        }
    }

    /// Строка для дерева объектов.
    pub fn caption(&self) -> String {
        match &self.kind {
            NodeKind::Layout { kind, .. } => kind.rust().to_string(),
            NodeKind::Widget(w) => {
                let t = w.text.replace('~', "");
                let what = w.kind.title().split(" (").next().unwrap_or("");
                if t.is_empty() {
                    format!("{} — {}", w.name, what)
                } else {
                    format!("{} — {} «{}»", w.name, what, t)
                }
            }
            NodeKind::Stretch(f) => format!("пружина ×{}", f),
            NodeKind::Space(n) => format!("промежуток {}", n),
        }
    }
}

/// Документ формы.
#[derive(Clone, Debug, PartialEq)]
pub struct FormDoc {
    /// Имя структуры в коде (`LoginDialog`).
    pub name: String,
    /// Заголовок окна.
    pub title: String,
    pub kind: FormKind,
    /// Размер окна (если `auto_size` выключен — точный, иначе начальный).
    pub size: (i32, i32),
    /// Размер окна по содержимому (`adjust_size`).
    pub auto_size: bool,
    /// С какого номера нумеровать свои команды кнопок.
    pub command_base: u16,
    /// Корневая раскладка.
    pub root: Node,
}

impl Default for FormDoc {
    fn default() -> Self {
        FormDoc::new("Form1")
    }
}

impl FormDoc {
    /// Пустая форма: диалог с вертикальной раскладкой.
    pub fn new(name: &str) -> FormDoc {
        let mut root = Node::layout(LayoutKind::VBox);
        if let NodeKind::Layout { margins, .. } = &mut root.kind {
            *margins = [1, 1, 1, 0];
        }
        FormDoc {
            name: name.to_string(), title: "Форма".into(), kind: FormKind::Dialog, size: (40, 12), auto_size: true, command_base: 100, root
        }
    }

    // ---------- навигация ----------

    /// Узел по пути (`[]` — корневая раскладка).
    pub fn node(&self, path: &[usize]) -> Option<&Node> {
        let mut n = &self.root;
        for &i in path {
            n = n.children.get(i)?;
        }
        Some(n)
    }

    pub fn node_mut(&mut self, path: &[usize]) -> Option<&mut Node> {
        let mut n = &mut self.root;
        for &i in path {
            n = n.children.get_mut(i)?;
        }
        Some(n)
    }

    /// Все узлы в порядке обхода (родитель раньше детей): (путь, узел).
    pub fn walk(&self) -> Vec<(Vec<usize>, &Node)> {
        fn go<'a>(n: &'a Node, path: &mut Vec<usize>, out: &mut Vec<(Vec<usize>, &'a Node)>) {
            out.push((path.clone(), n));
            for (i, c) in n.children.iter().enumerate() {
                path.push(i);
                go(c, path, out);
                path.pop();
            }
        }
        let mut out = Vec::new();
        go(&self.root, &mut Vec::new(), &mut out);
        out
    }

    /// Все виджеты в порядке обхода (это и порядок Tab).
    pub fn widgets(&self) -> Vec<&Widget> {
        self.walk().into_iter().filter_map(|(_, n)| n.as_widget()).collect()
    }

    /// Путь виджета по имени.
    pub fn find_widget(&self, name: &str) -> Option<Vec<usize>> {
        self.walk().into_iter().find(|(_, n)| n.as_widget().is_some_and(|w| w.name == name)).map(|(p, _)| p)
    }

    /// Свободное имя: `button1`, `button2`…
    pub fn unique_name(&self, prefix: &str) -> String {
        let names: Vec<String> = self.widgets().iter().map(|w| w.name.clone()).collect();
        (1..).map(|i| format!("{}{}", prefix, i)).find(|n| !names.contains(n)).unwrap()
    }

    // ---------- правка ----------

    /// Вставить узел: в выбранную раскладку последним, а если выбран не
    /// контейнер — сразу после него. Возвращает путь нового узла.
    pub fn insert(&mut self, selected: &[usize], node: Node) -> Vec<usize> {
        let selected: &[usize] = if self.node(selected).is_some() { selected } else { &[] };
        let into_self = self.node(selected).map_or(true, |n| n.is_layout());
        if into_self {
            let parent = self.node_mut(selected).expect("узел");
            parent.children.push(node);
            let mut p = selected.to_vec();
            p.push(parent.children.len() - 1);
            p
        } else {
            let (parent_path, idx) = selected.split_at(selected.len() - 1);
            let at = idx[0] + 1;
            let parent = self.node_mut(parent_path).expect("родитель");
            parent.children.insert(at, node);
            let mut p = parent_path.to_vec();
            p.push(at);
            p
        }
    }

    /// Удалить узел (корень удалить нельзя). Возвращает путь, который
    /// разумно выбрать после удаления.
    pub fn remove(&mut self, path: &[usize]) -> Option<Vec<usize>> {
        if path.is_empty() {
            return None;
        }
        let (pp, idx) = path.split_at(path.len() - 1);
        let parent = self.node_mut(pp)?;
        if idx[0] >= parent.children.len() {
            return None;
        }
        parent.children.remove(idx[0]);
        let n = parent.children.len();
        let mut next = pp.to_vec();
        if n > 0 {
            next.push(idx[0].min(n - 1));
        }
        Some(next)
    }

    /// Сдвинуть узел среди соседей (`-1` — выше, `+1` — ниже).
    pub fn shift(&mut self, path: &[usize], delta: i32) -> Option<Vec<usize>> {
        let (pp, idx) = path.split_last().map(|(i, p)| (p, *i))?;
        let parent = self.node_mut(pp)?;
        let to = idx as i32 + delta;
        if to < 0 || to as usize >= parent.children.len() {
            return None;
        }
        parent.children.swap(idx, to as usize);
        let mut p = pp.to_vec();
        p.push(to as usize);
        Some(p)
    }

    /// Обернуть узел в новую раскладку (как «Lay out horizontally» в Qt).
    pub fn wrap(&mut self, path: &[usize], kind: LayoutKind) -> Vec<usize> {
        let mut lay = Node::layout(kind);
        if path.is_empty() {
            let old = std::mem::replace(&mut self.root, lay);
            self.root.children.push(old);
            return vec![];
        }
        let n = self.node_mut(path).expect("узел");
        let old = std::mem::replace(n, Node::layout(kind));
        lay = old;
        // Уточнения размера переходят к обёртке: место в родителе то же.
        let item = std::mem::take(&mut lay.item);
        let n = self.node_mut(path).unwrap();
        n.item = item;
        n.children.push(lay);
        path.to_vec()
    }

    /// Переименовать виджет и все ссылки подписей на него.
    pub fn rename(&mut self, old: &str, new: &str) {
        fn go(n: &mut Node, old: &str, new: &str) {
            if let Some(w) = n.as_widget_mut() {
                if w.name == old {
                    w.name = new.to_string();
                }
                if w.link == old {
                    w.link = new.to_string();
                }
            }
            for c in &mut n.children {
                go(c, old, new);
            }
        }
        go(&mut self.root, old, new);
    }

    /// Проблемы формы, из-за которых код не соберётся или поведёт себя
    /// странно: повторяющиеся имена, подписи без цели…
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if !is_type_name(&self.name) {
            out.push(format!("Имя формы «{}» должно быть именем типа Rust: латиница, с заглавной буквы.", self.name));
        }
        let ws = self.widgets();
        for (i, w) in ws.iter().enumerate() {
            if !is_ident(&w.name) {
                out.push(format!("Имя «{}» не годится для переменной Rust.", w.name));
            }
            if ws[..i].iter().any(|x| x.name == w.name) {
                out.push(format!("Имя «{}» встречается дважды.", w.name));
            }
            if w.kind == WidgetKind::Label && !w.link.is_empty() && !ws.iter().any(|x| x.name == w.link && x.kind != WidgetKind::Label) {
                out.push(format!("Подпись «{}» ссылается на несуществующий виджет «{}».", w.name, w.link));
            }
            if w.kind == WidgetKind::Button && command_value(self, &w.command).is_none() {
                out.push(format!("Кнопка «{}»: команда «{}» — нужна OK, CANCEL, YES, NO, CLOSE, HELP или своя CM_ИМЯ.", w.name, w.command));
            }
        }
        out
    }

    // ---------- файл ----------

    pub fn to_json(&self) -> Json {
        Json::object(vec![
            ("tvcreator", Json::from_i64(FORMAT_VERSION)),
            ("name", Json::from_str(&self.name)),
            ("title", Json::from_str(&self.title)),
            ("kind", Json::from_str(if self.kind == FormKind::Dialog { "dialog" } else { "window" })),
            ("size", Json::Arr(vec![Json::from_i64(self.size.0 as i64), Json::from_i64(self.size.1 as i64)])),
            ("auto_size", Json::Bool(self.auto_size)),
            ("command_base", Json::from_i64(self.command_base as i64)),
            ("layout", node_to_json(&self.root)),
        ])
    }

    /// Текст файла `.tvui`.
    pub fn to_text(&self) -> String {
        self.to_json().pretty() + "\n"
    }

    pub fn from_json(j: &Json) -> Result<FormDoc, String> {
        if j.get("tvcreator").and_then(Json::as_i64).is_none() {
            return Err("Это не файл формы TVCreator (нет ключа \"tvcreator\").".into());
        }
        let s = |k: &str, d: &str| j.get(k).and_then(Json::as_str).unwrap_or(d).to_string();
        let size = j.get("size").and_then(Json::as_array).unwrap_or(&[]);
        let root = match j.get("layout") {
            Some(l) => node_from_json(l)?,
            None => Node::layout(LayoutKind::VBox),
        };
        if !root.is_layout() {
            return Err("Корень формы должен быть раскладкой (hbox, vbox, grid, form).".into());
        }
        Ok(FormDoc {
            name: s("name", "Form1"),
            title: s("title", ""),
            kind: if s("kind", "dialog") == "window" { FormKind::Window } else { FormKind::Dialog },
            size: (size.first().and_then(Json::as_i64).unwrap_or(40) as i32, size.get(1).and_then(Json::as_i64).unwrap_or(12) as i32),
            auto_size: j.get("auto_size").and_then(Json::as_bool).unwrap_or(true),
            command_base: j.get("command_base").and_then(Json::as_i64).unwrap_or(100).clamp(100, 250) as u16,
            root,
        })
    }

    pub fn parse(text: &str) -> Result<FormDoc, String> {
        let j = parse_json(text).map_err(|e| e.to_string())?;
        FormDoc::from_json(&j)
    }

    pub fn load(path: &std::path::Path) -> Result<FormDoc, String> {
        let t = std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
        FormDoc::parse(&t).map_err(|e| format!("{}: {}", path.display(), e))
    }

    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        std::fs::write(path, self.to_text()).map_err(|e| format!("Не удалось записать {}:\n{}", path.display(), e))
    }
}

fn align_key(a: Alignment) -> &'static str {
    match a {
        Alignment::Fill => "fill",
        Alignment::Start => "start",
        Alignment::Center => "center",
        Alignment::End => "end",
    }
}

pub fn align_from(s: &str) -> Option<Alignment> {
    match s {
        "fill" => Some(Alignment::Fill),
        "start" => Some(Alignment::Start),
        "center" => Some(Alignment::Center),
        "end" => Some(Alignment::End),
        _ => None,
    }
}

fn node_to_json(n: &Node) -> Json {
    let mut o: Vec<(&str, Json)> = Vec::new();
    match &n.kind {
        NodeKind::Layout { kind, spacing, margins, columns } => {
            o.push(("type", Json::from_str(kind.key())));
            if let Some(s) = spacing {
                o.push(("spacing", Json::from_i64(*s as i64)));
            }
            if *margins != [0; 4] {
                o.push(("margins", Json::Arr(margins.iter().map(|m| Json::from_i64(*m as i64)).collect())));
            }
            if *kind == LayoutKind::Grid {
                o.push(("columns", Json::from_i64(*columns as i64)));
            }
        }
        NodeKind::Widget(w) => {
            o.push(("type", Json::from_str(w.kind.key())));
            o.push(("name", Json::from_str(&w.name)));
            match w.kind {
                WidgetKind::Label => {
                    o.push(("text", Json::from_str(&w.text)));
                    if !w.link.is_empty() {
                        o.push(("link", Json::from_str(&w.link)));
                    }
                }
                WidgetKind::StaticText => o.push(("text", Json::from_str(&w.text))),
                WidgetKind::Button => {
                    o.push(("text", Json::from_str(&w.text)));
                    o.push(("command", Json::from_str(&w.command)));
                    if w.default {
                        o.push(("default", Json::Bool(true)));
                    }
                }
                WidgetKind::InputLine => o.push(("max_len", Json::from_i64(w.max_len as i64))),
                WidgetKind::CheckBoxes | WidgetKind::RadioButtons | WidgetKind::ListBox | WidgetKind::Table => {
                    o.push(("items", Json::Arr(w.items.iter().map(|s| Json::from_str(s)).collect())));
                }
                WidgetKind::ProgressBar => o.push(("max", Json::from_i64(w.max))),
                WidgetKind::Line => {
                    for (k, v) in [("vertical", w.vertical), ("double", w.double), ("to_frame", w.to_frame)] {
                        if v {
                            o.push((k, Json::Bool(true)));
                        }
                    }
                }
            }
        }
        NodeKind::Stretch(f) => {
            o.push(("type", Json::from_str("stretch")));
            o.push(("factor", Json::from_i64(*f as i64)));
        }
        NodeKind::Space(s) => {
            o.push(("type", Json::from_str("space")));
            o.push(("size", Json::from_i64(*s as i64)));
        }
    }
    let it = &n.item;
    if it.stretch > 0 {
        o.push(("stretch", Json::from_i64(it.stretch as i64)));
    }
    if it.expand {
        o.push(("expand", Json::Bool(true)));
    }
    if it.align_h != Alignment::Fill {
        o.push(("align_h", Json::from_str(align_key(it.align_h))));
    }
    if it.align_v != Alignment::Fill {
        o.push(("align_v", Json::from_str(align_key(it.align_v))));
    }
    for (k, v) in [("min_w", it.min_w), ("min_h", it.min_h), ("fixed_w", it.fixed_w), ("fixed_h", it.fixed_h)] {
        if let Some(v) = v {
            o.push((k, Json::from_i64(v as i64)));
        }
    }
    if let Some((r, c)) = it.cell {
        o.push(("cell", Json::Arr(vec![Json::from_i64(r as i64), Json::from_i64(c as i64)])));
    }
    if it.span != (1, 1) {
        o.push(("span", Json::Arr(vec![Json::from_i64(it.span.0 as i64), Json::from_i64(it.span.1 as i64)])));
    }
    if it.wide {
        o.push(("wide", Json::Bool(true)));
    }
    if n.is_layout() {
        o.push(("items", Json::Arr(n.children.iter().map(node_to_json).collect())));
    }
    Json::object(o)
}

fn node_from_json(j: &Json) -> Result<Node, String> {
    let t = j.get("type").and_then(Json::as_str).ok_or("У элемента нет \"type\".")?;
    let int = |k: &str| j.get(k).and_then(Json::as_i64);
    let s = |k: &str| j.get(k).and_then(Json::as_str).unwrap_or("").to_string();
    let mut n = if let Some(kind) = LayoutKind::from_key(t) {
        let mut n = Node::layout(kind);
        let m = j.get("margins").and_then(Json::as_array).unwrap_or(&[]);
        let mut margins = [0; 4];
        for (i, v) in m.iter().take(4).enumerate() {
            margins[i] = v.as_i64().unwrap_or(0) as i32;
        }
        n.kind = NodeKind::Layout { kind, spacing: int("spacing").map(|v| v as i32), margins, columns: int("columns").unwrap_or(2).max(1) as usize };
        for c in j.get("items").and_then(Json::as_array).unwrap_or(&[]) {
            n.children.push(node_from_json(c)?);
        }
        n
    } else if let Some(kind) = WidgetKind::from_key(t) {
        let name = s("name");
        let mut w = Widget::new(kind, &name);
        if j.get("text").is_some() {
            w.text = s("text");
        }
        if j.get("command").is_some() {
            w.command = s("command");
        }
        w.default = j.get("default").and_then(Json::as_bool).unwrap_or(false);
        w.max_len = int("max_len").unwrap_or(30).clamp(1, 1000) as usize;
        if let Some(a) = j.get("items").and_then(Json::as_array) {
            w.items = a.iter().filter_map(|x| x.as_str().map(String::from)).collect();
        }
        w.link = s("link");
        w.max = int("max").unwrap_or(100);
        let flag = |k: &str| j.get(k).and_then(Json::as_bool).unwrap_or(false);
        w.vertical = flag("vertical");
        w.double = flag("double");
        w.to_frame = flag("to_frame");
        let mut n = Node::widget(w);
        n.item = ItemProps::default(); // уточнения — только из файла
        n
    } else if t == "stretch" {
        Node::stretch(int("factor").unwrap_or(1).clamp(1, 100) as u16)
    } else if t == "space" {
        Node::space(int("size").unwrap_or(1).clamp(0, 100) as i32)
    } else {
        return Err(format!("Неизвестный тип элемента «{}».", t));
    };
    let it = &mut n.item;
    it.stretch = int("stretch").unwrap_or(0).clamp(0, 100) as u16;
    it.expand = j.get("expand").and_then(Json::as_bool).unwrap_or(false);
    it.align_h = align_from(&s("align_h")).unwrap_or(Alignment::Fill);
    it.align_v = align_from(&s("align_v")).unwrap_or(Alignment::Fill);
    it.min_w = int("min_w").map(|v| v as i32);
    it.min_h = int("min_h").map(|v| v as i32);
    it.fixed_w = int("fixed_w").map(|v| v as i32);
    it.fixed_h = int("fixed_h").map(|v| v as i32);
    let pair = |k: &str| -> Option<(usize, usize)> {
        let a = j.get(k)?.as_array()?;
        Some((a.first()?.as_i64()?.max(0) as usize, a.get(1)?.as_i64()?.max(0) as usize))
    };
    it.cell = pair("cell");
    it.span = pair("span").map(|(r, c)| (r.max(1), c.max(1))).unwrap_or((1, 1));
    it.wide = j.get("wide").and_then(Json::as_bool).unwrap_or(false);
    Ok(n)
}

// ----------------------------------------------------------------------
// Имена и команды
// ----------------------------------------------------------------------

const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
    "move", "mut", "pub", "ref", "return", "self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "async",
    "await", "dyn", "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "d",
    "w", "ui",
];

/// Годится ли для имени переменной: `snake_case`, не ключевое слово и не
/// занятые сгенерированным кодом `d`, `w`, `ui`.
pub fn is_ident(s: &str) -> bool {
    let mut ch = s.chars();
    matches!(ch.next(), Some(c) if c.is_ascii_lowercase() || c == '_')
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !KEYWORDS.contains(&s)
        && s != "_"
}

/// Годится ли для имени типа: `CamelCase` латиницей.
pub fn is_type_name(s: &str) -> bool {
    matches!(s.chars().next(), Some(c) if c.is_ascii_uppercase()) && s.chars().all(|c| c.is_ascii_alphanumeric()) && s != "Self"
}

/// Стандартные команды кнопок: имя в файле → константа в коде и номер.
pub const BUILTIN_COMMANDS: &[(&str, &str, u16)] = &[
    ("OK", "cm::OK", tvision::cm::OK),
    ("CANCEL", "cm::CANCEL", tvision::cm::CANCEL),
    ("YES", "cm::YES", tvision::cm::YES),
    ("NO", "cm::NO", tvision::cm::NO),
    ("CLOSE", "cm::CLOSE", tvision::cm::CLOSE),
    ("HELP", "cm::HELP", tvision::cm::HELP),
];

/// Свои команды формы (`CM_…`) в порядке появления и их номера.
pub fn custom_commands(doc: &FormDoc) -> Vec<(String, u16)> {
    let mut out: Vec<(String, u16)> = Vec::new();
    for w in doc.widgets() {
        if w.kind == WidgetKind::Button
            && BUILTIN_COMMANDS.iter().all(|b| b.0 != w.command)
            && is_command_name(&w.command)
            && !out.iter().any(|c| c.0 == w.command)
        {
            let v = doc.command_base as usize + out.len();
            out.push((w.command.clone(), v.min(255) as u16));
        }
    }
    out
}

/// Своя команда: `CM_` и заглавные латинские буквы, цифры, `_`.
pub fn is_command_name(s: &str) -> bool {
    s.len() > 3 && s.starts_with("CM_") && s.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

/// Номер команды кнопки.
pub fn command_value(doc: &FormDoc, cmd: &str) -> Option<u16> {
    if let Some(b) = BUILTIN_COMMANDS.iter().find(|b| b.0 == cmd) {
        return Some(b.2);
    }
    custom_commands(doc).into_iter().find(|c| c.0 == cmd).map(|c| c.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> FormDoc {
        let mut d = FormDoc::new("LoginDialog");
        let form = d.insert(&[], Node::layout(LayoutKind::Form));
        let mut lbl = Widget::new(WidgetKind::Label, "label1");
        lbl.link = "user".into();
        d.insert(&form, Node::widget(lbl));
        d.insert(&form, Node::widget(Widget::new(WidgetKind::InputLine, "user")));
        d.insert(&[], Node::stretch(1));
        let row = d.insert(&[], Node::layout(LayoutKind::HBox));
        d.insert(&row, Node::stretch(1));
        let mut ok = Widget::new(WidgetKind::Button, "ok");
        ok.command = "CM_LOGIN".into();
        ok.default = true;
        d.insert(&row, Node::widget(ok));
        d
    }

    #[test]
    fn roundtrip_file() {
        let d = sample();
        let text = d.to_text();
        assert!(text.contains("\"type\": \"form\"") && text.contains("\"link\": \"user\""), "{}", text);
        assert_eq!(FormDoc::parse(&text).unwrap(), d);
        assert!(FormDoc::parse("{\"name\": 1}").is_err());
        assert!(FormDoc::parse("{\"tvcreator\": 1, \"layout\": {\"type\": \"button\", \"name\": \"b\"}}").is_err());
    }

    #[test]
    fn insert_remove_shift_wrap() {
        let mut d = sample();
        // Выбран виджет — новый встаёт после него.
        let user = d.find_widget("user").unwrap();
        assert_eq!(user, vec![0, 1]);
        let p = d.insert(&user, Node::widget(Widget::new(WidgetKind::Button, "b2")));
        assert_eq!(p, vec![0, 2]);
        // Вверх и вниз.
        assert_eq!(d.shift(&p, -1), Some(vec![0, 1]));
        assert_eq!(d.shift(&[0, 0], -1), None);
        // Обернуть в строку: уточнения переходят к обёртке.
        d.node_mut(&[0, 1]).unwrap().item.stretch = 2;
        let w = d.wrap(&[0, 1], LayoutKind::HBox);
        assert_eq!(d.node(&w).unwrap().layout_kind(), Some(LayoutKind::HBox));
        assert_eq!(d.node(&w).unwrap().item.stretch, 2);
        assert_eq!(d.node(&[0, 1, 0]).unwrap().as_widget().unwrap().name, "b2");
        // Удалить: выбирается сосед.
        assert_eq!(d.remove(&[0, 1]), Some(vec![0, 1]));
        assert_eq!(d.remove(&[]), None);
    }

    #[test]
    fn names_and_commands() {
        let mut d = sample();
        assert_eq!(d.unique_name("button"), "button1");
        assert_eq!(command_value(&d, "OK"), Some(tvision::cm::OK));
        assert_eq!(custom_commands(&d), vec![("CM_LOGIN".to_string(), 100)]);
        d.rename("user", "login");
        assert_eq!(d.widgets()[0].link, "login");
        assert!(d.problems().is_empty(), "{:?}", d.problems());
        d.node_mut(&[0, 1]).unwrap().as_widget_mut().unwrap().name = "ok".into();
        d.node_mut(&[2, 1]).unwrap().as_widget_mut().unwrap().command = "save".into();
        let p = d.problems();
        assert!(
            p.iter().any(|x| x.contains("дважды")) && p.iter().any(|x| x.contains("команда")) && p.iter().any(|x| x.contains("несуществующий")),
            "{:?}",
            p
        );
        assert!(is_ident("user_name2") && !is_ident("Name") && !is_ident("fn") && !is_ident("2a"));
        assert!(is_type_name("LoginDialog") && !is_type_name("login"));
    }
}
