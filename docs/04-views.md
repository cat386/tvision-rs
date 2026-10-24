# 4. Свои представления

Любой элемент интерфейса — тип, реализующий трейт `View`. Стандартные
элементы (кнопки, списки, окна) устроены так же, как ваши собственные,
поэтому всё, что описано здесь, применимо и к чтению исходников
библиотеки.

## Минимальное представление

Представлению нужны:

1. поле `vs: ViewState` — общие данные любого представления (положение,
   размер, флаги, курсор, id);
2. реализация `View` с методами `vs`, `vs_mut`, `as_any`, `as_any_mut`,
   `into_any` (их пишет макрос `impl_view_basics!`) и `draw`.

```rust
use tvision::*;

/// Счётчик: показывает число, «+»/«-» меняют его, щелчок увеличивает.
pub struct Counter {
    pub vs: ViewState,
    pub value: i64,
}

impl Counter {
    pub fn new(bounds: Rect) -> Self {
        let mut vs = ViewState::new(bounds);
        // Может получать фокус; щелчок по невыбранному сразу обрабатывается.
        vs.options |= of::SELECTABLE | of::FIRST_CLICK;
        Counter { vs, value: 0 }
    }
}

impl View for Counter {
    tvision::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        // Цвет: 1 — обычный, 2 — в фокусе (номера в палитре INPUT_LINE).
        let c = if self.vs.is_focused() { dc.color(2) } else { dc.color(1) };
        let w = self.vs.size.x.max(0) as usize;
        let mut b = DrawBuffer::new(w);
        b.move_char(0, ' ', c, w); // закрасить всю ширину
        b.move_str(1, &format!("{:>6}", self.value), c);
        dc.write_line(0, 0, self.vs.size.x, self.vs.size.y, &b);
    }

    fn palette(&self) -> &'static [u8] {
        color::palettes::INPUT_LINE // цвета как у строки ввода
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        // Стандартное поведение: щелчок по невыбранному — выбрать.
        view::default_handle_event(&self.vs, ev, ctx);
        match ev {
            Event::MouseDown(_) => {
                self.value += 1;
                ev.clear();
            }
            Event::KeyDown(k) if self.vs.is_focused() => match k.text_char() {
                Some('+') => {
                    self.value += 1;
                    ev.clear();
                }
                Some('-') => {
                    self.value -= 1;
                    ev.clear();
                }
                _ => {}
            },
            _ => {}
        }
    }

    // Участие в обмене данными диалога (getData/setData).
    fn has_data(&self) -> bool {
        true
    }
    fn get_data(&self) -> Value {
        Value::Int(self.value)
    }
    fn set_data(&mut self, v: &Value) {
        if let Value::Int(i) = v {
            self.value = *i;
        }
    }
}
```

Такой счётчик можно вставить в любое окно или диалог: `d.add(Counter::new(r))`.

## `ViewState` — общие поля

| Поле | Смысл | Аналог |
|------|-------|--------|
| `id: ViewId` | уникальный номер, выдаётся в `ViewState::new` | указатель `this` |
| `origin: Point` | левый верхний угол в координатах владельца | `origin` |
| `size: Point` | размер | `size` |
| `cursor: Point` | позиция курсора (локальная) | `cursor` |
| `state: u16` | флаги состояния `sf::*` | `state` |
| `options: u16` | флаги параметров `of::*` | `options` |
| `grow_mode: u8` | изменение размера вместе с владельцем `gf::*` | `growMode` |
| `drag_mode: u8` | ограничения перетаскивания `dm::*` | `dragMode` |
| `help_ctx: u16` | номер контекста справки | `helpCtx` |

Методы: `bounds()`, `extent()`, `set_bounds(r)`, `make_local(p)`,
`get_state(flag)`, `is_visible()`, `is_disabled()`, `is_selected()`,
`is_focused()`, `is_active()`, `is_selectable()`, `show_cursor()`,
`hide_cursor()`, `set_cursor(x, y)`, `block_cursor()`, `normal_cursor()`.

### Флаги состояния `sf`

| Флаг | Когда установлен |
|------|------------------|
| `VISIBLE` | представление видно (снимите, чтобы спрятать) |
| `CURSOR_VIS` | показывать курсор, когда представление в фокусе |
| `CURSOR_INS` | курсор блочный (режим замены) |
| `SHADOW` | у представления есть тень (окна) |
| `ACTIVE` | представление в активном окне |
| `SELECTED` | текущий ребёнок своего владельца |
| `FOCUSED` | выбрано на всей цепочке до корня |
| `DRAGGING` | окно перетаскивают |
| `DISABLED` | недоступно: не получает клавиши и мышь |
| `MODAL` | выполняется модально |
| `DEFAULT` | кнопка по умолчанию |
| `EXPOSED` | видно на экране |

