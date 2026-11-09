//! Редактор тем: группы цветов, сетки выбора, образец, предпросмотр.

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use tvision::color::*;
use tvision::layout::{item, Alignment, Form, HBox, VBox};
use tvision::theme::{self, attr_to_text, Base, Theme};
use tvision::widgets::colorsel::{Color256Dialog, ColorDisplay, ColorGroup, ColorSel, ColorSelector};
use tvision::*;

use crate::preview::Preview;

// ----------------------------------------------------------------------
// Команды
// ----------------------------------------------------------------------

pub const CM_NEW_CLASSIC: u16 = 100;
pub const CM_NEW_GRUVBOX: u16 = 101;
pub const CM_NEW_BW: u16 = 102;
pub const CM_NEW_MONO: u16 = 103;
pub const CM_OPEN: u16 = 104;
pub const CM_SAVE: u16 = 105;
pub const CM_SAVE_AS: u16 = 106;
pub const CM_EXPORT: u16 = 107;
pub const CM_EXIT: u16 = 108;
pub const CM_UNDO: u16 = 110;
pub const CM_REDO: u16 = 111;
pub const CM_FORE256: u16 = 112;
pub const CM_BACK256: u16 = 113;
pub const CM_RESET_ITEM: u16 = 114;
pub const CM_PROPS: u16 = 115;
pub const CM_LIVE: u16 = 116;
pub const CM_ABOUT: u16 = 117;
pub const CM_COPY: u16 = 118;
pub const CM_PASTE: u16 = 119;
pub const CM_SHOW_CODE: u16 = 120;

// ----------------------------------------------------------------------
// Главное окно
// ----------------------------------------------------------------------

/// Id частей главного окна.
#[derive(Clone, Copy, Debug)]
pub struct Ids {
    pub window: ViewId,
    pub groups: ViewId,
    pub items: ViewId,
    pub fore: ViewId,
    pub back: ViewId,
    pub display: ViewId,
    pub info: ViewId,
    pub preview: ViewId,
}

/// Главное окно; при несохранённой теме выход сначала спрашивает.
pub struct MainWindow {
    pub win: Window,
    pub dirty: Rc<Cell<bool>>,
}

impl MainWindow {
    pub fn new(r: Rect, theme: Rc<RefCell<Theme>>, groups: &[ColorGroup], dirty: Rc<Cell<bool>>) -> (MainWindow, Ids) {
        let mut win = Window::new(r, "Редактор тем", 0);
        win.flags &= !wf::CLOSE;
        let mut gl = ListBox::new(Rect::default(), 1, None);
        gl.set_items(groups.iter().map(|g| g.name.clone()).collect());
        let g_id = win.add(gl);
        let i_id = win.add(ListBox::new(Rect::default(), 1, None));
        let lg = win.add(Label::new(Rect::default(), "~Г~руппа", Some(g_id)));
        let li = win.add(Label::new(Rect::default(), "~Ц~вет", Some(i_id)));
        let fore = win.add(ColorSelector::new(Rect::new(0, 0, 12, 4), ColorSel::Foreground));
        let back = win.add(ColorSelector::new(Rect::new(0, 0, 12, 2), ColorSel::Background));
        let lf = win.add(Label::new(Rect::default(), "~Т~екст", Some(fore)));
        let lb = win.add(Label::new(Rect::default(), "~Ф~он", Some(back)));
        let display = win.add(ColorDisplay::new(Rect::new(0, 0, 14, 2), "Текст "));
        let info = win.add(StaticText::new(Rect::new(0, 0, 24, 3), ""));
        let preview = win.add(Preview::new(theme));
        let lp = win.add(StaticText::new(Rect::default(), "Предпросмотр"));
        win.set_layout(
            HBox::new()
                .spacing(2)
                .margins(1, 0, 0, 0)
                .add(item(VBox::new().add(lg).add(item(g_id).stretch(1)).space(1).add(li).add(item(i_id).stretch(2))).min_width(26))
                .add(
                    VBox::new()
                        .add(lf)
                        .add(item(fore).align_h(Alignment::Start))
                        .space(1)
                        .add(lb)
                        .add(item(back).align_h(Alignment::Start))
                        .space(1)
                        .add(item(display).align_h(Alignment::Start))
                        .space(1)
                        .add(item(info).min(26, 3))
                        .stretch(1),
                )
                .add(item(VBox::new().add(lp).add(item(preview).stretch(1))).stretch(1)),
        );
        win.select(g_id);
        let ids = Ids { window: ViewId::default(), groups: g_id, items: i_id, fore, back, display, info, preview };
        (MainWindow { win, dirty }, ids)
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
        self.win.handle_window_event(ev, ctx);
    }
    fn valid(&mut self, command: u16, ctx: &mut Ctx) -> bool {
        if command == cm::QUIT && self.dirty.get() {
            ctx.post_command(CM_EXIT, Info::None);
            return false;
        }
        self.win.group.valid(command, ctx)
    }
}

