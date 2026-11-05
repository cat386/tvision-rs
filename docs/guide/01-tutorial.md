# Урок: записная книжка «Контакты»

В этом уроке мы шаг за шагом построим настоящее приложение — записную
книжку с таблицей контактов, диалогом правки, проверкой ввода, файлами и
тестами. Готовый код — `examples/guide_contacts.rs`:

```text
cargo run --example guide_contacts            # с образцом данных
cargo run --example guide_contacts -- my.tsv  # открыть или создать файл
cargo test --example guide_contacts           # тесты из шага 6
```

```text
  Файл  Контакт
┌──────────────────── Контакты — без имени ──────────────────────1─────┐
│         Имя          │     Телефон      │        Почта         │ ★ │▲
│──────────────────────┼──────────────────┼──────────────────────┼───┼▒
│Анна Петрова          │+7 900 111-22-33  │anna@example.ru       │ ★ │▒
│Борис Смирнов         │+7 900 222-33-44  │boris@example.ru      │   │▒
 Ins Добавить  F4 Изменить  Del Удалить  F2 Сохранить  Alt-X Выход
```

Фрагменты кода ниже взяты из этого файла; чтобы увидеть их вместе, держите
его открытым рядом. Подробности о каждом типе — в
[справочнике](README.md#справочник-по-классам).

## Шаг 0. Как устроена программа на tvision-rs

Прежде чем писать код, полезно знать четыре идеи — они такие же, как в
Turbo Vision для DOS:

1. **Всё на экране — представления** (`View`): строка меню, рабочий стол,
   окно, кнопка. Представления вложены друг в друга: программа → рабочий
   стол → окно → таблица. Группы (`Group`, `Window`, `Dialog`) владеют
   детьми.
2. **Программа живёт в цикле событий.** `Program::run` ждёт нажатие
   клавиши или мыши, доставляет событие нужному представлению, перерисовывает
   экран — и так до команды выхода.
3. **Команды** — числа (`u16`). Пункт меню, кнопка, клавиша строки
   состояния посылают команду; кто-то её обрабатывает. Команды можно
   запрещать — тогда пункты меню становятся серыми.
4. **Свой код приложения** — обработчик `AppHandler`. Ему достаются
   команды, которые не обработали представления.

Важное отличие от C++: представление не хранит указатель на своё окно.
Чтобы что-то сделать «снаружи себя» (послать команду, закрыть себя,
открыть модальный диалог), оно пишет запрос в контекст `Ctx`, и владелец
выполняет его сразу после обработки события.

## Шаг 1. Программа, меню и строка состояния

Сначала — номера своих команд. Стандартные (`cm::OK`, `cm::QUIT`,
`cm::SAVE`…) занимают номера до 100, свои начинаем со 100:

```rust,ignore
const CM_ADD: u16 = 100;
const CM_EDIT: u16 = 101;
const CM_DELETE: u16 = 102;
const CM_SORT: u16 = 103;
const CM_EXIT: u16 = 104;
```

> Номера до 255 можно **запрещать** (`disable_command`); команды с
> большими номерами разрешены всегда. Поэтому команды, которые бывают
> недоступны, нумеруйте от 100 до 255.

Меню собирается цепочкой вызовов. Тильды выделяют горячую букву: в
`"~Ф~айл"` подменю открывается по Alt+Ф (и по Alt+A в латинской
раскладке — см. [язык и раскладки](../13-locale.md)).

```rust,ignore
fn menu() -> Menu {
    Menu::new()
        .submenu(
            "~Ф~айл",
            Menu::new()
                .item("~Н~овый", cm::NEW)
                .item_key("~О~ткрыть...", cm::OPEN, kb::F3, "F3")
                .item_key("~С~охранить", cm::SAVE, kb::F2, "F2")
                .item("Сохранить ~к~ак...", cm::SAVE_AS)
                .separator()
                .item_key("~В~ыход", CM_EXIT, kb::ALT_X, "Alt-X"),
        )
        .submenu(
            "~К~онтакт",
            Menu::new()
                .item_key("~Д~обавить...", CM_ADD, kb::key(Key::Ins), "Ins")
                .item_key("~И~зменить...", CM_EDIT, kb::f(4), "F4")
                .item_key("~У~далить", CM_DELETE, kb::key(Key::Del), "Del")
                .separator()
                .item("~С~ортировать по имени", CM_SORT),
        )
}
```

`item_key(название, команда, клавиша, подпись)` — пункт с клавишей
быстрого вызова: F3 работает, даже когда меню закрыто. Подпись `"F3"`
только рисуется справа от названия.

Строка состояния — список наборов пунктов (`StatusDef`); набор выбирается
по контексту справки. Нам хватит одного набора на все случаи:

```rust,ignore
fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![
        StatusItem::new("~Ins~ Добавить", kb::key(Key::Ins), CM_ADD),
        StatusItem::new("~F4~ Изменить", kb::f(4), CM_EDIT),
        StatusItem::new("~Del~ Удалить", kb::key(Key::Del), CM_DELETE),
        StatusItem::new("~F2~ Сохранить", kb::F2, cm::SAVE),
        StatusItem::new("~Alt-X~ Выход", kb::ALT_X, CM_EXIT),
        StatusItem::new("", kb::F10, cm::MENU), // невидимый пункт: F10 — меню
    ])]
}
```

Пункт с пустым текстом не виден, но его клавиша работает — так F10
открывает меню.

> **Осторожно с Enter.** Строка состояния видит клавиши раньше всех, даже
> во время модальных диалогов. Пункт на Enter забрал бы Enter у диалогов,
> поэтому «Изменить» висит на F4.

Программа создаётся одной строкой и запускается методом `run`, которому
передаётся обработчик приложения:

```rust,ignore
fn main() -> std::io::Result<()> {
    let mut app = Program::with_terminal(Some(menu()), Some(status()))?;
    // …открыть окно (шаг 2)…
    app.run(&mut App);
    Ok(())
}
```

`with_terminal` переводит терминал в полноэкранный режим (на Windows —
консоль), а при завершении программы возвращает его в обычное состояние.

Обработчик пока может быть пустым — `struct App; impl AppHandler for App {}`.
Такая программа уже работает: меню открывается по F10 и мышью, Alt-X…
пока ничего не делает, потому что `CM_EXIT` никто не обрабатывает. Это мы
исправим в шаге 4.

## Шаг 2. Своё окно с таблицей

Данные — обычная структура Rust:

```rust,ignore
pub struct Contact {
    pub name: String,
    pub phone: String,
    pub email: String,
    /// Номер группы (0 — «Семья», 1 — «Работа», 2 — «Друзья»).
    pub group: u32,
    pub favorite: bool,
}
```

Окно, которое показывает контакты, — **своё представление**. Оно хранит
стандартное окно `Window` (рамка, заголовок, дети) и данные:

```rust,ignore
pub struct ContactsWindow {
    pub win: Window,
    pub table: ViewId,
    pub contacts: Vec<Contact>,
    pub file: Option<PathBuf>,
    pub modified: bool,
}
```

`table` — **идентификатор** таблицы, а не ссылка на неё: таблицей владеет
группа окна, а найти её можно в любой момент через `find_mut`.

Конструктор строит содержимое окна:

```rust,ignore
pub fn new(bounds: Rect, contacts: Vec<Contact>, file: Option<PathBuf>) -> Self {
    let mut win = Window::new(bounds, "", 1);
    // Полосы прокрутки — на рамке окна.
    let v = win.standard_scroll_bar(true);
    let h = win.standard_scroll_bar(false);
    let model = VecTableModel::new(&["Имя", "Телефон", "Почта", "Группа", "★"], Vec::new());
    let cols = vec![
        TableColumn::new(22),
        TableColumn::new(18),
        TableColumn::new(22),
        TableColumn::new(8),
        TableColumn::new(3).align(Align::Center),
    ];
    let table = Table::new(win.interior(), Box::new(model), Some(v), Some(h))
        .with_columns(cols)
        .selection(SelectionMode::Row)
        .striped(true)
        .sortable(false);
    let table = win.add(table);
    let mut w = ContactsWindow { win, table, contacts, file, modified: false };
    w.reload();
    w
}
```

Что здесь происходит:

* `Window::new(границы, заголовок, номер)` — окно; номер 1 показывается в
  рамке, и окно выбирается по Alt+1.
* `standard_scroll_bar(true)` — вертикальная полоса прокрутки на правой
  рамке; возвращает id полосы. Таблица получает эти id и сама двигает
  полосы.
* `win.interior()` — внутренность окна без рамки: таблица занимает её
  целиком и растягивается вместе с окном.
* `Table` берёт строки из **модели** (`VecTableModel` — строки в памяти).
  Столбцы описывает `TableColumn`: ширина и выравнивание.
* `.sortable(false)` — щелчок по заголовку не сортирует. Мы будем
  сортировать сами (команда «Сортировать»), чтобы номер строки таблицы
  всегда совпадал с номером контакта в `contacts`.

Метод `reload` перекладывает контакты в таблицу:

```rust,ignore
pub fn reload(&mut self) {
    let rows: Vec<Vec<String>> = self.contacts.iter().map(|c| c.cells()).collect();
    let t = self.table_mut();
    let (row, _) = t.cursor();
    let cols = t.columns.clone();
    t.set_model(Box::new(VecTableModel::new(&["Имя", "Телефон", "Почта", "Группа", "★"], rows)));
    t.set_columns(cols); // `set_model` пересоздаёт столбцы — возвращаем свои
    t.set_cursor(row, 0);
    t.refresh();
    // …заголовок окна: имя файла и «*», если есть несохранённые изменения…
}
```

Теперь самое интересное — **реализация трейта `View`**. Окно построено
вокруг `Window`, поэтому почти всё делегируется ему; рутину берёт на себя
макрос `impl_group_basics!`:

```rust,ignore
impl View for ContactsWindow {
    tvision::impl_group_basics!(win.group);

    fn as_window(&self) -> Option<&Window> { Some(&self.win) }
    fn draw(&self, dc: &mut DrawCtx) { self.win.draw_window(dc); }
    fn palette(&self) -> &'static [u8] { self.win.palette }
    fn set_state(&mut self, flag: u16, enable: bool) { self.win.set_window_state(flag, enable); }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        let double = matches!(ev, Event::MouseDown(m) if m.double);
        self.win.handle_window_event(ev, ctx);
        // Таблица не поглощает Enter, и он возвращается к окну: превращаем
        // его (и двойной щелчок) в команду «Изменить».
        if matches!(ev, Event::KeyDown(k) if k.key == Key::Enter && k.mods == Mods::NONE) || double {
            ctx.post_command(CM_EDIT, Info::None);
            ev.clear();
        }
    }
}
```

* `as_window` сообщает программе, что это окно: так работают команды
  «Закрыть» (Alt-F3), «Распахнуть» (F5) и выбор по Alt+номер.
* `handle_event` сначала отдаёт событие окну (а оно — таблице). Если
  событие осталось необработанным (`ev` не очищен), окно решает, что с ним
  делать. `ctx.post_command` ставит команду в очередь — её получит
  обработчик приложения.
* `ev.clear()` помечает событие обработанным, чтобы оно не пошло дальше.

Это и есть «наследование» по-растовски: вместо `class ContactsWindow :
public TWindow` — структура с полем `win` и делегирование.

Окно вставляется на рабочий стол:

```rust,ignore
let r = app.desktop().group.vs.extent(); // весь рабочий стол
app.insert_window(ContactsWindow::new(r, contacts, file));
```

## Шаг 3. Диалог «Контакт»

Диалог — это окно с элементами управления, которое обычно выполняется
**модально**: пока оно открыто, остальная программа ждёт.

```rust,ignore
pub fn contact_dialog(title: &str, c: &Contact) -> Dialog {
    let mut d = Dialog::centered(52, 17, title);

    // Имя: не может быть пустым. Проверка — при нажатии OK.
    let name = InputLine::new(Rect::new(13, 2, 49, 3), 60)
        .with_validator(FnValidator { f: |s: &str| !s.trim().is_empty(), message: "Имя не может быть пустым.".into() });
    let name = d.add(name);
    d.add(Label::new(Rect::new(2, 2, 12, 3), "~И~мя:", Some(name)));

    // Телефон: только цифры и знаки + - ( ) и пробел — лишнее не наберётся.
    let phone = d.add(InputLine::new(Rect::new(13, 4, 49, 5), 30).with_validator(FilterValidator::new("0123456789+-() ")));
    d.add(Label::new(Rect::new(2, 4, 12, 5), "~Т~елефон:", Some(phone)));

    // Почта: пусто или с «@».
    let email = InputLine::new(Rect::new(13, 6, 49, 7), 60)
        .with_validator(FnValidator { f: |s: &str| s.trim().is_empty() || s.contains('@'), message: "В адресе почты нет «@».".into() });
    let email = d.add(email);
    d.add(Label::new(Rect::new(2, 6, 12, 7), "~П~очта:", Some(email)));

    let group = d.add(Cluster::radio_buttons(Rect::new(3, 9, 22, 12), &["~С~емья", "~Р~абота", "~Д~рузья"]));
    d.add(Label::new(Rect::new(2, 8, 12, 9), "~Г~руппа", Some(group)));

    d.add(Cluster::check_boxes(Rect::new(26, 9, 49, 10), &["Из~б~ранный"]));

    d.add(Button::new(Rect::new(12, 13, 24, 15), "O~K~", cm::OK, bf::DEFAULT));
    d.add(Button::new(Rect::new(27, 13, 39, 15), "Отмена", cm::CANCEL, bf::NORMAL));

    d.set_data(&c.to_value());
    d.select(name);
    d
}
```

Разберём по элементам:

* **Координаты** элементов отсчитываются от левого верхнего угла диалога.
  Строка 0 и столбец 0 — рамка. `Rect::new(13, 2, 49, 3)` — от столбца 13
  до 48 (правый край **не входит**), строка 2: поле высотой в одну строку.
* **`InputLine`** — строка ввода. Второй аргумент — наибольшая длина
  текста. Поле шире видимой части прокручивается.
* **Валидаторы** проверяют ввод. `FilterValidator` просто не даёт набрать
  лишние символы. `FnValidator` проверяет готовое значение, когда
  пользователь нажимает OK: если проверка не прошла, появляется сообщение,
  фокус возвращается в поле, а диалог остаётся открытым. Есть ещё маски
  (`PictureValidator`), диапазоны чисел и списки допустимых значений — см.
  [справочник](15-validators.md).
* **`Label`** — подпись, привязанная к полю: щелчок по ней или Alt+буква
  переводят фокус в поле, а сама подпись подсвечивается, когда поле в
  фокусе.
* **`Cluster`** — группа переключателей (`radio_buttons`) или флажков
  (`check_boxes`). Стрелки двигают выбор, пробел отмечает.
* **`Button`** с флагом `bf::DEFAULT` нажимается по Enter. Команды
  `cm::OK` и `cm::CANCEL` завершают модальный диалог; Esc — тоже
  `cm::CANCEL`.
* **Порядок `add`** — это порядок перехода по Tab и порядок значений в
  данных диалога.

### Обмен данными

Диалог заполняется и читается **одним значением** `Value` — списком
значений всех элементов, у которых есть данные, в порядке вставки. У
строки ввода значение — `Value::Str`, у переключателей — номер
выбранного (`Value::U32`), у флажков — битовая маска (`Value::U32`).
Подписи и кнопки данных не имеют и в список не входят.

```rust,ignore
fn to_value(&self) -> Value {
    Value::List(vec![
        Value::Str(self.name.clone()),
        Value::Str(self.phone.clone()),
        Value::Str(self.email.clone()),
        Value::U32(self.group),               // переключатели: номер выбранного
        Value::U32(self.favorite as u32),     // флажки: битовая маска
    ])
}

fn from_value(v: &Value) -> Self {
    let l = v.as_list().unwrap_or(&[]);
    let s = |i: usize| l.get(i).and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
    let n = |i: usize| l.get(i).and_then(|x| x.as_u32()).unwrap_or(0);
    Contact { name: s(0), phone: s(1), email: s(2), group: n(3), favorite: n(4) & 1 != 0 }
}
```

Это прямой аналог записи `struct TDialogData` из C++: там данные
копировались в память побайтно, здесь — типизированным списком.

### Запуск диалога

```rust,ignore
fn edit_contact(app: &mut Program, title: &str, c: &Contact) -> Option<Contact> {
    let (cmd, d) = app.exec_dialog(contact_dialog(title, c));
    (cmd == cm::OK).then(|| Contact::from_value(&d.get_data()))
}
```

`exec_dialog` выполняет диалог модально и **возвращает его обратно** вместе
с командой, которой он закрылся. Поэтому прочитать данные можно уже после
закрытия — диалог никуда не делся.

## Шаг 4. Команды: добавить, изменить, удалить

Все команды, которые не обработали окна, приходят в `AppHandler::handle_event`:

```rust,ignore
impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        let Some(cmd) = ev.command() else { return };
        match cmd {
            CM_ADD => self.add(app),
            CM_EDIT => self.edit(app),
            CM_DELETE => self.delete(app),
            CM_SORT => self.sort(app),
            // …команды файлов (шаг 5)…
            CM_EXIT => {
                if self.confirm_save(app) {
                    app.quit();
                }
            }
            _ => return,
        }
        ev.clear();
    }
}
```

Найти окно контактов на рабочем столе помогает приведение типов:
представления хранятся как `Box<dyn View>`, а `as_any_mut().downcast_mut`
возвращает конкретный тип:

```rust,ignore
fn window(app: &mut Program) -> Option<&mut ContactsWindow> {
    let desk = &mut app.desktop_mut().group;
    desk.children.iter_mut().find_map(|w| w.as_any_mut().downcast_mut::<ContactsWindow>())
}
```

Правка контакта:

```rust,ignore
fn edit(&mut self, app: &mut Program) {
    // Данные копируем до диалога: пока он открыт, окно «взаймы» не держим.
    let Some((i, c)) = Self::window(app).and_then(|w| w.current().map(|i| (i, w.contacts[i].clone()))) else { return };
    if let Some(new) = edit_contact(app, "Контакт", &c) {
        let w = Self::window(app).unwrap();
        if w.contacts[i] != new {
            w.contacts[i] = new;
            w.modified = true;
            w.reload();
        }
    }
}
```

Обратите внимание на порядок: сначала **копируем** нужные данные из
окна, потом показываем диалог, потом **снова** находим окно. Так требует
проверка заимствований Rust: `exec_dialog` берёт `&mut app` целиком, и
держать ссылку на окно в это время нельзя. Этот приём встречается во всех
примерах библиотеки.

Удаление спрашивает подтверждение стандартным окном сообщения:

```rust,ignore
let q = format!("\x03Удалить «{}»?", name);
if message_box(app, &q, mf::CONFIRMATION | mf::YES_BUTTON | mf::NO_BUTTON) == cm::YES {
    // …удалить…
}
```

Символ `\x03` в начале строки центрирует текст (как в оригинале).

### Запрет недоступных команд

Редактировать нечего, если список пуст. Запрещённая команда рисуется
серым в меню и строке состояния, а её клавиша не срабатывает:

```rust,ignore
fn idle(&mut self, app: &mut Program) {
    let (has_window, has_rows) = match Self::window(app) {
        Some(w) => (true, !w.contacts.is_empty()),
        None => (false, false),
    };
    for (cmd, on) in [(CM_ADD, has_window), (CM_EDIT, has_rows), (CM_DELETE, has_rows) /* … */] {
        if on { app.enable_command(cmd); } else { app.disable_command(cmd); }
    }
}
```

`AppHandler::idle` вызывается, когда событий нет. Другой способ — метод
`View::update_commands(&self, ctx)` у представления: программа вызывает
его после каждого события, и так команды могут зависеть от того, какое
окно в фокусе (так сделан редактор).

## Шаг 5. Файлы

Чтение и запись — обычный Rust. Интерфейсная часть — стандартный диалог
выбора файла:

```rust,ignore
let d = FileDialog::new("*.tsv", "Открыть", "~И~мя", fd::OPEN_BUTTON, 100);
let (cmd, d) = app.exec_dialog(d);
if cmd == cm::CANCEL {
    return;
}
let path = d.file_name();
```

`FileDialog::new(шаблон, заголовок, подпись поля, кнопки, номер истории)`.
Последний аргумент — номер списка истории: диалог запоминает введённые
имена. Для «Сохранить как» используется кнопка `fd::OK_BUTTON`.

Перед открытием другого файла и при выходе программа спрашивает, сохранить
ли изменения:

```rust,ignore
fn confirm_save(&mut self, app: &mut Program) -> bool {
    if !Self::window(app).map(|w| w.modified).unwrap_or(false) {
        return true;
    }
    match message_box(app, "\x03Сохранить изменения?", mf::CONFIRMATION | mf::YES_NO_CANCEL) {
        cm::YES => self.save(app, false),
        cm::NO => true,
        _ => false,
    }
}
```

Выход сделан своей командой `CM_EXIT`, а не `cm::QUIT`, чтобы спросить до
завершения. Есть и способ «как в оригинале»: `cm::QUIT` вызывает у всех
окон `View::valid(cm::QUIT, ctx)`, и окно может отказаться закрываться.
Но показать вопрос изнутри `valid` можно только асинхронно, через
`ctx.exec` и `View::exec_result` — так устроен `FileEditor`. Для
приложения своя команда проще.

## Шаг 6. Тесты

Интерфейс проверяется обычными тестами `cargo test` — без настоящего
терминала. `HeadlessBackend` — «невидимый» терминал: события берутся из
очереди, кадр хранится в памяти.

```rust,ignore
fn program(input: Vec<RawEvent>) -> Program {
    let mut b = HeadlessBackend::new(100, 30);
    b.input.extend(input);
    let mut app = Program::new(Box::new(b), Some(menu()), Some(status()));
    App::open_window(&mut app, sample_contacts(), None);
    app
}

#[test]
fn add_contact_through_dialog() {
    // Ins → диалог; имя, Tab, телефон (буквы фильтр не пропустит), Enter — OK.
    let mut input = vec![key(Key::Ins)];
    input.extend(text("Дарья"));
    input.push(key(Key::Tab));
    input.extend(text("+7 (999) abc 12"));
    input.push(key(Key::Enter));
    let mut app = program(input);
    app.pump(&mut App);
    let w = App::window(&mut app).unwrap();
    assert_eq!(w.contacts.len(), 5);
    let c = w.contacts.last().unwrap();
    assert_eq!((c.name.as_str(), c.phone.as_str()), ("Дарья", "+7 (999)  12"));
}
```

* `app.pump(&mut App)` обрабатывает все события очереди, включая
  вложенные модальные диалоги.
* Когда очередь кончается, тестовый терминал сам «нажимает» Esc, чтобы
  забытое модальное окно не подвесило тест.
* `app.screen_text()` — последний кадр текстом, его можно проверять через
  `contains`.

Больше приёмов — в главе [«Тестирование»](../14-testing.md).

## Что дальше

* Добавьте поле «День рождения» с маской `PictureValidator::new("##.##.####", true)`.
* Сделайте поиск: строка ввода над таблицей и фильтрация строк
  (подсказка: `examples/three_panes.rs` — окно из нескольких частей).
* Покажите группы деревом слева (`TreeView`, [справочник](17-data-views.md#treeview)).
* Добавьте справку по F1 (`HelpFile`, `HelpWindow`, [справочник](19-std-dialogs.md#helpfile)).
