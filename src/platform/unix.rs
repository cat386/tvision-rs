//! Терминал Unix (Linux, macOS, BSD): termios + ANSI-последовательности.
//!
//! Аналог `platform/unixcon.cpp`, `termio.cpp`, `sigwinch.cpp` оригинала.
//! Чтобы не зависеть от крейта `libc`, нужные функции объявлены вручную.
//! Структура `termios` различается между ОС, поэтому мы не обращаемся к её
//! полям, а используем `cfmakeraw()` — она сама включает «сырой» режим.

use std::io::{self, Write};
use std::os::raw::{c_int, c_ulong, c_void};
use std::time::{Duration, Instant};

use super::ansi::{AnsiWriter, InputParser, Parsed, ENTER_SEQ, LEAVE_SEQ};
use super::{Backend, RawEvent};
use crate::drawbuf::ScreenBuffer;
use crate::event::{Key, KeyEvent, Mods};
use crate::geometry::Point;

/// Непрозрачный буфер под `struct termios` (с запасом по размеру).
#[repr(C, align(8))]
#[derive(Clone, Copy)]
struct Termios([u8; 256]);

#[repr(C)]
struct PollFd {
    fd: c_int,
    events: i16,
    revents: i16,
}

#[repr(C)]
#[derive(Default)]
struct Winsize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16,
    ws_ypixel: u16,
}

#[cfg(any(target_os = "linux", target_os = "android"))]
type NfdsT = c_ulong;
#[cfg(not(any(target_os = "linux", target_os = "android")))]
type NfdsT = std::os::raw::c_uint;

#[cfg(any(target_os = "linux", target_os = "android"))]
const TIOCGWINSZ: c_ulong = 0x5413;
#[cfg(not(any(target_os = "linux", target_os = "android")))]
const TIOCGWINSZ: c_ulong = 0x40087468;

const POLLIN: i16 = 1;
const TCSANOW: c_int = 0;
const STDIN: c_int = 0;
const STDOUT: c_int = 1;

extern "C" {
    fn tcgetattr(fd: c_int, t: *mut Termios) -> c_int;
    fn tcsetattr(fd: c_int, act: c_int, t: *const Termios) -> c_int;
    fn cfmakeraw(t: *mut Termios);
    fn ioctl(fd: c_int, req: c_ulong, ...) -> c_int;
    fn read(fd: c_int, buf: *mut c_void, n: usize) -> isize;
    fn poll(fds: *mut PollFd, nfds: NfdsT, timeout: c_int) -> c_int;
    fn isatty(fd: c_int) -> c_int;
}

/// Терминал Unix.
pub struct UnixBackend {
    /// Исходные настройки терминала — восстанавливаются при выходе.
    saved: Termios,
    size: Point,
    writer: AnsiWriter,
    parser: InputParser,
    /// Прочитанные, но ещё не разобранные байты.
    inbuf: Vec<u8>,
}

impl UnixBackend {
    /// Переводит терминал в полноэкранный «сырой» режим.
    pub fn new() -> io::Result<Self> {
        unsafe {
            if isatty(STDIN) == 0 {
                return Err(io::Error::new(io::ErrorKind::Other, "стандартный ввод не является терминалом"));
            }
            let mut saved = Termios([0; 256]);
            if tcgetattr(STDIN, &mut saved) != 0 {
                return Err(io::Error::last_os_error());
            }
            let mut raw = saved;
            cfmakeraw(&mut raw);
            if tcsetattr(STDIN, TCSANOW, &raw) != 0 {
                return Err(io::Error::last_os_error());
            }
            let mut b = UnixBackend { saved, size: Point::new(80, 25), writer: AnsiWriter::new(), parser: InputParser::new(), inbuf: Vec::new() };
            b.size = b.query_size();
            let mut out = io::stdout();
            out.write_all(ENTER_SEQ.as_bytes())?;
            out.flush()?;
            Ok(b)
        }
    }

    /// Узнаёт размер терминала через ioctl(TIOCGWINSZ).
    fn query_size(&self) -> Point {
        let mut ws = Winsize::default();
        let r = unsafe { ioctl(STDOUT, TIOCGWINSZ, &mut ws as *mut Winsize) };
        if r == 0 && ws.ws_col > 0 && ws.ws_row > 0 {
            Point::new(ws.ws_col as i32, ws.ws_row as i32)
        } else {
            self.size
        }
    }

    /// Ждёт данных на stdin не дольше `ms` миллисекунд и дочитывает их в буфер.
    fn fill(&mut self, ms: i32) -> bool {
        let mut pfd = PollFd { fd: STDIN, events: POLLIN, revents: 0 };
        let r = unsafe { poll(&mut pfd, 1, ms) };
        if r <= 0 || pfd.revents & POLLIN == 0 {
            return false;
        }
        let mut tmp = [0u8; 1024];
        let n = unsafe { read(STDIN, tmp.as_mut_ptr() as *mut c_void, tmp.len()) };
        if n > 0 {
            self.inbuf.extend_from_slice(&tmp[..n as usize]);
            true
        } else {
            false
        }
    }

    /// Пытается разобрать событие из уже прочитанных байтов.
    fn take_event(&mut self) -> Option<RawEvent> {
        loop {
            if self.inbuf.is_empty() {
                return None;
            }
            match self.parser.parse(&self.inbuf) {
                Parsed::Event(e, n) => {
                    self.inbuf.drain(..n);
                    return Some(e);
                }
                Parsed::Skip(n) => {
                    self.inbuf.drain(..n.max(1));
                }
                Parsed::Incomplete => {
                    // Даём терминалу 30 мс дослать хвост последовательности.
                    if self.fill(30) {
                        continue;
                    }
                    // Не дослал: одиночный ESC — это клавиша Esc.
                    if self.inbuf[0] == 0x1b {
                        self.inbuf.drain(..1);
                        return Some(RawEvent::Key(KeyEvent::new(Key::Esc, Mods::NONE)));
                    }
                    // Обрывок UTF-8 — выбрасываем.
                    self.inbuf.drain(..1);
                }
            }
        }
    }
}

impl Backend for UnixBackend {
    fn size(&self) -> Point {
        self.size
    }

    fn read(&mut self, timeout: Duration) -> Option<RawEvent> {
        let deadline = Instant::now() + timeout;
        loop {
            // Изменение размера окна проверяем опросом вместо обработчика SIGWINCH:
            // это проще и не требует глобального состояния.
            let s = self.query_size();
            if s != self.size {
                self.size = s;
                self.writer.invalidate();
                return Some(RawEvent::Resize(s));
            }
            if let Some(e) = self.take_event() {
                return Some(e);
            }
            let now = Instant::now();
            if now >= deadline {
                return None;
            }
            // Ждём порциями не больше 100 мс, чтобы вовремя заметить изменение размера.
            let left = (deadline - now).as_millis().min(100) as i32;
            self.fill(left.max(1));
        }
    }

    fn system_clipboard(&self) -> bool {
        true
    }

    fn flush(&mut self, screen: &ScreenBuffer, cursor: Option<Point>, insert: bool) {
        let s = self.writer.render(screen, cursor, insert);
        let mut out = io::stdout().lock();
        let _ = out.write_all(s.as_bytes());
        let _ = out.flush();
    }
}

impl Drop for UnixBackend {
    /// Восстанавливаем терминал даже при панике.
    fn drop(&mut self) {
        let mut out = io::stdout();
        let _ = out.write_all(LEAVE_SEQ.as_bytes());
        let _ = out.flush();
        unsafe {
            tcsetattr(STDIN, TCSANOW, &self.saved);
        }
    }
}
