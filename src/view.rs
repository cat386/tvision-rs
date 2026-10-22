//! Базовое представление (`TView`), контекст событий и контекст отрисовки.
//!
//! В C++ `TView` — базовый класс с виртуальными методами `draw()`,
//! `handleEvent()`, `setState()`, `getPalette()`, `valid()` и т.д.
//! В Rust это трейт [`View`]. Общие поля `TView` (`origin`, `size`, `state`,
//! `options`, `growMode`, ...) вынесены в структуру [`ViewState`], которую
//! каждое представление хранит у себя и отдаёт через `vs()`/`vs_mut()`.

use std::any::Any;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::color::{Attr, AttrPair, ERROR_ATTR, SHADOW_ATTR};
use crate::drawbuf::{Cell, DrawBuffer, ScreenBuffer};
use crate::event::{Event, Info, Message};
use crate::geometry::{Point, Rect};
use crate::group::Group;
use crate::text::char_width;

/// Уникальный идентификатор представления. В C++ представления
/// сравнивались по указателям; в Rust вместо указателей используются id.
pub type ViewId = u32;

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

/// Выдаёт новый уникальный идентификатор.
pub fn next_id() -> ViewId {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// Флаги состояния (`sfXXX`). Поле `ViewState::state`.
pub mod sf {
    /// Представление видимо.
    pub const VISIBLE: u16 = 0x001;
    /// Курсор видим.
    pub const CURSOR_VIS: u16 = 0x002;
    /// Курсор в режиме вставки (блочный).
    pub const CURSOR_INS: u16 = 0x004;
    /// У представления есть тень.
    pub const SHADOW: u16 = 0x008;
    /// Представление находится в активном окне.
    pub const ACTIVE: u16 = 0x010;
    /// Представление выбрано (текущее у владельца).
    pub const SELECTED: u16 = 0x020;
    /// Представление в фокусе (выбрано на всей цепочке до корня).
    pub const FOCUSED: u16 = 0x040;
    /// Окно перетаскивается мышью.
    pub const DRAGGING: u16 = 0x080;
    /// Представление недоступно.
    pub const DISABLED: u16 = 0x100;
    /// Представление выполняется модально.
    pub const MODAL: u16 = 0x200;
    /// Кнопка «по умолчанию».
    pub const DEFAULT: u16 = 0x400;
    /// Представление видно на экране.
    pub const EXPOSED: u16 = 0x800;
}

/// Флаги параметров (`ofXXX`). Поле `ViewState::options`.
pub mod of {
    /// Может получать фокус.
    pub const SELECTABLE: u16 = 0x001;
    /// При выборе поднимается наверх (окна).
    pub const TOP_SELECT: u16 = 0x002;
    /// Щелчок по невыбранному представлению не только выбирает его, но и обрабатывается.
    pub const FIRST_CLICK: u16 = 0x004;
    /// Есть рамка.
    pub const FRAMED: u16 = 0x008;
    /// Получает фокусные события ДО текущего представления владельца.
    pub const PRE_PROCESS: u16 = 0x010;
    /// Получает фокусные события ПОСЛЕ текущего представления владельца.
    pub const POST_PROCESS: u16 = 0x020;
    /// Буферизация (в этом порте не используется — всё рисуется в теневой буфер).
    pub const BUFFERED: u16 = 0x040;
    /// Участвует в «мозаике» и «каскаде» окон.
    pub const TILEABLE: u16 = 0x080;
    /// Центрировать по горизонтали при вставке.
    pub const CENTER_X: u16 = 0x100;
    /// Центрировать по вертикали при вставке.
    pub const CENTER_Y: u16 = 0x200;
    /// Центрировать по обеим осям.
    pub const CENTERED: u16 = 0x300;
    /// Проверять `valid()` при потере фокуса.
    pub const VALIDATE: u16 = 0x400;
}

/// Режимы изменения размера при изменении размера владельца (`gfXXX`).
pub mod gf {
    pub const GROW_LO_X: u8 = 0x01;
    pub const GROW_LO_Y: u8 = 0x02;
    pub const GROW_HI_X: u8 = 0x04;
    pub const GROW_HI_Y: u8 = 0x08;
    pub const GROW_ALL: u8 = 0x0F;
    /// Пропорциональное изменение (окна на рабочем столе).
    pub const GROW_REL: u8 = 0x10;
    /// Не ограничивать размер размером владельца.
    pub const FIXED: u8 = 0x20;
}

/// Режимы перетаскивания (`dmXXX`).
pub mod dm {
    pub const DRAG_MOVE: u8 = 0x01;
    pub const DRAG_GROW: u8 = 0x02;
    pub const DRAG_GROW_LEFT: u8 = 0x04;
    pub const LIMIT_LO_X: u8 = 0x10;
    pub const LIMIT_LO_Y: u8 = 0x20;
    pub const LIMIT_HI_X: u8 = 0x40;
    pub const LIMIT_HI_Y: u8 = 0x80;
    pub const LIMIT_ALL: u8 = 0xF0;
}

/// Справочный контекст «нет контекста» (`hcNoContext`).
pub const HC_NO_CONTEXT: u16 = 0;
/// Справочный контекст во время перетаскивания (`hcDragging`).
pub const HC_DRAGGING: u16 = 1;

/// Общие поля любого представления (поля класса `TView`).
#[derive(Clone, Debug)]
pub struct ViewState {
    /// Уникальный идентификатор.
    pub id: ViewId,
    /// Левый верхний угол в координатах владельца.
    pub origin: Point,
    /// Размер.
    pub size: Point,
    /// Позиция курсора в локальных координатах.
    pub cursor: Point,
    /// Флаги состояния `sf::*`.
    pub state: u16,
    /// Флаги параметров `of::*`.
    pub options: u16,
    /// Режим изменения размера `gf::*`.
    pub grow_mode: u8,
    /// Режим перетаскивания `dm::*`.
    pub drag_mode: u8,
    /// Номер контекста справки (для строки состояния и справки).
    pub help_ctx: u16,
}

impl ViewState {
    /// Аналог конструктора `TView(bounds)`.
    pub fn new(bounds: Rect) -> Self {
        ViewState {
            id: next_id(),
            origin: bounds.a,
            size: bounds.size(),
            cursor: Point::default(),
            state: sf::VISIBLE,
            options: 0,
            grow_mode: 0,
            drag_mode: dm::LIMIT_LO_Y,
            help_ctx: HC_NO_CONTEXT,
        }
    }

    /// Границы в координатах владельца (`getBounds`).
    pub fn bounds(&self) -> Rect {
        Rect::from_origin_size(self.origin, self.size)
    }

    /// Границы в собственных координатах: (0,0)-(size) (`getExtent`).
    pub fn extent(&self) -> Rect {
        Rect::from_origin_size(Point::default(), self.size)
    }

    /// Установить границы (`setBounds`).
    pub fn set_bounds(&mut self, r: Rect) {
        self.origin = r.a;
        self.size = r.size();
    }

    /// Перевести точку из координат владельца в локальные (`makeLocal`).
    pub fn make_local(&self, p: Point) -> Point {
        p - self.origin
    }

    /// Установлен ли флаг состояния (`getState`).
    pub fn get_state(&self, flag: u16) -> bool {
        self.state & flag == flag
    }

    pub fn is_visible(&self) -> bool {
        self.state & sf::VISIBLE != 0
    }

    pub fn is_disabled(&self) -> bool {
        self.state & sf::DISABLED != 0
    }

    pub fn is_selected(&self) -> bool {
        self.state & sf::SELECTED != 0
    }

    pub fn is_focused(&self) -> bool {
        self.state & sf::FOCUSED != 0
    }

    pub fn is_active(&self) -> bool {
        self.state & sf::ACTIVE != 0
    }

    pub fn is_selectable(&self) -> bool {
        self.options & of::SELECTABLE != 0
    }

    /// Курсор: показать / скрыть / поставить / режим вставки.
    pub fn show_cursor(&mut self) {
        self.state |= sf::CURSOR_VIS;
    }
    pub fn hide_cursor(&mut self) {
        self.state &= !sf::CURSOR_VIS;
    }
    pub fn set_cursor(&mut self, x: i32, y: i32) {
        self.cursor = Point::new(x, y);
    }
    pub fn block_cursor(&mut self) {
        self.state |= sf::CURSOR_INS;
    }
    pub fn normal_cursor(&mut self) {
        self.state &= !sf::CURSOR_INS;
    }
}

/// Данные для обмена с элементами диалога (`getData`/`setData`).
///
/// В C++ данные передавались как сырой блок памяти (`void* rec`) нужного
/// размера (`dataSize()`). В Rust вместо этого используется типизированное
/// значение. Группа (диалог) возвращает `Value::List` со значениями своих
/// элементов в порядке вставки.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum Value {
    #[default]
    None,
    /// Строка (строка ввода `InputLine`).
    Str(String),
    /// Битовая маска / номер (флажки `CheckBoxes`, переключатели `RadioButtons`).
    U32(u32),
    /// Номер элемента (список `ListBox`).
    Int(i64),
    /// Список значений (группа элементов).
    List(Vec<Value>),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Value::U32(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(v) => Some(v),
            _ => None,
        }
    }
}

