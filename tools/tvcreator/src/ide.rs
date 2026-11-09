//! Среда TVCreator: палитра, дерево объектов, холст, свойства — как в
//! Qt Creator (режим «Дизайн»).

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use tvision::layout::{item, HBox, VBox};
use tvision::*;

use crate::build::build;
use crate::canvas::Canvas;
use crate::codegen;
use crate::model::Node;
use crate::model::*;
use crate::props::{next_value, props, set_prop, Prop, Sel};
use crate::themes::{load_themes, resolve, theme_dirs, ThemeEntry};
use tvision::color::Attr;
use tvision::json::Json;
use tvision::settings::{settings_dialog_with, standard_palette, LanguageChoice, Settings};
use tvision::theme::{choose_theme, Theme};

// ----------------------------------------------------------------------
// Команды
// ----------------------------------------------------------------------

pub const CM_NEW: u16 = 100;
pub const CM_OPEN: u16 = 101;
pub const CM_SAVE: u16 = 102;
pub const CM_SAVE_AS: u16 = 103;
pub const CM_GENERATE: u16 = 104;
pub const CM_EXIT: u16 = 105;
pub const CM_ADD: u16 = 110;
pub const CM_DELETE: u16 = 111;
pub const CM_UP: u16 = 112;
pub const CM_DOWN: u16 = 113;
pub const CM_WRAP_H: u16 = 114;
pub const CM_WRAP_V: u16 = 115;
pub const CM_WRAP_GRID: u16 = 116;
pub const CM_WRAP_FORM: u16 = 117;
pub const CM_UNDO: u16 = 118;
pub const CM_REDO: u16 = 119;
pub const CM_PROP_EDIT: u16 = 120;
pub const CM_PREVIEW: u16 = 121;
pub const CM_CODE: u16 = 122;
pub const CM_CHECK: u16 = 123;
pub const CM_ABOUT: u16 = 124;
pub const CM_THEME_CHOOSE: u16 = 130;
pub const CM_THEME_DEFAULT: u16 = 131;
pub const CM_THEME_OPEN: u16 = 132;
pub const CM_FORM_THEME_CHOOSE: u16 = 133;
pub const CM_FORM_THEME_SAME: u16 = 134;
pub const CM_SETTINGS: u16 = 135;
/// Тема среды из меню: 140 + номер темы.
pub const CM_IDE_THEME: u16 = 140;
/// Тема формы из меню: 180 + номер темы.
pub const CM_FORM_THEME: u16 = 180;
/// Сколько тем помещается в меню.
pub const MENU_THEMES: usize = 36;

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

// ----------------------------------------------------------------------
// Среда
// ----------------------------------------------------------------------

/// Состояние среды и обработчик команд.
pub struct Ide {
    pub doc: FormDoc,
    pub path: Option<PathBuf>,
    pub sel: Sel,
    pub window: ViewId,
    ids: (ViewId, ViewId, ViewId, ViewId),
    dirty: Rc<Cell<bool>>,
    undo: Vec<(FormDoc, Sel)>,
    redo: Vec<(FormDoc, Sel)>,
    last_tree: Option<Vec<usize>>,
    /// Последнее сообщение среды (для тестов).
    pub last_message: String,
    /// Найденные темы.
    pub themes: Vec<ThemeEntry>,
    /// Настройки среды (стандартные `tvision::settings`; тема формы — в
    /// `extra["form_theme"]`). Файл — `settings_path`.
    pub settings: Settings,
    pub settings_path: Option<PathBuf>,
}

/// Ключ настроек для темы формы.
const FORM_THEME_KEY: &str = "form_theme";

/// Настройки, если файла ещё нет: как у библиотеки, но по-русски —
/// надписи среды русские.
pub fn default_settings() -> Settings {
    Settings { language: LanguageChoice::Ru, ..Settings::default() }
}

/// Ключ главного окна в настройках: заголовок меняется вместе с именем
/// файла, поэтому окно узнаётся по началу заголовка.
pub fn window_key(w: &Window) -> Option<String> {
    w.title.starts_with("TVCreator").then(|| "tvcreator".to_string())
}

fn sel_to_tree(s: &Sel) -> Vec<usize> {
    match s {
        Sel::Form => vec![0],
        Sel::Node(p) => [vec![0, 0], p.clone()].concat(),
    }
}

fn tree_to_sel(t: &[usize]) -> Option<Sel> {
    match t {
        [0] => Some(Sel::Form),
        [0, 0, rest @ ..] => Some(Sel::Node(rest.to_vec())),
        _ => None,
    }
}

fn tree_of(n: &Node) -> TreeNode {
    TreeNode::new(&n.caption()).expanded(true).children(n.children.iter().map(tree_of).collect())
}

impl Ide {
    /// Открыть среду в программе `app` с документом `doc`.
    pub fn new(app: &mut Program, doc: FormDoc, path: Option<PathBuf>) -> Ide {
        Ide::with_themes(app, doc, path, load_themes(&[]), None)
    }

    /// Среда со списком тем и файлом настроек. Если файл задан, его
    /// настройки (тема, язык, цвета, мышь…) сразу применяются.
    pub fn with_themes(app: &mut Program, doc: FormDoc, path: Option<PathBuf>, themes: Vec<ThemeEntry>, settings_path: Option<PathBuf>) -> Ide {
        let dirty = Rc::new(Cell::new(false));
        let r = app.desktop().group.vs.extent();
        let w = MainWindow::new(r, dirty.clone());
        let ids = (w.palette, w.tree, w.canvas, w.props);
        let window = app.insert_window(w);
        let settings = settings_path.as_deref().map(|p| Settings::load_or(p, default_settings())).unwrap_or_else(default_settings);
        let applied = settings_path.as_ref().map(|_| settings.apply(app));
        let mut ide = Ide {
            doc,
            path,
            sel: Sel::Form,
            window,
            ids,
            dirty,
            undo: Vec::new(),
            redo: Vec::new(),
            last_tree: None,
            last_message: String::new(),
            themes,
            settings,
            settings_path,
        };
        if let Some(Err(e)) = applied {
            ide.last_message = e;
        }
        let ft = ide.settings.get(FORM_THEME_KEY).and_then(Json::as_str).unwrap_or("").to_string();
        ide.set_form_theme(app, (!ft.is_empty()).then(|| resolve(&ide.themes, &ft)).flatten());
        ide.refresh_all(app);
        ide
    }

