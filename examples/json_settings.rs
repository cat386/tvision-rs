//! Настройки программы в файле JSON: `tvision::json` напрямую, без MVC.
//!
//! Запуск:
//! ```text
//! cargo run --example json_settings                     # файл во временном каталоге
//! cargo run --example json_settings -- настройки.json
//! ```
//!
//! Что показывает (урок 12, рецепт 10):
//! * структура настроек ↔ JSON: чтение с значениями по умолчанию для
//!   отсутствующих ключей, запись с отступами;
//! * чужие ключи в файле не теряются (правится исходный объект);
//! * массив в JSON — список недавних файлов;
//! * диалог настроек и сохранение сразу после OK.
//!
//! Клавиши: F2 — настройки, F3 — «открыть файл» (пополняет список
//! недавних), Alt-X — выход.

use std::path::{Path, PathBuf};

use tvision::json::{parse_json, Json};
use tvision::*;

const CM_SETTINGS: u16 = 150;
const CM_OPEN_RECENT: u16 = 151;

/// Темы — номер в переключателях и строка в файле.
const THEMES: [&str; 3] = ["classic", "gruvbox", "dusk"];

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub user: String,
    pub theme: String,
    pub tab_size: i64,
    pub show_clock: bool,
    pub autosave: bool,
    pub recent: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { user: String::new(), theme: "classic".into(), tab_size: 4, show_clock: true, autosave: false, recent: Vec::new() }
    }
}

/// Сколько недавних файлов помнить.
const RECENT_MAX: usize = 5;

impl Settings {
    /// Из JSON: нет ключа или он не того вида — значение по умолчанию.
    pub fn from_json(j: &Json) -> Settings {
        let d = Settings::default();
        let s = |k: &str, def: &str| j.get(k).and_then(Json::as_str).unwrap_or(def).to_string();
        Settings {
            user: s("user", &d.user),
            theme: Some(s("theme", &d.theme)).filter(|t| THEMES.contains(&t.as_str())).unwrap_or(d.theme),
            tab_size: j.get("tab_size").and_then(Json::as_i64).filter(|n| (1..=16).contains(n)).unwrap_or(d.tab_size),
            show_clock: j.get("show_clock").and_then(Json::as_bool).unwrap_or(d.show_clock),
            autosave: j.get("autosave").and_then(Json::as_bool).unwrap_or(d.autosave),
            recent: j
                .get("recent")
                .and_then(Json::as_array)
                .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                .unwrap_or_default(),
        }
    }

    /// Записать поля в объект `into` (остальные ключи объекта остаются).
    pub fn write_json(&self, into: &mut Json) {
        if !matches!(into, Json::Obj(_)) {
            *into = Json::Obj(Vec::new());
        }
        into.set("user", Json::from_str(&self.user));
        into.set("theme", Json::from_str(&self.theme));
        into.set("tab_size", Json::from_i64(self.tab_size));
        into.set("show_clock", Json::Bool(self.show_clock));
        into.set("autosave", Json::Bool(self.autosave));
        into.set("recent", Json::Arr(self.recent.iter().map(|s| Json::from_str(s)).collect()));
    }

    /// Добавить файл в начало списка недавних (без повторов).
    pub fn push_recent(&mut self, file: &str) {
        self.recent.retain(|f| f != file);
        self.recent.insert(0, file.to_string());
        self.recent.truncate(RECENT_MAX);
    }

    /// Для диалога: имя, тема (номер), размер табуляции, флажки (маска).
    fn to_value(&self) -> Value {
        let theme = THEMES.iter().position(|t| *t == self.theme).unwrap_or(0) as u32;
        Value::List(vec![
            Value::Str(self.user.clone()),
            Value::U32(theme),
            Value::Str(self.tab_size.to_string()),
            Value::U32(self.show_clock as u32 | (self.autosave as u32) << 1),
        ])
    }

    fn apply_value(&mut self, v: &Value) {
        let l = v.as_list().unwrap_or(&[]);
        self.user = l.first().and_then(Value::as_str).unwrap_or("").trim().to_string();
        self.theme = THEMES[l.get(1).and_then(Value::as_u32).unwrap_or(0).min(2) as usize].to_string();
        self.tab_size = l.get(2).and_then(Value::as_str).and_then(|s| s.trim().parse().ok()).unwrap_or(4);
        let flags = l.get(3).and_then(Value::as_u32).unwrap_or(0);
        self.show_clock = flags & 1 != 0;
        self.autosave = flags & 2 != 0;
    }
}

