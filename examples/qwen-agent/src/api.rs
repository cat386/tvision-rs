use std::io::Write;
use std::process::{Command, Stdio};
use tvision::json::{parse_json, Json};

use crate::{Config, Message};

fn quoted(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r");
    format!("\"{escaped}\"")
}

pub fn request(config: &Config, messages: &[Message], key: &str) -> Result<String, String> {
    if key.is_empty() || key.chars().any(char::is_control) {
        return Err("Некорректный API-ключ в переменной окружения.".into());
    }
    let body = Json::object(vec![
        ("model", Json::from_str(&config.model)),
        ("stream", Json::Bool(false)),
        (
            "messages",
            Json::Arr(
                std::iter::once(Json::object(vec![("role", Json::from_str("system")), ("content", Json::from_str(&config.system))]))
                    .chain(
                        messages
                            .iter()
                            .map(|message| Json::object(vec![("role", Json::from_str(&message.role)), ("content", Json::from_str(&message.text))])),
                    )
                    .collect(),
            ),
        ),
    ])
    .compact();
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    // Ключ и запрос поступают curl через stdin и отсутствуют в командной строке процесса.
    let input = format!(
        "url = {}\nheader = {}\nheader = \"Content-Type: application/json\"\ndata-binary = {}\n",
        quoted(&url),
        quoted(&format!("Authorization: Bearer {key}")),
        quoted(&body),
    );
    let mut process = Command::new(if cfg!(windows) { "curl.exe" } else { "curl" });
    process
        .args([
            "--disable",
            "--config",
            "-",
            "--silent",
            "--show-error",
            "--connect-timeout",
            "15",
            "--max-time",
            "120",
            "--max-filesize",
            "4194304",
            "--proto",
            "=https,http",
            "--write-out",
            "\n%{http_code}",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        process.creation_flags(0x08000000);
    }
    let mut child = process.spawn().map_err(|e| format!("Не удалось запустить curl: {e}"))?;
    let written = child.stdin.take().ok_or("Нет stdin curl")?.write_all(input.as_bytes());
    if let Err(error) = written {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("Не удалось передать запрос curl: {error}"));
    }
    let output = child.wait_with_output().map_err(|e| format!("Ошибка ожидания curl: {e}"))?;
    if !output.status.success() {
        return Err(format!("Сетевая ошибка: {}", String::from_utf8_lossy(&output.stderr).replace(key, "[ключ скрыт]")));
    }
    let response = String::from_utf8(output.stdout).map_err(|_| "Ответ сервера не UTF-8")?;
    parse_response(&response).map_err(|e| e.replace(key, "[ключ скрыт]"))
}

fn parse_response(response: &str) -> Result<String, String> {
    let (body, status) = response.rsplit_once('\n').ok_or("Нет HTTP-статуса ответа")?;
    if status != "200" {
        let reason = parse_json(body).ok().and_then(|j| j.get("error")?.get("message")?.as_str().map(str::to_owned));
        return Err(format!("HTTP {status}: {}", reason.unwrap_or_else(|| "Проверьте URL, модель, ключ и квоту в Model Studio.".into())));
    }
    let json = parse_json(body).map_err(|e| format!("Некорректный JSON: {e}"))?;
    let answer = json
        .get("choices")
        .and_then(|j| j.at(0))
        .and_then(|j| j.get("message"))
        .and_then(|j| j.get("content"))
        .and_then(Json::as_str)
        .filter(|s| !s.trim().is_empty());
    answer.map(str::to_owned).ok_or_else(|| "Сервер не вернул текст ответа.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_answer_and_api_error() {
        assert_eq!(parse_response("{\"choices\":[{\"message\":{\"content\":\"Привет\"}}]}\n200").unwrap(), "Привет");
        assert!(parse_response("{\"error\":{\"message\":\"Invalid key\"}}\n401").unwrap_err().contains("Invalid key"));
        assert!(parse_response("{\"choices\":[]}\n200").is_err());
        assert_eq!(quoted("a\"b\nc\\d"), "\"a\\\"b\\nc\\\\d\"");
    }
}

#[cfg(test)]
mod http_tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::time::{Duration, Instant};

    #[test]
    fn sends_openai_compatible_request_to_local_server() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        listener.set_nonblocking(true).unwrap();
        let server = std::thread::spawn(move || {
            let start = Instant::now();
            let mut socket = loop {
                if let Ok((socket, _)) = listener.accept() {
                    break socket;
                }
                assert!(start.elapsed() < Duration::from_secs(10));
                std::thread::sleep(Duration::from_millis(10));
            };
            socket.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut bytes = Vec::new();
            let mut buffer = [0; 4096];
            let (header_end, length) = loop {
                let n = socket.read(&mut buffer).unwrap();
                assert!(n > 0);
                bytes.extend_from_slice(&buffer[..n]);
                if let Some(end) = bytes.windows(4).position(|x| x == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&bytes[..end]);
                    assert!(headers.starts_with("POST /v1/chat/completions HTTP/1.1"));
                    assert!(headers.contains("Authorization: Bearer fixture-key"));
                    let length = headers
                        .lines()
                        .find_map(|line| line.to_ascii_lowercase().strip_prefix("content-length: ").and_then(|v| v.parse::<usize>().ok()))
                        .unwrap();
                    break (end + 4, length);
                }
            };
            while bytes.len() < header_end + length {
                let n = socket.read(&mut buffer).unwrap();
                assert!(n > 0);
                bytes.extend_from_slice(&buffer[..n]);
            }
            let request = parse_json(std::str::from_utf8(&bytes[header_end..header_end + length]).unwrap()).unwrap();
            assert_eq!(request.get("model").and_then(Json::as_str), Some("fixture-model"));
            assert_eq!(request.get("messages").and_then(|j| j.at(1)).and_then(|j| j.get("content")).and_then(Json::as_str), Some("Привет\nмир"));
            let body = "{\"choices\":[{\"message\":{\"content\":\"Здравствуйте\"}}]}";
            write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
                .unwrap();
        });
        let config = Config { base_url: format!("http://127.0.0.1:{port}/v1"), model: "fixture-model".into(), ..Config::default() };
        let result = request(&config, &[Message { role: "user".into(), text: "Привет\nмир".into() }], "fixture-key");
        server.join().unwrap();
        assert_eq!(result.unwrap(), "Здравствуйте");
    }
}
