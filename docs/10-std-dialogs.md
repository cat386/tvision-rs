# 10. Стандартные диалоги и окна

## Выбор файла `FileDialog`

```rust
use tvision::*;

fn open(app: &mut Program) -> Option<std::path::PathBuf> {
    // Шаблон, заголовок, подпись строки ввода, кнопки, номер списка истории.
    let d = FileDialog::new("*.txt", "Открыть файл", "~И~мя", fd::OPEN_BUTTON, 100);
    let (cmd, d) = app.exec_dialog(d);
    if cmd != cm::CANCEL {
        Some(d.file_name())
    } else {
        None
    }
}
```

`FileDialog::new(wildcard, title, input_name, options, hist_id)`:

* строка ввода с историей (`hist_id`) — имя файла или шаблон;
* список файлов в два столбца: каталоги (с разделителем на конце) и файлы
  по шаблону, поиск по первым буквам, двойной щелчок — выбрать;
* панель сведений внизу: путь и шаблон, размер и дата выбранного файла;
* кнопки по флагам `fd`.

| Флаг `fd` | Кнопка | Код завершения |
|-----------|--------|----------------|
| `OPEN_BUTTON` | «Открыть» | `cm::FILE_OPEN` |
| `OK_BUTTON` | «OK» | `cm::FILE_OPEN` |
| `REPLACE_BUTTON` | «Заменить» | `cm::FILE_REPLACE` |
| `CLEAR_BUTTON` | «Очистить» | `cm::FILE_CLEAR` |
| `HELP_BUTTON` | «Справка» | `cm::HELP` (дойдёт до `modal_hook`) |
| `NO_LOAD_DIR` | — | не читать каталог при создании |

Кнопка «Отмена» есть всегда. Проверяйте `cmd != cm::CANCEL` — разные
кнопки возвращают разные коды.

Если ввести шаблон (`*.rs`) или каталог и нажать Enter, диалог перейдёт
туда и останется открытым. Неверное имя — сообщение «Неверное имя файла».

| Метод / поле | Что |
|--------------|-----|
| `file_name()` | полное имя выбранного файла (`PathBuf`) |
| `wildcard`, `directory` | текущий шаблон и каталог |
| `read_directory()` | перечитать каталог |
| данные (`get_data`) | `Value::Str` — полное имя |

Вспомогательные функции модуля: `wild_match(pattern, name)` (`*`, `?`;
на Windows без учёта регистра), `is_wild(s)`, `read_directory(dir, wildcard)`,
`civil_time(t)`.

## Смена каталога `ChDirDialog`

```rust
use tvision::*;

fn change_dir(app: &mut Program) {
    let d = ChDirDialog::new(cd::NORMAL, 0);
    app.exec_dialog(d); // на OK текущий каталог программы меняется
}
```

Строка ввода с именем каталога, дерево каталогов (корень → путь →
подкаталоги), кнопки «Перейти» (`cm::CHANGE_DIR`) и «Вернуть»
(`cm::REVERT`). При OK вызывается `std::env::set_current_dir`; при ошибке
показывается сообщение, и диалог остаётся открытым. Флаги `cd`:
`NORMAL`, `NO_LOAD_DIR`, `HELP_BUTTON`. Метод `directory()` — выбранный
каталог.

## Настройка цветов `ColorDialog`

```rust
use tvision::*;

fn colors(app: &mut Program) {
    let d = ColorDialog::new(&app.palette, standard_color_groups());
    let (cmd, d) = app.exec_dialog(d);
    if cmd == cm::OK {
        app.palette = d.palette.clone(); // применить ко всей программе
    }
}
```

Слева — группы (Desktop, Menus, Blue window, Dialogs…) и элементы группы,
справа — сетки 16 цветов текста и фона и образец. Кнопки
«Текст 256…» и «Фон 256…» открывают `Color256Dialog` — сетку всех
256 цветов xterm (32 × 8, клавиши-стрелки, Home/End, PgUp/PgDn, мышь)
с номером, RGB и образцом. Сетки 16 цветов меняют только свою часть
атрибута: если у текста цвет xterm, выбор фона в сетке его не сбросит.
Диалог правит **копию** палитры приложения (`palette`); применяет её
программа.

`Color256Dialog` можно открыть и отдельно:

