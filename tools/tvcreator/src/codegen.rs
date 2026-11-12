//! Генерация Rust-кода по форме — как `uic` у Qt.
//!
//! Получается модуль со структурой, в которой id всех виджетов, и
//! функцией `create()`, которая строит окно: виджеты, раскладку, размер,
//! фокус. Подключается обычным `mod`:
//!
//! ```text
//! mod login_dialog;                         // файл login_dialog.rs из TVCreator
//! use login_dialog::LoginDialog;
//!
//! let (d, ui) = LoginDialog::create();
//! let (cmd, d) = app.exec_dialog(d);
//! let name = d.win.group.find::<InputLine>(ui.user).unwrap().text();
//! ```

use crate::build::{form_rows, grid_cells, FormRow};
use crate::model::*;
use tvision::layout::Alignment;

/// Строковый литерал Rust.
fn lit(s: &str) -> String {
    format!("{:?}", s)
}

fn align(a: Alignment) -> &'static str {
    match a {
        Alignment::Fill => "Alignment::Fill",
        Alignment::Start => "Alignment::Start",
        Alignment::Center => "Alignment::Center",
        Alignment::End => "Alignment::End",
    }
}

/// `x` или `item(x).stretch(2)…` — в том же порядке, что `build::apply_item`.
fn item_expr(x: &str, p: &ItemProps) -> String {
    if p.is_plain() {
        return x.to_string();
    }
    let mut s = format!("item({})", x);
    if p.stretch > 0 {
        s += &format!(".stretch({})", p.stretch);
    }
    if p.expand {
        s += ".expand()";
    }
    if p.align_h != Alignment::Fill {
        s += &format!(".align_h({})", align(p.align_h));
    }
    if p.align_v != Alignment::Fill {
        s += &format!(".align_v({})", align(p.align_v));
    }
    if let Some(v) = p.min_w {
        s += &format!(".min_width({})", v);
    }
    if let Some(v) = p.min_h {
        s += &format!(".min_height({})", v);
    }
    if let Some(v) = p.fixed_w {
        s += &format!(".fixed_width({})", v);
    }
    if let Some(v) = p.fixed_h {
        s += &format!(".fixed_height({})", v);
    }
    s
}

/// Куда писать код формы: рядом с файлом формы, имя — по имени типа
/// (`forms/login.tvui` с `LoginDialog` → `forms/login_dialog.rs`).
pub fn output_path(form: &std::path::Path, doc: &FormDoc) -> std::path::PathBuf {
    form.with_file_name(format!("{}.rs", module_name(&doc.name)))
}

/// Имя файла модуля для формы: `LoginDialog` → `login_dialog`.
pub fn module_name(type_name: &str) -> String {
    let mut out = String::new();
    for (i, c) in type_name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn widget_ctor(doc: &FormDoc, w: &Widget, link: Option<&str>) -> String {
    let items = || w.items.iter().map(|s| lit(s)).collect::<Vec<_>>().join(", ");
    match w.kind {
        WidgetKind::Label => {
            format!("Label::new(Rect::default(), {}, {})", lit(&w.text), link.map_or("None".to_string(), |l| format!("Some({})", l)))
        }
        WidgetKind::StaticText => format!("StaticText::new(Rect::default(), {})", lit(&w.text)),
        WidgetKind::InputLine => format!("InputLine::new(Rect::default(), {})", w.max_len),
        WidgetKind::Button => {
            let cmd = if let Some(b) = BUILTIN_COMMANDS.iter().find(|b| b.0 == w.command) {
                b.1.to_string()
            } else if command_value(doc, &w.command).is_some() {
                w.command.clone()
            } else {
                "cm::OK".to_string()
            };
            format!("Button::new(Rect::default(), {}, {}, {})", lit(&w.text), cmd, if w.default { "bf::DEFAULT" } else { "bf::NORMAL" })
        }
        WidgetKind::CheckBoxes => format!("Cluster::check_boxes(Rect::default(), &[{}])", items()),
        WidgetKind::RadioButtons => format!("Cluster::radio_buttons(Rect::default(), &[{}])", items()),
        WidgetKind::ListBox => {
            let v = w.items.iter().map(|s| format!("{}.into()", lit(s))).collect::<Vec<_>>().join(", ");
            format!(
                "{{\n            let mut l = ListBox::new(Rect::default(), 1, None);\n            l.set_items(vec![{}]);\n            l\n        }}",
                v
            )
        }
        WidgetKind::Table => format!("Table::new(Rect::default(), Box::new(VecTableModel::new(&[{}], vec![])), None, None)", items()),
        WidgetKind::ProgressBar => format!("ProgressBar::new(Rect::default(), {})", w.max),
        WidgetKind::Line => format!(
            "Line::{}(Rect::default()){}{}",
            if w.vertical { "vertical" } else { "horizontal" },
            if w.double { ".double()" } else { "" },
            if w.to_frame { ".to_frame()" } else { "" }
        ),
    }
}

fn widget_doc(w: &Widget) -> String {
    let what = w.kind.title().split(" (").next().unwrap_or("").to_string();
    let text = match w.kind {
        WidgetKind::Label | WidgetKind::StaticText | WidgetKind::Button => w.text.replace('~', ""),
        _ => String::new(),
    };
    let text = text.lines().next().unwrap_or("").to_string();
    if text.is_empty() {
        what
    } else {
        format!("{} «{}»", what, text)
    }
}

struct Gen<'a> {
    doc: &'a FormDoc,
    /// Строки с объявлениями раскладок (внутренние — раньше внешних).
    lines: Vec<String>,
    used: Vec<String>,
    counters: [usize; 4],
}