// ----------------------------------------------------------------------
// Редактор
// ----------------------------------------------------------------------

pub struct ThemeEditor {
    pub theme: Rc<RefCell<Theme>>,
    pub path: Option<PathBuf>,
    pub ids: Ids,
    pub groups: Vec<ColorGroup>,
    dirty: Rc<Cell<bool>>,
    /// Что показано: (группа, пункт, цвет).
    shown: Option<(usize, usize, Attr)>,
    /// Что стоит в сетках (текст, фон).
    sel_shown: (u8, u8),
    undo: Vec<Theme>,
    redo: Vec<Theme>,
    /// Тема — на весь редактор (иначе — только в предпросмотре).
    pub live: bool,
    own_palette: Vec<Attr>,
    clipboard: Option<Attr>,
    pub last_message: String,
}

impl ThemeEditor {
    pub fn new(app: &mut Program, theme: Theme, path: Option<PathBuf>) -> ThemeEditor {
        let groups = theme::color_groups();
        let theme = Rc::new(RefCell::new(theme));
        let dirty = Rc::new(Cell::new(false));
        let r = app.desktop().group.vs.extent();
        let (w, mut ids) = MainWindow::new(r, theme.clone(), &groups, dirty.clone());
        ids.window = app.insert_window(w);
        let own_palette = app.palette.clone();
        let mut e = ThemeEditor {
            theme,
            path,
            ids,
            groups,
            dirty,
            shown: None,
            sel_shown: (0, 0),
            undo: Vec::new(),
            redo: Vec::new(),
            live: false,
            own_palette,
            clipboard: None,
            last_message: String::new(),
        };
        e.sync(app);
        e.update_title(app);
        e
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty.get()
    }

