//! Команды среды и синхронизация палитры, дерева, холста и свойств.

use super::*;

impl AppHandler for Ide {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        let Some(cmd) = ev.command() else { return };
        match cmd {
            CM_NEW => {
                if self.confirm_discard(app) {
                    self.doc = FormDoc::new("Form1");
                    self.path = None;
                    self.sel = Sel::Form;
                    self.undo.clear();
                    self.redo.clear();
                    self.set_dirty(false);
                    self.refresh_all(app);
                }
            }
            CM_OPEN => {
                if self.confirm_discard(app) {
                    let (c, d) = app.exec_dialog(FileDialog::new("*.tvui", "Открыть форму", "~И~мя", fd::OPEN_BUTTON, 100));
                    if c != cm::CANCEL {
                        self.open_path(app, &d.file_name());
                    }
                }
            }
            CM_SAVE => {
                self.save(app);
            }
            CM_SAVE_AS => {
                self.save_as(app);
            }
            CM_GENERATE => {
                self.generate(app);
            }
            CM_EXIT => {
                if self.confirm_discard(app) {
                    self.set_dirty(false);
                    app.put_event(Event::command_event(cm::QUIT));
                }
            }
            CM_ADD => {
                let i = app.find::<ListBox>(self.ids.0).map(|l| l.focused).unwrap_or(0);
                self.add(app, i);
            }
            CM_DELETE => self.delete(app),
            CM_UP => self.shift(app, -1),
            CM_DOWN => self.shift(app, 1),
            CM_WRAP_H => self.wrap(app, LayoutKind::HBox),
            CM_WRAP_V => self.wrap(app, LayoutKind::VBox),
            CM_WRAP_GRID => self.wrap(app, LayoutKind::Grid),
            CM_WRAP_FORM => self.wrap(app, LayoutKind::Form),
            CM_UNDO => self.undo(app),
            CM_REDO => self.redo(app),
            CM_PROP_EDIT => self.edit_prop(app),
            CM_PREVIEW => self.preview(app),
            CM_CODE => self.show_code(app),
            CM_CHECK => {
                let p = self.doc.problems();
                if p.is_empty() {
                    self.info(app, "\x03Ошибок нет: код соберётся.");
                } else {
                    self.error(app, &p.join("\n"));
                }
            }
            CM_THEME_CHOOSE => {
                let list: Vec<Theme> = self.themes.iter().map(|e| e.theme.clone()).collect();
                if let Some(i) = choose_theme(app, &list) {
                    let e = self.themes[i].clone();
                    self.set_ide_theme(app, Some(&e));
                }
            }
            CM_THEME_DEFAULT => self.set_ide_theme(app, None),
            CM_THEME_OPEN => {
                let (c, d) = app.exec_dialog(FileDialog::new("*.tvtheme", "Открыть тему", "~И~мя", fd::OPEN_BUTTON, 102));
                if c != cm::CANCEL {
                    let p = d.file_name();
                    match Theme::load(&p) {
                        Ok(theme) => {
                            let i = self.add_theme(ThemeEntry::from_file(&p, theme));
                            let e = self.themes[i].clone();
                            self.set_ide_theme(app, Some(&e));
                        }
                        Err(e) => self.error(app, &e),
                    }
                }
            }
            CM_FORM_THEME_CHOOSE => self.choose_form_theme(app),
            CM_SETTINGS => self.settings_dialog(app),
            CM_FORM_THEME_SAME => self.set_form_theme(app, None),
            c if (CM_IDE_THEME..CM_IDE_THEME + MENU_THEMES as u16).contains(&c) => {
                if let Some(e) = self.themes.get((c - CM_IDE_THEME) as usize).cloned() {
                    self.set_ide_theme(app, Some(&e));
                }
            }
            c if (CM_FORM_THEME..CM_FORM_THEME + MENU_THEMES as u16).contains(&c) => {
                let e = self.themes.get((c - CM_FORM_THEME) as usize).cloned();
                if e.is_some() {
                    self.set_form_theme(app, e);
                }
            }
            CM_ABOUT => self.info(
                app,
                "\x03TVCreator\n\n\x03Дизайнер форм для tvision-rs\n\x03в духе Qt Creator.\n\n\x03Формы — файлы .tvui,\n\x03код — F9 или tvcreator --gen.",
            ),
            _ => return,
        }
        ev.clear();
    }

    fn idle(&mut self, app: &mut Program) {
        // Выбор мышью на холсте.
        let picked = app.find_mut::<Canvas>(self.ids.2).and_then(|c| c.picked.take());
        if let Some(s) = picked {
            if s != self.sel {
                self.sel = s;
                self.refresh_selection(app);
            }
        } else {
            // Курсор в дереве объектов.
            let tp = app.find::<TreeView>(self.ids.1).and_then(|t| t.cursor_path()).map(|p| p.to_vec());
            if tp.is_some() && tp != self.last_tree {
                self.last_tree = tp.clone();
                if let Some(s) = tp.as_deref().and_then(tree_to_sel) {
                    if s != self.sel {
                        self.sel = s;
                        self.refresh_selection(app);
                    }
                }
            }
        }
        let node_sel = matches!(&self.sel, Sel::Node(p) if !p.is_empty());
        for (c, on) in
            [(CM_DELETE, node_sel), (CM_UP, node_sel), (CM_DOWN, node_sel), (CM_UNDO, !self.undo.is_empty()), (CM_REDO, !self.redo.is_empty())]
        {
            if on {
                app.enable_command(c)
            } else {
                app.disable_command(c)
            }
        }
    }
}
