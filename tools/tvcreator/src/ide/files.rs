//! Файлы формы, генерация кода, предпросмотр.

use super::*;

impl Ide {
    // ---------- файлы ----------

    /// Несохранённые изменения: спросить. `true` — можно продолжать.
    pub(super) fn confirm_discard(&mut self, app: &mut Program) -> bool {
        if !self.is_dirty() {
            return true;
        }
        match message_box(app, &format!("\x03Форма «{}» изменена.\n\x03Сохранить?", self.file_name()), mf::CONFIRMATION | mf::YES_NO_CANCEL)
        {
            c if c == cm::YES => self.save(app),
            c if c == cm::NO => true,
            _ => false,
        }
    }

    pub fn save_to(&mut self, app: &mut Program, path: &Path) -> bool {
        match self.doc.save(path) {
            Ok(()) => {
                self.path = Some(path.to_path_buf());
                self.set_dirty(false);
                self.refresh_all(app);
                true
            }
            Err(e) => {
                self.error(app, &e);
                false
            }
        }
    }

    pub fn save(&mut self, app: &mut Program) -> bool {
        match self.path.clone() {
            Some(p) => self.save_to(app, &p),
            None => self.save_as(app),
        }
    }

    pub fn save_as(&mut self, app: &mut Program) -> bool {
        let (cmd, d) = app.exec_dialog(FileDialog::new("*.tvui", "Сохранить форму", "~И~мя", fd::OK_BUTTON, 101));
        if cmd == cm::CANCEL {
            return false;
        }
        let mut p = d.file_name();
        if p.extension().is_none() {
            p.set_extension("tvui");
        }
        self.save_to(app, &p)
    }

    pub fn open_path(&mut self, app: &mut Program, path: &Path) {
        match FormDoc::load(path) {
            Ok(doc) => {
                self.doc = doc;
                self.path = Some(path.to_path_buf());
                self.sel = Sel::Form;
                self.undo.clear();
                self.redo.clear();
                self.set_dirty(false);
                self.refresh_all(app);
            }
            Err(e) => self.error(app, &e),
        }
    }

    /// Записать код рядом с формой. Возвращает путь файла.
    pub fn generate(&mut self, app: &mut Program) -> Option<PathBuf> {
        let problems = self.doc.problems();
        if !problems.is_empty() {
            self.error(app, &format!("Код не сгенерирован:\n{}", problems.join("\n")));
            return None;
        }
        if self.path.is_none() && !self.save_as(app) {
            return None;
        }
        let form = self.path.clone()?;
        let out = codegen::output_path(&form, &self.doc);
        let code = codegen::generate(&self.doc, &self.file_name());
        match std::fs::write(&out, code) {
            Ok(()) => {
                self.info(app, &format!("Код записан:\n{}", out.display()));
                Some(out)
            }
            Err(e) => {
                self.error(app, &format!("{}: {}", out.display(), e));
                None
            }
        }
    }

    pub(super) fn show_code(&mut self, app: &mut Program) {
        let code = codegen::generate(&self.doc, &self.file_name());
        let r = app.desktop().group.vs.extent().grown(-4, -2);
        let mut w = Window::new(r, &format!("Код: {}.rs", codegen::module_name(&self.doc.name)), 0);
        let v = w.standard_scroll_bar(true);
        let h = w.standard_scroll_bar(false);
        let mut e = Editor::new(w.interior(), Some(h), Some(v), None);
        e.set_text(&code);
        e.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
        w.add(e);
        app.insert_window(w);
    }

    pub(super) fn preview(&mut self, app: &mut Program) {
        let theme = self.form_palette(app);
        // В теме формы предпросмотр модальный (палитра — на всю программу
        // и возвращается после закрытия), поэтому окно показывается как диалог.
        let mut doc = self.doc.clone();
        if theme.is_some() {
            doc.kind = FormKind::Dialog;
        }
        let built = build(&doc);
        match built.view.into_any().downcast::<Dialog>() {
            Ok(d) => {
                let saved = theme.map(|p| std::mem::replace(&mut app.palette, p));
                app.exec_dialog(*d);
                if let Some(p) = saved {
                    app.palette = p;
                }
            }
            Err(any) => {
                if let Ok(mut w) = any.downcast::<Window>() {
                    w.title = format!("Предпросмотр: {}", w.title);
                    w.group.vs.origin = Point::new(2, 1);
                    app.insert_window(*w);
                }
            }
        }
    }
}
