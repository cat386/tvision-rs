//! Платформенный слой: ввод с клавиатуры/мыши и вывод на экран терминала.
//!
//! Аналог каталога `source/platform` оригинального tvision
//! (`unixcon.cpp`, `win32con.cpp`, `ansiwrit.cpp`, `termio.cpp` ...).
//!
//! Программа работает с терминалом только через трейт [`Backend`]:
//! * [`unix::UnixBackend`] — Linux/macOS/BSD: termios + ANSI-последовательности;
//! * [`windows::WindowsBackend`] — консоль Windows 10+: Console API для ввода,
//!   VT-последовательности для вывода;
//! * [`HeadlessBackend`] — «невидимый» терминал для тестов: события задаются
//!   скриптом, а результат отрисовки можно прочитать как текст.

use std::collections::VecDeque;
use std::time::Duration;

use crate::drawbuf::ScreenBuffer;
use crate::event::{KeyEvent, Mods};
use crate::geometry::Point;

pub mod ansi;
#[cfg(unix)]
pub mod unix;
#[cfg(windows)]
pub mod windows;

/// «Сырое» событие от терминала, ещё не превращённое в [`crate::Event`].
#[derive(Clone, Debug, PartialEq)]
pub enum RawEvent {
    /// Нажата клавиша.
    Key(KeyEvent),
    /// Состояние мыши: позиция и **текущие** нажатые кнопки.
    /// Программа сама определяет, было ли это нажатие, отпускание или движение,
    /// сравнивая с предыдущим состоянием (так же делает `THardwareInfo` в оригинале).
    Mouse { pos: Point, buttons: u8, mods: Mods, wheel: i8 },
    /// Размер терминала изменился.
    Resize(Point),
}

/// Интерфейс терминала.
pub trait Backend {
    /// Текущий размер экрана в ячейках.
    fn size(&self) -> Point;

    /// Ждёт событие не дольше `timeout`. `None` — событий не было.
    fn read(&mut self, timeout: Duration) -> Option<RawEvent>;

    /// Выводит экранный буфер. `cursor` — позиция видимого курсора
    /// (`None` — курсор скрыт), `insert` — режим вставки (блочный курсор).
    fn flush(&mut self, screen: &ScreenBuffer, cursor: Option<Point>, insert: bool);

    /// Для тестового терминала — последний выведенный кадр.
    fn last_frame(&self) -> Option<&ScreenBuffer> {
        None
    }
}

/// Создаёт терминал, подходящий для текущей ОС.
pub fn default_backend() -> std::io::Result<Box<dyn Backend>> {
    #[cfg(unix)]
    {
        Ok(Box::new(unix::UnixBackend::new()?))
    }
    #[cfg(windows)]
    {
        Ok(Box::new(windows::WindowsBackend::new()?))
    }
    #[cfg(not(any(unix, windows)))]
    {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "платформа не поддерживается"))
    }
}

/// Тестовый терминал без реального экрана.
///
/// События берутся из очереди `input`; когда очередь пуста, `read` возвращает
/// `None` (программа считает, что пользователь бездействует). Последний
/// выведенный кадр хранится в `frame`.
pub struct HeadlessBackend {
    pub size: Point,
    pub input: VecDeque<RawEvent>,
    pub frame: Option<ScreenBuffer>,
    pub cursor: Option<Point>,
    /// Сколько раз был вызван `flush` (для проверок в тестах).
    pub flushes: usize,
    /// Защита тестов от зависания: когда ввод кончился, после нескольких
    /// пустых чтений терминал начинает «нажимать» Esc (не больше
    /// `esc_budget` раз), чтобы закрыть забытые модальные окна.
    pub esc_budget: usize,
    empty_reads: usize,
}

impl HeadlessBackend {
    pub fn new(width: i32, height: i32) -> Self {
        HeadlessBackend {
            size: Point::new(width, height),
            input: VecDeque::new(),
            frame: None,
            cursor: None,
            flushes: 0,
            esc_budget: 50,
            empty_reads: 0,
        }
    }

    /// Добавляет нажатие клавиши в очередь ввода.
    pub fn push_key(&mut self, k: KeyEvent) {
        self.input.push_back(RawEvent::Key(k));
    }

    /// Добавляет щелчок мышью (нажатие + отпускание) в очередь ввода.
    pub fn push_click(&mut self, x: i32, y: i32) {
        let pos = Point::new(x, y);
        self.input.push_back(RawEvent::Mouse { pos, buttons: 1, mods: Mods::NONE, wheel: 0 });
        self.input.push_back(RawEvent::Mouse { pos, buttons: 0, mods: Mods::NONE, wheel: 0 });
    }
}

impl Backend for HeadlessBackend {
    fn size(&self) -> Point {
        self.size
    }

    fn read(&mut self, _timeout: Duration) -> Option<RawEvent> {
        if let Some(e) = self.input.pop_front() {
            self.empty_reads = 0;
            return Some(e);
        }
        self.empty_reads += 1;
        if self.empty_reads > 5 && self.esc_budget > 0 {
            self.esc_budget -= 1;
            self.empty_reads = 0;
            return Some(RawEvent::Key(KeyEvent::new(crate::event::Key::Esc, Mods::NONE)));
        }
        None
    }

    fn flush(&mut self, screen: &ScreenBuffer, cursor: Option<Point>, _insert: bool) {
        self.frame = Some(screen.clone());
        self.cursor = cursor;
        self.flushes += 1;
    }

    fn last_frame(&self) -> Option<&ScreenBuffer> {
        self.frame.as_ref()
    }
}
