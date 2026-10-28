# 1. Быстрый старт

## Подключение

Библиотека не имеет внешних зависимостей: работа с терминалом (termios и
ANSI-последовательности на Unix, Console API на Windows) реализована внутри
крейта. Подключите её по пути или из git:

```toml
[dependencies]
tvision-rs = { path = "../tvision-rs" }
```

В коде крейт называется `tvision`:

```rust
use tvision::*;

fn main() {}
```

`use tvision::*` даёт все основные типы: `Program`, `Window`, `Dialog`,
`Button`, `Rect`, `Event`, модули констант `cm`, `kb`, `of`, `sf` и т.д.

Требования:

* Rust 2021 (stable);
* Linux, macOS, BSD — любой терминал с поддержкой ANSI и мыши
  (xterm, GNOME Terminal, Konsole, iTerm2, Terminal.app, kitty, WezTerm…);
* Windows 10 и новее — обычная консоль или Windows Terminal.

## Первая программа

Программа на Turbo Vision — это объект [`Program`](05-windows.md) (меню,
рабочий стол, строка состояния) и **обработчик приложения** — ваш тип,
реализующий трейт `AppHandler`. Обработчик получает команды, которые не
обработали окна: обычно это команды меню.

```rust
use tvision::*;

/// Своя команда. Пользовательские команды удобно нумеровать от 100.
const CM_HELLO: u16 = 100;

struct App;

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(CM_HELLO) {
            message_box(app, "\x03Привет, мир!", mf::INFORMATION | mf::OK_BUTTON);
            ev.clear(); // событие обработано
        }
    }
}

fn main() -> std::io::Result<()> {
    // Строка меню: одно подменю с двумя пунктами.
    let menu = Menu::new().submenu(
        "~Ф~айл",
        Menu::new()
            .item_key("~П~ривет", CM_HELLO, kb::F2, "F2")
            .separator()
            .item_key("~В~ыход", cm::QUIT, kb::ALT_X, "Alt-X"),
    );
    // Строка состояния: подсказки и «горячие» клавиши.
    let status = vec![StatusDef::all(vec![
        StatusItem::new("~F2~ Привет", kb::F2, CM_HELLO),
        StatusItem::new("~Alt-X~ Выход", kb::ALT_X, cm::QUIT),
        StatusItem::new("", kb::F10, cm::MENU), // невидимый пункт: F10 — меню
    ])];

    let mut app = Program::with_terminal(Some(menu), Some(status))?;
    app.run(&mut App); // главный цикл; выход — по команде cm::QUIT
    Ok(())
}
```

Что здесь происходит:

1. `Program::with_terminal` переводит терминал в полноэкранный режим и
   создаёт три представления: строку меню (верхняя строка), рабочий стол
   (середина) и строку состояния (нижняя строка).
2. `app.run(&mut App)` крутит цикл: перерисовать экран → получить событие →
   доставить его представлениям → если никто не обработал, отдать в
   `App::handle_event`.
3. Пункт меню «Привет» (или F2) посылает команду `CM_HELLO`. Окон нет,
   поэтому команда доходит до обработчика приложения, и тот показывает
   окно сообщения.
4. `cm::QUIT` (Alt+X) завершает `run`. При выходе терминал возвращается в
   обычный режим.

Символ `\x03` в начале строки центрирует её в окне сообщения (так было и в
оригинале).

## Окно с содержимым

Окна вставляются на рабочий стол методом `Program::insert_window`. Внутрь
окна вставляются элементы — координаты элементов отсчитываются от левого
верхнего угла **окна** (рамка занимает строку 0, столбец 0 и последние
строку и столбец).

```rust
use tvision::*;

fn new_window(app: &mut Program, n: i16) {
    let mut w = Window::new(Rect::new(5, 2, 45, 14), &format!("Окно {}", n), n);
    w.group.vs.options |= of::TILEABLE; // участвует в «Мозаике» и «Каскаде»
    // Текст на всю внутреннюю область (без рамки).
    let mut t = StaticText::new(w.interior(), "Окно можно двигать мышью за заголовок.");
    // При изменении размера окна текст растягивается вместе с ним.
    t.vs.grow_mode = gf::GROW_HI_X | gf::GROW_HI_Y;
    w.add(t);
    app.insert_window(w);
}
```

