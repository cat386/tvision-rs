# Урок 6. Анкета: флажки, переключатели, список

В диалоге бывает много полей разных видов. Читать каждое по id, как в
прошлом уроке, утомительно — поэтому у диалога есть **обмен данными**:
все поля заполняются и читаются одним вызовом.

## Программа

Файл `examples/learn_06_form.rs`:

```rust,ignore
//! Урок 6. Анкета: флажки, переключатели, список и обмен данными.
//!
//! Запуск: `cargo run --example learn_06_form`. F2 — анкета, Alt+X — выход.

use tvision::*;

const CM_FORM: u16 = 100;

const DRINKS: [&str; 3] = ["Чай", "Кофе", "Сок"];
const HOBBIES: [&str; 4] = ["Книги", "Спорт", "Музыка", "Игры"];
const CITIES: [&str; 6] = ["Москва", "Тула", "Казань", "Тверь", "Сочи", "Омск"];

/// Данные анкеты — обычная структура Rust.
#[derive(Clone, Debug, Default, PartialEq)]
struct Profile {
    name: String,
    age: String,
    drink: u32,   // номер выбранного переключателя: 0 — чай, 1 — кофе, 2 — сок
    hobbies: u32, // флажки: бит 0 — книги, бит 1 — спорт, …
    city: i64,    // номер строки в списке городов
}

impl Profile {
    /// Превратить в данные для диалога. Порядок — как порядок `add`
    /// элементов, у которых есть данные (подписи и кнопки не считаются).
    fn to_value(&self) -> Value {
        Value::List(vec![
            Value::Str(self.name.clone()), // строка ввода «Имя»
            Value::Str(self.age.clone()),  // строка ввода «Возраст»
            Value::U32(self.drink),        // переключатели
            Value::U32(self.hobbies),      // флажки
            Value::Int(self.city),         // список
        ])
    }

    /// Обратно: из данных диалога.
    fn from_value(v: &Value) -> Profile {
        let l = v.as_list().unwrap_or(&[]);
        Profile {
            name: l.first().and_then(|x| x.as_str()).unwrap_or("").to_string(),
            age: l.get(1).and_then(|x| x.as_str()).unwrap_or("").to_string(),
            drink: l.get(2).and_then(|x| x.as_u32()).unwrap_or(0),
            hobbies: l.get(3).and_then(|x| x.as_u32()).unwrap_or(0),
            city: l.get(4).and_then(|x| x.as_int()).unwrap_or(0),
        }
    }

    /// Описание словами — для окна сообщения.
    fn describe(&self) -> String {
        let hobbies: Vec<&str> = (0..HOBBIES.len()).filter(|i| self.hobbies & (1 << i) != 0).map(|i| HOBBIES[i]).collect();
        format!(
            "Имя: {}\nВозраст: {}\nНапиток: {}\nУвлечения: {}\nГород: {}",
            self.name,
            self.age,
            DRINKS[self.drink as usize % DRINKS.len()],
            if hobbies.is_empty() { "нет".to_string() } else { hobbies.join(", ") },
            CITIES[self.city as usize % CITIES.len()]
        )
    }
}

/// Диалог «Анкета».
fn form_dialog(p: &Profile) -> Dialog {
    let mut d = Dialog::centered(56, 19, "Анкета");

    // 1. Имя — обычная строка ввода.
    let name = d.add(InputLine::new(Rect::new(14, 2, 52, 3), 40));
    d.add(Label::new(Rect::new(2, 2, 13, 3), "~И~мя:", Some(name)));

    // 2. Возраст — строка с проверкой: целое от 1 до 120.
    let age = d.add(InputLine::new(Rect::new(14, 4, 20, 5), 3).with_validator(RangeValidator::new(1, 120)));
    d.add(Label::new(Rect::new(2, 4, 13, 5), "~В~озраст:", Some(age)));

    // 3. Переключатели: выбрать можно только один вариант.
    let drink = d.add(Cluster::radio_buttons(Rect::new(3, 7, 20, 10), &["~Ч~ай", "~К~офе", "~С~ок"]));
    d.add(Label::new(Rect::new(2, 6, 13, 7), "Напиток", Some(drink)));

    // 4. Флажки: можно отметить сколько угодно. Высота 2 — значит, 2 строки
    //    в столбце, и 4 флажка лягут в два столбца.
    let hobbies = d.add(Cluster::check_boxes(Rect::new(23, 7, 52, 9), &["~К~ниги", "С~п~орт", "~М~узыка", "И~г~ры"]));
    d.add(Label::new(Rect::new(22, 6, 35, 7), "Увлечения", Some(hobbies)));

    // 5. Список городов с полосой прокрутки справа.
    let sb = d.add(ScrollBar::new(Rect::new(20, 12, 21, 15)));
    let mut list = ListBox::new(Rect::new(3, 12, 20, 15), 1, Some(sb));
    list.set_items(CITIES.iter().map(|s| s.to_string()).collect());
    let list = d.add(list);
    d.add(Label::new(Rect::new(2, 11, 13, 12), "~Г~ород", Some(list)));

    d.add(Button::new(Rect::new(30, 12, 42, 14), "O~K~", cm::OK, bf::DEFAULT));
    d.add(Button::new(Rect::new(30, 15, 42, 17), "Отмена", cm::CANCEL, bf::NORMAL));

    // Заполнить все элементы разом — значениями из структуры.
    d.set_data(&p.to_value());
    d.select(name); // курсор — в поле «Имя»
    d
}

struct App {
    profile: Profile,
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() != Some(CM_FORM) {
            return;
        }
        ev.clear();
        let (cmd, d) = app.exec_dialog(form_dialog(&self.profile));
        if cmd == cm::OK {
            // Прочитать все элементы разом.
            self.profile = Profile::from_value(&d.get_data());
            message_box_rect(app, Rect::new(0, 0, 50, 13), &self.profile.describe(), mf::INFORMATION | mf::OK_BUTTON, true);
        }
    }
}

fn status() -> Vec<StatusDef> {
    vec![StatusDef::all(vec![StatusItem::new("~F2~ Анкета", kb::F2, CM_FORM), StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT)])]
}

fn main() -> std::io::Result<()> {
    locale::set_language(locale::Language::Ru);
    let mut app = Program::with_terminal(None, Some(status()))?;
    let start = Profile { name: "Аня".into(), age: "30".into(), drink: 1, hobbies: 0b0101, city: 2 };
    app.put_event(Event::command_event(CM_FORM));
    app.run(&mut App { profile: start });
    Ok(())
}
```

