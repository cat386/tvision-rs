//! Файловые диалоги: `TFileDialog` (открыть/сохранить файл), `TFileList`,
//! `TFileInputLine`, `TFileInfoPane`, `TChDirDialog` (смена каталога),
//! `TDirListBox` (дерево каталогов). Файлы `tfildlg.cpp`, `tfillist.cpp`,
//! `stddlg.cpp`, `tchdrdlg.cpp`, `tdirlist.cpp`.
//!
//! Вместо `findfirst`/`findnext` из DOS используется `std::fs::read_dir`;
//! шаблоны имён (`*.txt`, `data?.csv`) сопоставляются функцией [`wild_match`].

use crate::locale::tr;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::time::SystemTime;

use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Key};
use crate::geometry::{Point, Rect};
use crate::text::to_upper;
use crate::view::{gf, of, Ctx, DrawCtx, Value, View, ViewId, ViewState};
use crate::widgets::button::{bf, Button};
use crate::widgets::dialog::Dialog;
use crate::widgets::history::History;
use crate::widgets::inputline::InputLine;
use crate::widgets::listbox::ListBox;
use crate::widgets::msgbox::{build_message_box, mf};
use crate::widgets::scrollbar::ScrollBar;
use crate::widgets::statictext::Label;
use crate::widgets::window::wf;

/// Флаги файлового диалога (`fdXXX`).
pub mod fd {
    pub const OK_BUTTON: u16 = 0x0001;
    pub const OPEN_BUTTON: u16 = 0x0002;
    pub const REPLACE_BUTTON: u16 = 0x0004;
    pub const CLEAR_BUTTON: u16 = 0x0008;
    pub const HELP_BUTTON: u16 = 0x0010;
    /// Не читать каталог при создании.
    pub const NO_LOAD_DIR: u16 = 0x0100;
}

/// Тексты диалога (в оригинале — статические поля `TFileDialog`).
pub struct FileDialogText;
impl FileDialogText {
    pub const FILES: &'static str = "~F~iles";
    pub const OPEN: &'static str = "~O~pen";
    pub const OK: &'static str = "O~K~";
    pub const REPLACE: &'static str = "~R~eplace";
    pub const CLEAR: &'static str = "~C~lear";
    pub const CANCEL: &'static str = "Cancel";
    pub const HELP: &'static str = "~H~elp";
    pub const INVALID_DRIVE: &'static str = "Invalid drive or directory";
    pub const INVALID_FILE: &'static str = "Invalid file name";
}

/// Сопоставление имени с шаблоном (`*` — любые символы, `?` — один символ).
/// На Windows регистр не учитывается.
pub fn wild_match(pattern: &str, name: &str) -> bool {
    // «*.*» в DOS означало «все файлы», в том числе без расширения.
    if pattern == "*" || pattern == "*.*" {
        return true;
    }
    let norm = |c: char| if cfg!(windows) { to_upper(c) } else { c };
    let p: Vec<char> = pattern.chars().map(norm).collect();
    let n: Vec<char> = name.chars().map(norm).collect();
    // Классический алгоритм с возвратом к последней звёздочке.
    let (mut i, mut j) = (0, 0);
    let (mut star, mut mark) = (None, 0);
    while j < n.len() {
        if i < p.len() && (p[i] == '?' || p[i] == n[j]) {
            i += 1;
            j += 1;
        } else if i < p.len() && p[i] == '*' {
            star = Some(i);
            mark = j;
            i += 1;
        } else if let Some(s) = star {
            i = s + 1;
            mark += 1;
            j = mark;
        } else {
            return false;
        }
    }
    while i < p.len() && p[i] == '*' {
        i += 1;
    }
    i == p.len()
}

/// Есть ли в имени символы шаблона (`isWild`).
pub fn is_wild(s: &str) -> bool {
    s.contains('*') || s.contains('?')
}

/// Запись о файле (`TSearchRec`).
#[derive(Clone, Debug, PartialEq)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

