mod api;
mod fileio;

use std::path::{Path, PathBuf};
use std::sync::mpsc::TryRecvError;
use tvision::json::{parse_json, Json};
use tvision::layout::{item, HBox, VBox};
use tvision::*;

const SEND: u16 = 2000;
const NEW: u16 = 2001;
const SETTINGS: u16 = 2002;
const RETRY: u16 = 2003;
const STOP: u16 = 2004;
const WORKSPACE: u16 = 2005;
const JOURNAL: u16 = 2006;

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
        if !(url.starts_with("https://")
            || url.starts_with("http://127.0.0.1:")
            || url.starts_with("http://localhost:"))
        {
            return Err("Укажите HTTPS Base URL из Model Studio (для локального сервера допустим HTTP localhost).".into());
        }
        let authority = url
            .split("://")
            .nth(1)
            .unwrap_or("")
            .split('/')
            .next()
            .unwrap_or("");
        if authority.is_empty()
            || authority.contains('@')
            || url.contains(['?', '#'])
            || url.chars().any(char::is_whitespace)
        {
            return Err(
                "Base URL должен содержать адрес сервера без паролей, пробелов и параметров."
                    .into(),
            );
        }
        if self.model.trim().is_empty()
            || self.key_env.is_empty()
            || !self
                .key_env
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
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
        Self {
            title: "Новый чат".into(),
            messages: Vec::new(),
            draft: String::new(),
        }
    }
}

struct Pending {
    chat: usize,
    journal: bool,
    worker: api::Worker,
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
    workspace: PathBuf,
    data_file: PathBuf,
}

fn obj_text(json: &Json, name: &str) -> Result<String, String> {
    json.get(name)
        .and_then(Json::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("Нет строкового поля {name} в файле чатов"))
}

impl Agent {
    fn build(app: &mut Program, data_file: PathBuf) -> Result<Self, String> {
        let (config, chats, current) = Self::load(&data_file)?;
        let mut window = Window::new(app.desktop().group.vs.extent(), "Agent Harness", 0);
        window.flags = wf::MOVE | wf::GROW | wf::ZOOM;
        let chat_list = window.add(ListBox::new(Rect::default(), 1, None));
        let scroll = window.add(ScrollBar::new(Rect::new(0, 0, 1, 10)));
        let transcript = window.add(Terminal::new(Rect::default(), None, Some(scroll), 100_000));
        let prompt = window.add(Memo::new(Rect::default(), None, None));
        let feedback = window.add(StaticText::new(
            Rect::default(),
            "F4 — подключение, F2 — отправить, Ctrl+N — новый чат",
        ));
        let send = window.add(Button::new(
            Rect::default(),
            "~О~тправить F2",
            SEND,
            bf::NORMAL,
        ));
        let new = window.add(Button::new(Rect::default(), "~Н~овый чат", NEW, bf::NORMAL));
        let settings = window.add(Button::new(
            Rect::default(),
            "Н~а~стройки",
            SETTINGS,
            bf::NORMAL,
        ));
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
        let mut agent = Self {
            config,
            chats,
            current,
            window,
            chat_list,
            transcript,
            prompt,
            feedback,
            pending: None,
            rendered_width: 0,
            workspace: std::env::current_dir().map_err(|e| e.to_string())?,
            data_file,
        };
        if let Ok(text) = std::fs::read_to_string(&agent.data_file) {
            if let Ok(data) = parse_json(&text) {
                if let Some(path) = data.get("workspace").and_then(Json::as_str) {
                    if let Ok(path) = std::fs::canonicalize(path) {
                        if path.is_dir() {
                            agent.workspace = path;
                        }
                    }
                }
            }
        }
        agent.refresh_chats(app);
        agent.show_chat(app);
        Ok(agent)
    }