/// Фаза обработки фокусного события (`phase` у `TGroup`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Phase {
    /// Событие получают представления с `of::PRE_PROCESS`.
    PreProcess,
    /// Событие получает текущее (выбранное) представление.
    #[default]
    Focused,
    /// Событие получают представления с `of::POST_PROCESS`.
    PostProcess,
}

/// Сообщение владельцу. Это то, что в C++ делалось вызовами методов `owner`.
#[derive(Debug)]
pub(crate) enum Msg {
    /// `message(owner, evBroadcast, cmd, info)` — разослать всем детям владельца.
    Broadcast(Message),
    /// `view->select()` — сделать представление текущим.
    Select(ViewId),
    /// `destroy(view)` / `close()` — удалить представление из владельца.
    Close(ViewId),
    /// `owner->selectNext(forwards)`.
    SelectNext(bool),
    /// `view->makeFirst()` — поднять наверх без выбора.
    MakeFirst(ViewId),
}

/// Запрос к программе (то, что требует доступа к корню дерева).
pub enum Request {
    /// Открыть меню строки меню; `Some(i)` — сразу раскрыть пункт `i`;
    /// `mouse` — меню открыто щелчком мыши.
    MenuBar { item: Option<usize>, mouse: bool },
    /// Показать всплывающее (контекстное) меню в абсолютной позиции.
    Popup { menu: Rc<crate::widgets::menu::Menu>, pos: Point },
    /// Выполнить представление модально (`owner->execView(view)` из C++).
    /// Если задан `anchor`, границы `view` заданы в системе координат
    /// владельца представления `anchor` (например, окно истории — рядом со
    /// строкой ввода в диалоге); иначе — в координатах рабочего стола.
    /// По завершении у представления `reply_to` вызывается `exec_result`.
    Exec { view: Box<dyn View>, reply_to: Option<ViewId>, tag: u16, anchor: Option<ViewId>, place: Option<PlaceFn> },
    /// Передать фокус представлению (выбрать его и всех его владельцев).
    Focus(ViewId),
}

