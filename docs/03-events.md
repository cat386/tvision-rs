# 3. События и команды

## Тип `Event`

Событие — перечисление `Event` (аналог `TEvent`):

| Вариант | Аналог | Когда |
|---------|--------|-------|
| `Event::Nothing` | `evNothing` | событий нет или событие уже обработано |
| `Event::KeyDown(KeyEvent)` | `evKeyDown` | нажата клавиша |
| `Event::MouseDown(MouseEvent)` | `evMouseDown` | нажата кнопка мыши |
| `Event::MouseUp(MouseEvent)` | `evMouseUp` | отпущена кнопка мыши |
| `Event::MouseMove(MouseEvent)` | `evMouseMove` | мышь сдвинулась |
| `Event::MouseAuto(MouseEvent)` | `evMouseAuto` | кнопка удерживается (повторяется примерно 16 раз в секунду) |
| `Event::MouseWheel(MouseEvent)` | `evMouseWheel` | колесо мыши |
| `Event::Command(Message)` | `evCommand` | команда: идёт по цепочке фокуса |
| `Event::Broadcast(Message)` | `evBroadcast` | сообщение всем детям группы |

Полезные методы:

```rust
use tvision::*;

fn inspect(ev: &mut Event) {
    if let Some(cmd) = ev.command() { /* это Command с номером cmd */ let _ = cmd; }
    if let Some(m) = ev.broadcast() { /* это Broadcast */ let _ = (m.command, &m.info); }
    if let Some(k) = ev.key() { /* это KeyDown */ let _ = k; }
    if let Some(m) = ev.mouse() { /* любое событие мыши */ let _ = m.pos; }
    let _ = ev.is_mouse();   // позиционное событие
    let _ = ev.is_focused(); // клавиша или команда
    ev.clear();              // обработано (= Event::Nothing)
    let _ = ev.is_nothing();
    let _ = Event::command_event(cm::OK); // создать команду
}
```

## Клавиатура

Вместо 16-битных скан-кодов DOS (`kbF1 = 0x3B00`) клавиша описывается
понятной структурой:

```rust
use tvision::*;

fn keys() {
    let k = KeyEvent::new(Key::Char('s'), Mods::CTRL); // Ctrl+S
    assert!(k.is(kb::ctrl('S')));   // сравнение (регистр букв не важен)
    let _ = KeyEvent::new(Key::F(5), Mods::NONE); // F5
    let _ = KeyEvent::new(Key::Tab, Mods::SHIFT); // Shift+Tab
    let _ = k.name(); // "Ctrl+S"
}
```

`Key` — `Char(char)`, `Enter`, `Esc`, `Tab`, `BackTab`, `Backspace`,
`Up`, `Down`, `Left`, `Right`, `Home`, `End`, `PgUp`, `PgDn`, `Ins`,
`Del`, `F(1..=12)`. `Mods` — поля `shift`, `ctrl`, `alt` и константы
`Mods::NONE`, `SHIFT`, `CTRL`, `ALT`, `CTRL_SHIFT`.

Методы `KeyEvent`:

| Метод | Что возвращает |
|-------|----------------|
| `is(other)` | совпадает ли сочетание (буквы — без учёта регистра; `Shift+Tab` = `BackTab`) |
| `text_char()` | печатный символ без Ctrl/Alt — то, что вставляется в текст |
| `alt_char()` | буква для Alt+буква (в верхнем регистре) |
| `name()` | человекочитаемое имя: `"Alt+X"`, `"F10"` |

Модуль `kb` содержит конструкторы и частые сочетания:
`kb::alt('X')`, `kb::ctrl('N')`, `kb::f(9)`, `kb::alt_f(3)`,
`kb::ctrl_f(5)`, `kb::shift_f(6)`, `kb::key(Key::Del)` и константы
`kb::ENTER`, `ESC`, `TAB`, `SHIFT_TAB`, `F1`, `F2`, `F3`, `F5`, `F6`, `F10`,
`ALT_X`, `ALT_F3`, `CTRL_F5`, `SHIFT_F6`.

> На Windows Alt+буква и Ctrl+буква приходят **латинской** буквой той же
> клавиши, даже в русской раскладке (Alt+Ф — это `alt('a')`). Меню,
> метки, кнопки и флажки учитывают это сами — см. [главу 13](13-locale.md).