    fn file_name(&self) -> String {
        self.path.as_ref().and_then(|p| p.file_name()).map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "без имени.tvtheme".into())
    }

    fn update_title(&self, app: &mut Program) {
        let t = format!("Редактор тем — {} «{}»{}", self.file_name(), self.theme.borrow().name, if self.is_dirty() { " *" } else { "" });
        if let Some(w) = app.find_mut::<MainWindow>(self.ids.window) {
            w.win.title = t;
        }
    }

    fn focused(&self, app: &Program, id: ViewId) -> usize {
        app.find::<ListBox>(id).map(|l| l.focused).unwrap_or(0)
    }

    /// Номер цвета палитры под курсором.
    pub fn current_index(&self, app: &Program) -> Option<u8> {
        let g = self.groups.get(self.focused(app, self.ids.groups))?;
        Some(g.items.get(self.focused(app, self.ids.items))?.index)
    }

    /// Поменять цвет пункта с отменой.
    pub fn set_color(&mut self, app: &mut Program, index: u8, a: Attr) {
        let before = self.theme.borrow().clone();
        if before.get(index) == a {
            return;
        }
        self.undo.push(before);
        if self.undo.len() > 500 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.theme.borrow_mut().set(index, a);
        self.changed(app);
    }

    fn changed(&mut self, app: &mut Program) {
        self.dirty.set(true);
        if self.live {
            app.palette = self.theme.borrow().palette.clone();
        }
        self.update_title(app);
        self.sync(app);
    }

    fn replace_theme(&mut self, app: &mut Program, t: Theme, path: Option<PathBuf>) {
        *self.theme.borrow_mut() = t;
        self.path = path;
        self.undo.clear();
        self.redo.clear();
        self.dirty.set(false);
        self.shown = None;
        if self.live {
            app.palette = self.theme.borrow().palette.clone();
        }
        self.update_title(app);
        self.sync(app);
    }

    /// Согласовать списки, сетки и образец (как `ColorDialog::sync`).
    pub fn sync(&mut self, app: &mut Program) {
        let gi = self.focused(app, self.ids.groups);
        let mut ii = self.focused(app, self.ids.items);
        let group_changed = self.shown.map(|s| s.0) != Some(gi);
        if group_changed {
            if let Some(g) = self.groups.get(gi) {
                let names: Vec<String> = g.items.iter().map(|i| i.name.clone()).collect();
                let idx = g.index;
                if let Some(l) = app.find_mut::<ListBox>(self.ids.items) {
                    l.set_items(names);
                    l.focus_item(idx);
                    ii = l.focused;
                }
            }
        } else if let Some(g) = self.groups.get_mut(gi) {
            g.index = ii;
        }
        let Some(index) = self.current_index(app) else { return };
        let a = self.theme.borrow().get(index);
        let fg = app.find::<ColorSelector>(self.ids.fore).map(|s| s.color).unwrap_or(0);
        let bg = app.find::<ColorSelector>(self.ids.back).map(|s| s.color).unwrap_or(0);
        let item_changed = group_changed || self.shown.map(|s| s.1) != Some(ii) || self.shown.map(|s| s.2) != Some(a);
        if item_changed {
            // Показать цвет в сетках (у 256-цветного — ближайшие цвета BIOS).
            let (f, b) = (fore(a), back(a) & 7);
            if let Some(s) = app.find_mut::<ColorSelector>(self.ids.fore) {
                s.color = f;
            }
            if let Some(s) = app.find_mut::<ColorSelector>(self.ids.back) {
                s.color = b;
            }
            self.sel_shown = (f, b);
        } else if (fg, bg) != self.sel_shown {
            // Щёлкнули в сетке — поменять только эту сторону цвета.
            let mut na = a;
            if fg != self.sel_shown.0 {
                na = with_fore(na, fg);
            }
            if bg != self.sel_shown.1 {
                na = with_back(na, bg);
            }
            self.sel_shown = (fg, bg);
            self.set_color(app, index, na);
            return;
        }
        let a = self.theme.borrow().get(index);
        if let Some(d) = app.find_mut::<ColorDisplay>(self.ids.display) {
            d.color = a;
        }
        let base = self.theme.borrow().base.palette()[index as usize - 1];
        let info = format!(
            "№ {}: {}\n{}",
            index,
            attr_to_text(a),
            if a == base { "как в основе".to_string() } else { format!("в основе: {}", attr_to_text(base)) }
        );
        if let Some(t) = app.find_mut::<StaticText>(self.ids.info) {
            t.text = info;
        }
        self.shown = Some((gi, ii, a));
    }

    // ---------- команды ----------

    pub fn undo(&mut self, app: &mut Program) {
        if let Some(t) = self.undo.pop() {
            let cur = std::mem::replace(&mut *self.theme.borrow_mut(), t);
            self.redo.push(cur);
            self.changed(app);
        }
    }

    pub fn redo(&mut self, app: &mut Program) {
        if let Some(t) = self.redo.pop() {
            let cur = std::mem::replace(&mut *self.theme.borrow_mut(), t);
            self.undo.push(cur);
            self.changed(app);
        }
    }

    fn pick256(&mut self, app: &mut Program, sel: ColorSel) {
        let Some(index) = self.current_index(app) else { return };
        let a = self.theme.borrow().get(index);
        let (cmd, d) = app.exec_dialog(Color256Dialog::new(sel, a));
        if cmd == cm::OK {
            self.set_color(app, index, d.result());
        }
    }

    pub fn reset_item(&mut self, app: &mut Program) {
        let Some(index) = self.current_index(app) else { return };
        let base = self.theme.borrow().base.palette()[index as usize - 1];
        self.set_color(app, index, base);
    }

    fn info(&mut self, app: &mut Program, text: &str) {
        self.last_message = text.to_string();
        message_box(app, text, mf::INFORMATION | mf::OK_BUTTON);
    }

    fn error(&mut self, app: &mut Program, text: &str) {
        self.last_message = text.to_string();
        message_box(app, text, mf::ERROR | mf::OK_BUTTON);
    }

    /// Свойства темы: имя, описание, основа.
    pub fn edit_props(&mut self, app: &mut Program) {
        let t = self.theme.borrow().clone();
        let mut d = Dialog::centered(50, 12, "Свойства темы");
        let name = d.add(InputLine::new(Rect::default(), 60));
        let desc = d.add(InputLine::new(Rect::default(), 120));
        let base = d.add(Cluster::radio_buttons(Rect::default(), &Base::ALL.map(|b| b.title())));
        let ln = d.add(Label::new(Rect::default(), "~И~мя:", Some(name)));
        let ld = d.add(Label::new(Rect::default(), "~О~писание:", Some(desc)));
        let lb = d.add(Label::new(Rect::default(), "О~с~нова:", Some(base)));
        let note = d.add(StaticText::new(Rect::default(), "Основа — с чем сравнивать при записи:\nв файл попадают только отличия."));
        let ok = d.add(Button::new(Rect::default(), "O~K~", cm::OK, bf::DEFAULT));
        let cancel = d.add(Button::new(Rect::default(), "Отмена", cm::CANCEL, bf::NORMAL));
        d.set_layout(
            VBox::new()
                .margins(2, 1, 2, 0)
                .add(Form::new().row(ln, name).row(ld, desc).row(lb, base))
                .space(1)
                .add(note)
                .space(1)
                .add(HBox::new().stretch(1).add(ok).add(cancel)),
        );
        d.adjust_size();
        let bi = Base::ALL.iter().position(|b| *b == t.base).unwrap_or(0) as u32;
        d.set_data(&Value::List(vec![Value::Str(t.name.clone()), Value::Str(t.description.clone()), Value::U32(bi)]));
        let (cmd, d) = app.exec_dialog(d);
        if cmd != cm::OK {
            return;
        }
        let v = d.get_data();
        let l = v.as_list().unwrap_or(&[]);
        let mut nt = t.clone();
        nt.name = l.first().and_then(Value::as_str).unwrap_or("").trim().to_string();
        nt.description = l.get(1).and_then(Value::as_str).unwrap_or("").trim().to_string();
        nt.base = Base::ALL[l.get(2).and_then(Value::as_u32).unwrap_or(0).min(3) as usize];
        if nt != t {
            self.undo.push(t);
            self.redo.clear();
            *self.theme.borrow_mut() = nt;
            self.changed(app);
        }
    }

    // ---------- файлы ----------

    fn confirm_discard(&mut self, app: &mut Program) -> bool {
        if !self.is_dirty() {
            return true;
        }
        match message_box(app, &format!("\x03Тема «{}» изменена.\n\x03Сохранить?", self.theme.borrow().name), mf::CONFIRMATION | mf::YES_NO_CANCEL)
        {
            c if c == cm::YES => self.save(app),
            c if c == cm::NO => true,
            _ => false,
        }
    }

    pub fn save_to(&mut self, app: &mut Program, path: &Path) -> bool {
        let r = self.theme.borrow().save(path);
        match r {
            Ok(()) => {
                self.path = Some(path.to_path_buf());
                self.dirty.set(false);
                self.update_title(app);
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
        let (cmd, d) = app.exec_dialog(FileDialog::new("*.tvtheme", "Сохранить тему", "~И~мя", fd::OK_BUTTON, 111));
        if cmd == cm::CANCEL {
            return false;
        }
        let mut p = d.file_name();
        if p.extension().is_none() {
            p.set_extension("tvtheme");
        }
        self.save_to(app, &p)
    }

    pub fn open_path(&mut self, app: &mut Program, path: &Path) {
        match Theme::load(path) {
            Ok(t) => self.replace_theme(app, t, Some(path.to_path_buf())),
            Err(e) => self.error(app, &e),
        }
    }

    /// Имя константы для экспорта: из имени файла, заглавными.
    pub fn const_name(&self) -> String {
        let stem = self.path.as_ref().and_then(|p| p.file_stem()).map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let mut s: String = stem.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_uppercase() } else { '_' }).collect();
        if s.is_empty() || s.starts_with(|c: char| c.is_ascii_digit()) {
            s = format!("THEME_{}", s);
        }
        format!("{}_THEME", s.trim_end_matches('_'))
    }

    /// Записать тему как Rust-код рядом с файлом темы.
    pub fn export(&mut self, app: &mut Program) -> Option<PathBuf> {
        if self.path.is_none() && !self.save_as(app) {
            return None;
        }
        let out = self.path.as_ref()?.with_extension("rs");
        let code = self.theme.borrow().to_rust(&self.const_name());
        match std::fs::write(&out, code) {
            Ok(()) => {
                self.info(app, &format!("Код темы записан:\n{}\n\nПодключение: app.palette = {}.to_vec();", out.display(), self.const_name()));
                Some(out)
            }
            Err(e) => {
                self.error(app, &format!("{}: {}", out.display(), e));
                None
            }
        }
    }

    fn show_code(&mut self, app: &mut Program) {
        let code = self.theme.borrow().to_rust(&self.const_name());
        let r = app.desktop().group.vs.extent().grown(-4, -2);
        let mut w = Window::new(r, "Тема как Rust-код", 0);
        let v = w.standard_scroll_bar(true);
        let h = w.standard_scroll_bar(false);
        let mut e = Editor::new(w.interior(), Some(h), Some(v), None);
        e.set_text(&code);
        e.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
        w.add(e);
        app.insert_window(w);
    }

    pub fn toggle_live(&mut self, app: &mut Program) {
        self.live = !self.live;
        app.palette = if self.live { self.theme.borrow().palette.clone() } else { self.own_palette.clone() };
    }
}

