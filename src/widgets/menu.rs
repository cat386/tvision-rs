//! Меню: данные (`TMenu`, `TMenuItem`, `TSubMenu`), строка меню (`TMenuBar`)
//! и выпадающие окна меню (`TMenuBox`, `TMenuPopup`).
//!
//! В C++ открытое меню работает как вложенный модальный цикл
//! (`TMenuView::execute`). Здесь строка меню только просит программу открыть
//! меню (`Request::MenuBar`), а программа ведёт «сеанс меню» —
//! [`MenuSession`]: перехватывает все события, пока меню открыто, и рисует
//! выпадающие окна поверх остального экрана.

use std::rc::Rc;

use crate::color::palettes;
use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Info, Key, KeyEvent};
use crate::geometry::{Point, Rect};
use crate::text::{cstr_width, hotkey, same_hotkey, str_width, to_upper};
use crate::view::{gf, of, CommandSet, Ctx, DrawCtx, Request, View, ViewState};

/// Пункт меню.
#[derive(Clone, Debug)]
pub enum MenuItem {
    /// Обычный пункт: выполняет команду.
    Command {
        /// Текст с горячей буквой: `"~O~pen..."`.
        name: String,
        command: u16,
        /// Клавиша быстрого вызова (работает и при закрытом меню).
        key: Option<KeyEvent>,
        /// Подпись клавиши справа: `"F3"`.
        param: String,
        help_ctx: u16,
    },
    /// Подменю.
    /// `key` — клавиша, открывающая подменю из строки меню (как `kbAltSpace` у меню «≡»).
    SubMenu { name: String, menu: Rc<Menu>, help_ctx: u16, key: Option<KeyEvent> },
    /// Разделитель (`newLine()`).
    Separator,
}

impl MenuItem {
    pub fn name(&self) -> &str {
        match self {
            MenuItem::Command { name, .. } | MenuItem::SubMenu { name, .. } => name,
            MenuItem::Separator => "",
        }
    }

    pub fn is_separator(&self) -> bool {
        matches!(self, MenuItem::Separator)
    }

    /// Доступен ли пункт (команда не запрещена).
    pub fn enabled(&self, cmds: &CommandSet) -> bool {
        match self {
            MenuItem::Command { command, .. } => cmds.is_enabled(*command),
            MenuItem::SubMenu { .. } => true,
            MenuItem::Separator => false,
        }
    }

    pub fn help_ctx(&self) -> u16 {
        match self {
            MenuItem::Command { help_ctx, .. } | MenuItem::SubMenu { help_ctx, .. } => *help_ctx,
            MenuItem::Separator => 0,
        }
    }
}

/// Меню — список пунктов (`TMenu`).
#[derive(Clone, Debug, Default)]
pub struct Menu {
    pub items: Vec<MenuItem>,
}

impl Menu {
    pub fn new() -> Self {
        Menu { items: Vec::new() }
    }

    /// Добавить пункт-команду без клавиши.
    pub fn item(mut self, name: &str, command: u16) -> Self {
        self.items.push(MenuItem::Command { name: name.into(), command, key: None, param: String::new(), help_ctx: 0 });
        self
    }

    /// Добавить пункт-команду с клавишей быстрого вызова и её подписью.
    pub fn item_key(mut self, name: &str, command: u16, key: KeyEvent, param: &str) -> Self {
        self.items.push(MenuItem::Command { name: name.into(), command, key: Some(key), param: param.into(), help_ctx: 0 });
        self
    }

    /// Добавить подменю.
    pub fn submenu(mut self, name: &str, menu: Menu) -> Self {
        self.items.push(MenuItem::SubMenu { name: name.into(), menu: Rc::new(menu), help_ctx: 0, key: None });
        self
    }

    /// Добавить подменю, которое открывается ещё и клавишей `key`.
    pub fn submenu_key(mut self, name: &str, key: KeyEvent, menu: Menu) -> Self {
        self.items.push(MenuItem::SubMenu { name: name.into(), menu: Rc::new(menu), help_ctx: 0, key: Some(key) });
        self
    }

    /// Задать контекст справки последнему добавленному пункту.
    pub fn help(mut self, ctx: u16) -> Self {
        match self.items.last_mut() {
            Some(MenuItem::Command { help_ctx, .. }) | Some(MenuItem::SubMenu { help_ctx, .. }) => *help_ctx = ctx,
            _ => {}
        }
        self
    }

