# 14. Тестирование

Интерфейс на tvision-rs можно проверять обычными тестами `cargo test` —
без настоящего терминала. Для этого есть «невидимый» терминал
`platform::HeadlessBackend`:

* события берутся из очереди `input` (клавиши и мышь, подготовленные
  заранее);
* последний кадр хранится в памяти, его можно прочитать как текст или
  проверить по ячейкам;
* когда очередь пуста, программа считает, что пользователь бездействует;
  а чтобы тест не завис на забытом модальном окне, после нескольких
  пустых чтений терминал начинает «нажимать» Esc (не больше `esc_budget`
  раз, по умолчанию 50).

## Пример теста

```rust
use tvision::platform::{HeadlessBackend, RawEvent};
use tvision::*;

const CM_GREET: u16 = 100;

#[derive(Default)]
struct App {
    greeted: u32,
}

impl AppHandler for App {
    fn handle_event(&mut self, _app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(CM_GREET) {
            self.greeted += 1;
            ev.clear();
        }
    }
}

fn menu() -> Menu {
    Menu::new().submenu("~Ф~айл", Menu::new().item_key("~П~ривет", CM_GREET, kb::F2, "F2"))
}

fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![StatusItem::new("~F2~ Привет", kb::F2, CM_GREET)])]
}

#[test]
fn f2_greets() {
    let mut b = HeadlessBackend::new(80, 25);
    b.push_key(kb::F2); // нажатие F2
    // То же самое «сырым» событием: b.input.push_back(RawEvent::Key(kb::F2));
    let mut app = Program::new(Box::new(b), Some(menu()), Some(status()));
    let mut h = App::default();
    app.pump(&mut h); // обработать всё

    assert_eq!(h.greeted, 1);
    let screen = app.screen_text();
    let lines: Vec<&str> = screen.lines().collect();
    assert!(lines[0].starts_with("  Файл"), "строка меню: {:?}", lines[0]);
    assert!(lines[24].contains("F2 Привет"));
    // Цвет ячейки: строка меню — цвет №2 палитры приложения
    // (по умолчанию gruvbox; в классической APP_COLOR это 0x70).
    assert_eq!(app.screen().get(1, 0).attr, app.palette[1]);
}

fn main() {}
```

## Инструменты

| Что | Как |
|-----|-----|
| подготовить ввод | `HeadlessBackend::push_key(k)`, `push_click(x, y)` (нажатие + отпускание), `input.push_back(RawEvent::…)` — до `Program::new` |
| дать команду | `app.put_event(Event::command_event(cm))` |
| обработать события | `app.pump(&mut handler)` — пока очередь не опустеет; `app.step(&mut h)` — один шаг |
| экран как текст | `app.screen_text()` (строки через `\n`), `app.screen().line_text(y)` |
| ячейка | `app.screen().get(x, y)` → `Cell { ch, attr }` |
| представление | `app.find::<T>(id)` — состояние элемента, `get_data()` диалога |
| перерисовать | `app.redraw()` |

`RawEvent::Mouse { pos, buttons, mods, wheel }` описывает **состояние**
мыши: нажатие — `buttons: 1`, отпускание — следующее событие с
`buttons: 0`; движение с нажатой кнопкой — несколько событий с
`buttons: 1` и разными `pos`.

```rust
use tvision::platform::RawEvent;
use tvision::*;

fn drag(input: &mut std::collections::VecDeque<RawEvent>, from: Point, to: Point) {
    let m = |pos, buttons| RawEvent::Mouse { pos, buttons, mods: Mods::NONE, wheel: 0 };
    input.push_back(m(from, 1));
    input.push_back(m(to, 1));
    input.push_back(m(to, 0));
}
```

## Проверка модального диалога

`exec_dialog` крутит свой цикл, получая события из той же очереди
терминала. Подготовьте всё заранее: команду, которая откроет диалог,
затем ввод в диалог, затем Enter или Esc.

```rust
use tvision::platform::{HeadlessBackend, RawEvent};
use tvision::*;

const CM_ASK: u16 = 101;

#[derive(Default)]
struct Asker {
    answer: Option<String>,
}

impl AppHandler for Asker {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(CM_ASK) {
            self.answer = input_box(app, "Вопрос", "~И~мя", "", 20);
            ev.clear();
        }
    }
}

#[test]
fn typed_answer() {
    let mut b = HeadlessBackend::new(80, 25);
    for c in "Ия".chars() {
        b.input.push_back(RawEvent::Key(KeyEvent::new(Key::Char(c), Mods::NONE)));
    }
    b.push_key(kb::ENTER); // OK по умолчанию
    let mut app = Program::new(Box::new(b), None, None);
    app.put_event(Event::command_event(CM_ASK)); // команда раньше ввода
    let mut h = Asker::default();
    app.pump(&mut h);
    assert_eq!(h.answer.as_deref(), Some("Ия"));
}

fn main() {}
```

## Советы

* Проверяйте **данные** (`get_data`, поля представлений), а не только
  текст экрана: экран меняется при любой правке раскладки.
* Для экрана сравнивайте подстроки (`contains`), а не строки целиком.
* Буфер обмена у тестового терминала — внутренний, системный не
  трогается.
* `HeadlessBackend::new(w, h)` — любого размера; маленький экран полезен,
  чтобы проверить поведение при нехватке места.
