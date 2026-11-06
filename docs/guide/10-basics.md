# Справочник: геометрия, цвета, буферы, текст

Базовые типы, на которых стоит всё остальное. Подключаются вместе с
`use tvision::*;`; функции цвета — из модуля `tvision::color`, текста — из
`tvision::text`.

* [`Point`](#point) · [`Rect`](#rect)
* [`Attr`, `AttrPair` и функции цвета](#attr-и-attrpair) · [`ColorMode`](#colormode) · [`palettes`](#палитры-colorpalettes)
* [`Cell`](#cell) · [`DrawBuffer`](#drawbuffer) · [`ScreenBuffer`](#screenbuffer)
* [Функции текста](#функции-текста-text)

---

## `Point`

**Точка или размер** в ячейках экрана: `x` — столбец, `y` — строка (ось
Y направлена вниз). Аналог `TPoint`. Модуль `geometry`.

```rust
use tvision::*;

fn points() {
    let a = Point::new(10, 5);
    let b = Point::new(2, 1);
    assert_eq!(a + b, Point::new(12, 6));
    assert_eq!(a - b, Point::new(8, 4));
    let mut c = a;
    c += b; // есть и `-=`
    assert_eq!(c.x, 12);
}
```

| Поле / метод | Что |
|--------------|-----|
| `x`, `y` (`i32`) | координаты |
| `Point::new(x, y)` | создать (`const fn`) |
| `+`, `-`, `+=`, `-=` | сложение и вычитание по координатам |

`Point` используется и как **размер** (`ViewState::size`, `Rect::size()`):
тогда `x` — ширина, `y` — высота.

## `Rect`

**Прямоугольник**: `a` — левый верхний угол (включительно), `b` — правый
нижний (**не включительно**). Ширина — `b.x - a.x`. Аналог `TRect`.

```rust
use tvision::*;

fn rects() {
    let r = Rect::new(2, 1, 12, 4); // столбцы 2..11, строки 1..3
    assert_eq!((r.width(), r.height()), (10, 3));
    assert!(r.contains(Point::new(11, 3)));
    assert!(!r.contains(Point::new(12, 3))); // правая граница не входит

    let inner = r.grown(-1, -1); // сжать на 1 с каждой стороны: (3,2)-(11,3)
    let moved = r.moved(5, 0); // (7,1)-(17,4)
    assert_eq!(r.intersect(&moved), Rect::new(7, 1, 12, 4));
    assert_eq!(r.union(&moved), Rect::new(2, 1, 17, 4));
    assert!(!inner.is_empty());

    let full = Rect::from_origin_size(Point::new(0, 0), Point::new(80, 25));
    assert_eq!(full.size(), Point::new(80, 25));
}
```

| Метод | Что делает |
|-------|-----------|
| `Rect::new(ax, ay, bx, by)` | по углам |
| `Rect::from_origin_size(origin, size)` | по началу и размеру |
| `a`, `b` | углы (поля) |
| `width()`, `height()`, `size()` | размеры |
| `moved(dx, dy)` | сдвинутая копия (`TRect::move`) |
| `grown(dx, dy)` | расширенная на `dx`/`dy` с каждой стороны (отрицательные — сжать) |
| `intersect(&r)`, `union(&r)` | пересечение, охватывающий прямоугольник |
| `contains(p)` | лежит ли точка внутри |
| `is_empty()` | нулевая или отрицательная площадь |

Как прямоугольники складываются в раскладку окон и диалогов, в каких
системах координат задаются и как переводятся — тема
[«Координаты»](02-coordinates.md).

`moved` и `grown` принимают `self` по значению и возвращают новый
прямоугольник — их удобно сцеплять: `r.grown(-1, -1).moved(0, 1)`.

---

## `Attr` и `AttrPair`

`Attr` (`u32`) — **цвет ячейки**. Младший байт — как в BIOS: младшие 4
бита — цвет текста, старшие 4 — фон (`0x1E` — жёлтый на синем). Выше
лежат необязательные номера цветов xterm-256 и флаги `FG_256` / `BG_256`.

`AttrPair` (`u64`) — **пара** атрибутов для текста с горячей буквой:
обычный цвет и цвет букв между тильдами.

| Номер | Цвет BIOS | Номер | Цвет BIOS |
|-------|-----------|-------|-----------|
| 0 | чёрный | 8 | тёмно-серый |
| 1 | синий | 9 | ярко-синий |
| 2 | зелёный | 10 | ярко-зелёный |
| 3 | бирюзовый | 11 | ярко-бирюзовый |
| 4 | красный | 12 | ярко-красный |
| 5 | малиновый | 13 | ярко-малиновый |
| 6 | коричневый | 14 | жёлтый |
| 7 | светло-серый | 15 | белый |

```rust
use tvision::color::*;

fn colors() {
    // 16 цветов BIOS.
    let a = attr(14, 1);
    assert_eq!(a, 0x1E);
    assert_eq!((fore(a), back(a)), (14, 1));
    assert_eq!(with_back(a, 4), 0x4E); // тот же текст на красном

    // 256 цветов xterm.
    let b = attr256(208, 236); // оранжевый на тёмно-сером
    assert!(is_fg256(b) && is_bg256(b));
    assert_eq!((fg256(b), bg256(b)), (208, 236));
    let c = rgb256(0xFFD700, 0x303030); // ближайшие к RGB
    assert_eq!(fg256(c), 220);
    assert_eq!(to_bios(c) & !0xFF, 0); // свести к 16 цветам

    // Пара для C-строк.
    let p = pair(0x70, 0x74);
    assert_eq!((lo(p), hi(p)), (0x70, 0x74));
}
```

### Функции цвета

| Функция | Что делает |
|---------|-----------|
| `attr(fg, bg)` | атрибут BIOS из цветов 0–15 |
| `attr256(fg, bg)` | атрибут из номеров xterm 0–255 |
| `rgb256(fg_rgb, bg_rgb)` | из цветов `0xRRGGBB` (ближайшие xterm) |
| `with_fore(a, c)`, `with_back(a, c)` | заменить текст / фон цветом BIOS |
| `with_fg256(a, n)`, `with_bg256(a, n)` | заменить текст / фон номером xterm |
| `fore(a)`, `back(a)` | цвет BIOS текста / фона (для xterm — ближайший) |
| `fg256(a)`, `bg256(a)` | номер xterm (у цвета BIOS — 0–15 в порядке ANSI) |
| `is_fg256(a)`, `is_bg256(a)` | задан ли цвет номером xterm |
| `bios(a)` | байт BIOS — то, что покажет 16-цветный терминал |
| `to_bios(a)` | атрибут без цветов xterm |
| `xterm_rgb(n)`, `rgb_to_xterm(rgb)`, `xterm_to_bios(n)` | переводы между xterm, RGB и BIOS |
| `bios_to_ansi(c)`, `ansi_to_bios(c)` | порядок цветов BIOS ↔ ANSI (у BIOS 1 — синий, у ANSI — красный) |
| `fg_rgb(a, &base)`, `bg_rgb(a, &base)` | цвет в RGB; `base` — RGB 16 цветов BIOS (`VGA_RGB` или `GRUVBOX_RGB`) |
| `contrast256(n)` | чёрный (16) или белый (231) — читаемый на фоне `n` |
| `shade_bg(a)` | чуть другой фон (для «зебры» таблиц) |
| `remap(palette, &map)` | перекрасить палитру: BIOS `i` → xterm `map[i]` |
| `pair(normal, hi)`, `unpair(p)`, `lo(p)`, `hi(p)` | собрать / разобрать пару |

| Константа | Что |
|-----------|-----|
| `FG_256`, `BG_256` | флаги «цвет задан номером xterm» (биты 24, 25) |
| `ERROR_ATTR` | `0xCF` — ошибка палитры (номер цвета вне палитры) |
| `SHADOW_ATTR`, `SHADOW_ATTR_256` | цвет тени окон |
| `VGA_RGB`, `GRUVBOX_RGB` | RGB 16 цветов BIOS: классика VGA и тема gruvbox |

Подробно о том, как атрибут хранит цвета, — [глава 11](../11-colors.md).

## `ColorMode`

**Сколько цветов выводить в терминал.** `ColorMode::Ansi16` — 16 цветов
(цвета xterm заменяются ближайшими BIOS); `ColorMode::Xterm256` —
256 цветов.

```rust
use tvision::color::{color_mode, set_color_mode, ColorMode};

fn force_16() {
    set_color_mode(Some(ColorMode::Ansi16)); // None — снова определять по терминалу
    assert_eq!(color_mode(), ColorMode::Ansi16);
    set_color_mode(None);
}
```

| Функция | Что |
|---------|-----|
| `color_mode()` | текущий режим: заданный или определённый |
| `set_color_mode(Some(m) / None)` | задать для всей программы / вернуть автоопределение |
| `detect_color_mode()` | определить по `COLORTERM`, `TERM`, Windows Terminal… |
| `color_mode_from(forced, colorterm, term, wt, term_program)` | то же без чтения окружения (для тестов) |

Переменная окружения `TVISION_COLORS=16` или `256` задаёт режим без
изменения программы.

## Палитры (`color::palettes`)

**Палитра** — цепочка номеров цветов. У каждого представления своя
палитра: номер цвета ищется в ней, результат — в палитре владельца и так
до **палитры приложения**, которая даёт настоящий атрибут. Так одна и та
же кнопка выглядит по-разному в сером и синем диалоге.

| Константа | Что |
|-----------|-----|
| `GRUVBOX` | палитра приложения по умолчанию (256 цветов, тема gruvbox) |
| `APP_COLOR`, `APP_BLACK_WHITE`, `APP_MONOCHROME` | классические палитры приложения (`cpAppColor`…) |
| `APP_COLOR_BYTES`… | они же байтами, как в C++ |
| `gruvbox::*` | номера цветов gruvbox в палитре xterm: `BG0`, `FG1`, `RED`, `AQUA`… |
| `BLUE_WINDOW`, `CYAN_WINDOW`, `GRAY_WINDOW` | палитры окон |
| `GRAY_DIALOG`, `BLUE_DIALOG`, `CYAN_DIALOG` | палитры диалогов |
| `BUTTON`, `CLUSTER`, `INPUT_LINE`, `LABEL`, `STATIC_TEXT`, `LIST_VIEWER`, `SCROLL_BAR`, `FRAME`, `MENU_VIEW`, `STATUS_LINE`, `BACKGROUND`, `EDITOR`, `MEMO`, `HELP_VIEWER`, … | палитры элементов (`cpButton`…) |

```rust
use tvision::*;

fn classic(app: &mut Program) {
    // Классические цвета Borland вместо gruvbox.
    app.palette = color::palettes::APP_COLOR.to_vec();
}
```

Палитра приложения — обычный `Vec<Attr>` в `Program::palette`: её можно
менять на ходу (это делает `ColorDialog`) и хранить в файле. Подробно —
[глава 11](../11-colors.md).

---

## `Cell`

**Ячейка экрана**: символ и атрибут.

| Поле / метод | Что |
|--------------|-----|
| `ch: char` | символ; у второй половины широкого символа — `'\0'` |
| `attr: Attr` | цвет |
| `Cell::new(ch, attr)` | создать |
| `is_wide_tail()` | вторая половина иероглифа или эмодзи |

## `DrawBuffer`

**Строка ячеек** для рисования: заполняется методами `move_*` и выводится
`DrawCtx::write_line`. Аналог `TDrawBuffer`. Широкие символы (занимающие
две ячейки) учитываются сами.

```rust
use tvision::*;

/// Строка «[ OK ]  Сохранить» с жёлтой буквой «С».
fn line(width: usize) -> DrawBuffer {
    let mut b = DrawBuffer::new(width); // пробелы
    b.move_char(0, ' ', 0x70, width); // залить серым
    let x = b.move_str(0, "[ OK ]", 0x2F); // зелёный; вернёт ширину = 6
    b.move_cstr(x + 2, "~С~охранить", color::pair(0x70, 0x7E)); // текст между ~ — цветом подсветки
    b.put_attribute(0, 0x4F); // перекрасить одну ячейку
    b.put_char(1, '*'); // заменить символ, не трогая цвет
    b
}
```

| Метод | Что делает |
|-------|-----------|
| `DrawBuffer::new(w)` | `w` пробелов |
| `move_char(x, ch, attr, count)` | повторить символ (`moveChar`) |
| `move_str(x, s, attr) -> ширина` | строка (`moveStr`) |
| `move_str_limited(x, s, attr, max) -> ширина` | не шире `max` ячеек |
| `move_cstr(x, s, pair) -> ширина` | C-строка: текст между `~` — цветом подсветки (`moveCStr`) |
| `move_buf(x, &cells, attr)` | готовые ячейки; `attr ≠ 0` — перекрасить |
| `put_char(x, ch)`, `put_attribute(x, attr)` | символ без цвета / цвет без символа |
| `len()`, `is_empty()`, `cells` | размер и сами ячейки |

> Атрибут `0` в `move_str`, `move_char` и `move_cstr` означает «не менять
> цвет ячейки»: так можно написать текст поверх уже закрашенного фона.

## `ScreenBuffer`

**Весь экран** — прямоугольный массив ячеек. Программа рисует кадр в него,
а терминал выводит только изменившиеся ячейки. В своём коде он нужен в
основном в тестах.

```rust
use tvision::*;

fn check_frame(app: &Program) {
    let s: &ScreenBuffer = app.screen();
    let top = s.line_text(0); // строка меню текстом
    let cell = s.get(2, 0); // ячейка (x, y)
    let _ = (top, cell.attr, s.to_text(), s.width, s.height);
}
```

| Метод | Что |
|-------|-----|
| `ScreenBuffer::new(w, h)` | пустой экран |
| `width`, `height`, `cells` | размер и ячейки (построчно) |
| `get(x, y)`, `get_mut(x, y)` | ячейка (за пределами — паника) |
| `clear(attr)` | залить пробелами |
| `line_text(y)`, `to_text()` | строка / весь экран текстом (для тестов) |

---

## Функции текста (`text`)

Ширина строк на экране и «C-строки» — тексты с горячей буквой между
тильдами (`"~F~ile"`).

```rust
use tvision::text::*;

fn texts() {
    assert_eq!(char_width('Ж'), 1);
    assert_eq!(char_width('漢'), 2); // иероглиф — две ячейки
    assert_eq!(str_width("Привет"), 6);
    assert_eq!(cstr_width("~Ф~айл"), 4); // без тильд
    assert_eq!(strip_tildes("~Ф~айл"), "Файл");
    assert_eq!(hotkey("~Ф~айл"), Some('Ф'));
    assert!(same_hotkey('a', 'Ф')); // Alt+A в латинской раскладке = Alt+Ф
    assert_eq!(truncate_width("Привет, мир", 6), "Привет");
    assert_eq!(wrap_text("один два три", 8), vec!["один два", "три"]);
    assert_eq!(to_upper('ё'), 'Ё');
}
```

| Функция | Что делает |
|---------|-----------|
| `char_width(c)` | ширина символа: 0 (комбинирующие, управляющие), 1 или 2 |
| `str_width(s)` | ширина строки (`strwidth`) |
| `cstr_width(s)` | ширина C-строки без тильд (`cstrlen`) |
| `strip_tildes(s)` | убрать тильды |
| `hotkey(s)` | горячая буква в верхнем регистре |
| `same_hotkey(нажата, горячая)` | сравнить с учётом регистра и раскладки (й↔q, ф↔a…) |
| `to_upper(c)` | верхний регистр, в том числе кириллицы |
| `truncate_width(s, max)` | начало строки не шире `max` ячеек |
| `wrap_text(s, w)` | перенос по словам (как у `StaticText`) |
