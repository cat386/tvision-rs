//! Главное окно среды и палитра виджетов.

use super::*;

/// Пункты палитры: раскладки, распорки, виджеты.
pub fn palette_items() -> Vec<String> {
    let mut v: Vec<String> = vec![
        "▤ Строка (HBox)".into(),
        "▥ Столбец (VBox)".into(),
        "▦ Сетка (Grid)".into(),
        "☰ Форма (Form)".into(),
        "↔ Пружина (stretch)".into(),
        "· Промежуток (space)".into(),
    ];
    v.extend(WidgetKind::ALL.iter().map(|k| format!("  {}", k.title())));
    v
}

/// Новый узел для пункта палитры.
pub fn palette_node(doc: &FormDoc, i: usize) -> Option<Node> {
    Some(match i {
        0 => Node::layout(LayoutKind::HBox),
        1 => Node::layout(LayoutKind::VBox),
        2 => Node::layout(LayoutKind::Grid),
        3 => Node::layout(LayoutKind::Form),
        4 => Node::stretch(1),
        5 => Node::space(1),
        _ => {
            let k = *WidgetKind::ALL.get(i - 6)?;
            Node::widget(Widget::new(k, &doc.unique_name(k.prefix())))
        }
    })
}

// ----------------------------------------------------------------------
// Главное окно
// ----------------------------------------------------------------------

/// Главное окно: Enter и двойной щелчок в палитре — «добавить», в
/// свойствах — «изменить»; при несохранённой форме выход сначала
/// спрашивает.
pub struct MainWindow {
    pub win: Window,
    pub palette: ViewId,
    pub tree: ViewId,
    pub canvas: ViewId,
    pub props: ViewId,
    /// Есть несохранённые изменения (общий с [`Ide`]).
    pub dirty: Rc<Cell<bool>>,
}

impl MainWindow {
    pub fn new(r: Rect, dirty: Rc<Cell<bool>>) -> Self {
        let mut win = Window::new(r, "TVCreator", 0);
        win.flags &= !wf::CLOSE; // главное окно не закрывается
        let h1 = win.add(StaticText::new(Rect::default(), "Виджеты"));
        let mut pal = ListBox::new(Rect::default(), 1, None);
        pal.set_items(palette_items());
        let palette = win.add(pal);
        let h2 = win.add(StaticText::new(Rect::default(), "Объекты"));
        let tree = win.add(TreeView::new(Rect::default(), vec![], None, None));
        let h3 = win.add(StaticText::new(Rect::default(), "Форма"));
        let canvas = win.add(Canvas::new());
        let h4 = win.add(StaticText::new(Rect::default(), "Свойства"));
        let table = Table::new(Rect::default(), Box::new(VecTableModel::new(&["Свойство", "Значение"], vec![])), None, None)
            .with_columns(vec![TableColumn::new(30), TableColumn::new(40)])
            .selection(SelectionMode::Row);
        let props = win.add(table);
        // Само окно — тоже раскладкой: левая колонка, холст, свойства 1 : 3 : 2.
        win.set_layout(
            HBox::new()
                .add(item(VBox::new().add(h1).add(item(palette).stretch(1)).space(1).add(h2).add(item(tree).stretch(2))).stretch(1).min_width(30))
                .add(item(VBox::new().add(h3).add(item(canvas).stretch(1))).stretch(3))
                .add(item(VBox::new().add(h4).add(item(props).stretch(1))).stretch(2).min_width(40)),
        );
        MainWindow { win, palette, tree, canvas, props, dirty }
    }

    fn current_id(&self) -> Option<ViewId> {
        self.win.group.current.map(|i| self.win.group.children[i].vs().id)
    }
}

impl View for MainWindow {
    tvision::impl_group_basics!(win.group);

    fn as_window(&self) -> Option<&Window> {
        Some(&self.win)
    }

    fn as_window_mut(&mut self) -> Option<&mut Window> {
        Some(&mut self.win)
    }
    fn draw(&self, dc: &mut DrawCtx) {
        self.win.draw_window(dc);
    }
    fn palette(&self) -> &'static [u8] {
        self.win.palette
    }
    fn set_state(&mut self, flag: u16, enable: bool) {
        self.win.set_window_state(flag, enable);
    }
    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        let double = matches!(ev, Event::MouseDown(m) if m.double);
        self.win.handle_window_event(ev, ctx);
        let enter = matches!(ev, Event::KeyDown(k) if k.key == Key::Enter && k.mods == Mods::NONE);
        if enter || double {
            let cur = self.current_id();
            if cur == Some(self.palette) {
                ctx.post_command(CM_ADD, Info::None);
                ev.clear();
            } else if cur == Some(self.props) {
                ctx.post_command(CM_PROP_EDIT, Info::None);
                ev.clear();
            }
        }
    }
    fn valid(&mut self, command: u16, ctx: &mut Ctx) -> bool {
        if command == cm::QUIT && self.dirty.get() {
            ctx.post_command(CM_EXIT, Info::None); // спросить «Сохранить?»
            return false;
        }
        self.win.group.valid(command, ctx)
    }
}
