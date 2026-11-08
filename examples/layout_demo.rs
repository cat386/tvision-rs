//! Раскладки в духе Qt (`tvision::layout`): ни одной координаты вручную.
//!
//! Запуск: `cargo run --example layout_demo`
//!
//! * **«Почта»** (открыта сразу) — вложенные `HBox`/`VBox`: папки слева
//!   (1 часть ширины), письма и текст справа (3 части), кнопки внизу
//!   справа. Потяните окно за правый нижний угол — всё перераскладывается.
//! * **F2 — «Вход»**: `Form` (подписи — поля) и ряд кнопок; размер
//!   диалога — по содержимому (`adjust_size`).
//! * **F3 — «Калькулятор»**: `Grid` 4×5 с растягивающимися кнопками и
//!   табло на три клетки. Работает: цифры, + − × ÷, =, C.
//!
//! Alt-X — выход.

use tvision::layout::*;
use tvision::*;

const CM_LOGIN: u16 = 160;
const CM_CALC: u16 = 161;
/// Кнопки калькулятора: 200 + номер символа в `KEYS`.
const CM_KEY: u16 = 200;
const KEYS: &str = "789/456*123-0.=+C";

// ----------------------------------------------------------------------
// «Почта»: вложенные строки и столбцы
// ----------------------------------------------------------------------

pub fn mail_window(r: Rect) -> Window {
    let mut w = Window::new(r, "Почта", 1);
    // Все элементы — с пустым прямоугольником: места расставит раскладка.
    let mut folders = ListBox::new(Rect::default(), 1, None);
    folders.set_items(["Входящие", "Отправленные", "Черновики", "Спам", "Корзина"].iter().map(|s| s.to_string()).collect());
    let folders = w.add(folders);
    let rows = vec![
        vec!["Анна".into(), "Отчёт за ноябрь".into(), "08.11".into()],
        vec!["Борис".into(), "Встреча в пятницу".into(), "07.11".into()],
        vec!["Вера".into(), "Фотографии с дачи".into(), "05.11".into()],
    ];
    let mut t = Table::new(Rect::default(), Box::new(VecTableModel::new(&["От", "Тема", "Дата"], rows)), None, None);
    t.auto_width(50);
    let letters = w.add(t);
    let text = w.add(StaticText::new(Rect::default(), "Добрый день!\n\nОтчёт во вложении. Посмотрите, пожалуйста, до понедельника.\n\nАнна"));
    let reply = w.add(Button::new(Rect::default(), "~О~тветить", cm::OK, bf::NORMAL));
    let delete = w.add(Button::new(Rect::default(), "~У~далить", cm::OK, bf::NORMAL));

    w.set_layout(
        VBox::new()
            .add(
                HBox::new()
                    .add(item(folders).stretch(1)) // 1 часть ширины
                    .add(
                        item(
                            VBox::new()
                                .spacing(1)
                                .add(item(letters).stretch(2)) // 2/3 высоты
                                .add(item(text).expand().stretch(1)), // 1/3 высоты
                        )
                        .stretch(3), // 3 части ширины
                    ),
            )
            .space(1)
            .add(HBox::new().stretch(1).add(reply).add(delete)), // кнопки — вправо
    );
    w
}

// ----------------------------------------------------------------------
// «Вход»: форма
// ----------------------------------------------------------------------

pub fn login_dialog() -> Dialog {
    let mut d = Dialog::centered(30, 10, "Вход"); // размер — временный
    let user = d.add(InputLine::new(Rect::default(), 30));
    let pass = d.add(InputLine::new(Rect::default(), 30));
    let server = d.add(Cluster::radio_buttons(Rect::default(), &["~Р~абочий", "~Д~омашний"]));
    let remember = d.add(Cluster::check_boxes(Rect::default(), &["~З~апомнить пароль"]));
    let lu = d.add(Label::new(Rect::default(), "~И~мя:", Some(user)));
    let lp = d.add(Label::new(Rect::default(), "~П~ароль:", Some(pass)));
    let ls = d.add(Label::new(Rect::default(), "~С~ервер:", Some(server)));
    let ok = d.add(Button::new(Rect::default(), "O~K~", cm::OK, bf::DEFAULT));
    let cancel = d.add(Button::new(Rect::default(), "Отмена", cm::CANCEL, bf::NORMAL));
    d.set_layout(
        VBox::new()
            .margins(2, 1, 2, 0)
            .add(Form::new().row(lu, user).row(lp, pass).row(ls, server).row_wide(remember))
            .space(1)
            .add(HBox::new().stretch(1).add(ok).add(cancel).stretch(1)), // кнопки — по центру
    );
    d.adjust_size(); // по содержимому
    d.select(user);
    d
}

