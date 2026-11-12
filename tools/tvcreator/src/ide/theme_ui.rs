//! Темы среды и формы, стандартный диалог настроек.

use super::*;

impl Ide {
    // ---------- темы ----------

    pub(super) fn save_settings(&mut self) {
        if let Some(p) = &self.settings_path {
            let _ = self.settings.save(p); // не записалось — просто не запомнится
        }
    }

    /// Тема среды (`None` — цвета по умолчанию).
    pub fn set_ide_theme(&mut self, app: &mut Program, e: Option<&ThemeEntry>) {
        match e {
            Some(e) => {
                app.set_theme(&e.theme);
                self.settings.theme = e.key.clone();
            }
            None => {
                app.palette = standard_palette();
                self.settings.theme.clear();
            }
        }
        self.save_settings();
    }

    /// Тема формы на холсте и в предпросмотре (`None` — как у среды).
    pub fn set_form_theme(&mut self, app: &mut Program, e: Option<ThemeEntry>) {
        let key = e.as_ref().map(|e| e.key.clone()).unwrap_or_default();
        self.settings.set(FORM_THEME_KEY, Json::from_str(&key));
        if let Some(c) = app.find_mut::<Canvas>(self.ids.2) {
            c.form_theme = e.map(|e| (e.theme.name.clone(), e.theme.palette.clone()));
        }
        self.save_settings();
    }

    /// Стандартный диалог «Настройки» (темы — те же, что в меню «Тема»).
    pub fn settings_dialog(&mut self, app: &mut Program) {
        let list: Vec<(String, Theme)> = self.themes.iter().map(|e| (e.key.clone(), e.theme.clone())).collect();
        if settings_dialog_with(app, &mut self.settings, &list) {
            if let Some(p) = &self.settings_path {
                if let Err(e) = self.settings.save(p) {
                    self.error(app, &e);
                }
            }
        }
    }

    /// Запомнить, где стоит окно среды, и записать настройки (при выходе).
    pub fn save_on_exit(&mut self, app: &Program) {
        self.settings.capture_windows_with(app, window_key);
        self.save_settings();
    }

    /// Палитра темы формы, если она выбрана.
    pub fn form_palette(&self, app: &Program) -> Option<Vec<Attr>> {
        app.find::<Canvas>(self.ids.2).and_then(|c| c.form_theme.as_ref().map(|(_, p)| p.clone()))
    }

    pub(super) fn add_theme(&mut self, e: ThemeEntry) -> usize {
        match self.themes.iter().position(|x| x.key == e.key) {
            Some(i) => i,
            None => {
                self.themes.push(e);
                self.themes.len() - 1
            }
        }
    }

    /// Выбор темы формы: как `choose_theme`, но перекрашивается только
    /// холст. Первый пункт — «как у среды».
    pub fn choose_form_theme(&mut self, app: &mut Program) {
        let mut d = Dialog::centered(46, 16, "Тема формы");
        let mut list = ListBox::new(Rect::default(), 1, None);
        let mut names = vec!["(как у среды)".to_string()];
        names.extend(self.themes.iter().map(|e| e.theme.name.clone()));
        list.set_items(names);
        let current = self.form_palette(app);
        list.focused = current.as_ref().and_then(|p| self.themes.iter().position(|e| &e.theme.palette == p)).map_or(0, |i| i + 1);
        let list_id = d.add(list);
        let note = d.add(StaticText::new(
            Rect::default(),
            "Форма на холсте перекрашивается сразу;
цвета среды не меняются.",
        ));
        let ok = d.add(Button::new(Rect::default(), "O~K~", cm::OK, bf::DEFAULT));
        let cancel = d.add(Button::new(Rect::default(), "Отмена", cm::CANCEL, bf::NORMAL));
        d.set_layout(
            VBox::new()
                .margins(2, 1, 2, 0)
                .add(item(list_id).min(30, 8).stretch(1))
                .space(1)
                .add(note)
                .space(1)
                .add(HBox::new().stretch(1).add(ok).add(cancel)),
        );
        // Справа — чтобы не закрывать холст.
        d.win.group.vs.options &= !of::CENTERED;
        let ext = app.desktop().group.vs.extent();
        d.win.group.vs.origin = Point::new((ext.b.x - 48).max(0), 2);
        d.select(list_id);
        let canvas = self.ids.2;
        let themes = self.themes.clone();
        let before = app.find::<Canvas>(canvas).and_then(|c| c.form_theme.clone());
        let mut shown = usize::MAX;
        let (cmd, d) = app.exec_dialog_idle(d, |app, _| {
            let i = app.find::<ListBox>(list_id).map(|l| l.focused).unwrap_or(0);
            if i != shown {
                shown = i;
                if let Some(c) = app.find_mut::<Canvas>(canvas) {
                    c.form_theme = i.checked_sub(1).and_then(|k| themes.get(k)).map(|e| (e.theme.name.clone(), e.theme.palette.clone()));
                }
            }
            None
        });
        let i = d.win.group.find::<ListBox>(list_id).map(|l| l.focused).unwrap_or(0);
        if cmd == cm::OK {
            let e = i.checked_sub(1).and_then(|k| self.themes.get(k)).cloned();
            self.set_form_theme(app, e);
        } else if let Some(c) = app.find_mut::<Canvas>(canvas) {
            c.form_theme = before;
        }
    }
}
