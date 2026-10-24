//! Диалог настройки цветов (`TColorDialog` и его части из `colorsel.cpp`):
//! `TColorSelector` (сетка цветов), `TColorDisplay` (образец текста),
//! `TColorGroupList` / `TColorItemList` (списки групп и элементов),
//! `TColorGroup` / `TColorItem` (описание палитры).
//!
//! ```ignore
//! let d = ColorDialog::new(&app.palette, standard_color_groups());
//! let (cmd, d) = app.exec_dialog(d);
//! if cmd == cm::OK { app.palette = d.palette.clone(); }
//! ```
//!
//! Кроме сеток 16 цветов BIOS, в диалоге есть кнопки «Текст 256…» и
//! «Фон 256…»: они открывают [`Color256Dialog`] — выбор любого из 256
//! цветов палитры xterm (см. [`crate::color`]).
//!
//! В C++ части диалога общались широковещательными сообщениями
//! (`cmNewColorItem`, `cmNewColorIndex`, `cmColorSet`, `cmColorForegroundChanged`...).
//! Здесь диалог после каждого события сам сверяет состояние частей и
//! передаёт изменения — результат тот же, а связей меньше.

use crate::color::{attr256, back, bg256, contrast256, fg256, fore, with_back, with_bg256, with_fg256, with_fore, xterm_rgb, Attr, ERROR_ATTR};
use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Key, Mods};
use crate::geometry::Rect;
use crate::locale::tr;
use crate::view::{default_handle_event, of, Ctx, DrawCtx, Value, View, ViewId, ViewState};
use crate::widgets::button::{bf, Button};
use crate::widgets::dialog::Dialog;
use crate::widgets::listbox::ListBox;
use crate::widgets::scrollbar::ScrollBar;
use crate::widgets::statictext::Label;

/// Элемент палитры: название и номер цвета в палитре приложения (`TColorItem`).
#[derive(Clone, Debug, PartialEq)]
pub struct ColorItem {
    pub name: String,
    pub index: u8,
}

/// Группа элементов (`TColorGroup`).
#[derive(Clone, Debug, PartialEq)]
pub struct ColorGroup {
    pub name: String,
    pub items: Vec<ColorItem>,
    /// Последний выбранный элемент группы (`index`).
    pub index: usize,
}

impl ColorGroup {
    pub fn new(name: &str, items: &[(&str, u8)]) -> Self {
        ColorGroup { name: name.into(), items: items.iter().map(|(n, i)| ColorItem { name: n.to_string(), index: *i }).collect(), index: 0 }
    }
}

/// Группы для стандартных элементов (по образцу `tvdemo`).
pub fn standard_color_groups() -> Vec<ColorGroup> {
    let window = |base: u8| -> Vec<(&'static str, u8)> {
        vec![
            ("Frame passive", base),
            ("Frame active", base + 1),
            ("Frame icons", base + 2),
            ("Scroll bar page", base + 3),
            ("Scroll bar icons", base + 4),
            ("Normal text", base + 5),
            ("Selected text", base + 6),
        ]
    };
    vec![
        ColorGroup::new("Desktop", &[("Color", 1)]),
        ColorGroup::new(
            "Menus",
            &[("Normal", 2), ("Disabled", 3), ("Shortcut", 4), ("Selected", 5), ("Selected disabled", 6), ("Shortcut selected", 7)],
        ),
        ColorGroup::new("Blue window", &window(8)),
        ColorGroup::new("Cyan window", &window(16)),
        ColorGroup::new("Gray window", &window(24)),
        ColorGroup::new(
            "Dialogs",
            &[
                ("Frame/background", 33),
                ("Frame icons", 34),
                ("Scroll bar page", 35),
                ("Scroll bar icons", 36),
                ("Static text", 37),
                ("Label normal", 38),
                ("Label selected", 39),
                ("Label shortcut", 40),
                ("Button normal", 41),
                ("Button default", 42),
                ("Button selected", 43),
                ("Button disabled", 44),
                ("Button shortcut", 45),
                ("Button shadow", 46),
                ("Cluster normal", 47),
                ("Cluster selected", 48),
                ("Cluster shortcut", 49),
                ("Input normal", 50),
                ("Input selected", 51),
                ("Input arrow", 52),
                ("History button", 53),
                ("History sides", 54),
                ("History bar page", 55),
                ("History bar icons", 56),
                ("List normal", 57),
                ("List focused", 58),
                ("List selected", 59),
                ("List divider", 60),
                ("Information pane", 61),
            ],
        ),
    ]
}

