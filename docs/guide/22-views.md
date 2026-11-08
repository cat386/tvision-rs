# Справочник: представления

**Представление** (view) — всё, что видно на экране: кнопка, поле,
окно, рабочий стол. Здесь — как они устроены: трейт `View`, общие поля
`ViewState`, связь с программой через `Ctx`, рисование через `DrawCtx` и
представления с детьми — `Group`.

Если вы ещё не писали своих представлений, начните с
[урока 7](07-own-view.md) и [урока 8](08-own-window.md).

На этой странице:
[`View`](#view) ·
[`ViewState`](#viewstate) ·
[`ViewId`](#viewid) ·
[флаги](#флаги-sf-of-gf-dm) ·
[`Value`](#value) ·
[`Ctx`](#ctx) ·
[`Request`, `PlaceFn`](#request-и-placefn) ·
[`CommandSet`](#commandset) ·
[`Phase`](#phase) ·
[`DrawCtx`](#drawctx) ·
[`Group`](#group) ·
[макросы](#макросы) ·
[вспомогательные функции](#вспомогательные-функции) ·
[координаты подробно](#координаты-подробно)

---

## `View`

**Что это.** Трейт, который реализует каждое представление. Он отвечает
на два главных вопроса: **как себя нарисовать** (`draw`) и **что делать
с событием** (`handle_event`). Остальные методы нужны реже, и у них есть
поведение по умолчанию.

**Аналог:** класс `TView`. В C++ от него наследовались; в Rust
реализуют трейт.

### Пример: «оценка» звёздочками

Пять звёзд `★★★☆☆`: стрелки ←/→ и щелчок меняют оценку, а значение
участвует в обмене данными диалога, как у обычного поля.

```rust
use tvision::*;

pub struct Rating {
    pub vs: ViewState,
    pub value: u32, // 0..=5
}

impl Rating {
    pub fn new(x: i32, y: i32) -> Self {
        let mut vs = ViewState::new(Rect::new(x, y, x + 5, y + 1));
        vs.options |= of::SELECTABLE | of::FIRST_CLICK;
        Rating { vs, value: 3 }
    }
}

impl View for Rating {
    tvision::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let a = dc.color(if self.vs.is_focused() { 2 } else { 1 });
        let s: String = (0..5).map(|i| if i < self.value { '★' } else { '☆' }).collect();
        dc.write_str(0, 0, &s, a);
    }

    fn palette(&self) -> &'static [u8] {
        color::palettes::INPUT_LINE // 1 — обычный цвет, 2 — в фокусе
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        view::default_handle_event(&self.vs, ev, ctx);
        match ev {
            Event::MouseDown(m) => {
                let p = self.vs.make_local(m.pos);
                self.value = (p.x + 1).clamp(0, 5) as u32;
                ev.clear();
            }
            Event::KeyDown(k) if self.vs.is_focused() => {
                match k.key {
                    Key::Left => self.value = self.value.saturating_sub(1),
                    Key::Right => self.value = (self.value + 1).min(5),
                    _ => return,
                }
                ev.clear();
            }
            _ => {}
        }
    }

    // Обмен данными: значение — число звёзд.
    fn has_data(&self) -> bool {
        true
    }
    fn get_data(&self) -> Value {
        Value::U32(self.value)
    }
    fn set_data(&mut self, v: &Value) {
        self.value = v.as_u32().unwrap_or(0).min(5);
    }
}

fn rating_test() {
    let mut r = Rating::new(0, 0);
    r.set_data(&Value::U32(9));
    assert_eq!(r.get_data(), Value::U32(5)); // больше пяти не бывает
}
```

### Методы трейта

**Почти всегда нужны:**

| Метод | Когда вызывается | Что писать |
|-------|------------------|-----------|
| `draw(&self, dc)` | при каждой перерисовке | нарисовать всю свою область |
| `handle_event(&mut self, ev, ctx)` | пришло событие | обработать своё и вызвать `ev.clear()` |
| `palette(&self)` | при рисовании | палитра для `dc.color(n)`; можно взять у похожего элемента |
| служебные `vs`, `vs_mut`, `as_any`, `as_any_mut`, `into_any` | библиотеке нужен доступ | пишет макрос `impl_view_basics!()` |

**Для элементов диалога:**

| Метод | Зачем |
|-------|------|
| `has_data`, `get_data`, `set_data` | участвовать в обмене данными диалога (`Value`) |
| `valid(&mut self, cmd, ctx) -> bool` | можно ли закрыть окно: `false` — не закрывать (так работают валидаторы) |

**Редко нужны:**

| Метод | Зачем |
|-------|------|
| `set_state(flag, on)` | узнать, что представление получило или потеряло фокус и т. п. |
| `update_commands(&self, ctx)` | разрешать и запрещать команды, пока представление в фокусе |
| `exec_result(tag, cmd, view, ctx)` | получить ответ модального окна, открытого через `ctx.exec` (см. [`Ctx`](#ctx)) |
| `change_bounds(r)`, `size_limits(owner)` | изменение размера и его пределы |
| `size_hint()`, `size_policy()` | желаемый размер и политика для [раскладок](33-layout.md#свой-элемент-в-раскладке) |
| `help_ctx()` | номер темы справки для F1 |
| `monitor_event(ev)` | видеть все события программы (для отладочного окна) |
| `as_window()`, `as_group()`, `as_group_mut()` | «я окно», «у меня есть дети» — пишут макросы и шаблон окна |
| `contains_id(id)` | есть ли `id` среди детей |

### Как найти представление нужного типа

Представления хранятся как «какое-то представление» — `Box<dyn View>`.
Чтобы получить свой тип, есть три способа:

* по id в группе: `group.find::<InputLine>(id)` или `find_mut`;
* по id во всей программе: `app.find::<InputLine>(id)`;
* из `dyn View`: `v.as_any().downcast_ref::<InputLine>()`.

```rust
use tvision::*;

fn text_of(v: &dyn View) -> Option<String> {
    v.as_any().downcast_ref::<InputLine>().map(|il| il.text())
}
```

---

## `ViewState`

**Что это.** Общие поля любого представления: где лежит, какого
размера, в каком состоянии. Хранится в поле `vs`.

**Аналог:** поля класса `TView` (`origin`, `size`, `state`, `options`…).

| Поле | Что хранит |
|------|-----------|
| `id` | номер представления — выдаётся сам |
| `origin` | левый верхний угол **в координатах владельца** |
| `size` | ширина и высота |
| `state` | флаги состояния `sf::…` (видно ли, в фокусе ли) |
| `options` | флаги поведения `of::…` (может ли получать фокус) |
| `grow_mode` | как меняться, когда владелец меняет размер (`gf::…`) |
| `drag_mode` | как перетаскивать (`dm::…`) |
| `cursor` | где текстовый курсор (в своих координатах) |
| `help_ctx` | номер темы справки |

| Метод | Что делает |
|-------|-----------|
| `ViewState::new(rect)` | создать для прямоугольника; представление видимо |
| `bounds()` | прямоугольник в координатах владельца |
| `extent()` | прямоугольник в своих координатах: от `(0, 0)` до размера |
| `set_bounds(r)` | задать положение и размер |
| `make_local(p)` | точку владельца — в свои координаты (для мыши) |
| `is_focused()` | в фокусе (получает клавиши) |
| `is_selected()`, `is_active()`, `is_visible()`, `is_disabled()`, `is_selectable()` | другие состояния |
| `get_state(флаг)` | установлен ли флаг |
| `show_cursor()`, `hide_cursor()`, `set_cursor(x, y)`, `block_cursor()`, `normal_cursor()` | текстовый курсор |

## `ViewId`

**Что это.** Номер представления (`u32`). Его возвращают `add` и
`insert_window`; по нему потом находят представление (`find`,
`find_mut`). Номер выдаётся сам и не повторяется.

> Почему номер, а не ссылка? Представлением владеет окно; пока вы держите
> ссылку, Rust не даст окну им пользоваться. Номер можно хранить сколько
> угодно.

## Флаги: `sf`, `of`, `gf`, `dm`

### `sf` — состояние (`vs.state`)

Меняет программа, вы обычно только **читаете** — через методы
`is_focused()` и т. п.

| Флаг | Значит |
|------|--------|
| `VISIBLE` | видно |
| `SELECTED` | выбрано в своём окне |
| `FOCUSED` | получает клавиши (выбрано, и окно активно) |
| `ACTIVE` | лежит в активном окне |
| `DISABLED` | недоступно |
| `MODAL` | выполняется как модальное |
| `CURSOR_VIS`, `CURSOR_INS` | курсор виден; курсор-блок |
| `SHADOW` | рисуется тень (у окон) |
| `DRAGGING`, `DEFAULT`, `EXPOSED` | служебные |

### `of` — поведение (`vs.options`)

Задаёте **вы** при создании.

| Флаг | Значит |
|------|--------|
| `SELECTABLE` | может получить фокус (по Tab или щелчку) |
| `FIRST_CLICK` | первый щелчок по невыбранному тоже обрабатывается (иначе он только выбирает) |
| `TOP_SELECT` | при выборе поднимается наверх (окна) |
| `TILEABLE` | участвует в «Мозаике» и «Каскаде» |
| `CENTER_X`, `CENTER_Y`, `CENTERED` | встать по центру при вставке |
| `PRE_PROCESS`, `POST_PROCESS` | видеть клавиши раньше / позже элемента в фокусе (строка состояния, кнопки) |
| `VALIDATE` | проверять при уходе фокуса |

### `gf` — растягивание (`vs.grow_mode`)

Когда окно растягивают, каждый элемент сдвигает свои края по этим
флагам: `GROW_LO_X` — левый край, `GROW_LO_Y` — верхний, `GROW_HI_X` —
правый, `GROW_HI_Y` — нижний. `GROW_ALL` — все четыре (элемент едет вместе
с правым нижним углом), `GROW_REL` — пропорционально. Подробнее — в
[координатах](#растягивание).

### `dm` — перетаскивание (`vs.drag_mode`)

`DRAG_MOVE`, `DRAG_GROW`, `LIMIT_ALL` и т. п. Нужны только окнам; что
пользователю можно делать с окном, проще задать флагами окна `wf::…`.

---

## `Value`

**Что это.** Данные элемента диалога для обмена (`get_data`/`set_data`).

| Вариант | У кого |
|---------|--------|
| `Str(String)` | строка ввода, многострочное поле, редактор |
| `U32(u32)` | флажки (биты), переключатели (номер) |
| `Int(i64)` | список (номер строки), числовое поле, полоса выполнения |
| `List(Vec<Value>)` | диалог или группа: значения всех элементов по порядку |
| `None` | данных нет |

Методы `as_str()`, `as_u32()`, `as_int()`, `as_list()` возвращают
`Some(…)`, если вариант подходит.

```rust
use tvision::*;

fn values() {
    let v = Value::List(vec![Value::Str("Аня".into()), Value::U32(0b101)]);
    let l = v.as_list().unwrap();
    assert_eq!(l[0].as_str(), Some("Аня"));
    assert_eq!(l[1].as_u32(), Some(5));
    assert_eq!(l[1].as_int(), None); // не тот вариант
}
```

Как этим пользоваться — [урок 6](06-form.md#обмен-данными).

---

## `Ctx`

**Что это.** «Связь с внешним миром» для представления. Представление
не знает ни своего окна, ни программы — когда ему нужно что-то сделать
снаружи себя, оно **просит** через `Ctx`, а программа выполняет просьбу
сразу после того, как `handle_event` закончится.

`Ctx` передаётся в `handle_event`, `valid`, `exec_result` и
`update_commands`.

**Чаще всего нужно:**

| Метод | Что делает |
|-------|-----------|
| `post_command(cmd, Info::None)` | послать команду — она придёт другим представлениям и в `AppHandler` |
| `message(cmd, info)` | разослать сообщение соседям по окну |
| `select(id)` | перевести фокус на соседа |
| `close(id)` | закрыть (удалить) представление, например своё окно |
| `exec(Box::new(диалог), Some(self.vs.id), метка)` | открыть модальное окно; ответ придёт в `exec_result` |
| `with_program(\|app\| …)` | сделать что-то с программой целиком |
| `clipboard` | буфер обмена: `ctx.clipboard.set("…")`, `get()` |

**Реже:** `focus(id)` (фокус куда угодно), `select_next(вперёд)`,
`make_first(id)`, `end_modal(cmd)`, `put_event(ev)`, `popup_menu(menu,
pos)` (контекстное меню; позиция — на экране), `exec_near`, `exec_placed`
(окно рядом с другим представлением), `grab_mouse(id)`,
`release_mouse()`, `mouse_grabbed_by(id)` (захват мыши для протяжки),
`command_enabled`, `enable_command`, `disable_command`, `set_command`,
`owner_size()`, поля `screen_size`, `phase`.

### Модальное окно изнутри представления

Программа может показать диалог и дождаться ответа (`exec_dialog`), но
только у неё есть для этого `&mut Program`. Представление поэтому
**просит** открыть окно и получает ответ позже — в метод `exec_result`:

```rust
use tvision::*;

const TAG_NAME: u16 = 1;

/// Надпись: щелчок спрашивает имя и показывает его.
pub struct AskName {
    pub vs: ViewState,
    pub name: String,
}

impl View for AskName {
    tvision::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let a = dc.color(1);
        dc.fill(self.vs.extent(), ' ', a);
        let t = if self.name.is_empty() { "[ввести имя]" } else { self.name.as_str() };
        dc.write_str(0, 0, t, a);
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        if let Event::MouseDown(_) = ev {
            let mut d = Dialog::centered(40, 8, "Имя");
            d.add(InputLine::new(Rect::new(3, 2, 37, 3), 30));
            d.add(Button::new(Rect::new(14, 4, 26, 6), "O~K~", cm::OK, bf::DEFAULT));
            // «Откройте, пожалуйста, и ответ пришлите мне с меткой TAG_NAME».
            ctx.exec(Box::new(d), Some(self.vs.id), TAG_NAME);
            ev.clear();
        }
    }

    fn exec_result(&mut self, tag: u16, cmd: u16, view: Box<dyn View>, _ctx: &mut Ctx) {
        if tag == TAG_NAME && cmd == cm::OK {
            let data = view.get_data(); // [Str(имя)]
            self.name = data.as_list().and_then(|l| l.first()).and_then(|v| v.as_str()).unwrap_or("").to_string();
        }
    }
}
```

Метка (`tag`) нужна, если представление открывает несколько разных окон:
по ней видно, на какое пришёл ответ.

### Действие с программой

```rust
use tvision::*;

/// Из любого представления: сменить узор рабочего стола.
fn change_pattern(ctx: &mut Ctx) {
    ctx.with_program(|app| app.desktop_mut().set_pattern('▒'));
}
```

## `Request` и `PlaceFn`

Внутреннее устройство `Ctx`, нужно редко. `Request` — просьба к
программе (открыть меню, контекстное меню, модальное окно, передать
фокус, выполнить функцию); её отправляет `ctx.request(r)`, но обычно
вместо этого вызывают готовые методы `exec`, `popup_menu`, `focus`,
`with_program`. `PlaceFn` — функция `(границы якоря, размер владельца) →
границы окна` для `ctx.exec_placed`: так окно истории встаёт прямо под
строкой ввода.

## `CommandSet`

**Что это.** Список запрещённых команд. Хранится в программе; по нему
меню и строка состояния решают, какие пункты рисовать серыми. Методы
`is_enabled(cmd)`, `enable(cmd)`, `disable(cmd)`. Напрямую обычно не
нужен — удобнее `app.enable_command(cmd)` / `app.disable_command(cmd)`.
Запретить можно только команды с номерами до 255.

## `Phase`

**Что это.** На каком этапе доставки клавиша (`ctx.phase`): `PreProcess`
— её видят строка состояния и меню, `Focused` — элемент в фокусе,
`PostProcess` — кнопки «по умолчанию». Нужна редко.

---

## `DrawCtx`

**Что это.** «Холст» для метода `draw`. Координаты на нём **свои**:
`(0, 0)` — левый верхний угол представления. Всё, что выходит за его
границы или закрыто другими окнами, обрезается само.

**Аналог:** методы `writeLine`, `writeStr`, `getColor` у `TView`.

```rust
use tvision::*;

/// Рамка из точек и надпись по центру.
fn draw_box(dc: &mut DrawCtx, text: &str) {
    let a = dc.color(1);
    let size = dc.size;
    dc.fill(Rect::new(0, 0, size.x, size.y), '·', a); // вся область
    let w = text::str_width(text) as i32;
    dc.write_str((size.x - w) / 2, size.y / 2, text, a);
}
```

| Метод | Что делает |
|-------|-----------|
| `color(n)` | цвет номер `n` из палитры представления |
| `fill(r, символ, цвет)` | залить прямоугольник |
| `write_str(x, y, текст, цвет)` | написать текст; возвращает его ширину |
| `write_char(x, y, символ, цвет, сколько)` | повторить символ |
| `write_line(x, y, ширина, высота, &буфер)` | вывести готовый `DrawBuffer` |
| `size` | размер представления |

**Реже:** `get_color(0xHHLL)` (пара цветов для текста с горячей буквой),
`map_color(n)`, `write_cells(x, y, &клетки)`, `recolor(r, цвет)`
(перекрасить уже нарисованное), `shadow(r)` (тень), `owner_palette_len()`
(какой длины палитра владельца: 8 — окно, 32 — диалог; так таблица
подбирает цвета), `commands` (какие команды запрещены), `origin()`,
`clip()`, `new`, `sub` (служебные).

> Представление должно закрашивать **всю** свою область — что не
> нарисовано, останется от прошлого кадра. Последний столбец —
> `size.x - 1`, последняя строка — `size.y - 1`.

---

## `Group`

**Что это.** Представление с детьми. Рабочий стол, окно, диалог — это
группы (или построены вокруг группы). Группа рисует детей, раздаёт им
события, переводит фокус по Tab.

**Аналог:** `TGroup`.

Дети лежат в `children` по порядку **наложения**: первый — самый нижний,
последний — верхний. Выбранный ребёнок — `current`.

**Чаще всего нужно:**

| Метод | Что делает |
|-------|-----------|
| `add(v) -> ViewId` | добавить ребёнка |
| `find::<T>(id)`, `find_mut::<T>(id)` | найти представление типа `T` по id (и среди внуков) |
| `current_view()`, `current_view_mut()` | выбранный ребёнок (у рабочего стола — активное окно) |
| `remove(id)` | убрать ребёнка |
| `get_data()`, `set_data(v)` | данные всех детей разом |

**Реже:** `set_layout(раскладка)`, `apply_layout()`, поле `layout` ([раскладки](33-layout.md#раскладка-в-окне-и-группе)), `Group::new(rect)`, `insert(Box<dyn View>)`, `index_of(id)`,
`child(id)`, `view(id)`, `view_mut(id)` (без приведения типа),
`select_id(id, ctx)`, `select_index(i, ctx)`, `set_current`,
`select_next(вперёд, ctx)`, `reset_current()`, `make_first(i)`,
`put_at(i, v)`, `focus_path(id, ctx)` (выбрать представление вместе с
его окном), `call_on(id, ctx, f)`, `for_each_view(f)` (обойти всех),
`broadcast_children`, `dispatch`, `has_data()`, `geometry_of(id)`,
`owner_origin_of(id)` (для расчёта [экранных
координат](#перевод-в-экранные-координаты)), `cursor_pos`,
`announce_focus`, `help_ctx()`.

**Своя группа.** Обычно своё «составное» представление делают на основе
окна ([урок 8](08-own-window.md)). Если окно не нужно — на основе
группы:

```rust
use tvision::*;

/// Подпись сверху и поле ввода под ней — как один элемент.
pub struct Field {
    pub group: Group,
    pub input: ViewId,
}

impl Field {
    pub fn new(r: Rect, title: &str) -> Self {
        let mut group = Group::new(r);
        group.vs.options |= of::SELECTABLE;
        let w = r.width();
        group.add(StaticText::new(Rect::new(0, 0, w, 1), title)); // координаты — внутри группы
        let input = group.add(InputLine::new(Rect::new(0, 1, w, 2), 80));
        Field { group, input }
    }

    pub fn text(&self) -> String {
        self.group.find::<InputLine>(self.input).map(|il| il.text()).unwrap_or_default()
    }
}

impl View for Field {
    tvision::impl_group_basics!(group);

    fn draw(&self, dc: &mut DrawCtx) {
        self.group.draw_children(dc);
    }
    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        self.group.handle_group_event(ev, ctx);
    }
    fn set_state(&mut self, flag: u16, on: bool) {
        self.group.set_group_state(flag, on);
    }
    fn valid(&mut self, cmd: u16, ctx: &mut Ctx) -> bool {
        self.group.valid(cmd, ctx)
    }
}
```

---

## Макросы

Пишут служебные методы `View`, одинаковые у всех. Вызываются **внутри**
`impl View for …`.

| Макрос | Когда |
|--------|-------|
| `impl_view_basics!()` | у вашей структуры есть поле `vs: ViewState` |
| `impl_group_basics!(путь)` | структура построена вокруг группы: `group`, `win.group` (своё окно), `dlg.win.group` (свой диалог); дополнительно пишет работу с детьми и данными |
| `impl_view_basics_field!(поле)` | `ViewState` лежит во вложенной структуре: `self.поле.vs` |
| `impl_group_core!(путь)` | как `impl_group_basics!`, но данные (`has_data`/`get_data`/`set_data`) вы пишете сами |

## Вспомогательные функции

* `view::default_handle_event(&self.vs, ev, ctx)` — стандартное
  поведение: щелчок по представлению переводит на него фокус. Вызывайте
  первой строкой своего `handle_event`.
* `view::calc_bounds(v, сдвиг, размер_владельца)` — новые границы по
  `grow_mode`, когда владелец меняет размер. Нужна своим группам.
* `view::next_id()` — новый номер представления (его вызывает
  `ViewState::new`).
* `view::HC_NO_CONTEXT` (0), `view::HC_DRAGGING` (1) — стандартные
  номера тем справки.

---

## Координаты подробно

Основы — в [уроке 4](04-coordinates.md). Здесь — то, что нужно реже.

### Три системы координат

У каждой точки три «адреса»:

1. **экранные** — от левого верхнего угла терминала;
2. **владельца** — от угла группы, в которой лежит представление
   (`vs.origin` и `Rect` в конструкторе — в них);
3. **свои** — от левого верхнего угла самого представления.

| Что | В каких координатах |
|-----|---------------------|
| `Rect` в конструкторе, `vs.origin`, `vs.bounds()` | владельца |
| рисование (`DrawCtx`), `vs.extent()`, `vs.cursor` | свои |
| `m.pos` мыши в `View::handle_event` | владельца → свои через `make_local` |
| `m.pos` мыши в `AppHandler::handle_event` | экранные |
| окно для `app.insert_window` | рабочего стола |
| элемент в `Dialog::add` / `Window::add` | окна (рамка — строка и столбец 0) |
| `ctx.popup_menu(…, pos)` | экранные |
| `Editor::cursor()`, `Table::cursor()` | **не экранные**: строка и номер символа в тексте, строка и столбец данных |

### Перевод в экранные координаты

```rust
use tvision::platform::HeadlessBackend;
use tvision::*;

/// Границы представления на экране.
fn screen_rect(app: &Program, id: ViewId) -> Option<Rect> {
    let (bounds, _) = app.group.geometry_of(id)?; // у владельца
    let o = app.group.owner_origin_of(id)?; // где на экране угол владельца
    Some(bounds.moved(o.x, o.y))
}

fn layers() {
    let menu = Menu::new().submenu("~Ф~айл", Menu::new().item("~В~ыход", cm::QUIT));
    let mut app = Program::new(Box::new(HeadlessBackend::new(80, 25)), Some(menu), None);
    let mut w = Window::new(Rect::new(4, 2, 40, 12), "Окно", 1); // на рабочем столе
    let button = w.add(Button::new(Rect::new(7, 1, 19, 3), "O~K~", cm::OK, bf::DEFAULT)); // в окне
    let win = app.insert_window(w);
    assert_eq!(screen_rect(&app, win).unwrap().a, Point::new(4, 3)); // стол — со строки 1, под меню
    assert_eq!(screen_rect(&app, button).unwrap().a, Point::new(11, 4)); // 4 + 7, 3 + 1
}
```

### Рабочий стол и экран

Окна лежат на рабочем столе, а он — между строкой меню (строка 0) и
строкой состояния (последняя строка). На экране 80×25 с меню и строкой
состояния стол — 80×23. Размер «во весь стол» берите у стола:
`app.desktop().group.vs.extent()`. Размер экрана — `app.ctx.screen_size`.

### Центрирование

`Dialog::centered(w, h, …)` и флаг `of::CENTERED` (или `CENTER_X`,
`CENTER_Y`) ставят представление по центру владельца **в момент
вставки**. Чтобы диалог оставался по центру и после изменения размера
терминала, добавьте `grow_mode = gf::GROW_REL`.

```rust
use tvision::platform::HeadlessBackend;
use tvision::*;

fn centering() {
    let status = vec![StatusDef::all(vec![StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT)])];
    let menu = Menu::new().submenu("~Ф~айл", Menu::new().item("~В~ыход", cm::QUIT));
    let mut app = Program::new(Box::new(HeadlessBackend::new(80, 25)), Some(menu), Some(status));
    assert_eq!(app.desktop().group.vs.size, Point::new(80, 23));
    let id = app.insert_window(Dialog::centered(40, 9, "По центру"));
    let o = app.find::<Dialog>(id).unwrap().win.group.vs.origin;
    assert_eq!(o, Point::new((80 - 40) / 2, (23 - 9) / 2));
}
```

### Растягивание

Когда владелец меняет размер на `(dx, dy)`, края ребёнка сдвигаются по
флагам `grow_mode`:

| Чего хотим | `grow_mode` |
|------------|-------------|
| растягиваться вместе с окном | `gf::GROW_HI_X \| gf::GROW_HI_Y` |
| оставаться на месте | `0` |
| прижаться к низу и тянуться по ширине (строка сведений) | `gf::GROW_LO_Y \| gf::GROW_HI_Y \| gf::GROW_HI_X` |
| прижаться к правому нижнему углу (кнопка) | `gf::GROW_ALL` |

```rust
use tvision::*;

fn grow() {
    // Окно выросло на (+6, +2): с 30×10 до 36×12.
    let (owner, delta) = (Point::new(36, 12), Point::new(6, 2));
    let mut ok = Button::new(Rect::new(17, 7, 29, 9), "O~K~", cm::OK, bf::DEFAULT);
    ok.vs.grow_mode = gf::GROW_ALL;
    assert_eq!(view::calc_bounds(&ok, delta, owner), Rect::new(23, 9, 35, 11)); // уехала в угол
}
```

Что растягивается само: `Editor`, `TreeView`, `Outline`, `DualPanel` и
полосы прокрутки. **Не** растягиваются сами: `Table`, `ListBox`,
`FilePanel` и все поля и кнопки — им `grow_mode` задают вручную.

### Прокрутка

У прокручиваемых представлений строка на экране и строка данных —
разные числа: **строка данных = первая видимая + строка на экране**.
Таблица и дерево отдают строку данных (`cursor()`), а не экранную.

### Широкие символы

Иероглифы и многие эмодзи занимают **две** клетки. Поэтому ширину
текста считают `text::str_width`, а не `chars().count()`. У редактора
позиция — номер символа; экранный столбец дают `col_to_x` / `x_to_col`.

### Частые ошибки

| Симптом | Причина | Как исправить |
|---------|---------|---------------|
| элемент на клетку шире | ширина как `x2 - x1 + 1` | `Rect::new(x, y, x + ширина, y + высота)` |
| элемент залез на рамку | начали с 0 или кончили на ширине окна | от 1 до `ширина − 2`, или `interior()` |
| у кнопки нет тени | высота 1 | у кнопки высота 2 |
| щелчок «промахивается» | сравнили `m.pos` со своим прямоугольником | сначала `make_local(m.pos)` |
| контекстное меню не там | передали свои координаты | нужны экранные |
| окно залезает под строку состояния | размер взяли у экрана | у рабочего стола |
| элемент не растёт вместе с окном | нет `grow_mode` | `gf::GROW_HI_X \| gf::GROW_HI_Y` |
