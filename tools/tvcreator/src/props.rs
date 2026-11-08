//! Свойства узлов для редактора свойств: что показать и как поменять.

use tvision::layout::Alignment;

use crate::model::*;

/// Что выбрано: сама форма или узел дерева.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sel {
    Form,
    Node(Vec<usize>),
}

/// Как редактируется свойство.
#[derive(Clone, Debug, PartialEq)]
pub enum PropKind {
    /// Строка.
    Text,
    /// Целое в пределах.
    Int(i64, i64),
    /// Целое или пусто (не задано).
    OptInt(i64, i64),
    /// Да/нет — Enter переключает.
    Bool,
    /// Один из вариантов — Enter выбирает следующий.
    Choice(Vec<&'static str>),
    /// Список через «;».
    List,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prop {
    pub key: &'static str,
    pub label: &'static str,
    pub value: String,
    pub kind: PropKind,
}

fn p(key: &'static str, label: &'static str, value: impl Into<String>, kind: PropKind) -> Prop {
    Prop { key, label, value: value.into(), kind }
}

fn yes(b: bool) -> &'static str {
    if b {
        "да"
    } else {
        "нет"
    }
}

const ALIGN: [&str; 4] = ["заполнить", "в начало", "по центру", "в конец"];

fn align_name(a: Alignment) -> &'static str {
    match a {
        Alignment::Fill => ALIGN[0],
        Alignment::Start => ALIGN[1],
        Alignment::Center => ALIGN[2],
        Alignment::End => ALIGN[3],
    }
}

fn align_of(s: &str) -> Option<Alignment> {
    [Alignment::Fill, Alignment::Start, Alignment::Center, Alignment::End].into_iter().zip(ALIGN).find(|(_, n)| *n == s).map(|(a, _)| a)
}

fn opt(v: Option<i32>) -> String {
    v.map(|x| x.to_string()).unwrap_or_default()
}

