//! Сборка формы «на лету»: по документу создаются настоящие виджеты
//! tvision-rs и раскладка. Так холст дизайнера и предпросмотр показывают
//! ровно то, что получится из сгенерированного кода ([`crate::codegen`]):
//! оба идут по дереву в одном порядке (тест `generated_matches_runtime`
//! проверяет это до клетки).

use tvision::layout::{item, Form, Grid, HBox, Item, Layout, VBox};
use tvision::*;

use crate::model::Node;
use crate::model::*;

/// Собранная форма.
pub struct Built {
    /// `Dialog` или `Window`.
    pub view: Box<dyn View>,
    /// Путь узла → id виджета.
    pub ids: Vec<(Vec<usize>, ViewId)>,
}

impl Built {
    /// Путь виджета по id.
    pub fn path_of(&self, id: ViewId) -> Option<&[usize]> {
        self.ids.iter().find(|(_, i)| *i == id).map(|(p, _)| p.as_slice())
    }
    /// Id виджета по пути.
    pub fn id_of(&self, path: &[usize]) -> Option<ViewId> {
        self.ids.iter().find(|(p, _)| p == path).map(|(_, i)| *i)
    }
    /// Окно формы (у диалога — его окно).
    pub fn window(&self) -> &Window {
        self.view.as_window().expect("форма — окно")
    }
    pub fn group(&self) -> &Group {
        self.view.as_group().expect("форма — группа")
    }
}

/// Создать виджет по описанию.
pub fn make_widget(doc: &FormDoc, w: &Widget, link: Option<ViewId>) -> Box<dyn View> {
    let r = Rect::default();
    let items: Vec<&str> = w.items.iter().map(String::as_str).collect();
    match w.kind {
        WidgetKind::Label => Box::new(Label::new(r, &w.text, link)),
        WidgetKind::StaticText => Box::new(StaticText::new(r, &w.text)),
        WidgetKind::InputLine => Box::new(InputLine::new(r, w.max_len)),
        WidgetKind::Button => {
            let cmd = command_value(doc, &w.command).unwrap_or(cm::OK);
            Box::new(Button::new(r, &w.text, cmd, if w.default { bf::DEFAULT } else { bf::NORMAL }))
        }
        WidgetKind::CheckBoxes => Box::new(Cluster::check_boxes(r, &items)),
        WidgetKind::RadioButtons => Box::new(Cluster::radio_buttons(r, &items)),
        WidgetKind::ListBox => {
            let mut l = ListBox::new(r, 1, None);
            l.set_items(w.items.clone());
            Box::new(l)
        }
        WidgetKind::Table => Box::new(Table::new(r, Box::new(VecTableModel::new(&items, vec![])), None, None)),
        WidgetKind::ProgressBar => Box::new(ProgressBar::new(r, w.max)),
        WidgetKind::Line => {
            let mut l = if w.vertical { Line::vertical(r) } else { Line::horizontal(r) };
            if w.double {
                l = l.double();
            }
            l.to_frame = w.to_frame;
            Box::new(l)
        }
    }
}

/// Применить уточнения элемента.
pub fn apply_item(mut it: Item, p: &ItemProps) -> Item {
    if p.stretch > 0 {
        it = it.stretch(p.stretch);
    }
    if p.expand {
        it = it.expand();
    }
    if p.align_h != layout::Alignment::Fill {
        it = it.align_h(p.align_h);
    }
    if p.align_v != layout::Alignment::Fill {
        it = it.align_v(p.align_v);
    }
    if let Some(v) = p.min_w {
        it = it.min_width(v);
    }
    if let Some(v) = p.min_h {
        it = it.min_height(v);
    }
    if let Some(v) = p.fixed_w {
        it = it.fixed_width(v);
    }
    if let Some(v) = p.fixed_h {
        it = it.fixed_height(v);
    }
    it
}