impl FileEntry {
    /// Текст в списке: каталоги — с разделителем на конце (`getText`).
    pub fn display(&self) -> String {
        if self.is_dir {
            format!("{}{}", self.name, MAIN_SEPARATOR)
        } else {
            self.name.clone()
        }
    }
}

/// Порядок файлов (`TFileCollection::compare`): сначала файлы, потом каталоги,
/// `..` — в самом конце.
fn compare_entries(a: &FileEntry, b: &FileEntry) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    if a.name == b.name {
        return Equal;
    }
    if a.name == ".." {
        return Greater;
    }
    if b.name == ".." {
        return Less;
    }
    match (a.is_dir, b.is_dir) {
        (true, false) => Greater,
        (false, true) => Less,
        _ => a.name.cmp(&b.name),
    }
}

/// Читает каталог: файлы по шаблону, все подкаталоги и `..` (`readDirectory`).
pub fn read_directory(dir: &Path, wildcard: &str) -> Vec<FileEntry> {
    let mut v = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let meta = e.metadata().ok();
            let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            if is_dir {
                if name.starts_with('.') {
                    continue;
                }
            } else if !wild_match(wildcard, &name) {
                continue;
            }
            v.push(FileEntry { name, is_dir, size: meta.as_ref().map(|m| m.len()).unwrap_or(0), modified: meta.and_then(|m| m.modified().ok()) });
        }
    }
    if dir.parent().is_some() {
        let modified = std::fs::metadata(dir.join("..")).and_then(|m| m.modified()).ok();
        v.push(FileEntry { name: "..".into(), is_dir: true, size: 0, modified });
    }
    v.sort_by(compare_entries);
    v
}

/// Перевод времени в местные дату и время: (год, месяц 1..12, день, час, минута).
pub fn civil_time(t: SystemTime) -> (i64, u32, u32, u32, u32) {
    let d = crate::platform::time::local(t);
    (d.year, d.month, d.day, d.hour, d.minute)
}

// ----------------------------------------------------------------------
// TFileList
// ----------------------------------------------------------------------

/// Список файлов в два столбца с поиском по первым буквам
/// (`TFileList` + `TSortedListBox`).
pub struct FileList {
    pub lb: ListBox,
    pub entries: Vec<FileEntry>,
    /// Набранный префикс для поиска (`searchPos`).
    search: String,
    /// Двойной щелчок по элементу (диалог превращает его в OK).
    pub activated: bool,
}

impl FileList {
    pub fn new(bounds: Rect, scroll: Option<ViewId>) -> Self {
        FileList { lb: ListBox::new(bounds, 2, scroll), entries: Vec::new(), search: String::new(), activated: false }
    }

    pub fn set_entries(&mut self, entries: Vec<FileEntry>) {
        self.lb.set_items(entries.iter().map(|e| e.display()).collect());
        self.entries = entries;
        self.search.clear();
    }

    pub fn focused_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.lb.focused)
    }
}

impl View for FileList {
    crate::impl_view_basics_field!(lb);

    fn draw(&self, dc: &mut DrawCtx) {
        self.lb.draw(dc);
    }

    fn palette(&self) -> &'static [u8] {
        self.lb.palette()
    }

    fn set_state(&mut self, flag: u16, enable: bool) {
        self.lb.set_state(flag, enable);
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        match ev {
            Event::MouseDown(m) if m.double => {
                self.activated = true;
            }
            Event::KeyDown(k) if self.lb.vs.is_focused() => {
                // Поиск по первым буквам (`TSortedListBox::handleEvent`).
                if let Some(c) = k.text_char() {
                    if c != ' ' {
                        self.search.push(c);
                        let s = self.search.to_lowercase();
                        if let Some(i) = self.entries.iter().position(|e| e.name.to_lowercase().starts_with(&s)) {
                            self.lb.focus_item(i);
                        } else {
                            self.search.pop();
                        }
                        ev.clear();
                        return;
                    }
                } else if k.key == Key::Backspace && !self.search.is_empty() {
                    self.search.pop();
                    ev.clear();
                    return;
                } else {
                    self.search.clear();
                }
            }
            _ => {}
        }
        self.lb.handle_event(ev, ctx);
    }
}