/// Свойства выбранного.
pub fn props(doc: &FormDoc, sel: &Sel) -> Vec<Prop> {
    let path = match sel {
        Sel::Form => {
            return vec![
                p("form.name", "Имя типа", &doc.name, PropKind::Text),
                p("form.title", "Заголовок", &doc.title, PropKind::Text),
                p("form.kind", "Вид", if doc.kind == FormKind::Dialog { "Dialog" } else { "Window" }, PropKind::Choice(vec!["Dialog", "Window"])),
                p("form.width", "Ширина", doc.size.0.to_string(), PropKind::Int(16, 250)),
                p("form.height", "Высота", doc.size.1.to_string(), PropKind::Int(6, 100)),
                p("form.auto_size", "Размер по содержимому", yes(doc.auto_size), PropKind::Bool),
                p("form.command_base", "Свои команды с номера", doc.command_base.to_string(), PropKind::Int(100, 250)),
            ];
        }
        Sel::Node(p) => p,
    };
    let Some(n) = doc.node(path) else { return vec![] };
    let mut out = Vec::new();
    match &n.kind {
        NodeKind::Layout { kind, spacing, margins, columns } => {
            out.push(p("layout.kind", "Раскладка", kind.rust(), PropKind::Choice(vec!["HBox", "VBox", "Grid", "Form"])));
            out.push(p("layout.spacing", "Промежуток (пусто — обычный)", opt(*spacing), PropKind::OptInt(0, 20)));
            out.push(p(
                "layout.margins",
                "Поля: слева, сверху, справа, снизу",
                margins.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(", "),
                PropKind::Text,
            ));
            if *kind == LayoutKind::Grid {
                out.push(p("layout.columns", "Столбцов (по порядку)", columns.to_string(), PropKind::Int(1, 20)));
            }
        }
        NodeKind::Widget(w) => {
            out.push(p("w.name", "Имя", &w.name, PropKind::Text));
            out.push(p("w.class", "Класс", w.kind.title(), PropKind::Choice(vec![])));
            match w.kind {
                WidgetKind::Label => {
                    out.push(p("w.text", "Текст (~буква~ — горячая)", &w.text, PropKind::Text));
                    out.push(p("w.link", "Подписывает (имя виджета)", &w.link, PropKind::Text));
                }
                WidgetKind::StaticText => out.push(p("w.text", "Текст (\\n — новая строка)", w.text.replace('\n', "\\n"), PropKind::Text)),
                WidgetKind::Button => {
                    out.push(p("w.text", "Надпись", &w.text, PropKind::Text));
                    out.push(p("w.command", "Команда (OK, CANCEL… или CM_ИМЯ)", &w.command, PropKind::Text));
                    out.push(p("w.default", "По умолчанию (Enter)", yes(w.default), PropKind::Bool));
                }
                WidgetKind::InputLine => out.push(p("w.max_len", "Длина", w.max_len.to_string(), PropKind::Int(1, 255))),
                WidgetKind::CheckBoxes | WidgetKind::RadioButtons | WidgetKind::ListBox => {
                    out.push(p("w.items", "Пункты (через ;)", w.items.join("; "), PropKind::List))
                }
                WidgetKind::Table => out.push(p("w.items", "Столбцы (через ;)", w.items.join("; "), PropKind::List)),
                WidgetKind::ProgressBar => out.push(p("w.max", "Максимум", w.max.to_string(), PropKind::Int(1, 1_000_000_000))),
            }
        }
        NodeKind::Stretch(f) => out.push(p("stretch.factor", "Коэффициент", f.to_string(), PropKind::Int(1, 100))),
        NodeKind::Space(s) => out.push(p("space.size", "Клеток", s.to_string(), PropKind::Int(0, 100))),
    }
    // Место в родительской раскладке.
    if !path.is_empty() && !matches!(n.kind, NodeKind::Stretch(_) | NodeKind::Space(_)) {
        let it = &n.item;
        out.push(p("item.stretch", "Растяжение", it.stretch.to_string(), PropKind::Int(0, 100)));
        out.push(p("item.expand", "Тянуться во все стороны", yes(it.expand), PropKind::Bool));
        out.push(p("item.align_h", "По горизонтали", align_name(it.align_h), PropKind::Choice(ALIGN.to_vec())));
        out.push(p("item.align_v", "По вертикали", align_name(it.align_v), PropKind::Choice(ALIGN.to_vec())));
        out.push(p("item.min_w", "Мин. ширина", opt(it.min_w), PropKind::OptInt(0, 250)));
        out.push(p("item.min_h", "Мин. высота", opt(it.min_h), PropKind::OptInt(0, 100)));
        out.push(p("item.fixed_w", "Ширина ровно", opt(it.fixed_w), PropKind::OptInt(0, 250)));
        out.push(p("item.fixed_h", "Высота ровно", opt(it.fixed_h), PropKind::OptInt(0, 100)));
    }
    if !path.is_empty() {
        let parent = doc.node(&path[..path.len() - 1]).and_then(|n| n.layout_kind());
        let it = &n.item;
        if parent == Some(LayoutKind::Grid) {
            out.push(p(
                "item.cell",
                "Клетка: строка, столбец (пусто — по порядку)",
                it.cell.map(|(r, c)| format!("{}, {}", r, c)).unwrap_or_default(),
                PropKind::Text,
            ));
            out.push(p("item.span", "Занимает: строк, столбцов", format!("{}, {}", it.span.0, it.span.1), PropKind::Text));
        }
        if parent == Some(LayoutKind::Form) {
            out.push(p("item.wide", "На всю ширину формы", yes(it.wide), PropKind::Bool));
        }
    }
    out
}

/// Следующее значение для Enter: «да» ↔ «нет», следующий вариант.
/// `None` — свойство правится вводом текста.
pub fn next_value(prop: &Prop) -> Option<String> {
    match &prop.kind {
        PropKind::Bool => Some(yes(prop.value != "да").to_string()),
        PropKind::Choice(v) if !v.is_empty() => {
            let i = v.iter().position(|x| *x == prop.value).map_or(0, |i| (i + 1) % v.len());
            Some(v[i].to_string())
        }
        _ => None,
    }
}

fn ints(s: &str, n: usize) -> Result<Vec<i64>, String> {
    let v: Result<Vec<i64>, _> = s.split(',').map(|x| x.trim().parse::<i64>()).collect();
    match v {
        Ok(v) if v.len() == n => Ok(v),
        _ => Err(format!("Нужно {} целых числа через запятую.", n)),
    }
}

fn int_in(s: &str, lo: i64, hi: i64) -> Result<i64, String> {
    match s.trim().parse::<i64>() {
        Ok(v) if (lo..=hi).contains(&v) => Ok(v),
        _ => Err(format!("Нужно целое число от {} до {}.", lo, hi)),
    }
}

fn opt_in(s: &str, lo: i64, hi: i64) -> Result<Option<i32>, String> {
    if s.trim().is_empty() {
        Ok(None)
    } else {
        int_in(s, lo, hi).map(|v| Some(v as i32))
    }
}

fn list(s: &str) -> Vec<String> {
    s.split(';').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
}