Флаги `SELECTED`, `FOCUSED`, `ACTIVE` ставит владелец через
`View::set_state(flag, enable)`. Переопределите `set_state`, если нужно
реагировать на получение фокуса (строка ввода, например, выделяет весь
текст), но не забудьте изменить сам флаг:

```rust
use tvision::*;

struct Highlighter {
    vs: ViewState,
    entered: u32,
}

impl View for Highlighter {
    tvision::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        dc.fill(self.vs.extent(), ' ', dc.color(1));
    }

    fn set_state(&mut self, flag: u16, enable: bool) {
        if enable {
            self.vs.state |= flag;
        } else {
            self.vs.state &= !flag;
        }
        if flag & sf::FOCUSED != 0 && enable {
            self.entered += 1; // получили фокус
        }
    }
}
```

### Флаги параметров `of`

| Флаг | Смысл |
|------|-------|
| `SELECTABLE` | может получать фокус (Tab, щелчок) |
| `TOP_SELECT` | при выборе поднимается наверх (окна) |
| `FIRST_CLICK` | щелчок по невыбранному не только выбирает, но и обрабатывается |
| `FRAMED` | есть рамка |
| `PRE_PROCESS` | получает фокусные события **до** текущего ребёнка владельца |
| `POST_PROCESS` | получает фокусные события **после** текущего ребёнка |
| `TILEABLE` | окно участвует в «Мозаике» и «Каскаде» |
| `CENTER_X`, `CENTER_Y`, `CENTERED` | центрировать при вставке |
| `VALIDATE` | проверять `valid()` при потере фокуса |
| `BUFFERED` | оставлен для совместимости (всё и так рисуется в буфер) |

## Методы трейта `View`

Обязательные — `vs`, `vs_mut`, `draw`, `as_any`, `as_any_mut`, `into_any`.
Остальные имеют разумную реализацию по умолчанию.

| Метод | Когда вызывается | По умолчанию |
|-------|------------------|--------------|
| `draw(&self, dc)` | на каждом шаге цикла | — |
| `handle_event(&mut self, ev, ctx)` | доставка события | выбор щелчком (`default_handle_event`) |
| `palette(&self)` | при отрисовке: перевод номеров цветов | пустая палитра — номера уходят владельцу как есть |
| `set_state(&mut self, flag, enable)` | изменение `SELECTED`/`FOCUSED`/`ACTIVE`… | меняет флаг |
| `valid(&mut self, cmd, ctx)` | перед закрытием окна или диалога, при `cm::QUIT`, при потере фокуса с `of::VALIDATE` | `true` |
| `exec_result(&mut self, tag, cmd, view, ctx)` | закрылось модальное окно, запущенное через `ctx.exec` | ничего |
| `monitor_event(&mut self, ev)` | для `Program::event_monitor` | ничего |
| `update_commands(&self, ctx)` | после каждого события | ничего |
| `change_bounds(&mut self, r)` | изменили границы (растягивание, `zoom`) | `vs.set_bounds(r)` |
| `size_limits(&self, owner)` | минимальный и максимальный размер | (0,0) и размер владельца |
| `has_data`, `get_data`, `set_data` | обмен данными диалога | нет данных |
| `help_ctx(&self)` | строка состояния и справка | `vs.help_ctx` (или `HC_DRAGGING`) |
| `as_window(&self)` | программе нужны флаги активного окна | `None` |
| `as_group`, `as_group_mut` | обход дерева | `None` |
| `contains_id(&self, id)` | поиск в поддереве | сравнение с `vs.id` и детьми |

### Макросы для обязательных методов

| Макрос | Когда |
|--------|-------|
| `tvision::impl_view_basics!()` | у типа есть поле `vs: ViewState` |
| `tvision::impl_view_basics_field!(поле)` | `ViewState` лежит во вложенном представлении `self.поле.vs` — «наследование» (так `Memo` построен на `Editor`, `InputLong` — на `InputLine`) |
| `tvision::impl_group_basics!(путь)` | тип построен вокруг группы `self.путь` (`win.group`, `dlg.win.group`): методы данных, `as_group`, `change_bounds`, `help_ctx` делегируются группе |
| `tvision::impl_group_core!(путь)` | то же, но без методов данных — когда данные у типа свои |

