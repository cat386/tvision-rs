# Справочник: меню и строка состояния

Модули `widgets::menu` и `widgets::statusline`.

* [`Menu`](#menu) · [`MenuItem`](#menuitem) · [`MenuBar`](#menubar)
* [Контекстное меню](#контекстное-меню) · [`MenuSession`, `MenuOutcome`](#menusession-и-menuoutcome)
* [`StatusLine`](#statusline) · [`StatusDef`](#statusdef) · [`StatusItem`](#statusitem) · [`HintFn`](#hintfn)

---

## `Menu`

**Меню** — список пунктов (`TMenu`). Собирается цепочкой вызовов и
передаётся программе (`Program::with_terminal(Some(menu), …)`), где
становится строкой меню; вложенные `Menu` — выпадающие подменю.

```rust
use tvision::*;

const CM_ABOUT: u16 = 120;
const HC_ABOUT: u16 = 2;

fn menu() -> Menu {
    Menu::new()
        // «≡» открывается ещё и по Alt+Пробел.
        .submenu_key("~≡~", KeyEvent::new(Key::Char(' '), Mods::ALT), Menu::new().item("~О~ программе...", CM_ABOUT).help(HC_ABOUT))
        .submenu(
            "~Ф~айл",
            Menu::new()
                .item_key("~О~ткрыть...", cm::OPEN, kb::F3, "F3")
                .submenu("~Н~едавние", Menu::new().item("a.txt", 200).item("b.txt", 201))
                .separator()
                .item_key("~В~ыход", cm::QUIT, kb::ALT_X, "Alt-X"),
        )
}

fn check() {
    let m = menu();
    assert_eq!(m.items.len(), 2);
    assert_eq!(m.find_hot_key(&kb::F3), Some(cm::OPEN)); // поиск по всему дереву
    assert_eq!(m.find_item('Ф'), Some(1));
}
```

| Метод | Что делает |
|-------|-----------|
| `Menu::new()` | пустое меню |
| `item(name, cmd)` | пункт-команда |
| `item_key(name, cmd, key, "подпись")` | пункт с клавишей быстрого вызова — работает и при закрытом меню; подпись рисуется справа |
| `submenu(name, menu)` | подменю |
| `submenu_key(name, key, menu)` | подменю, которое открывается ещё и клавишей |
| `separator()` | разделитель |
| `help(ctx)` | контекст справки **последнего** пункта (строка состояния и F1, пока пункт подсвечен) |
| `items` | пункты (`Vec<MenuItem>`) |
| `find_hot_key(&key) -> Option<cmd>` | команда по клавише во всём дереве (`findHotKey`) |
| `find_item(c) -> Option<i>` | пункт с горячей буквой (с учётом раскладки) |
| `box_size()`, `bar_positions()` | размеры для рисования (служебные) |

Как работает меню: F10 (`cm::MENU`) или щелчок открывает строку меню;
Alt+буква — подменю с этой горячей буквой (в любой раскладке); стрелки,
Enter, Esc, буквы — внутри; запрещённые команды серые. Выбранный пункт
посылает команду обычным путём: к активному окну, затем к `AppHandler`.

> Клавиши быстрого вызова ловятся **раньше** элемента в фокусе. Не
> назначайте пунктам меню клавиши, нужные строкам ввода и редактору
> (буквы, Ctrl+стрелки, Enter).

## `MenuItem`

**Пункт меню** — перечисление:

| Вариант | Поля |
|---------|------|
| `Command` | `name`, `command`, `key: Option<KeyEvent>`, `param` (подпись клавиши), `help_ctx` |
| `SubMenu` | `name`, `menu: Rc<Menu>`, `help_ctx`, `key` |
| `Separator` | — |

Методы: `name()`, `is_separator()`, `enabled(&commands)` (доступен ли —
команда не запрещена), `help_ctx()`. Обычно пункты создают методы `Menu`,
а не вручную.

## `MenuBar`

**Строка меню** (`TMenuBar`) — первая строка экрана. Программа создаёт её
сама из переданного `Menu`; её id — `app.menu_bar_id`.

| Поле / метод | Что |
|--------------|-----|
| `MenuBar::new(bounds, menu)` | создать (обычно не нужно) |
| `menu: Rc<Menu>` | меню — его можно заменить на ходу |
| `active` | подсвеченный пункт во время работы с меню |
| `item_at(x)` | пункт под столбцом `x` |

```rust
use std::rc::Rc;
use tvision::*;

/// Заменить меню целиком (например, после смены языка).
fn replace_menu(app: &mut Program, m: Menu) {
    if let Some(id) = app.menu_bar_id {
        if let Some(bar) = app.find_mut::<MenuBar>(id) {
            bar.menu = Rc::new(m);
        }
    }
}
```

## Контекстное меню

Всплывающее меню в любом месте экрана (`TMenuPopup`). Из обработчика
приложения — `app.ctx.popup_menu(menu, pos)`, из представления —
`ctx.popup_menu(menu, pos)`. Позиция — **абсолютная** (экранная).

```rust
use std::rc::Rc;
use tvision::*;

const CM_COPY_PATH: u16 = 131;

/// Правая кнопка мыши по свободному месту рабочего стола.
fn context_menu(app: &mut Program, ev: &mut Event) {
    if let Event::MouseDown(m) = ev {
        if m.buttons & event::MB_RIGHT != 0 {
            let menu = Rc::new(Menu::new().item("~К~опировать путь", CM_COPY_PATH).separator().item("~М~озаика", cm::TILE));
            app.ctx.popup_menu(menu, m.pos); // у AppHandler координаты мыши уже экранные
            ev.clear();
        }
    }
}
```

## `MenuSession` и `MenuOutcome`

Пока меню открыто, программа ведёт **сеанс меню**: `MenuSession`
перехватывает все события, рисует выпадающие окна поверх экрана и
возвращает `MenuOutcome` — `Continue` (меню открыто), `Closed` (закрыто
без выбора) или `Command(cmd)` (выбрана команда). Вызывать его самому не
нужно; он описан для тех, кто строит своё меню.
`menu::draw_menu_box(dc, &menu, sel)` рисует выпадающее окно.

---

## `StatusLine`

**Строка состояния** (`TStatusLine`) — последняя строка экрана: подсказки
клавиш, которые сами превращают нажатия в команды. Набор пунктов зависит
от **контекста справки** представления в фокусе.

```rust
use tvision::*;

const HC_EDITOR: u16 = 100;
const CM_BUILD: u16 = 140;

fn status() -> Vec<StatusDef> {
    vec![
        // Когда в фокусе представление с vs.help_ctx == HC_EDITOR:
        StatusDef::range(HC_EDITOR, HC_EDITOR, vec![
            StatusItem::new("~F2~ Сохранить", kb::F2, cm::SAVE),
            StatusItem::new("~F9~ Собрать", kb::f(9), CM_BUILD),
            StatusItem::new("", kb::F10, cm::MENU),
        ]),
        // Во всех остальных случаях (первый подходящий набор побеждает):
        StatusDef::all(vec![
            StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT),
            StatusItem::new("", kb::F10, cm::MENU), // невидимые пункты — только клавиши
            StatusItem::new("", kb::ALT_F3, cm::CLOSE),
            StatusItem::new("", kb::F5, cm::ZOOM),
            StatusItem::new("", kb::F6, cm::NEXT),
            StatusItem::new("", kb::CTRL_F5, cm::RESIZE),
        ]),
    ]
}

fn add_hint(app: &mut Program) {
    if let Some(sl) = app.status_line_mut() {
        sl.hint = Some(Box::new(|ctx| if ctx == HC_EDITOR { "Правка текста".into() } else { String::new() }));
    }
}
```

| Поле / метод | Что |
|--------------|-----|
| `StatusLine::new(bounds, defs)` | создать (обычно делает программа) |
| `defs: Vec<StatusDef>` | наборы пунктов — можно заменить на ходу |
| `help_ctx` | текущий контекст (ставит программа) |
| `hint: Option<HintFn>` | подсказка справа от пунктов |
| `items()` | пункты для текущего контекста |

* Пункт с пустым текстом не виден, но его клавиша работает.
* Пункты с запрещённой командой серые и не срабатывают; пункт с командой
  0 — просто надпись.
* Строка видит клавиши **раньше всех и во время модальных диалогов** —
  так F1 открывает справку из любого диалога. Поэтому не вешайте пункты на
  Enter, Esc, Tab и буквы.
* Контекст справки берётся у самого глубокого представления в фокусе с
  ненулевым `help_ctx`; пока открыто меню — у подсвеченного пункта.

## `StatusDef`

**Набор пунктов** для диапазона контекстов `min..=max` (`TStatusDef`).
`StatusDef::all(items)` — для всех контекстов, `StatusDef::range(min, max,
items)` — для диапазона. Поля `min`, `max`, `items`.

## `StatusItem`

**Пункт строки состояния** (`TStatusItem`):
`StatusItem::new("~F2~ Сохранить", kb::F2, cm::SAVE)`. Поля `text`
(C-строка: часть между `~` выделяется), `key: Option<KeyEvent>`,
`command`.

## `HintFn`

`Box<dyn Fn(u16) -> String>` — функция подсказки для `StatusLine::hint`:
получает контекст справки, возвращает текст справа (`TStatusLine::hint`).