/// Что выбирает сетка: цвет текста (16 цветов) или фона (8 цветов).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorSel {
    Background,
    Foreground,
}

/// Сетка цветов 4×4 (`TColorSelector`).
pub struct ColorSelector {
    pub vs: ViewState,
    pub sel_type: ColorSel,
    pub color: u8,
}

/// Значок клетки цвета (`TColorSelector::icon`).
const ICON: char = '█';

impl ColorSelector {
    pub fn new(bounds: Rect, sel_type: ColorSel) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.options |= of::SELECTABLE | of::FIRST_CLICK | of::FRAMED;
        ColorSelector { vs, sel_type, color: 0 }
    }

    fn max_col(&self) -> u8 {
        if self.sel_type == ColorSel::Background {
            7
        } else {
            15
        }
    }
}

impl View for ColorSelector {
    crate::impl_view_basics!();

    /// Порт `TColorSelector::draw`: цвета рисуются «как есть», без палитры.
    fn draw(&self, dc: &mut DrawCtx) {
        let w = self.vs.size.x.max(0) as usize;
        for i in 0..self.vs.size.y {
            let mut b = DrawBuffer::new(w);
            b.move_char(0, ' ', 0x70, w);
            if i < 4 {
                for j in 0..4 {
                    let c = (i * 4 + j) as u8;
                    if c > self.max_col() {
                        continue;
                    }
                    // Цвет 0 (чёрный) — это «не менять атрибут» в DrawBuffer,
                    // поэтому для него рисуем чёрный текст на сером фоне.
                    b.move_char((j * 3) as usize, ICON, if c == 0 { 0x70 } else { c as Attr }, 3);
                    if c == self.color {
                        b.put_char((j * 3 + 1) as usize, '◘');
                        if c == 0 {
                            b.put_attribute((j * 3 + 1) as usize, 0x70);
                        }
                    }
                }
            }
            dc.write_line(0, i, self.vs.size.x, 1, &b);
        }
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        default_handle_event(&self.vs, ev, ctx);
        let max = self.max_col();
        const W: u8 = 4;
        match ev {
            Event::MouseDown(_) | Event::MouseMove(_) if matches!(ev, Event::MouseDown(_)) || ctx.mouse_grabbed_by(self.vs.id) => {
                let p = self.vs.make_local(ev.mouse().unwrap().pos);
                if self.vs.extent().contains(p) {
                    let c = (p.y * 4 + p.x / 3) as u8;
                    if c <= max {
                        self.color = c;
                    }
                }
                if matches!(ev, Event::MouseDown(_)) {
                    ctx.grab_mouse(self.vs.id);
                }
                ev.clear();
            }
            Event::MouseUp(_) if ctx.mouse_grabbed_by(self.vs.id) => {
                ctx.release_mouse();
                ev.clear();
            }
            Event::KeyDown(k) if self.vs.is_focused() && k.mods == Mods::NONE => {
                let c = self.color;
                self.color = match k.key {
                    Key::Left => {
                        if c > 0 {
                            c - 1
                        } else {
                            max
                        }
                    }
                    Key::Right => {
                        if c < max {
                            c + 1
                        } else {
                            0
                        }
                    }
                    Key::Up => {
                        if c > W - 1 {
                            c - W
                        } else if c == 0 {
                            max
                        } else {
                            c + max - W
                        }
                    }
                    Key::Down => {
                        if c < max - (W - 1) {
                            c + W
                        } else if c == max {
                            0
                        } else {
                            c - (max - W)
                        }
                    }
                    _ => return,
                };
                ev.clear();
            }
            _ => {}
        }
    }
}

/// Образец текста выбранным цветом (`TColorDisplay`).
pub struct ColorDisplay {
    pub vs: ViewState,
    pub text: String,
    pub color: Attr,
}