/// Файл настроек: исходный JSON (чтобы не терять чужие ключи) и путь.
pub struct SettingsFile {
    pub path: PathBuf,
    pub raw: Json,
    pub settings: Settings,
}

impl SettingsFile {
    /// Прочитать; нет файла — настройки по умолчанию. Испорченный файл —
    /// ошибка с номером строки (файл не перезаписываем молча).
    pub fn load(path: &Path) -> Result<SettingsFile, String> {
        let raw = match std::fs::read_to_string(path) {
            Ok(t) => parse_json(&t).map_err(|e| format!("{}: {}", path.display(), e))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Json::Obj(Vec::new()),
            Err(e) => return Err(format!("{}: {}", path.display(), e)),
        };
        let settings = Settings::from_json(&raw);
        Ok(SettingsFile { path: path.to_path_buf(), raw, settings })
    }

    pub fn save(&mut self) -> Result<(), String> {
        self.settings.write_json(&mut self.raw);
        std::fs::write(&self.path, self.raw.pretty() + "\n").map_err(|e| format!("Не удалось записать {}:\n{}", self.path.display(), e))
    }
}

/// Диалог настроек.
fn settings_dialog(s: &Settings) -> Dialog {
    let mut d = Dialog::centered(52, 16, "Настройки");
    let user = d.add(InputLine::new(Rect::new(20, 2, 49, 3), 40));
    d.add(Label::new(Rect::new(2, 2, 19, 3), "~И~мя пользователя:", Some(user)));
    let theme = d.add(Cluster::radio_buttons(Rect::new(3, 5, 24, 8), &["~К~лассика", "~G~ruvbox", "~С~умерки"]));
    d.add(Label::new(Rect::new(2, 4, 12, 5), "~Т~ема", Some(theme)));
    let tab = d.add(InputLine::new(Rect::new(44, 5, 49, 6), 2).with_validator(RangeValidator::new(1, 16)));
    d.add(Label::new(Rect::new(27, 5, 43, 6), "~Р~азмер табуляции", Some(tab)));
    d.add(Cluster::check_boxes(Rect::new(27, 8, 49, 10), &["~Ч~асы", "~А~втосохранение"]));
    d.add(Button::new(Rect::new(12, 12, 24, 14), "O~K~", cm::OK, bf::DEFAULT));
    d.add(Button::new(Rect::new(27, 12, 39, 14), "Отмена", cm::CANCEL, bf::NORMAL));
    d.set_data(&s.to_value());
    d.select(user);
    d
}

/// Текст окна «Настройки сейчас».
fn describe(f: &SettingsFile) -> String {
    let s = &f.settings;
    let yes = |b: bool| if b { "да" } else { "нет" };
    let mut t = format!(
        "Файл: {}\n\nПользователь: {}\nТема: {}\nТабуляция: {}\nЧасы: {}\nАвтосохранение: {}\n\nНедавние файлы:\n",
        f.path.display(),
        if s.user.is_empty() { "—" } else { &s.user },
        s.theme,
        s.tab_size,
        yes(s.show_clock),
        yes(s.autosave)
    );
    if s.recent.is_empty() {
        t.push_str("  (пусто)\n");
    }
    for (i, r) in s.recent.iter().enumerate() {
        t.push_str(&format!("  {}. {}\n", i + 1, r));
    }
    t
}

struct App {
    file: SettingsFile,
    text: ViewId,
}

impl App {
    fn update_view(&self, app: &mut Program) {
        let s = describe(&self.file);
        if let Some(t) = app.find_mut::<StaticText>(self.text) {
            t.text = s;
        }
    }

