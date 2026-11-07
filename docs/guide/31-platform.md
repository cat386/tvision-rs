# Справочник: терминал, буфер обмена, время, язык

Связь программы с операционной системой. Обычно всё это работает само —
страница нужна, если вы пишете тесты, хотите свой буфер обмена или
переводите надписи.

На этой странице:
[`Backend`](#backend--терминал) ·
[`RawEvent`](#rawevent) ·
[`default_backend`](#default_backend) ·
[`HeadlessBackend`](#headlessbackend--невидимый-терминал) ·
[`UnixBackend`, `WindowsBackend`](#unixbackend-и-windowsbackend) ·
[`AnsiWriter`, `InputParser`, `Parsed`](#ansiwriter-inputparser-parsed) ·
[`Clipboard`](#clipboard--буфер-обмена) ·
[`ClipboardBackend`](#clipboardbackend--свой-буфер-обмена) ·
[готовые буферы](#готовые-буферы-обмена) ·
[`DateTime`](#datetime--местное-время) ·
[язык](#язык-интерфейса-language)

---

## `Backend` — терминал

**Что это.** Трейт, через который программа общается с терминалом:
узнаёт размер экрана, получает нажатия и выводит готовый кадр. Сама
программа о терминале больше ничего не знает — поэтому одна и та же
программа работает в Linux, Windows, в тестах и даже в графическом окне.

**Когда нужен.** Почти никогда: `Program::with_terminal` выбирает
терминал сам. Свой — если выводить нужно куда-то ещё (в окно, по сети).

```rust
use std::time::Duration;
use tvision::platform::{Backend, RawEvent};
use tvision::*;

/// Заготовка своего терминала.
struct MyBackend {
    size: Point,
}

impl Backend for MyBackend {
    fn size(&self) -> Point {
        self.size // размер экрана
    }
    fn read(&mut self, _timeout: Duration) -> Option<RawEvent> {
        None // ждать нажатие не дольше timeout; None — ничего не было
    }
    fn flush(&mut self, _screen: &ScreenBuffer, _cursor: Option<Point>, _insert: bool) {
        // показать кадр `screen`
    }
}

fn custom() {
    let mut app = Program::new(Box::new(MyBackend { size: Point::new(80, 25) }), None, None);
    app.pump(&mut program::NoHandler);
}
```

Необязательные методы: `suspend()`, `resume()` (вернуть терминал в
обычный режим и обратно), `system_clipboard()`, `last_frame()`.

## `RawEvent`

То, что приходит от терминала: `Key(KeyEvent)` (клавиша),
`Mouse { pos, buttons, mods, wheel }` (что сейчас с мышью — двойной
щелчок и прочее программа вычисляет сама), `Resize(размер)` (окно
терминала изменило размер) и `Command(команда)` — команда от самого
«экрана»: например, графическое окно tvision-gui присылает `cm::QUIT`,
когда окно закрывают крестиком.

## `default_backend`

`platform::default_backend()` — терминал для текущей системы. Его
вызывает `Program::with_terminal`.

## `HeadlessBackend` — невидимый терминал

**Что это.** Терминал для тестов: нажатия берутся из очереди, экран
хранится в памяти. Подробно — [урок 10](10-tests.md).

```rust
use tvision::platform::HeadlessBackend;
use tvision::*;

fn headless() {
    let mut b = HeadlessBackend::new(80, 25);
    b.push_key(kb::F10); // открыть меню
    b.push_key(kb::ESC); // закрыть
    b.push_key(kb::ALT_X); // выйти
    let menu = Menu::new().submenu("~Ф~айл", Menu::new().item_key("~В~ыход", cm::QUIT, kb::ALT_X, "Alt-X"));
    let mut app = Program::new(Box::new(b), Some(menu), None);
    app.pump(&mut program::NoHandler);
    assert!(!app.is_running()); // вышли
    assert!(app.screen_text().contains("Файл")); // меню на экране
}
```

| Метод / поле | Что |
|--------------|-----|
| `HeadlessBackend::new(ширина, высота)` | терминал |
| `push_key(клавиша)`, `push_click(x, y)` | добавить нажатие / щелчок в очередь |
| `input` | сама очередь — можно класть любые `RawEvent` |
| `esc_budget` | когда очередь пуста, терминал сам «нажимает» Esc, чтобы закрыть забытые окна; `0` — не нажимать |

**Реже:** `frame`, `cursor`, `flushes`, `size`.

## `UnixBackend` и `WindowsBackend`

Настоящие терминалы: `UnixBackend` — Linux, macOS, BSD; `WindowsBackend`
— консоль Windows 10 и новее. Создаются сами через `with_terminal`. При
завершении программы (и даже при панике) терминал возвращается в
обычный вид.

## `AnsiWriter`, `InputParser`, `Parsed`

Детали для тех, кто пишет свой терминал, понимающий ANSI-коды:
`AnsiWriter` превращает кадр в коды (`render(…)`, `invalidate()`,
`with_mode(…)`; функция `ansi::push_sgr`), `InputParser` разбирает
байты от терминала (`parse(&байты) -> Parsed`: `Event(…)`, `Incomplete`
— дочитать, `Skip(n)` — пропустить). Константы `ansi::ENTER_SEQ`,
`ansi::LEAVE_SEQ`.

---

## `Clipboard` — буфер обмена

**Что это.** Буфер обмена программы. Он связан с буфером операционной
системы: скопированное в программе можно вставить в браузер, и
наоборот. Строки ввода и редактор пользуются им сами (Ctrl+Ins,
Shift+Ins).

```rust
use tvision::platform::clipboard::Clipboard;
use tvision::*;

fn clipboard(app: &mut Program) {
    app.ctx.clipboard.set("текст"); // скопировать
    let s = app.ctx.clipboard.get(); // вставить
    let _ = s;
}

fn internal() {
    let mut c = Clipboard::internal(); // только внутри программы
    c.set("привет");
    assert_eq!(c.get(), "привет");
}
```

Из обработчика приложения — `app.ctx.clipboard`, из представления —
`ctx.clipboard`. Методы `get()`, `set(текст)`, `has_text()`,
`backend_name()` (что используется: `"windows"`, `"wl-clipboard"`,
`"osc52"`…). Создать: `Clipboard::system()` (с буфером системы),
`Clipboard::internal()` (только свой), `Clipboard::with_backend(…)`.

**Как связан с системой:**

| Система | Как |
|---------|-----|
| Windows | напрямую |
| macOS | программы `pbcopy` / `pbpaste` |
| Linux (Wayland) | `wl-copy` / `wl-paste` |
| Linux (X11) | `xclip` или `xsel` |
| нет ничего (например, по SSH) | копирование через терминал (OSC 52), вставка — из своего буфера |

## `ClipboardBackend` — свой буфер обмена

**Что это.** Трейт источника буфера: `get()`, `set(текст)`, `name()`.
Свой — если буфер должен быть где-то ещё.

```rust
use tvision::platform::clipboard::{Clipboard, ClipboardBackend};

/// Буфер обмена в файле.
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

## Готовые буферы обмена

`CommandClipboard` — через программы `pbcopy`, `wl-copy`, `xclip`,
`xsel` (`CommandClipboard::detect()` найдёт доступную); `Osc52Clipboard`
— через терминал (только копирование); `WinClipboard` — Windows.
Функция `clipboard::base64_encode` — служебная.

---

## `DateTime` — местное время

**Что это.** Дата и время: поля `year`, `month`, `day`, `hour`, `minute`,
`second`. Модуль `tvision::platform::time`.

```rust
use tvision::platform::time;

fn clock() -> String {
    let t = time::now();
    format!("{:02}.{:02}.{} {:02}:{:02}", t.day, t.month, t.year, t.hour, t.minute)
}

fn epoch() {
    let t = time::utc_from_secs(86_400); // сутки после 1 января 1970
    assert_eq!((t.year, t.month, t.day), (1970, 1, 2));
}
```

Функции: `time::now()` — сейчас, `time::local(момент)` — местное время
для `SystemTime`, `time::utc_from_secs(секунды)`.

---

## Язык интерфейса: `Language`

**Что это.** Язык надписей, которые рисует **сама библиотека**: кнопки
окон сообщений («OK», «Отмена»), файловые диалоги, сообщения
валидаторов, справка. По умолчанию — английский. Надписи **вашей**
программы вы пишете сами на любом языке.

```rust
use tvision::locale::{self, Language};

fn russian() {
    locale::set_language(Language::Ru); // обычно — первой строкой main
    assert_eq!(locale::tr("~Y~es"), "~Д~а");
    locale::set_language(Language::En);
}
```

| Что | Описание |
|-----|---------|
| `Language` | `En` (английский), `Ru` (русский) |
| `locale::set_language(язык)` | выбрать |
| `locale::language()` | текущий |
| `locale::tr(текст)` | перевести надпись библиотеки (если перевода нет — вернёт как есть) |

Горячие буквы работают в любой раскладке: Alt+Ф и Alt+A — одно и то же
([глава 13](../13-locale.md)).