    /// Добавить разделитель.
    pub fn separator(mut self) -> Self {
        self.items.push(MenuItem::Separator);
        self
    }

    /// Поиск команды по клавише быстрого вызова во всём дереве (`findHotKey`).
    pub fn find_hot_key(&self, k: &KeyEvent) -> Option<u16> {
        for it in &self.items {
            match it {
                MenuItem::Command { command, key: Some(key), .. } if k.is(*key) => return Some(*command),
                MenuItem::SubMenu { menu, .. } => {
                    if let Some(c) = menu.find_hot_key(k) {
                        return Some(c);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Пункт с горячей буквой `c` (`findItem`).
    pub fn find_item(&self, c: char) -> Option<usize> {
        // Сначала точное совпадение, затем — с учётом раскладки клавиатуры.
        let c = to_upper(c);
        self.items
            .iter()
            .position(|it| !it.is_separator() && hotkey(it.name()) == Some(c))
            .or_else(|| self.items.iter().position(|it| !it.is_separator() && hotkey(it.name()).map(|h| same_hotkey(c, h)).unwrap_or(false)))
    }

    /// Размер выпадающего окна (`TMenuBox::getRect`).
    pub fn box_size(&self) -> Point {
        let mut w = 10;
        for it in &self.items {
            let l = match it {
                MenuItem::Command { name, param, .. } => cstr_width(name) + 6 + if param.is_empty() { 0 } else { str_width(param) + 2 },
                MenuItem::SubMenu { name, .. } => cstr_width(name) + 6 + 3,
                MenuItem::Separator => 0,
            };
            w = w.max(l);
        }
        Point::new(w as i32, self.items.len() as i32 + 2)
    }

    /// Положения пунктов в строке меню: (x, ширина) каждого пункта.
    pub fn bar_positions(&self) -> Vec<(i32, i32)> {
        let mut x = 1;
        self.items
            .iter()
            .map(|it| {
                let l = cstr_width(it.name()) as i32 + 2;
                let r = (x, l);
                x += l;
                r
            })
            .collect()
    }
}

/// Цвета меню: обычный, выбранный, запрещённый, выбранный запрещённый.
/// Номера цветов как в `TMenuBar::draw` / `TMenuBox::draw`.
fn menu_colors(dc: &DrawCtx) -> [crate::color::AttrPair; 4] {
    [dc.get_color(0x0301), dc.get_color(0x0604), dc.get_color(0x0202), dc.get_color(0x0505)]
}

/// Строка меню (`TMenuBar`).
pub struct MenuBar {
    pub vs: ViewState,
    pub menu: Rc<Menu>,
    /// Подсвеченный пункт (устанавливается программой во время сеанса меню).
    pub active: Option<usize>,
}

impl MenuBar {
    pub fn new(bounds: Rect, menu: Menu) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.grow_mode = gf::GROW_HI_X;
        vs.options |= of::PRE_PROCESS;
        MenuBar { vs, menu: Rc::new(menu), active: None }
    }

    /// Номер пункта строки меню под точкой `x` (локальные координаты).
    pub fn item_at(&self, x: i32) -> Option<usize> {
        self.menu.bar_positions().iter().position(|&(ix, w)| x >= ix && x < ix + w)
    }
}

impl View for MenuBar {
    crate::impl_view_basics!();

    /// Порт `TMenuBar::draw`.
    fn draw(&self, dc: &mut DrawCtx) {
        let [c_normal, c_select, c_norm_dis, c_sel_dis] = menu_colors(dc);
        let w = self.vs.size.x.max(0) as usize;
        let mut b = DrawBuffer::new(w);
        b.move_char(0, ' ', crate::color::lo(c_normal), w);
        for (i, (it, (x, l))) in self.menu.items.iter().zip(self.menu.bar_positions()).enumerate() {
            let (x, l) = (x as usize, l as usize);
            if x + l > w {
                break;
            }
            let enabled = it.enabled(dc.commands);
            let color = match (self.active == Some(i), enabled) {
                (true, true) => c_select,
                (true, false) => c_sel_dis,
                (false, true) => c_normal,
                (false, false) => c_norm_dis,
            };
            b.move_char(x, ' ', crate::color::lo(color), 1);
            b.move_cstr(x + 1, it.name(), color);
            b.move_char(x + l - 1, ' ', crate::color::lo(color), 1);
        }
        dc.write_line(0, 0, self.vs.size.x, 1, &b);
    }

    fn palette(&self) -> &'static [u8] {
        palettes::MENU_VIEW
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        match ev {
            Event::KeyDown(k) => {
                // F10 — войти в меню.
                if k.key == Key::F(10) && k.mods == Default::default() {
                    ctx.request(Request::MenuBar { item: None, mouse: false });
                    ev.clear();
                    return;
                }
                // Клавиша подменю (например, Alt+Пробел для «≡»).
                if let Some(i) = self.menu.items.iter().position(|it| matches!(it, MenuItem::SubMenu { key: Some(key), .. } if k.is(*key))) {
                    ctx.request(Request::MenuBar { item: Some(i), mouse: false });
                    ev.clear();
                    return;
                }
                // Alt+буква — раскрыть соответствующий пункт строки меню.
                if let Some(c) = k.alt_char() {
                    if let Some(i) = self.menu.find_item(c) {
                        ctx.request(Request::MenuBar { item: Some(i), mouse: false });
                        ev.clear();
                        return;
                    }
                }
                // Клавиши быстрого вызова пунктов меню → команда.
                if let Some(c) = self.menu.find_hot_key(k) {
                    if ctx.command_enabled(c) {
                        *ev = Event::command_event(c);
                    }
                }
            }
            Event::Command(m) if m.command == cm::MENU => {
                ctx.request(Request::MenuBar { item: None, mouse: false });
                ev.clear();
            }
            Event::MouseDown(m) => {
                let local = self.vs.make_local(m.pos);
                ctx.request(Request::MenuBar { item: self.item_at(local.x), mouse: true });
                ev.clear();
            }
            _ => {}
        }
    }
}

/// Открытое выпадающее окно меню.
struct Level {
    menu: Rc<Menu>,
    /// Абсолютные границы окна (с рамкой, без тени).
    rect: Rect,
    /// Выбранный пункт.
    sel: Option<usize>,
}

/// Результат обработки события сеансом меню.
pub enum MenuOutcome {
    /// Меню остаётся открытым.
    Continue,
    /// Меню закрыто без выбора.
    Closed,
    /// Выбрана команда — меню закрыто, команду нужно выполнить.
    Command(u16),
}

/// Сеанс работы с открытым меню (замена `TMenuView::execute`).
pub struct MenuSession {
    /// Строка меню: её меню и абсолютная позиция (если сеанс начат из строки меню).
    bar: Option<(Rc<Menu>, Point)>,
    /// Подсвеченный пункт строки меню.
    pub bar_sel: Option<usize>,
    levels: Vec<Level>,
    screen: Point,
}

impl MenuSession {
    /// Сеанс из строки меню. `item` — какой пункт раскрыть сразу.
    pub fn from_bar(menu: Rc<Menu>, bar_origin: Point, item: Option<usize>, screen: Point, cmds: &CommandSet) -> (Self, MenuOutcome) {
        let mut s = MenuSession { bar: Some((menu.clone(), bar_origin)), bar_sel: Some(item.unwrap_or(0)), levels: Vec::new(), screen };
        let out = if item.is_some() { s.open_bar_item(cmds, true) } else { MenuOutcome::Continue };
        (s, out)
    }

    /// Всплывающее меню в абсолютной позиции (`TMenuPopup`).
    pub fn popup(menu: Rc<Menu>, pos: Point, screen: Point) -> Self {
        let mut s = MenuSession { bar: None, bar_sel: None, levels: Vec::new(), screen };
        s.push_level(menu, pos);
        s
    }

    /// Раскрывает выбранный пункт строки меню.
    /// `execute` — если пункт строки является командой, выполнить её.
    fn open_bar_item(&mut self, cmds: &CommandSet, execute: bool) -> MenuOutcome {
        self.levels.clear();
        let (menu, origin) = match &self.bar {
            Some(b) => (b.0.clone(), b.1),
            None => return MenuOutcome::Continue,
        };
        let i = match self.bar_sel {
            Some(i) if i < menu.items.len() => i,
            _ => return MenuOutcome::Continue,
        };
        match &menu.items[i] {
            MenuItem::SubMenu { menu: sub, .. } => {
                let x = menu.bar_positions()[i].0;
                // Как в оригинале: окно строки меню сдвинуто на 1 влево (`r.a.x--`).
                self.push_level(sub.clone(), Point::new(origin.x + x - 1, origin.y + 1));
                MenuOutcome::Continue
            }
            MenuItem::Command { command, .. } if execute && cmds.is_enabled(*command) => MenuOutcome::Command(*command),
            _ => MenuOutcome::Continue,
        }
    }

    /// Открывает выпадающее окно с левым верхним углом в `pos` (с подгонкой под экран).
    fn push_level(&mut self, menu: Rc<Menu>, pos: Point) {
        let size = menu.box_size();
        let mut r = Rect::from_origin_size(pos, size);
        // Подгоняем под экран (как `TMenuView::execute` ограничивает `r`).
        if r.b.x > self.screen.x {
            r = r.moved(self.screen.x - r.b.x, 0);
        }
        if r.b.y > self.screen.y {
            r = r.moved(0, self.screen.y - r.b.y);
        }
        if r.a.x < 0 {
            r = r.moved(-r.a.x, 0);
        }
        if r.a.y < 0 {
            r = r.moved(0, -r.a.y);
        }
        let sel = menu.items.iter().position(|it| !it.is_separator());
        self.levels.push(Level { menu, rect: r, sel });
    }

    /// Контекст справки выбранного пункта (для строки состояния).
    pub fn help_ctx(&self) -> u16 {
        if let Some(l) = self.levels.last() {
            if let Some(s) = l.sel {
                return l.menu.items[s].help_ctx();
            }
        }
        0
    }

    /// Активирует выбранный пункт верхнего окна: открывает подменю или выдаёт команду.
    fn activate(&mut self, cmds: &CommandSet) -> MenuOutcome {
        let (item, rect, sel) = match self.levels.last() {
            Some(l) => match l.sel {
                Some(s) => (l.menu.items[s].clone(), l.rect, s),
                None => return MenuOutcome::Continue,
            },
            None => return self.open_bar_item(cmds, true),
        };
        match item {
            MenuItem::SubMenu { menu, .. } => {
                // Подменю открывается со сдвигом (`getItemRect` + origin).
                self.push_level(menu, Point::new(rect.a.x + 2, rect.a.y + sel as i32 + 2));
                MenuOutcome::Continue
            }
            MenuItem::Command { command, .. } => {
                if cmds.is_enabled(command) {
                    MenuOutcome::Command(command)
                } else {
                    MenuOutcome::Continue
                }
            }
            MenuItem::Separator => MenuOutcome::Continue,
        }
    }

    /// Сдвигает выделение в верхнем окне на `dir` (±1), пропуская разделители.
    fn move_sel(&mut self, dir: i32, to_end: Option<bool>) {
        if let Some(l) = self.levels.last_mut() {
            let n = l.menu.items.len() as i32;
            if n == 0 {
                return;
            }
            let mut i = match to_end {
                Some(true) => n,
                Some(false) => -1,
                None => l.sel.map(|s| s as i32).unwrap_or(-1),
            };
            for _ in 0..n {
                i = (i + dir).rem_euclid(n);
                if !l.menu.items[i as usize].is_separator() {
                    l.sel = Some(i as usize);
                    return;
                }
            }
        }
    }

    /// Сдвигает выделение в строке меню и раскрывает пункт.
    fn move_bar(&mut self, dir: i32, cmds: &CommandSet) {
        let n = match &self.bar {
            Some(b) => b.0.items.len() as i32,
            None => return,
        };
        if n == 0 {
            return;
        }
        let i = self.bar_sel.map(|s| s as i32).unwrap_or(0);
        self.bar_sel = Some((i + dir).rem_euclid(n) as usize);
        let was_open = !self.levels.is_empty();
        self.levels.clear();
        if was_open {
            self.open_bar_item(cmds, false);
        }
    }

    /// Обработка события. Вызывается программой вместо обычной доставки,
    /// пока меню открыто.
    pub fn handle(&mut self, ev: &Event, cmds: &CommandSet) -> MenuOutcome {
        match ev {
            Event::KeyDown(k) => self.handle_key(*k, cmds),
            Event::MouseDown(m) | Event::MouseMove(m) | Event::MouseAuto(m) => {
                let pressed = matches!(ev, Event::MouseDown(_)) || m.buttons != 0;
                if !pressed {
                    return MenuOutcome::Continue;
                }
                self.track_mouse(m.pos, cmds, matches!(ev, Event::MouseDown(_)))
            }
            Event::MouseUp(m) => self.release_mouse(m.pos, cmds),
            _ => MenuOutcome::Continue,
        }
    }

    fn handle_key(&mut self, k: KeyEvent, cmds: &CommandSet) -> MenuOutcome {
        let plain = k.mods == Default::default();
        match k.key {
            Key::F(10) if plain => MenuOutcome::Closed,
            Key::Esc => {
                // Esc закрывает верхнее окно. Если это было окно строки меню —
                // остаёмся в строке меню (как в оригинале); второй Esc — выход.
                if self.levels.pop().is_some() {
                    if self.levels.is_empty() && self.bar.is_none() {
                        MenuOutcome::Closed
                    } else {
                        MenuOutcome::Continue
                    }
                } else {
                    MenuOutcome::Closed
                }
            }
            Key::Up | Key::Down => {
                if self.levels.is_empty() {
                    // В строке меню стрелка вниз раскрывает пункт.
                    if k.key == Key::Down {
                        return self.open_bar_item(cmds, false);
                    }
                    return MenuOutcome::Continue;
                }
                self.move_sel(if k.key == Key::Down { 1 } else { -1 }, None);
                MenuOutcome::Continue
            }
            Key::Home | Key::End => {
                self.move_sel(if k.key == Key::End { -1 } else { 1 }, Some(k.key == Key::End));
                MenuOutcome::Continue
            }
            Key::Left | Key::Right => {
                let right = k.key == Key::Right;
                // Вложенное подменю: влево — закрыть, вправо — открыть.
                if self.levels.len() > 1 && !right {
                    self.levels.pop();
                    return MenuOutcome::Continue;
                }
                if right {
                    if let Some(l) = self.levels.last() {
                        if let Some(s) = l.sel {
                            if matches!(l.menu.items[s], MenuItem::SubMenu { .. }) {
                                return self.activate(cmds);
                            }
                        }
                    }
                }
                self.move_bar(if right { 1 } else { -1 }, cmds);
                MenuOutcome::Continue
            }
            Key::Enter => self.activate(cmds),
            Key::Char(c) if !k.mods.ctrl => {
                // Горячая буква пункта (с Alt или без).
                if let Some(l) = self.levels.last_mut() {
                    if let Some(i) = l.menu.find_item(c) {
                        l.sel = Some(i);
                        return self.activate(cmds);
                    }
                } else if let Some((menu, _)) = &self.bar {
                    if let Some(i) = menu.find_item(c) {
                        self.bar_sel = Some(i);
                        return self.open_bar_item(cmds, true);
                    }
                }
                // Alt+буква пункта строки меню при открытом окне.
                if k.mods.alt {
                    if let Some((menu, _)) = &self.bar {
                        if let Some(i) = menu.find_item(c) {
                            self.bar_sel = Some(i);
                            return self.open_bar_item(cmds, true);
                        }
                    }
                }
                MenuOutcome::Continue
            }
            _ => {
                // Клавиша быстрого вызова работает и при открытом меню.
                if let Some((menu, _)) = &self.bar {
                    if let Some(c) = menu.find_hot_key(&k) {
                        if cmds.is_enabled(c) {
                            return MenuOutcome::Command(c);
                        }
                    }
                }
                MenuOutcome::Continue
            }
        }
    }

    /// Что находится под мышью: (номер окна, номер пункта) или пункт строки меню.
    fn hit(&self, p: Point) -> Hit {
        for (li, l) in self.levels.iter().enumerate().rev() {
            if l.rect.contains(p) {
                let row = p.y - l.rect.a.y - 1;
                if row >= 0 && (row as usize) < l.menu.items.len() && p.x > l.rect.a.x && p.x < l.rect.b.x - 1 {
                    return Hit::Item(li, row as usize);
                }
                return Hit::Box;
            }
        }
        if let Some((menu, origin)) = &self.bar {
            if p.y == origin.y {
                let x = p.x - origin.x;
                if let Some(i) = menu.bar_positions().iter().position(|&(ix, w)| x >= ix && x < ix + w) {
                    return Hit::Bar(i);
                }
                return Hit::BarEmpty;
            }
        }
        Hit::Nothing
    }

    fn track_mouse(&mut self, p: Point, cmds: &CommandSet, is_down: bool) -> MenuOutcome {
        match self.hit(p) {
            Hit::Bar(i) => {
                if self.bar_sel != Some(i) || self.levels.is_empty() {
                    self.bar_sel = Some(i);
                    self.open_bar_item(cmds, false);
                }
                MenuOutcome::Continue
            }
            Hit::Item(li, row) => {
                self.levels.truncate(li + 1);
                let l = &mut self.levels[li];
                if !l.menu.items[row].is_separator() {
                    l.sel = Some(row);
                    // Наведение на подменю с нажатой кнопкой раскрывает его.
                    if matches!(l.menu.items[row], MenuItem::SubMenu { .. }) {
                        return self.activate(cmds);
                    }
                }
                MenuOutcome::Continue
            }
            Hit::Box | Hit::BarEmpty => MenuOutcome::Continue,
            Hit::Nothing => {
                if is_down {
                    MenuOutcome::Closed
                } else {
                    MenuOutcome::Continue
                }
            }
        }
    }

    fn release_mouse(&mut self, p: Point, cmds: &CommandSet) -> MenuOutcome {
        match self.hit(p) {
            Hit::Item(li, row) => {
                self.levels.truncate(li + 1);
                let l = &mut self.levels[li];
                if l.menu.items[row].is_separator() {
                    return MenuOutcome::Continue;
                }
                l.sel = Some(row);
                self.activate(cmds)
            }
            Hit::Bar(i) => {
                // Отпустили на пункте строки меню: если это команда — выполнить.
                self.bar_sel = Some(i);
                if self.levels.is_empty() {
                    return self.open_bar_item(cmds, true);
                }
                MenuOutcome::Continue
            }
            Hit::Box | Hit::BarEmpty => MenuOutcome::Continue,
            Hit::Nothing => MenuOutcome::Closed,
        }
    }

    /// Рисует открытые окна меню поверх экрана.
    /// `dc` — корневой контекст (палитра приложения).
    pub fn draw(&self, dc: &mut DrawCtx) {
        for l in &self.levels {
            {
                let mut sub = dc.sub(l.rect, palettes::MENU_VIEW);
                draw_menu_box(&mut sub, &l.menu, l.sel);
            }
            dc.shadow(l.rect);
        }
    }
}

enum Hit {
    Item(usize, usize),
    Box,
    Bar(usize),
    BarEmpty,
    Nothing,
}

/// Порт `TMenuBox::draw`: рамка из `TMenuBox::frameChars` и пункты.
pub fn draw_menu_box(dc: &mut DrawCtx, menu: &Menu, sel: Option<usize>) {
    let [c_normal, c_select, c_norm_dis, c_sel_dis] = menu_colors(dc);
    let size = dc.size;
    let w = size.x.max(4) as usize;
    let normal = crate::color::lo(c_normal);

    // frameLine(n): 2 символа слева, заполнитель, 2 символа справа.
    let frame_line = |b: &mut DrawBuffer, left: &str, fill: char, right: &str, color: crate::color::Attr| {
        b.move_str(0, left, normal);
        b.move_char(2, fill, color, w - 4);
        b.move_str(w - 2, right, normal);
    };

    let mut b = DrawBuffer::new(w);
    frame_line(&mut b, " ┌", '─', "┐ ", normal);
    dc.write_line(0, 0, size.x, 1, &b);

    for (y, it) in menu.items.iter().enumerate() {
        let mut b = DrawBuffer::new(w);
        match it {
            MenuItem::Separator => frame_line(&mut b, " ├", '─', "┤ ", normal),
            _ => {
                let enabled = it.enabled(dc.commands);
                let color = match (sel == Some(y), enabled) {
                    (true, true) => c_select,
                    (true, false) => c_sel_dis,
                    (false, true) => c_normal,
                    (false, false) => c_norm_dis,
                };
                frame_line(&mut b, " │", ' ', "│ ", crate::color::lo(color));
                b.move_cstr(3, it.name(), color);
                match it {
                    MenuItem::SubMenu { .. } => b.put_char(w - 4, '►'),
                    MenuItem::Command { param, .. } if !param.is_empty() => {
                        let pw = str_width(param);
                        b.move_str(w.saturating_sub(3 + pw), param, crate::color::lo(color));
                    }
                    _ => {}
                }
            }
        }
        dc.write_line(0, y as i32 + 1, size.x, 1, &b);
    }

    let mut b = DrawBuffer::new(w);
    frame_line(&mut b, " └", '─', "┘ ", normal);
    dc.write_line(0, size.y - 1, size.x, 1, &b);
}

/// Отправка команды из меню (для удобства программы).
pub(crate) fn command_event(cmd: u16) -> Event {
    Event::Command(crate::event::Message::with(cmd, Info::None))
}
