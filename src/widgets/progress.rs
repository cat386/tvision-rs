//! Полоса выполнения (`ProgressBar`).
//!
//! В оригинальном Turbo Vision такого виджета не было — его обычно писали
//! сами (в Borland Pascal 7 был похожий `TProgressBar` в примерах). Здесь он
//! сделан в духе библиотеки: обычное представление со своей палитрой,
//! данными (`get_data`/`set_data`) и управлением через сообщение.
//!
//! Возможности:
//! * диапазон `min..=max` и текущее значение;
//! * плавное заполнение с точностью 1/8 клетки (`▏▎▍▌▋▊▉█`) или «блоками»
//!   в стиле DOS (`█` и `░`);
//! * надпись поверх полосы: проценты, свой текст или шаблон
//!   `"{value} из {max} ({percent}%)"`;
//! * режим «неизвестной длительности» (`indeterminate`): по полосе бегает
//!   блок — когда общий объём работы заранее неизвестен.
//!
//! Обновлять полосу можно напрямую (`app.find_mut::<ProgressBar>(id)`),
//! изнутри другого представления — сообщением [`cm::PROGRESS_SET`] с
//! [`Info::Progress`], а в модальном диалоге — из функции, переданной в
//! [`crate::Program::exec_view_idle`].

use std::time::Instant;

use crate::color::{self, Attr};
use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Info};
use crate::geometry::Rect;
use crate::text::str_width;
use crate::view::{Ctx, DrawCtx, Value, View, ViewState};

/// Палитра полосы (индексы в палитре диалога):
/// 1 — пустая часть (как статический текст), 2 — заполненная часть
/// (как выбранная строка ввода).
///
/// В окне с короткой палитрой (обычное синее окно) этих индексов нет —
/// задайте цвета явно через [`ProgressBar::with_colors`].
pub const PROGRESS_PALETTE: &[u8] = b"\x05\x13";

/// Вид заполнения.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ProgressStyle {
    /// Плавно, с точностью до 1/8 клетки.
    #[default]
    Smooth,
    /// Целыми клетками: `█` — заполнено, `░` — пусто (как в программах для DOS).
    Blocks,
}

/// Символы частичного заполнения клетки: 1/8 … 7/8.
const EIGHTHS: [char; 7] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉'];

/// Полоса выполнения.
pub struct ProgressBar {
    pub vs: ViewState,
    pub min: i64,
    pub max: i64,
    value: i64,
    pub style: ProgressStyle,
    /// Показывать надпись поверх полосы.
    pub show_text: bool,
    /// Своя надпись вместо процентов. Поддерживает подстановки
    /// `{value}`, `{min}`, `{max}`, `{percent}`.
    pub caption: Option<String>,
    /// Режим неизвестной длительности: бегающий блок вместо заполнения.
    pub indeterminate: bool,
    /// Свои цвета (пустая часть, заполненная часть) вместо палитры.
    pub colors: Option<(Attr, Attr)>,
    /// Момент создания — от него считается анимация бегающего блока.
    started: Instant,
}

impl ProgressBar {
    /// Полоса с диапазоном `0..=max`. Обычно высота — одна строка.
    pub fn new(bounds: Rect, max: i64) -> Self {
        ProgressBar {
            vs: ViewState::new(bounds),
            min: 0,
            max: max.max(0),
            value: 0,
            style: ProgressStyle::Smooth,
            show_text: true,
            caption: None,
            indeterminate: false,
            colors: None,
            started: Instant::now(),
        }
    }

    /// Задать диапазон (значение подрезается под него).
    pub fn with_range(mut self, min: i64, max: i64) -> Self {
        self.set_range(min, max);
        self
    }

    pub fn with_style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    /// Своя надпись: `"Копирование: {percent}%"`, `"{value}/{max} файлов"`.
    pub fn with_caption(mut self, caption: &str) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// Свои цвета (атрибуты BIOS или 256-цветные): пустая часть и заполненная.
    pub fn with_colors(mut self, empty: Attr, filled: Attr) -> Self {
        self.colors = Some((empty, filled));
        self
    }

    /// Режим неизвестной длительности.
    pub fn indeterminate(mut self) -> Self {
        self.indeterminate = true;
        self
    }

    pub fn set_range(&mut self, min: i64, max: i64) {
        self.min = min;
        self.max = max.max(min);
        self.value = self.value.clamp(self.min, self.max);
    }

    /// Установить значение (подрезается под диапазон).
    pub fn set_value(&mut self, v: i64) {
        self.value = v.clamp(self.min, self.max);
    }

    pub fn value(&self) -> i64 {
        self.value
    }

