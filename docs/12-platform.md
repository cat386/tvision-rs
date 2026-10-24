# 12. Терминал и платформа

## Трейт `Backend`

Программа общается с терминалом только через трейт `platform::Backend`
(аналог каталога `source/platform` оригинала):

```rust
use std::time::Duration;
use tvision::platform::RawEvent;
use tvision::*;

/// Каркас своего терминала (например, для вывода в окно или по сети).
struct MyBackend {
    size: Point,
}

impl platform::Backend for MyBackend {
    fn size(&self) -> Point {
        self.size
    }
    fn read(&mut self, _timeout: Duration) -> Option<RawEvent> {
        None // ждать событие не дольше timeout
    }
    fn flush(&mut self, _screen: &ScreenBuffer, _cursor: Option<Point>, _insert: bool) {
        // вывести кадр; cursor — позиция видимого курсора, insert — блочный курсор
    }
}

fn run_custom() {
    let mut app = Program::new(Box::new(MyBackend { size: Point::new(80, 25) }), None, None);
    app.pump(&mut program::NoHandler);
}
```

| Метод | Обязателен | Назначение |
|-------|-----------|-----------|
| `size()` | да | размер экрана в ячейках |
| `read(timeout)` | да | следующее «сырое» событие или `None` по тайм-ауту |
| `flush(screen, cursor, insert)` | да | показать кадр |
| `suspend()`, `resume()` | нет | вернуть терминал в обычный режим и обратно (для `dos_shell`) |
| `system_clipboard()` | нет | подключать ли системный буфер обмена |
| `last_frame()` | нет | последний кадр (для тестов) |

`RawEvent`:

* `Key(KeyEvent)` — клавиша;
* `Mouse { pos, buttons, mods, wheel }` — **текущее** состояние мыши;
  программа сама определяет нажатие, отпускание, движение, двойной щелчок
  и автоповтор;
* `Resize(Point)` — новый размер экрана.

`platform::default_backend()` создаёт терминал для текущей ОС; его же
использует `Program::with_terminal`.

## Unix: Linux, macOS, BSD

`platform::unix::UnixBackend`:

* «сырой» режим termios (`cfmakeraw`), без крейта `libc` — функции
  объявлены вручную;
* альтернативный экран, скрытый системный курсор на время работы;
* мышь в формате SGR (режимы 1002/1003/1006): щелчки, движение, колесо,
  модификаторы;
* разбор последовательностей клавиш (`ESC [ … ~`, `ESC O P`), UTF-8;
* изменение размера окна терминала (размер опрашивается через
  `ioctl(TIOCGWINSZ)`, обработчик `SIGWINCH` не нужен);
* при выходе (`Drop`) терминал восстанавливается.

Вывод (`platform::ansi`) сравнивает кадр с предыдущим и посылает только
изменившиеся ячейки.

## Windows

`platform::windows::WindowsBackend` (Windows 10 и новее):

* ввод — `ReadConsoleInputW`: клавиатура, мышь, изменение размера;
* вывод — VT-последовательности (`ENABLE_VIRTUAL_TERMINAL_PROCESSING`)
  в UTF-16, независимо от кодовой страницы консоли;
* функции kernel32 объявлены вручную, без крейтов `winapi`/`windows-sys`.

Особенность Windows: при Alt или Ctrl буква берётся по **коду клавиши**,
поэтому Alt+X работает и в русской раскладке — но и Alt+Ч приходит как
`alt('x')`. Библиотека учитывает это при сравнении горячих букв
([глава 13](13-locale.md)).

## Буфер обмена

`Ctx::clipboard` (`platform::clipboard::Clipboard`) связан с системным
буфером обмена; строки ввода и редактор пользуются им для
копирования/вставки.

| ОС | Как |
|----|-----|
| Windows | WinAPI, `CF_UNICODETEXT` |
| macOS | `pbcopy` / `pbpaste` |
| Linux/BSD, Wayland | `wl-copy` / `wl-paste` |
| Linux/BSD, X11 | `xclip` или `xsel` |
| ничего из этого (например, по SSH) | копирование — последовательностью терминала OSC 52, вставка — из внутреннего буфера |

Внутренний буфер помнит последний скопированный текст всегда, поэтому
копирование и вставка внутри программы работают в любом случае.

```rust
use tvision::*;

fn clipboard(app: &mut Program) {
    app.ctx.clipboard.set("текст");
    let s = app.ctx.clipboard.get();
    let _ = (s, app.ctx.clipboard.backend_name()); // имя источника ("windows", "osc52"…) или None
}
```

Свой источник — реализуйте `clipboard::ClipboardBackend` и передайте в
`Clipboard::with_backend`. `Clipboard::internal()` — только внутренний
буфер (у `HeadlessBackend` по умолчанию именно он).

## Время

`platform::time` — местное время без внешних крейтов:

```rust
use tvision::*;

fn clock() -> String {
    let t = platform::time::now(); // DateTime { year, month, day, hour, minute, second }
    format!("{:02}.{:02}.{} {:02}:{:02}", t.day, t.month, t.year, t.hour, t.minute)
}
```

`time::local(system_time)` — местное время для момента,
`time::utc_from_secs(secs)` — UTC по секундам от 1970 года.

## Unicode и ширина символов

* Текст — обычные `String`/`char` в UTF-8.
* `text::char_width(c)` — ширина символа в ячейках: 0 (комбинирующие и
  управляющие), 1, 2 (иероглифы, многие эмодзи). `text::str_width(s)`,
  `text::cstr_width(s)` (без тильд), `text::truncate_width(s, max)`.
* Экранная ячейка `Cell { ch, attr }`; вторая половина широкого символа
  хранит `'\0'` (`is_wide_tail()`).
* `text::wrap_text(text, width)` — перенос по словам (как у `StaticText`).
* Кириллица, `to_upper` для горячих букв — с учётом Unicode.

## Оболочка ОС

`app.dos_shell("Введите exit, чтобы вернуться…")` временно возвращает
терминал в обычный режим, запускает `$SHELL` (Unix) или `%COMSPEC%`
(Windows) и после выхода из него продолжает работу программы.