impl AppHandler for ThemeEditor {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        let Some(cmd) = ev.command() else { return };
        match cmd {
            CM_NEW_CLASSIC | CM_NEW_GRUVBOX | CM_NEW_BW | CM_NEW_MONO => {
                if self.confirm_discard(app) {
                    let base = Base::ALL[(cmd - CM_NEW_CLASSIC) as usize];
                    let mut t = Theme::builtin(base);
                    t.name = "Новая тема".into();
                    self.replace_theme(app, t, None);
                }
            }
            CM_OPEN => {
                if self.confirm_discard(app) {
                    let (c, d) = app.exec_dialog(FileDialog::new("*.tvtheme", "Открыть тему", "~И~мя", fd::OPEN_BUTTON, 110));
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
            CM_EXPORT => {
                self.export(app);
            }
            CM_SHOW_CODE => self.show_code(app),
            CM_EXIT => {
                if self.confirm_discard(app) {
                    self.dirty.set(false);
                    app.put_event(Event::command_event(cm::QUIT));
                }
            }
            CM_UNDO => self.undo(app),
            CM_REDO => self.redo(app),
            CM_FORE256 => self.pick256(app, ColorSel::Foreground),
            CM_BACK256 => self.pick256(app, ColorSel::Background),
            CM_RESET_ITEM => self.reset_item(app),
            CM_COPY => {
                if let Some(i) = self.current_index(app) {
                    self.clipboard = Some(self.theme.borrow().get(i));
                }
            }
            CM_PASTE => {
                if let (Some(i), Some(a)) = (self.current_index(app), self.clipboard) {
                    self.set_color(app, i, a);
                }
            }
            CM_PROPS => self.edit_props(app),
            CM_LIVE => self.toggle_live(app),
            CM_ABOUT => self.info(
                app,
                "\x03Редактор тем tvision-rs\n\n\x03Тема — 135 цветов палитры\n\x03в файле .tvtheme.\n\n\x03В программе: app.load_theme(путь)\n\x03или TVISION_THEME=путь.",
            ),
            _ => return,
        }
        ev.clear();
    }

    fn idle(&mut self, app: &mut Program) {
        self.sync(app);
        for (c, on) in [(CM_UNDO, !self.undo.is_empty()), (CM_REDO, !self.redo.is_empty()), (CM_PASTE, self.clipboard.is_some())] {
            if on {
                app.enable_command(c)
            } else {
                app.disable_command(c)
            }
        }
    }
}

pub fn menu() -> Menu {
    Menu::new()
        .submenu(
            "~Ф~айл",
            Menu::new()
                .submenu(
                    "~Н~овая тема",
                    Menu::new()
                        .item("~К~лассическая", CM_NEW_CLASSIC)
                        .item("~G~ruvbox (256 цветов)", CM_NEW_GRUVBOX)
                        .item("~Ч~ёрно-белая", CM_NEW_BW)
                        .item("~М~онохромная", CM_NEW_MONO),
                )
                .item_key("~О~ткрыть...", CM_OPEN, kb::F3, "F3")
                .item_key("~С~охранить", CM_SAVE, kb::F2, "F2")
                .item("Сохранить ~к~ак...", CM_SAVE_AS)
                .separator()
                .item_key("~Э~кспорт в Rust", CM_EXPORT, kb::f(9), "F9")
                .item_key("~П~оказать код", CM_SHOW_CODE, kb::f(7), "F7")
                .separator()
                .item_key("~В~ыход", cm::QUIT, kb::ALT_X, "Alt-X"),
        )
        .submenu(
            "~П~равка",
            Menu::new()
                .item_key("~О~тменить", CM_UNDO, kb::ctrl('z'), "Ctrl-Z")
                .item_key("~П~овторить", CM_REDO, kb::ctrl('y'), "Ctrl-Y")
                .separator()
                .item_key("~К~опировать цвет", CM_COPY, kb::ctrl('c'), "Ctrl-C")
                .item_key("~В~ставить цвет", CM_PASTE, kb::ctrl('v'), "Ctrl-V")
                .item("~С~бросить к основе", CM_RESET_ITEM)
                .separator()
                .item_key("~Т~екст: 256 цветов...", CM_FORE256, kb::f(5), "F5")
                .item_key("~Ф~он: 256 цветов...", CM_BACK256, kb::f(6), "F6")
                .separator()
                .item_key("Св~о~йства темы...", CM_PROPS, kb::f(4), "F4"),
        )
        .submenu(
            "~В~ид",
            Menu::new().item_key("~Т~ема на весь редактор", CM_LIVE, kb::ctrl('t'), "Ctrl-T").separator().item("О ~п~рограмме", CM_ABOUT),
        )
}

/// Строка состояния — только F-клавиши (она видит клавиши и в диалогах).
pub fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![
        StatusItem::new("~F2~ Сохранить", kb::F2, CM_SAVE),
        StatusItem::new("~F3~ Открыть", kb::F3, CM_OPEN),
        StatusItem::new("~F4~ Свойства", kb::f(4), CM_PROPS),
        StatusItem::new("~F5~ Текст 256", kb::f(5), CM_FORE256),
        StatusItem::new("~F6~ Фон 256", kb::f(6), CM_BACK256),
        StatusItem::new("~F9~ В Rust", kb::f(9), CM_EXPORT),
        StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT),
        StatusItem::new("", kb::F10, cm::MENU),
    ])]
}