impl ColorDisplay {
    pub fn new(bounds: Rect, text: &str) -> Self {
        ColorDisplay { vs: ViewState::new(bounds), text: text.into(), color: 0 }
    }
}

impl View for ColorDisplay {
    crate::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let c = if self.color == 0 { ERROR_ATTR } else { self.color };
        let w = self.vs.size.x.max(0) as usize;
        let len = crate::text::str_width(&self.text).max(1);
        let mut b = DrawBuffer::new(w);
        let mut x = 0;
        while x < w {
            b.move_str(x, &self.text, c);
            x += len;
        }
        dc.write_line(0, 0, self.vs.size.x, self.vs.size.y, &b);
    }
}

/// Тексты диалога (статические поля `TColorDialog`).
pub struct ColorDialogText;
impl ColorDialogText {
    pub const COLORS: &'static str = "Colors";
    pub const GROUP: &'static str = "~G~roup";
    pub const ITEM: &'static str = "~I~tem";
    pub const FOREGROUND: &'static str = "~F~oreground";
    pub const BACKGROUND: &'static str = "~B~ackground";
    pub const TEXT: &'static str = "Text ";
    pub const OK: &'static str = "O~K~";
    pub const CANCEL: &'static str = "Cancel";
    /// Кнопка выбора 256-цветного текста.
    pub const FORE_256: &'static str = "Te~x~t 256...";
    /// Кнопка выбора 256-цветного фона.
    pub const BACK_256: &'static str = "B~a~ck 256...";
    /// Заголовок окна выбора цвета текста из 256.
    pub const PICK_FORE: &'static str = "Text color (256)";
    /// Заголовок окна выбора цвета фона из 256.
    pub const PICK_BACK: &'static str = "Background color (256)";
    /// Подпись сетки 256 цветов.
    pub const PICK_LABEL: &'static str = "~C~olor";
    /// Подпись номера цвета: «Color 123  #87AFFF».
    pub const NUMBER: &'static str = "Color";
    /// Образец текста в окне выбора.
    pub const SAMPLE: &'static str = " Sample text ";
}

/// Внутренние команды кнопок «256…» (номера из служебного диапазона).
const CM_FORE_256: u16 = 0xFF20;
const CM_BACK_256: u16 = 0xFF21;

// ----------------------------------------------------------------------
// Выбор из 256 цветов
// ----------------------------------------------------------------------

/// Сетка всех 256 цветов xterm: 32 столбца × 8 строк, клетка — 2 символа
/// (номер цвета = строка · 32 + столбец). Выбранный цвет помечен `◄►`.
///
/// Клавиши: стрелки, Home/End (начало/конец строки), PgUp/PgDn (первая/
/// последняя строка). Мышь: щелчок или протяжка.
pub struct Color256Selector {
    pub vs: ViewState,
    pub color: u8,
}

impl Color256Selector {
    /// Столбцов в сетке.
    pub const COLS: i32 = 32;
    /// Строк в сетке.
    pub const ROWS: i32 = 8;

    /// Сетка 64×8 с левым верхним углом в `(x, y)`.
    pub fn new(x: i32, y: i32, color: u8) -> Self {
        let mut vs = ViewState::new(Rect::new(x, y, x + Self::COLS * 2, y + Self::ROWS));
        vs.options |= of::SELECTABLE | of::FIRST_CLICK;
        Color256Selector { vs, color }
    }
}