// ----------------------------------------------------------------------
// TFileInfoPane
// ----------------------------------------------------------------------

/// Сведения о выбранном файле внизу диалога (`TFileInfoPane`).
pub struct FileInfoPane {
    pub vs: ViewState,
    /// Каталог + шаблон (первая строка).
    pub path: String,
    pub entry: Option<FileEntry>,
}

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

impl FileInfoPane {
    pub fn new(bounds: Rect) -> Self {
        FileInfoPane { vs: ViewState::new(bounds), path: String::new(), entry: None }
    }
}

impl View for FileInfoPane {
    crate::impl_view_basics!();

    /// Порт `TFileInfoPane::draw`.
    fn draw(&self, dc: &mut DrawCtx) {
        let color = dc.color(1);
        let w = self.vs.size.x.max(0) as usize;
        let mut b = DrawBuffer::new(w);
        b.move_char(0, ' ', color, w);
        b.move_str(1, &self.path, color);
        dc.write_line(0, 0, self.vs.size.x, 1, &b);

        let mut b = DrawBuffer::new(w);
        b.move_char(0, ' ', color, w);
        if let Some(e) = &self.entry {
            b.move_str(1, &e.display(), color);
            if w > 38 && e.name != ".." {
                if !e.is_dir {
                    b.move_str(w - 38, &e.size.to_string(), color);
                }
                if let Some(t) = e.modified {
                    let (y, mo, d, h, mi) = civil_time(t);
                    let (h12, pm) = (if h % 12 == 0 { 12 } else { h % 12 }, h >= 12);
                    let s = if crate::locale::language() == crate::locale::Language::Ru {
                        // По-русски: день.месяц.год часы:минуты.
                        format!("{:02}.{:02}.{} {:2}:{:02}", d, mo, y, h, mi)
                    } else {
                        format!("{} {:02}, {} {:2}:{:02}{}", MONTHS[(mo - 1) as usize], d, y, h12, mi, if pm { "pm" } else { "am" })
                    };
                    b.move_str(w - 22, &s, color);
                }
            }
        }
        dc.write_line(0, 1, self.vs.size.x, 1, &b);
        if self.vs.size.y > 2 {
            let mut b = DrawBuffer::new(w);
            b.move_char(0, ' ', color, w);
            dc.write_line(0, 2, self.vs.size.x, self.vs.size.y - 2, &b);
        }
    }

    fn palette(&self) -> &'static [u8] {
        crate::color::palettes::INFO_PANE
    }
}

// ----------------------------------------------------------------------
// TFileDialog
// ----------------------------------------------------------------------

/// Диалог выбора файла (`TFileDialog`).
///
/// ```ignore
/// let d = FileDialog::new("*.txt", "Open a file", "~N~ame", fd::OPEN_BUTTON, 100);
/// let (cmd, d) = app.exec_dialog(d);
/// if cmd != cm::CANCEL { println!("{}", d.file_name().display()); }
/// ```
pub struct FileDialog {
    pub dlg: Dialog,
    /// Шаблон имён (`wildCard`).
    pub wildcard: String,
    /// Текущий каталог диалога (`directory`).
    pub directory: PathBuf,
    name_id: ViewId,
    list_id: ViewId,
    info_id: ViewId,
    /// Последний элемент списка, отражённый в строке ввода и панели сведений.
    last_focused: Option<(usize, usize)>,
    /// Размер уже подогнан под экран.
    sized: bool,
}