/// Функция размещения модального окна относительно представления-якоря:
/// получает границы якоря и размер его владельца, возвращает границы окна
/// (в координатах того же владельца).
pub type PlaceFn = Box<dyn Fn(Rect, Point) -> Rect>;

/// Набор запрещённых команд (аналог `TCommandSet`).
#[derive(Clone, Debug, Default)]
pub struct CommandSet {
    disabled: HashSet<u16>,
}

impl CommandSet {
    /// Разрешена ли команда. Команды больше 255 разрешены всегда (как в оригинале).
    pub fn is_enabled(&self, cmd: u16) -> bool {
        cmd > 255 || !self.disabled.contains(&cmd)
    }
    pub fn enable(&mut self, cmd: u16) -> bool {
        self.disabled.remove(&cmd)
    }
    pub fn disable(&mut self, cmd: u16) -> bool {
        cmd <= 255 && self.disabled.insert(cmd)
    }
}

/// Контекст обработки событий. Передаётся в `View::handle_event`.
///
/// Через него представление делает то, что в C++ делалось через указатель
/// на владельца и глобальные объекты (`TProgram::application`, `putEvent`...).
pub struct Ctx {
    /// Очередь отложенных событий (`putEvent`).
    pub(crate) queue: VecDeque<Event>,
    /// «Почтовый ящик» для владельцев: (глубина отправителя, сообщение).
    pub(crate) outbox: Vec<(u32, Msg)>,
    /// Текущая глубина вложенности при доставке события.
    pub(crate) depth: u32,
    /// Размеры владельцев на пути доставки (вершина — владелец текущего представления).
    pub(crate) owner_sizes: Vec<Point>,
    /// Текущая фаза обработки (для `of::PRE_PROCESS`/`POST_PROCESS`).
    pub phase: Phase,
    /// Представление, «захватившее» мышь (получает все её события до отпускания).
    pub(crate) mouse_grab: Option<ViewId>,
    /// Код завершения модального цикла (`endModal`).
    pub(crate) end_state: Option<u16>,
    /// Разрешённые/запрещённые команды.
    pub(crate) commands: CommandSet,
    pub(crate) commands_changed: bool,
    /// Запросы к программе.
    pub(crate) requests: Vec<Request>,
    /// Размер экрана.
    pub screen_size: Point,
    /// Внутренний буфер обмена (для строк ввода).
    pub clipboard: String,
}

