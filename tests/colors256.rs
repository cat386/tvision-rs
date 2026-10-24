//! 256 цветов: палитра приложения, диалог цветов, вывод в терминал.

use tvision::color::{self, attr256, fg256, is_bg256, is_fg256};
use tvision::platform::{HeadlessBackend, RawEvent};
use tvision::*;

const CM_COLORS: u16 = 200;

fn plain(k: Key) -> RawEvent {
    RawEvent::Key(KeyEvent::new(k, Mods::NONE))
}

/// Атрибут ячейки рабочего стола (вне окон) берётся из палитры приложения.
#[test]
fn app_palette_256() {
    let b = HeadlessBackend::new(40, 10);
    let mut app = Program::new(Box::new(b), None, None);
    // Цвет 1 палитры приложения — фон рабочего стола.
    app.palette[0] = attr256(250, 24);
    // Синее окно: обычный текст (цвет 13 приложения) — 256-цветный.
    app.palette[12] = attr256(223, 17);
    let mut w = Window::new(Rect::new(2, 1, 30, 8), "Окно", 1);
    w.add(StaticText::new(Rect::new(1, 1, 20, 2), "Привет"));
    app.insert_window(w);
    app.pump(&mut program::NoHandler);
    app.redraw();
    assert_eq!(app.screen().get(35, 5).attr, attr256(250, 24));
    // StaticText в окне берёт цвет 6 окна → 13 приложения.
    let a = app.screen().get(4, 2).attr;
    assert_eq!(a, attr256(223, 17), "{}", app.screen_text());
}

struct ColorsApp {
    result: Option<(u16, Vec<Attr>)>,
}

impl AppHandler for ColorsApp {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(CM_COLORS) {
            let d = ColorDialog::new(&app.palette, standard_color_groups());
            let (c, d) = app.exec_dialog(d);
            self.result = Some((c, d.palette.clone()));
            ev.clear();
        }
    }
}

/// «Текст 256…» → стрелка вниз → OK → OK: цвет текста первого элемента
/// (рабочий стол) становится номером xterm, фон остаётся цветом BIOS.
#[test]
fn color_dialog_picks_256() {
    let mut b = HeadlessBackend::new(80, 25);
    b.push_key(kb::alt('x')); // кнопка «Te~x~t 256...»
    b.input.push_back(plain(Key::Down)); // в окне выбора: 4 → 36
    b.input.push_back(plain(Key::Enter)); // OK окна выбора
    b.push_key(kb::alt('k')); // O~K~ диалога цветов
    let mut app = Program::new(Box::new(b), None, None);
    app.palette = color::palettes::APP_COLOR.to_vec(); // классическая палитра
    app.put_event(Event::command_event(CM_COLORS));
    let mut h = ColorsApp { result: None };
    app.pump(&mut h);
    let (c, pal) = h.result.expect("диалог не открылся");
    assert_eq!(c, cm::OK);
    let a = pal[0];
    assert!(is_fg256(a) && !is_bg256(a), "атрибут {:#x}", a);
    // Было 0x71: текст синий BIOS (xterm 4) → на строку ниже в сетке 32×8.
    assert_eq!(fg256(a), 4 + 32);
    assert_eq!(color::back(a), 7);
}

/// Изменение фона в сетке 16 цветов не сбрасывает 256-цветный текст.
#[test]
fn selector_keeps_256_part() {
    let mut pal = color::palettes::APP_COLOR.to_vec();
    pal[0] = color::with_fg256(0x71, 208);
    let mut d = ColorDialog::new(&pal, standard_color_groups());
    // Произвольные события без изменения сеток не трогают палитру.
    let mut ctx = Ctx::new();
    let mut ev = Event::Nothing;
    d.handle_event(&mut ev, &mut ctx);
    assert_eq!(d.palette[0], color::with_fg256(0x71, 208));
    // Сетка фона: фон → зелёный BIOS, текст остаётся 208.
    for v in d.dlg.win.group.children.iter_mut() {
        if let Some(s) = v.as_any_mut().downcast_mut::<ColorSelector>() {
            if s.sel_type == ColorSel::Background {
                s.color = 2;
            }
        }
    }
    let mut ev = Event::Nothing;
    d.handle_event(&mut ev, &mut ctx);
    assert_eq!(fg256(d.palette[0]), 208);
    assert_eq!(color::back(d.palette[0]), 2);
    assert!(!is_bg256(d.palette[0]));
}

/// Сетка 256 цветов рисует каждую клетку своим цветом фона.
#[test]
fn selector_256_draws_all_colors() {
    let b = HeadlessBackend::new(80, 25);
    let mut app = Program::new(Box::new(b), None, None);
    let mut d = Color256Dialog::new(ColorSel::Background, 0x1E);
    d.dlg.win.group.vs.state &= !sf::MODAL;
    app.insert_window(d);
    app.redraw();
    let s = app.screen_text();
    let (x0, y0) = s.lines().enumerate().find_map(|(y, l)| l.chars().position(|c| c == '◄').map(|x| (x, y))).expect("метка выбранного цвета");
    // Начальный цвет — синий фон BIOS = xterm 4 → столбец 4 первой строки.
    let (gx, gy) = (x0 as i32 - 8, y0 as i32);
    for idx in [0u8, 17, 100, 231, 232, 255] {
        let (cx, cy) = (gx + (idx as i32 % 32) * 2, gy + idx as i32 / 32);
        let a = app.screen().get(cx as usize, cy as usize).attr;
        assert_eq!(color::bg256(a), idx, "клетка {}", idx);
    }
}
