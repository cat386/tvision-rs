//! Среда TVCreator: палитра, дерево объектов, холст, свойства — как в
//! Qt Creator (режим «Дизайн»).
//!
//! Части среды:
//! * `window` — главное окно и палитра виджетов;
//! * `edit` — правка формы, отмена, свойства;
//! * `files` — файлы, генерация кода, предпросмотр;
//! * `theme_ui` — темы среды и формы, диалог настроек;
//! * `handler` — команды и синхронизация частей окна;
//! * `menu` — меню и строка состояния.

mod edit;
mod files;
mod handler;
mod menu;
#[cfg(test)]
mod tests;
mod theme_ui;
mod window;

pub use menu::{menu, menu_with, status};
pub use window::{palette_items, palette_node, MainWindow};

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
