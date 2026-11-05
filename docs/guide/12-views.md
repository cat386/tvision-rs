# Справочник: представления

Представление (view) — всё, что видно на экране. Здесь — трейт `View` и
то, что ему нужно: общие поля, флаги, данные, контекст событий, контекст
отрисовки и группа. Модули `view` и `group`.

* [`View`](#view) — трейт представления
* [`ViewState`](#viewstate) · [`ViewId`](#viewid) · [флаги `sf`, `of`, `gf`, `dm`](#флаги)
* [`Value`](#value) — данные для диалогов
* [`Ctx`](#ctx) · [`Request`, `PlaceFn`](#request-и-placefn) · [`CommandSet`](#commandset) · [`Phase`](#phase)
* [`DrawCtx`](#drawctx) — рисование
* [`Group`](#group) — представление с детьми
* [Макросы `impl_*`](#макросы) · [`default_handle_event`, `calc_bounds`](#вспомогательные-функции)

---

## `View`

**Трейт представления** — аналог класса `TView`. Любой видимый объект
реализует его: кнопка, окно, рабочий стол и ваши собственные элементы.

Обязательны два метода — `draw` и пять «технических» (`vs`, `vs_mut`,
`as_any`, `as_any_mut`, `into_any`), которые пишет макрос
[`impl_view_basics!`](#макросы). Остальные имеют разумное поведение по
умолчанию.

### Пример: своё представление «Оценка»

Звёзды `★★★☆☆`: ←/→ и щелчок меняют оценку, значение участвует в обмене
данными диалога.

```rust
use tvision::*;

pub struct Rating {
    pub vs: ViewState,
    pub value: u32, // 0..=5
}

impl Rating {
    pub fn new(x: i32, y: i32) -> Self {
        let mut vs = ViewState::new(Rect::new(x, y, x + 5, y + 1));
        vs.options |= of::SELECTABLE | of::FIRST_CLICK; // получает фокус; первый щелчок не теряется
        Rating { vs, value: 3 }
    }
}

impl View for Rating {
    tvision::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        // Номера цветов — в своей палитре (здесь — как у строки ввода):
        // 1 — обычный, 2 — в фокусе.
        let a = dc.color(if self.vs.is_focused() { 2 } else { 1 });
        let s: String = (0..5).map(|i| if i < self.value { '★' } else { '☆' }).collect();
        dc.write_str(0, 0, &s, a); // представление закрашивает всю свою область
    }

    fn palette(&self) -> &'static [u8] {
        color::palettes::INPUT_LINE
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        view::default_handle_event(&self.vs, ev, ctx); // щелчок по невыбранному — выбрать
        match ev {
            Event::MouseDown(m) => {
                let p = self.vs.make_local(m.pos); // координаты мыши — владельца
                self.value = (p.x + 1).clamp(0, 5) as u32;
                ev.clear();
            }
            Event::KeyDown(k) if self.vs.is_focused() => {
                match k.key {
                    Key::Left => self.value = self.value.saturating_sub(1),
                    Key::Right => self.value = (self.value + 1).min(5),
                    _ => return, // не наше — пусть обработает кто-то другой
                }
                ev.clear();
            }
            _ => {}
        }
    }

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

fn rating_dialog() -> Dialog {
    let mut d = Dialog::centered(30, 8, "Оценка");
    let r = d.add(Rating::new(12, 2));
    d.add(Label::new(Rect::new(2, 2, 11, 3), "~О~ценка:", Some(r)));
    d.add(Button::new(Rect::new(9, 4, 21, 6), "O~K~", cm::OK, bf::DEFAULT));
    d
}

fn test_rating() {
    let mut r = Rating::new(0, 0);
    r.set_data(&Value::U32(9));
    assert_eq!(r.get_data(), Value::U32(5));
    let _ = rating_dialog();
}
```

### Методы трейта

| Метод | Когда вызывается | По умолчанию |
|-------|------------------|--------------|
| `draw(&self, dc)` | при каждой перерисовке; **обязателен** | — |
| `handle_event(&mut self, ev, ctx)` | пришло событие | выбор щелчком мыши |
| `palette(&self)` | при рисовании: палитра для `dc.color(n)` | пустая — номера идут владельцу как есть |
| `set_state(&mut self, flag, on)` | меняется флаг `sf::*` (фокус, активность…) | просто меняет `vs.state` |
| `valid(&mut self, cmd, ctx) -> bool` | перед закрытием окна или выходом (`cmd` — команда) | `true` |
| `exec_result(&mut self, tag, cmd, view, ctx)` | закрылось модальное окно, запущенное через `ctx.exec` | ничего |
| `update_commands(&self, ctx)` | после каждого события — разрешить/запретить команды | ничего |
| `monitor_event(&mut self, ev)` | все события программы (если представление — `Program::event_monitor`) | ничего |
| `change_bounds(&mut self, r)` | меняются границы (окно растянули) | `vs.set_bounds(r)` |
| `size_limits(&self, owner)` | при изменении размера: (мин, макс) | (0, размер владельца) |
| `has_data`, `get_data`, `set_data` | обмен данными диалога | данных нет |
| `help_ctx(&self)` | строке состояния и справке нужен контекст | `vs.help_ctx` |
| `as_window`, `as_group`, `as_group_mut` | программе нужно окно / дети | `None` |
| `as_any`, `as_any_mut`, `into_any` | приведение типа (`downcast`) | пишет макрос |
| `contains_id(id)` | есть ли `id` в поддереве | только сам |

### Приведение типа

Представления хранятся как `Box<dyn View>`. Конкретный тип возвращает
`as_any().downcast_ref::<T>()`; удобнее `group.find::<T>(id)`,
`app.find_mut::<T>(id)`, а у модального окна — `app.exec_dialog(d)`,
который отдаёт диалог обратно его собственным типом.

```rust
use tvision::*;

fn text_of(v: &dyn View) -> Option<String> {
    v.as_any().downcast_ref::<InputLine>().map(|il| il.text())
}
```

---

## `ViewState`

**Общие поля** любого представления (поля `TView`). Хранится в поле `vs`.

| Поле | Что |
|------|-----|
| `id: ViewId` | уникальный номер (выдаётся в `ViewState::new`) |
| `origin: Point` | левый верхний угол **в координатах владельца** |
| `size: Point` | ширина и высота |
| `cursor: Point` | позиция курсора внутри представления |
| `state: u16` | флаги состояния `sf::*` |
| `options: u16` | флаги параметров `of::*` |
| `grow_mode: u8` | как меняться при изменении владельца `gf::*` |
| `drag_mode: u8` | как перетаскивать `dm::*` |
| `help_ctx: u16` | контекст справки (для строки состояния и F1) |

| Метод | Что |
|-------|-----|
| `ViewState::new(bounds)` | новые поля, видимое (`sf::VISIBLE`) |
| `bounds()`, `extent()` | границы у владельца / свои, от (0, 0) |
| `set_bounds(r)` | задать границы |
| `make_local(p)` | точку владельца — в свои координаты (для мыши) |
| `get_state(flag)` | установлен ли флаг |
| `is_visible()`, `is_selected()`, `is_focused()`, `is_active()`, `is_disabled()`, `is_selectable()` | частые проверки |
| `show_cursor()`, `hide_cursor()`, `set_cursor(x, y)`, `block_cursor()`, `normal_cursor()` | текстовый курсор |

## `ViewId`

`u32` — **номер представления**. В C++ представления знали друг друга по
указателям, здесь — по номерам: `d.add(v)` возвращает id, а
`group.find::<T>(id)` находит представление. Новый номер выдаёт
`view::next_id()` (его вызывает `ViewState::new`).

## Флаги

### `sf` — состояние (`vs.state`)

| Флаг | Значит |
|------|--------|
| `VISIBLE` | видно |
| `SELECTED` | текущее у своего владельца |
| `FOCUSED` | выбрано на всей цепочке до корня — получает клавиши |
| `ACTIVE` | лежит в активном окне |
| `DISABLED` | недоступно |
| `MODAL` | выполняется модально |
| `CURSOR_VIS`, `CURSOR_INS` | курсор виден; блочный (режим вставки) |
| `SHADOW` | рисовать тень (окна) |
| `DRAGGING` | окно перетаскивают |
| `DEFAULT` | кнопка сейчас «по умолчанию» |
| `EXPOSED` | видно на экране |

Флаги состояния меняет программа (через `set_state`); сами вы обычно
только читаете их: `self.vs.is_focused()`.

### `of` — параметры (`vs.options`)

| Флаг | Значит |
|------|--------|
| `SELECTABLE` | может получить фокус |
| `FIRST_CLICK` | щелчок по невыбранному не только выбирает, но и обрабатывается |
| `TOP_SELECT` | при выборе поднимается наверх (окна) |
| `PRE_PROCESS`, `POST_PROCESS` | получать клавиши до / после элемента в фокусе (строка состояния, кнопки) |
| `TILEABLE` | участвует в «Мозаике» и «Каскаде» |
| `CENTER_X`, `CENTER_Y`, `CENTERED` | центрировать при вставке |
| `VALIDATE` | вызывать `valid(cm::RELEASED_FOCUS)` при уходе фокуса |
| `FRAMED`, `BUFFERED` | оставлены для совместимости |

### `gf` — изменение размера (`vs.grow_mode`)

Что происходит с представлением, когда владелец меняет размер на `delta`:
`GROW_LO_X` / `GROW_LO_Y` двигают левый / верхний край, `GROW_HI_X` /
`GROW_HI_Y` — правый / нижний. `GROW_ALL` — все четыре (представление
едет вместе с правым нижним углом), `GROW_HI_X | GROW_HI_Y` —
растягивается. `GROW_REL` — пропорционально (окна на рабочем столе),
`FIXED` — без ограничения размером владельца.

```rust
use tvision::*;

/// Кнопка в правом нижнем углу окна, которая остаётся там при растягивании.
fn corner_button(w: &mut Window) {
    let s = w.group.vs.size;
    let mut b = Button::new(Rect::new(s.x - 14, s.y - 3, s.x - 2, s.y - 1), "O~K~", cm::OK, bf::DEFAULT);
    b.vs.grow_mode = gf::GROW_ALL;
    w.add(b);
}
```

### `dm` — перетаскивание (`vs.drag_mode`)

`DRAG_MOVE`, `DRAG_GROW`, `DRAG_GROW_LEFT` и ограничения `LIMIT_LO_X`,
`LIMIT_LO_Y`, `LIMIT_HI_X`, `LIMIT_HI_Y`, `LIMIT_ALL` (не выходить за
края владельца). Нужны окнам; что разрешено пользователю, задают флаги
окна `wf::*`.

---

## `Value`

**Данные элемента диалога** для `get_data`/`set_data` — замена записи
`TDialogData`, которую в C++ копировали побайтно.

| Вариант | У кого |
|---------|--------|
| `Str(String)` | `InputLine`, `Memo`, `Editor` |
| `U32(u32)` | флажки (маска), переключатели (номер) |
| `Int(i64)` | `ListBox` (номер строки), `InputLong`, `ProgressBar` |
| `List(Vec<Value>)` | группа: значения детей с данными в порядке вставки |
| `None` | нет данных |

Методы: `as_str()`, `as_u32()`, `as_int()`, `as_list()` — `Option` с
содержимым, если вариант подходит.

```rust
use tvision::*;

fn values() {
    let v = Value::List(vec![Value::Str("Аня".into()), Value::U32(0b101)]);
    let l = v.as_list().unwrap();
    assert_eq!(l[0].as_str(), Some("Аня"));
    assert_eq!(l[1].as_u32(), Some(5));
    assert_eq!(l[1].as_int(), None);
}
```

---

## `Ctx`

**Контекст событий** — то, через что представление действует «снаружи
себя». Передаётся в `handle_event`, `valid`, `exec_result`,
`update_commands`. Аналог доступа к `owner`, `TProgram::application` и
`TEventQueue` из C++.

Запросы выполняются **сразу после** того, как представление вернётся из
`handle_event`: владелец (или программа) забирает их и исполняет.

| Метод | Что делает | В C++ |
|-------|-----------|-------|
| `post_command(cmd, info)`, `put_event(ev)` | поставить команду / событие в очередь | `putEvent` |
| `message(cmd, info)` | разослать сообщение соседям (детям своего владельца) | `message(owner, evBroadcast, …)` |
| `select(id)` | сделать представление текущим у владельца | `view->select()` |
| `select_next(forward)` | перейти к следующему / предыдущему | `owner->selectNext` |
| `focus(id)` | передать фокус представлению где угодно (с окном и всеми владельцами) | — |
| `make_first(id)` | поднять наверх | `makeFirst` |
| `close(id)` | удалить представление из владельца | `destroy`, `close` |
| `end_modal(cmd)` | завершить модальное окно с кодом | `endModal` |
| `exec(view, reply_to, tag)` | открыть модальное окно; результат — в `exec_result` представления `reply_to` | `owner->execView` |
| `exec_near(…, anchor)`, `exec_placed(…, anchor, place)` | то же, с границами относительно другого представления | — |
| `popup_menu(menu, pos)` | контекстное меню | `TMenuPopup` |
| `with_program(f)` | выполнить `f(&mut Program)` после события | обращение к `application` |
| `grab_mouse(id)`, `release_mouse()`, `mouse_grabbed_by(id)` | захват мыши (протяжка) | `mouseEvent` в цикле |
| `command_enabled(cmd)`, `enable_command`, `disable_command`, `set_command(cmd, on)` | набор команд | `commandEnabled`… |
| `owner_size()` | размер владельца | `owner->size` |
| `screen_size`, `clipboard`, `phase` | поля: размер экрана, буфер обмена, фаза доставки | — |

### Модальное окно изнутри представления

Модальный цикл умеет запускать только программа, поэтому представление
**просит** её и получает ответ позже:

```rust
use tvision::*;

const TAG_NAME: u16 = 1;

/// Кнопка-подпись: щелчок спрашивает имя и показывает его.
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
            ctx.exec(Box::new(d), Some(self.vs.id), TAG_NAME); // откроется после этого события
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

Так работают сообщения валидаторов, окно истории, вопрос «Сохранить
изменения?» редактора. Из кода приложения (у которого есть `&mut Program`)
проще вызвать `app.exec_dialog(d)` напрямую.

### Действие с программой

```rust
use tvision::*;

/// Из любого представления: сменить узор рабочего стола.
fn change_pattern(ctx: &mut Ctx) {
    ctx.with_program(|app| app.desktop_mut().set_pattern('▒'));
}
```

## `Request` и `PlaceFn`

`Request` — запрос к программе, который `Ctx` отправляет методом
`ctx.request(r)`: открыть меню (`MenuBar`), контекстное меню (`Popup`),
модальное окно (`Exec`), передать фокус (`Focus`), выполнить функцию
(`WithProgram`). Обычно вместо него вызывают готовые методы `Ctx`
(`exec`, `popup_menu`, `focus`, `with_program`).

`PlaceFn` — `Box<dyn Fn(Rect, Point) -> Rect>`: по границам якоря и
размеру его владельца вычисляет границы модального окна (так окно истории
встаёт под строкой ввода). Передаётся в `ctx.exec_placed`.

## `CommandSet`

**Набор запрещённых команд** (`TCommandSet`). Хранится в программе;
читают его меню, строка состояния и кнопки (`DrawCtx::commands`).
Методы: `is_enabled(cmd)`, `enable(cmd)`, `disable(cmd)` (возвращают,
изменилось ли что-то). Команды больше 255 разрешены всегда. В своём коде
удобнее `app.enable_command` / `ctx.disable_command`.

## `Phase`

Фаза доставки фокусного события (`ctx.phase`): `PreProcess` (получают
представления с `of::PRE_PROCESS`), `Focused` (элемент в фокусе),
`PostProcess` (с `of::POST_PROCESS`). Нужна редко — например, кнопке,
чтобы отличить «Enter для меня» от «Enter для кнопки по умолчанию».

---

## `DrawCtx`

**Контекст рисования** одного представления: окно в экранный буфер с
обрезкой по видимой области и цепочкой палитр. Координаты — **свои**, от
(0, 0). Аналог методов `writeLine`, `writeStr`, `getColor` у `TView`.

```rust
use tvision::*;

/// Рамка из точек и надпись по центру.
fn draw_box(dc: &mut DrawCtx, size: Point, text: &str) {
    let a = dc.color(1);
    dc.fill(Rect::new(0, 0, size.x, size.y), '·', a);
    let w = text::str_width(text) as i32;
    dc.write_str((size.x - w) / 2, size.y / 2, text, a);
    let mut b = DrawBuffer::new(size.x as usize);
    b.move_char(0, '─', a, size.x as usize);
    dc.write_line(0, 0, size.x, 1, &b); // верхняя строка
}
```

| Метод / поле | Что |
|--------------|-----|
| `color(n)` | атрибут по номеру цвета своей палитры (через цепочку до палитры приложения) |
| `get_color(0xHHLL)` | пара цветов (`AttrPair`): младший байт — обычный, старший — подсветка |
| `map_color(n)` | то же, что `color` |
| `owner_palette_len()` | длина палитры владельца: 8 — окно, 32 — диалог, 135 — рабочий стол (так `Table` подбирает цвета) |
| `write_line(x, y, w, h, &buf)` | вывести буфер в `h` строк подряд |
| `write_str(x, y, s, attr) -> ширина`, `write_char(x, y, ch, attr, n)`, `write_cells(x, y, &cells)` | строка, символ, ячейки |
| `fill(r, ch, attr)` | залить прямоугольник |
| `recolor(r, attr)` | перекрасить уже нарисованное |
| `shadow(r)` | тень справа и снизу от `r` |
| `size`, `commands` | размер представления; набор команд (серые пункты) |
| `origin()`, `clip()` | положение на экране и видимая часть (для отладки) |
| `DrawCtx::new`, `sub` | создать корневой / дочерний контекст — это делает программа и группы |

---

## `Group`

**Представление с детьми** (`TGroup`). Рабочий стол, окно, диалог — это
группы или построены вокруг них. Дети лежат в `children` в порядке Z:
первый — нижний, последний — верхний; текущий — `current`.

```rust
use tvision::*;

/// Панель из двух строк: подпись и поле ввода как одно целое.
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
    tvision::impl_group_basics!(group); // данные, справка, размеры, дети — у группы

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

| Метод | Что делает |
|-------|-----------|
| `Group::new(bounds)` | пустая группа |
| `add(v) -> ViewId`, `insert(Box<dyn View>)` | добавить ребёнка наверх |
| `remove(id) -> Option<Box<dyn View>>` | вынуть ребёнка |
| `index_of(id)`, `child(id)` | среди прямых детей |
| `view(id)`, `view_mut(id)` | в **поддереве** (с внуками) |
| `find::<T>(id)`, `find_mut::<T>(id)` | то же с приведением типа |
| `current_view()`, `current_view_mut()` | текущий ребёнок (активное окно рабочего стола) |
| `select_id(id, ctx)`, `select_index(i, ctx)`, `set_current(i, ctx)` | выбрать ребёнка |
| `select_next(forward, ctx)` | Tab / Shift+Tab |
| `reset_current()` | текущим — верхний видимый выбираемый |
| `make_first(i)`, `put_at(i, new)` | поменять место в Z-порядке |
| `focus_path(id, ctx)` | выбрать `id` и всех его владельцев |
| `call_on(id, ctx, f)` | выполнить `f` над представлением так, будто оно обрабатывает событие |
| `for_each_view(f)` | обойти поддерево |
| `broadcast_children(ev, ctx)`, `dispatch(i, ev, ctx)` | разослать / доставить событие |
| `get_data()`, `set_data(v)`, `has_data()` | данные всех детей (`Value::List`) |
| `draw_children(dc)`, `handle_group_event(ev, ctx)`, `set_group_state(flag, on)`, `change_bounds(r)`, `valid(cmd, ctx)` | «родительские» реализации методов `View` для своих групп |
| `geometry_of(id)`, `owner_origin_of(id)`, `cursor_pos(…)`, `announce_focus(ctx)`, `help_ctx()` | служебные |

---

## Макросы

| Макрос | Для чего | Что пишет |
|--------|----------|-----------|
| `impl_view_basics!()` | представление с полем `vs: ViewState` | `vs`, `vs_mut`, `as_any`, `as_any_mut`, `into_any` |
| `impl_view_basics_field!(f)` | обёртка: `ViewState` лежит в `self.f.vs` (как `Memo` вокруг `Editor`) | то же |
| `impl_group_basics!(путь)` | построено вокруг группы: `group`, `win.group`, `dlg.win.group` | всё выше + `as_group`, `as_group_mut`, данные (`has_data`/`get_data`/`set_data`), `help_ctx`, `change_bounds` |
| `impl_group_core!(путь)` | то же, но данные свои (`has_data`/`get_data`/`set_data` пишете сами) | |

Макросы вызываются внутри `impl View for …` (они разворачиваются в методы)
и экспортированы из корня крейта: `tvision::impl_view_basics!()`.

## Вспомогательные функции

* `view::default_handle_event(&vs, ev, ctx)` — поведение `TView::handleEvent`:
  щелчок по невыбранному выбираемому представлению выбирает его (а без
  `of::FIRST_CLICK` ещё и «съедает» щелчок). Вызывайте первой строкой в
  своём `handle_event`.
* `view::calc_bounds(v, delta, owner_size)` — новые границы по `grow_mode`
  при изменении владельца на `delta` (`calcBounds`); нужна своим группам.
* `view::HC_NO_CONTEXT` (0), `view::HC_DRAGGING` (1) — стандартные
  контексты справки.