impl FileDialog {
    /// Аналог `TFileDialog(wildCard, title, inputName, options, histId)`.
    pub fn new(wildcard: &str, title: &str, input_name: &str, options: u16, hist_id: u16) -> Self {
        let mut dlg = Dialog::new(Rect::new(15, 1, 64, 20), title);
        dlg.win.group.vs.options |= of::CENTERED;
        dlg.win.flags |= wf::GROW;

        let mut name = InputLine::new(Rect::new(3, 3, 31, 4), 260);
        name.vs.grow_mode = gf::GROW_HI_X;
        name.history_id = Some(hist_id);
        name.set_text(wildcard);
        let name_id = dlg.add(name);
        dlg.add(Label::new(Rect::new(2, 2, 3 + crate::text::cstr_width(input_name) as i32, 3), input_name, Some(name_id)));
        let mut h = History::new(Rect::new(31, 3, 34, 4), name_id, hist_id);
        h.vs.grow_mode = gf::GROW_LO_X | gf::GROW_HI_X;
        dlg.add(h);

        let sb = dlg.add(ScrollBar::new(Rect::new(3, 14, 34, 15)));
        let mut list = FileList::new(Rect::new(3, 6, 34, 14), Some(sb));
        list.lb.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
        let list_id = dlg.add(list);
        dlg.add(Label::new(Rect::new(2, 5, 8, 6), tr(FileDialogText::FILES), Some(list_id)));

        // Кнопки справа, первая — по умолчанию.
        let mut opt = bf::DEFAULT;
        let mut r = Rect::new(35, 3, 46, 5);
        let add_button = |dlg: &mut Dialog, r: &mut Rect, text: &str, cmd: u16, flags: u8| {
            let mut b = Button::new(*r, text, cmd, flags);
            b.vs.grow_mode = gf::GROW_LO_X | gf::GROW_HI_X;
            dlg.add(b);
            *r = r.moved(0, 3);
        };
        if options & fd::OPEN_BUTTON != 0 {
            add_button(&mut dlg, &mut r, tr(FileDialogText::OPEN), cm::FILE_OPEN, opt);
            opt = bf::NORMAL;
        }
        if options & fd::OK_BUTTON != 0 {
            add_button(&mut dlg, &mut r, tr(FileDialogText::OK), cm::FILE_OPEN, opt);
            opt = bf::NORMAL;
        }
        if options & fd::REPLACE_BUTTON != 0 {
            add_button(&mut dlg, &mut r, tr(FileDialogText::REPLACE), cm::FILE_REPLACE, opt);
            opt = bf::NORMAL;
        }
        if options & fd::CLEAR_BUTTON != 0 {
            add_button(&mut dlg, &mut r, tr(FileDialogText::CLEAR), cm::FILE_CLEAR, opt);
        }
        add_button(&mut dlg, &mut r, tr(FileDialogText::CANCEL), cm::CANCEL, bf::NORMAL);
        if options & fd::HELP_BUTTON != 0 {
            add_button(&mut dlg, &mut r, tr(FileDialogText::HELP), cm::HELP, bf::NORMAL);
        }

        let mut info = FileInfoPane::new(Rect::new(1, 16, 48, 18));
        info.vs.grow_mode = gf::GROW_ALL & !gf::GROW_LO_X;
        let info_id = dlg.add(info);
        dlg.select(name_id);

        let mut d = FileDialog {
            dlg,
            wildcard: wildcard.to_string(),
            directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            name_id,
            list_id,
            info_id,
            last_focused: None,
            sized: false,
        };
        if options & fd::NO_LOAD_DIR == 0 {
            d.read_directory();
        }
        d
    }

    /// Перечитать каталог (`readDirectory`).
    pub fn read_directory(&mut self) {
        let entries = read_directory(&self.directory, &self.wildcard);
        if let Some(l) = self.dlg.win.group.find_mut::<FileList>(self.list_id) {
            l.set_entries(entries);
        }
        self.last_focused = None;
        let path = format!("{}{}", self.dir_with_sep(), self.wildcard);
        if let Some(p) = self.dlg.win.group.find_mut::<FileInfoPane>(self.info_id) {
            p.path = path;
        }
        self.sync_focus();
    }

    fn dir_with_sep(&self) -> String {
        let mut s = self.directory.display().to_string();
        if !s.ends_with(MAIN_SEPARATOR) {
            s.push(MAIN_SEPARATOR);
        }
        s
    }

