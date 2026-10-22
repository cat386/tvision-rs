//! Проверка ввода (`TValidator` и наследники из `validate.h` / `tvalidat.cpp`).
//!
//! * [`PictureValidator`] — маска ввода в стиле Paradox (`TPXPictureValidator`),
//!   например `"{##}-{##}-{####}"` для даты или `"(###) ###-####"` для телефона;
//! * [`FilterValidator`] — только разрешённые символы (`TFilterValidator`);
//! * [`RangeValidator`] — целое число в диапазоне (`TRangeValidator`);
//! * [`StringLookupValidator`] — значение из списка (`TStringLookupValidator`).
//!
//! Строка ввода ([`crate::InputLine`]) вызывает `is_valid_input` после каждого
//! изменения (некорректный ввод откатывается) и `is_valid` при закрытии диалога
//! (при ошибке показывается сообщение и фокус возвращается в строку).

/// Проверка ввода.
pub trait Validator {
    /// Синтаксис самого валидатора корректен (`status == vsOk`).
    fn status_ok(&self) -> bool {
        true
    }

    /// Допустим ли ввод «по ходу набора» (`isValidInput`). Может дописать
    /// строку (автозаполнение литералов маски), если `suppress_fill == false`.
    fn is_valid_input(&self, _s: &mut String, _suppress_fill: bool) -> bool {
        true
    }

    /// Допустимо ли окончательное значение (`isValid`).
    fn is_valid(&self, _s: &str) -> bool {
        true
    }

    /// Текст сообщения об ошибке (то, что `error()` показывал в `messageBox`).
    fn error_message(&self) -> String;
}

// ----------------------------------------------------------------------
// Маска ввода (TPXPictureValidator)
// ----------------------------------------------------------------------

/// Результат разбора маски (`TPicResult`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PicResult {
    Complete,
    Incomplete,
    Empty,
    Error,
    Syntax,
    Ambiguous,
    IncompNoFill,
}

fn is_complete(r: PicResult) -> bool {
    r == PicResult::Complete || r == PicResult::Ambiguous
}

fn is_incomplete(r: PicResult) -> bool {
    r == PicResult::Incomplete || r == PicResult::IncompNoFill
}

/// Маска ввода. Символы маски:
/// `#` — цифра, `?` — буква, `&` — буква (в верхний регистр), `!` — любой
/// символ (в верхний регистр), `@` — любой символ, `;` — следующий символ
/// буквально, `*` — повтор (`*3#` — три цифры, `*#` — сколько угодно),
/// `[]` — необязательная часть, `{}` — группа, `,` — варианты.
pub struct PictureValidator {
    pic: Vec<char>,
    auto_fill: bool,
    status_ok: bool,
}

/// Текст ошибки (`TPXPictureValidator::errorMsg`).
pub const PICTURE_ERROR: &str = "Input does not conform to picture:";

/// Состояние разбора: позиции в маске (`index`) и во вводе (`jndex`).
struct PicState<'a> {
    pic: &'a [char],
    index: usize,
    jndex: usize,
    input: Vec<char>,
}

impl<'a> PicState<'a> {
    /// Символ маски; за концом — `'\0'` (как нуль-терминатор в C).
    fn p(&self, i: usize) -> char {
        self.pic.get(i).copied().unwrap_or('\0')
    }

    fn consume(&mut self, ch: char) {
        if self.jndex < self.input.len() {
            self.input[self.jndex] = ch;
        } else {
            self.input.push(ch);
        }
        self.index += 1;
        self.jndex += 1;
    }

    /// Пропустить символ или группу маски (`toGroupEnd`).
    fn to_group_end(&self, i: &mut usize, term: usize) {
        let mut brk = 0i32;
        let mut brc = 0i32;
        loop {
            if *i >= term {
                *i = term;
                return;
            }
            match self.p(*i) {
                '[' => brk += 1,
                ']' => brk -= 1,
                '{' => brc += 1,
                '}' => brc -= 1,
                ';' => *i += 1,
                _ => {}
            }
            *i += 1;
            if brk == 0 && brc == 0 {
                return;
            }
        }
    }

    /// Перейти к следующему варианту после запятой (`skipToComma`).
    fn skip_to_comma(&mut self, term: usize) -> bool {
        loop {
            let mut i = self.index;
            self.to_group_end(&mut i, term);
            self.index = i;
            if self.index >= term || self.p(self.index) == ',' {
                break;
            }
        }
        if self.p(self.index) == ',' && self.index < term {
            self.index += 1;
        }
        self.index < term
    }