/// Запустить редактор (из `main`).
pub fn run(file: Option<PathBuf>) -> Result<(), String> {
    locale::set_language(locale::Language::Ru);
    let (theme, path) = match file {
        Some(p) if p.exists() => (Theme::load(&p)?, Some(p)),
        Some(p) => {
            let mut t = Theme::builtin(Base::Classic);
            t.name = "Новая тема".into();
            (t, Some(p))
        }
        None => {
            let mut t = Theme::builtin(Base::Classic);
            t.name = "Новая тема".into();
            (t, None)
        }
    };
    let mut app = Program::with_terminal(Some(menu()), Some(status())).map_err(|e| e.to_string())?;
    let mut ed = ThemeEditor::new(&mut app, theme, path);
    app.run(&mut ed);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tvision::platform::HeadlessBackend;
    use tvision::theme::attr_from_text;

    fn editor(keys: &[KeyEvent]) -> (Program, ThemeEditor) {
        let mut b = HeadlessBackend::new(130, 36);
        for k in keys {
            b.push_key(*k);
        }
        let mut app = Program::new(Box::new(b), Some(menu()), Some(status()));
        let mut t = Theme::builtin(Base::Classic);
        t.name = "Тест".into();
        let mut e = ThemeEditor::new(&mut app, t, None);
        app.pump(&mut e);
        (app, e)
    }

    fn temp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("tvtheme-{}-{}", std::process::id(), name));
        let _ = std::fs::remove_file(&p);
        p
    }

    #[test]
    fn screen_has_parts() {
        let (mut app, _e) = editor(&[]);
        app.redraw();
        let s = app.screen_text();
        for w in [
            "Группа",
            "Рабочий стол",
            "Меню и строка состояния",
            "Цвет",
            "Фон",
            "Текст",
            "Предпросмотр",
            "Синее окно",
            "№ 1: blue/lightgray",
            "без имени.tvtheme «Тест»",
        ] {
            assert!(s.contains(w), "нет «{}»:\n{}", w, s);
        }
    }

    #[test]
    fn grid_click_changes_one_side() {
        let (mut app, mut e) = editor(&[]);
        // Пункт 1 (рабочий стол); «щёлкнуть» жёлтый в сетке текста.
        app.find_mut::<ColorSelector>(e.ids.fore).unwrap().color = 14;
        e.sync(&mut app);
        let a = e.theme.borrow().get(1);
        assert_eq!(attr_to_text(a), "yellow/lightgray", "фон остался (было blue/lightgray)");
        assert!(e.is_dirty());
        // Отменить и повторить.
        e.undo(&mut app);
        assert_eq!(attr_to_text(e.theme.borrow().get(1)), "blue/lightgray");
        e.redo(&mut app);
        assert_eq!(attr_to_text(e.theme.borrow().get(1)), "yellow/lightgray");
        // Сбросить к основе.
        e.reset_item(&mut app);
        assert_eq!(attr_to_text(e.theme.borrow().get(1)), "blue/lightgray");
    }

    #[test]
    fn group_and_item_navigation() {
        // Группа «Синее окно», в ней второй пункт — «Рамка активная» (№ 9).
        // (idle, а с ним sync, в настоящей работе идёт между нажатиями.)
        let (mut app, mut e) = editor(&[kb::key(Key::Down), kb::key(Key::Down)]);
        e.sync(&mut app);
        assert_eq!(e.current_index(&app), Some(8));
        app.find_mut::<ListBox>(e.ids.items).unwrap().focus_item(1);
        e.sync(&mut app);
        assert_eq!(e.current_index(&app), Some(9));
        app.redraw();
        assert!(app.screen_text().contains("№ 9:"), "{}", app.screen_text());
    }

    #[test]
    fn save_open_export() {
        let (mut app, mut e) = editor(&[]);
        e.set_color(&mut app, 1, attr_from_text("cyan/blue").unwrap());
        e.set_color(&mut app, 42, attr256(221, 24));
        let path = temp("sea.tvtheme");
        assert!(e.save_to(&mut app, &path));
        assert!(!e.is_dirty());
        let back = Theme::load(&path).unwrap();
        assert_eq!(back.changed(), vec![1, 42]);
        assert_eq!(&back, &*e.theme.borrow());
        // Экспорт: код рядом (окно «записан» закроет Esc терминала).
        let out = e.export(&mut app).unwrap();
        assert_eq!(out, path.with_extension("rs"));
        assert_eq!(e.const_name(), format!("TVTHEME_{}_SEA_THEME", std::process::id()));
        assert!(std::fs::read_to_string(&out).unwrap().contains("pub const TVTHEME_"));
        // Открыть в новом редакторе.
        let (mut app2, mut e2) = editor(&[]);
        e2.open_path(&mut app2, &path);
        assert_eq!(e2.theme.borrow().get(1), attr_from_text("cyan/blue").unwrap());
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn live_mode_and_copy_paste() {
        let (mut app, mut e) = editor(&[]);
        let own = app.palette.clone();
        e.toggle_live(&mut app);
        e.set_color(&mut app, 1, attr_from_text("red/black").unwrap());
        assert_eq!(app.palette[0], attr_from_text("red/black").unwrap(), "весь редактор в цветах темы");
        e.toggle_live(&mut app);
        assert_eq!(app.palette, own);
        // Копировать цвет пункта 1 и вставить в пункт 2 (меню).
        app.put_event(Event::command_event(CM_COPY));
        app.pump(&mut e);
        app.find_mut::<ListBox>(e.ids.groups).unwrap().focus_item(1);
        e.sync(&mut app);
        app.put_event(Event::command_event(CM_PASTE));
        app.pump(&mut e);
        assert_eq!(e.theme.borrow().get(2), attr_from_text("red/black").unwrap());
    }

    #[test]
    fn quit_asks_when_dirty() {
        let (mut app, mut e) = editor(&[]);
        e.set_color(&mut app, 1, 0x4F);
        app.put_event(Event::command_event(cm::QUIT));
        app.pump(&mut e); // вопрос закроет Esc — «Отмена»
        assert!(app.is_running());
    }
}
