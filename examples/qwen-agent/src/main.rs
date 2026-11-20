mod api;
mod fileio;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use tvision::json::{parse_json, Json};
use tvision::layout::{item, HBox, VBox};
use tvision::*;

const SEND: u16 = 2000;
const NEW: u16 = 2001;
const SETTINGS: u16 = 2002;
const RETRY: u16 = 2003;

#[derive(Clone)]
struct Config {
    base_url: String,
    model: String,
    key_env: String,
    system: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: std::env::var("QWEN_BASE_URL").unwrap_or_default(),
            model: std::env::var("QWEN_MODEL").unwrap_or_else(|_| "qwen-plus".into()),
            key_env: "DASHSCOPE_API_KEY".into(),
            system: "Ты полезный помощник. Отвечай на языке пользователя.".into(),
        }
    }
}

impl Config {
    fn validate(&self) -> Result<(), String> {
        let url = self.base_url.as_str();
        if !(url.starts_with("https://") || url.starts_with("http://127.0.0.1:") || url.starts_with("http://localhost:")) {
            return Err("Укажите HTTPS Base URL из Model Studio (для локального сервера допустим HTTP localhost).".into());
        }
        let authority = url.split("://").nth(1).unwrap_or("").split('/').next().unwrap_or("");
        if authority.is_empty() || authority.contains('@') || url.contains(['?', '#']) || url.chars().any(char::is_whitespace) {
            return Err("Base URL должен содержать адрес сервера без паролей, пробелов и параметров.".into());
        }
        if self.model.trim().is_empty() || self.key_env.is_empty() || !self.key_env.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
            return Err("Укажите модель и имя переменной окружения с ключом.".into());
        }
        Ok(())
    }

    fn json(&self) -> Json {
        Json::object(vec![
            ("base_url", Json::from_str(&self.base_url)),
            ("model", Json::from_str(&self.model)),
            ("key_env", Json::from_str(&self.key_env)),
            ("system", Json::from_str(&self.system)),
        ])
    }
}

#[derive(Clone)]
struct Message {
    role: String,
    text: String,
}

struct Chat {
    title: String,
    messages: Vec<Message>,
    draft: String,
}

impl Chat {
    fn new() -> Self {
        Self { title: "Новый чат".into(), messages: Vec::new(), draft: String::new() }
    }
}

struct Pending {
    chat: usize,
    result: Receiver<Result<String, String>>,
}

struct Agent {
    config: Config,
    chats: Vec<Chat>,
    current: usize,
    window: ViewId,
    chat_list: ViewId,
    transcript: ViewId,
    prompt: ViewId,
    feedback: ViewId,
    pending: Option<Pending>,
    rendered_width: i32,
    data_file: PathBuf,
}

fn obj_text(json: &Json, name: &str) -> Result<String, String> {
    json.get(name).and_then(Json::as_str).map(str::to_owned).ok_or_else(|| format!("Нет строкового поля {name} в файле чатов"))
}

impl Agent {
    fn build(app: &mut Program, data_file: PathBuf) -> Result<Self, String> {
        let (config, chats, current) = Self::load(&data_file)?;
        let mut window = Window::new(app.desktop().group.vs.extent(), "Qwen Agent", 0);
        window.flags = wf::MOVE | wf::GROW | wf::ZOOM;
        let chat_list = window.add(ListBox::new(Rect::default(), 1, None));
        let scroll = window.add(ScrollBar::new(Rect::new(0, 0, 1, 10)));
        let transcript = window.add(Terminal::new(Rect::default(), None, Some(scroll), 100_000));
        let prompt = window.add(Memo::new(Rect::default(), None, None));
        let feedback = window.add(StaticText::new(Rect::default(), "F4 — подключение, F2 — отправить, Ctrl+N — новый чат"));
        let send = window.add(Button::new(Rect::default(), "~О~тправить F2", SEND, bf::NORMAL));
        let new = window.add(Button::new(Rect::default(), "~Н~овый чат", NEW, bf::NORMAL));
        let settings = window.add(Button::new(Rect::default(), "Н~а~стройки", SETTINGS, bf::NORMAL));
        window.set_layout(
            VBox::new()
                .add(
                    item(
                        HBox::new()
                            .add(item(chat_list).fixed_width(22).expand())
                            .add(item(transcript).expand().stretch(1))
                            .add(item(scroll).fixed_width(1).expand()),
                    )
                    .expand()
                    .stretch(1),
                )
                .add(item(feedback).fixed_height(2))
                .add(item(prompt).expand().fixed_height(5))
                .add(HBox::new().spacing(1).add(send).add(new).add(settings)),
        );
        window.select(prompt);
        let window = app.insert_window(window);
        let mut agent = Self { config, chats, current, window, chat_list, transcript, prompt, feedback, pending: None, rendered_width: 0, data_file };
        agent.refresh_chats(app);
        agent.show_chat(app);
        Ok(agent)
    }