impl View for Color256Selector {
    crate::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let w = self.vs.size.x.max(0) as usize;
        for row in 0..self.vs.size.y {
            let mut b = DrawBuffer::new(w);
            for col in 0..Self::COLS {
                let i = row * Self::COLS + col;
                if i > 255 {
                    break;
                }
                let idx = i as u8;
                let a = attr256(contrast256(idx), idx);
                let mark = if idx == self.color { "◄►" } else { "  " };
                b.move_str((col * 2) as usize, mark, a);
            }
            dc.write_line(0, row, self.vs.size.x, 1, &b);
        }
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        default_handle_event(&self.vs, ev, ctx);
        let id = self.vs.id;
        match ev {
            Event::MouseDown(_) | Event::MouseMove(_) if matches!(ev, Event::MouseDown(_)) || ctx.mouse_grabbed_by(id) => {
                let p = self.vs.make_local(ev.mouse().unwrap().pos);
                if self.vs.extent().contains(p) {
                    self.color = (p.y * Self::COLS + p.x / 2).clamp(0, 255) as u8;
                }
                if matches!(ev, Event::MouseDown(_)) {
                    ctx.grab_mouse(id);
                }
                ev.clear();
            }
            Event::MouseUp(_) if ctx.mouse_grabbed_by(id) => {
                ctx.release_mouse();
                ev.clear();
            }
            Event::KeyDown(k) if self.vs.is_focused() && k.mods == Mods::NONE => {
                let c = self.color as i32;
                let (row, col) = (c / Self::COLS, c % Self::COLS);
                let n = match k.key {
                    Key::Left => (c + 255) % 256,
                    Key::Right => (c + 1) % 256,
                    Key::Up => ((row + Self::ROWS - 1) % Self::ROWS) * Self::COLS + col,
                    Key::Down => ((row + 1) % Self::ROWS) * Self::COLS + col,
                    Key::Home => row * Self::COLS,
                    Key::End => row * Self::COLS + Self::COLS - 1,
                    Key::PgUp => col,
                    Key::PgDn => (Self::ROWS - 1) * Self::COLS + col,
                    _ => return,
                };
                self.color = n as u8;
                ev.clear();
            }
            _ => {}
        }
    }

    fn has_data(&self) -> bool {
        true
    }

    fn get_data(&self) -> Value {
        Value::U32(self.color as u32)
    }

    fn set_data(&mut self, value: &Value) {
        if let Some(v) = value.as_u32() {
            self.color = v.min(255) as u8;
        }
    }
}

/// Номер и образец выбранного цвета в [`Color256Dialog`].
struct Color256Info {
    vs: ViewState,
    /// Атрибут образца.
    sample: Attr,
    /// Номер цвета.
    index: u8,
}

impl View for Color256Info {
    crate::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let w = self.vs.size.x.max(0) as usize;
        let normal = dc.color(1);
        let mut b = DrawBuffer::new(w);
        b.move_char(0, ' ', normal, w);
        let info = format!("{} {:3}  #{:06X}", tr(ColorDialogText::NUMBER), self.index, xterm_rgb(self.index));
        let x = b.move_str(0, &info, normal);
        b.move_str(x + 3, tr(ColorDialogText::SAMPLE), self.sample);
        dc.write_line(0, 0, self.vs.size.x, 1, &b);
    }

    fn palette(&self) -> &'static [u8] {
        crate::color::palettes::STATIC_TEXT
    }
}

/// Окно выбора одного из 256 цветов (для текста или фона).
///
/// ```ignore
/// let d = Color256Dialog::new(ColorSel::Foreground, 0x1E);
/// let (cmd, d) = app.exec_dialog(d);
/// if cmd == cm::OK { let a = d.result(); }
/// ```
pub struct Color256Dialog {
    pub dlg: Dialog,
    /// Что выбирается: текст или фон.
    pub sel_type: ColorSel,
    /// Исходный атрибут: образец показывает его с новым цветом.
    pub base: Attr,
    selector: ViewId,
    info: ViewId,
}

impl Color256Dialog {
    /// Окно выбора для атрибута `base`; начальный цвет — текущий цвет
    /// текста или фона `base`.
    pub fn new(sel_type: ColorSel, base: Attr) -> Self {
        let title = if sel_type == ColorSel::Foreground { ColorDialogText::PICK_FORE } else { ColorDialogText::PICK_BACK };
        let mut dlg = Dialog::centered(68, 17, tr(title));
        let start = if sel_type == ColorSel::Foreground { fg256(base) } else { bg256(base) };
        let selector = dlg.add(Color256Selector::new(2, 3, start));
        dlg.add(Label::new(Rect::new(2, 2, 20, 3), tr(ColorDialogText::PICK_LABEL), Some(selector)));
        let info = dlg.add(Color256Info { vs: ViewState::new(Rect::new(2, 12, 66, 13)), sample: base, index: start });
        dlg.add(Button::new(Rect::new(20, 14, 32, 16), tr(ColorDialogText::OK), cm::OK, bf::DEFAULT));
        dlg.add(Button::new(Rect::new(35, 14, 47, 16), tr(ColorDialogText::CANCEL), cm::CANCEL, bf::NORMAL));
        dlg.select(selector);
        let mut d = Color256Dialog { dlg, sel_type, base, selector, info };
        d.sync();
        d
    }