    // ---------- темы ----------

    fn save_settings(&mut self) {
        if let Some(p) = &self.settings_path {
            let _ = self.settings.save(p); // не записалось — просто не запомнится
        }
    }

    /// Тема среды (`None` — цвета по умолчанию).
    pub fn set_ide_theme(&mut self, app: &mut Program, e: Option<&ThemeEntry>) {
        match e {
            Some(e) => {
                app.set_theme(&e.theme);
                self.settings.theme = e.key();
            }
            None => {
                app.palette = standard_palette();
                self.settings.theme.clear();
            }
        }
        self.save_settings();
    }

    /// Тема формы на холсте и в предпросмотре (`None` — как у среды).
    pub fn set_form_theme(&mut self, app: &mut Program, e: Option<ThemeEntry>) {
        let key = e.as_ref().map(|e| e.key()).unwrap_or_default();
        self.settings.set(FORM_THEME_KEY, Json::from_str(&key));
        if let Some(c) = app.find_mut::<Canvas>(self.ids.2) {
            c.form_theme = e.map(|e| (e.theme.name.clone(), e.theme.palette.clone()));
        }
        self.save_settings();
    }

    /// Стандартный диалог «Настройки» (темы — те же, что в меню «Тема»).
    pub fn settings_dialog(&mut self, app: &mut Program) {
        let list: Vec<(String, Theme)> = self.themes.iter().map(|e| (e.key(), e.theme.clone())).collect();
        if settings_dialog_with(app, &mut self.settings, &list) {
            if let Some(p) = &self.settings_path {
                if let Err(e) = self.settings.save(p) {
                    self.error(app, &e);
                }
            }
        }
    }

    /// Запомнить, где стоит окно среды, и записать настройки (при выходе).
    pub fn save_on_exit(&mut self, app: &Program) {
        self.settings.capture_windows_with(app, window_key);
        self.save_settings();
    }

    /// Палитра темы формы, если она выбрана.
    pub fn form_palette(&self, app: &Program) -> Option<Vec<Attr>> {
        app.find::<Canvas>(self.ids.2).and_then(|c| c.form_theme.as_ref().map(|(_, p)| p.clone()))
    }

    fn add_theme(&mut self, e: ThemeEntry) -> usize {
        match self.themes.iter().position(|x| x.key() == e.key()) {
            Some(i) => i,
            None => {
                self.themes.push(e);
                self.themes.len() - 1
            }
        }
    }

    /// Выбор темы формы: как `choose_theme`, но перекрашивается только
    /// холст. Первый пункт — «как у среды».
    pub fn choose_form_theme(&mut self, app: &mut Program) {
        let mut d = Dialog::centered(46, 16, "Тема формы");
        let mut list = ListBox::new(Rect::default(), 1, None);
        let mut names = vec!["(как у среды)".to_string()];
        names.extend(self.themes.iter().map(|e| e.theme.name.clone()));
        list.set_items(names);
        let current = self.form_palette(app);
        list.focused = current.as_ref().and_then(|p| self.themes.iter().position(|e| &e.theme.palette == p)).map_or(0, |i| i + 1);
        let list_id = d.add(list);
        let note = d.add(StaticText::new(
            Rect::default(),
            "Форма на холсте перекрашивается сразу;
цвета среды не меняются.",
        ));
        let ok = d.add(Button::new(Rect::default(), "O~K~", cm::OK, bf::DEFAULT));
        let cancel = d.add(Button::new(Rect::default(), "Отмена", cm::CANCEL, bf::NORMAL));
        d.set_layout(
            VBox::new()
                .margins(2, 1, 2, 0)
                .add(item(list_id).min(30, 8).stretch(1))
                .space(1)
                .add(note)
                .space(1)
                .add(HBox::new().stretch(1).add(ok).add(cancel)),
        );
        // Справа — чтобы не закрывать холст.
        d.win.group.vs.options &= !of::CENTERED;
        let ext = app.desktop().group.vs.extent();
        d.win.group.vs.origin = Point::new((ext.b.x - 48).max(0), 2);
        d.select(list_id);
        let canvas = self.ids.2;
        let themes = self.themes.clone();
        let before = app.find::<Canvas>(canvas).and_then(|c| c.form_theme.clone());
        let mut shown = usize::MAX;
        let (cmd, d) = app.exec_dialog_idle(d, |app, _| {
            let i = app.find::<ListBox>(list_id).map(|l| l.focused).unwrap_or(0);
            if i != shown {
                shown = i;
                if let Some(c) = app.find_mut::<Canvas>(canvas) {
                    c.form_theme = i.checked_sub(1).and_then(|k| themes.get(k)).map(|e| (e.theme.name.clone(), e.theme.palette.clone()));
                }
            }
            None
        });
        let i = d.win.group.find::<ListBox>(list_id).map(|l| l.focused).unwrap_or(0);
        if cmd == cm::OK {
            let e = i.checked_sub(1).and_then(|k| self.themes.get(k)).cloned();
            self.set_form_theme(app, e);
        } else if let Some(c) = app.find_mut::<Canvas>(canvas) {
            c.form_theme = before;
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty.get()
    }

    fn set_dirty(&mut self, d: bool) {
        self.dirty.set(d);
    }

    fn file_name(&self) -> String {
        self.path.as_ref().and_then(|p| p.file_name()).map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "без имени.tvui".into())
    }