    /// Конец текущей группы (`calcTerm`).
    fn calc_term(&self, term: usize) -> usize {
        let mut k = self.index;
        self.to_group_end(&mut k, term);
        k
    }

    /// Повтор `*N` (`iteration`).
    fn iteration(&mut self, in_term: usize) -> PicResult {
        let mut itr = 0usize;
        let mut rslt = PicResult::Error;
        self.index += 1; // пропускаем '*'
        while self.p(self.index).is_ascii_digit() {
            itr = itr * 10 + (self.p(self.index) as usize - '0' as usize);
            self.index += 1;
        }
        let k = self.index;
        let term = self.calc_term(in_term);
        if itr != 0 {
            for _ in 0..itr {
                self.index = k;
                rslt = self.process(term);
                if !is_complete(rslt) {
                    if rslt == PicResult::Empty {
                        rslt = PicResult::Incomplete;
                    }
                    return rslt;
                }
            }
        } else {
            loop {
                self.index = k;
                rslt = self.process(term);
                if rslt != PicResult::Complete {
                    break;
                }
            }
            if rslt == PicResult::Empty || rslt == PicResult::Error {
                self.index += 1;
                rslt = PicResult::Ambiguous;
            }
        }
        self.index = term;
        rslt
    }

    /// Группа `{...}` или `[...]` (`group`).
    fn group(&mut self, in_term: usize) -> PicResult {
        let term = self.calc_term(in_term);
        self.index += 1;
        let rslt = self.process(term.saturating_sub(1));
        if !is_incomplete(rslt) {
            self.index = term;
        }
        rslt
    }

    /// Можно ли считать неполный ввод полным — остались только
    /// необязательные части (`checkComplete`).
    fn check_complete(&self, mut rslt: PicResult, term: usize) -> PicResult {
        let mut j = self.index;
        if is_incomplete(rslt) {
            loop {
                match self.p(j) {
                    '[' => self.to_group_end(&mut j, term),
                    '*' => {
                        if !self.p(j + 1).is_ascii_digit() {
                            j += 1;
                        }
                        self.to_group_end(&mut j, term);
                    }
                    _ => break,
                }
                if j >= term {
                    break;
                }
            }
            if j == term {
                rslt = PicResult::Ambiguous;
            }
        }
        rslt
    }

    /// Сопоставление одного варианта маски с вводом (`scan`).
    fn scan(&mut self, term: usize) -> PicResult {
        let mut rslt = PicResult::Empty;
        while self.index != term && self.p(self.index) != ',' {
            if self.index > term {
                break;
            }
            if self.jndex >= self.input.len() {
                return self.check_complete(rslt, term);
            }
            let ch = self.input[self.jndex];
            match self.p(self.index) {
                '#' => {
                    if !ch.is_ascii_digit() {
                        return PicResult::Error;
                    }
                    self.consume(ch);
                }
                '?' => {
                    if !ch.is_alphabetic() {
                        return PicResult::Error;
                    }
                    self.consume(ch);
                }
                '&' => {
                    if !ch.is_alphabetic() {
                        return PicResult::Error;
                    }
                    self.consume(crate::text::to_upper(ch));
                }
                '!' => self.consume(crate::text::to_upper(ch)),
                '@' => self.consume(ch),
                '*' => {
                    rslt = self.iteration(term);
                    if !is_complete(rslt) {
                        return rslt;
                    }
                    if rslt == PicResult::Error {
                        rslt = PicResult::Ambiguous;
                    }
                }
                '{' => {
                    rslt = self.group(term);
                    if !is_complete(rslt) {
                        return rslt;
                    }
                }
                '[' => {
                    rslt = self.group(term);
                    if is_incomplete(rslt) {
                        return rslt;
                    }
                    if rslt == PicResult::Error {
                        rslt = PicResult::Ambiguous;
                    }
                }
                _ => {
                    if self.p(self.index) == ';' {
                        self.index += 1;
                    }
                    let pc = self.p(self.index);
                    if crate::text::to_upper(pc) != crate::text::to_upper(ch) && ch != ' ' {
                        return PicResult::Error;
                    }
                    // Пробел заменяется литералом маски.
                    self.consume(pc);
                }
            }
            rslt = if rslt == PicResult::Ambiguous { PicResult::IncompNoFill } else { PicResult::Incomplete };
        }
        if rslt == PicResult::IncompNoFill {
            PicResult::Ambiguous
        } else {
            PicResult::Complete
        }
    }