## Мышь

`MouseEvent`:

| Поле | Смысл |
|------|-------|
| `pos` | позиция в координатах **владельца** получателя (переведите: `self.vs.make_local(m.pos)`) |
| `buttons` | нажатые кнопки: `MB_LEFT` (1), `MB_RIGHT` (2), `MB_MIDDLE` (4) |
| `double` | двойной щелчок (интервал — `Program::double_delay`, по умолчанию 400 мс) |
| `mods` | Shift/Ctrl/Alt в момент события |
| `wheel` | колесо: −1 вверх, +1 вниз |

Перетаскивание делается захватом мыши: на `MouseDown` вызовите
`ctx.grab_mouse(self.vs.id)` — все последующие `MouseMove`/`MouseAuto`/
`MouseUp` придут вам, даже если курсор ушёл за пределы представления. На
`MouseUp` — `ctx.release_mouse()` (программа снимает захват и сама, когда
отпущены все кнопки).

```rust
use tvision::*;

struct Slider {
    vs: ViewState,
    value: i32,
}

impl View for Slider {
    tvision::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let w = self.vs.size.x.max(1);
        let c = dc.color(1);
        dc.write_char(0, 0, '─', c, w);
        dc.write_char(self.value.clamp(0, w - 1), 0, '█', c, 1);
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        let id = self.vs.id;
        match ev {
            Event::MouseDown(m) => {
                self.value = self.vs.make_local(m.pos).x;
                ctx.grab_mouse(id);
                ev.clear();
            }
            Event::MouseMove(m) | Event::MouseAuto(m) if ctx.mouse_grabbed_by(id) => {
                self.value = self.vs.make_local(m.pos).x.clamp(0, self.vs.size.x - 1);
                ev.clear();
            }
            Event::MouseUp(_) if ctx.mouse_grabbed_by(id) => {
                ctx.release_mouse();
                ev.clear();
            }
            _ => {}
        }
    }
}
```

## Команды

Команда — это число `u16` с необязательными данными (`Message { command, info }`).
Её посылают пункты меню, строка состояния, кнопки и программа. Команда
доставляется **по цепочке фокуса**: текущее окно → его текущий элемент
(с фазами pre/post-process), и если её никто не обработал — обработчику
приложения (`AppHandler::handle_event`) или, в модальном режиме,
`Program::modal_hook`.

Послать команду:

```rust
use tvision::*;

const CM_REFRESH: u16 = 120;

fn post(ctx: &mut Ctx, app: &mut Program) {
    ctx.post_command(CM_REFRESH, Info::None);           // из представления
    app.put_event(Event::command_event(CM_REFRESH));    // из программы
}
```

