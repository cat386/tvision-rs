# 5. Программа, рабочий стол и окна

## `Program`

`Program` — корень всего (аналог `TProgram`/`TApplication`): владеет
деревом представлений, терминалом, очередью событий и палитрой.

### Создание

```rust
use tvision::*;

fn create() -> std::io::Result<Program> {
    // Настоящий терминал; меню и строка состояния — по желанию.
    Program::with_terminal(None, None)
}
```

* `Program::with_terminal(menu, status)` — настоящий терминал текущей ОС;
* `Program::new(backend, menu, status)` — любой `Backend`, например
  `platform::HeadlessBackend` для тестов ([глава 14](14-testing.md)).

`menu: Option<Menu>` и `status: Option<Vec<StatusDef>>` задают строку меню
и строку состояния ([глава 6](06-menus.md)). Без меню рабочий стол
начинается с верхней строки экрана, без строки состояния — доходит до
нижней.

### Поля и методы

| Поле / метод | Назначение |
|--------------|-----------|
| `run(&mut handler)` | главный цикл до `cm::QUIT` (`TProgram::run`) |
| `step(&mut handler)` | один шаг цикла; `false`, если событий не было |
| `pump(&mut handler)` | обработать все накопившиеся события (для тестов) |
| `quit()`, `is_running()` | завершить цикл / идёт ли он |
| `insert_window(w)`, `insert_window_boxed(b)` | вставить окно на рабочий стол, сделать текущим; возвращает id (0, если `valid(cm::VALID)` отказал) |
| `exec_dialog(d) -> (cmd, d)` | модально выполнить представление и вернуть его того же типа |
| `exec_view(b) -> (cmd, b)` | то же для `Box<dyn View>` (`execView`) |
| `modal_top()` | id верхнего модального представления, если идёт `exec_*` |
| `desktop()`, `desktop_mut()` | рабочий стол |
| `status_line_mut()` | строка состояния |
| `find::<T>(id)`, `find_mut::<T>(id)` | представление по id во всём дереве |
| `help_ctx()` | контекст справки представления в фокусе (для F1) |
| `enable_command`, `disable_command`, `command_enabled` | доступность команд |
| `put_event(ev)` | поставить событие в очередь |
| `dos_shell(banner)` | выход в оболочку ОС (`$SHELL` / `cmd.exe`) и возврат |
| `redraw()` | перерисовать экран сейчас |
| `screen()`, `screen_text()` | последний кадр (буфер / текст) |
| `group` | корневая группа (рабочий стол, строка состояния, меню и ваши «гаджеты») |
| `ctx` | контекст событий (буфер обмена: `app.ctx.clipboard`) |
| `palette` | палитра приложения `Vec<u8>` — её можно менять на ходу |
| `desktop_id`, `menu_bar_id`, `status_line_id` | id стандартных частей |
| `double_delay` | интервал двойного щелчка (по умолчанию 400 мс) |
| `reverse_buttons` | поменять местами левую и правую кнопки мыши |
| `event_monitor` | представление, которому показываются все события |
| `modal_hook` | обработчик необработанных событий во время модального цикла |

## Обработчик приложения `AppHandler`

```rust
use tvision::*;

struct MyApp {
    windows: i16,
}

impl AppHandler for MyApp {
    /// События, которые не обработали представления (команды меню и т.п.).
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        match ev.command() {
            Some(cm::NEW) => {
                self.windows += 1;
                let w = Window::new(Rect::new(1, 1, 40, 12), "Новое", self.windows);
                app.insert_window(w);
            }
            _ => return, // не наше — не трогаем
        }
        ev.clear();
    }

    /// Нет событий — можно сделать фоновую работу (`idle`).
    fn idle(&mut self, _app: &mut Program) {}
}
```

Обработчик получает `&mut Program`, поэтому из него можно открывать окна,
выполнять диалоги, менять палитру, читать представления.

> Во время модального диалога (`exec_dialog`) `AppHandler` не вызывается.
> Для этого случая есть `Program::modal_hook` — см. ниже.

## Модальные диалоги

```rust
use tvision::*;

fn confirm(app: &mut Program) -> bool {
    message_box(app, "\x03Удалить файл?", mf::CONFIRMATION | mf::YES_BUTTON | mf::NO_BUTTON) == cm::YES
}
```

`exec_dialog(d)`:

1. вставляет `d` на рабочий стол (с `of::CENTERED` — по центру) и ставит
   флаг `sf::MODAL`;