impl Default for Ctx {
    fn default() -> Self {
        Ctx::new()
    }
}

impl Ctx {
    pub fn new() -> Self {
        Ctx {
            queue: VecDeque::new(),
            outbox: Vec::new(),
            depth: 0,
            owner_sizes: Vec::new(),
            phase: Phase::Focused,
            mouse_grab: None,
            end_state: None,
            commands: CommandSet::default(),
            commands_changed: false,
            requests: Vec::new(),
            screen_size: Point::new(80, 25),
            clipboard: String::new(),
        }
    }

    /// Поставить событие в очередь (`putEvent`). Оно будет обработано
    /// на следующей итерации цикла событий.
    pub fn put_event(&mut self, ev: Event) {
        self.queue.push_back(ev);
    }

    /// Послать команду (удобная обёртка над `put_event`).
    pub fn post_command(&mut self, command: u16, info: Info) {
        self.put_event(Event::Command(Message::with(command, info)));
    }

    /// Разослать широковещательное сообщение всем представлениям владельца
    /// (аналог `message(owner, evBroadcast, command, info)`).
    /// Сообщение доставляется сразу после возврата из текущего `handle_event`.
    pub fn message(&mut self, command: u16, info: Info) {
        self.outbox.push((self.depth, Msg::Broadcast(Message::with(command, info))));
    }

    /// Сделать представление `id` текущим у его владельца (`select()`).
    /// Вызывается представлением для самого себя или для «соседа» (метка → поле ввода).
    pub fn select(&mut self, id: ViewId) {
        self.outbox.push((self.depth, Msg::Select(id)));
    }

    /// Удалить представление `id` из владельца (`close()` / `destroy()`).
    pub fn close(&mut self, id: ViewId) {
        self.outbox.push((self.depth, Msg::Close(id)));
    }

    /// Попросить владельца выбрать следующее (`true`) или предыдущее представление.
    pub fn select_next(&mut self, forward: bool) {
        self.outbox.push((self.depth, Msg::SelectNext(forward)));
    }

    /// Поднять представление наверх по Z-порядку.
    pub fn make_first(&mut self, id: ViewId) {
        self.outbox.push((self.depth, Msg::MakeFirst(id)));
    }

    /// Завершить модальный цикл с кодом `command` (`endModal`).
    pub fn end_modal(&mut self, command: u16) {
        self.end_state = Some(command);
    }

    /// Захватить мышь: все события мыши пойдут в представление `id`,
    /// пока оно не вызовет `release_mouse`. Заменяет циклы
    /// `while (mouseEvent(event, evMouseMove))` из C++.
    pub fn grab_mouse(&mut self, id: ViewId) {
        self.mouse_grab = Some(id);
    }

    pub fn release_mouse(&mut self) {
        self.mouse_grab = None;
    }

    pub fn mouse_grabbed_by(&self, id: ViewId) -> bool {
        self.mouse_grab == Some(id)
    }

    /// Размер владельца текущего представления.
    pub fn owner_size(&self) -> Point {
        self.owner_sizes.last().copied().unwrap_or(self.screen_size)
    }

    /// Разрешена ли команда (`commandEnabled`).
    pub fn command_enabled(&self, cmd: u16) -> bool {
        self.commands.is_enabled(cmd)
    }

    /// Разрешить команду (`enableCommand`).
    pub fn enable_command(&mut self, cmd: u16) {
        if self.commands.enable(cmd) {
            self.commands_changed = true;
        }
    }

    /// Запретить команду (`disableCommand`).
    pub fn disable_command(&mut self, cmd: u16) {
        if self.commands.disable(cmd) {
            self.commands_changed = true;
        }
    }

    pub fn set_command(&mut self, cmd: u16, enable: bool) {
        if enable {
            self.enable_command(cmd)
        } else {
            self.disable_command(cmd)
        }
    }

    /// Отправить запрос программе.
    pub fn request(&mut self, r: Request) {
        self.requests.push(r);
    }

    /// Запустить представление модально. Результат придёт в
    /// `View::exec_result` представления `reply_to` с меткой `tag`.
    pub fn exec(&mut self, view: Box<dyn View>, reply_to: Option<ViewId>, tag: u16) {
        self.requests.push(Request::Exec { view, reply_to, tag, anchor: None, place: None });
    }

    /// Как [`Ctx::exec`], но границы `view` заданы относительно владельца `anchor`.
    pub fn exec_near(&mut self, view: Box<dyn View>, reply_to: Option<ViewId>, tag: u16, anchor: ViewId) {
        self.requests.push(Request::Exec { view, reply_to, tag, anchor: Some(anchor), place: None });
    }