    /// Запомнить состояние для «Отменить» перед изменением.
    fn checkpoint(&mut self) {
        self.undo.push((self.doc.clone(), self.sel.clone()));
        if self.undo.len() > 200 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    fn changed(&mut self, app: &mut Program) {
        self.set_dirty(true);
        self.refresh_all(app);
    }

    fn info(&mut self, app: &mut Program, text: &str) {
        self.last_message = text.to_string();
        message_box(app, text, mf::INFORMATION | mf::OK_BUTTON);
    }

    fn error(&mut self, app: &mut Program, text: &str) {
        self.last_message = text.to_string();
        message_box(app, text, mf::ERROR | mf::OK_BUTTON);
    }

    // ---------- показ ----------

    /// Перестроить всё: дерево, холст, свойства, заголовок.
    pub fn refresh_all(&mut self, app: &mut Program) {
        if let Sel::Node(p) = &self.sel {
            if self.doc.node(p).is_none() {
                self.sel = Sel::Form;
            }
        }
        let (_, tree, canvas, _) = self.ids;
        let kind = if self.doc.kind == FormKind::Dialog { "Dialog" } else { "Window" };
        let root = TreeNode::new(&format!("{} — {} «{}»", self.doc.name, kind, self.doc.title)).expanded(true).child(tree_of(&self.doc.root));
        if let Some(t) = app.find_mut::<TreeView>(tree) {
            t.set_roots(vec![root]);
        }
        let built = build(&self.doc);
        if let Some(c) = app.find_mut::<Canvas>(canvas) {
            c.built = Some(built);
        }
        let title = format!("TVCreator — {}{}", self.file_name(), if self.is_dirty() { " *" } else { "" });
        if let Some(w) = app.find_mut::<MainWindow>(self.window) {
            w.win.title = title;
        }
        self.refresh_selection(app);
    }

    /// Показать выделение: в дереве, на холсте, в свойствах.
    pub fn refresh_selection(&mut self, app: &mut Program) {
        let (_, tree, canvas, props_id) = self.ids;
        let tp = sel_to_tree(&self.sel);
        if let Some(t) = app.find_mut::<TreeView>(tree) {
            t.select_path(&tp);
            self.last_tree = t.cursor_path().map(|p| p.to_vec());
        }
        if let Some(c) = app.find_mut::<Canvas>(canvas) {
            c.selected = self.sel.clone();
        }
        let rows: Vec<Vec<String>> = props(&self.doc, &self.sel).into_iter().map(|p| vec![p.label.to_string(), p.value]).collect();
        if let Some(t) = app.find_mut::<Table>(props_id) {
            let (row, _) = t.cursor();
            let n = rows.len();
            let cols = t.columns.clone(); // set_model сбрасывает ширины столбцов
            t.set_model(Box::new(VecTableModel::new(&["Свойство", "Значение"], rows)));
            t.set_columns(cols);
            t.set_cursor(row.min(n.saturating_sub(1)), 0);
            t.refresh();
        }
    }

    /// Свойство под курсором таблицы.
    fn current_prop(&self, app: &Program) -> Option<Prop> {
        let (row, _) = app.find::<Table>(self.ids.3)?.cursor();
        props(&self.doc, &self.sel).into_iter().nth(row)
    }

    // ---------- правка ----------

    /// Добавить пункт палитры номер `i` у выделенного.
    pub fn add(&mut self, app: &mut Program, i: usize) {
        let Some(mut node) = palette_node(&self.doc, i) else { return };
        self.checkpoint();
        let at: Vec<usize> = match &self.sel {
            Sel::Form => vec![],
            Sel::Node(p) => p.clone(),
        };
        // Подпись перед полем подписывает его сама (как «buddy» в Qt).
        let new_name = node.as_widget().map(|w| (w.name.clone(), w.kind));
        let path = self.doc.insert(&at, std::mem::replace(&mut node, Node::space(0)));
        if let (Some((name, kind)), Some(&idx)) = (new_name, path.last()) {
            if kind != WidgetKind::Label && idx > 0 {
                let mut prev = path.clone();
                *prev.last_mut().unwrap() = idx - 1;
                if let Some(w) = self.doc.node_mut(&prev).and_then(|n| n.as_widget_mut()) {
                    if w.kind == WidgetKind::Label && w.link.is_empty() {
                        w.link = name;
                    }
                }
            }
        }
        self.sel = Sel::Node(path);
        self.changed(app);
    }

    pub fn delete(&mut self, app: &mut Program) {
        let Sel::Node(p) = self.sel.clone() else { return };
        if p.is_empty() {
            return;
        }
        self.checkpoint();
        let next = self.doc.remove(&p);
        // Подписи, чей виджет удалён, больше ничего не подписывают.
        let names: Vec<String> = self.doc.widgets().iter().map(|w| w.name.clone()).collect();
        let paths: Vec<Vec<usize>> = self.doc.walk().into_iter().map(|(p, _)| p).collect();
        for q in paths {
            if let Some(w) = self.doc.node_mut(&q).and_then(|n| n.as_widget_mut()) {
                if !w.link.is_empty() && !names.contains(&w.link) {
                    w.link.clear();
                }
            }
        }
        self.sel = next.map(Sel::Node).unwrap_or(Sel::Form);
        self.changed(app);
    }

    pub fn shift(&mut self, app: &mut Program, delta: i32) {
        let Sel::Node(p) = self.sel.clone() else { return };
        let before = self.doc.clone();
        if let Some(np) = self.doc.shift(&p, delta) {
            self.undo.push((before, self.sel.clone()));
            self.redo.clear();
            self.sel = Sel::Node(np);
            self.changed(app);
        }
    }

    pub fn wrap(&mut self, app: &mut Program, kind: LayoutKind) {
        self.checkpoint();
        let p = match &self.sel {
            Sel::Form => vec![],
            Sel::Node(p) => p.clone(),
        };
        let np = self.doc.wrap(&p, kind);
        self.sel = Sel::Node(np);
        self.changed(app);
    }

    pub fn undo(&mut self, app: &mut Program) {
        if let Some((d, s)) = self.undo.pop() {
            self.redo.push((std::mem::replace(&mut self.doc, d), std::mem::replace(&mut self.sel, s)));
            self.changed(app);
        }
    }

    pub fn redo(&mut self, app: &mut Program) {
        if let Some((d, s)) = self.redo.pop() {
            self.undo.push((std::mem::replace(&mut self.doc, d), std::mem::replace(&mut self.sel, s)));
            self.changed(app);
        }
    }

    /// Изменить свойство (без диалога — для тестов и Enter на флажках).
    pub fn apply_prop(&mut self, app: &mut Program, key: &str, value: &str) -> Result<(), String> {
        let before = self.doc.clone();
        match set_prop(&mut self.doc, &self.sel, key, value) {
            Ok(()) => {
                if self.doc != before {
                    self.undo.push((before, self.sel.clone()));
                    self.redo.clear();
                    self.changed(app);
                }
                Ok(())
            }
            Err(e) => {
                self.doc = before;
                Err(e)
            }
        }
    }

    /// Enter в свойствах: флажок и выбор — следующее значение, остальное —
    /// окно ввода.
    pub fn edit_prop(&mut self, app: &mut Program) {
        let Some(p) = self.current_prop(app) else { return };
        if matches!(&p.kind, crate::props::PropKind::Choice(v) if v.is_empty()) {
            return; // только для чтения (класс)
        }
        let value = match next_value(&p) {
            Some(v) => v,
            None => match input_box(app, "Свойство", &format!("~{}~:", p.label), &p.value, 250) {
                Some(v) => v,
                None => return,
            },
        };
        if let Err(e) = self.apply_prop(app, p.key, &value) {
            self.error(app, &e);
        }
    }

    // ---------- файлы ----------

    /// Несохранённые изменения: спросить. `true` — можно продолжать.
    fn confirm_discard(&mut self, app: &mut Program) -> bool {
        if !self.is_dirty() {
            return true;
        }
        match message_box(app, &format!("\x03Форма «{}» изменена.\n\x03Сохранить?", self.file_name()), mf::CONFIRMATION | mf::YES_NO_CANCEL)
        {
            c if c == cm::YES => self.save(app),
            c if c == cm::NO => true,
            _ => false,
        }
    }

    pub fn save_to(&mut self, app: &mut Program, path: &Path) -> bool {
        match self.doc.save(path) {
            Ok(()) => {
                self.path = Some(path.to_path_buf());
                self.set_dirty(false);
                self.refresh_all(app);
                true
            }
            Err(e) => {
                self.error(app, &e);
                false
            }
        }
    }

    pub fn save(&mut self, app: &mut Program) -> bool {
        match self.path.clone() {
            Some(p) => self.save_to(app, &p),
            None => self.save_as(app),
        }
    }

    pub fn save_as(&mut self, app: &mut Program) -> bool {
        let (cmd, d) = app.exec_dialog(FileDialog::new("*.tvui", "Сохранить форму", "~И~мя", fd::OK_BUTTON, 101));
        if cmd == cm::CANCEL {
            return false;
        }
        let mut p = d.file_name();
        if p.extension().is_none() {
            p.set_extension("tvui");
        }
        self.save_to(app, &p)
    }

    pub fn open_path(&mut self, app: &mut Program, path: &Path) {
        match FormDoc::load(path) {
            Ok(doc) => {
                self.doc = doc;
                self.path = Some(path.to_path_buf());
                self.sel = Sel::Form;
                self.undo.clear();
                self.redo.clear();
                self.set_dirty(false);
                self.refresh_all(app);
            }
            Err(e) => self.error(app, &e),
        }
    }

    /// Записать код рядом с формой. Возвращает путь файла.
    pub fn generate(&mut self, app: &mut Program) -> Option<PathBuf> {
        let problems = self.doc.problems();
        if !problems.is_empty() {
            self.error(app, &format!("Код не сгенерирован:\n{}", problems.join("\n")));
            return None;
        }
        if self.path.is_none() && !self.save_as(app) {
            return None;
        }
        let form = self.path.clone()?;
        let out = codegen::output_path(&form, &self.doc);
        let code = codegen::generate(&self.doc, &self.file_name());
        match std::fs::write(&out, code) {
            Ok(()) => {
                self.info(app, &format!("Код записан:\n{}", out.display()));
                Some(out)
            }
            Err(e) => {
                self.error(app, &format!("{}: {}", out.display(), e));
                None
            }
        }
    }

    fn show_code(&mut self, app: &mut Program) {
        let code = codegen::generate(&self.doc, &self.file_name());
        let r = app.desktop().group.vs.extent().grown(-4, -2);
        let mut w = Window::new(r, &format!("Код: {}.rs", codegen::module_name(&self.doc.name)), 0);
        let v = w.standard_scroll_bar(true);
        let h = w.standard_scroll_bar(false);
        let mut e = Editor::new(w.interior(), Some(h), Some(v), None);
        e.set_text(&code);
        e.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
        w.add(e);
        app.insert_window(w);
    }

    fn preview(&mut self, app: &mut Program) {
        let theme = self.form_palette(app);
        // В теме формы предпросмотр модальный (палитра — на всю программу
        // и возвращается после закрытия), поэтому окно показывается как диалог.
        let mut doc = self.doc.clone();
        if theme.is_some() {
            doc.kind = FormKind::Dialog;
        }
        let built = build(&doc);
        match built.view.into_any().downcast::<Dialog>() {
            Ok(d) => {
                let saved = theme.map(|p| std::mem::replace(&mut app.palette, p));
                app.exec_dialog(*d);
                if let Some(p) = saved {
                    app.palette = p;
                }
            }
            Err(any) => {
                if let Ok(mut w) = any.downcast::<Window>() {
                    w.title = format!("Предпросмотр: {}", w.title);
                    w.group.vs.origin = Point::new(2, 1);
                    app.insert_window(*w);
                }
            }
        }
    }
}

impl AppHandler for Ide {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        let Some(cmd) = ev.command() else { return };
        match cmd {
            CM_NEW => {
                if self.confirm_discard(app) {
                    self.doc = FormDoc::new("Form1");
                    self.path = None;
                    self.sel = Sel::Form;
                    self.undo.clear();
                    self.redo.clear();
                    self.set_dirty(false);
                    self.refresh_all(app);
                }
            }
            CM_OPEN => {
                if self.confirm_discard(app) {
                    let (c, d) = app.exec_dialog(FileDialog::new("*.tvui", "Открыть форму", "~И~мя", fd::OPEN_BUTTON, 100));
                    if c != cm::CANCEL {
                        self.open_path(app, &d.file_name());
                    }
                }
            }
            CM_SAVE => {
                self.save(app);
            }
            CM_SAVE_AS => {
                self.save_as(app);
            }
            CM_GENERATE => {
                self.generate(app);
            }
            CM_EXIT => {
                if self.confirm_discard(app) {
                    self.set_dirty(false);
                    app.put_event(Event::command_event(cm::QUIT));
                }
            }
            CM_ADD => {
                let i = app.find::<ListBox>(self.ids.0).map(|l| l.focused).unwrap_or(0);
                self.add(app, i);
            }
            CM_DELETE => self.delete(app),
            CM_UP => self.shift(app, -1),
            CM_DOWN => self.shift(app, 1),
            CM_WRAP_H => self.wrap(app, LayoutKind::HBox),
            CM_WRAP_V => self.wrap(app, LayoutKind::VBox),
            CM_WRAP_GRID => self.wrap(app, LayoutKind::Grid),
            CM_WRAP_FORM => self.wrap(app, LayoutKind::Form),
            CM_UNDO => self.undo(app),
            CM_REDO => self.redo(app),
            CM_PROP_EDIT => self.edit_prop(app),
            CM_PREVIEW => self.preview(app),
            CM_CODE => self.show_code(app),
            CM_CHECK => {
                let p = self.doc.problems();
                if p.is_empty() {
                    self.info(app, "\x03Ошибок нет: код соберётся.");
                } else {
                    self.error(app, &p.join("\n"));
                }
            }
            CM_THEME_CHOOSE => {
                let list: Vec<Theme> = self.themes.iter().map(|e| e.theme.clone()).collect();
                if let Some(i) = choose_theme(app, &list) {
                    let e = self.themes[i].clone();
                    self.set_ide_theme(app, Some(&e));
                }
            }
            CM_THEME_DEFAULT => self.set_ide_theme(app, None),
            CM_THEME_OPEN => {
                let (c, d) = app.exec_dialog(FileDialog::new("*.tvtheme", "Открыть тему", "~И~мя", fd::OPEN_BUTTON, 102));
                if c != cm::CANCEL {
                    let p = d.file_name();
                    match Theme::load(&p) {
                        Ok(theme) => {
                            let i = self.add_theme(ThemeEntry { theme, path: Some(p) });
                            let e = self.themes[i].clone();
                            self.set_ide_theme(app, Some(&e));
                        }
                        Err(e) => self.error(app, &e),
                    }
                }
            }
            CM_FORM_THEME_CHOOSE => self.choose_form_theme(app),
            CM_SETTINGS => self.settings_dialog(app),
            CM_FORM_THEME_SAME => self.set_form_theme(app, None),
            c if (CM_IDE_THEME..CM_IDE_THEME + MENU_THEMES as u16).contains(&c) => {
                if let Some(e) = self.themes.get((c - CM_IDE_THEME) as usize).cloned() {
                    self.set_ide_theme(app, Some(&e));
                }
            }
            c if (CM_FORM_THEME..CM_FORM_THEME + MENU_THEMES as u16).contains(&c) => {
                let e = self.themes.get((c - CM_FORM_THEME) as usize).cloned();
                if e.is_some() {
                    self.set_form_theme(app, e);
                }
            }
            CM_ABOUT => self.info(
                app,
                "\x03TVCreator\n\n\x03Дизайнер форм для tvision-rs\n\x03в духе Qt Creator.\n\n\x03Формы — файлы .tvui,\n\x03код — F9 или tvcreator --gen.",
            ),
            _ => return,
        }
        ev.clear();
    }

