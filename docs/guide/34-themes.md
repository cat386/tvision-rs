# Справочник: темы

Модуль `tvision::theme`: палитра приложения в файле `.tvtheme`. Урок —
[15. Темы](15-themes.md), редактор — `cargo run -p tvtheme`, пример —
`cargo run --example themes`.

На этой странице:
[`Theme`](#theme--тема) ·
[`Base`](#base--основа) ·
[подключение](#подключение-в-программе) ·
[`choose_theme`, `find_themes`](#choose_theme-и-find_themes) ·
[цвет текстом](#цвет-текстом-color-attr_to_text-attr_from_text) ·
[`color_groups`](#color_groups--все-цвета-по-группам) ·
[`DrawCtx::sub_themed`](#drawctxsub_themed--нарисовать-в-цветах-темы)

---

## `Theme` — тема

**Что это.** Имя, описание, основа и полная палитра приложения (135
атрибутов).

```rust
use tvision::theme::{attr_from_text, Base, Theme};

fn make_theme() {
    let mut t = Theme::builtin(Base::Classic); // начать с классической
    t.name = "Моя".into();
    t.set(1, attr_from_text("cyan/blue").unwrap()); // рабочий стол
    assert_eq!(t.changed(), vec![1]); // отличия от основы
    let text = t.to_text(); // файл .tvtheme
    assert_eq!(Theme::parse(&text).unwrap(), t);
    let code = t.to_rust("MY_THEME"); // или константа Rust
    assert!(code.starts_with("/// Тема «Моя»"));
}
```

| Метод / поле | Что |
|--------------|-----|
| `Theme::builtin(основа)` | тема, равная встроенной палитре |
| `Theme::from_palette(имя, основа, &палитра)` | тема из готовой палитры (например, `app.palette`) |
| `Theme::load(путь)`, `save(путь)` | файл |
| `Theme::parse(текст)`, `to_text()`, `from_json`, `to_json` | текст файла |
| `get(номер)`, `set(номер, цвет)` | цвет палитры (номера с 1) |
| `changed()` | номера цветов, отличающихся от основы |
| `to_rust(имя_константы)` | Rust-код с палитрой |
| поля `name`, `description`, `base`, `palette` | |

Константы модуля: `PALETTE_LEN` (135), `FORMAT_VERSION`, `THEME_ENV`
(`"TVISION_THEME"`).

## `Base` — основа

Встроенная палитра, с которой сравнивается тема: `Classic` (16 цветов
Turbo Vision), `Gruvbox` (256 цветов, по умолчанию в tvision-rs),
`BlackWhite`, `Monochrome`. Методы: `palette()`, `key()` (`"classic"`…),
`title()` (по-русски), `Base::from_key`, `Base::ALL`.

## Подключение в программе

| Метод | Что делает |
|-------|-----------|
| `app.load_theme(путь)` | прочитать `.tvtheme` и подключить; ошибка — текстом |
| `app.set_theme(&тема)` | подключить тему |
| `app.palette` | сама палитра приложения (можно менять напрямую) |

`Program::with_terminal` сам подключает тему из переменной окружения
`TVISION_THEME` (путь к файлу); если файла нет или он испорчен, программа
остаётся в своих цветах.

## `choose_theme` и `find_themes`

```rust,ignore
let (found, errors) = tvision::theme::find_themes("themes");
let themes: Vec<_> = found.into_iter().map(|(_, t)| t).collect();
if let Some(i) = tvision::theme::choose_theme(app, &themes) {
    // themes[i] подключена
}
```

* `find_themes(каталог) -> (Vec<(путь, Theme)>, Vec<ошибка>)` — все
  `*.tvtheme` каталога по алфавиту; нечитаемые — во втором списке.
* `choose_theme(app, &темы) -> Option<номер>` — диалог со списком тем и
  описанием; пока курсор ходит по списку, программа перекрашивается. OK
  — тема остаётся, Отмена — прежние цвета.

## Цвет текстом: `Color`, `attr_to_text`, `attr_from_text`

В файле цвет записан как `текст/фон`.

```rust
use tvision::color::{attr256, fg256};
use tvision::theme::{attr_from_text, attr_to_text, Color};

fn colors() {
    assert_eq!(attr_from_text("white/blue").unwrap(), 0x1F);
    assert_eq!(attr_to_text(0x1F), "white/blue");
    let a = attr_from_text("214/#282828").unwrap(); // номер xterm и #rrggbb
    assert_eq!(fg256(a), 214);
    assert_eq!(attr_to_text(attr256(221, 24)), "221/24");
    assert_eq!(Color::parse("lightblue").unwrap(), Color::Bios(9));
}
```

| Что | Описание |
|-----|---------|
| `Color` | `Bios(0–15)` или `Xterm(0–255)`; `parse`, `to_text`, `fg_of(attr)`, `bg_of(attr)` |
| `make_attr(текст, фон)` | атрибут из двух `Color` |
| `attr_to_text`, `attr_from_text` | атрибут ↔ `текст/фон` |
| `COLOR_NAMES` | имена цветов BIOS: `black` … `white` |
| `COLOR_TITLES` | они же по-русски |

## `color_groups` — все цвета по группам

`color_groups() -> Vec<ColorGroup>` — все цвета палитры по группам с
русскими названиями (их показывают редактор тем и годится для
`ColorDialog`): рабочий стол, меню и строка состояния, синее, бирюзовое
и серое окна, серый, синий и бирюзовый диалоги, окно справки.
`item_name(номер)` — название цвета: `"Серый диалог: кнопка по
умолчанию"`.

## `DrawCtx::sub_themed` — нарисовать в цветах темы

`dc.sub_themed(границы, палитра_представления, &палитра_приложения)` —
контекст рисования, в котором представление видит другую палитру
приложения. Так предпросмотр редактора тем рисует настоящие окна и
диалоги в цветах темы, не меняя цветов самого редактора:

```rust,ignore
fn draw(&self, dc: &mut DrawCtx) {
    let theme = self.theme.borrow();
    for v in &self.samples {
        let mut sub = dc.sub_themed(v.vs().bounds(), v.palette(), &theme.palette);
        v.draw(&mut sub);
    }
}
```

Родственный `sub_root(границы, палитра)` — то же с палитрой самой
программы.
