//! Предпросмотр темы: маленький рабочий стол из настоящих виджетов,
//! нарисованных в цветах редактируемой темы (`DrawCtx::sub_themed`), —
//! цвета самого редактора при этом не меняются.

use std::cell::RefCell;
use std::rc::Rc;

use tvision::layout::SizePolicy;
use tvision::theme::Theme;
use tvision::widgets::editor::Pos;
use tvision::*;

/// Образцы: что и где нарисовать. Координаты — в предпросмотре.
pub struct Preview {
    pub vs: ViewState,
    pub theme: Rc<RefCell<Theme>>,
    samples: Vec<Box<dyn View>>,
}

fn editor_window(r: Rect, title: &str, num: i16, text: &str, select: Option<(Pos, Pos)>, palette: Option<&'static [u8]>) -> Window {
    let mut w = Window::new(r, title, num);
    if let Some(p) = palette {
        w.palette = p;
    }
    let v = w.standard_scroll_bar(true);
    let h = w.standard_scroll_bar(false);
    let mut e = Editor::new(w.interior(), Some(h), Some(v), None);
    e.set_text(text);
    if let Some((a, b)) = select {
        e.set_select(a, b);
    }
    w.add(e);
    w
}

fn sample_dialog(r: Rect) -> Dialog {
    let mut d = Dialog::new(r, "Диалог");
    let name = d.add(InputLine::new(Rect::new(12, 2, 34, 3), 40));
    d.add(Label::new(Rect::new(2, 2, 11, 3), "~И~мя:", Some(name)));
    let pass = d.add(InputLine::new(Rect::new(12, 4, 34, 5), 40));
    d.add(Label::new(Rect::new(2, 4, 11, 5), "~П~ароль:", Some(pass)));
    d.add(StaticText::new(Rect::new(2, 6, 34, 7), "Обычный текст диалога"));
    let mut checks = Cluster::check_boxes(Rect::new(2, 8, 20, 10), &["~З~апомнить", "~А~втовход"]);
    checks.value = 1;
    d.add(checks);
    d.add(Cluster::radio_buttons(Rect::new(21, 8, 34, 10), &["~Р~абота", "~Д~ом"]));
    let mut list = ListBox::new(Rect::new(36, 2, 50, 10), 1, None);
    list.set_items(["Первый", "Второй", "Третий", "Четвёртый"].iter().map(|s| s.to_string()).collect());
    list.focused = 1;
    d.add(list);
    d.add(Button::new(Rect::new(14, 11, 24, 13), "O~K~", cm::OK, bf::DEFAULT));
    d.add(Button::new(Rect::new(26, 11, 38, 13), "~О~тмена", cm::CANCEL, bf::NORMAL));
    d.select(name);
    d
}

impl Preview {
    pub fn new(theme: Rc<RefCell<Theme>>) -> Self {
        let mut vs = ViewState::new(Rect::default());
        vs.options |= of::SELECTABLE | of::FIRST_CLICK;
        let mut samples: Vec<Box<dyn View>> = Vec::new();

        let menu = Menu::new()
            .submenu("~Ф~айл", Menu::new().item("~О~ткрыть", cm::OK))
            .submenu("~П~равка", Menu::new().item("~К~опировать", cm::OK))
            .submenu("~О~кна", Menu::new().item("~С~ледующее", cm::OK));
        let mut bar = MenuBar::new(Rect::new(0, 0, 60, 1), menu);
        bar.active = Some(1); // выбранный пункт
        samples.push(Box::new(bar));

        let cyan = editor_window(
            Rect::new(30, 2, 60, 9),
            "Бирюзовое",
            2,
            "Неактивное окно\nдругой палитры.",
            None,
            Some(tvision::color::palettes::CYAN_WINDOW),
        );
        samples.push(Box::new(cyan));

        let mut blue = editor_window(
            Rect::new(1, 2, 33, 11),
            "Синее окно",
            1,
            "fn main() {\n    println!(\"Привет!\");\n}\n\nВыделенный текст —\nвот этот.",
            Some((Pos { line: 5, col: 0 }, Pos { line: 5, col: 9 })),
            None,
        );
        blue.set_state(sf::ACTIVE, true);
        samples.push(Box::new(blue));

        let mut dlg = sample_dialog(Rect::new(6, 9, 58, 23));
        dlg.set_state(sf::ACTIVE, true);
        dlg.set_state(sf::FOCUSED, true);
        dlg.set_state(sf::SELECTED, true);
        samples.push(Box::new(dlg));

        let status = StatusLine::new(
            Rect::new(0, 24, 60, 25),
            vec![StatusDef::all(vec![
                StatusItem::new("~F1~ Справка", kb::F1, cm::HELP),
                StatusItem::new("~F2~ Сохранить", kb::F2, cm::OK),
                StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT),
            ])],
        );
        samples.push(Box::new(status));
        Preview { vs, theme, samples }
    }
}