2. крутит цикл событий, пока представление не вызовет `ctx.end_modal(cmd)`
   (диалог делает это на `cm::OK`, `cm::CANCEL`, `cm::YES`, `cm::NO`);
3. спрашивает `valid(cmd)` — при отказе цикл продолжается;
4. вынимает представление и возвращает `(cmd, d)`.

Модальные циклы вкладываются: из `exec_dialog` можно открыть ещё один.

### `modal_hook`: команды во время модального цикла

Пусть в диалоге есть кнопка «Обзор…», которая должна открыть файловый
диалог. Кнопка посылает команду; диалог её не знает; во время модального
цикла `AppHandler` не вызывается — команда попадёт в `modal_hook`:

```rust
use tvision::*;

const CM_BROWSE: u16 = 1300;

/// Диалог с полем имени файла и кнопкой «Обзор…».
fn pick_file(app: &mut Program) -> Option<String> {
    let mut d = Dialog::centered(52, 9, "Экспорт");
    let field = d.add(InputLine::new(Rect::new(3, 3, 36, 4), 200));
    d.add(Label::new(Rect::new(2, 2, 20, 3), "~Ф~айл:", Some(field)));
    d.add(Button::new(Rect::new(37, 2, 49, 4), "~О~бзор…", CM_BROWSE, bf::NORMAL));
    d.add(Button::new(Rect::new(12, 6, 24, 8), "O~K~", cm::OK, bf::DEFAULT));
    d.add(Button::new(Rect::new(26, 6, 38, 8), "Отмена", cm::CANCEL, bf::NORMAL));

    // Команда кнопки дойдёт сюда: диалог её не обрабатывает.
    app.modal_hook = Some(Box::new(move |app, ev| {
        if ev.command() == Some(CM_BROWSE) {
            let fd = FileDialog::new("*", "Выберите файл", "~И~мя", fd::OK_BUTTON, 0);
            let (cmd, fd) = app.exec_dialog(fd); // вложенный модальный цикл
            if cmd != cm::CANCEL {
                let name = fd.file_name().display().to_string();
                // Поле лежит в диалоге на рабочем столе — find его найдёт.
                if let Some(il) = app.find_mut::<InputLine>(field) {
                    il.set_text(&name);
                }
            }
            ev.clear();
        }
    }));
    let (cmd, d) = app.exec_dialog(d);
    app.modal_hook = None;
    if cmd == cm::OK {
        d.win.group.find::<InputLine>(field).map(|il| il.text())
    } else {
        None
    }
}
```