// ----------------------------------------------------------------------
// «Калькулятор»: сетка
// ----------------------------------------------------------------------

pub struct Calc {
    pub display: ViewId,
}

pub fn calc_window(r: Rect) -> (Window, ViewId) {
    let mut w = Window::new(r, "Калькулятор", 2);
    let display = w.add(StaticText::new(Rect::default(), "0"));
    let mut grid = Grid::new().vspacing(0).span(0, 0, 1, 3, item(display).align_v(Alignment::Center));
    for (i, ch) in KEYS.chars().enumerate() {
        let label = match ch {
            '*' => "×".to_string(),
            '/' => "÷".to_string(),
            c => c.to_string(),
        };
        let b = w.add(Button::new(Rect::default(), &label, CM_KEY + i as u16, bf::NORMAL));
        let b = item(b).expand(); // кнопки заполняют свои клетки
        grid = if ch == 'C' { grid.cell(0, 3, b) } else { grid.cell(1 + i / 4, i % 4, b) };
    }
    for c in 0..4 {
        grid = grid.col_stretch(c, 1);
    }
    for r in 1..5 {
        grid = grid.row_stretch(r, 1);
    }
    w.set_layout(grid.margins(1, 0, 1, 0));
    (w, display)
}

/// Посчитать «12+3*2» слева направо (как простой калькулятор).
pub fn eval(expr: &str) -> Option<f64> {
    let mut acc: Option<f64> = None;
    let mut op = '+';
    let mut num = String::new();
    for c in expr.chars().chain(std::iter::once('=')) {
        if c.is_ascii_digit() || c == '.' {
            num.push(c);
            continue;
        }
        let n: f64 = num.parse().ok()?;
        num.clear();
        let a = acc.unwrap_or(0.0);
        acc = Some(match op {
            '+' => a + n,
            '-' => a - n,
            '*' => a * n,
            '/' if n != 0.0 => a / n,
            _ => return None,
        });
        op = c;
    }
    acc
}

fn format_num(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 1e15 {
        format!("{}", x as i64)
    } else {
        format!("{:.6}", x).trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Нажатие кнопки калькулятора: новый текст табло.
pub fn press(display: &str, key: char) -> String {
    let d = if display == "0" || display == "Ошибка" { "" } else { display };
    match key {
        'C' => "0".into(),
        '=' => eval(d).map(format_num).unwrap_or_else(|| "Ошибка".into()),
        c if "+-*/".contains(c) && (d.is_empty() || d.ends_with(|x: char| "+-*/".contains(x))) => display.to_string(),
        c => format!("{}{}", d, c),
    }
}

// ----------------------------------------------------------------------
// Приложение
// ----------------------------------------------------------------------

struct App {
    calc: Option<ViewId>,
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        let Some(cmd) = ev.command() else { return };
        match cmd {
            CM_LOGIN => {
                app.exec_dialog(login_dialog());
            }
            CM_CALC => {
                let (w, display) = calc_window(Rect::new(50, 2, 76, 16));
                app.insert_window(w);
                self.calc = Some(display);
            }
            c if (CM_KEY..CM_KEY + KEYS.len() as u16).contains(&c) => {
                let key = KEYS.chars().nth((c - CM_KEY) as usize).unwrap();
                if let Some(t) = self.calc.and_then(|id| app.find_mut::<StaticText>(id)) {
                    t.text = press(&t.text, key);
                }
            }
            _ => return,
        }
        ev.clear();
    }
}

fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![
        StatusItem::new("~F2~ Вход", kb::F2, CM_LOGIN),
        StatusItem::new("~F3~ Калькулятор", kb::F3, CM_CALC),
        StatusItem::new("~Ctrl-F5~ Размер окна", kb::ctrl_f(5), cm::RESIZE),
        StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT),
    ])]
}

