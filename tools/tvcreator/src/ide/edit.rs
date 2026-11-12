//! Правка формы: добавить, удалить, переставить, обернуть, отмена, свойства.

use super::*;

impl Ide {
    // ---------- правка ----------

    /// Добавить пункт палитры номер `i` у выделенного.
    pub fn add(&mut self, app: &mut Program, i: usize) {
        let Some(mut node) = palette_node(&self.doc, i) else { return };
        self.checkpoint();
        let at: Vec<usize> = match &self.sel {
            Sel::Form => vec![],
            Sel::Node(p) => p.clone(),
        };
        // Подпись перед полем подписывает его сама (как «buddy» в Qt).
        let new_name = node.as_widget().map(|w| (w.name.clone(), w.kind));
        let path = self.doc.insert(&at, std::mem::replace(&mut node, Node::space(0)));
        if let (Some((name, kind)), Some(&idx)) = (new_name, path.last()) {
            if kind != WidgetKind::Label && idx > 0 {
                let mut prev = path.clone();
                *prev.last_mut().unwrap() = idx - 1;
                if let Some(w) = self.doc.node_mut(&prev).and_then(|n| n.as_widget_mut()) {
                    if w.kind == WidgetKind::Label && w.link.is_empty() {
                        w.link = name;
                    }
                }
            }
        }
        self.sel = Sel::Node(path);
        self.changed(app);
    }

    pub fn delete(&mut self, app: &mut Program) {
        let Sel::Node(p) = self.sel.clone() else { return };
        if p.is_empty() {
            return;
        }
        self.checkpoint();
        let next = self.doc.remove(&p);
        // Подписи, чей виджет удалён, больше ничего не подписывают.
        let names: Vec<String> = self.doc.widgets().iter().map(|w| w.name.clone()).collect();
        let paths: Vec<Vec<usize>> = self.doc.walk().into_iter().map(|(p, _)| p).collect();
        for q in paths {
            if let Some(w) = self.doc.node_mut(&q).and_then(|n| n.as_widget_mut()) {
                if !w.link.is_empty() && !names.contains(&w.link) {
                    w.link.clear();
                }
            }
        }
        self.sel = next.map(Sel::Node).unwrap_or(Sel::Form);
        self.changed(app);
    }

    pub fn shift(&mut self, app: &mut Program, delta: i32) {
        let Sel::Node(p) = self.sel.clone() else { return };
        let before = self.doc.clone();
        if let Some(np) = self.doc.shift(&p, delta) {
            self.undo.push((before, self.sel.clone()));
            self.redo.clear();
            self.sel = Sel::Node(np);
            self.changed(app);
        }
    }

    pub fn wrap(&mut self, app: &mut Program, kind: LayoutKind) {
        self.checkpoint();
        let p = match &self.sel {
            Sel::Form => vec![],
            Sel::Node(p) => p.clone(),
        };
        let np = self.doc.wrap(&p, kind);
        self.sel = Sel::Node(np);
        self.changed(app);
    }

    pub fn undo(&mut self, app: &mut Program) {
        if let Some((d, s)) = self.undo.pop() {
            self.redo.push((std::mem::replace(&mut self.doc, d), std::mem::replace(&mut self.sel, s)));
            self.changed(app);
        }
    }

    pub fn redo(&mut self, app: &mut Program) {
        if let Some((d, s)) = self.redo.pop() {
            self.undo.push((std::mem::replace(&mut self.doc, d), std::mem::replace(&mut self.sel, s)));
            self.changed(app);
        }
    }

    /// Изменить свойство (без диалога — для тестов и Enter на флажках).
    pub fn apply_prop(&mut self, app: &mut Program, key: &str, value: &str) -> Result<(), String> {
        let before = self.doc.clone();
        match set_prop(&mut self.doc, &self.sel, key, value) {
            Ok(()) => {
                if self.doc != before {
                    self.undo.push((before, self.sel.clone()));
                    self.redo.clear();
                    self.changed(app);
                }
                Ok(())
            }
            Err(e) => {
                self.doc = before;
                Err(e)
            }
        }
    }

    /// Enter в свойствах: флажок и выбор — следующее значение, остальное —
    /// окно ввода.
    pub fn edit_prop(&mut self, app: &mut Program) {
        let Some(p) = self.current_prop(app) else { return };
        if matches!(&p.kind, crate::props::PropKind::Choice(v) if v.is_empty()) {
            return; // только для чтения (класс)
        }
        let value = match next_value(&p) {
            Some(v) => v,
            None => match input_box(app, "Свойство", &format!("~{}~:", p.label), &p.value, 250) {
                Some(v) => v,
                None => return,
            },
        };
        if let Err(e) = self.apply_prop(app, p.key, &value) {
            self.error(app, &e);
        }
    }
}