impl Gen<'_> {
    fn fresh(&mut self, kind: LayoutKind) -> String {
        let k = LayoutKind::ALL.iter().position(|x| *x == kind).unwrap();
        loop {
            self.counters[k] += 1;
            let name = format!("{}{}", kind.key(), self.counters[k]);
            if !self.used.contains(&name) {
                self.used.push(name.clone());
                return name;
            }
        }
    }

    /// Выражение для элемента-ребёнка (у раскладки — имя её переменной).
    fn child_expr(&mut self, n: &Node) -> String {
        match &n.kind {
            NodeKind::Widget(w) => item_expr(&w.name, &n.item),
            NodeKind::Layout { .. } => {
                let v = self.layout(n);
                item_expr(&v, &n.item)
            }
            NodeKind::Stretch(f) => format!("item(Spacer::horizontal()).stretch({})", f),
            NodeKind::Space(s) => format!("Spacer::fixed({}, {})", s, s),
        }
    }

    /// Объявить переменную раскладки; вернуть её имя.
    fn layout(&mut self, n: &Node) -> String {
        let NodeKind::Layout { kind, spacing, margins, columns } = &n.kind else { unreachable!() };
        // Сначала дети (их переменные должны быть объявлены раньше).
        let mut calls: Vec<String> = Vec::new();
        if let Some(s) = spacing {
            calls.push(format!(".spacing({})", s));
        }
        if *margins != [0; 4] {
            calls.push(format!(".margins({}, {}, {}, {})", margins[0], margins[1], margins[2], margins[3]));
        }
        match kind {
            LayoutKind::HBox | LayoutKind::VBox => {
                for c in &n.children {
                    calls.push(match c.kind {
                        NodeKind::Stretch(f) => format!(".stretch({})", f),
                        NodeKind::Space(s) => format!(".space({})", s),
                        _ => format!(".add({})", self.child_expr(c)),
                    });
                }
            }
            LayoutKind::Grid => {
                let cells = grid_cells(&n.children, *columns);
                for (i, c) in n.children.iter().enumerate() {
                    let (r, col) = cells[i];
                    let e = self.child_expr(c);
                    calls.push(if c.item.span == (1, 1) {
                        format!(".cell({}, {}, {})", r, col, e)
                    } else {
                        format!(".span({}, {}, {}, {}, {})", r, col, c.item.span.0, c.item.span.1, e)
                    });
                }
            }
            LayoutKind::Form => {
                for row in form_rows(&n.children) {
                    calls.push(match row {
                        FormRow::Pair(a, b) => {
                            let x = self.child_expr(&n.children[a]);
                            let y = self.child_expr(&n.children[b]);
                            format!(".row({}, {})", x, y)
                        }
                        FormRow::Wide(a) => format!(".row_wide({})", self.child_expr(&n.children[a])),
                    });
                }
            }
        }
        let name = self.fresh(*kind);
        let mut s = format!("        let {} = {}::new()", name, kind.rust());
        for c in calls {
            s += &format!("\n            {}", c);
        }
        s += ";";
        self.lines.push(s);
        name
    }
}

/// Код для `include!` из `build.rs`: без внутренних атрибутов `#![…]` и
/// `//!` (их `include!` не принимает), `allow` — у каждого элемента.
///
/// ```text
/// // build.rs
/// let doc = tvcreator::model::FormDoc::load("forms/login.tvui".as_ref()).unwrap();
/// let out = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("login_dialog.rs");
/// std::fs::write(out, tvcreator::codegen::generate_for_include(&doc, "login.tvui")).unwrap();
/// println!("cargo:rerun-if-changed=forms/login.tvui");
///
/// // main.rs
/// mod login_dialog {
///     include!(concat!(env!("OUT_DIR"), "/login_dialog.rs"));
/// }
/// ```
pub fn generate_for_include(doc: &FormDoc, source: &str) -> String {
    let full = generate(doc, source);
    let mut out = String::new();
    for line in full.lines() {
        if let Some(rest) = line.strip_prefix("//!") {
            out += "//";
            out += rest;
        } else if line == "#![allow(dead_code)]" {
            continue;
        } else if line.starts_with("pub const ") || line.starts_with("pub struct ") || line.starts_with("impl ") {
            out += "#[allow(dead_code)]\n";
            out += line;
        } else {
            out += line;
        }
        out += "\n";
    }
    out
}