    /// Как [`Ctx::exec_near`], но границы окна вычисляются функцией `place`
    /// по границам якоря и размеру его владельца (так `THistory` ставит
    /// окно истории под строкой ввода).
    pub fn exec_placed(&mut self, view: Box<dyn View>, reply_to: Option<ViewId>, tag: u16, anchor: ViewId, place: PlaceFn) {
        self.requests.push(Request::Exec { view, reply_to, tag, anchor: Some(anchor), place: Some(place) });
    }

    /// Передать фокус представлению `id` (где бы оно ни находилось).
    pub fn focus(&mut self, id: ViewId) {
        self.requests.push(Request::Focus(id));
    }

    /// Показать контекстное меню в абсолютной позиции экрана.
    pub fn popup_menu(&mut self, menu: Rc<crate::widgets::menu::Menu>, pos: Point) {
        self.requests.push(Request::Popup { menu, pos });
    }
}

/// Контекст отрисовки: «окно» в экранный буфер для одного представления.
///
/// Все координаты в методах — локальные координаты представления.
/// Вывод автоматически обрезается по видимой области (своей и всех владельцев).
pub struct DrawCtx<'a> {
    screen: &'a mut ScreenBuffer,
    /// Абсолютная позиция левого верхнего угла представления.
    origin: Point,
    /// Абсолютная область, в которую можно рисовать.
    clip: Rect,
    /// Цепочка палитр от приложения (первая) до текущего представления (последняя).
    palettes: Vec<&'static [u8]>,
    /// Разрешённые команды (нужны, например, строке состояния и кнопкам).
    pub commands: &'a CommandSet,
    /// Размер текущего представления.
    pub size: Point,
}

impl<'a> DrawCtx<'a> {
    /// Корневой контекст на весь экран.
    pub fn new(screen: &'a mut ScreenBuffer, app_palette: &'static [u8], commands: &'a CommandSet) -> Self {
        let size = Point::new(screen.width as i32, screen.height as i32);
        DrawCtx {
            screen,
            origin: Point::default(),
            clip: Rect::from_origin_size(Point::default(), size),
            palettes: vec![app_palette],
            commands,
            size,
        }
    }