### «Наследование» через вложение

В C++ новый элемент получали наследованием (`class TMyInput : public
TInputLine`). В Rust встраивают готовое представление и делегируют ему:

```rust
use tvision::*;

/// Строка ввода, которая принимает только заглавные буквы.
struct UpperInput {
    il: InputLine,
}

impl View for UpperInput {
    tvision::impl_view_basics_field!(il);

    fn draw(&self, dc: &mut DrawCtx) {
        self.il.draw(dc);
    }
    fn palette(&self) -> &'static [u8] {
        self.il.palette()
    }
    fn set_state(&mut self, flag: u16, enable: bool) {
        self.il.set_state(flag, enable);
    }
    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        // Подменяем символ до того, как его увидит строка ввода.
        if let Event::KeyDown(k) = ev {
            if let Some(c) = k.text_char() {
                let up = text::to_upper(c);
                *ev = Event::KeyDown(KeyEvent::new(Key::Char(up), k.mods));
            }
        }
        self.il.handle_event(ev, ctx);
    }
    fn valid(&mut self, cmd: u16, ctx: &mut Ctx) -> bool {
        self.il.valid(cmd, ctx)
    }
    fn has_data(&self) -> bool {
        true
    }
    fn get_data(&self) -> Value {
        self.il.get_data()
    }
    fn set_data(&mut self, v: &Value) {
        self.il.set_data(v)
    }
}
```

Не забывайте делегировать **все** методы, поведение которых нужно
сохранить (`set_state`, `valid`, данные, `exec_result`…), — макрос пишет
только обязательные.

## Рисование

### `DrawBuffer` — строка ячеек

Обычный приём (как в оригинале): собрать строку в `DrawBuffer` и вывести
её `write_line`.

| Метод | Аналог | Что делает |
|-------|--------|-----------|
| `DrawBuffer::new(w)` | `TDrawBuffer` | буфер из `w` пробелов |
| `move_char(x, ch, attr, n)` | `moveChar` | `n` раз символ; `ch == '\0'` — менять только цвет |
| `move_str(x, s, attr)` | `moveStr` | строка; возвращает ширину |
| `move_str_limited(x, s, attr, max)` | — | не шире `max` |
| `move_cstr(x, s, pair)` | `moveCStr` | C-строка: текст между `~` — цветом старшего байта `pair` |
| `move_buf(x, cells, attr)` | `moveBuf` | готовые ячейки (`attr != 0` — перекрасить) |
| `put_char(x, ch)` | `putChar` | символ без смены цвета |
| `put_attribute(x, attr)` | `putAttribute` | цвет без смены символа |

### `DrawCtx` — вывод на экран

| Метод | Аналог | Что делает |
|-------|--------|-----------|
| `write_line(x, y, w, h, &buf)` | `writeLine` | буфер в `h` строк подряд |
| `write_cells(x, y, &cells)` | `writeBuf` | ячейки одной строки |
| `write_char(x, y, ch, attr, n)` | `writeChar` | повтор символа |
| `write_str(x, y, s, attr)` | `writeStr` | строка |
| `fill(r, ch, attr)` | — | закрасить прямоугольник |
| `recolor(r, attr)` | — | перекрасить уже нарисованное |
| `shadow(r)` | — | тень справа и снизу от `r` |
| `color(n)`, `map_color(n)` | `getColor(n)` (младший байт) | атрибут по номеру цвета палитры |
| `get_color(0xHHLL)` | `getColor` | пара: младший байт — обычный цвет, старший — подсветка |
| `sub(r, palette)` | — | контекст для дочернего представления |
| `origin()`, `size`, `commands` | — | абсолютная позиция, размер, набор разрешённых команд |

Все координаты — локальные. Рисуйте всю свою область: незакрашенные
ячейки покажут то, что под представлением.

### Цвета

В `draw` используются не сами цвета, а **номера** в палитре
представления. Метод `palette()` возвращает палитру — массив байтов, где
байт № n (считая с 1) — номер цвета в палитре владельца. Номер
переводится по цепочке до палитры приложения, которая даёт настоящий
атрибут. Пустая палитра (по умолчанию) передаёт номера владельцу без
изменений.

Готовые палитры стандартных элементов лежат в `color::palettes`
(`STATIC_TEXT`, `INPUT_LINE`, `BUTTON`, `LIST_VIEWER`, `SCROLLER`…).
Если ваш элемент похож на стандартный, возьмите его палитру — он будет
правильно выглядеть и в синем окне, и в сером диалоге. Подробнее —
[глава 11](11-colors.md).