/// Клетки детей сетки: явные или по порядку, `columns` в строке.
pub fn grid_cells(children: &[Node], columns: usize) -> Vec<(usize, usize)> {
    let columns = columns.max(1);
    let (mut r, mut c) = (0usize, 0usize);
    children
        .iter()
        .map(|n| match n.item.cell {
            Some(rc) => rc,
            None => {
                if c >= columns {
                    r += 1;
                    c = 0;
                }
                let at = (r, c);
                c += n.item.span.1.max(1);
                at
            }
        })
        .collect()
}

/// Строки формы: пары (подпись, поле) или элемент на всю ширину.
pub enum FormRow {
    Pair(usize, usize),
    Wide(usize),
}

pub fn form_rows(children: &[Node]) -> Vec<FormRow> {
    let mut out = Vec::new();
    let mut pending: Option<usize> = None;
    for (i, n) in children.iter().enumerate() {
        if n.item.wide {
            if let Some(p) = pending.take() {
                out.push(FormRow::Wide(p));
            }
            out.push(FormRow::Wide(i));
        } else if let Some(p) = pending.take() {
            out.push(FormRow::Pair(p, i));
        } else {
            pending = Some(i);
        }
    }
    if let Some(p) = pending {
        out.push(FormRow::Wide(p));
    }
    out
}

/// Элемент раскладки для узла. У пружин и промежутков уточнений нет.
fn to_item(doc_node: &Node, path: &mut Vec<usize>, ids: &[(Vec<usize>, ViewId)]) -> Option<Item> {
    Some(match &doc_node.kind {
        NodeKind::Widget(_) => apply_item(item(ids.iter().find(|(p, _)| p == path)?.1), &doc_node.item),
        NodeKind::Layout { .. } => apply_item(item(to_layout(doc_node, path, ids)), &doc_node.item),
        NodeKind::Stretch(f) => item(layout::Spacer::horizontal()).stretch(*f),
        NodeKind::Space(n) => item(layout::Spacer::fixed(*n, *n)),
    })
}

/// Раскладка узла.
pub fn to_layout(n: &Node, path: &mut Vec<usize>, ids: &[(Vec<usize>, ViewId)]) -> Layout {
    let NodeKind::Layout { kind, spacing, margins, columns } = &n.kind else { panic!("не раскладка") };
    let mut l = match kind {
        LayoutKind::HBox => HBox::new(),
        LayoutKind::VBox => VBox::new(),
        LayoutKind::Grid => Grid::new(),
        LayoutKind::Form => Form::new(),
    };
    if let Some(s) = spacing {
        l = l.spacing(*s);
    }
    if *margins != [0; 4] {
        l = l.margins(margins[0], margins[1], margins[2], margins[3]);
    }
    let child = |i: usize, path: &mut Vec<usize>| -> Option<Item> {
        path.push(i);
        let it = to_item(&n.children[i], path, ids);
        path.pop();
        it
    };
    match kind {
        LayoutKind::HBox | LayoutKind::VBox => {
            for (i, c) in n.children.iter().enumerate() {
                match c.kind {
                    // Пружины и промежутки — как в коде: .stretch(n) / .space(n).
                    NodeKind::Stretch(f) => l = l.stretch(f),
                    NodeKind::Space(s) => l = l.space(s),
                    _ => {
                        if let Some(it) = child(i, path) {
                            l = l.add(it);
                        }
                    }
                }
            }
        }
        LayoutKind::Grid => {
            let cells = grid_cells(&n.children, *columns);
            for (i, c) in n.children.iter().enumerate() {
                let (r, col) = cells[i];
                if let Some(it) = child(i, path) {
                    l = l.span(r, col, c.item.span.0, c.item.span.1, it);
                }
            }
        }
        LayoutKind::Form => {
            for row in form_rows(&n.children) {
                match row {
                    FormRow::Pair(a, b) => {
                        if let (Some(x), Some(y)) = (child(a, path), child(b, path)) {
                            l = l.row(x, y);
                        }
                    }
                    FormRow::Wide(a) => {
                        if let Some(x) = child(a, path) {
                            l = l.row_wide(x);
                        }
                    }
                }
            }
        }
    }
    l
}