    /// Выбранный элемент списка → строка ввода и панель сведений
    /// (то, что в C++ делал `cmFileFocused`).
    fn sync_focus(&mut self) {
        let (entry, key) = match self.dlg.win.group.find::<FileList>(self.list_id) {
            Some(l) => (l.focused_entry().cloned(), (l.lb.focused, l.entries.len())),
            None => return,
        };
        if self.last_focused == Some(key) {
            return;
        }
        self.last_focused = Some(key);
        if let Some(p) = self.dlg.win.group.find_mut::<FileInfoPane>(self.info_id) {
            p.entry = entry.clone();
        }
        // Строка ввода меняется, только если она не в фокусе.
        let wild = self.wildcard.clone();
        if let (Some(e), Some(il)) = (entry, self.dlg.win.group.find_mut::<InputLine>(self.name_id)) {
            if !il.vs.is_selected() {
                let text = if e.is_dir { format!("{}{}{}", e.name, MAIN_SEPARATOR, wild) } else { e.name };
                il.set_text(&text);
                il.select_all(false);
            }
        }
    }

    /// Полное имя выбранного файла (`getFileName`).
    pub fn file_name(&self) -> PathBuf {
        let text = self.dlg.win.group.find::<InputLine>(self.name_id).map(|i| i.text()).unwrap_or_default();
        let t = text.trim();
        let mut p = if Path::new(t).is_absolute() { PathBuf::from(t) } else { self.directory.join(t) };
        // Без имени файла — подставляем шаблон.
        if t.is_empty() || t.ends_with('/') || t.ends_with(MAIN_SEPARATOR) {
            p = p.join(&self.wildcard);
        }
        normalize(&p)
    }

    /// Сообщение об ошибке и фокус в строку ввода.
    fn error(&mut self, ctx: &mut Ctx, what: &str, name: &Path) {
        let d = build_message_box(Rect::new(0, 0, 50, 9), &format!("{}: '{}'", what, name.display()), mf::ERROR | mf::OK_BUTTON);
        ctx.exec(Box::new(d), None, 0);
        ctx.focus(self.name_id);
    }
}