```rust
use tvision::*;

/// Выбрать фон для атрибута `a`; `None` — отмена.
fn pick_background(app: &mut Program, a: Attr) -> Option<Attr> {
    let (cmd, d) = app.exec_dialog(Color256Dialog::new(ColorSel::Background, a));
    (cmd == cm::OK).then(|| d.result()) // d.color() — сам номер 0–255
}
```

Свои группы: `ColorGroup::new("Мой редактор", &[("Текст", 6), ("Выделение", 7)])`
— пары «название, номер цвета в палитре приложения». Сохранить палитру
между запусками можно как угодно (это просто `Vec<Attr>` — числа `u32`),
например в файл.

## Справка

Справка пишется в исходном формате утилиты `tvhc` и читается программой
**напрямую** — компилировать в `.hlp` не нужно.

### Формат

```text
; комментарий (строка начинается с «;»)
.topic NoContext=0
 Моя программа
 ─────────────
Строки без пробела в начале собираются в абзацы и переносятся по ширине
окна. Строки с пробелом в начале выводятся как есть — для заголовков,
таблиц и списков.

Ссылка на другую тему: {окна:Windows}, просто {Windows}.

.topic Windows=10, WindowsAlias
 Окна
Текст темы…

.topic FileOpen
Номер не указан — будет следующий по порядку.
```

* `.topic Имя=номер, Синоним` начинает тему; номер — контекст справки
  (`help_ctx`). Без номера номера идут подряд, начиная с 2 (1 занят
  `HC_DRAGGING`).
* `{текст:Тема}` — перекрёстная ссылка с текстом, `{Тема}` — ссылка,
  текстом которой служит имя темы. `{{` — сам символ `{`; внутри ссылки
  `::` и `}}` — символы `:` и `}`.

### Показ справки по F1

```rust
use std::rc::Rc;
use tvision::*;

struct App {
    help: Rc<HelpFile>,
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(cm::HELP) {
            let ctx = app.help_ctx(); // контекст представления в фокусе
            app.exec_dialog(HelpWindow::new(self.help.clone(), ctx));
            ev.clear();
        }
    }
}

fn load_help() -> Rc<HelpFile> {
    // Можно встроить в программу: HelpFile::parse(include_str!("help.txt"))
    Rc::new(HelpFile::parse(".topic NoContext=0\nСправка пока не написана.").expect("справка"))
}
```

Чтобы F1 работал и внутри диалогов, обработайте `cm::HELP` ещё и в
`Program::modal_hook`, а в строку состояния добавьте
`StatusItem::new("~F1~ Справка", kb::F1, cm::HELP)` — она видит клавиши и
во время модального цикла.

В окне справки: Tab / Shift+Tab — по ссылкам, Enter или щелчок — перейти
по ссылке, стрелки, PgUp/PgDn и колесо — прокрутка, Esc — закрыть. Окно
можно двигать и растягивать. Истории переходов («назад») в этой версии
нет — добавьте в темы ссылку на содержание.

| Тип / метод | Что |
|-------------|-----|
| `HelpFile::parse(src)`, `HelpFile::load(path)` | разобрать исходник (ошибки — `Err(текст)`) |
| `topic_for_context(n)`, `topic_by_name(name)` | тема |
| `symbols`, `topics`, `warnings` | имена тем с номерами, темы, предупреждения (ссылки на несуществующие темы) |
| `context_constants()` | константы `pub const HC_…: u16` на Rust — замена `.h`-файла `tvhc` |
| `HelpWindow::new(file, ctx)` | окно 50×18 по центру |
| `HelpViewer` | просмотр темы (встраивается в своё окно) |

Пример `tvhc` (`cargo run --example tvhc -- help.txt hc.rs`) проверяет
исходник и записывает константы контекстов.

## Дерево `Outline`

```rust
use tvision::*;

fn tree_window(app: &mut Program) {
    let root = Node::with_children(
        "Проект",
        vec![
            Node::with_children("src", vec![Node::new("main.rs"), Node::new("lib.rs")], true),
            Node::new("Cargo.toml"),
        ],
        true, // раскрыт
    );
    let mut w = Window::new(Rect::new(5, 2, 45, 18), "Дерево", 1);
    let h = w.standard_scroll_bar(false);
    let v = w.standard_scroll_bar(true);
    w.add(Outline::new(w.interior(), Some(h), Some(v), vec![root]));
    app.insert_window(w);
}
```