Запуск: `cargo run --example learn_06_form`.

```text
╔═[■]═══════════════════ Анкета ═══════════════════════╗
║                                                      ║
║  Имя:        Аня                                     ║
║                                                      ║
║  Возраст:    30                                      ║
║                                                      ║
║  Напиток             Увлечения                       ║
║   ( ) Чай             [X] Книги  [X] Музыка          ║
║   (•) Кофе            [ ] Спорт  [ ] Игры            ║
║   ( ) Сок                                            ║
║                                                      ║
║  Город                                               ║
║   Москва          ▲              OK    ▄             ║
║   Тула            ■           ▀▀▀▀▀▀▀▀▀▀             ║
║   Казань          ▼                                  ║
║                                Отмена  ▄             ║
║                               ▀▀▀▀▀▀▀▀▀▀             ║
╚══════════════════════════════════════════════════════╝
```

Перемещение между полями — Tab и Shift+Tab или мышью. В переключателях и
флажках — стрелки и пробел (или горячие буквы). В списке — стрелки.
Попробуйте ввести возраст 200 и нажать OK: появится сообщение, а диалог
не закроется.

## Новые элементы

### Переключатели — выбрать один вариант

```rust,ignore
Cluster::radio_buttons(Rect::new(3, 7, 20, 10), &["~Ч~ай", "~К~офе", "~С~ок"])
```

Как кнопки на старом радиоприёмнике: нажал одну — остальные отжались.
Значение — **номер** выбранного варианта: 0, 1, 2. Высота прямоугольника
— число вариантов (по строке на вариант).

### Флажки — отметить сколько угодно

```rust,ignore
Cluster::check_boxes(Rect::new(23, 7, 52, 9), &["~К~ниги", "С~п~орт", "~М~узыка", "И~г~ры"])
```