Другой путь — запускать вложенное окно из самого представления через
`ctx.exec` ([глава 2](02-architecture.md#модальные-окна-изнутри-представления)).

## Рабочий стол `Desktop`

Рабочий стол — группа между меню и строкой состояния: фон
(`Background`) и окна.

| Метод / поле | Что делает |
|--------------|-----------|
| `pattern()`, `set_pattern(c)` | узор фона (по умолчанию `░`) |
| `tile(r)`, `cascade(r)` | мозаика / каскад окон с `of::TILEABLE` |
| `tile_columns_first` | мозаика сначала по столбцам |
| `group` | группа: окна — её дети |
| `background_id` | id фона |

Рабочий стол сам обрабатывает команды `cm::NEXT` (F6 — следующее окно),
`cm::PREV`, `cm::TILE`, `cm::CASCADE`. Чтобы окно участвовало в мозаике и
каскаде, поставьте ему `of::TILEABLE`.

```rust
use tvision::*;

fn arrange(app: &mut Program) {
    app.desktop_mut().set_pattern('▒');
    let r = app.desktop().group.vs.extent();
    app.desktop_mut().tile(r);
}
```

Границы окна задаются в координатах рабочего стола: `(0, 0)` — первая
строка под меню. Размер стола — `app.desktop().group.vs.size`.

## Окно `Window`

```rust
use tvision::*;

fn list_window(app: &mut Program) {
    let mut w = Window::new(Rect::new(10, 3, 50, 18), "Список", 1);
    w.group.vs.options |= of::TILEABLE;
    w.flags = wf::MOVE | wf::GROW | wf::CLOSE | wf::ZOOM; // так по умолчанию
    let sb = w.standard_scroll_bar(true); // вертикальная полоса на рамке
    let mut lb = ListBox::new(w.interior(), 1, Some(sb));
    lb.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
    lb.set_items((1..=100).map(|i| format!("Строка {}", i)).collect());
    w.add(lb);
    app.insert_window(w);
}
```

| Поле / метод | Назначение |
|--------------|-----------|
| `Window::new(bounds, title, number)` | окно; `number` 1–9 показывается в рамке и выбирается Alt+цифра (0 — без номера) |
| `title`, `number`, `flags` | заголовок, номер, флаги `wf::*` |
| `palette` | палитра: `palettes::BLUE_WINDOW` (по умолчанию), `CYAN_WINDOW`, `GRAY_WINDOW` |
| `group` | группа элементов окна |
| `add(v)`, `insert(box)` | вставить элемент (координаты — от угла окна, рамка — строка/столбец 0) |
| `interior()` | внутренняя область без рамки |
| `standard_scroll_bar(vertical)` | полоса прокрутки на рамке (справа или снизу); возвращает id |
| `zoom(owner_size)` | распахнуть / восстановить |
| `id()` | id окна |
| `handle_window_event`, `draw_window`, `set_window_state` | общая часть поведения — для своих окон-обёрток |

Флаги `wf`: `MOVE` (можно двигать), `GROW` (менять размер), `CLOSE`
(кнопка закрытия `[■]`), `ZOOM` (кнопка распахивания `[↑]`).

Что окно умеет само:

* мышь: перетаскивание за заголовок, изменение размера за правый нижний
  угол, кнопки `[■]` и `[↑]`, двойной щелчок по заголовку — распахнуть;
* команды: `cm::CLOSE` (Alt+F3), `cm::ZOOM` (F5), `cm::RESIZE` (Ctrl+F5 —
  перемещение стрелками, Shift+стрелки — размер, Enter или Esc —
  закончить);
* Tab / Shift+Tab — переход между элементами;
* Alt+цифра — стать текущим.

Перед закрытием окно спрашивает `valid(cm::CLOSE)` у себя и всех элементов
(редактор так спрашивает «Сохранить изменения?»). Модальное окно на
`cm::CLOSE` завершается как «Отмена».

### Своё окно

Чтобы добавить окну поведение, оберните `Window` и делегируйте ему:

```rust
use tvision::*;

/// Окно, которое считает нажатия F7.
struct CountingWindow {
    win: Window,
    presses: u32,
}

impl View for CountingWindow {
    tvision::impl_group_basics!(win.group);

    fn as_window(&self) -> Option<&Window> {
        Some(&self.win) // нужно программе для команд окна
    }
    fn draw(&self, dc: &mut DrawCtx) {
        self.win.draw_window(dc);
    }
    fn palette(&self) -> &'static [u8] {
        self.win.palette
    }
    fn set_state(&mut self, flag: u16, enable: bool) {
        self.win.set_window_state(flag, enable);
    }
    fn size_limits(&self, owner: Point) -> (Point, Point) {
        self.win.size_limits(owner)
    }
    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        self.win.handle_window_event(ev, ctx);
        if ev.key().map(|k| k.is(kb::f(7))).unwrap_or(false) {
            self.presses += 1;
            ev.clear();
        }
    }
    fn valid(&mut self, cmd: u16, ctx: &mut Ctx) -> bool {
        self.win.valid(cmd, ctx)
    }
}
```

`as_window` обязателен: по нему программа узнаёт флаги активного окна и
разрешает `cm::CLOSE`, `cm::ZOOM`, `cm::RESIZE`.

## Свои элементы в строке меню и строке состояния

В корневую группу можно добавить свои представления поверх стандартных —
так в `tvdemo` сделаны часы и счётчик памяти:

```rust
use tvision::*;

struct Clock {
    vs: ViewState,
}

impl View for Clock {
    tvision::impl_view_basics!();
    fn draw(&self, dc: &mut DrawCtx) {
        let t = platform::time::now();
        let c = dc.color(2);
        dc.fill(self.vs.extent(), ' ', c);
        dc.write_str(0, 0, &format!("{:02}:{:02}:{:02}", t.hour, t.minute, t.second), c);
    }
}

fn add_clock(app: &mut Program) {
    let w = app.group.vs.size.x;
    let mut vs = ViewState::new(Rect::new(w - 9, 0, w, 1));
    vs.grow_mode = gf::GROW_LO_X | gf::GROW_HI_X; // держаться правого края
    app.group.add(Clock { vs });
}
```

Экран перерисовывается на каждом шаге цикла (не реже 10 раз в секунду
при простое), поэтому часы просто читают время при отрисовке.

## Изменение размера терминала

Когда окно терминала меняет размер, программа пересоздаёт экранный буфер
и пересчитывает дерево по `grow_mode`: рабочий стол растягивается, окна
(`gf::GROW_REL`) масштабируются пропорционально, строка состояния
остаётся внизу. Ничего делать не нужно.