* `Node::new(text)` — лист; `Node::with_children(text, children, expanded)`.
* Клавиши: стрелки, PgUp/PgDn, Home/End, Ctrl+PgUp/PgDn; `+` / `-` —
  раскрыть/свернуть, `*` — раскрыть всё; щелчок по «ветке» слева от текста
  — раскрыть/свернуть.
* Enter или двойной щелчок рассылают `cm::LIST_ITEM_SELECTED` с id дерева.
* Методы: `focused_node()`, `focused_path()` (индексы от корня),
  `node_at(i)`, `row_count()`, `set_expanded(i, bool)`, `expand_all(i)`,
  `update()` (после изменения `roots`); поле `foc` — строка с фокусом.

## Прокрутка: `Scroller`

`TScroller` в оригинале был базовым классом. Здесь `Scroller` —
**движок**, который встраивают в своё представление: он хранит смещение
(`delta`) и размер содержимого (`limit`) и связан с полосами прокрутки.

```rust
use tvision::*;

/// Просмотр длинного списка строк с прокруткой.
struct LinesView {
    vs: ViewState,
    scroller: Scroller,
    lines: Vec<String>,
}

impl LinesView {
    fn new(r: Rect, h: Option<ViewId>, v: Option<ViewId>, lines: Vec<String>) -> Self {
        let mut vs = ViewState::new(r);
        vs.options |= of::SELECTABLE;
        vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
        let mut scroller = Scroller::new(h, v);
        let w = lines.iter().map(|l| text::str_width(l)).max().unwrap_or(0) as i32;
        scroller.set_limit(w, lines.len() as i32, r.size());
        LinesView { vs, scroller, lines }
    }
}

impl View for LinesView {
    tvision::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let c = dc.color(1);
        let w = self.vs.size.x.max(0) as usize;
        let d = self.scroller.delta;
        for y in 0..self.vs.size.y {
            let mut b = DrawBuffer::new(w);
            b.move_char(0, ' ', c, w);
            if let Some(l) = self.lines.get((d.y + y) as usize) {
                let s: String = l.chars().skip(d.x.max(0) as usize).collect();
                b.move_str_limited(0, &s, c, w);
            }
            dc.write_line(0, y, self.vs.size.x, 1, &b);
        }
    }

    fn palette(&self) -> &'static [u8] {
        color::palettes::SCROLLER
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        view::default_handle_event(&self.vs, ev, ctx);
        let size = self.vs.size;
        // Полосы, колесо, стрелки и PgUp/PgDn (если в фокусе).
        self.scroller.handle(ev, size, self.vs.is_focused());
        // Сообщить полосам новые параметры.
        self.scroller.sync(size, ctx);
    }

    fn change_bounds(&mut self, r: Rect) {
        self.vs.set_bounds(r);
        let l = self.scroller.limit;
        self.scroller.set_limit(l.x, l.y, r.size());
    }
}
```

| Метод | Что делает |
|-------|-----------|
| `Scroller::new(h, v)` | движок с полосами |
| `set_limit(x, y, size)` | размер содержимого |
| `scroll_to(x, y, size)` | прокрутить (в пределах) |
| `handle(ev, size, focused) -> bool` | обработать прокрутку; `true`, если событие взято |
| `sync(size, ctx)` | отправить полосам текущие значения (только при изменениях) |

## Окно вывода `Terminal`

`Terminal` (`TTerminal`) — представление-журнал с кольцевым буфером
строк: дописывайте текст, прокрутка следует за концом, пока пользователь
не прокрутил вверх.

```rust
use tvision::*;

fn log_window(app: &mut Program) -> ViewId {
    let mut w = Window::new(Rect::new(0, 0, 60, 15), "Журнал", 0);
    let v = w.standard_scroll_bar(true);
    let log = w.add(Terminal::new(w.interior(), None, Some(v), 1000));
    app.insert_window(w);
    log
}

fn log(app: &mut Program, id: ViewId, msg: &str) {
    if let Some(t) = app.find_mut::<Terminal>(id) {
        t.write_line(msg);
    }
}
```

Методы: `write(text)` (`\n` — новая строка), `write_line(text)`,
`clear()`, `lines()`; `max_lines` — сколько строк хранить.

## Окно истории, выпадающий список

`HistoryWindow` и `ComboWindow` открываются сами значками `History` и
`Combo` ([глава 7](07-dialogs.md)); создавать их вручную обычно не нужно.