    /// Прибавить `delta` к значению.
    pub fn step(&mut self, delta: i64) {
        self.set_value(self.value.saturating_add(delta));
    }

    /// Готово ли (значение дошло до максимума).
    pub fn is_done(&self) -> bool {
        !self.indeterminate && self.value >= self.max
    }

    /// Доля выполненного: 0.0 … 1.0.
    pub fn fraction(&self) -> f64 {
        let span = self.max - self.min;
        if span <= 0 {
            return 1.0;
        }
        (self.value - self.min) as f64 / span as f64
    }

    /// Процент выполненного (0–100, округление вниз — 100% только в самом конце).
    pub fn percent(&self) -> u8 {
        (self.fraction() * 100.0).floor().clamp(0.0, 100.0) as u8
    }

    /// Текст надписи.
    pub fn text(&self) -> String {
        match &self.caption {
            Some(c) => c
                .replace("{value}", &self.value.to_string())
                .replace("{min}", &self.min.to_string())
                .replace("{max}", &self.max.to_string())
                .replace("{percent}", &self.percent().to_string()),
            None if self.indeterminate => String::new(),
            None => format!("{}%", self.percent()),
        }
    }

    /// Сколько восьмых долей клетки заполнено при ширине `w`.
    fn filled_eighths(&self, w: usize) -> usize {
        ((self.fraction() * (w * 8) as f64).round() as usize).min(w * 8)
    }

    /// Границы бегающего блока `[a, b)` в клетках (режим неизвестной длительности).
    fn marquee(&self, w: usize) -> (usize, usize) {
        let len = (w / 5).max(3).min(w);
        let travel = w - len;
        if travel == 0 {
            return (0, w);
        }
        // Шаг — 60 мс, блок ходит туда и обратно.
        let t = (self.started.elapsed().as_millis() / 60) as usize % (travel * 2);
        let a = if t <= travel { t } else { travel * 2 - t };
        (a, a + len)
    }
}

/// Атрибут «цвет текста = фон заполненной части, фон = фон пустой части» —
/// для символа частичного заполнения.
fn partial_attr(empty: Attr, filled: Attr) -> Attr {
    if color::is_bg256(filled) {
        color::with_fg256(empty, color::bg256(filled))
    } else {
        color::with_fore(empty, color::back(filled))
    }
}

impl View for ProgressBar {
    crate::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let (empty, filled) = self.colors.unwrap_or_else(|| (dc.color(1), dc.color(2)));
        let w = self.vs.size.x.max(0) as usize;
        if w == 0 {
            return;
        }
        let track_char = match self.style {
            ProgressStyle::Smooth => ' ',
            ProgressStyle::Blocks => '░',
        };
        let mut b = DrawBuffer::new(w);
        b.move_char(0, track_char, empty, w);

        // Какие клетки заполнены: [fa, fb) целиком, плюс, возможно, одна частичная.
        let (fa, fb, partial) = if self.indeterminate {
            let (a, b) = self.marquee(w);
            (a, b, None)
        } else {
            let e = self.filled_eighths(w);
            match self.style {
                ProgressStyle::Smooth => (0, e / 8, if e % 8 != 0 { Some((e / 8, e % 8)) } else { None }),
                // Блоками: клетка заполнена, если заполнено больше половины.
                ProgressStyle::Blocks => (0, (e + 4) / 8, None),
            }
        };
        match self.style {
            ProgressStyle::Smooth => {
                b.move_char(fa, ' ', filled, fb - fa);
                if let Some((x, n)) = partial {
                    b.move_char(x, EIGHTHS[n - 1], partial_attr(empty, filled), 1);
                }
            }
            ProgressStyle::Blocks => {
                b.move_char(fa, '█', partial_attr(empty, filled), fb - fa);
            }
        }