Номер окна (`n`) рисуется в рамке; Alt+цифра делает окно текущим.
F5 распахивает, Ctrl+F5 двигает стрелками, Alt+F3 закрывает — если
в строке состояния или меню есть пункты с этими командами (`cm::ZOOM`,
`cm::RESIZE`, `cm::CLOSE`).

## Диалог

Диалог — окно, которое обычно выполняется **модально**: программа ждёт,
пока пользователь не нажмёт OK или Отмену.

```rust
use tvision::*;

fn ask_name(app: &mut Program) -> Option<String> {
    let mut d = Dialog::centered(44, 9, "Знакомство");
    let name = d.add(InputLine::new(Rect::new(3, 3, 41, 4), 40));
    // Метка вставляется ПОСЛЕ элемента: ей нужен его id.
    d.add(Label::new(Rect::new(2, 2, 20, 3), "~К~ак вас зовут?", Some(name)));
    d.add(Button::new(Rect::new(9, 5, 21, 7), "O~K~", cm::OK, bf::DEFAULT));
    d.add(Button::new(Rect::new(23, 5, 35, 7), "Отмена", cm::CANCEL, bf::NORMAL));

    let (cmd, d) = app.exec_dialog(d); // модальный запуск, диалог возвращается
    if cmd == cm::OK {
        d.win.group.find::<InputLine>(name).map(|il| il.text())
    } else {
        None
    }
}
```

`exec_dialog` возвращает код завершения (`cm::OK`, `cm::CANCEL`, …) и сам
диалог — из него можно прочитать поля. Удобнее читать все поля разом через
`get_data()` — см. [главу 7](07-dialogs.md#обмен-данными).

Для простого вопроса «введите строку» есть готовая функция:

```rust
use tvision::*;

fn ask(app: &mut Program) {
    if let Some(s) = input_box(app, "Поиск", "~Ч~то искать:", "", 60) {
        message_box(app, &format!("Ищем «{}»", s), mf::INFORMATION | mf::OK_BUTTON);
    }
}
```

## Примеры из комплекта

| Команда | Что показывает |
|---------|----------------|
| `cargo run --example hello` | порт `hello.cpp`: меню и диалог |
| `cargo run --example demo` | окна, список, диалог со всеми элементами и обменом данными, дерево каталогов, цвета, контекстное меню, справка по F1 |
| `cargo run --example tvedit` | многооконный редактор файлов: открыть/сохранить, поиск и замена, буфер обмена, смена каталога |
| `cargo run --example tvdemo` | порт `tvdemo`: калькулятор, календарь, пятнашки, таблица ASCII, просмотр файлов, просмотр событий, часы, настройки мыши и цветов, сохранение рабочего стола |
| `cargo run --example table` | таблица: миллион строк × тысяча столбцов, своя модель с вычисляемым столбцом, правка и сортировка, выбор строки в диалоге |
| `cargo run --example colors` | 256 цветов: таблица палитры xterm, темы gruvbox / классическая / «Сумерки», диалог цветов с выбором из 256, переключение режима 16/256 |
| `cargo run --example lfm` | двухпанельный файловый менеджер по мотивам lfm: вкладки, фильтры, закладки, поиск, копирование в потоке с прогрессом, PowerCLI |
| `cargo run --example tvhc -- examples/demohelp.txt hc.rs` | проверка исходника справки и генерация констант контекстов |

Тесты запускаются так:

```text
cargo test                # библиотека и тесты на «невидимом» терминале
cargo test --all-targets  # плюс тесты примеров (tvdemo, table, colors)
```

## Как читать дальше

* Чтобы понять, *как всё устроено* (кто получает события, почему
  представление «не знает» своего окна), — [глава 2](02-architecture.md).
* Чтобы написать свой элемент — [глава 4](04-views.md).
* Чтобы собрать диалог и получить из него данные — [глава 7](07-dialogs.md).