    /// Контекст для дочернего представления с границами `bounds`
    /// (в локальных координатах текущего) и палитрой `palette`.
    pub fn sub(&mut self, bounds: Rect, palette: &'static [u8]) -> DrawCtx<'_> {
        let abs = bounds.moved(self.origin.x, self.origin.y);
        let mut palettes = self.palettes.clone();
        palettes.push(palette);
        DrawCtx { screen: self.screen, origin: abs.a, clip: self.clip.intersect(&abs), palettes, commands: self.commands, size: bounds.size() }
    }

    /// Абсолютная позиция представления на экране.
    pub fn origin(&self) -> Point {
        self.origin
    }

    /// Перевод номера цвета через цепочку палитр (`mapColor`).
    ///
    /// Номер `color` ищется в палитре представления, полученный номер — в
    /// палитре владельца и т.д. до палитры приложения, которая даёт атрибут.
    /// Пустая палитра пропускает номер без изменений (как у групп без палитры).
    pub fn map_color(&self, color: u8) -> Attr {
        if color == 0 {
            return ERROR_ATTR;
        }
        let mut c = color;
        for pal in self.palettes.iter().rev() {
            if pal.is_empty() {
                continue;
            }
            let i = c as usize;
            if i == 0 || i > pal.len() {
                return ERROR_ATTR;
            }
            c = pal[i - 1];
            if c == 0 {
                return ERROR_ATTR;
            }
        }
        c
    }

    /// Пара цветов (`getColor`): младший байт — номер обычного цвета,
    /// старший — номер цвета подсветки.
    pub fn get_color(&self, pair: u16) -> AttrPair {
        let lo = self.map_color((pair & 0xFF) as u8) as u16;
        let hi_idx = (pair >> 8) as u8;
        let hi = if hi_idx == 0 { lo } else { self.map_color(hi_idx) as u16 };
        lo | (hi << 8)
    }

    /// Атрибут по номеру цвета (младший байт `get_color`).
    pub fn color(&self, idx: u8) -> Attr {
        self.map_color(idx)
    }

    /// Пишет одну ячейку по абсолютным координатам с учётом обрезки
    /// и широких символов.
    fn put_abs(&mut self, x: i32, y: i32, cell: Cell) {
        if !self.clip.contains(Point::new(x, y)) {
            return;
        }
        let w = self.screen.width;
        let (ux, uy) = (x as usize, y as usize);
        // Не оставляем «половинок» широких символов на экране.
        let old = self.screen.get(ux, uy);
        if old.is_wide_tail() && ux > 0 {
            let head = self.screen.get_mut(ux - 1, uy);
            if !head.is_wide_tail() && char_width(head.ch) == 2 {
                head.ch = ' ';
            }
        } else if char_width(old.ch) == 2 && ux + 1 < w && !cell.is_wide_tail() {
            let tail = self.screen.get_mut(ux + 1, uy);
            if tail.is_wide_tail() {
                tail.ch = ' ';
            }
        }
        let mut c = cell;
        // Широкий символ, вторая половина которого обрезана, заменяем пробелом.
        if char_width(c.ch) == 2 && !self.clip.contains(Point::new(x + 1, y)) {
            c.ch = ' ';
        }
        // «Хвост», голова которого обрезана, — тоже пробел.
        if c.is_wide_tail() && !self.clip.contains(Point::new(x - 1, y)) {
            c.ch = ' ';
        }
        *self.screen.get_mut(ux, uy) = c;
    }

    /// Выводит `count` ячеек буфера, начиная с (x, y), в `h` строк подряд
    /// (аналог `writeLine(x, y, w, h, buf)`).
    pub fn write_line(&mut self, x: i32, y: i32, w: i32, h: i32, buf: &DrawBuffer) {
        for dy in 0..h {
            for dx in 0..w.min(buf.len() as i32) {
                let cell = buf.cells[dx as usize];
                self.put_abs(self.origin.x + x + dx, self.origin.y + y + dy, cell);
            }
        }
    }

    /// Выводит ячейки (аналог `writeBuf` для одной строки).
    pub fn write_cells(&mut self, x: i32, y: i32, cells: &[Cell]) {
        for (dx, c) in cells.iter().enumerate() {
            self.put_abs(self.origin.x + x + dx as i32, self.origin.y + y, *c);
        }
    }

    /// Повторяет символ `count` раз (аналог `writeChar`).
    pub fn write_char(&mut self, x: i32, y: i32, ch: char, attr: Attr, count: i32) {
        for dx in 0..count {
            self.put_abs(self.origin.x + x + dx, self.origin.y + y, Cell::new(ch, attr));
        }
    }

    /// Выводит строку (аналог `writeStr`). Возвращает ширину.
    pub fn write_str(&mut self, x: i32, y: i32, s: &str, attr: Attr) -> i32 {
        let mut cx = x;
        for ch in s.chars() {
            let w = char_width(ch) as i32;
            if w == 0 {
                continue;
            }
            self.put_abs(self.origin.x + cx, self.origin.y + y, Cell::new(ch, attr));
            if w == 2 {
                self.put_abs(self.origin.x + cx + 1, self.origin.y + y, Cell::new('\0', attr));
            }
            cx += w;
        }
        cx - x
    }

    /// Заливает прямоугольник символом.
    pub fn fill(&mut self, r: Rect, ch: char, attr: Attr) {
        for y in r.a.y..r.b.y {
            self.write_char(r.a.x, y, ch, attr, r.width());
        }
    }

    /// Рисует тень справа и снизу от прямоугольника `r` (локальные координаты):
    /// 2 столбца справа и 1 строка снизу, со сдвигом — как в оригинале.
    pub fn shadow(&mut self, r: Rect) {
        let right = Rect::new(r.b.x, r.a.y + 1, r.b.x + 2, r.b.y + 1);
        let bottom = Rect::new(r.a.x + 2, r.b.y, r.b.x, r.b.y + 1);
        for rr in [right, bottom] {
            let abs = rr.moved(self.origin.x, self.origin.y).intersect(&self.clip);
            for y in abs.a.y..abs.b.y {
                for x in abs.a.x..abs.b.x {
                    let c = self.screen.get_mut(x as usize, y as usize);
                    c.attr = SHADOW_ATTR;
                }
            }
        }
    }

    /// Видна ли абсолютная точка в этом контексте (для отладки и курсора).
    pub fn clip(&self) -> Rect {
        self.clip
    }
}

/// Представление (аналог `TView`).
///
/// Обязательные методы: `vs`, `vs_mut`, `draw`, `as_any`, `as_any_mut`.
/// Для простых представлений их удобно получить макросом [`crate::impl_view_basics!`].
pub trait View: Any {
    /// Общие поля представления.
    fn vs(&self) -> &ViewState;
    fn vs_mut(&mut self) -> &mut ViewState;