/// Поменять свойство. Ошибка — сообщение для человека; документ при
/// ошибке не меняется.
pub fn set_prop(doc: &mut FormDoc, sel: &Sel, key: &str, value: &str) -> Result<(), String> {
    let v = value;
    if let Sel::Form = sel {
        match key {
            "form.name" => {
                if !is_type_name(v.trim()) {
                    return Err("Имя типа — латиницей, с заглавной буквы, без пробелов: LoginDialog.".into());
                }
                doc.name = v.trim().to_string();
            }
            "form.title" => doc.title = v.to_string(),
            "form.kind" => doc.kind = if v == "Window" { FormKind::Window } else { FormKind::Dialog },
            "form.width" => doc.size.0 = int_in(v, 16, 250)? as i32,
            "form.height" => doc.size.1 = int_in(v, 6, 100)? as i32,
            "form.auto_size" => doc.auto_size = v == "да",
            "form.command_base" => doc.command_base = int_in(v, 100, 250)? as u16,
            _ => return Err(format!("Нет свойства {}.", key)),
        }
        return Ok(());
    }
    let Sel::Node(path) = sel else { unreachable!() };
    // Имя проверяем до заимствования узла: нужен весь документ.
    if key == "w.name" {
        let new = v.trim();
        let old = doc.node(path).and_then(|n| n.as_widget()).map(|w| w.name.clone()).ok_or("Это не виджет.")?;
        if new == old {
            return Ok(());
        }
        if !is_ident(new) {
            return Err("Имя — латиницей в нижнем регистре, цифры и _, не ключевое слово Rust: user_name.".into());
        }
        if doc.widgets().iter().any(|w| w.name == new) {
            return Err(format!("Имя «{}» уже занято.", new));
        }
        doc.rename(&old, new);
        return Ok(());
    }
    if key == "w.link" && !v.trim().is_empty() && !doc.widgets().iter().any(|w| w.name == v.trim() && w.kind != WidgetKind::Label) {
        return Err(format!("Нет виджета «{}» (подписывать можно всё, кроме подписей).", v.trim()));
    }
    if key == "w.command" && !BUILTIN_COMMANDS.iter().any(|b| b.0 == v.trim()) && !is_command_name(v.trim()) {
        return Err("Команда — OK, CANCEL, YES, NO, CLOSE, HELP или своя: CM_ и заглавные латинские буквы (CM_SAVE).".into());
    }
    let n = doc.node_mut(path).ok_or("Узел не найден.")?;
    match key {
        "layout.kind" => {
            if let NodeKind::Layout { kind, .. } = &mut n.kind {
                *kind = LayoutKind::from_key(v).ok_or("Нет такой раскладки.")?;
            }
        }
        "layout.spacing" => {
            if let NodeKind::Layout { spacing, .. } = &mut n.kind {
                *spacing = opt_in(v, 0, 20)?;
            }
        }
        "layout.margins" => {
            let m = ints(v, 4)?;
            if m.iter().any(|x| !(0..=20).contains(x)) {
                return Err("Поля — от 0 до 20.".into());
            }
            if let NodeKind::Layout { margins, .. } = &mut n.kind {
                *margins = [m[0] as i32, m[1] as i32, m[2] as i32, m[3] as i32];
            }
        }
        "layout.columns" => {
            if let NodeKind::Layout { columns, .. } = &mut n.kind {
                *columns = int_in(v, 1, 20)? as usize;
            }
        }
        "stretch.factor" => n.kind = NodeKind::Stretch(int_in(v, 1, 100)? as u16),
        "space.size" => n.kind = NodeKind::Space(int_in(v, 0, 100)? as i32),
        "item.stretch" => n.item.stretch = int_in(v, 0, 100)? as u16,
        "item.expand" => n.item.expand = v == "да",
        "item.align_h" => n.item.align_h = align_of(v).ok_or("Нет такого выравнивания.")?,
        "item.align_v" => n.item.align_v = align_of(v).ok_or("Нет такого выравнивания.")?,
        "item.min_w" => n.item.min_w = opt_in(v, 0, 250)?,
        "item.min_h" => n.item.min_h = opt_in(v, 0, 100)?,
        "item.fixed_w" => n.item.fixed_w = opt_in(v, 0, 250)?,
        "item.fixed_h" => n.item.fixed_h = opt_in(v, 0, 100)?,
        "item.cell" => {
            n.item.cell = if v.trim().is_empty() {
                None
            } else {
                let x = ints(v, 2)?;
                if x.iter().any(|a| !(0..=50).contains(a)) {
                    return Err("Строка и столбец — от 0 до 50.".into());
                }
                Some((x[0] as usize, x[1] as usize))
            }
        }
        "item.span" => {
            let x = ints(v, 2)?;
            if x.iter().any(|a| !(1..=20).contains(a)) {
                return Err("Строк и столбцов — от 1 до 20.".into());
            }
            n.item.span = (x[0] as usize, x[1] as usize);
        }
        "item.wide" => n.item.wide = v == "да",
        _ => {
            let w = n.as_widget_mut().ok_or("Это не виджет.")?;
            match key {
                "w.text" => w.text = if w.kind == WidgetKind::StaticText { v.replace("\\n", "\n") } else { v.to_string() },
                "w.link" => w.link = v.trim().to_string(),
                "w.command" => w.command = v.trim().to_string(),
                "w.default" => w.default = v == "да",
                "w.max_len" => w.max_len = int_in(v, 1, 255)? as usize,
                "w.items" => {
                    let l = list(v);
                    if l.is_empty() {
                        return Err("Нужен хотя бы один пункт.".into());
                    }
                    if l.len() > 32 && matches!(w.kind, WidgetKind::CheckBoxes | WidgetKind::RadioButtons) {
                        return Err("Не больше 32 пунктов.".into());
                    }
                    w.items = l;
                }
                "w.max" => w.max = int_in(v, 1, 1_000_000_000)?,
                "w.class" => {}
                _ => return Err(format!("Нет свойства {}.", key)),
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc() -> FormDoc {
        let mut d = FormDoc::new("T");
        let g = d.insert(&[], Node::layout(LayoutKind::Grid));
        d.insert(&g, Node::widget(Widget::new(WidgetKind::Button, "b")));
        let f = d.insert(&[], Node::layout(LayoutKind::Form));
        d.insert(&f, Node::widget(Widget::new(WidgetKind::Label, "l")));
        d.insert(&f, Node::widget(Widget::new(WidgetKind::InputLine, "i")));
        d
    }

    #[test]
    fn lists_depend_on_place() {
        let d = doc();
        let keys = |s: Sel| props(&d, &s).into_iter().map(|p| p.key).collect::<Vec<_>>();
        assert!(keys(Sel::Node(vec![0, 0])).contains(&"item.cell"), "в сетке — клетка");
        assert!(keys(Sel::Node(vec![1, 1])).contains(&"item.wide"), "в форме — на всю ширину");
        assert!(!keys(Sel::Node(vec![])).contains(&"item.stretch"), "у корня места в родителе нет");
        assert!(keys(Sel::Form).contains(&"form.title"));
    }

    #[test]
    fn set_and_validate() {
        let mut d = doc();
        let b = Sel::Node(vec![0, 0]);
        set_prop(&mut d, &b, "w.text", "~С~охранить").unwrap();
        set_prop(&mut d, &b, "w.command", "CM_SAVE").unwrap();
        assert!(set_prop(&mut d, &b, "w.command", "save").is_err());
        set_prop(&mut d, &b, "item.span", "1, 2").unwrap();
        assert_eq!(d.node(&[0, 0]).unwrap().item.span, (1, 2));
        assert!(set_prop(&mut d, &b, "item.span", "0, 2").is_err());
        // Переименование меняет и ссылки подписей.
        set_prop(&mut d, &Sel::Node(vec![1, 0]), "w.link", "i").unwrap();
        set_prop(&mut d, &Sel::Node(vec![1, 1]), "w.name", "login").unwrap();
        assert_eq!(d.node(&[1, 0]).unwrap().as_widget().unwrap().link, "login");
        assert!(set_prop(&mut d, &Sel::Node(vec![1, 1]), "w.name", "b").unwrap_err().contains("занято"));
        assert!(set_prop(&mut d, &Sel::Node(vec![1, 0]), "w.link", "nobody").is_err());
        // Числа и пусто.
        set_prop(&mut d, &b, "item.min_w", "12").unwrap();
        set_prop(&mut d, &b, "item.min_w", "").unwrap();
        assert_eq!(d.node(&[0, 0]).unwrap().item.min_w, None);
        assert!(set_prop(&mut d, &Sel::Form, "form.width", "5").is_err());
        set_prop(&mut d, &Sel::Node(vec![]), "layout.margins", "2, 1, 2, 0").unwrap();
        assert!(set_prop(&mut d, &Sel::Form, "form.name", "bad name").is_err());
    }

    #[test]
    fn enter_cycles() {
        let pr = Prop { key: "x", label: "", value: "да".into(), kind: PropKind::Bool };
        assert_eq!(next_value(&pr).as_deref(), Some("нет"));
        let pr = Prop { key: "x", label: "", value: "Grid".into(), kind: PropKind::Choice(vec!["HBox", "VBox", "Grid", "Form"]) };
        assert_eq!(next_value(&pr).as_deref(), Some("Form"));
        let pr = Prop { key: "x", label: "", value: "1".into(), kind: PropKind::Text };
        assert_eq!(next_value(&pr), None);
    }
}