    fn idle(&mut self, app: &mut Program) {
        // Выбор мышью на холсте.
        let picked = app.find_mut::<Canvas>(self.ids.2).and_then(|c| c.picked.take());
        if let Some(s) = picked {
            if s != self.sel {
                self.sel = s;
                self.refresh_selection(app);
            }
        } else {
            // Курсор в дереве объектов.
            let tp = app.find::<TreeView>(self.ids.1).and_then(|t| t.cursor_path()).map(|p| p.to_vec());
            if tp.is_some() && tp != self.last_tree {
                self.last_tree = tp.clone();
                if let Some(s) = tp.as_deref().and_then(tree_to_sel) {
                    if s != self.sel {
                        self.sel = s;
                        self.refresh_selection(app);
                    }
                }
            }
        }
        let node_sel = matches!(&self.sel, Sel::Node(p) if !p.is_empty());
        for (c, on) in
            [(CM_DELETE, node_sel), (CM_UP, node_sel), (CM_DOWN, node_sel), (CM_UNDO, !self.undo.is_empty()), (CM_REDO, !self.redo.is_empty())]
        {
            if on {
                app.enable_command(c)
            } else {
                app.disable_command(c)
            }
        }
    }
}

/// Меню с темами по умолчанию (встроенными).
pub fn menu() -> Menu {
    menu_with(&load_themes(&[]))
}