    /// Нарисовать себя (`draw`). Представление должно закрасить всю свою область.
    fn draw(&self, dc: &mut DrawCtx);

    /// Обработать событие (`handleEvent`). По умолчанию — выбор щелчком мыши.
    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        default_handle_event(self.vs(), ev, ctx);
    }

    /// Палитра (`getPalette`). Пустая палитра — номера цветов передаются владельцу как есть.
    fn palette(&self) -> &'static [u8] {
        &[]
    }

    /// Изменить флаг состояния (`setState`). Группы переопределяют этот метод,
    /// чтобы передать состояние детям.
    fn set_state(&mut self, flag: u16, enable: bool) {
        let vs = self.vs_mut();
        if enable {
            vs.state |= flag;
        } else {
            vs.state &= !flag;
        }
    }

    /// Можно ли завершить работу с представлением командой `command` (`valid`).
    fn valid(&mut self, _command: u16, _ctx: &mut Ctx) -> bool {
        true
    }

    /// Результат модального окна, запущенного этим представлением через
    /// [`Ctx::exec`]: `tag` — метка запроса, `command` — код завершения,
    /// `view` — само окно (его можно привести к нужному типу через `into_any`).
    fn exec_result(&mut self, _tag: u16, _command: u16, _view: Box<dyn View>, _ctx: &mut Ctx) {}

    /// Обновить разрешённые команды (`updateCommands`). Программа вызывает
    /// этот метод после каждого события: сначала у всех представлений вне
    /// фокуса, затем у представлений в фокусе — так команды «в фокусе» побеждают.
    fn update_commands(&self, _ctx: &mut Ctx) {}

    /// Изменить границы (`changeBounds`). Группы пересчитывают детей.
    fn change_bounds(&mut self, bounds: Rect) {
        self.vs_mut().set_bounds(bounds);
    }

    /// Минимальный и максимальный размер (`sizeLimits`).
    fn size_limits(&self, owner_size: Point) -> (Point, Point) {
        let max = if self.vs().grow_mode & gf::FIXED != 0 { Point::new(i32::MAX, i32::MAX) } else { owner_size };
        (Point::new(0, 0), max)
    }

    /// Есть ли у представления данные для `get_data`/`set_data` (`dataSize() > 0`).
    fn has_data(&self) -> bool {
        false
    }
    /// Получить данные (`getData`).
    fn get_data(&self) -> Value {
        Value::None
    }
    /// Установить данные (`setData`).
    fn set_data(&mut self, _value: &Value) {}

    /// Текущий контекст справки (`getHelpCtx`).
    fn help_ctx(&self) -> u16 {
        if self.vs().state & sf::DRAGGING != 0 {
            HC_DRAGGING
        } else {
            self.vs().help_ctx
        }
    }

    /// Если представление — окно (или построено вокруг окна), вернуть его.
    /// Программа по этому узнаёт флаги активного окна для команд
    /// `cm::CLOSE`, `cm::ZOOM`, `cm::RESIZE`.
    fn as_window(&self) -> Option<&crate::widgets::Window> {
        None
    }

    /// Если представление — группа (или содержит группу), вернуть её.
    fn as_group(&self) -> Option<&Group> {
        None
    }
    fn as_group_mut(&mut self) -> Option<&mut Group> {
        None
    }

    /// Для приведения типов (`dynamic_cast`).
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    /// Превратить `Box<dyn View>` обратно в `Box<dyn Any>` (для `downcast`).
    fn into_any(self: Box<Self>) -> Box<dyn Any>;

    /// Содержит ли поддерево представление с данным id.
    fn contains_id(&self, id: ViewId) -> bool {
        if self.vs().id == id {
            return true;
        }
        self.as_group().map(|g| g.children.iter().any(|c| c.contains_id(id))).unwrap_or(false)
    }
}

/// Поведение `TView::handleEvent` по умолчанию: щелчок мышью по невыбранному
/// представлению делает его текущим. Если у представления нет `of::FIRST_CLICK`,
/// щелчок на этом и заканчивается (событие очищается).
pub fn default_handle_event(vs: &ViewState, ev: &mut Event, ctx: &mut Ctx) {
    if let Event::MouseDown(_) = ev {
        if !vs.is_selected() && vs.is_selectable() && !vs.is_disabled() {
            ctx.select(vs.id);
            if vs.options & of::FIRST_CLICK == 0 {
                ev.clear();
            }
        }
    }
}

