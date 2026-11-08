//! Холст дизайнера: собранная форма, выделение, выбор мышью.
//!
//! Виджеты формы — настоящие (их собирает [`crate::build`]), но события
//! им не передаются: щелчок только выбирает элемент, как в Qt Designer.

use tvision::layout::SizePolicy;
use tvision::view::default_handle_event;
use tvision::*;

use crate::build::Built;
use crate::props::Sel;

/// Где на холсте левый верхний угол формы.
pub const FORM_OFFSET: Point = Point::new(2, 1);

/// Цвет пометок выделения: жёлтый на красном.
const MARK: Attr = 0x4E;

pub struct Canvas {
    pub vs: ViewState,
    pub built: Option<Built>,
    /// Что выделено (рисуется пометками).
    pub selected: Sel,
    /// Выбор мышью, который ещё не забрала среда.
    pub picked: Option<Sel>,
    /// Тема формы: палитра приложения, в которой рисовать форму
    /// (`None` — как у самой среды) и её имя для подсказки.
    pub form_theme: Option<(String, Vec<Attr>)>,
}

impl Canvas {
    pub fn new() -> Self {
        let mut vs = ViewState::new(Rect::default());
        vs.options |= of::SELECTABLE | of::FIRST_CLICK;
        Canvas { vs, built: None, selected: Sel::Form, picked: None, form_theme: None }
    }

    /// Прямоугольник выделенного в координатах формы.
    pub fn selection_rect(&self) -> Option<Rect> {
        let b = self.built.as_ref()?;
        let size = b.group().vs.size;
        let inner = Rect::new(1, 1, size.x - 1, size.y - 1);
        match &self.selected {
            Sel::Form => None,
            Sel::Node(p) if p.is_empty() => Some(inner),
            Sel::Node(p) => {
                // Виджет — его границы; раскладка — все её виджеты вместе.
                let mut r: Option<Rect> = None;
                for (q, id) in &b.ids {
                    if q.starts_with(p) {
                        if let Some(v) = b.group().view(*id) {
                            let vb = v.vs().bounds();
                            r = Some(match r {
                                Some(x) => x.union(&vb),
                                None => vb,
                            });
                        }
                    }
                }
                r
            }
        }
    }

    /// Что под точкой (координаты холста).
    pub fn pick(&self, p: Point) -> Option<Sel> {
        let b = self.built.as_ref()?;
        let q = p - FORM_OFFSET;
        let size = b.group().vs.size;
        if !Rect::new(0, 0, size.x, size.y).contains(q) {
            return None;
        }
        // Сверху вниз: последний вставленный — верхний.
        for c in b.group().children.iter().rev() {
            if c.vs().bounds().contains(q) {
                if let Some(path) = b.path_of(c.vs().id) {
                    return Some(Sel::Node(path.to_vec()));
                }
            }
        }
        Some(Sel::Form)
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Canvas::new()
    }
}

impl View for Canvas {
    tvision::impl_view_basics!();

    fn size_policy(&self) -> SizePolicy {
        SizePolicy::EXPANDING
    }

    fn draw(&self, dc: &mut DrawCtx) {
        let size = self.vs.size;
        let bg = dc.map_color(4);
        dc.fill(Rect::new(0, 0, size.x, size.y), '░', bg);
        let Some(b) = &self.built else { return };
        let fs = b.group().vs.size;
        {
            let r = Rect::new(FORM_OFFSET.x, FORM_OFFSET.y, FORM_OFFSET.x + fs.x, FORM_OFFSET.y + fs.y);
            match &self.form_theme {
                Some((_, pal)) => {
                    let mut sub = dc.sub_themed(r, b.view.palette(), pal);
                    b.view.draw(&mut sub);
                }
                None => {
                    let mut sub = dc.sub_root(r, b.view.palette());
                    b.view.draw(&mut sub);
                }
            }
        }
        if let Some(r) = self.selection_rect() {
            let r = r.moved(FORM_OFFSET.x, FORM_OFFSET.y);
            for y in r.a.y..r.b.y.max(r.a.y + 1) {
                dc.write_char(r.a.x - 1, y, '►', MARK, 1);
                dc.write_char(r.b.x, y, '◄', MARK, 1);
            }
        }
        // Подсказка под формой.
        let y = FORM_OFFSET.y + fs.y + 1;
        if y < size.y {
            let hint = if b.ids.is_empty() {
                "Пустая форма: выберите элемент в палитре и нажмите Ins"
            } else {
                "Щелчок — выбрать элемент; свойства — справа"
            };
            let a = dc.map_color(6);
            dc.write_str(FORM_OFFSET.x, y, hint, a);
            let info = format!("{} × {}", fs.x, fs.y);
            dc.write_str(FORM_OFFSET.x + fs.x - info.chars().count() as i32, FORM_OFFSET.y + fs.y, &info, a);
            if let Some((name, _)) = &self.form_theme {
                dc.write_str(FORM_OFFSET.x, y + 1, &format!("Тема формы: {}", name), a);
            }
        }
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        default_handle_event(&self.vs, ev, ctx);
        if let Event::MouseDown(m) = ev {
            let p = self.vs.make_local(m.pos);
            if let Some(s) = self.pick(p) {
                self.selected = s.clone();
                self.picked = Some(s);
            }
            ev.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::build;
    use crate::model::Node;
    use crate::model::*;

    #[test]
    fn pick_and_highlight() {
        let mut d = FormDoc::new("T");
        let row = d.insert(&[], Node::layout(LayoutKind::HBox));
        d.insert(&row, Node::widget(Widget::new(WidgetKind::Button, "a")));
        d.insert(&row, Node::widget(Widget::new(WidgetKind::Button, "b")));
        let mut c = Canvas::new();
        c.built = Some(build(&d));
        let b_bounds = c.built.as_ref().unwrap().group().view(c.built.as_ref().unwrap().id_of(&[0, 1]).unwrap()).unwrap().vs().bounds();
        // Щелчок по второй кнопке.
        let at = b_bounds.a + FORM_OFFSET;
        assert_eq!(c.pick(at), Some(Sel::Node(vec![0, 1])));
        // По пустому месту формы — сама форма; мимо формы — ничего.
        assert_eq!(c.pick(FORM_OFFSET + Point::new(1, 1)), Some(Sel::Form));
        assert_eq!(c.pick(Point::new(0, 0)), None);
        // Выделение раскладки — обе кнопки.
        c.selected = Sel::Node(vec![0]);
        let r = c.selection_rect().unwrap();
        assert_eq!(r.b, b_bounds.b);
    }
}
