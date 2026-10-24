//! Буфер обмена: внутренний и системный.
//!
//! Аналог `TClipboard` и платформенного кода буфера обмена оригинального
//! tvision (`platform/unixclip.cpp`, `win32con.cpp`, `far2l.cpp`):
//! * **Windows** — WinAPI (`OpenClipboard`, `CF_UNICODETEXT`);
//! * **macOS** — утилиты `pbcopy` / `pbpaste`;
//! * **Linux/BSD** — Wayland (`wl-copy` / `wl-paste`), X11 (`xclip` или `xsel`);
//! * если ничего из этого нет — копирование через последовательность
//!   терминала **OSC 52** (работает в большинстве современных эмуляторов
//!   терминала, в том числе по SSH), а вставка — из внутреннего буфера.
//!
//! Внутренний буфер помнит последний скопированный текст всегда, поэтому
//! копирование и вставка внутри программы работают в любом случае.

/// Источник системного буфера обмена.
pub trait ClipboardBackend {
    /// Прочитать текст. `None` — не удалось (тогда используется внутренний буфер).
    fn get(&mut self) -> Option<String>;
    /// Записать текст. `false` — не удалось.
    fn set(&mut self, text: &str) -> bool;
    /// Название (для отладки и сообщений).
    fn name(&self) -> &'static str;
}

/// Буфер обмена программы (поле `Ctx::clipboard`).
pub struct Clipboard {
    text: String,
    system: Option<Box<dyn ClipboardBackend>>,
}

impl Default for Clipboard {
    fn default() -> Self {
        Clipboard::internal()
    }
}

impl Clipboard {
    /// Только внутренний буфер (для тестов и встраивания).
    pub fn internal() -> Self {
        Clipboard { text: String::new(), system: None }
    }

    /// Буфер, связанный с системным буфером обмена текущей ОС.
    pub fn system() -> Self {
        Clipboard { text: String::new(), system: detect() }
    }

    /// Буфер с заданным источником.
    pub fn with_backend(backend: Box<dyn ClipboardBackend>) -> Self {
        Clipboard { text: String::new(), system: Some(backend) }
    }

    /// Текст буфера: из системы, если она доступна, иначе внутренний.
    pub fn get(&mut self) -> String {
        if let Some(s) = self.system.as_mut() {
            if let Some(t) = s.get() {
                // Переводы строк Windows приводим к `\n`.
                self.text = t.replace("\r\n", "\n");
            }
        }
        self.text.clone()
    }

    /// Положить текст в буфер (и в системный буфер, если он есть).
    pub fn set(&mut self, text: &str) {
        self.text = text.to_string();
        if let Some(s) = self.system.as_mut() {
            s.set(text);
        }
    }

    /// Есть ли, возможно, что вставить. С системным буфером ответ «да»:
    /// узнавать его содержимое после каждого события слишком дорого.
    pub fn has_text(&self) -> bool {
        !self.text.is_empty() || self.system.is_some()
    }

    /// Название системного источника, если он есть.
    pub fn backend_name(&self) -> Option<&'static str> {
        self.system.as_ref().map(|s| s.name())
    }
}

/// Выбор системного источника для текущей ОС.
fn detect() -> Option<Box<dyn ClipboardBackend>> {
    #[cfg(windows)]
    {
        Some(Box::new(windows::WinClipboard))
    }
    #[cfg(not(windows))]
    {
        unix::detect()
    }
}

/// Кодирование Base64 (для OSC 52) — без внешних зависимостей.
pub fn base64_encode(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

#[cfg(not(windows))]
mod unix {
    use super::{base64_encode, ClipboardBackend};
    use std::io::Write;
    use std::process::{Command, Stdio};

    /// Буфер через внешние утилиты: команда чтения и команда записи.
    pub struct CommandClipboard {
        name: &'static str,
        get: &'static [&'static str],
        set: &'static [&'static str],
    }

    /// Варианты в порядке предпочтения.
    const CANDIDATES: &[CommandClipboard] = &[
        CommandClipboard { name: "pbcopy", get: &["pbpaste"], set: &["pbcopy"] },
        CommandClipboard { name: "wl-clipboard", get: &["wl-paste", "--no-newline"], set: &["wl-copy"] },
        CommandClipboard { name: "xclip", get: &["xclip", "-selection", "clipboard", "-o"], set: &["xclip", "-selection", "clipboard", "-i"] },
        CommandClipboard { name: "xsel", get: &["xsel", "--clipboard", "--output"], set: &["xsel", "--clipboard", "--input"] },
    ];

    /// Есть ли программа в `PATH`.
    fn in_path(prog: &str) -> bool {
        std::env::var_os("PATH").map(|p| std::env::split_paths(&p).any(|d| d.join(prog).is_file())).unwrap_or(false)
    }

    pub fn detect() -> Option<Box<dyn ClipboardBackend>> {
        let mac = cfg!(target_os = "macos");
        let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
        let x11 = std::env::var_os("DISPLAY").is_some();
        for c in CANDIDATES {
            let usable = match c.name {
                "pbcopy" => mac,
                "wl-clipboard" => wayland,
                _ => x11,
            };
            if usable && in_path(c.get[0]) && in_path(c.set[0]) {
                return Some(Box::new(CommandClipboard { name: c.name, get: c.get, set: c.set }));
            }
        }
        // Нет утилит — копируем последовательностью OSC 52 (вставка — изнутри программы).
        Some(Box::new(Osc52Clipboard))
    }

    impl ClipboardBackend for CommandClipboard {
        fn get(&mut self) -> Option<String> {
            let out = Command::new(self.get[0]).args(&self.get[1..]).stdin(Stdio::null()).stderr(Stdio::null()).output().ok()?;
            if out.status.success() {
                Some(String::from_utf8_lossy(&out.stdout).into_owned())
            } else {
                None
            }
        }

        fn set(&mut self, text: &str) -> bool {
            let child = Command::new(self.set[0]).args(&self.set[1..]).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
            match child {
                Ok(mut ch) => {
                    if let Some(mut stdin) = ch.stdin.take() {
                        let _ = stdin.write_all(text.as_bytes());
                    }
                    // xclip/wl-copy остаются жить, чтобы «держать» буфер, — не ждём их.
                    std::thread::spawn(move || {
                        let _ = ch.wait();
                    });
                    true
                }
                Err(_) => false,
            }
        }

        fn name(&self) -> &'static str {
            self.name
        }
    }

    /// Копирование через OSC 52: `ESC ] 52 ; c ; <base64> BEL`.
    pub struct Osc52Clipboard;

    impl ClipboardBackend for Osc52Clipboard {
        fn get(&mut self) -> Option<String> {
            // Чтение через OSC 52 почти нигде не разрешено — берём внутренний буфер.
            None
        }

        fn set(&mut self, text: &str) -> bool {
            let seq = format!("\x1b]52;c;{}\x07", base64_encode(text.as_bytes()));
            let mut out = std::io::stdout();
            out.write_all(seq.as_bytes()).is_ok() && out.flush().is_ok()
        }

        fn name(&self) -> &'static str {
            "osc52"
        }
    }
}