/// Пересчёт границ представления при изменении размера владельца на `delta`
/// (порт `TView::calcBounds`). `owner_size` — НОВЫЙ размер владельца.
pub fn calc_bounds(v: &dyn View, delta: Point, owner_size: Point) -> Rect {
    let vs = v.vs();
    let mut b = vs.bounds();
    let gm = vs.grow_mode;
    let grow = |i: &mut i32, s: i32, d: i32| {
        if gm & gf::GROW_REL != 0 {
            let old = s - d;
            if old != 0 {
                *i = (*i * s + (old >> 1)) / old;
            }
        } else {
            *i += d;
        }
    };
    if gm & gf::GROW_LO_X != 0 {
        grow(&mut b.a.x, owner_size.x, delta.x);
    }
    if gm & gf::GROW_HI_X != 0 {
        grow(&mut b.b.x, owner_size.x, delta.x);
    }
    if gm & gf::GROW_LO_Y != 0 {
        grow(&mut b.a.y, owner_size.y, delta.y);
    }
    if gm & gf::GROW_HI_Y != 0 {
        grow(&mut b.b.y, owner_size.y, delta.y);
    }
    let (min, max) = v.size_limits(owner_size);
    b.b.x = b.a.x + (b.b.x - b.a.x).clamp(min.x, max.x.max(min.x));
    b.b.y = b.a.y + (b.b.y - b.a.y).clamp(min.y, max.y.max(min.y));
    b
}

/// Реализует обязательные методы трейта `View` для структуры,
/// у которой есть поле `vs: ViewState`.
///
/// ```ignore
/// impl View for MyView {
///     tvision::impl_view_basics!();
///     fn draw(&self, dc: &mut DrawCtx) { ... }
/// }
/// ```
#[macro_export]
macro_rules! impl_view_basics {
    () => {
        fn vs(&self) -> &$crate::ViewState {
            &self.vs
        }
        fn vs_mut(&mut self) -> &mut $crate::ViewState {
            &mut self.vs
        }
        fn as_any(&self) -> &dyn ::std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
            self
        }
        fn into_any(self: Box<Self>) -> Box<dyn ::std::any::Any> {
            self
        }
    };
}

/// Реализует обязательные методы трейта `View` для структуры, построенной
/// вокруг группы (поле `$g: Group`): окна, диалоги, рабочий стол.
/// Состояние, отрисовка, изменение границ и данные делегируются группе.
#[macro_export]
macro_rules! impl_group_basics {
    ($($g:ident).+) => {
        $crate::impl_group_core!($($g).+);
        fn has_data(&self) -> bool {
            self.$($g).+.has_data()
        }
        fn get_data(&self) -> $crate::Value {
            self.$($g).+.get_data()
        }
        fn set_data(&mut self, value: &$crate::Value) {
            self.$($g).+.set_data(value)
        }
    };
}

/// Как [`impl_group_basics!`], но без методов данных (`has_data`/`get_data`/
/// `set_data`) — для групп, у которых данные свои (например, `FileDialog`).
#[macro_export]
macro_rules! impl_group_core {
    ($($g:ident).+) => {
        fn vs(&self) -> &$crate::ViewState {
            &self.$($g).+.vs
        }
        fn vs_mut(&mut self) -> &mut $crate::ViewState {
            &mut self.$($g).+.vs
        }
        fn as_any(&self) -> &dyn ::std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
            self
        }
        fn into_any(self: Box<Self>) -> Box<dyn ::std::any::Any> {
            self
        }
        fn as_group(&self) -> Option<&$crate::Group> {
            Some(&self.$($g).+)
        }
        fn as_group_mut(&mut self) -> Option<&mut $crate::Group> {
            Some(&mut self.$($g).+)
        }
        fn change_bounds(&mut self, bounds: $crate::Rect) {
            self.$($g).+.change_bounds(bounds)
        }
        fn help_ctx(&self) -> u16 {
            self.$($g).+.help_ctx()
        }
    };
}

/// Как [`impl_view_basics!`], но `ViewState` берётся из вложенного поля
/// (`self.$f.vs`). Удобно для представлений-«обёрток»: `Memo` вокруг
/// `Editor`, `FileList` вокруг `ListBox` и т.п. — аналог наследования в C++.
#[macro_export]
macro_rules! impl_view_basics_field {
    ($f:ident) => {
        fn vs(&self) -> &$crate::ViewState {
            &self.$f.vs
        }
        fn vs_mut(&mut self) -> &mut $crate::ViewState {
            &mut self.$f.vs
        }
        fn as_any(&self) -> &dyn ::std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
            self
        }
        fn into_any(self: Box<Self>) -> Box<dyn ::std::any::Any> {
            self
        }
    };
}