### Курсор

Курсор виден, если у представления в фокусе есть `sf::CURSOR_VIS`.
Позицию задаёт `vs.set_cursor(x, y)` (локальные координаты); блочный
курсор — `vs.block_cursor()`. Программа сама найдёт представление в
фокусе и поставит курсор терминала.

## Фокус и выбор

* Представление с `of::SELECTABLE` можно выбрать Tab-ом (в окне) или
  щелчком (это делает `default_handle_event`, если вы его вызываете).
* Выбрать программно: `ctx.select(id)` — для соседей по группе; `ctx.focus(id)`
  — где угодно, вместе с окном; при построении диалога — `dialog.select(id)`.
* При вставке в группу представление становится текущим, если текущего
  ещё нет или у него `of::TOP_SELECT` (окна). Поэтому в диалоге фокус
  получает первый вставленный выбираемый элемент.
* `sf::DISABLED` делает представление недоступным: оно не получает клавиши
  и мышь и пропускается при Tab. Снимите заодно `of::SELECTABLE`, чтобы
  оно не получило фокус при вставке.

## Проверка перед закрытием: `valid`

Программа спрашивает `valid(cmd, ctx)` у окна перед его закрытием
(`cm::CLOSE`), у диалога — перед завершением модального цикла (`cm::OK` и
др., кроме `cm::CANCEL`), у всего дерева — перед выходом (`cm::QUIT`).
Группа спрашивает всех детей. Верните `false`, чтобы отменить действие;
сообщение об ошибке покажите через `ctx.exec(Box::new(build_message_box(...)), None, 0)`,
фокус верните себе через `ctx.focus(self.vs.id)`.

Команда `cm::VALID` (0) — проверка «представление вообще в рабочем
состоянии» при вставке окна (`insert_window` не вставит окно, если
`valid(cm::VALID)` вернул `false`).

## Обмен данными: `Value`

В C++ данные диалога передавались блоком памяти (`setData(&rec)`). Здесь —
типизированное значение `Value`:

| Вариант | Чьи данные |
|---------|-----------|
| `Value::Str(String)` | `InputLine`, `Memo`, `FileDialog` (полное имя) |
| `Value::U32(u32)` | `Cluster`: маска флажков, номер переключателя, биты многопозиционных флажков |
| `Value::Int(i64)` | `ListBox` (номер выбранного), `InputLong` (число) |
| `Value::List(Vec<Value>)` | группа: значения детей с данными в порядке вставки; `Combo` (строки списка) |
| `Value::None` | нет данных |

Методы доступа: `as_str()`, `as_u32()`, `as_int()`, `as_list()`.

Своё представление участвует в обмене, если `has_data()` возвращает `true`.
Как собрать данные целого диалога — [глава 7](07-dialogs.md#обмен-данными).

## Составное представление на основе группы

Если элемент сам состоит из нескольких представлений, постройте его
вокруг `Group`:

```rust
use tvision::*;

/// Панель: заголовок и строка ввода, ведёт себя как одно целое.
struct Field {
    group: Group,
}

impl Field {
    fn new(r: Rect, title: &str) -> Self {
        let mut group = Group::new(r);
        group.vs.options |= of::SELECTABLE;
        let w = r.width();
        group.add(StaticText::new(Rect::new(0, 0, w, 1), title));
        group.add(InputLine::new(Rect::new(0, 1, w, 2), 80));
        Field { group }
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
    fn set_state(&mut self, flag: u16, enable: bool) {
        self.group.set_group_state(flag, enable);
    }
    fn valid(&mut self, cmd: u16, ctx: &mut Ctx) -> bool {
        self.group.valid(cmd, ctx)
    }
}
```

`impl_group_basics!` уже делегирует группе данные, справку, изменение
размеров и доступ к детям (`as_group`), поэтому такой элемент участвует в
обмене данными (`Value::List` из значений детей), и `find` находит его
детей.

## Приведение типов

`View` расширяет `Any`, поэтому конкретный тип можно получить обратно:

```rust
use tvision::*;

fn read(view: &dyn View) -> Option<String> {
    view.as_any().downcast_ref::<InputLine>().map(|il| il.text())
}

fn take(view: Box<dyn View>) -> Option<Dialog> {
    view.into_any().downcast::<Dialog>().ok().map(|b| *b)
}
```

Обычно проще пользоваться `group.find::<T>(id)` / `app.find::<T>(id)` и
`app.exec_dialog(d)`, которые делают приведение сами.
