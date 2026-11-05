# Справочник: проверка ввода

Валидатор прикрепляется к строке ввода (`InputLine::with_validator`) и
проверяет текст по ходу набора и перед закрытием диалога. Модуль
`widgets::validate`.

* [`Validator`](#validator) — трейт
* [`PictureValidator`](#picturevalidator) · [`PicResult`](#picresult)
* [`FilterValidator`](#filtervalidator) · [`RangeValidator`](#rangevalidator) · [`StringLookupValidator`](#stringlookupvalidator) · [`FnValidator`](#fnvalidator)
* [Свой валидатор](#свой-валидатор)

Как работает проверка:

* **по ходу набора** — после каждой правки строка спрашивает
  `is_valid_input`; «нет» — правка откатывается, неправильный символ просто
  не появляется (валидатор может и дописать строку — автозаполнение маски);
* **при закрытии** — командой, отличной от `cm::CANCEL` (обычно OK),
  строка спрашивает `is_valid`; «нет» — окно с `error_message()`, фокус
  возвращается в поле, диалог остаётся открытым;
* `status_ok()` — правилен ли сам валидатор (синтаксис маски); окно с
  неправильной маской `insert_window` не вставит.

```rust
use tvision::*;

fn fields(d: &mut Dialog) {
    d.add(InputLine::new(Rect::new(3, 2, 15, 3), 10).with_validator(PictureValidator::new("##.##.####", true)));
    d.add(InputLine::new(Rect::new(3, 4, 15, 5), 8).with_validator(FilterValidator::new("0123456789ABCDEFabcdef")));
    d.add(InputLine::new(Rect::new(3, 6, 15, 7), 3).with_validator(RangeValidator::new(1, 100)));
    d.add(InputLine::new(Rect::new(3, 8, 15, 9), 10).with_validator(StringLookupValidator::new(&["да", "нет"])));
}
```

---

## `Validator`

**Трейт проверки** (`TValidator`). Обязателен только `error_message`.

| Метод | Когда | По умолчанию |
|-------|-------|--------------|
| `is_valid_input(&self, s: &mut String, suppress_fill) -> bool` | после каждой правки; может изменить `s` | `true` |
| `is_valid(&self, s) -> bool` | при закрытии диалога | `true` |
| `error_message(&self) -> String` | текст ошибки для окна сообщения | — |
| `status_ok(&self) -> bool` | правилен ли сам валидатор | `true` |

`suppress_fill = true` передаётся при удалении символов: не дописывайте
строку, иначе пользователь не сможет стереть литерал маски.

## `PictureValidator`

**Маска ввода** в стиле Paradox (`TPXPictureValidator`).
`PictureValidator::new(маска, auto_fill)`; при `auto_fill = true`
литералы маски дописываются сами.

| Символ маски | Значит |
|--------------|--------|
| `#` | цифра |
| `?` | буква |
| `&` | буква, переводится в верхний регистр |
| `!` | любой символ в верхнем регистре |
| `@` | любой символ |
| `;` | следующий символ — буквально (`;#` — знак «#») |
| `*` | повтор: `*3#` — три цифры, `*#` — сколько угодно |
| `[ ]` | необязательная часть |
| `{ }` | группа |
| `,` | варианты: `{Да,Нет}` |
| остальное | литерал |

```rust
use tvision::*;

fn masks() {
    let date = PictureValidator::new("##.##.####", true);
    assert!(date.status_ok());
    assert!(date.is_valid("05.11.2026"));
    assert!(!date.is_valid("05.11"));      // значение должно соответствовать маске полностью
    assert!(!date.is_valid("ab.11.2026"));

    let yes_no = PictureValidator::new("{Да,Нет}", true);
    assert!(yes_no.is_valid("Да") && !yes_no.is_valid("Может"));

    let broken = PictureValidator::new("[##", false); // нет закрывающей скобки
    assert!(!broken.status_ok());
}
```

| Маска | Принимает |
|-------|-----------|
| `##.##.####` | дату `05.11.2026` |
| `(###) ###-####` | телефон |
| `#[#][#]` | число из 1–3 цифр |
| `&*?` | слово с заглавной буквы (`иван` → `Иван`) |
| `*#[.##]` | число с необязательными двумя знаками после точки |

Пустая строка маску не проходит. Для необязательного поля оберните
проверку в `FnValidator` (`|s| s.is_empty() || …`) или напишите свой
валидатор. Метод `picture(&mut s, auto_fill) -> PicResult` разбирает
строку по маске — пригодится для своих проверок. Текст ошибки —
`validate::PICTURE_ERROR` (переводится, см. [язык](21-platform.md#язык-интерфейса)).

## `PicResult`

Результат разбора маски (`TPicResult`): `Complete` (строка полностью
соответствует), `Incomplete` (можно продолжать набор), `Empty`, `Error`
(не соответствует), `Syntax` (ошибка в самой маске), `Ambiguous`,
`IncompNoFill`.

## `FilterValidator`

**Только символы из набора** (`TFilterValidator`):
`FilterValidator::new("0123456789")`. Набор — поле `valid_chars`. Лишний
символ просто не набирается.

```rust
use tvision::*;

fn filter() {
    let v = FilterValidator::new("0123456789+-() ");
    let mut s = "+7 (900)".to_string();
    assert!(v.is_valid_input(&mut s, false));
    let mut bad = "+7 abc".to_string();
    assert!(!v.is_valid_input(&mut bad, false));
}
```

## `RangeValidator`

**Целое в диапазоне** `min..=max` (`TRangeValidator`):
`RangeValidator::new(1, 100)`. По ходу набора пропускает цифры и `-`
(только при `min < 0`), при закрытии проверяет диапазон. Поля `min`,
`max`. Данные строки остаются строкой (`Value::Str`); для числа в данных
удобнее [`InputLong`](16-controls.md#inputlong).

```rust
use tvision::*;

fn range() {
    let v = RangeValidator::new(1, 100);
    assert!(v.is_valid("42"));
    assert!(!v.is_valid("0") && !v.is_valid("101"));
}
```

## `StringLookupValidator`

**Значение из списка** (`TStringLookupValidator`):
`StringLookupValidator::new(&["да", "нет"])`, поле `strings`. Набор не
ограничивается — проверяется только готовое значение. Для выбора из
списка с подсказкой есть [`Combo`](16-controls.md#combo-и-combowindow) с `cbx::ONLY_LIST`.

## `FnValidator`

**Проверка функцией** — удобство Rust-версии. Поля `f: Fn(&str) -> bool`
и `message`. Проверяет только окончательное значение.

```rust
use tvision::*;

fn email() -> InputLine {
    InputLine::new(Rect::new(3, 2, 40, 3), 60).with_validator(FnValidator {
        f: |s: &str| s.is_empty() || (s.contains('@') && s.contains('.')),
        message: "Введите адрес вида имя@домен.ru".into(),
    })
}
```

## Свой валидатор

```rust
use tvision::*;

/// Логин: латиница и цифры, 3–16 символов, всегда в нижнем регистре.
struct Login;

impl Validator for Login {
    fn is_valid_input(&self, s: &mut String, _suppress_fill: bool) -> bool {
        *s = s.to_lowercase(); // можно исправлять ввод
        s.chars().all(|c| c.is_ascii_alphanumeric()) && s.chars().count() <= 16
    }
    fn is_valid(&self, s: &str) -> bool {
        (3..=16).contains(&s.chars().count())
    }
    fn error_message(&self) -> String {
        "Логин: от 3 до 16 латинских букв и цифр".into()
    }
}

fn login() {
    let mut s = "Ivan".to_string();
    assert!(Login.is_valid_input(&mut s, false));
    assert_eq!(s, "ivan");
    assert!(!Login.is_valid("iv"));
    let _field = InputLine::new(Rect::new(3, 2, 22, 3), 16).with_validator(Login);
}
```
