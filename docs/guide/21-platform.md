# Справочник: терминал, буфер обмена, время, язык

Всё, что связывает программу с операционной системой. Модули `platform`,
`platform::clipboard`, `platform::time`, `platform::ansi`, `locale`.

* [`Backend`](#backend) · [`RawEvent`](#rawevent) · [`default_backend`](#default_backend)
* [`HeadlessBackend`](#headlessbackend) · [`UnixBackend`](#unixbackend) · [`WindowsBackend`](#windowsbackend)
* [`AnsiWriter`, `InputParser`, `Parsed`](#ansiwriter-inputparser-parsed)
* [`Clipboard`](#clipboard) · [`ClipboardBackend`](#clipboardbackend) · [`CommandClipboard`, `Osc52Clipboard`, `WinClipboard`](#готовые-источники-буфера)
* [`DateTime` и функции времени](#datetime-и-функции-времени)
* [Язык интерфейса: `Language`, `set_language`, `tr`](#язык-интерфейса)

---

## `Backend`

**Терминал** глазами программы — трейт (аналог каталога
`source/platform` оригинала). Программа знает о терминале только то, что
даёт этот трейт: так одна и та же программа работает в Unix, Windows, в
тестах и в графическом окне (tvision-gui).

```rust
use std::time::Duration;
use tvision::platform::{Backend, RawEvent};
use tvision::*;

/// Каркас своего терминала (вывод в окно, по сети…).
struct MyBackend {
    size: Point,
}

impl Backend for MyBackend {
    fn size(&self) -> Point {
        self.size
    }
    fn read(&mut self, _timeout: Duration) -> Option<RawEvent> {
        None // ждать событие не дольше timeout
    }
    fn flush(&mut self, _screen: &ScreenBuffer, _cursor: Option<Point>, _insert: bool) {
        // показать кадр; cursor — видимый курсор, insert — блочный курсор
    }
}

fn custom() {
    let mut app = Program::new(Box::new(MyBackend { size: Point::new(80, 25) }), None, None);
    app.pump(&mut program::NoHandler);
}
```

| Метод | Обязателен | Что |
|-------|-----------|-----|
| `size()` | да | размер экрана в ячейках |
| `read(timeout)` | да | следующее событие или `None` по тайм-ауту |
| `flush(screen, cursor, insert)` | да | показать кадр |
| `suspend()`, `resume()` | нет | вернуть терминал в обычный режим и обратно (`dos_shell`, `suspended`) |
| `system_clipboard()` | нет | подключать ли системный буфер обмена |
| `last_frame()` | нет | последний кадр (для тестов) |

## `RawEvent`

«Сырое» событие от терминала: `Key(KeyEvent)`, `Mouse { pos, buttons,
mods, wheel }` (**текущее** состояние мыши — нажатие, отпускание, двойной
щелчок и автоповтор программа вычисляет сама), `Resize(Point)`.

## `default_backend`

`platform::default_backend() -> io::Result<Box<dyn Backend>>` — терминал
для текущей ОС. Его вызывает `Program::with_terminal`.

## `HeadlessBackend`

**«Невидимый» терминал** для тестов: события из очереди, кадр в памяти.

```rust
use tvision::platform::{HeadlessBackend, RawEvent};
use tvision::*;

fn headless() {
    let mut b = HeadlessBackend::new(80, 25);
    b.push_key(kb::F10); // открыть меню
    b.push_key(kb::ESC); // закрыть
    b.input.push_back(RawEvent::Key(KeyEvent::new(Key::Char('x'), Mods::ALT)));
    let menu = Menu::new().submenu("~Ф~айл", Menu::new().item_key("~В~ыход", cm::QUIT, kb::ALT_X, "Alt-X"));
    let mut app = Program::new(Box::new(b), Some(menu), None);
    app.pump(&mut program::NoHandler);
    assert!(!app.is_running()); // Alt+X — выход
    assert!(app.screen_text().contains("Файл"));
}
```

| Поле / метод | Что |
|--------------|-----|
| `HeadlessBackend::new(w, h)` | экран `w`×`h` |
| `input: VecDeque<RawEvent>` | очередь ввода |
| `push_key(k)`, `push_click(x, y)` | добавить клавишу / щелчок |
| `frame`, `cursor`, `flushes` | последний кадр, курсор, число выводов |
| `size` | размер (смените и положите `RawEvent::Resize`, чтобы проверить изменение размера) |
| `esc_budget` | когда очередь пуста, после нескольких пустых чтений терминал «нажимает» Esc (не больше `esc_budget` раз, по умолчанию 50), чтобы забытое модальное окно не подвесило тест; `0` — не нажимать (нужно для окон, ждущих поток) |

Приёмы — [глава 14](../14-testing.md) и шаг 6 [урока](01-tutorial.md#шаг-6-тесты).

## `UnixBackend`

**Терминал Unix** (Linux, macOS, BSD): `termios` в «сыром» режиме,
альтернативный экран, мышь (SGR 1006), ANSI-цвета, `SIGWINCH`.
`UnixBackend::new()`; при удалении (`Drop`) терминал восстанавливается,
в том числе при панике.

## `WindowsBackend`

**Консоль Windows 10+**: Console API для ввода (клавиши, мышь, размер) и
VT-последовательности для вывода. `WindowsBackend::new()`. Alt+буква
приходит латинской буквой той же клавиши в любой раскладке.

## `AnsiWriter`, `InputParser`, `Parsed`

Строительные блоки своих терминалов, говорящих на ANSI:

* `AnsiWriter` — превращает кадр в управляющие последовательности, выводя
  только изменившиеся ячейки: `new()`, `with_mode(ColorMode)`,
  `render(&screen, cursor, insert) -> String`, `invalidate()` (следующий
  кадр целиком), функция `ansi::push_sgr(&mut out, attr, mode)`;
* `InputParser` — разбирает входные байты терминала: `parse(&buf) ->
  Parsed`, где `Parsed::Event(RawEvent, длина)`, `Incomplete` (дочитать),
  `Skip(n)` (пропустить мусор);
* `ansi::ENTER_SEQ`, `ansi::LEAVE_SEQ` — последовательности входа в
  полноэкранный режим и выхода из него.

---

## `Clipboard`

**Буфер обмена** программы — поле `ctx.clipboard` (из приложения —
`app.ctx.clipboard`). Строки ввода и редактор пользуются им сами.

```rust
use tvision::platform::clipboard::Clipboard;
use tvision::*;

fn clipboard(app: &mut Program) {
    app.ctx.clipboard.set("текст");
    let s = app.ctx.clipboard.get();
    let _ = (s, app.ctx.clipboard.backend_name()); // "wl-clipboard", "osc52", "windows"… или None
}

fn internal() {
    let mut c = Clipboard::internal(); // без системного буфера
    c.set("привет");
    assert_eq!(c.get(), "привет");
    assert!(c.has_text());
}
```

| Метод | Что |
|-------|-----|
| `Clipboard::system()` | связанный с буфером ОС (так у настоящего терминала) |
| `Clipboard::internal()` | только внутренний (так у `HeadlessBackend`) |
| `Clipboard::with_backend(b)` | со своим источником |
| `get()`, `set(s)` | прочитать (из системы, если можно) / записать (и в систему) |
| `has_text()`, `backend_name()` | есть ли что вставить; имя источника |

| ОС | Как |
|----|-----|
| Windows | WinAPI, `CF_UNICODETEXT` |
| macOS | `pbcopy` / `pbpaste` |
| Linux / BSD, Wayland | `wl-copy` / `wl-paste` |
| Linux / BSD, X11 | `xclip` или `xsel` |
| ничего из этого (SSH) | копирование — OSC 52, вставка — из внутреннего буфера |

## `ClipboardBackend`

**Источник системного буфера** — трейт: `get() -> Option<String>`,
`set(text) -> bool`, `name()`.

```rust
use tvision::platform::clipboard::{Clipboard, ClipboardBackend};

/// Буфер в файле (например, для общего буфера двух программ).
struct FileClipboard(std::path::PathBuf);

impl ClipboardBackend for FileClipboard {
    fn get(&mut self) -> Option<String> {
        std::fs::read_to_string(&self.0).ok()
    }
    fn set(&mut self, text: &str) -> bool {
        std::fs::write(&self.0, text).is_ok()
    }
    fn name(&self) -> &'static str {
        "file"
    }
}

fn use_file(app: &mut tvision::Program) {
    app.ctx.clipboard = Clipboard::with_backend(Box::new(FileClipboard("/tmp/clip.txt".into())));
}
```

## Готовые источники буфера

* `CommandClipboard` — через внешние утилиты (`pbcopy`, `wl-copy`,
  `xclip`, `xsel`); `CommandClipboard::detect()` находит доступную;
* `Osc52Clipboard` — копирование последовательностью терминала OSC 52
  (работает по SSH в большинстве терминалов; вставки нет);
* `WinClipboard` — WinAPI на Windows;
* `clipboard::base64_encode(bytes)` — для OSC 52, без внешних крейтов.

---

## `DateTime` и функции времени

`platform::time` — местное время без внешних крейтов (`localtime_r` на
Unix, WinAPI на Windows).

```rust
use tvision::platform::time;

fn clock() -> String {
    let t = time::now(); // DateTime { year, month, day, hour, minute, second }
    format!("{:02}.{:02}.{} {:02}:{:02}", t.day, t.month, t.year, t.hour, t.minute)
}

fn epoch() {
    let t = time::utc_from_secs(86_400);
    assert_eq!((t.year, t.month, t.day), (1970, 1, 2));
}
```

| Функция | Что |
|---------|-----|
| `time::now()` | текущее местное время |
| `time::local(system_time)` | местное время для момента |
| `time::utc_from_secs(secs)` | UTC по секундам от 1970 года |

---

## Язык интерфейса

Надписи, которые рисует сама библиотека (кнопки окон сообщений, файловые
диалоги, сообщения валидаторов, справка, поиск в редакторе), по умолчанию
английские. Русский включается одной строкой в начале `main`.

```rust
use tvision::locale::{self, Language};

fn russian() {
    locale::set_language(Language::Ru);
    assert_eq!(locale::tr("~Y~es"), "~Д~а");
    assert_eq!(locale::tr("Моя надпись"), "Моя надпись"); // без перевода — как есть
    locale::set_language(Language::En);
}
```

| Тип / функция | Что |
|---------------|-----|
| `Language` | `En` (по умолчанию), `Ru` |
| `locale::set_language(l)`, `locale::language()` | выбрать / узнать язык |
| `locale::tr(s)` | перевод встроенной надписи (английская строка — ключ) |

Надписи **своей** программы пишите сразу на нужном языке. Горячие буквы
работают в любой раскладке: Alt+Ф и Alt+A равнозначны
([глава 13](../13-locale.md)).