    /// Перебор вариантов (`process`).
    fn process(&mut self, term: usize) -> PicResult {
        let mut incomp = false;
        let mut old_i = self.index;
        let old_j = self.jndex;
        let (mut incomp_i, mut incomp_j) = (0, 0);
        let mut rslt;
        loop {
            rslt = self.scan(term);
            // Полное совпадение принимаем, только если оно дальше неполного.
            if rslt == PicResult::Complete && incomp && self.jndex < incomp_j {
                rslt = PicResult::Incomplete;
                self.jndex = incomp_j;
            }
            if rslt == PicResult::Error || rslt == PicResult::Incomplete {
                let r_process = rslt;
                if !incomp && rslt == PicResult::Incomplete {
                    incomp = true;
                    incomp_i = self.index;
                    incomp_j = self.jndex;
                }
                self.index = old_i;
                self.jndex = old_j;
                if !self.skip_to_comma(term) {
                    if incomp {
                        self.index = incomp_i;
                        self.jndex = incomp_j;
                        return PicResult::Incomplete;
                    }
                    return r_process;
                }
                old_i = self.index;
            } else {
                break;
            }
        }
        if rslt == PicResult::Complete && incomp {
            PicResult::Ambiguous
        } else {
            rslt
        }
    }
}

impl PictureValidator {
    /// Создаёт валидатор по маске. `auto_fill` — дописывать литералы маски
    /// автоматически (например, дефисы в дате).
    pub fn new(pic: &str, auto_fill: bool) -> Self {
        let mut v = PictureValidator { pic: pic.chars().collect(), auto_fill, status_ok: true };
        let mut empty = String::new();
        if v.picture(&mut empty, false) != PicResult::Empty {
            v.status_ok = false;
        }
        v
    }

    /// Проверка скобок маски (`syntaxCheck`).
    fn syntax_check(&self) -> bool {
        if self.pic.is_empty() || *self.pic.last().unwrap() == ';' {
            return false;
        }
        let (mut brk, mut brc) = (0, 0);
        let mut i = 0;
        while i < self.pic.len() {
            match self.pic[i] {
                '[' => brk += 1,
                ']' => brk -= 1,
                '{' => brc += 1,
                '}' => brc -= 1,
                ';' => i += 1,
                _ => {}
            }
            i += 1;
        }
        brk == 0 && brc == 0
    }

    /// Сопоставление строки с маской (`picture`). Может дописать `input`.
    pub fn picture(&self, input: &mut String, auto_fill: bool) -> PicResult {
        if !self.syntax_check() {
            return PicResult::Syntax;
        }
        if input.is_empty() {
            return PicResult::Empty;
        }
        let mut st = PicState { pic: &self.pic, index: 0, jndex: 0, input: input.chars().collect() };
        let len = self.pic.len();
        let mut rslt = st.process(len);
        if rslt != PicResult::Error && st.jndex < st.input.len() {
            rslt = PicResult::Error;
        }
        if rslt == PicResult::Incomplete && auto_fill {
            let mut reprocess = false;
            while st.index < len && !"#?&!@*{}[],".contains(self.pic[st.index]) {
                if self.pic[st.index] == ';' {
                    st.index += 1;
                }
                if st.index < len {
                    st.input.push(self.pic[st.index]);
                }
                st.index += 1;
                reprocess = true;
            }
            st.jndex = 0;
            st.index = 0;
            if reprocess {
                rslt = st.process(len);
            }
        }
        *input = st.input.iter().collect();
        match rslt {
            PicResult::Ambiguous => PicResult::Complete,
            PicResult::IncompNoFill => PicResult::Incomplete,
            r => r,
        }
    }
}

impl Validator for PictureValidator {
    fn status_ok(&self) -> bool {
        self.status_ok
    }

    fn is_valid_input(&self, s: &mut String, suppress_fill: bool) -> bool {
        let fill = self.auto_fill && !suppress_fill;
        self.picture(s, fill) != PicResult::Error
    }

    fn is_valid(&self, s: &str) -> bool {
        let mut t = s.to_string();
        self.picture(&mut t, false) == PicResult::Complete
    }

    fn error_message(&self) -> String {
        format!("{}\n {}", PICTURE_ERROR, self.pic.iter().collect::<String>())
    }
}

// ----------------------------------------------------------------------
// Фильтр, диапазон, список
// ----------------------------------------------------------------------

/// Разрешены только символы из набора (`TFilterValidator`).
pub struct FilterValidator {
    pub valid_chars: String,
}

impl FilterValidator {
    pub fn new(valid_chars: &str) -> Self {
        FilterValidator { valid_chars: valid_chars.into() }
    }

    fn check(&self, s: &str) -> bool {
        s.chars().all(|c| self.valid_chars.contains(c))
    }
}