/// Меню «Тема»: тема среды и тема формы — по пункту на тему.
fn themes_menu(themes: &[ThemeEntry]) -> Menu {
    let mut ide = Menu::new();
    let mut form = Menu::new();
    for (i, e) in themes.iter().enumerate().take(MENU_THEMES) {
        ide = ide.item(&e.theme.name, CM_IDE_THEME + i as u16);
        form = form.item(&e.theme.name, CM_FORM_THEME + i as u16);
    }
    ide = ide
        .separator()
        .item_key("~В~ыбрать с предпросмотром...", CM_THEME_CHOOSE, kb::ctrl('t'), "Ctrl-T")
        .item("~О~ткрыть файл темы...", CM_THEME_OPEN)
        .item("По ~у~молчанию", CM_THEME_DEFAULT);
    form = form.separator().item("~В~ыбрать с предпросмотром...", CM_FORM_THEME_CHOOSE).item("~К~ак у среды", CM_FORM_THEME_SAME);
    Menu::new().submenu("~С~реда", ide).submenu("~Ф~орма на холсте", form)
}

pub fn menu_with(themes: &[ThemeEntry]) -> Menu {
    Menu::new()
        .submenu(
            "~Ф~айл",
            Menu::new()
                .item_key("~Н~овая форма", CM_NEW, kb::ctrl('n'), "Ctrl-N")
                .item_key("~О~ткрыть...", CM_OPEN, kb::F3, "F3")
                .item_key("~С~охранить", CM_SAVE, kb::F2, "F2")
                .item("Сохранить ~к~ак...", CM_SAVE_AS)
                .separator()
                .item_key("Сгенерировать ~к~од", CM_GENERATE, kb::f(9), "F9")
                .separator()
                .item("Нас~т~ройки...", CM_SETTINGS)
                .separator()
                .item_key("~В~ыход", cm::QUIT, kb::ALT_X, "Alt-X"),
        )
        .submenu(
            "~П~равка",
            Menu::new()
                .item_key("~О~тменить", CM_UNDO, kb::ctrl('z'), "Ctrl-Z")
                .item_key("~П~овторить", CM_REDO, kb::ctrl('y'), "Ctrl-Y")
                .separator()
                .item_key("~Д~обавить из палитры", CM_ADD, kb::key(Key::Ins), "Ins")
                .item_key("~У~далить", CM_DELETE, kb::key(Key::Del), "Del")
                .item_key("~В~ыше", CM_UP, KeyEvent::new(Key::Up, Mods::CTRL), "Ctrl-↑")
                .item_key("~Н~иже", CM_DOWN, KeyEvent::new(Key::Down, Mods::CTRL), "Ctrl-↓")
                .item_key("~С~войство...", CM_PROP_EDIT, kb::f(4), "F4"),
        )
        .submenu(
            "~Р~аскладка",
            Menu::new()
                .item("Обернуть в ~с~троку (HBox)", CM_WRAP_H)
                .item("Обернуть в с~т~олбец (VBox)", CM_WRAP_V)
                .item("Обернуть в се~т~ку (Grid)", CM_WRAP_GRID)
                .item("Обернуть в ~ф~орму (Form)", CM_WRAP_FORM),
        )
        .submenu(
            "Ф~о~рма",
            Menu::new()
                .item_key("~П~редпросмотр", CM_PREVIEW, kb::ctrl('r'), "Ctrl-R")
                .item_key("Показать ~к~од", CM_CODE, kb::f(7), "F7")
                .item("П~р~оверить", CM_CHECK)
                .separator()
                .item("О ~п~рограмме", CM_ABOUT),
        )
        .submenu("~Т~ема", themes_menu(themes))
        .submenu("~О~кна", Menu::new().item_key("~С~ледующее", cm::NEXT, kb::F6, "F6").item_key("~З~акрыть", cm::CLOSE, kb::ALT_F3, "Alt-F3"))
}