    /// Выбранный номер цвета (0–255).
    pub fn color(&self) -> u8 {
        self.dlg.win.group.find::<Color256Selector>(self.selector).map(|s| s.color).unwrap_or(0)
    }

    /// Исходный атрибут с выбранным цветом текста или фона.
    pub fn result(&self) -> Attr {
        match self.sel_type {
            ColorSel::Foreground => with_fg256(self.base, self.color()),
            ColorSel::Background => with_bg256(self.base, self.color()),
        }
    }

    fn sync(&mut self) {
        let (index, sample) = (self.color(), self.result());
        if let Some(i) = self.dlg.win.group.find_mut::<Color256Info>(self.info) {
            i.index = index;
            i.sample = sample;
        }
    }
}

impl View for Color256Dialog {
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

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        self.dlg.handle_event(ev, ctx);
        self.sync();
    }

    fn valid(&mut self, command: u16, ctx: &mut Ctx) -> bool {
        self.dlg.valid(command, ctx)
    }

    fn has_data(&self) -> bool {
        true
    }

    /// Данные — номер выбранного цвета (`Value::U32`).
    fn get_data(&self) -> Value {
        Value::U32(self.color() as u32)
    }

    fn set_data(&mut self, value: &Value) {
        if let Some(s) = self.dlg.win.group.find_mut::<Color256Selector>(self.selector) {
            s.set_data(value);
        }
        self.sync();
    }
}

/// Диалог настройки цветов (`TColorDialog`).
pub struct ColorDialog {
    pub dlg: Dialog,
    /// Редактируемая копия палитры приложения (атрибуты могут быть
    /// 256-цветными).
    pub palette: Vec<Attr>,
    pub groups: Vec<ColorGroup>,
    group_list: ViewId,
    item_list: ViewId,
    fore_sel: ViewId,
    back_sel: ViewId,
    display: ViewId,
    /// Что последний раз показывалось: (группа, элемент, атрибут).
    shown: Option<(usize, usize, Attr)>,
    /// Значения сеток при последней сверке: (текст, фон). Палитра меняется
    /// из сеток, только когда пользователь их действительно изменил, —
    /// иначе 256-цветный атрибут затёрся бы его 16-цветным приближением.
    sel_shown: (u8, u8),
}

