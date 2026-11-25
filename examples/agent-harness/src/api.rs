//! Построчный протокол между интерфейсом и изолированным исполнителем Python.
use crate::{Config, Message};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use tvision::json::{parse_json, Json};

pub enum Event {
    Progress(String),
    Approval {
        name: String,
        arguments: String,
        answer: mpsc::Sender<bool>,
    },
    Done(Result<String, String>),
}

pub struct Worker {
    pub events: mpsc::Receiver<Event>,
    child: Arc<Mutex<Child>>,
}

impl Worker {
    pub fn stop(&self) {
        if let Ok(mut child) = self.child.lock() {
            if child.try_wait().ok().flatten().is_some() {
                return;
            }
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                if let Some(root) = std::env::var_os("SystemRoot") {
                    let _ = Command::new(Path::new(&root).join("System32/taskkill.exe"))
                        .args(["/PID", &child.id().to_string(), "/T", "/F"])
                        .creation_flags(0x08000000)
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                }
            }
            #[cfg(unix)]
            {
                let _ = Command::new("/bin/kill")
                    .args(["-KILL", "--", &format!("-{}", child.id())])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
            let _ = child.kill();
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop();
    }
}

pub enum Mode {
    Run { resume: bool },
    Journal,
}

pub fn start(
    config: &Config,
    messages: &[Message],
    key: &str,
    workspace: &Path,
    journal: &Path,
    mode: Mode,
    conversation: usize,
) -> Result<Worker, String> {
    let journal_only = matches!(mode, Mode::Journal);
    let resume = matches!(mode, Mode::Run { resume: true });
    let request = Json::object(vec![
        (
            "mode",
            Json::from_str(if journal_only { "journal" } else { "run" }),
        ),
        ("config", config.json()),
        ("key", Json::from_str(key)),
        ("conversation", Json::from_i64(conversation as i64)),
        ("resume", Json::Bool(resume)),
        ("workspace", Json::from_str(&workspace.to_string_lossy())),
        ("journal", Json::from_str(&journal.to_string_lossy())),
        (
            "messages",
            Json::Arr(
                messages
                    .iter()
                    .filter(|m| matches!(m.role.as_str(), "user" | "assistant"))
                    .map(|m| {
                        Json::object(vec![
                            ("role", Json::from_str(&m.role)),
                            ("content", Json::from_str(&m.text)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ]);
    let python = std::env::var("HARNESS_PYTHON").unwrap_or_else(|_| "python".into());
    let mut command = Command::new(python);
    command
        .args(["-u", "-c", include_str!("../tools/harness.py")])
        .env("PYTHONIOENCODING", "utf-8")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command
        .spawn()
        .map_err(|e| format!("Не удалось запустить Python 3: {e}"))?;
    let mut input = child.stdin.take().ok_or("Нет stdin Python")?;
    let output = child.stdout.take().ok_or("Нет stdout Python")?;
    let error = child.stderr.take().ok_or("Нет stderr Python")?;
    let errors = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = error.take(16000).read_to_string(&mut text);
        text
    });
    let child = Arc::new(Mutex::new(child));
    let handle = child.clone();
    let (sender, events) = mpsc::channel();
    let request = request.compact();
    let key = key.to_string();
    if let Err(e) = std::thread::Builder::new()
        .name("agent-harness".into())
        .spawn(move || {
            let result = (|| -> Result<(), String> {
                writeln!(input, "{request}").map_err(|e| e.to_string())?;
                input.flush().map_err(|e| e.to_string())?;
                for line in BufReader::new(output).lines() {
                    let line = line.map_err(|e| e.to_string())?;
                    let event =
                        parse_json(&line).map_err(|e| format!("Ошибка протокола Python: {e}"))?;
                    let get = |name| {
                        event
                            .get(name)
                            .and_then(Json::as_str)
                            .unwrap_or("")
                            .to_owned()
                    };
                    match get("type").as_str() {
                        "progress" => sender
                            .send(Event::Progress(get("text")))
                            .map_err(|_| "Окно закрыто")?,
                        "approval" => {
                            let (answer, reply) = mpsc::channel();
                            sender
                                .send(Event::Approval {
                                    name: get("name"),
                                    arguments: get("arguments"),
                                    answer,
                                })
                                .map_err(|_| "Окно закрыто")?;
                            let allow = reply.recv().unwrap_or(false);
                            writeln!(input, "{{\"allow\":{allow}}}").map_err(|e| e.to_string())?;
                            input.flush().map_err(|e| e.to_string())?;
                        }
                        "done" => {
                            let _ = sender.send(Event::Done(Ok(get("text"))));
                            return Ok(());
                        }
                        "error" => return Err(get("text")),
                        _ => return Err("Неизвестное событие Python".into()),
                    }
                }
                let status = handle
                    .lock()
                    .map_err(|_| "Ошибка процесса")?
                    .wait()
                    .map_err(|e| e.to_string())?;
                let mut detail = errors.join().unwrap_or_default();
                if !key.is_empty() {
                    detail = detail.replace(&key, "[ключ скрыт]");
                }
                Err(format!(
                    "Исполнитель завершился без ответа ({status}): {detail}"
                ))
            })();
            if let Err(error) = result {
                let _ = sender.send(Event::Done(Err(error)));
            }
        })
    {
        let worker = Worker { events, child };
        worker.stop();
        return Err(e.to_string());
    }
    Ok(Worker { events, child })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_bridge_reads_journal_without_api_key() {
        let root = std::env::temp_dir().join(format!("harness-bridge-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let journal = root.join("empty.sqlite");
        let worker = start(
            &Config::default(),
            &[],
            "",
            &root,
            &journal,
            Mode::Journal,
            0,
        )
        .unwrap();
        match worker
            .events
            .recv_timeout(std::time::Duration::from_secs(15))
            .unwrap()
        {
            Event::Done(Ok(text)) => assert_eq!(text, "Журнал пуст"),
            Event::Done(Err(error)) => panic!("{error}"),
            _ => panic!("Unexpected event"),
        }
        drop(worker);
        let _ = std::fs::remove_file(journal);
        let _ = std::fs::remove_dir(root);
    }
}
