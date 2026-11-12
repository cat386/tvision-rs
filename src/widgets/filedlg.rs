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
use crate::event::{cm, Event, Info, Key};
use crate::geometry::{Point, Rect};
use crate::text::to_upper;
use crate::view::{gf, of, Ctx, DrawCtx, Value, View, ViewId, ViewState};
use crate::widgets::button::{bf, Button};
use crate::widgets::dialog::Dialog;
use crate::widgets::filepanel::available_roots;
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
    /// Кнопка «Диск…» (выбор диска). На Windows она есть всегда; флаг
    /// добавляет её и в других системах (там в списке — `/`, домашний
    /// каталог и точки монтирования).
    pub const DRIVE_BUTTON: u16 = 0x0020;
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
    pub const DRIVE: &'static str = "~D~rive...";
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
    fn size_policy(&self) -> crate::layout::SizePolicy {
        crate::layout::SizePolicy::EXPANDING
    }
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
        if drive_button_wanted(options & fd::DRIVE_BUTTON != 0) {
            add_button(&mut dlg, &mut r, tr(FileDialogText::DRIVE), cm::CHANGE_DRIVE, bf::NORMAL);
        }
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

    fn as_window_mut(&mut self) -> Option<&mut crate::widgets::Window> {
        Some(&mut self.dlg.win)
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
            Some(cm::CHANGE_DRIVE) => {
                ctx.exec(Box::new(DriveDialog::new(&available_roots(), &self.directory)), Some(self.dlg.win.group.vs.id), 0);
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

    /// Выбран диск — перейти в его корень.
    fn exec_result(&mut self, _tag: u16, command: u16, view: Box<dyn View>, ctx: &mut Ctx) {
        if let Some(root) = drive_choice(command, view) {
            self.directory = root;
            self.read_directory();
            ctx.focus(self.list_id);
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
    fn size_policy(&self) -> crate::layout::SizePolicy {
        crate::layout::SizePolicy::EXPANDING
    }
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
    /// Кнопка «Диск…»: на Windows есть всегда, флаг — и в других системах.
    pub const DRIVE_BUTTON: u16 = 0x0004;
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
        // «Диск…» — рядом со строкой ввода.
        if drive_button_wanted(options & cd::DRIVE_BUTTON != 0) {
            dlg.add(Button::new(Rect::new(35, 3, 45, 5), tr(FileDialogText::DRIVE), cm::CHANGE_DRIVE, bf::NORMAL));
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

    fn as_window_mut(&mut self) -> Option<&mut crate::widgets::Window> {
        Some(&mut self.dlg.win)
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
        if ev.command() == Some(cm::CHANGE_DRIVE) {
            ctx.exec(Box::new(DriveDialog::new(&available_roots(), &self.directory())), Some(self.dlg.win.group.vs.id), 0);
            ev.clear();
            return;
        }
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

    /// Выбран диск — дерево и строка ввода переходят в его корень.
    fn exec_result(&mut self, _tag: u16, command: u16, view: Box<dyn View>, ctx: &mut Ctx) {
        if let Some(root) = drive_choice(command, view) {
            if let Some(l) = self.dlg.win.group.find_mut::<DirListBox>(self.list_id) {
                l.new_directory(&root);
            }
            if let Some(il) = self.dlg.win.group.find_mut::<InputLine>(self.input_id) {
                il.set_text(&root.display().to_string());
            }
            ctx.focus(self.list_id);
        }
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

// ----------------------------------------------------------------------
// Выбор диска
// ----------------------------------------------------------------------

/// Показывать ли кнопку «Диск…»: на Windows — всегда, иначе — по флагу.
fn drive_button_wanted(flag: bool) -> bool {
    cfg!(windows) || flag
}

/// Корень пути: `C:\` для `C:\work\a.txt`, `/` для `/home/a`.
pub fn root_of(p: &Path) -> PathBuf {
    p.ancestors().filter(|a| !a.as_os_str().is_empty()).last().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from(MAIN_SEPARATOR.to_string()))
}

/// Номер диска, на котором лежит `path`: самый длинный из корней, с
/// которого путь начинается (`/media/usb/a` — `/media/usb`, а не `/`).
pub fn current_drive(roots: &[PathBuf], path: &Path) -> Option<usize> {
    roots.iter().enumerate().filter(|(_, r)| path.starts_with(r)).max_by_key(|(_, r)| r.as_os_str().len()).map(|(i, _)| i)
}

/// Надпись диска в списке: `C:` для `C:\`, остальное — путь целиком.
fn drive_label(p: &Path) -> String {
    let s = p.display().to_string();
    let t = s.trim_end_matches(['\\', '/']);
    if t.len() == 2 && t.ends_with(':') {
        t.to_string()
    } else {
        s
    }
}

/// Корень выбранного диска из закрытого [`DriveDialog`].
fn drive_choice(command: u16, view: Box<dyn View>) -> Option<PathBuf> {
    if command != cm::OK {
        return None;
    }
    view.into_any().downcast::<DriveDialog>().ok().and_then(|d| d.selection())
}

/// Выбор диска (кнопка «Диск…» файловых диалогов): список дисков, курсор —
/// на текущем; Enter или двойной щелчок — выбрать.
pub struct DriveDialog {
    pub dlg: Dialog,
    list_id: ViewId,
    /// Корни дисков в порядке списка.
    pub roots: Vec<PathBuf>,
}

impl DriveDialog {
    /// Диалог для дисков `roots`; курсор — на диске, где лежит `current`.
    pub fn new(roots: &[PathBuf], current: &Path) -> DriveDialog {
        let labels: Vec<String> = roots.iter().map(|r| drive_label(r)).collect();
        let w = labels.iter().map(|l| crate::text::cstr_width(l) as i32).max().unwrap_or(2).clamp(12, 50) + 6;
        let rows = (roots.len() as i32).clamp(1, 12);
        let title = tr("Drive");
        let w = w.max(crate::text::cstr_width(title) as i32 + 10);
        let mut dlg = Dialog::new(Rect::new(0, 0, w, rows + 5), title);
        dlg.win.group.vs.options |= of::CENTERED;
        let mut list = ListBox::new(Rect::new(2, 2, w - 2, rows + 2), 1, None);
        let focused = current_drive(roots, current).unwrap_or(0);
        list.set_items(labels);
        list.focus_item(focused);
        let list_id = dlg.add(list);
        // Невидимая кнопка по умолчанию: Enter в списке — OK.
        let mut ok = Button::new(Rect::new(0, 0, 0, 0), "O~K~", cm::OK, bf::DEFAULT);
        ok.vs.options &= !of::SELECTABLE;
        dlg.add(ok);
        dlg.select(list_id);
        DriveDialog { dlg, list_id, roots: roots.to_vec() }
    }

    /// Выбранный диск.
    pub fn selection(&self) -> Option<PathBuf> {
        let i = self.dlg.win.group.find::<ListBox>(self.list_id)?.focused;
        self.roots.get(i).cloned()
    }
}

impl View for DriveDialog {
    crate::impl_group_basics!(dlg.win.group);

    fn as_window(&self) -> Option<&crate::widgets::Window> {
        Some(&self.dlg.win)
    }

    fn as_window_mut(&mut self) -> Option<&mut crate::widgets::Window> {
        Some(&mut self.dlg.win)
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
        // Двойной щелчок по диску — выбрать.
        if let Event::Broadcast(m) = ev {
            if m.command == cm::LIST_ITEM_SELECTED && matches!(m.info, Info::Id(id) if id == self.list_id) {
                ctx.end_modal(cm::OK);
                ev.clear();
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

    // ---------- выбор диска ----------

    use crate::event::{KeyEvent, Mods};
    use crate::platform::HeadlessBackend;
    use crate::Program;

    fn app_with(keys: &[KeyEvent]) -> Program {
        let mut b = HeadlessBackend::new(80, 25);
        for k in keys {
            b.push_key(*k);
        }
        Program::new(Box::new(b), None, None)
    }

    fn alt(c: char) -> KeyEvent {
        KeyEvent::new(Key::Char(c), Mods::ALT)
    }

    #[test]
    fn drive_labels_and_roots() {
        assert_eq!(drive_label(Path::new("C:\\")), "C:");
        assert_eq!(drive_label(Path::new("/media/usb")), "/media/usb");
        if cfg!(windows) {
            assert_eq!(root_of(Path::new("C:\\work\\a.txt")), PathBuf::from("C:\\"));
        } else {
            assert_eq!(root_of(Path::new("/home/a/b")), PathBuf::from("/"));
        }
        let d = DriveDialog::new(&[PathBuf::from("C:\\"), PathBuf::from("D:\\")], Path::new("D:\\"));
        assert_eq!(d.selection(), Some(PathBuf::from("D:\\")), "курсор — на текущем диске");
        let roots = [PathBuf::from("/"), PathBuf::from("/media/usb")];
        assert_eq!(current_drive(&roots, Path::new("/media/usb/фото")), Some(1));
        assert_eq!(current_drive(&roots, Path::new("/home/a")), Some(0));
    }

    #[test]
    fn drive_button_only_where_needed() {
        let has = |d: &FileDialog| {
            d.dlg.win.group.children.iter().any(|v| v.as_any().downcast_ref::<Button>().is_some_and(|b| b.command == cm::CHANGE_DRIVE))
        };
        assert_eq!(has(&FileDialog::new("*", "Открыть", "~N~ame", fd::OPEN_BUTTON, 1)), cfg!(windows), "на Windows — всегда, иначе — по флагу");
        assert!(has(&FileDialog::new("*", "Открыть", "~N~ame", fd::OPEN_BUTTON | fd::DRIVE_BUTTON, 1)));
    }

    /// Кнопка «Диск…» → список дисков → Enter: диалог в корне диска.
    #[test]
    fn file_dialog_changes_drive() {
        let mut app = app_with(&[alt('d'), crate::event::kb::ENTER, crate::event::kb::ESC]);
        let d = FileDialog::new("*", "Открыть", "~N~ame", fd::OPEN_BUTTON | fd::DRIVE_BUTTON, 1);
        let roots = available_roots();
        let expect = roots[current_drive(&roots, &d.directory).unwrap()].clone();
        let (_cmd, d) = app.exec_dialog(d);
        assert_eq!(d.directory, expect);
    }

    #[test]
    fn chdir_dialog_changes_drive() {
        let mut app = app_with(&[alt('d'), crate::event::kb::ENTER, crate::event::kb::ESC]);
        let roots = available_roots();
        let expect = roots[current_drive(&roots, &std::env::current_dir().unwrap()).unwrap()].clone();
        let (_cmd, d) = app.exec_dialog(ChDirDialog::new(cd::DRIVE_BUTTON, 1));
        assert_eq!(d.directory(), expect);
    }

    /// По-русски кнопка — «Дис~к~…» (Alt+К); список дисков поверх диалога.
    #[test]
    fn drive_button_in_russian() {
        use std::cell::RefCell;
        use std::rc::Rc;
        crate::locale::set_language(crate::locale::Language::Ru);
        let f12 = KeyEvent::new(Key::F(12), Mods::NONE);
        let mut app = app_with(&[f12, alt('к'), f12, crate::event::kb::ESC, crate::event::kb::ESC]);
        // Снимки экрана: F12 никто не обрабатывает — событие попадает в modal_hook.
        let shots = Rc::new(RefCell::new(Vec::<String>::new()));
        let sh = shots.clone();
        app.modal_hook = Some(Box::new(move |app, _| {
            app.redraw();
            sh.borrow_mut().push(app.screen_text());
        }));
        app.exec_dialog(FileDialog::new("*.txt", "Открыть файл", "~И~мя", fd::OPEN_BUTTON | fd::DRIVE_BUTTON, 1));
        crate::locale::set_language(crate::locale::Language::En);
        let shots = shots.borrow();
        if std::env::var("SHOW_DRIVE").is_ok() {
            println!("{}\n{}", shots[0], shots.get(1).cloned().unwrap_or_default());
        }
        assert!(shots[0].contains("Диск..."), "{}", shots[0]);
        assert!(shots.len() == 2 && shots[1].contains("═ Диск ═"), "Alt+К открывает список дисков");
    }
}