impl ColorDialog {
    /// Аналог `TColorDialog(palette, groups)`.
    pub fn new(palette: &[Attr], groups: Vec<ColorGroup>) -> Self {
        let mut dlg = Dialog::centered(61, 18, tr(ColorDialogText::COLORS));
        let sb = dlg.add(ScrollBar::new(Rect::new(18, 3, 19, 14)));
        let mut gl = ListBox::new(Rect::new(3, 3, 18, 14), 1, Some(sb));
        gl.set_items(groups.iter().map(|g| g.name.clone()).collect());
        let group_list = dlg.add(gl);
        dlg.add(Label::new(Rect::new(2, 2, 9, 3), tr(ColorDialogText::GROUP), Some(group_list)));

        let sb = dlg.add(ScrollBar::new(Rect::new(41, 3, 42, 14)));
        let item_list = dlg.add(ListBox::new(Rect::new(21, 3, 41, 14), 1, Some(sb)));
        dlg.add(Label::new(Rect::new(20, 2, 28, 3), tr(ColorDialogText::ITEM), Some(item_list)));

        let fore_sel = dlg.add(ColorSelector::new(Rect::new(45, 3, 57, 7), ColorSel::Foreground));
        dlg.add(Label::new(Rect::new(45, 2, 57, 3), tr(ColorDialogText::FOREGROUND), Some(fore_sel)));
        let back_sel = dlg.add(ColorSelector::new(Rect::new(45, 9, 57, 11), ColorSel::Background));
        dlg.add(Label::new(Rect::new(45, 8, 57, 9), tr(ColorDialogText::BACKGROUND), Some(back_sel)));
        let display = dlg.add(ColorDisplay::new(Rect::new(44, 12, 58, 14), tr(ColorDialogText::TEXT)));

        dlg.add(Button::new(Rect::new(2, 15, 18, 17), tr(ColorDialogText::FORE_256), CM_FORE_256, bf::NORMAL));
        dlg.add(Button::new(Rect::new(19, 15, 35, 17), tr(ColorDialogText::BACK_256), CM_BACK_256, bf::NORMAL));
        dlg.add(Button::new(Rect::new(36, 15, 46, 17), tr(ColorDialogText::OK), cm::OK, bf::DEFAULT));
        dlg.add(Button::new(Rect::new(48, 15, 58, 17), tr(ColorDialogText::CANCEL), cm::CANCEL, bf::NORMAL));
        dlg.select(group_list);

        let mut d = ColorDialog {
            dlg,
            palette: palette.to_vec(),
            groups,
            group_list,
            item_list,
            fore_sel,
            back_sel,
            display,
            shown: None,
            sel_shown: (0, 0),
        };
        d.sync();
        d
    }

    fn focused(&self, id: ViewId) -> usize {
        self.dlg.win.group.find::<ListBox>(id).map(|l| l.focused).unwrap_or(0)
    }

    /// Номер цвета в палитре для текущей группы и элемента.
    fn current_index(&self) -> Option<usize> {
        let g = self.groups.get(self.focused(self.group_list))?;
        let it = g.items.get(self.focused(self.item_list))?;
        Some(it.index as usize)
    }

    /// Согласовать части диалога (замена сообщений `cmNewColorItem` и др.).
    fn sync(&mut self) {
        let gi = self.focused(self.group_list);
        let mut ii = self.focused(self.item_list);
        let group_changed = self.shown.map(|s| s.0) != Some(gi);
        if group_changed {
            // Новая группа: список элементов и запомненный в группе элемент.
            if let Some(g) = self.groups.get(gi) {
                let names: Vec<String> = g.items.iter().map(|i| i.name.clone()).collect();
                let idx = g.index;
                if let Some(l) = self.dlg.win.group.find_mut::<ListBox>(self.item_list) {
                    l.set_items(names);
                    l.focus_item(idx);
                    ii = l.focused;
                }
            }
        } else if let Some(g) = self.groups.get_mut(gi) {
            g.index = ii; // `cmSaveColorIndex`
        }
        let pi = match self.current_index() {
            Some(i) if i >= 1 && i <= self.palette.len() => i - 1,
            _ => return,
        };
        let fg = self.dlg.win.group.find::<ColorSelector>(self.fore_sel).map(|s| s.color).unwrap_or(0);
        let bg = self.dlg.win.group.find::<ColorSelector>(self.back_sel).map(|s| s.color).unwrap_or(0);
        let item_changed = group_changed || self.shown.map(|s| s.1) != Some(ii) || self.shown.map(|s| s.2) != Some(self.palette[pi]);
        if item_changed {
            // Новый элемент (или цвет поменяли окном «256…»): показать его
            // цвет в сетках (`cmColorSet`). Для 256-цветного атрибута сетки
            // показывают ближайшие цвета BIOS.
            self.show_in_selectors(self.palette[pi]);
        } else {
            // Изменили цвет в сетке — записать в палитру только изменённую
            // часть: второй цвет (может быть 256-цветным) остаётся как был.
            let mut a = self.palette[pi];
            if fg != self.sel_shown.0 {
                a = with_fore(a, fg);
            }
            if bg != self.sel_shown.1 {
                a = with_back(a, bg);
            }
            self.palette[pi] = a;
            self.sel_shown = (fg, bg);
        }
        let a = self.palette[pi];
        if let Some(d) = self.dlg.win.group.find_mut::<ColorDisplay>(self.display) {
            d.color = a;
        }
        self.shown = Some((gi, ii, a));
    }

