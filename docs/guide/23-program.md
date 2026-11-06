# Справочник: программа, рабочий стол, окна, диалоги

Каркас любого приложения: сама программа, рабочий стол, окна и диалоги.

На этой странице:
[`Program`](#program--программа) ·
[`AppHandler`](#apphandler--код-приложения) ·
[`NoHandler`](#nohandler) ·
[`Desktop`](#desktop--рабочий-стол) ·
[`Background`](#background--фон) ·
[`Window`](#window--окно) ·
[`wf`](#wf--что-можно-делать-с-окном) ·
[`Frame`, `FrameInfo`](#frame-и-frameinfo--рамка) ·
[`Dialog`](#dialog--диалог) ·
[`DialogPalette`](#dialogpalette)

---

## `Program` — программа

**Что это.** Вся программа целиком: экран, строка меню, рабочий стол,
строка состояния, цикл событий, палитра, буфер обмена.

**Аналог:** `TProgram` и `TApplication`.

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

### Как создать

* `Program::with_terminal(меню, строка_состояния)` — в настоящем
  терминале. Меню — `Option<Menu>`, строка состояния —
  `Option<Vec<StatusDef>>`; любое можно не задавать (`None`).
* `Program::new(терминал, меню, строка_состояния)` — с любым
  терминалом, например невидимым для тестов ([урок 10](10-tests.md)).

### Запуск

| Метод | Что делает |
|-------|-----------|
| `run(&mut обработчик)` | работать, пока не выйдут (Alt+X, `cm::QUIT`, `quit()`) |
| `quit()` | выйти из `run` |
| `put_event(ev)` | поставить событие в очередь (например, команду при запуске) |
| `pump(&mut обработчик)` | обработать всё, что в очереди, и вернуться (тесты) |

### Окна и диалоги

| Метод | Что делает |
|-------|-----------|
| `insert_window(окно) -> ViewId` | положить окно на рабочий стол |
| `exec_dialog(диалог) -> (команда, диалог)` | показать модально, дождаться, вернуть диалог обратно |
| `exec_dialog_idle(диалог, функция)` | то же, и пока диалог открыт, вызывать `функция` — для фоновой работы с полосой выполнения |
| `find::<T>(id)`, `find_mut::<T>(id)` | найти представление по id где угодно, даже в открытом диалоге |
| `desktop()`, `desktop_mut()` | рабочий стол |

```rust
use tvision::*;

/// Диалог, который «работает» сам: полоса доходит до конца, и он закрывается.
fn countdown(app: &mut Program) -> u16 {
    let mut d = Dialog::centered(40, 8, "Подождите");
    let bar = d.add(ProgressBar::new(Rect::new(3, 2, 37, 3), 50));
    d.add(Button::new(Rect::new(14, 4, 26, 6), "Отмена", cm::CANCEL, bf::DEFAULT));
    let (cmd, _) = app.exec_dialog_idle(d, |app, _| {
        let b = app.find_mut::<ProgressBar>(bar)?;
        b.step(1);
        b.is_done().then_some(cm::OK) // Some(команда) — закрыть диалог
    });
    cmd
}
```

### Команды и настройки

| Метод / поле | Что |
|--------------|-----|
| `enable_command(cmd)`, `disable_command(cmd)`, `command_enabled(cmd)` | разрешить / запретить команду (запрещённые пункты серые) |
| `palette` | палитра программы — поменяйте, чтобы перекрасить всё |
| `ctx.clipboard` | буфер обмена |
| `ctx.screen_size` | размер экрана |

**Реже:** `step()` (один шаг цикла), `get_event()`, `handle(ev, h)`,
`redraw()`, `is_running()`, `exec_view` и `exec_view_idle` (то же, что
`exec_dialog`, для `Box<dyn View>`), `insert_window_boxed`, `modal_top()`,
`focus(id)` (перевести фокус на любое представление), `status_line_mut()`,
`help_ctx()` (номер темы справки элемента в фокусе), `screen()`,
`screen_text()` (тесты), `group`, `desktop_id`, `menu_bar_id`,
`status_line_id`, `double_delay` (интервал двойного щелчка),
`reverse_buttons` (мышь для левши), `event_monitor`, `dos_shell(текст)`
(выйти в оболочку ОС и вернуться), `suspended(|| …)` (запустить другую
программу в терминале — например, `vim` — и вернуться),
`program::DOUBLE_CLICK`.

### `modal_hook` — команды из открытого диалога

Пока открыт модальный диалог, `AppHandler` **не вызывается**. Если в
диалоге есть кнопка со своей командой (не `OK`/`CANCEL`), её ловят в
`app.modal_hook`:

```rust
use tvision::*;

const CM_BROWSE: u16 = 130;

/// Диалог «Экспорт» с кнопкой «Обзор…», которая открывает выбор файла.
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
    app.modal_hook = None; // не забыть убрать
    (cmd == cm::OK).then(|| d.win.group.find::<InputLine>(field).unwrap().text())
}
```

## `AppHandler` — код приложения

**Что это.** Трейт для вашего обработчика: сюда приходят команды,
которые не обработали окна и элементы.

**Аналог:** переопределённые методы наследника `TApplication`.

| Метод | Когда вызывается |
|-------|------------------|
| `handle_event(&mut self, app, ev)` | пришло необработанное событие (обычно команда из меню) |
| `idle(&mut self, app)` | событий нет: обновить часы, запретить ненужные команды |

Оба метода необязательны. `app` — программа: через неё показывают
диалоги, кладут окна, меняют палитру. Во время модального диалога
обработчик не вызывается (см. `modal_hook` выше).

## `NoHandler`

Пустой обработчик: `app.run(&mut program::NoHandler)` — для программ,
где всё делают сами окна. Лежит в модуле `program`.

---

## `Desktop` — рабочий стол

**Что это.** Область между строкой меню и строкой состояния: фон и окна.

**Аналог:** `TDesktop`.

```rust
use tvision::*;

fn arrange(app: &mut Program) {
    app.desktop_mut().set_pattern('▒'); // узор фона
    let r = app.desktop().group.vs.extent(); // весь рабочий стол
    app.desktop_mut().tile(r); // окна плиткой
}
```

| Метод / поле | Что |
|--------------|-----|
| `set_pattern(c)`, `pattern()` | символ фона (по умолчанию `░`) |
| `tile(r)`, `cascade(r)` | разложить окна плиткой / каскадом (окна с флагом `of::TILEABLE`) |
| `group` | окна — его дети; активное окно — `group.current_view()` |
| `group.vs.extent()` | прямоугольник всего стола — для окна «во весь стол» |

**Реже:** `tile_columns_first` (плитка сначала по столбцам),
`background_id`.

Сам обрабатывает команды `cm::NEXT` (следующее окно), `cm::PREV`,
`cm::TILE`, `cm::CASCADE`.

## `Background` — фон

**Что это.** Фон рабочего стола, залитый одним символом.
`Background::new(границы, символ)`. Обычно его не трогают, а меняют узор
через `desktop.set_pattern(c)`.

---

## `Window` — окно

**Что это.** Рамка с заголовком, номером и значками `[■]` (закрыть) и
`[↑]` (распахнуть) — и место для содержимого. Окна можно двигать,
растягивать, распахивать и закрывать — всё это они умеют сами.

**Аналог:** `TWindow`.

```rust
use tvision::*;

fn list_window(app: &mut Program) {
    let mut w = Window::new(Rect::new(2, 1, 42, 15), "Список", 1);
    let sb = w.standard_scroll_bar(true); // полоса прокрутки на правой рамке
    let mut lb = ListBox::new(w.interior(), 1, Some(sb));
    lb.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y; // тянуться вместе с окном
    lb.set_items((1..=50).map(|i| format!("Строка {}", i)).collect());
    w.add(lb);
    app.insert_window(w);
}
```

| Метод / поле | Что |
|--------------|-----|
| `Window::new(границы, заголовок, номер)` | окно; номер 1–9 виден в рамке (Alt+номер), 0 — без номера |
| `add(элемент) -> ViewId` | положить внутрь; координаты — от угла окна, рамка — строка и столбец 0 |
| `interior()` | место внутри рамки |
| `standard_scroll_bar(вертикальная) -> ViewId` | полоса прокрутки на правой (`true`) или нижней (`false`) рамке |
| `title` | заголовок — можно менять на ходу |
| `flags` | что можно делать с окном (`wf::…`) |
| `palette` | цвета: `palettes::BLUE_WINDOW` (по умолчанию), `CYAN_WINDOW`, `GRAY_WINDOW` |
| `group` | содержимое; `group.vs` — положение самого окна |

**Реже:** `insert(Box<dyn View>)`, `zoom(размер)`, `number`, `id()`;
`handle_window_event`, `draw_window`, `set_window_state` — для своих
окон ([урок 8](08-own-window.md)).

Окно само понимает: мышь за заголовок — двигать, за правый нижний угол —
менять размер, `cm::ZOOM` (F5) — распахнуть, `cm::CLOSE` (Alt+F3) —
закрыть, `cm::RESIZE` (Ctrl+F5) — двигать и тянуть клавишами.

### Своё окно

Своё окно с данными — структура с полем `Window` и шаблонной реализацией
`View`. Подробно — [урок 8](08-own-window.md):

```rust
use tvision::*;

pub struct NotesWindow {
    pub win: Window,
    pub notes: Vec<String>,
}

impl View for NotesWindow {
    tvision::impl_group_basics!(win.group);
    fn as_window(&self) -> Option<&Window> {
        Some(&self.win)
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
        // здесь — своя обработка того, что окно не забрало
    }
}
```

## `wf` — что можно делать с окном

Флаги в `window.flags`: `MOVE` — двигать, `GROW` — менять размер, `CLOSE`
— закрывать (значок `[■]`), `ZOOM` — распахивать (значок `[↑]`). У окна
по умолчанию все четыре, у диалога — `MOVE` и `CLOSE`. Окно, которое
нельзя закрыть: `w.flags &= !wf::CLOSE;`.

`window::MIN_WIN_SIZE` — наименьший размер окна (16×6).

## `Frame` и `FrameInfo` — рамка

**Что это.** То, что рисует рамку окна. Окно вызывает его само; вручную
нужно, только если вы рисуете окно без `Window`:
`Frame::draw_frame(dc, &FrameInfo { title, number, flags, active,
dragging, zoomed })`, палитра — `Frame::palette()`. Значки:
`frame::CLOSE_ICON` (`[■]`), `ZOOM_ICON` (`[↑]`), `UNZOOM_ICON` (`[↕]`),
`DRAG_ICON`, `DRAG_LEFT_ICON`. Активное окно — двойная рамка,
неактивное — одинарная.

---

## `Dialog` — диалог

**Что это.** Окно для вопросов пользователю: серое, с полями и кнопками.
Обычно показывается **модально** — пока его не закроют, остальная
программа ждёт.

**Аналог:** `TDialog`.

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
| `Dialog::centered(ширина, высота, заголовок)` | диалог, который встанет по центру |
| `Dialog::new(границы, заголовок)` | в заданном месте |
| `add(элемент) -> ViewId` | добавить; **порядок добавления = порядок Tab = порядок данных** |
| `select(id)` | какой элемент получит фокус первым |
| `set_data(v)`, `get_data()` | заполнить / прочитать все поля разом |
| `win.group.find::<T>(id)` | найти элемент — например, после `exec_dialog` |

**Реже:** `set_palette(DialogPalette::…)`, `insert(Box<dyn View>)`,
`id()`, `win.title`, `win.flags`.

**Как ведёт себя:**

* Esc — то же, что «Отмена» (`cm::CANCEL`);
* Enter — нажимает кнопку с флагом `bf::DEFAULT`;
* `cm::OK`, `cm::CANCEL`, `cm::YES`, `cm::NO` закрывают диалог (перед
  этим поля проверяются валидаторами — кроме «Отмены»);
* другие команды кнопок идут дальше — в `modal_hook` или, если диалог
  не модальный, в `AppHandler`.

Диалог можно положить и как обычное окно — `app.insert_window(d)`: он
останется на рабочем столе, пока его не закроют.

## `DialogPalette`

Цвета диалога: `Gray` (серый, по умолчанию), `Blue`, `Cyan`.
`d.set_palette(DialogPalette::Blue)`.