/// Убирает `.` и `..` из пути без обращения к файловой системе.
pub fn normalize(p: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

impl View for FileDialog {
    crate::impl_group_core!(dlg.win.group);

    fn as_window(&self) -> Option<&crate::widgets::Window> {
        Some(&self.dlg.win)
    }

    fn draw(&self, dc: &mut DrawCtx) {
        self.dlg.draw(dc);
    }

    fn palette(&self) -> &'static [u8] {
        self.dlg.palette()
    }

    fn set_state(&mut self, flag: u16, enable: bool) {
        self.dlg.set_state(flag, enable);
    }

    fn size_limits(&self, owner: Point) -> (Point, Point) {
        (Point::new(49, 19), owner)
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        // При первом показе подгоняем размер под экран (как в конструкторе C++).
        if !self.sized && ev.broadcast().map(|m| m.command) == Some(cm::COMMAND_SET_CHANGED) {
            self.sized = true;
            let screen = ctx.screen_size;
            let owner = ctx.owner_size();
            let mut size = self.dlg.win.group.vs.size;
            if screen.x > 90 {
                size.x = 79;
            } else if screen.x > 63 {
                size.x = screen.x - 14;
            }
            if screen.y > 34 {
                size.y = 29;
            } else if screen.y > 25 {
                size.y = screen.y - 6;
            }
            size.x = size.x.min(owner.x);
            size.y = size.y.min(owner.y);
            let origin = Point::new((owner.x - size.x) / 2, (owner.y - size.y) / 2);
            self.dlg.win.group.change_bounds(Rect::from_origin_size(origin, size));
        }
        self.dlg.handle_event(ev, ctx);
        match ev.command() {
            Some(c) if matches!(c, cm::FILE_OPEN | cm::FILE_REPLACE | cm::FILE_CLEAR) => {
                ctx.end_modal(c);
                ev.clear();
            }
            _ => {}
        }
        // Двойной щелчок по файлу = OK (`cmFileDoubleClicked`).
        let activated = self.dlg.win.group.find_mut::<FileList>(self.list_id).map(|l| std::mem::take(&mut l.activated)).unwrap_or(false);
        if activated {
            ctx.post_command(cm::OK, Default::default());
        }
        self.sync_focus();
    }

    /// `TFileDialog::valid`: шаблон или каталог в строке ввода — перейти туда
    /// и остаться в диалоге; имя файла — закрыть диалог.
    fn valid(&mut self, command: u16, ctx: &mut Ctx) -> bool {
        if command == cm::VALID {
            return true;
        }
        if !self.dlg.valid(command, ctx) {
            return false;
        }
        if command == cm::CANCEL || command == cm::FILE_CLEAR {
            return true;
        }
        let f = self.file_name();
        let fname = f.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        if is_wild(&fname) {
            let dir = f.parent().map(|p| p.to_path_buf()).unwrap_or_default();
            if dir.is_dir() {
                self.directory = dir;
                self.wildcard = fname;
                if command != cm::FILE_INIT {
                    ctx.focus(self.list_id);
                }
                self.read_directory();
            } else {
                self.error(ctx, tr(FileDialogText::INVALID_DRIVE), &dir);
            }
            false
        } else if f.is_dir() {
            self.directory = f;
            if command != cm::FILE_INIT {
                ctx.focus(self.list_id);
            }
            self.read_directory();
            false
        } else if f.parent().map(|p| p.is_dir()).unwrap_or(false) && !fname.is_empty() {
            true
        } else {
            self.error(ctx, tr(FileDialogText::INVALID_FILE), &f);
            false
        }
    }

    fn has_data(&self) -> bool {
        true
    }

    /// Данные диалога — полное имя файла (`getData`).
    fn get_data(&self) -> Value {
        Value::Str(self.file_name().display().to_string())
    }

    fn set_data(&mut self, value: &Value) {
        if let Value::Str(s) = value {
            if let Some(il) = self.dlg.win.group.find_mut::<InputLine>(self.name_id) {
                il.set_text(s);
            }
        }
    }
}

// ----------------------------------------------------------------------
// TDirListBox и TChDirDialog
// ----------------------------------------------------------------------

/// Дерево каталогов (`TDirListBox`): корень, путь до текущего каталога
/// и его подкаталоги.
pub struct DirListBox {
    pub lb: ListBox,
    /// Полные пути элементов.
    pub paths: Vec<PathBuf>,
    /// Текущий каталог (`dir`).
    pub dir: PathBuf,
    /// Номер элемента текущего каталога (`cur`).
    pub cur: usize,
    pub activated: bool,
}

impl DirListBox {
    pub fn new(bounds: Rect, scroll: Option<ViewId>) -> Self {
        DirListBox { lb: ListBox::new(bounds, 1, scroll), paths: Vec::new(), dir: PathBuf::new(), cur: 0, activated: false }
    }

    /// Построить дерево для каталога (`newDirectory` / `showDirs`).
    pub fn new_directory(&mut self, dir: &Path) {
        let dir = normalize(&crate::widgets::editor::absolute(dir));
        let mut items = Vec::new();
        let mut paths = Vec::new();
        // Путь до текущего каталога: «└─┬» у каждого уровня.
        let comps: Vec<PathBuf> = dir.ancestors().map(|p| p.to_path_buf()).collect::<Vec<_>>().into_iter().rev().collect();
        for (i, p) in comps.iter().enumerate() {
            let name = if i == 0 { p.display().to_string() } else { p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default() };
            let indent = " ".repeat(i * 2);
            let prefix = if i == 0 { "" } else { "└─┬" };
            items.push(format!("{}{}{}", indent, prefix, name));
            paths.push(p.clone());
        }
        let cur = comps.len().saturating_sub(1);
        // Подкаталоги текущего.
        let mut subs: Vec<String> = std::fs::read_dir(&dir)
            .map(|rd| {
                rd.flatten()
                    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .filter(|n| !n.starts_with('.'))
                    .collect()
            })
            .unwrap_or_default();
        subs.sort();
        let indent = " ".repeat(comps.len() * 2);
        for (i, s) in subs.iter().enumerate() {
            let mark = if i + 1 == subs.len() { "└──" } else { "├──" };
            items.push(format!("{}{}{}", indent, mark, s));
            paths.push(dir.join(s));
        }
        self.lb.set_items(items);
        self.paths = paths;
        self.dir = dir;
        self.cur = cur;
        self.lb.focus_item(cur);
    }