    /// Показать атрибут `a` в сетках 16 цветов.
    fn show_in_selectors(&mut self, a: Attr) {
        let (fg, bg) = (fore(a), back(a) & 7);
        if let Some(s) = self.dlg.win.group.find_mut::<ColorSelector>(self.fore_sel) {
            s.color = fg;
        }
        if let Some(s) = self.dlg.win.group.find_mut::<ColorSelector>(self.back_sel) {
            s.color = bg;
        }
        self.sel_shown = (fg, bg);
    }

    /// Атрибут текущего элемента (если элемент выбран).
    pub fn current_attr(&self) -> Option<Attr> {
        let i = self.current_index()?;
        self.palette.get(i.checked_sub(1)?).copied()
    }

    /// Задать атрибут текущего элемента (например, после [`Color256Dialog`]).
    pub fn set_current_attr(&mut self, a: Attr) {
        if let Some(i) = self.current_index() {
            if i >= 1 && i <= self.palette.len() {
                self.palette[i - 1] = a;
                self.sync();
            }
        }
    }
}

impl View for ColorDialog {
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

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        // Кнопки «Текст 256…» / «Фон 256…» открывают окно выбора цвета;
        // ответ придёт в `exec_result`.
        if let Some(c @ (CM_FORE_256 | CM_BACK_256)) = ev.command() {
            if let Some(a) = self.current_attr() {
                let sel = if c == CM_FORE_256 { ColorSel::Foreground } else { ColorSel::Background };
                ctx.exec(Box::new(Color256Dialog::new(sel, a)), Some(self.dlg.win.group.vs.id), c);
            }
            ev.clear();
            return;
        }
        self.dlg.handle_event(ev, ctx);
        self.sync();
    }

    fn exec_result(&mut self, _tag: u16, command: u16, view: Box<dyn View>, _ctx: &mut Ctx) {
        if command != cm::OK {
            return;
        }
        if let Ok(d) = view.into_any().downcast::<Color256Dialog>() {
            self.set_current_attr(d.result());
        }
    }

    fn valid(&mut self, command: u16, ctx: &mut Ctx) -> bool {
        self.dlg.valid(command, ctx)
    }

    fn has_data(&self) -> bool {
        true
    }

    /// Данные — палитра в виде списка чисел.
    fn get_data(&self) -> Value {
        Value::List(self.palette.iter().map(|&a| Value::U32(a)).collect())
    }

    fn set_data(&mut self, value: &Value) {
        if let Value::List(v) = value {
            self.palette = v.iter().filter_map(|x| x.as_u32()).collect();
            self.shown = None;
            self.sync();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::palettes::APP_COLOR;

    #[test]
    fn selector_keys() {
        let mut s = ColorSelector::new(Rect::new(0, 0, 12, 4), ColorSel::Foreground);
        s.vs.state |= crate::view::sf::FOCUSED;
        let mut ctx = Ctx::new();
        let mut press = |s: &mut ColorSelector, k| {
            let mut e = Event::KeyDown(crate::event::KeyEvent::new(k, Mods::NONE));
            s.handle_event(&mut e, &mut ctx);
        };
        press(&mut s, Key::Left);
        assert_eq!(s.color, 15);
        press(&mut s, Key::Down);
        assert_eq!(s.color, 0);
        press(&mut s, Key::Down);
        assert_eq!(s.color, 4);
    }

    #[test]
    fn dialog_sync() {
        let mut d = ColorDialog::new(APP_COLOR, standard_color_groups());
        // Desktop/Color = индекс 1 = 0x71: текст синий (1), фон серый (7).
        assert_eq!(d.dlg.win.group.find::<ColorSelector>(d.fore_sel).unwrap().color, 1);
        assert_eq!(d.dlg.win.group.find::<ColorSelector>(d.back_sel).unwrap().color, 7);
        // Меняем цвет текста на красный.
        d.dlg.win.group.find_mut::<ColorSelector>(d.fore_sel).unwrap().color = 4;
        d.sync();
        assert_eq!(d.palette[0], 0x74);
    }
}