    fn load(path: &Path) -> Result<(Config, Vec<Chat>, usize), String> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((Config::default(), vec![Chat::new()], 0)),
            Err(e) => return Err(format!("Не удалось прочитать {}: {e}", path.display())),
        };
        let data = parse_json(&text).map_err(|e| format!("Не удалось прочитать чаты: {e}"))?;
        let settings = data.get("config").ok_or("Нет настроек в файле чатов")?;
        let config = Config {
            base_url: obj_text(settings, "base_url")?,
            model: obj_text(settings, "model")?,
            key_env: obj_text(settings, "key_env")?,
            system: obj_text(settings, "system")?,
        };
        let mut chats = Vec::new();
        for chat in data.get("chats").and_then(Json::as_array).ok_or("Нет массива чатов")? {
            let mut messages = Vec::new();
            for message in chat.get("messages").and_then(Json::as_array).ok_or("Нет сообщений чата")? {
                let role = obj_text(message, "role")?;
                if !matches!(role.as_str(), "user" | "assistant") {
                    return Err("Некорректная роль в файле чатов".into());
                }
                messages.push(Message { role, text: obj_text(message, "text")? });
            }
            chats.push(Chat { title: obj_text(chat, "title")?, messages, draft: obj_text(chat, "draft")? });
        }
        if chats.is_empty() {
            chats.push(Chat::new());
        }
        let current = data.get("current").and_then(Json::as_i64).unwrap_or(0).max(0) as usize;
        let current = current.min(chats.len() - 1);
        Ok((config, chats, current))
    }

    fn save(&self) -> Result<(), String> {
        let data = Json::object(vec![
            ("config", self.config.json()),
            ("current", Json::from_i64(self.current as i64)),
            (
                "chats",
                Json::Arr(
                    self.chats
                        .iter()
                        .map(|chat| {
                            Json::object(vec![
                                ("title", Json::from_str(&chat.title)),
                                ("draft", Json::from_str(&chat.draft)),
                                (
                                    "messages",
                                    Json::Arr(
                                        chat.messages
                                            .iter()
                                            .map(|m| Json::object(vec![("role", Json::from_str(&m.role)), ("text", Json::from_str(&m.text))]))
                                            .collect(),
                                    ),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
        ]);
        fileio::write_atomic(&self.data_file, &data.pretty()).map_err(|e| format!("Ошибка сохранения чатов: {e}"))
    }

    fn feedback(&self, app: &mut Program, text: &str) {
        if let Some(view) = app.find_mut::<StaticText>(self.feedback) {
            view.text = text.into();
        }
    }

    fn persist(&self, app: &mut Program) {
        if let Err(error) = self.save() {
            self.feedback(app, &error);
        }
    }

    fn draft(&mut self, app: &Program) {
        if let Some(input) = app.find::<Memo>(self.prompt) {
            self.chats[self.current].draft = input.editor.text();
        }
    }

    fn refresh_chats(&self, app: &mut Program) {
        if let Some(list) = app.find_mut::<ListBox>(self.chat_list) {
            list.set_items(self.chats.iter().map(|chat| chat.title.clone()).collect());
            list.focus_item(self.current);
        }
    }

    fn show_chat(&mut self, app: &mut Program) {
        if let Some(input) = app.find_mut::<Memo>(self.prompt) {
            input.editor.set_text(&self.chats[self.current].draft);
        }
        if let Some(terminal) = app.find_mut::<Terminal>(self.transcript) {
            self.rendered_width = terminal.vs.size.x;
            terminal.clear();
            for message in &self.chats[self.current].messages {
                terminal.write_line(if message.role == "user" { "ВЫ" } else { "QWEN" });
                for line in message.text.lines() {
                    // Терминальные управляющие символы не должны менять экран приложения.
                    let line: String = line.chars().map(|c| if c.is_control() { ' ' } else { c }).collect();
                    for wrapped in tvision::text::wrap_text(&line, terminal.vs.size.x.max(1) as usize) {
                        terminal.write_line(&wrapped);
                    }
                }
                terminal.write_line("");
            }
        }
    }

    fn settings(&mut self, app: &mut Program) {
        let mut dialog = Dialog::centered(76, 18, "Подключение Qwen / Model Studio");
        let mut fields = Vec::new();
        for (y, label, value) in [
            (2, "Base URL (без /chat/completions)", self.config.base_url.as_str()),
            (5, "Модель", self.config.model.as_str()),
            (8, "Переменная окружения с API-ключом", self.config.key_env.as_str()),
            (11, "Системная инструкция", self.config.system.as_str()),
        ] {
            let mut input = InputLine::new(Rect::new(2, y + 1, 74, y + 2), 2048);
            input.set_text(value);
            let id = dialog.add(input);
            dialog.add(Label::new(Rect::new(2, y, 74, y + 1), label, Some(id)));
            fields.push(id);
        }
        dialog.add(Button::new(Rect::new(20, 15, 36, 17), "Сохранить", cm::OK, bf::DEFAULT));
        dialog.add(Button::new(Rect::new(40, 15, 54, 17), "Отмена", cm::CANCEL, bf::NORMAL));
        dialog.select(fields[0]);
        let (cmd, dialog) = app.exec_dialog(dialog);
        if cmd != cm::OK {
            return;
        }
        let values: Vec<_> = fields.iter().map(|&id| dialog.win.group.find::<InputLine>(id).unwrap().text()).collect();
        let candidate =
            Config { base_url: values[0].trim().into(), model: values[1].trim().into(), key_env: values[2].trim().into(), system: values[3].clone() };
        if let Err(error) = candidate.validate() {
            self.feedback(app, &error);
            return;
        }
        self.config = candidate;
        self.feedback(app, "Настройки сохранены. API-ключ читается из окружения при отправке.");
        self.persist(app);
    }

    fn send(&mut self, app: &mut Program, retry: bool) {
        if self.pending.is_some() {
            self.feedback(app, "Дождитесь ответа на текущий запрос.");
            return;
        }
        if let Err(error) = self.config.validate() {
            self.feedback(app, &error);
            return;
        }
        let key = match std::env::var(&self.config.key_env) {
            Ok(key) if !key.is_empty() => key,
            _ => {
                self.feedback(app, &format!("Задайте {} перед запуском программы.", self.config.key_env));
                return;
            }
        };
        self.draft(app);
        let chat = &mut self.chats[self.current];
        if retry {
            if !chat.messages.last().is_some_and(|m| m.role == "user") {
                self.feedback(app, "Последний запрос уже получил ответ. Введите новый запрос.");
                return;
            }
        } else {
            let text = chat.draft.trim().to_owned();
            if text.is_empty() {
                return;
            }
            if chat.messages.last().is_some_and(|m| m.role == "user") {
                self.feedback(app, "Предыдущий запрос не получил ответ. F5 — повторить или создайте новый чат.");
                return;
            }
            if chat.messages.is_empty() {
                chat.title = text.chars().take(30).collect();
            }
            chat.messages.push(Message { role: "user".into(), text });
            chat.draft.clear();
        }
        let config = self.config.clone();
        let messages = chat.messages.clone();
        let (tx, result) = mpsc::channel();
        match std::thread::Builder::new().name("qwen-request".into()).spawn(move || {
            let _ = tx.send(api::request(&config, &messages, &key));
        }) {
            Ok(_) => self.pending = Some(Pending { chat: self.current, result }),
            Err(error) => self.feedback(app, &format!("Ошибка запуска запроса: {error}. F5 — повторить.")),
        }
        self.refresh_chats(app);
        self.show_chat(app);
        if self.pending.is_some() {
            self.feedback(app, "Qwen готовит ответ… Можно переключить чат. Тайм-аут 120 секунд.");
        }
        self.persist(app);
    }

    fn poll(&mut self, app: &mut Program) {
        let Some(pending) = &self.pending else { return };
        let result = match pending.result.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err("Рабочий поток завершился без ответа.".into()),
        };
        let chat = self.pending.take().unwrap().chat;
        self.draft(app);
        match result {
            Ok(text) => {
                self.chats[chat].messages.push(Message { role: "assistant".into(), text });
                self.feedback(app, &format!("Ответ получен в чате «{}».", self.chats[chat].title));
                if chat == self.current {
                    self.draft(app);
                    self.show_chat(app);
                }
            }
            Err(error) => self.feedback(app, &format!("{error} F5 — повторить запрос в исходном чате.")),
        }
        self.persist(app);
    }
}

impl AppHandler for Agent {
    fn handle_event(&mut self, app: &mut Program, event: &mut Event) {
        match event.command() {
            Some(SEND) => self.send(app, false),
            Some(RETRY) => self.send(app, true),
            Some(SETTINGS) => self.settings(app),
            Some(NEW) => {
                self.draft(app);
                self.chats.push(Chat::new());
                self.current = self.chats.len() - 1;
                self.refresh_chats(app);
                self.show_chat(app);
                app.focus(self.prompt);
                self.persist(app);
            }
            _ => return,
        }
        event.clear();
    }

    fn idle(&mut self, app: &mut Program) {
        let selected = app.find::<ListBox>(self.chat_list).map(|list| list.focused).unwrap_or(self.current);
        if selected != self.current && selected < self.chats.len() {
            self.draft(app);
            self.current = selected;
            self.show_chat(app);
            self.persist(app);
        }
        let width = app.find::<Terminal>(self.transcript).map(|view| view.vs.size.x).unwrap_or(0);
        if width != self.rendered_width {
            self.draft(app);
            self.show_chat(app);
        }
        self.poll(app);
        if let Some(window) = app.find_mut::<Window>(self.window) {
            window.title = format!("Qwen Agent — {}{}", self.config.model, if self.pending.is_some() { " • ожидание" } else { "" });
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    locale::set_language(locale::Language::Ru);
    let data_file = std::env::args_os().nth(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("qwen-chats.json"));
    let menu = Menu::new().submenu(
        "~Ч~ат",
        Menu::new()
            .item_key("~Н~овый", NEW, kb::ctrl('N'), "Ctrl+N")
            .item_key("~О~тправить", SEND, kb::F2, "F2")
            .item_key("~П~овторить запрос", RETRY, kb::f(5), "F5")
            .item_key("Н~а~стройки", SETTINGS, kb::f(4), "F4")
            .item_key("В~ы~ход", cm::QUIT, kb::ALT_X, "Alt+X"),
    );
    let status = vec![StatusDef::all(vec![
        StatusItem::new("~F2~ Отправить", kb::F2, SEND),
        StatusItem::new("~F4~ Настройки", kb::f(4), SETTINGS),
        StatusItem::new("~F5~ Повторить", kb::f(5), RETRY),
        StatusItem::new("~Ctrl+N~ Новый чат", kb::ctrl('N'), NEW),
        StatusItem::new("~Alt+X~ Выход", kb::ALT_X, cm::QUIT),
    ])];
    let mut app = Program::with_terminal(Some(menu), Some(status))?;
    let mut agent = Agent::build(&mut app, data_file)?;
    app.run(&mut agent);
    agent.draft(&app);
    agent.save()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tvision::platform::HeadlessBackend;

    #[test]
    fn asynchronous_reply_stays_in_original_chat_and_roundtrips() {
        let path = std::env::temp_dir().join(format!("qwen-agent-test-{}.json", std::process::id()));
        let mut app = Program::new(Box::new(HeadlessBackend::new(110, 35)), None, None);
        let mut agent = Agent::build(&mut app, path.clone()).unwrap();
        agent.chats[0].messages.push(Message { role: "user".into(), text: "Первый запрос".into() });
        let (sender, result) = mpsc::channel();
        agent.pending = Some(Pending { chat: 0, result });
        agent.handle_event(&mut app, &mut Event::command_event(NEW));
        app.find_mut::<Memo>(agent.prompt).unwrap().editor.set_text("Черновик другого чата");
        sender.send(Ok("Ответ Qwen".into())).unwrap();
        agent.poll(&mut app);
        assert_eq!(agent.current, 1);
        assert_eq!(agent.chats[0].messages[1].text, "Ответ Qwen");
        assert_eq!(agent.chats[1].draft, "Черновик другого чата");
        let (_, chats, current) = Agent::load(&path).unwrap();
        assert_eq!(current, 1);
        assert_eq!(chats[0].messages.len(), 2);
        assert_eq!(chats[1].draft, "Черновик другого чата");
        app.find_mut::<ListBox>(agent.chat_list).unwrap().focus_item(0);
        agent.idle(&mut app);
        app.redraw();
        assert!(app.screen_text().contains("Ответ Qwen"));
        let data = std::fs::read_to_string(&path).unwrap();
        assert!(!data.contains("Authorization"));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_unsafe_endpoint_and_keeps_broken_history() {
        let mut config = Config { base_url: "http://example.com/v1".into(), ..Config::default() };
        assert!(config.validate().is_err());
        config.base_url = "https://user:password@example.com/v1".into();
        assert!(config.validate().is_err());
        config.base_url = "https://example.com/compatible-mode/v1".into();
        assert!(config.validate().is_ok());
        let path = std::env::temp_dir().join(format!("qwen-agent-broken-{}.json", std::process::id()));
        std::fs::write(&path, "повреждено").unwrap();
        assert!(Agent::load(&path).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "повреждено");
        std::fs::remove_file(path).unwrap();
    }
}