        // Надпись по центру: над заполненной частью — цветом заполнения,
        // над пустой — цветом пустой части.
        if self.show_text {
            let text = self.text();
            let tw = str_width(&text);
            if tw > 0 && tw <= w {
                let x0 = (w - tw) / 2;
                let mut x = x0;
                for ch in text.chars() {
                    let cw = crate::text::char_width(ch).max(1);
                    let on_fill = x >= fa && x < fb;
                    let attr = if on_fill { filled } else { empty };
                    let mut tmp = [0u8; 4];
                    b.move_str(x, ch.encode_utf8(&mut tmp), attr);
                    x += cw;
                }
            }
        }
        dc.write_line(0, 0, self.vs.size.x, self.vs.size.y.max(1), &b);
    }

    fn palette(&self) -> &'static [u8] {
        PROGRESS_PALETTE
    }

    fn handle_event(&mut self, ev: &mut Event, _ctx: &mut Ctx) {
        // Обновление сообщением: `ctx.message(cm::PROGRESS_SET, Info::Progress { id, value })`.
        if let Event::Broadcast(m) = ev {
            if let (cm::PROGRESS_SET, Info::Progress { id, value }) = (m.command, &m.info) {
                if *id == self.vs.id {
                    self.set_value(*value);
                    ev.clear();
                }
            }
        }
    }

    fn has_data(&self) -> bool {
        true
    }

    /// Данные — текущее значение.
    fn get_data(&self) -> Value {
        Value::Int(self.value)
    }

    fn set_data(&mut self, value: &Value) {
        if let Value::Int(v) = value {
            self.set_value(*v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drawbuf::ScreenBuffer;
    use crate::view::CommandSet;

    /// Рисует полосу на экран шириной `w` с явными цветами и возвращает строку.
    fn render(p: &ProgressBar) -> (String, ScreenBuffer) {
        let w = p.vs.size.x as usize;
        let mut scr = ScreenBuffer::new(w, 1);
        let cmds = CommandSet::default();
        let pal = crate::color::palettes::APP_COLOR;
        {
            let mut dc = DrawCtx::new(&mut scr, pal, &cmds);
            let mut sub = dc.sub(p.vs.bounds(), &[]);
            p.draw(&mut sub);
        }
        (scr.line_text(0), scr)
    }

    #[test]
    fn values_and_percent() {
        let mut p = ProgressBar::new(Rect::new(0, 0, 20, 1), 200);
        assert_eq!(p.percent(), 0);
        p.set_value(50);
        assert_eq!(p.percent(), 25);
        p.step(1000);
        assert_eq!(p.value(), 200);
        assert!(p.is_done());
        p.set_value(-5);
        assert_eq!(p.value(), 0);
        let p = ProgressBar::new(Rect::new(0, 0, 20, 1), 10).with_range(10, 20);
        assert_eq!(p.value(), 10);
        // 199 из 200 — ещё не 100%.
        let mut p = ProgressBar::new(Rect::new(0, 0, 20, 1), 200);
        p.set_value(199);
        assert_eq!(p.percent(), 99);
    }

    #[test]
    fn caption_placeholders() {
        let mut p = ProgressBar::new(Rect::new(0, 0, 30, 1), 8).with_caption("{value} из {max} ({percent}%)");
        p.set_value(2);
        assert_eq!(p.text(), "2 из 8 (25%)");
    }

    #[test]
    fn smooth_drawing() {
        // 10 клеток, 25% = 2.5 клетки: 2 полных, половинка «▌», текст «25%» по центру.
        let mut p = ProgressBar::new(Rect::new(0, 0, 10, 1), 100).with_colors(0x70, 0x2F);
        p.set_value(25);
        p.show_text = false;
        let (s, scr) = render(&p);
        assert_eq!(s, "  ▌       ");
        assert_eq!(scr.get(0, 0).attr, 0x2F);
        assert_eq!(scr.get(2, 0).attr, 0x72, "частичная клетка: зелёный на сером");
        assert_eq!(scr.get(3, 0).attr, 0x70);
        p.show_text = true;
        let (s, scr) = render(&p);
        assert_eq!(s, "  ▌25%    ");
        assert_eq!(scr.get(3, 0).attr, 0x70);
    }

    #[test]
    fn blocks_drawing() {
        let mut p = ProgressBar::new(Rect::new(0, 0, 10, 1), 10).with_style(ProgressStyle::Blocks).with_colors(0x70, 0x2F);
        p.set_value(4);
        p.show_text = false;
        let (s, _) = render(&p);
        assert_eq!(s, "████░░░░░░");
    }

    #[test]
    fn indeterminate_block() {
        let p = ProgressBar::new(Rect::new(0, 0, 20, 1), 0).indeterminate().with_colors(0x70, 0x2F);
        let (a, b) = p.marquee(20);
        assert_eq!(b - a, 4);
        assert!(b <= 20);
        assert_eq!(p.text(), "");
        assert!(!p.is_done());
    }

    #[test]
    fn message_updates() {
        let mut p = ProgressBar::new(Rect::new(0, 0, 10, 1), 100);
        let id = p.vs.id;
        let mut ctx = Ctx::new();
        let mut ev = Event::Broadcast(crate::event::Message::with(cm::PROGRESS_SET, Info::Progress { id, value: 42 }));
        p.handle_event(&mut ev, &mut ctx);
        assert_eq!(p.value(), 42);
        assert!(ev.is_nothing());
    }
}