    fn load(path: &Path) -> Result<(Config, Vec<Chat>, usize), String> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok((Config::default(), vec![Chat::new()], 0))
            }
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
        for chat in data
            .get("chats")
            .and_then(Json::as_array)
            .ok_or("Нет массива чатов")?
        {
            let mut messages = Vec::new();
            for message in chat
                .get("messages")
                .and_then(Json::as_array)
                .ok_or("Нет сообщений чата")?
            {
                let role = obj_text(message, "role")?;
                if !matches!(role.as_str(), "user" | "assistant" | "event") {
                    return Err("Некорректная роль в файле чатов".into());
                }
                messages.push(Message {
                    role,
                    text: obj_text(message, "text")?,
                });
            }
            chats.push(Chat {
                title: obj_text(chat, "title")?,
                messages,
                draft: obj_text(chat, "draft")?,
            });
        }
        if chats.is_empty() {
            chats.push(Chat::new());
        }
        let current = data
            .get("current")
            .and_then(Json::as_i64)
            .unwrap_or(0)
            .max(0) as usize;
        let current = current.min(chats.len() - 1);
        Ok((config, chats, current))
    }

    fn save(&self) -> Result<(), String> {
        let data = Json::object(vec![
            ("config", self.config.json()),
            (
                "workspace",
                Json::from_str(&self.workspace.to_string_lossy()),
            ),
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
                                            .map(|m| {
                                                Json::object(vec![
                                                    ("role", Json::from_str(&m.role)),
                                                    ("text", Json::from_str(&m.text)),
                                                ])
                                            })
                                            .collect(),
                                    ),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
        ]);
        fileio::write_atomic(&self.data_file, &data.pretty())
            .map_err(|e| format!("Ошибка сохранения чатов: {e}"))
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
                terminal.write_line(match message.role.as_str() {
                    "user" => "ВЫ",
                    "event" => "ИНСТРУМЕНТ",
                    _ => "АГЕНТ",
                });
                for line in message.text.lines() {
                    // Терминальные управляющие символы не должны менять экран приложения.
                    let line: String = line
                        .chars()
                        .map(|c| if c.is_control() { ' ' } else { c })
                        .collect();
                    for wrapped in
                        tvision::text::wrap_text(&line, terminal.vs.size.x.max(1) as usize)
                    {
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
            (
                2,
                "Base URL (без /chat/completions)",
                self.config.base_url.as_str(),
            ),
            (5, "Модель", self.config.model.as_str()),
            (
                8,
                "Переменная окружения с API-ключом",
                self.config.key_env.as_str(),
            ),
            (11, "Системная инструкция", self.config.system.as_str()),
        ] {
            let mut input = InputLine::new(Rect::new(2, y + 1, 74, y + 2), 2048);
            input.set_text(value);
            let id = dialog.add(input);
            dialog.add(Label::new(Rect::new(2, y, 74, y + 1), label, Some(id)));
            fields.push(id);
        }
        dialog.add(Button::new(
            Rect::new(20, 15, 36, 17),
            "Сохранить",
            cm::OK,
            bf::DEFAULT,
        ));
        dialog.add(Button::new(
            Rect::new(40, 15, 54, 17),
            "Отмена",
            cm::CANCEL,
            bf::NORMAL,
        ));
        dialog.select(fields[0]);
        let (cmd, dialog) = app.exec_dialog(dialog);
        if cmd != cm::OK {
            return;
        }
        let values: Vec<_> = fields
            .iter()
            .map(|&id| dialog.win.group.find::<InputLine>(id).unwrap().text())
            .collect();
        let candidate = Config {
            base_url: values[0].trim().into(),
            model: values[1].trim().into(),
            key_env: values[2].trim().into(),
            system: values[3].clone(),
        };
        if let Err(error) = candidate.validate() {
            self.feedback(app, &error);
            return;
        }
        self.config = candidate;
        self.feedback(
            app,
            "Настройки сохранены. API-ключ читается из окружения при отправке.",
        );
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
                self.feedback(
                    app,
                    &format!("Задайте {} перед запуском программы.", self.config.key_env),
                );
                return;
            }
        };
        self.draft(app);
        let chat = &mut self.chats[self.current];
        if retry {
            if !chat
                .messages
                .iter()
                .rev()
                .find(|m| m.role != "event")
                .is_some_and(|m| m.role == "user")
            {
                self.feedback(
                    app,
                    "Последний запрос уже получил ответ. Введите новый запрос.",
                );
                return;
            }
        } else {
            let text = chat.draft.trim().to_owned();
            if text.is_empty() {
                return;
            }
            if chat
                .messages
                .iter()
                .rev()
                .find(|m| m.role != "event")
                .is_some_and(|m| m.role == "user")
            {
                self.feedback(
                    app,
                    "Предыдущий запрос не получил ответ. F5 — повторить или создайте новый чат.",
                );
                return;
            }
            if chat.messages.is_empty() {
                chat.title = text.chars().take(30).collect();
            }
            chat.messages.push(Message {
                role: "user".into(),
                text,
            });
            chat.draft.clear();
        }
        let messages = chat.messages.clone();
        match api::start(
            &self.config,
            &messages,
            &key,
            &self.workspace,
            &self.data_file.with_extension("runs.sqlite"),
            api::Mode::Run { resume: retry },
            self.current,
        ) {
            Ok(worker) => {
                self.pending = Some(Pending {
                    chat: self.current,
                    journal: false,
                    worker,
                })
            }
            Err(error) => self.feedback(app, &error),
        }
        self.refresh_chats(app);
        self.show_chat(app);
        if self.pending.is_some() {
            self.feedback(app, "Агент выполняет задачу. F8 — остановить, F7 — журнал.");
        }
        self.persist(app);
    }

    fn poll(&mut self, app: &mut Program) {
        let Some(pending) = &self.pending else { return };
        let chat = pending.chat;
        let journal = pending.journal;
        let result = match pending.worker.events.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => {
                api::Event::Done(Err("Исполнитель завершился без ответа.".into()))
            }
        };
        self.draft(app);
        match result {
            api::Event::Progress(text) => {
                self.chats[chat].messages.push(Message {
                    role: "event".into(),
                    text: text.clone(),
                });
                self.feedback(app, &text);
            }
            api::Event::Approval {
                name,
                arguments,
                answer,
            } => {
                let allow = approval(app, &name, &arguments, &self.workspace);
                let _ = answer.send(allow);
                self.chats[chat].messages.push(Message {
                    role: "event".into(),
                    text: format!(
                        "{name}: {}",
                        if allow {
                            "разрешено"
                        } else {
                            "отклонено"
                        }
                    ),
                });
            }
            api::Event::Done(result) => {
                self.pending.take();
                match result {
                    Ok(text) => {
                        self.chats[chat].messages.push(Message {
                            role: if journal { "event" } else { "assistant" }.into(),
                            text,
                        });
                        self.feedback(app, "Задача завершена.");
                    }
                    Err(error) => {
                        self.chats[chat].messages.push(Message {
                            role: "event".into(),
                            text: error.clone(),
                        });
                        self.feedback(app, &format!("{error}. F7 — журнал действий."));
                    }
                }
            }
        }
        if chat == self.current {
            self.show_chat(app);
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
            Some(STOP) => {
                if let Some(pending) = self.pending.take() {
                    pending.worker.stop();
                    self.chats[pending.chat].messages.push(Message { role: "event".into(), text: "Остановлено пользователем. F7 — проверьте журнал; прерванное действие может иметь неизвестный результат.".into() });
                    self.show_chat(app);
                    self.feedback(
                        app,
                        "Остановлено. Перед повтором проверьте состояние инструментов.",
                    );
                    self.persist(app);
                }
            }
            Some(WORKSPACE) => {
                if self.pending.is_some() {
                    self.feedback(app, "Сначала остановите текущую задачу.");
                } else if let Some(path) = input_box(
                    app,
                    "Рабочая папка",
                    "Путь к проекту:",
                    &self.workspace.to_string_lossy(),
                    2048,
                ) {
                    match std::fs::canonicalize(path).and_then(|path| {
                        if path.is_dir() {
                            Ok(path)
                        } else {
                            Err(std::io::Error::other("Это не каталог"))
                        }
                    }) {
                        Ok(path) => {
                            self.workspace = path;
                            self.feedback(app, &format!("Проект: {}", self.workspace.display()));
                        }
                        Err(error) => self.feedback(app, &format!("Папка недоступна: {error}")),
                    }
                }
            }
            Some(JOURNAL) => {
                if self.pending.is_none() {
                    match api::start(
                        &self.config,
                        &[],
                        "",
                        &self.workspace,
                        &self.data_file.with_extension("runs.sqlite"),
                        api::Mode::Journal,
                        self.current,
                    ) {
                        Ok(worker) => {
                            self.pending = Some(Pending {
                                chat: self.current,
                                journal: true,
                                worker,
                            })
                        }
                        Err(error) => self.feedback(app, &error),
                    }
                }
            }
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
        let selected = app
            .find::<ListBox>(self.chat_list)
            .map(|list| list.focused)
            .unwrap_or(self.current);
        if selected != self.current && selected < self.chats.len() {
            self.draft(app);
            self.current = selected;
            self.show_chat(app);
            self.persist(app);
        }
        let width = app
            .find::<Terminal>(self.transcript)
            .map(|view| view.vs.size.x)
            .unwrap_or(0);
        if width != self.rendered_width {
            self.draft(app);
            self.show_chat(app);
        }
        self.poll(app);
        if let Some(window) = app.find_mut::<Window>(self.window) {
            window.title = format!(
                "Agent Harness — {}{}",
                self.config.model,
                if self.pending.is_some() {
                    " • ожидание"
                } else {
                    ""
                }
            );
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    locale::set_language(locale::Language::Ru);
    let data_file = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("agent-chats.json"));
    let menu = Menu::new().submenu(
        "~Ч~ат",
        Menu::new()
            .item_key("~Н~овый", NEW, kb::ctrl('N'), "Ctrl+N")
            .item_key("~О~тправить", SEND, kb::F2, "F2")
            .item_key("~П~овторить запрос", RETRY, kb::f(5), "F5")
            .item_key("Н~а~стройки", SETTINGS, kb::f(4), "F4")
            .item_key("О~с~тановить", STOP, kb::f(8), "F8")
            .item_key("Рабочая ~п~апка", WORKSPACE, kb::f(6), "F6")
            .item_key("~Ж~урнал действий", JOURNAL, kb::f(7), "F7")
            .item_key("В~ы~ход", cm::QUIT, kb::ALT_X, "Alt+X"),
    );
    let status = vec![StatusDef::all(vec![
        StatusItem::new("~F2~ Отправить", kb::F2, SEND),
        StatusItem::new("~F4~ Настройки", kb::f(4), SETTINGS),
        StatusItem::new("~F5~ Повторить", kb::f(5), RETRY),
        StatusItem::new("~Ctrl+N~ Новый чат", kb::ctrl('N'), NEW),
        StatusItem::new("~F6~ Проект", kb::f(6), WORKSPACE),
        StatusItem::new("~F7~ Журнал", kb::f(7), JOURNAL),
        StatusItem::new("~F8~ Стоп", kb::f(8), STOP),
        StatusItem::new("~Alt+X~ Выход", kb::ALT_X, cm::QUIT),
    ])];
    let mut app = Program::with_terminal(Some(menu), Some(status))?;
    let mut agent = Agent::build(&mut app, data_file)?;
    app.run(&mut agent);
    agent.draft(&app);
    agent.save()?;
    Ok(())
}