/// Текст модуля Rust для формы. `source` — имя файла формы (для
/// комментария в начале).
pub fn generate(doc: &FormDoc, source: &str) -> String {
    let mut o = String::new();
    let what = if doc.kind == FormKind::Dialog { "Dialog" } else { "Window" };
    o += &format!("//! Форма «{}» — сгенерировано TVCreator из «{}».\n", doc.title, source);
    o += "//!\n//! Не правьте этот файл вручную: изменения пропадут при следующей\n//! генерации. Меняйте форму в TVCreator и генерируйте код заново.\n//!\n";
    o += &format!("//! ```text\n//! let (view, ui) = {}::create();\n", doc.name);
    if doc.kind == FormKind::Dialog {
        o += "//! let (cmd, view) = app.exec_dialog(view);\n";
    } else {
        o += "//! app.insert_window(view);\n";
    }
    o += "//! ```\n\n#![allow(dead_code)]\n\nuse tvision::layout::*;\nuse tvision::*;\n";

    let cmds = custom_commands(doc);
    if !cmds.is_empty() {
        o += "\n";
        for (name, v) in &cmds {
            let btn = doc
                .widgets()
                .into_iter()
                .find(|w| w.kind == WidgetKind::Button && &w.command == name)
                .map(|w| w.text.replace('~', ""))
                .unwrap_or_default();
            o += &format!("/// Команда кнопки «{}».\npub const {}: u16 = {};\n", btn, name, v);
        }
    }

    // Структура с id.
    let widgets = doc.widgets();
    o += &format!("\n/// Элементы формы «{}»: id для `find` / `find_mut`.\npub struct {} {{\n", doc.title, doc.name);
    for w in &widgets {
        o += &format!("    /// {}.\n    pub {}: ViewId,\n", widget_doc(w), w.name);
    }
    o += "}\n";

    // create()
    let var = if doc.kind == FormKind::Dialog { "d" } else { "w" };
    o += &format!(
        "\nimpl {} {{\n    /// Создать {} «{}».\n    pub fn create() -> ({}, {}) {{\n",
        doc.name,
        if doc.kind == FormKind::Dialog { "диалог" } else { "окно" },
        doc.title,
        what,
        doc.name
    );
    let (w, h) = doc.size;
    if doc.kind == FormKind::Dialog {
        o += &format!("        let mut d = Dialog::centered({}, {}, {});\n", w, h, lit(&doc.title));
    } else {
        o += &format!("        let mut w = Window::new(Rect::new(0, 0, {}, {}), {}, 0);\n", w, h, lit(&doc.title));
    }
    // Виджеты: сначала всё, кроме подписей, потом подписи (как build).
    let mut defined: Vec<&str> = Vec::new();
    for pass in 0..2 {
        for wd in &widgets {
            if (wd.kind == WidgetKind::Label) != (pass == 1) {
                continue;
            }
            let link = defined.iter().find(|n| **n == wd.link).copied();
            o += &format!("        let {} = {}.add({});\n", wd.name, var, widget_ctor(doc, wd, link));
            defined.push(&wd.name);
        }
    }
    // Раскладки.
    let mut g = Gen { doc, lines: Vec::new(), used: widgets.iter().map(|w| w.name.clone()).collect(), counters: [0; 4] };
    let root = g.layout(&doc.root);
    let _ = g.doc;
    for l in &g.lines {
        o += l;
        o += "\n";
    }
    o += &format!("        {}.set_layout({});\n", var, root);
    if doc.auto_size {
        o += &format!("        {}.adjust_size();\n", var);
    }
    if let Some(first) = widgets.iter().find(|w| w.kind.selectable()) {
        o += &format!("        {}.select({});\n", var, first.name);
    }
    let fields = widgets.iter().map(|w| w.name.as_str()).collect::<Vec<_>>().join(", ");
    o += &format!("        ({}, {} {{ {} }})\n    }}\n}}\n", var, doc.name, fields);
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert_eq!(module_name("LoginDialog"), "login_dialog");
        assert_eq!(module_name("Form1"), "form1");
        assert_eq!(lit("«а» \"b\"\n"), "\"«а» \\\"b\\\"\\n\"");
    }

    #[test]
    fn shape_of_code() {
        let mut d = FormDoc::new("Demo");
        let row = d.insert(&[], Node::layout(LayoutKind::HBox));
        d.insert(&row, Node::stretch(1));
        let mut b = Widget::new(WidgetKind::Button, "save");
        b.command = "CM_SAVE".into();
        b.text = "~С~охранить".into();
        d.insert(&row, Node::widget(b));
        d.node_mut(&[0]).unwrap().item.stretch = 2;
        let code = generate(&d, "demo.tvui");
        assert!(code.contains("pub const CM_SAVE: u16 = 100;"), "{}", code);
        assert!(code.contains("let save = d.add(Button::new(Rect::default(), \"~С~охранить\", CM_SAVE, bf::NORMAL));"), "{}", code);
        assert!(code.contains("let hbox1 = HBox::new()\n            .stretch(1)\n            .add(save);"), "{}", code);
        assert!(code.contains(".add(item(hbox1).stretch(2));"), "{}", code);
        assert!(code.contains("d.set_layout(vbox1);\n        d.adjust_size();\n        d.select(save);"), "{}", code);
    }
}