    pub fn focused_path(&self) -> Option<&PathBuf> {
        self.paths.get(self.lb.focused)
    }
}

impl View for DirListBox {
    crate::impl_view_basics_field!(lb);

    fn draw(&self, dc: &mut DrawCtx) {
        self.lb.draw(dc);
    }

    fn palette(&self) -> &'static [u8] {
        self.lb.palette()
    }

    fn set_state(&mut self, flag: u16, enable: bool) {
        self.lb.set_state(flag, enable);
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        if let Event::MouseDown(m) = ev {
            if m.double {
                self.activated = true;
            }
        }
        self.lb.handle_event(ev, ctx);
    }
}

/// Диалог смены каталога (`TChDirDialog`).
pub struct ChDirDialog {
    pub dlg: Dialog,
    input_id: ViewId,
    list_id: ViewId,
    ok_id: ViewId,
}

/// Флаги `TChDirDialog`.
pub mod cd {
    pub const NORMAL: u16 = 0x0000;
    /// Не читать текущий каталог при создании.
    pub const NO_LOAD_DIR: u16 = 0x0001;
    pub const HELP_BUTTON: u16 = 0x0002;
}

impl ChDirDialog {
    /// Аналог `TChDirDialog(options, histId)`.
    pub fn new(options: u16, hist_id: u16) -> Self {
        let mut dlg = Dialog::new(Rect::new(16, 2, 64, 20), tr("Change Directory"));
        dlg.win.group.vs.options |= of::CENTERED;
        let mut il = InputLine::new(Rect::new(3, 3, 30, 4), 260);
        il.history_id = Some(hist_id);
        let input_id = dlg.add(il);
        dlg.add(Label::new(Rect::new(2, 2, 19, 3), tr("Directory ~n~ame"), Some(input_id)));
        dlg.add(History::new(Rect::new(30, 3, 33, 4), input_id, hist_id));
        let sb = dlg.add(ScrollBar::new(Rect::new(32, 6, 33, 16)));
        let list_id = dlg.add(DirListBox::new(Rect::new(3, 6, 32, 16), Some(sb)));
        dlg.add(Label::new(Rect::new(2, 5, 20, 6), tr("Directory ~t~ree"), Some(list_id)));
        let ok_id = dlg.add(Button::new(Rect::new(35, 6, 45, 8), "O~K~", cm::OK, bf::DEFAULT));
        dlg.add(Button::new(Rect::new(35, 9, 45, 11), tr("~C~hdir"), cm::CHANGE_DIR, bf::NORMAL));
        dlg.add(Button::new(Rect::new(35, 12, 45, 14), tr("~R~evert"), cm::REVERT, bf::NORMAL));
        if options & cd::HELP_BUTTON != 0 {
            dlg.add(Button::new(Rect::new(35, 15, 45, 17), tr("Help"), cm::HELP, bf::NORMAL));
        }
        dlg.select(input_id);
        let mut d = ChDirDialog { dlg, input_id, list_id, ok_id };
        if options & cd::NO_LOAD_DIR == 0 {
            d.set_up_dialog();
        }
        d
    }

    /// Заполнить строку и дерево текущим каталогом (`setUpDialog`).
    fn set_up_dialog(&mut self) {
        let cur = std::env::current_dir().unwrap_or_default();
        if let Some(l) = self.dlg.win.group.find_mut::<DirListBox>(self.list_id) {
            l.new_directory(&cur);
        }
        if let Some(il) = self.dlg.win.group.find_mut::<InputLine>(self.input_id) {
            il.set_text(&cur.display().to_string());
        }
    }