Значение — **набор битов**: бит 0 — первый флажок, бит 1 — второй и так
далее. Отмечены «Книги» и «Музыка» — значение `0b0101` (1 + 4 = 5).
Если вариантов больше, чем строк в прямоугольнике, они раскладываются
по столбцам: здесь высота 2, поэтому четыре флажка — в два столбца.

Проверить, отмечен ли флажок `i`: `hobbies & (1 << i) != 0`.

### Список

```rust,ignore
let sb = d.add(ScrollBar::new(Rect::new(20, 12, 21, 15)));
let mut list = ListBox::new(Rect::new(3, 12, 20, 15), 1, Some(sb));
list.set_items(CITIES.iter().map(|s| s.to_string()).collect());
```

* `ListBox::new(прямоугольник, число столбцов, полоса прокрутки)`.
* Полосу прокрутки (`ScrollBar` шириной 1) кладут рядом **в тот же
  диалог** и передают списку её id — так они связываются.
* `set_items` — строки списка.
* Значение — номер выбранной строки (тип `i64`).

### Проверка ввода

```rust,ignore
InputLine::new(Rect::new(14, 4, 20, 5), 3).with_validator(RangeValidator::new(1, 120))
```

**Валидатор** проверяет, что ввели. `RangeValidator::new(1, 120)` — целое
от 1 до 120. По ходу набора он не пропустит буквы, а при нажатии OK
проверит диапазон и, если число не подходит, покажет сообщение и
вернёт курсор в поле. Есть и другие валидаторы — например, маска
`PictureValidator::new("##.##.####", true)` для даты; см.
[справочник](25-validators.md).

## Обмен данными

### Значения элементов

У каждого элемента, который что-то хранит, есть **значение** типа
`Value`:

| Элемент | Значение |
|---------|----------|
| `InputLine` (строка ввода) | `Value::Str(текст)` |
| переключатели | `Value::U32(номер)` |
| флажки | `Value::U32(биты)` |
| `ListBox` (список) | `Value::Int(номер строки)` |
| подписи, кнопки, полосы прокрутки | значения нет |

У диалога значение — **список** значений всех таких элементов **в том
порядке, в каком их добавляли** (`Value::List`).

### Структура ↔ диалог

Удобно держать данные анкеты в своей структуре и написать два метода
перевода:

```rust,ignore
fn to_value(&self) -> Value {
    Value::List(vec![
        Value::Str(self.name.clone()), // 1-й элемент с данными: «Имя»
        Value::Str(self.age.clone()),  // 2-й: «Возраст»
        Value::U32(self.drink),        // 3-й: переключатели
        Value::U32(self.hobbies),      // 4-й: флажки
        Value::Int(self.city),         // 5-й: список
    ])
}
```

`from_value` делает обратное: достаёт значения по номерам. Методы
`as_str()`, `as_u32()`, `as_int()` возвращают `Some(…)`, если значение
нужного вида, и `None` — если нет; `unwrap_or` даёт запасное значение.

И тогда работа с диалогом — две строки:

```rust,ignore
d.set_data(&p.to_value());                     // заполнить диалог
let p = Profile::from_value(&d.get_data());    // прочитать после OK
```

> **Порядок важен.** Если поменять местами `add` двух полей, поменяется и
> порядок значений. Держите `to_value` рядом с функцией, которая строит
> диалог, и меняйте их вместе. Тест `data_roundtrip` в конце файла ловит
> такие ошибки: он заполняет диалог и проверяет, что прочиталось то же.

## Попробуйте

1. Добавьте флажок «Кино» пятым. Что придётся поменять? (Только список
   надписей `HOBBIES` и подписи флажков — биты сдвигаются сами.)
2. Добавьте между «Возрастом» и «Напитком» поле «Телефон» с фильтром:
   `FilterValidator::new("0123456789+-() ")` — буквы набрать не
   получится. Не забудьте добавить его значение в `to_value` и
   `from_value` **на правильное место**.
3. Сделайте список в два столбца: `ListBox::new(…, 2, …)`.

## Важно запомнить

* Переключатели — номер, флажки — биты, список — номер строки, строка
  ввода — текст.
* `d.set_data(…)` / `d.get_data()` — все поля разом, в порядке `add`.
* Валидатор не пускает неправильные данные дальше кнопки OK.

Следующий урок — [своё представление](07-own-view.md).