    fn save(&mut self, app: &mut Program) {
        if let Err(e) = self.file.save() {
            message_box(app, &e, mf::ERROR | mf::OK_BUTTON);
        }
        self.update_view(app);
    }
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        match ev.command() {
            Some(CM_SETTINGS) => {
                let (cmd, d) = app.exec_dialog(settings_dialog(&self.file.settings));
                if cmd == cm::OK {
                    self.file.settings.apply_value(&d.get_data());
                    self.save(app);
                }
            }
            Some(CM_OPEN_RECENT) => {
                let (cmd, d) = app.exec_dialog(FileDialog::new("*.*", "Открыть", "~И~мя", fd::OPEN_BUTTON, 100));
                if cmd != cm::CANCEL {
                    self.file.settings.push_recent(&d.file_name().display().to_string());
                    self.save(app);
                }
            }
            _ => return,
        }
        ev.clear();
    }
}

fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![
        StatusItem::new("~F2~ Настройки", kb::F2, CM_SETTINGS),
        StatusItem::new("~F3~ Открыть файл", kb::F3, CM_OPEN_RECENT),
        StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT),
    ])]
}

fn start(app: &mut Program, file: SettingsFile) -> App {
    let mut w = Window::new(Rect::new(4, 1, 74, 20), "Настройки сейчас", 1);
    let mut t = StaticText::new(w.interior().grown(-1, 0), "");
    t.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
    let text = w.add(t);
    app.insert_window(w);
    let h = App { file, text };
    h.update_view(app);
    h
}

fn main() -> std::io::Result<()> {
    locale::set_language(locale::Language::Ru);
    let path = std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| std::env::temp_dir().join("tvision-settings.json"));
    let file = SettingsFile::load(&path).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1)
    });
    let mut app = Program::with_terminal(None, Some(status()))?;
    let mut h = start(&mut app, file);
    app.run(&mut h);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tvision::platform::HeadlessBackend;

    fn temp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("json-settings-{}-{}", std::process::id(), name));
        let _ = std::fs::remove_file(&p);
        p
    }

    #[test]
    fn defaults_and_bad_values() {
        let j = parse_json(r#"{"theme": "неон", "tab_size": 99, "show_clock": false, "recent": ["a", 5, "b"]}"#).unwrap();
        let s = Settings::from_json(&j);
        assert_eq!(s.theme, "classic", "неизвестная тема — по умолчанию");
        assert_eq!(s.tab_size, 4, "вне 1..16 — по умолчанию");
        assert!(!s.show_clock);
        assert_eq!(s.recent, ["a", "b"], "не-строки пропущены");
    }

    #[test]
    fn foreign_keys_survive_save() {
        let path = temp("a.json");
        std::fs::write(&path, r#"{"plugin": {"x": 1}, "user": "Аня"}"#).unwrap();
        let mut f = SettingsFile::load(&path).unwrap();
        assert_eq!(f.settings.user, "Аня");
        f.settings.push_recent("1.txt");
        f.settings.push_recent("2.txt");
        f.settings.push_recent("1.txt");
        f.save().unwrap();
        let back = parse_json(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(back.get("plugin").and_then(|p| p.get("x")).and_then(Json::as_i64), Some(1));
        assert_eq!(Settings::from_json(&back).recent, ["1.txt", "2.txt"]);
        std::fs::write(&path, "{\"user\": ").unwrap();
        assert!(SettingsFile::load(&path).err().unwrap().contains("строка 1"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn dialog_changes_and_saves() {
        let path = temp("b.json");
        // F2; имя «Боб»; Tab → тема, ↓ (Gruvbox); Enter — OK.
        let mut b = HeadlessBackend::new(80, 25);
        b.push_key(kb::F2);
        for c in "Боб".chars() {
            b.push_key(KeyEvent::new(Key::Char(c), Mods::NONE));
        }
        b.push_key(kb::key(Key::Tab));
        b.push_key(kb::key(Key::Down));
        b.push_key(kb::ENTER);
        let mut app = Program::new(Box::new(b), None, Some(status()));
        let mut h = start(&mut app, SettingsFile::load(&path).unwrap());
        app.pump(&mut h);
        app.redraw();
        assert!(app.screen_text().contains("Пользователь: Боб"), "{}", app.screen_text());
        let s = SettingsFile::load(&path).unwrap().settings;
        assert_eq!((s.user.as_str(), s.theme.as_str()), ("Боб", "gruvbox"));
        let _ = std::fs::remove_file(&path);
    }
}