    /// Выбранный в строке ввода каталог.
    pub fn directory(&self) -> PathBuf {
        let t = self.dlg.win.group.find::<InputLine>(self.input_id).map(|i| i.text()).unwrap_or_default();
        normalize(&crate::widgets::editor::absolute(Path::new(t.trim())))
    }
}

impl View for ChDirDialog {
    crate::impl_group_basics!(dlg.win.group);

    fn as_window(&self) -> Option<&crate::widgets::Window> {
        Some(&self.dlg.win)
    }

    fn draw(&self, dc: &mut DrawCtx) {
        self.dlg.draw(dc);
    }

    fn palette(&self) -> &'static [u8] {
        self.dlg.palette()
    }

    fn set_state(&mut self, flag: u16, enable: bool) {
        self.dlg.set_state(flag, enable);
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        self.dlg.handle_event(ev, ctx);
        let activated = self.dlg.win.group.find_mut::<DirListBox>(self.list_id).map(|l| std::mem::take(&mut l.activated)).unwrap_or(false);
        let cmd = ev.command();
        if activated || cmd == Some(cm::CHANGE_DIR) || cmd == Some(cm::REVERT) {
            let target = match cmd {
                Some(cm::REVERT) => std::env::current_dir().unwrap_or_default(),
                Some(cm::CHANGE_DIR) => {
                    // Переход по выбранному в дереве каталогу.
                    match self.dlg.win.group.find::<DirListBox>(self.list_id).and_then(|l| l.focused_path().cloned()) {
                        Some(p) => p,
                        None => self.directory(),
                    }
                }
                _ => match self.dlg.win.group.find::<DirListBox>(self.list_id).and_then(|l| l.focused_path().cloned()) {
                    Some(p) => p,
                    None => return,
                },
            };
            if let Some(l) = self.dlg.win.group.find_mut::<DirListBox>(self.list_id) {
                l.new_directory(&target);
            }
            if let Some(il) = self.dlg.win.group.find_mut::<InputLine>(self.input_id) {
                il.set_text(&target.display().to_string());
            }
            if cmd.is_some() {
                ev.clear();
            }
        }
        // Кнопка OK доступна всегда.
        let _ = self.ok_id;
    }

    /// `TChDirDialog::valid`: на OK — сменить текущий каталог программы.
    fn valid(&mut self, command: u16, ctx: &mut Ctx) -> bool {
        if command != cm::OK {
            return true;
        }
        let dir = self.directory();
        match std::env::set_current_dir(&dir) {
            Ok(()) => true,
            Err(_) => {
                let d = build_message_box(
                    Rect::new(0, 0, 50, 9),
                    &format!("{} '{}'", tr("Invalid directory:"), dir.display()),
                    mf::ERROR | mf::OK_BUTTON,
                );
                ctx.exec(Box::new(d), None, 0);
                ctx.focus(self.input_id);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcards() {
        assert!(wild_match("*.txt", "a.txt"));
        assert!(!wild_match("*.txt", "a.txt.bak"));
        assert!(wild_match("data?.csv", "data1.csv"));
        assert!(wild_match("*", "anything"));
        assert!(wild_match("*.*", "noext"));
        assert!(wild_match("a*b*c", "aXXbYYc"));
    }

    #[test]
    fn sorting() {
        let f = |n: &str, d: bool| FileEntry { name: n.into(), is_dir: d, size: 0, modified: None };
        let mut v = vec![f("..", true), f("zdir", true), f("b.txt", false), f("a.txt", false)];
        v.sort_by(compare_entries);
        let names: Vec<_> = v.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["a.txt", "b.txt", "zdir", ".."]);
    }

    #[test]
    fn dates() {
        let t = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1709296440);
        let (y, m, _, _, _) = civil_time(t);
        assert_eq!((y, m), (2024, 3));
    }
}