fn main() -> std::io::Result<()> {
    locale::set_language(locale::Language::Ru);
    let mut app = Program::with_terminal(None, Some(status()))?;
    app.insert_window(mail_window(Rect::new(1, 1, 60, 20)));
    app.run(&mut App { calc: None });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tvision::platform::HeadlessBackend;

    fn find(w: &Window, title: &str) -> Rect {
        w.group.children.iter().find(|c| c.as_any().downcast_ref::<Button>().is_some_and(|b| b.title == title)).unwrap().vs().bounds()
    }

    #[test]
    fn mail_relayouts_on_resize() {
        let mut w = mail_window(Rect::new(0, 0, 60, 20));
        let folders = w.group.children.iter().find(|c| c.as_any().is::<ListBox>()).unwrap().vs().bounds();
        // Внутри рамки 58 клеток: 1 промежуток, остальное 1:3 → 14 и 43.
        assert_eq!((folders.a.x, folders.b.x), (1, 15));
        assert_eq!(find(&w, "~У~далить"), Rect::new(48, 17, 59, 19)); // «Удалить» + 4 = 11
                                                                      // Окно шире — пропорции те же, кнопки у правого края.
        w.change_bounds(Rect::new(0, 0, 100, 30));
        let folders = w.group.children.iter().find(|c| c.as_any().is::<ListBox>()).unwrap().vs().bounds();
        assert_eq!((folders.a.x, folders.b.x), (1, 25));
        assert_eq!(find(&w, "~У~далить"), Rect::new(88, 27, 99, 29));
    }

    #[test]
    fn login_fits_content() {
        let d = login_dialog();
        let s = d.win.group.vs.size;
        // Ширина: поле 2 + подпись «Пароль:» 8 + 1 + строка ввода 30 + поле 2 + рамка 2.
        // Высота: 1 + четыре строки формы (1+1+2+1) с промежутками 3 + 1 + кнопки 2 + рамка 2.
        assert_eq!(s, Point::new(45, 14));
        // Кнопки — по центру.
        let ok = find(&d.win, "O~K~");
        let cancel = find(&d.win, "Отмена");
        assert_eq!(ok.a.x - 1, s.x - 1 - cancel.b.x);
    }

    #[test]
    fn calc_grid_scales() {
        let (mut w, _) = calc_window(Rect::new(0, 0, 26, 14));
        let seven = find(&w, "7");
        let plus = find(&w, "+");
        assert_eq!(seven.a, Point::new(2, 3));
        w.change_bounds(Rect::new(0, 0, 50, 26));
        let seven2 = find(&w, "7");
        assert!(seven2.b.x - seven2.a.x > seven.b.x - seven.a.x, "кнопки растут вместе с окном");
        assert_eq!(find(&w, "+").b.x, 48, "последний столбец — у правого края (рамка + поле)");
        let _ = plus;
    }

    #[test]
    fn calculator_math() {
        assert_eq!(eval("12+3*2"), Some(30.0));
        assert_eq!(eval("1/0"), None);
        let mut d = "0".to_string();
        for k in "7*6=".chars() {
            d = press(&d, k);
        }
        assert_eq!(d, "42");
        assert_eq!(press("5+", '*'), "5+", "два знака подряд нельзя");
        assert_eq!(press("1/0", '='), "Ошибка");
    }

    #[test]
    fn calc_buttons_work() {
        let mut b = HeadlessBackend::new(80, 25);
        b.esc_budget = 0;
        let mut app = Program::new(Box::new(b), None, Some(status()));
        let mut h = App { calc: None };
        for c in [CM_CALC, CM_KEY + 8, CM_KEY + 15, CM_KEY + 9, CM_KEY + 14] {
            app.put_event(Event::command_event(c)); // открыть, 1 + 2 =
            app.pump(&mut h);
        }
        let t = app.find::<StaticText>(h.calc.unwrap()).unwrap();
        assert_eq!(t.text, "3");
    }
}