fn approval(app: &mut Program, name: &str, arguments: &str, workspace: &Path) -> bool {
    let extent = app.desktop().group.vs.extent();
    let mut dialog = Dialog::centered(
        (extent.width() - 2).max(30),
        (extent.height() - 2).max(12),
        "Разрешить действие агента?",
    );
    let scroll = dialog.win.standard_scroll_bar(true);
    let mut terminal = Terminal::new(Rect::default(), None, Some(scroll), 10000);
    let description = format!("Инструмент: {name}\nРабочая папка: {}\n\n{arguments}\n\nКоманда терминала выполняется с правами пользователя. Браузер и снимки могут передавать данные модели.", workspace.display());
    for line in description.lines() {
        for wrapped in tvision::text::wrap_text(line, (extent.width() - 8).max(1) as usize) {
            terminal.write_line(&wrapped);
        }
    }
    terminal.scroller.delta = Point::default();
    let output = dialog.add(terminal);
    let yes = dialog.add(Button::new(
        Rect::default(),
        "Разрешить",
        cm::YES,
        bf::NORMAL,
    ));
    let no = dialog.add(Button::new(
        Rect::default(),
        "Отклонить",
        cm::CANCEL,
        bf::DEFAULT,
    ));
    dialog.set_layout(
        VBox::new()
            .add(item(output).expand().stretch(1))
            .add(HBox::new().add(yes).add(no)),
    );
    dialog.select(output);
    app.exec_dialog(dialog).0 == cm::YES
}
