# Справочник: программа, рабочий стол, окна, диалоги

Каркас любого приложения: программа с циклом событий, рабочий стол с
фоном, окна и диалоги. Модули `program` и `widgets::{desktop, background,
window, frame, dialog}`.

* [`Program`](#program) · [`AppHandler`](#apphandler) · [`NoHandler`](#nohandler)
* [`Desktop`](#desktop) · [`Background`](#background)
* [`Window`](#window) · [`wf`](#wf--флаги-окна) · [`Frame`, `FrameInfo`](#frame-и-frameinfo)
* [`Dialog`](#dialog) · [`DialogPalette`](#dialogpalette)

---

## `Program`

**Программа**: корневая группа (строка меню, рабочий стол, строка
состояния), цикл событий, палитра, набор команд, буфер обмена. Аналог
`TProgram` + `TApplication`.

```rust
use tvision::*;

struct App;

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(cm::NEW) {
            message_box(app, "\x03Привет!", mf::INFORMATION | mf::OK_BUTTON);
            ev.clear();
        }
    }
}

fn run() -> std::io::Result<()> {
    let menu = Menu::new().submenu("~Ф~айл", Menu::new().item_key("~П~ривет", cm::NEW, kb::F2, "F2").item_key("~В~ыход", cm::QUIT, kb::ALT_X, "Alt-X"));
    let status = vec![StatusDef::all(vec![StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT)])];
    let mut app = Program::with_terminal(Some(menu), Some(status))?;
    app.run(&mut App);
    Ok(())
}
```

### Создание

| Конструктор | Что |
|-------------|-----|
| `Program::with_terminal(menu, status)` | в настоящем терминале (Unix — termios + ANSI, Windows — консоль); при выходе терминал восстанавливается |
| `Program::new(backend, menu, status)` | с любым терминалом — например, `HeadlessBackend` для тестов |

`menu` — `Option<Menu>` (без меню — `None`, рабочий стол начнётся с первой
строки), `status` — `Option<Vec<StatusDef>>`.

### Цикл событий

| Метод | Что |
|-------|-----|
| `run(&mut handler)` | главный цикл до `cm::QUIT` или `quit()` |
| `step(&mut handler) -> bool` | одна итерация: отрисовка, событие, обработка (`false` — событий не было) |
| `pump(&mut handler)` | обработать всю очередь (тесты) |
| `quit()`, `is_running()` | завершить цикл / идёт ли он |
| `put_event(ev)` | поставить событие в очередь |
| `get_event()`, `handle(ev, handler)` | получить и обработать одно событие вручную |
| `redraw()` | перерисовать сейчас |

`cm::QUIT` сначала спрашивает `valid(cm::QUIT)` у всех окон: любое может
отказаться.

### Окна и модальность

| Метод | Что |
|-------|-----|
| `insert_window(w) -> ViewId` | вставить окно на рабочий стол и сделать текущим (0 — окно не прошло `valid(cm::VALID)`) |
| `insert_window_boxed(b)` | то же для `Box<dyn View>` |
| `exec_dialog(d) -> (cmd, d)` | **модально** выполнить и вернуть представление того же типа |
| `exec_view(b) -> (cmd, b)` | то же для `Box<dyn View>` (`execView`) |
| `exec_dialog_idle(d, f)`, `exec_view_idle(b, f)` | модально с фоновой работой: `f(&mut Program, id) -> Option<u16>` вызывается, когда событий нет; `Some(cmd)` закрывает окно |
| `modal_top()` | id верхнего модального окна |
| `focus(id)` | передать фокус представлению и всем его владельцам |

```rust
use tvision::*;

/// Модальный диалог с работой «в фоне»: полоса доходит до конца сама.
fn countdown(app: &mut Program) -> u16 {
    let mut d = Dialog::centered(40, 8, "Подождите");
    let bar = d.add(ProgressBar::new(Rect::new(3, 2, 37, 3), 50));
    d.add(Button::new(Rect::new(14, 4, 26, 6), "Отмена", cm::CANCEL, bf::DEFAULT));
    let (cmd, _) = app.exec_dialog_idle(d, |app, _dialog_id| {
        let b = app.find_mut::<ProgressBar>(bar)?;
        b.step(1);
        b.is_done().then_some(cm::OK)
    });
    cmd // cm::OK — дошло до конца, cm::CANCEL — нажали «Отмена»
}
```

### Доступ к частям

| Метод / поле | Что |
|--------------|-----|
| `desktop()`, `desktop_mut()` | рабочий стол |
| `status_line_mut()` | строка состояния |
| `find::<T>(id)`, `find_mut::<T>(id)` | представление по id во всём дереве (в том числе в модальном диалоге) |
| `group` | корневая группа |
| `desktop_id`, `menu_bar_id`, `status_line_id` | id стандартных частей |
| `help_ctx()` | контекст справки представления в фокусе (для F1) |
| `screen()`, `screen_text()` | последний кадр (тесты) |

### Команды, настройки, оболочка

| Метод / поле | Что |
|--------------|-----|
| `enable_command(cmd)`, `disable_command(cmd)`, `command_enabled(cmd)` | доступность команд (запрещённые пункты — серые) |
| `palette: Vec<Attr>` | палитра приложения (по умолчанию `palettes::GRUVBOX`) |
| `ctx: Ctx` | контекст событий; буфер обмена — `app.ctx.clipboard` |
| `double_delay`, `reverse_buttons` | интервал двойного щелчка, левша |
| `event_monitor: Option<ViewId>` | представление, которому показываются все события (`View::monitor_event`) |
| `modal_hook` | обработчик команд, которые не обработало **модальное** окно |
| `dos_shell(banner)` | выйти в оболочку ОС (`$SHELL` / `cmd.exe`) и вернуться |
| `suspended(\|\| …)` | выполнить функцию в обычном режиме терминала (запустить `vim`, `less`) и вернуться |

### `modal_hook`

Во время модального диалога `AppHandler` не вызывается. Если кнопка
диалога посылает свою команду (не `OK`/`CANCEL`), её можно поймать здесь:

```rust
use tvision::*;

const CM_BROWSE: u16 = 130;

fn export_dialog(app: &mut Program) -> Option<String> {
    let mut d = Dialog::centered(52, 9, "Экспорт");
    let field = d.add(InputLine::new(Rect::new(3, 3, 36, 4), 200));
    d.add(Button::new(Rect::new(37, 2, 49, 4), "~О~бзор…", CM_BROWSE, bf::NORMAL));
    d.add(Button::new(Rect::new(12, 6, 24, 8), "O~K~", cm::OK, bf::DEFAULT));
    app.modal_hook = Some(Box::new(move |app, ev| {
        if ev.command() == Some(CM_BROWSE) {
            let (cmd, f) = app.exec_dialog(FileDialog::new("*", "Файл", "~И~мя", fd::OK_BUTTON, 0));
            if cmd != cm::CANCEL {
                if let Some(il) = app.find_mut::<InputLine>(field) {
                    il.set_text(&f.file_name().display().to_string());
                }
            }
            ev.clear();
        }
    }));
    let (cmd, d) = app.exec_dialog(d);
    app.modal_hook = None;
    (cmd == cm::OK).then(|| d.win.group.find::<InputLine>(field).unwrap().text())
}
```

| Константа | Что |
|-----------|-----|
| `program::DOUBLE_CLICK` | 400 мс — интервал двойного щелчка по умолчанию |

## `AppHandler`

**Код приложения** — то, что в C++ было переопределёнными методами
наследника `TApplication`.

| Метод | Когда | По умолчанию |
|-------|-------|--------------|
| `handle_event(&mut self, app, ev)` | событие, которое не обработали представления (обычно команды меню) | ничего |
| `idle(&mut self, app)` | событий нет (часы, фоновая работа, обновление команд); во время модального окна не вызывается | ничего |

Обработчик получает `&mut Program` — из него можно открывать диалоги,
вставлять окна, менять палитру. Не забывайте `ev.clear()` для
обработанных команд.

## `NoHandler`

Пустой обработчик: `app.run(&mut program::NoHandler)` (тип лежит в модуле `program`) — для программ, где всё
делают сами представления.

---

## `Desktop`

**Рабочий стол** — группа между строкой меню и строкой состояния: фон и
окна. Аналог `TDesktop`.

```rust
use tvision::*;

fn arrange(app: &mut Program) {
    app.desktop_mut().set_pattern('▒');
    let r = app.desktop().group.vs.extent(); // весь стол, от (0, 0)
    app.desktop_mut().tile(r); // мозаика окон с of::TILEABLE
}
```

| Метод / поле | Что |
|--------------|-----|
| `pattern()`, `set_pattern(c)` | узор фона (по умолчанию `░`) |
| `tile(r)`, `cascade(r)` | мозаика / каскад окон с `of::TILEABLE` |
| `tile_columns_first` | мозаика сначала по столбцам |
| `group` | группа: окна — её дети (`children`), активное — `current_view()` |
| `background_id` | id фона |

Сам обрабатывает `cm::NEXT` (F6), `cm::PREV`, `cm::TILE`, `cm::CASCADE`.
Координаты окон — от левого верхнего угла стола (первая строка под меню).

## `Background`

**Фон рабочего стола**: залит символом `pattern`. Аналог `TBackground`.
`Background::new(bounds, pattern)`; обычно меняется через
`desktop.set_pattern(c)`. Свой фон (картинку, часы) можно сделать своим
представлением и вставить на стол самым нижним.

---

## `Window`

**Окно**: рамка с заголовком, номером и значками, группа детей. Аналог
`TWindow`.

```rust
use tvision::*;

fn list_window(app: &mut Program) {
    let r = Rect::new(2, 1, 42, 15);
    let mut w = Window::new(r, "Список", 1); // номер 1 — Alt+1
    w.group.vs.options |= of::TILEABLE;
    let sb = w.standard_scroll_bar(true); // полоса на правой рамке
    let mut lb = ListBox::new(w.interior(), 1, Some(sb));
    lb.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y; // тянется с окном
    lb.set_items((1..=50).map(|i| format!("Строка {}", i)).collect());
    w.add(lb);
    app.insert_window(w);
}
```

| Метод / поле | Что |
|--------------|-----|
| `Window::new(bounds, title, number)` | окно (`number` 1–9 видно в рамке; 0 — без номера) |
| `add(v)`, `insert(box)` | вставить элемент (координаты — от угла окна, рамка — строка/столбец 0) |
| `interior()` | внутренность без рамки |
| `standard_scroll_bar(vertical) -> ViewId` | полоса прокрутки на правой / нижней рамке |
| `zoom(owner_size)` | распахнуть / восстановить |
| `title`, `number`, `flags`, `palette` | заголовок, номер, флаги `wf::*`, палитра (`BLUE_WINDOW`, `CYAN_WINDOW`, `GRAY_WINDOW`) |
| `group` | группа: дети, `vs` окна |
| `id()` | id окна |
| `handle_window_event(ev, ctx)`, `draw_window(dc)`, `set_window_state(flag, on)` | «родительские» методы — для своих окон на основе `Window` |

Окно само обрабатывает перемещение и изменение размера мышью, Ctrl+F5
(`cm::RESIZE`), F5 (`cm::ZOOM`), Alt+F3 (`cm::CLOSE`), выбор по Alt+номер.

### Своё окно

Своё окно — структура с полем `Window` и делегированием (как
`ContactsWindow` в [уроке](01-tutorial.md#шаг-2-своё-окно-с-таблицей)):

```rust
use tvision::*;

pub struct NotesWindow {
    pub win: Window,
    pub memo: ViewId,
}

impl View for NotesWindow {
    tvision::impl_group_basics!(win.group);

    fn as_window(&self) -> Option<&Window> {
        Some(&self.win) // без этого не работают Alt+F3, F5 и Alt+номер
    }
    fn draw(&self, dc: &mut DrawCtx) {
        self.win.draw_window(dc);
    }
    fn palette(&self) -> &'static [u8] {
        self.win.palette
    }
    fn set_state(&mut self, flag: u16, on: bool) {
        self.win.set_window_state(flag, on);
    }
    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        self.win.handle_window_event(ev, ctx);
        // …своя обработка того, что осталось в `ev`…
    }
}
```

## `wf` — флаги окна

`Window::flags`: `MOVE` — можно двигать, `GROW` — менять размер, `CLOSE` —
значок закрытия `[■]`, `ZOOM` — значок распахивания `[↑]`. У окна по
умолчанию все четыре, у диалога — `MOVE | CLOSE`.

`window::MIN_WIN_SIZE` — наименьший размер окна (16×6).

## `Frame` и `FrameInfo`

**Рамка окна.** В этом порте рамка — не отдельное представление, а
рисовальщик: `Frame::draw_frame(dc, &FrameInfo { title, number, flags,
active, dragging, zoomed })` рисует рамку на всю область `dc`, палитра —
`Frame::palette()`. Окно вызывает его само в `draw_window`; вручную это
нужно, только если вы рисуете окно без `Window`.

Значки рамки — константы `frame::CLOSE_ICON` (`[■]`), `ZOOM_ICON` (`[↑]`),
`UNZOOM_ICON` (`[↕]`), `DRAG_ICON`, `DRAG_LEFT_ICON`. Активное окно
рисуется двойной рамкой, неактивное — одинарной.

---

## `Dialog`

**Диалог** — окно с серой палитрой для элементов управления; обычно
выполняется модально. Аналог `TDialog`.

```rust
use tvision::*;

fn ask_name(app: &mut Program) -> Option<String> {
    let mut d = Dialog::centered(40, 9, "Знакомство");
    let name = d.add(InputLine::new(Rect::new(10, 2, 37, 3), 40));
    d.add(Label::new(Rect::new(2, 2, 9, 3), "~И~мя:", Some(name)));
    d.add(Button::new(Rect::new(6, 5, 18, 7), "O~K~", cm::OK, bf::DEFAULT));
    d.add(Button::new(Rect::new(21, 5, 33, 7), "Отмена", cm::CANCEL, bf::NORMAL));
    let (cmd, d) = app.exec_dialog(d);
    (cmd == cm::OK).then(|| d.win.group.find::<InputLine>(name).unwrap().text())
}
```

| Метод / поле | Что |
|--------------|-----|
| `Dialog::new(bounds, title)` | в заданных границах |
| `Dialog::centered(w, h, title)` | размера `w`×`h`, центрируется при вставке |
| `add(v)`, `insert(box)` | вставить элемент; **порядок = порядок Tab = порядок данных** |
| `select(id)` | элемент, который получит фокус первым |
| `set_palette(DialogPalette::…)` | цветовая схема |
| `get_data()`, `set_data(v)` | данные всех элементов (`Value::List`) |
| `win` | окно: `win.title`, `win.flags`, `win.group` |
| `id()` | id диалога |

Поведение:

* **Esc** — `cm::CANCEL`, **Enter** — кнопка по умолчанию;
* `cm::OK`, `CANCEL`, `YES`, `NO` завершают модальный диалог (после
  `valid`; «Отмена» допустима всегда — валидаторы её не задерживают);
* остальные команды кнопок проходят дальше — к `modal_hook` или к
  `AppHandler`.

Диалог можно вставить и немодально — `app.insert_window(d)`: тогда он
живёт на рабочем столе как окно, а его кнопки посылают команды
приложению.

## `DialogPalette`

Цветовая схема диалога: `Gray` (по умолчанию), `Blue`, `Cyan` — палитры
`GRAY_DIALOG`, `BLUE_DIALOG`, `CYAN_DIALOG`
(`dpGrayDialog`, `dpBlueDialog`, `dpCyanDialog`).