/// Собрать форму.
pub fn build(doc: &FormDoc) -> Built {
    let (w, h) = doc.size;
    let mut dialog = None;
    let mut window = None;
    match doc.kind {
        FormKind::Dialog => dialog = Some(Dialog::centered(w, h, &doc.title)),
        FormKind::Window => window = Some(Window::new(Rect::new(0, 0, w, h), &doc.title, 0)),
    }
    {
        let win: &mut Window = match (&mut dialog, &mut window) {
            (Some(d), _) => &mut d.win,
            (_, Some(w)) => w,
            _ => unreachable!(),
        };
        let mut ids: Vec<(Vec<usize>, ViewId)> = Vec::new();
        let mut by_name: Vec<(String, ViewId)> = Vec::new();
        let nodes = doc.walk();
        // Сначала всё, кроме подписей (порядок Tab — порядок дерева),
        // потом подписи: им нужен id того, что они подписывают.
        for pass in 0..2 {
            for (path, n) in &nodes {
                let Some(wd) = n.as_widget() else { continue };
                if (wd.kind == WidgetKind::Label) != (pass == 1) {
                    continue;
                }
                let link = by_name.iter().find(|(nm, _)| *nm == wd.link).map(|(_, id)| *id);
                let id = win.insert(make_widget(doc, wd, link));
                ids.push((path.clone(), id));
                by_name.push((wd.name.clone(), id));
            }
        }
        let lay = to_layout(&doc.root, &mut Vec::new(), &ids);
        win.set_layout(lay);
        if doc.auto_size {
            win.adjust_size();
        }
        let first = doc.walk().into_iter().find(|(_, n)| n.as_widget().is_some_and(|w| w.kind.selectable())).map(|(p, _)| p);
        if let Some(id) = first.and_then(|p| ids.iter().find(|(q, _)| *q == p).map(|x| x.1)) {
            win.select(id);
        }
        let view: Box<dyn View> = match (dialog, window) {
            (Some(d), _) => Box::new(d),
            (_, Some(w)) => Box::new(w),
            _ => unreachable!(),
        };
        Built { view, ids }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_flow_and_form_rows() {
        let mut kids: Vec<Node> = (0..5).map(|_| Node::space(1)).collect();
        kids[1].item.span = (1, 2);
        assert_eq!(grid_cells(&kids, 3), vec![(0, 0), (0, 1), (1, 0), (1, 1), (1, 2)]);
        kids[4].item.cell = Some((5, 0));
        assert_eq!(grid_cells(&kids, 3)[4], (5, 0));
        let mut f: Vec<Node> = (0..5).map(|_| Node::space(1)).collect();
        f[2].item.wide = true;
        let rows: Vec<String> = form_rows(&f)
            .iter()
            .map(|r| match r {
                FormRow::Pair(a, b) => format!("{}+{}", a, b),
                FormRow::Wide(a) => format!("{}", a),
            })
            .collect();
        assert_eq!(rows, ["0+1", "2", "3+4"]);
    }

    #[test]
    fn builds_dialog_with_links() {
        let mut d = FormDoc::new("Test");
        let f = d.insert(&[], Node::layout(LayoutKind::Form));
        let mut l = Widget::new(WidgetKind::Label, "label1");
        l.link = "name".into();
        d.insert(&f, Node::widget(l));
        d.insert(&f, Node::widget(Widget::new(WidgetKind::InputLine, "name")));
        let b = build(&d);
        assert_eq!(b.ids.len(), 2);
        let input = b.id_of(&[0, 1]).unwrap();
        let label = b.group().view(b.id_of(&[0, 0]).unwrap()).unwrap().as_any().downcast_ref::<Label>().unwrap();
        assert_eq!(label.link, Some(input));
        assert_eq!(b.group().current.map(|i| b.group().children[i].vs().id), Some(input), "фокус — на первом поле");
        // Размер — по содержимому: поле 1 + «Подпись:» 9 + 1 + строка 30 + поле 1 + рамка 2.
        assert_eq!(b.group().vs.size.x, 44);
    }
}