impl View for Preview {
    tvision::impl_view_basics!();

    fn size_policy(&self) -> SizePolicy {
        SizePolicy::EXPANDING
    }

    fn size_hint(&self) -> Option<Point> {
        Some(Point::new(60, 25))
    }

    /// Строка меню — на всю ширину, строка состояния — внизу.
    fn change_bounds(&mut self, bounds: Rect) {
        self.vs.set_bounds(bounds);
        let size = self.vs.size;
        for v in &mut self.samples {
            if v.as_any().is::<MenuBar>() {
                v.change_bounds(Rect::new(0, 0, size.x, 1));
            } else if v.as_any().is::<StatusLine>() {
                v.change_bounds(Rect::new(0, size.y - 1, size.x, size.y));
            }
        }
    }

    fn draw(&self, dc: &mut DrawCtx) {
        let theme = self.theme.borrow();
        let pal = &theme.palette;
        let size = self.vs.size;
        // Рабочий стол — цвет 1 темы.
        let desk = pal.first().copied().unwrap_or(0x71);
        dc.fill(Rect::new(0, 0, size.x, size.y), '░', desk);
        for v in &self.samples {
            let mut sub = dc.sub_themed(v.vs().bounds(), v.palette(), pal);
            v.draw(&mut sub);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tvision::platform::HeadlessBackend;
    use tvision::theme::{attr_from_text, Base};

    /// Нарисовать предпросмотр в невидимом терминале и вернуть программу.
    fn draw(theme: Theme) -> Program {
        let mut app = Program::new(Box::new(HeadlessBackend::new(70, 27)), None, None);
        let mut w = Window::new(Rect::new(0, 0, 66, 27), "Предпросмотр", 0);
        let mut p = Preview::new(Rc::new(RefCell::new(theme)));
        p.change_bounds(w.interior());
        w.add(p);
        app.insert_window(w);
        app.redraw();
        app
    }

    #[test]
    fn shows_samples_in_theme_colors() {
        let mut t = Theme::builtin(Base::Classic);
        t.set(1, attr_from_text("red/green").unwrap()); // рабочий стол
        t.set(42, attr_from_text("yellow/magenta").unwrap()); // кнопка по умолчанию
        let app = draw(t);
        let s = app.screen_text();
        for w in ["Файл", "Синее окно", "Бирюзовое", "Диалог", "Пароль:", "Запомнить", "Второй", "OK", "F2 Сохранить"]
        {
            assert!(s.contains(w), "нет «{}»:\n{}", w, s);
        }
        // Цвета — из темы: найдём клетку рабочего стола и кнопки OK.
        let cells: Vec<(char, u32)> = app.screen().cells.iter().map(|c| (c.ch, c.attr)).collect();
        assert!(cells.iter().any(|&(ch, a)| ch == '░' && a == attr_from_text("red/green").unwrap()), "рабочий стол в цветах темы");
        assert!(cells.iter().any(|&(ch, a)| ch == 'O' && a == attr_from_text("yellow/magenta").unwrap()), "кнопка по умолчанию в цветах темы");
    }
}