/// Строка состояния — только F-клавиши: она видит клавиши даже в
/// диалогах, а Ins/Del там нужны строкам ввода.
pub fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![
        StatusItem::new("~F2~ Сохранить", kb::F2, CM_SAVE),
        StatusItem::new("~F3~ Открыть", kb::F3, CM_OPEN),
        StatusItem::new("~F4~ Свойство", kb::f(4), CM_PROP_EDIT),
        StatusItem::new("~F7~ Код", kb::f(7), CM_CODE),
        StatusItem::new("~F9~ Генерировать", kb::f(9), CM_GENERATE),
        StatusItem::new("~Ctrl-R~ Предпросмотр", kb::ctrl('r'), CM_PREVIEW),
        StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT),
        StatusItem::new("", kb::F10, cm::MENU),
    ])]
}

/// Запустить среду (из `main`).
pub fn run(file: Option<PathBuf>) -> Result<(), String> {
    // Настройки — рядом с программой (tvcreator.settings.json); язык —
    // до меню.
    let settings_path = Settings::default_path();
    Settings::load_or(&settings_path, default_settings()).apply_language();
    let (doc, path) = match file {
        Some(p) if p.exists() => (FormDoc::load(&p)?, Some(p)),
        Some(p) => (FormDoc::new("Form1"), Some(p)), // новый файл
        None => (FormDoc::new("Form1"), None),
    };
    let themes = load_themes(&theme_dirs());
    let mut app = Program::with_terminal(Some(menu_with(&themes)), Some(status())).map_err(|e| e.to_string())?;
    let mut ide = Ide::with_themes(&mut app, doc, path, themes, Some(settings_path));
    ide.settings.restore_windows_with(&mut app, window_key);
    if !ide.last_message.is_empty() {
        let m = ide.last_message.clone();
        ide.error(&mut app, &m);
    }
    app.run(&mut ide);
    ide.save_on_exit(&app);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tvision::platform::HeadlessBackend;

    fn ide_with(keys: &[KeyEvent]) -> (Program, Ide) {
        let mut b = HeadlessBackend::new(120, 40);
        for k in keys {
            b.push_key(*k);
        }
        let mut app = Program::new(Box::new(b), Some(menu()), Some(status()));
        let mut ide = Ide::new(&mut app, FormDoc::new("Form1"), None);
        app.pump(&mut ide);
        (app, ide)
    }

    fn temp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("tvcreator-{}-{}", std::process::id(), name));
        let _ = std::fs::remove_file(&p);
        p
    }

    /// Индекс пункта палитры для виджета.
    fn pal(k: WidgetKind) -> usize {
        6 + WidgetKind::ALL.iter().position(|x| *x == k).unwrap()
    }

    #[test]
    fn screen_has_all_panes() {
        let (mut app, _ide) = ide_with(&[]);
        app.redraw();
        let s = app.screen_text();
        for w in ["Виджеты", "Объекты", "Форма", "Свойства", "TVCreator — без имени.tvui", "Строка (HBox)", "Form1 — Dialog", "Пустая форма"]
        {
            assert!(s.contains(w), "нет «{}»:\n{}", w, s);
        }
    }

    #[test]
    fn enter_in_palette_adds() {
        // Фокус — на палитре (первый выбираемый); Enter — добавить «Строку».
        let (_app, ide) = ide_with(&[kb::ENTER]);
        assert_eq!(ide.doc.root.children.len(), 1);
        assert_eq!(ide.doc.root.children[0].layout_kind(), Some(LayoutKind::HBox));
        assert_eq!(ide.sel, Sel::Node(vec![0]));
        assert!(ide.is_dirty());
    }

    #[test]
    fn build_form_save_generate() {
        let (mut app, mut ide) = ide_with(&[]);
        // Форма «подпись — поле» и строка кнопок.
        ide.add(&mut app, 3); // Form
        ide.add(&mut app, pal(WidgetKind::Label)); // внутрь формы
        ide.add(&mut app, pal(WidgetKind::InputLine)); // после подписи — подпись свяжется
        assert_eq!(ide.doc.node(&[0, 0]).unwrap().as_widget().unwrap().link, "input1");
        ide.apply_prop(&mut app, "w.name", "user").unwrap();
        ide.sel = Sel::Node(vec![0, 0]);
        ide.apply_prop(&mut app, "w.text", "~И~мя:").unwrap();
        assert_eq!(ide.doc.node(&[0, 0]).unwrap().as_widget().unwrap().link, "user", "ссылка переименована");
        ide.sel = Sel::Node(vec![]);
        ide.add(&mut app, 0); // HBox в корень
        ide.add(&mut app, 4); // пружина внутрь
        ide.add(&mut app, pal(WidgetKind::Button)); // после пружины
        ide.apply_prop(&mut app, "w.default", "да").unwrap();
        assert!(ide.doc.problems().is_empty(), "{:?}", ide.doc.problems());
        // Свойства формы.
        ide.sel = Sel::Form;
        ide.apply_prop(&mut app, "form.name", "LoginDialog").unwrap();
        assert!(ide.apply_prop(&mut app, "form.name", "плохое имя").is_err());
        // Сохранить и прочитать.
        let path = temp("login.tvui");
        assert!(ide.save_to(&mut app, &path));
        assert!(!ide.is_dirty());
        assert_eq!(FormDoc::load(&path).unwrap(), ide.doc);
        // Код — рядом с формой (окно «Код записан» закроет Esc терминала).
        let out = ide.generate(&mut app).unwrap();
        assert_eq!(out, path.with_file_name("login_dialog.rs"));
        let code = std::fs::read_to_string(&out).unwrap();
        assert!(code.contains("pub struct LoginDialog") && code.contains("Label::new(Rect::default(), \"~И~мя:\", Some(user))"), "{}", code);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn undo_redo_and_delete() {
        let (mut app, mut ide) = ide_with(&[]);
        ide.add(&mut app, pal(WidgetKind::Button));
        ide.add(&mut app, pal(WidgetKind::Button));
        assert_eq!(ide.doc.widgets().len(), 2);
        ide.delete(&mut app);
        assert_eq!(ide.doc.widgets().len(), 1);
        ide.undo(&mut app);
        assert_eq!(ide.doc.widgets().len(), 2);
        ide.redo(&mut app);
        assert_eq!(ide.doc.widgets().len(), 1);
        // Вверх/вниз и обёртка.
        ide.add(&mut app, pal(WidgetKind::InputLine));
        ide.shift(&mut app, -1);
        assert_eq!(ide.sel, Sel::Node(vec![0]));
        assert_eq!(ide.doc.widgets()[0].kind, WidgetKind::InputLine);
        ide.wrap(&mut app, LayoutKind::HBox);
        assert_eq!(ide.doc.node(&[0]).unwrap().layout_kind(), Some(LayoutKind::HBox));
    }

    #[test]
    fn selection_follows_canvas_and_tree() {
        let (mut app, mut ide) = ide_with(&[]);
        ide.add(&mut app, pal(WidgetKind::Button));
        ide.add(&mut app, pal(WidgetKind::InputLine));
        // Щелчок на холсте (как его запоминает Canvas).
        app.find_mut::<Canvas>(ide.ids.2).unwrap().picked = Some(Sel::Node(vec![0]));
        ide.idle(&mut app);
        assert_eq!(ide.sel, Sel::Node(vec![0]));
        let t = app.find::<TreeView>(ide.ids.1).unwrap();
        assert_eq!(t.cursor_path().map(|p| p.to_vec()), Some(vec![0, 0, 0]), "дерево — на том же узле");
        // Курсор в дереве.
        app.find_mut::<TreeView>(ide.ids.1).unwrap().select_path(&[0, 0, 1]);
        ide.idle(&mut app);
        assert_eq!(ide.sel, Sel::Node(vec![1]));
        app.redraw();
        assert!(app.screen_text().contains("input1"), "свойства выбранного:\n{}", app.screen_text());
        // Команды по состоянию.
        ide.sel = Sel::Form;
        ide.idle(&mut app);
        assert!(!app.command_enabled(CM_DELETE) && app.command_enabled(CM_UNDO));
    }

    #[test]
    fn quit_asks_when_dirty() {
        let (mut app, mut ide) = ide_with(&[]);
        ide.add(&mut app, 0);
        // Alt+X: окно не даёт выйти и спрашивает; Esc терминала — «Отмена».
        app.put_event(Event::command_event(cm::QUIT));
        app.pump(&mut ide);
        assert!(app.is_running(), "изменения не потеряны");
        // Без изменений — выход сразу.
        ide.set_dirty(false);
        app.put_event(Event::command_event(cm::QUIT));
        app.pump(&mut ide);
        assert!(!app.is_running());
    }

    #[test]
    fn preview_and_code_window() {
        let (mut app, mut ide) = ide_with(&[]);
        ide.add(&mut app, pal(WidgetKind::Button));
        app.put_event(Event::command_event(CM_PREVIEW)); // модальный — закроет Esc
        app.pump(&mut ide);
        app.put_event(Event::command_event(CM_CODE));
        app.pump(&mut ide);
        app.redraw();
        let s = app.screen_text();
        assert!(s.contains("Код: form1.rs") && s.contains("pub struct Form1"), "{}", s);
    }

    // ---------- темы ----------

    fn canvas_theme(app: &Program, ide: &Ide) -> Option<String> {
        app.find::<Canvas>(ide.ids.2).and_then(|c| c.form_theme.as_ref().map(|(n, _)| n.clone()))
    }

    #[test]
    fn menu_has_theme_items() {
        let (mut app, _ide) = ide_with(&[KeyEvent::new(Key::Char('т'), Mods::ALT)]);
        app.redraw();
        let s = app.screen_text();
        assert!(s.contains("Среда") && s.contains("Форма на холсте"), "{}", s);
    }

    #[test]
    fn ide_theme_from_menu_and_default() {
        let (mut app, mut ide) = ide_with(&[]);
        let classic = ide.themes.iter().position(|e| e.theme.base == tvision::theme::Base::Classic).unwrap();
        app.put_event(Event::command_event(CM_IDE_THEME + classic as u16));
        app.pump(&mut ide);
        assert_eq!(app.palette, ide.themes[classic].theme.palette);
        assert_eq!(ide.settings.theme, "builtin:classic");
        app.put_event(Event::command_event(CM_THEME_DEFAULT));
        app.pump(&mut ide);
        assert_eq!(app.palette, standard_palette(), "стандартные цвета для этого терминала");
        assert!(ide.settings.theme.is_empty());
    }

    #[test]
    fn form_theme_paints_only_canvas() {
        let (mut app, mut ide) = ide_with(&[]);
        let ide_pal = app.palette.clone();
        let classic = ide.themes.iter().position(|e| e.theme.base == tvision::theme::Base::Classic).unwrap();
        app.put_event(Event::command_event(CM_FORM_THEME + classic as u16));
        app.pump(&mut ide);
        assert_eq!(app.palette, ide_pal, "цвета среды не меняются");
        assert_eq!(canvas_theme(&app, &ide).as_deref(), Some("Классическая"));
        app.redraw();
        let s = app.screen_text();
        assert!(s.contains("Тема формы: Классическая"), "{}", s);
        // Рамка диалога формы — в цвете классического серого диалога.
        let t = &ide.themes[classic].theme;
        let frame = [t.get(32), t.get(33)];
        assert!(app.screen().cells.iter().any(|c| c.ch == '┌' && frame.contains(&c.attr)));
        // Предпросмотр в теме формы возвращает цвета среды.
        app.put_event(Event::command_event(CM_PREVIEW));
        app.pump(&mut ide);
        assert_eq!(app.palette, ide_pal);
        app.put_event(Event::command_event(CM_FORM_THEME_SAME));
        app.pump(&mut ide);
        assert_eq!(canvas_theme(&app, &ide), None);
    }

    #[test]
    fn choose_form_theme_dialog() {
        let down = KeyEvent::new(Key::Down, Mods::NONE);
        let chosen = |last: KeyEvent| {
            let mut b = HeadlessBackend::new(120, 40);
            b.push_key(down);
            b.push_key(last);
            let mut app = Program::new(Box::new(b), Some(menu()), Some(status()));
            let mut ide = Ide::new(&mut app, FormDoc::new("Form1"), None);
            app.put_event(Event::command_event(CM_FORM_THEME_CHOOSE));
            app.pump(&mut ide);
            (canvas_theme(&app, &ide), ide.themes[0].theme.name.clone())
        };
        let (got, first) = chosen(kb::ENTER);
        assert_eq!(got, Some(first));
        // Отмена возвращает прежнюю тему формы.
        assert_eq!(chosen(kb::ESC).0, None);
    }

    #[test]
    fn themes_remembered_between_runs() {
        let cfg = std::env::temp_dir().join(format!("tvcreator-theme-cfg-{}", std::process::id())).join("settings.json");
        let _ = std::fs::remove_file(&cfg);
        let run = |cmds: &[u16]| {
            let mut app = Program::new(Box::new(HeadlessBackend::new(120, 40)), Some(menu()), Some(status()));
            let mut ide = Ide::with_themes(&mut app, FormDoc::new("Form1"), None, load_themes(&[]), Some(cfg.clone()));
            for c in cmds {
                app.put_event(Event::command_event(*c));
            }
            app.pump(&mut ide);
            let ft = canvas_theme(&app, &ide);
            (app.palette.clone(), ft)
        };
        let themes = load_themes(&[]);
        let mono = themes.iter().position(|e| e.theme.base == tvision::theme::Base::Monochrome).unwrap();
        let classic = themes.iter().position(|e| e.theme.base == tvision::theme::Base::Classic).unwrap();
        run(&[CM_IDE_THEME + mono as u16, CM_FORM_THEME + classic as u16]);
        let (pal, ft) = run(&[]);
        assert_eq!(pal, themes[mono].theme.palette);
        assert_eq!(ft.as_deref(), Some("Классическая"));
        let _ = std::fs::remove_dir_all(cfg.parent().unwrap());
    }

    #[test]
    fn settings_dialog_saves_file_and_window() {
        let cfg = std::env::temp_dir().join(format!("tvcreator-std-settings-{}", std::process::id())).join("tvcreator.settings.json");
        let _ = std::fs::remove_file(&cfg);
        // Диалог: узор фона (Alt+Ф) → ↓ «▒», Enter — OK.
        let mut b = HeadlessBackend::new(120, 40);
        for k in [KeyEvent::new(Key::Char('ф'), Mods::ALT), KeyEvent::new(Key::Down, Mods::NONE), kb::ENTER] {
            b.push_key(k);
        }
        let mut app = Program::new(Box::new(b), Some(menu()), Some(status()));
        let mut ide = Ide::with_themes(&mut app, FormDoc::new("Form1"), None, load_themes(&[]), Some(cfg.clone()));
        app.put_event(Event::command_event(CM_SETTINGS));
        app.pump(&mut ide);
        assert_eq!(app.desktop().pattern(), '▒');
        let saved = Settings::load(&cfg).unwrap();
        assert_eq!(saved.pattern, '▒');
        assert_eq!(saved.language, LanguageChoice::Ru, "по умолчанию — русский");
        // При выходе запоминается окно среды.
        ide.save_on_exit(&app);
        assert!(Settings::load(&cfg).unwrap().window("tvcreator").is_some());
        // Следующий запуск применяет настройки сам.
        let mut app2 = Program::new(Box::new(HeadlessBackend::new(120, 40)), Some(menu()), Some(status()));
        let _ide2 = Ide::with_themes(&mut app2, FormDoc::new("Form1"), None, load_themes(&[]), Some(cfg.clone()));
        assert_eq!(app2.desktop().pattern(), '▒');
        let _ = std::fs::remove_dir_all(cfg.parent().unwrap());
    }
}