#[cfg(windows)]
mod windows {
    #![allow(non_snake_case, clippy::upper_case_acronyms)]
    use super::ClipboardBackend;
    use std::os::raw::c_void;

    type HANDLE = *mut c_void;
    type BOOL = i32;
    const CF_UNICODETEXT: u32 = 13;
    const GMEM_MOVEABLE: u32 = 0x0002;

    #[link(name = "user32")]
    extern "system" {
        fn OpenClipboard(hwnd: HANDLE) -> BOOL;
        fn CloseClipboard() -> BOOL;
        fn EmptyClipboard() -> BOOL;
        fn GetClipboardData(format: u32) -> HANDLE;
        fn SetClipboardData(format: u32, mem: HANDLE) -> HANDLE;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GlobalAlloc(flags: u32, bytes: usize) -> HANDLE;
        fn GlobalLock(mem: HANDLE) -> *mut c_void;
        fn GlobalUnlock(mem: HANDLE) -> BOOL;
        fn GlobalFree(mem: HANDLE) -> HANDLE;
        fn Sleep(ms: u32);
    }

    /// Системный буфер Windows (`CF_UNICODETEXT`).
    pub struct WinClipboard;

    /// Открыть буфер с несколькими попытками (его может держать другая программа).
    fn open() -> bool {
        for _ in 0..10 {
            if unsafe { OpenClipboard(std::ptr::null_mut()) } != 0 {
                return true;
            }
            unsafe { Sleep(10) };
        }
        false
    }

    impl ClipboardBackend for WinClipboard {
        fn get(&mut self) -> Option<String> {
            if !open() {
                return None;
            }
            let result = unsafe {
                let h = GetClipboardData(CF_UNICODETEXT);
                if h.is_null() {
                    None
                } else {
                    let p = GlobalLock(h) as *const u16;
                    if p.is_null() {
                        None
                    } else {
                        // Строка UTF-16, завершённая нулём.
                        let mut len = 0;
                        while *p.add(len) != 0 {
                            len += 1;
                        }
                        let s = String::from_utf16_lossy(std::slice::from_raw_parts(p, len));
                        GlobalUnlock(h);
                        Some(s)
                    }
                }
            };
            unsafe { CloseClipboard() };
            result
        }

        fn set(&mut self, text: &str) -> bool {
            // В буфер Windows кладём переводы строк `\r\n`.
            let wide: Vec<u16> = text.replace('\n', "\r\n").encode_utf16().chain(std::iter::once(0)).collect();
            if !open() {
                return false;
            }
            let ok = unsafe {
                EmptyClipboard();
                let h = GlobalAlloc(GMEM_MOVEABLE, wide.len() * 2);
                if h.is_null() {
                    false
                } else {
                    let p = GlobalLock(h) as *mut u16;
                    if p.is_null() {
                        GlobalFree(h);
                        false
                    } else {
                        std::ptr::copy_nonoverlapping(wide.as_ptr(), p, wide.len());
                        GlobalUnlock(h);
                        // После успешного SetClipboardData памятью владеет система.
                        if SetClipboardData(CF_UNICODETEXT, h).is_null() {
                            GlobalFree(h);
                            false
                        } else {
                            true
                        }
                    }
                }
            };
            unsafe { CloseClipboard() };
            ok
        }

        fn name(&self) -> &'static str {
            "windows"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn b64() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode("Привет".as_bytes()), "0J/RgNC40LLQtdGC");
    }

    /// Поддельный системный буфер для проверки логики.
    struct Fake(String);
    impl ClipboardBackend for Fake {
        fn get(&mut self) -> Option<String> {
            Some(self.0.clone())
        }
        fn set(&mut self, t: &str) -> bool {
            self.0 = t.into();
            true
        }
        fn name(&self) -> &'static str {
            "fake"
        }
    }

    #[test]
    fn system_wins() {
        let mut c = Clipboard::with_backend(Box::new(Fake("из системы\r\nвторая".into())));
        assert_eq!(c.get(), "из системы\nвторая");
        c.set("своё");
        assert_eq!(c.get(), "своё");
        let mut i = Clipboard::internal();
        assert!(!i.has_text());
        i.set("x");
        assert_eq!(i.get(), "x");
    }
}