Стандартные номера (`cm::OK`, `cm::CLOSE`, `cm::QUIT`…) совпадают с
оригиналом — полный список в [справочнике](16-reference.md#cm--команды).
**Свои команды** нумеруйте от 100 до 255 (их можно запрещать) или от 1000
и выше (запретить нельзя, как и в оригинале).

### Данные `Info`

Вместо `void* infoPtr` / `long infoLong` у команды и сообщения есть
типизированное поле `info`:

| Вариант | Для чего |
|---------|----------|
| `Info::None` | без данных |
| `Info::Id(ViewId)` | id представления (кнопка посылает свой id; `cm::RECEIVED_FOCUS` — id получившего фокус) |
| `Info::Int(i64)` | число (номер окна для `cm::SELECT_WINDOW_NUM`) |
| `Info::Str(String)` | строка |
| `Info::Text { id, text }` | текст для конкретного представления (выбор из истории) |
| `Info::Indicator { .. }` | положение курсора редактора для индикатора |
| `Info::Scroll { id, value, min, max, page, arrow }` | параметры полосы прокрутки |

### Запрет команд

Команды 0–255 можно запрещать. Запрещённая команда:

* серым цветом рисуется в меню и строке состояния и не выбирается;
* делает серыми и неактивными кнопки с этой командой;
* не посылается клавишами меню.

```rust
use tvision::*;

const CM_SAVE_REPORT: u16 = 150;

fn update(app: &mut Program, have_report: bool) {
    if have_report {
        app.enable_command(CM_SAVE_REPORT);
    } else {
        app.disable_command(CM_SAVE_REPORT);
    }
}
```

Из представления — `ctx.enable_command` / `ctx.disable_command` /
`ctx.set_command(cmd, bool)`. Когда набор команд меняется, всем
представлениям рассылается `cm::COMMAND_SET_CHANGED`.

Представление может держать свои команды в актуальном состоянии методом
`View::update_commands(&self, ctx)`: программа вызывает его после каждого
события — сначала у представлений вне фокуса, затем у представлений в
фокусе, так что команды «в фокусе» побеждают. Так редактор разрешает
`cm::CUT` и `cm::COPY` только при выделенном тексте:

```rust
use tvision::*;

struct Doc {
    vs: ViewState,
    has_selection: bool,
}

impl View for Doc {
    tvision::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        dc.fill(self.vs.extent(), ' ', dc.color(1));
    }

    fn update_commands(&self, ctx: &mut Ctx) {
        if self.vs.is_focused() {
            ctx.set_command(cm::COPY, self.has_selection);
            ctx.set_command(cm::CUT, self.has_selection);
        }
    }
}
```

Программа сама разрешает и запрещает команды окон (`cm::CLOSE`, `cm::ZOOM`,
`cm::RESIZE`, `cm::NEXT`, `cm::PREV`) по флагам активного окна.

## Широковещательные сообщения

`Broadcast` получают **все** дети группы по порядку (пока кто-то не
обработает). Представление рассылает сообщение соседям вызовом
`ctx.message(cmd, info)` — оно уйдёт детям владельца сразу после возврата
из обработчика.

Сообщения, которые рассылает сама библиотека:

| Сообщение | Кто шлёт | Данные |
|-----------|----------|--------|
| `cm::RECEIVED_FOCUS` / `cm::RELEASED_FOCUS` | группа при смене текущего ребёнка | `Info::Id` ребёнка |
| `cm::COMMAND_SET_CHANGED` | программа при изменении набора команд и при вставке окна | — |
| `cm::SCROLL_BAR_CHANGED` | полоса прокрутки при изменении значения | `Info::Scroll` |
| `cm::SCROLL_BAR_CLICKED` | полоса прокрутки при щелчке | `Info::Id` полосы |
| `cm::SCROLL_BAR_SET` | список/редактор/прокрутка — полосе: новые параметры | `Info::Scroll` |
| `cm::LIST_ITEM_SELECTED` | список: двойной щелчок или пробел | `Info::Id` списка |
| `cm::DEFAULT` | диалог по Enter — кнопке по умолчанию | — |
| `cm::GRAB_DEFAULT` / `cm::RELEASE_DEFAULT` | кнопки при смене кнопки по умолчанию | — |
| `cm::RECORD_HISTORY` | кнопка перед нажатием — строкам ввода с историей | — |
| `cm::SELECT_WINDOW_NUM` | программа по Alt+цифра | `Info::Int` номер |

Пример: окно, которое реагирует на выбор элемента списка внутри себя.

```rust
use tvision::*;

/// Строка состояния окна: показывает, что выбрал пользователь.
struct Echo {
    vs: ViewState,
    list: ViewId,
    picks: usize,
}

impl View for Echo {
    tvision::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let c = dc.color(1);
        dc.fill(self.vs.extent(), ' ', c);
        dc.write_str(0, 0, &format!("Выбрано раз: {}", self.picks), c);
    }

    fn handle_event(&mut self, ev: &mut Event, _ctx: &mut Ctx) {
        if let Event::Broadcast(m) = ev {
            if m.command == cm::LIST_ITEM_SELECTED && m.info == Info::Id(self.list) {
                self.picks += 1;
                // Не очищаем: сообщение могут ждать и другие.
            }
        }
    }
}
```

## Очередь событий

`ctx.put_event(ev)` и `app.put_event(ev)` ставят событие в очередь — оно
будет получено на следующем шаге **раньше** событий терминала. Так кнопки
посылают свои команды, а тесты подкладывают действия пользователя.

## Наблюдение за всеми событиями

`Program::event_monitor = Some(id)` — представление `id` будет получать
копию **каждого** события через `View::monitor_event(&mut self, &Event)`
(так устроено окно Event Viewer в `tvdemo`). Если представление закрыли,
наблюдение снимается само.