impl Validator for FilterValidator {
    fn is_valid_input(&self, s: &mut String, _suppress_fill: bool) -> bool {
        self.check(s)
    }
    fn is_valid(&self, s: &str) -> bool {
        self.check(s)
    }
    fn error_message(&self) -> String {
        "Invalid character in input".into()
    }
}

/// Целое число в диапазоне `min..=max` (`TRangeValidator`).
pub struct RangeValidator {
    pub min: i64,
    pub max: i64,
    filter: FilterValidator,
}

impl RangeValidator {
    pub fn new(min: i64, max: i64) -> Self {
        let chars = if min >= 0 { "+0123456789" } else { "+-0123456789" };
        RangeValidator { min, max, filter: FilterValidator::new(chars) }
    }
}

impl Validator for RangeValidator {
    fn is_valid_input(&self, s: &mut String, f: bool) -> bool {
        self.filter.is_valid_input(s, f)
    }
    fn is_valid(&self, s: &str) -> bool {
        if !self.filter.is_valid(s) {
            return false;
        }
        match s.trim().parse::<i64>() {
            Ok(v) => v >= self.min && v <= self.max,
            Err(_) => false,
        }
    }
    fn error_message(&self) -> String {
        format!("Value not in the range {} to {}", self.min, self.max)
    }
}

/// Значение должно быть одним из строк списка (`TStringLookupValidator`).
pub struct StringLookupValidator {
    pub strings: Vec<String>,
}

impl StringLookupValidator {
    pub fn new(strings: &[&str]) -> Self {
        StringLookupValidator { strings: strings.iter().map(|s| s.to_string()).collect() }
    }
}

impl Validator for StringLookupValidator {
    fn is_valid(&self, s: &str) -> bool {
        self.strings.iter().any(|x| x == s)
    }
    fn error_message(&self) -> String {
        "Input is not in list of valid strings".into()
    }
}

/// Валидатор на основе функции (удобство Rust-версии).
pub struct FnValidator<F: Fn(&str) -> bool> {
    pub f: F,
    pub message: String,
}

impl<F: Fn(&str) -> bool> Validator for FnValidator<F> {
    fn is_valid(&self, s: &str) -> bool {
        (self.f)(s)
    }
    fn error_message(&self) -> String {
        self.message.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pic(p: &str, s: &str) -> (PicResult, String) {
        let v = PictureValidator::new(p, true);
        let mut t = s.to_string();
        let r = v.picture(&mut t, true);
        (r, t)
    }

    #[test]
    fn picture_basic() {
        assert_eq!(pic("###", "12").0, PicResult::Incomplete);
        assert_eq!(pic("###", "123").0, PicResult::Complete);
        assert_eq!(pic("###", "12a").0, PicResult::Error);
        assert_eq!(pic("###", "1234").0, PicResult::Error);
    }

    #[test]
    fn picture_autofill() {
        // Дата: после двух цифр дописывается дефис.
        let (r, t) = pic("##-##-####", "12");
        assert_eq!(r, PicResult::Incomplete);
        assert_eq!(t, "12-");
        let (r, _) = pic("##-##-####", "12-03-2024");
        assert_eq!(r, PicResult::Complete);
    }

    #[test]
    fn picture_optional_and_alternatives() {
        assert_eq!(pic("#[#]", "1").0, PicResult::Complete);
        assert_eq!(pic("#[#]", "12").0, PicResult::Complete);
        assert_eq!(pic("{Yes,No}", "No").0, PicResult::Complete);
        // С автозаполнением «Ye» дописывается до «Yes», без него — неполный ввод.
        assert_eq!(pic("{Yes,No}", "Ye"), (PicResult::Complete, "Yes".to_string()));
        let mut t = "Ye".to_string();
        assert_eq!(PictureValidator::new("{Yes,No}", false).picture(&mut t, false), PicResult::Incomplete);
        assert_eq!(pic("*#", "12345").0, PicResult::Complete);
        assert_eq!(pic("&*?", "ivan").1, "Ivan");
    }

    #[test]
    fn picture_syntax() {
        assert!(!PictureValidator::new("{##", false).status_ok());
        assert!(PictureValidator::new("{##}", false).status_ok());
    }

    #[test]
    fn range() {
        let r = RangeValidator::new(1, 100);
        assert!(r.is_valid("50"));
        assert!(!r.is_valid("500"));
        assert!(!r.is_valid("-5"));
        let mut s = "1a".to_string();
        assert!(!r.is_valid_input(&mut s, false));
    }
}
