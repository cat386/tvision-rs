//! Таблица (`Table`) — просмотр и правка данных в строках и столбцах.
//!
//! В оригинальном Turbo Vision таблицы не было; этот элемент написан для
//! tvision-rs в том же стиле, что `TListViewer`.
//!
//! Данные таблица берёт у **модели** — типа с трейтом [`TableModel`]. Модель
//! спрашивают только о видимых ячейках, поэтому таблица одинаково быстро
//! показывает и десять строк, и миллионы (например, строки, которые
//! вычисляются на лету или читаются из файла по мере прокрутки).
//!
//! Готовые модели:
//! * [`VecTableModel`] — данные в памяти (`Vec<Vec<String>>`): правка и
//!   сортировка уже реализованы;
//! * [`FnTableModel`] — «виртуальная» модель: число строк и столбцов плюс
//!   функция, возвращающая текст ячейки.
//!
//! Что настраивается:
//! * столбцы ([`TableColumn`]): заголовок, ширина и её пределы,
//!   выравнивание, видимость, можно ли менять ширину и сортировать;
//! * таблица: строка заголовков, линии сетки, номера строк, закреплённые
//!   слева столбцы, выбор ячейки или целой строки, выделение нескольких
//!   строк, правка ячеек, сортировка щелчком по заголовку, изменение
//!   ширины мышью и клавишами, «зебра», свои цвета ([`TableColors`]).
//!
//! Клавиши: стрелки, PgUp/PgDn, Home/End (первый/последний столбец),
//! Ctrl+Home/Ctrl+End или Ctrl+PgUp/Ctrl+PgDn (первая/последняя строка),
//! Shift+↑/↓ и пробел — выделение строк (при `multi_select`), Ctrl+A —
//! выделить всё, Ctrl+←/→ — ширина текущего столбца, Ctrl+C / Ctrl+Ins —
//! копировать (ячейку или выделенные строки, через табуляцию), F2 или
//! любая буква — правка ячейки (Enter — сохранить, Esc — отменить),
//! Enter — «выбрать» (`cm::LIST_ITEM_SELECTED`; клавиша идёт дальше, и в
//! диалоге срабатывает кнопка по умолчанию).
//!
//! Мышь: щелчок — ячейка, щелчок по заголовку — сортировка, протяжка
//! границы заголовка — ширина столбца, двойной щелчок — выбрать или
//! править, колесо — прокрутка (с Shift — по столбцам).
//!
//! Сообщения владельцу (`Broadcast`, `Info::Id(id таблицы)`):
//! `cm::TABLE_CURSOR_MOVED`, `cm::TABLE_CELL_EDITED`,
//! `cm::LIST_ITEM_SELECTED`. Данные таблицы (`get_data`) — номер строки
//! с курсором (`Value::Int`), как у списка.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use crate::color::Attr;
use crate::drawbuf::DrawBuffer;
use crate::event::{cm, Event, Info, Key};
use crate::geometry::{Point, Rect};
use crate::text::{char_width, str_width};
use crate::view::{default_handle_event, of, sf, Ctx, DrawCtx, Value, View, ViewId, ViewState};

// ----------------------------------------------------------------------
// Модель данных
// ----------------------------------------------------------------------

/// Источник данных таблицы.
///
/// Обязательны только `row_count`, `col_count` и `cell`; остальное —
/// по желанию (заголовки, правка, сортировка).
pub trait TableModel {
    /// Число строк.
    fn row_count(&self) -> usize;
    /// Число столбцов.
    fn col_count(&self) -> usize;
    /// Текст ячейки. Вызывается только для видимых ячеек.
    fn cell(&self, row: usize, col: usize) -> String;
    /// Заголовок столбца (если у [`TableColumn`] не задан свой).
    /// По умолчанию — буквы, как в электронных таблицах: A, B, …, Z, AA…
    fn header(&self, col: usize) -> String {
        column_letters(col)
    }
    /// Можно ли править ячейку.
    fn editable(&self, _row: usize, _col: usize) -> bool {
        false
    }
    /// Записать новое значение. `false` — значение не принято.
    fn set_cell(&mut self, _row: usize, _col: usize, _value: &str) -> bool {
        false
    }
    /// Отсортировать строки по столбцу. `false` — сортировка не поддерживается.
    fn sort(&mut self, _col: usize, _ascending: bool) -> bool {
        false
    }
}

/// Имя столбца в стиле электронных таблиц: 0 → «A», 25 → «Z», 26 → «AA».
pub fn column_letters(mut col: usize) -> String {
    let mut s = Vec::new();
    loop {
        s.push((b'A' + (col % 26) as u8) as char);
        if col < 26 {
            break;
        }
        col = col / 26 - 1;
    }
    s.iter().rev().collect()
}

/// Сравнение строк «по-человечески»: если обе — числа, сравниваются как
/// числа, иначе — как текст без учёта регистра.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let na = a.trim().replace(',', ".").parse::<f64>();
    let nb = b.trim().replace(',', ".").parse::<f64>();
    match (na, nb) {
        (Ok(x), Ok(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
        _ => a.to_lowercase().cmp(&b.to_lowercase()),
    }
}

/// Модель «данные в памяти»: строки — `Vec<String>`.
#[derive(Clone, Debug, Default)]
pub struct VecTableModel {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    /// Разрешена ли правка ячеек.
    pub editable: bool,
}

impl VecTableModel {
    /// Модель с заголовками и строками.
    pub fn new(headers: &[&str], rows: Vec<Vec<String>>) -> Self {
        VecTableModel { headers: headers.iter().map(|s| s.to_string()).collect(), rows, editable: false }
    }

    /// Разрешить правку.
    pub fn editable(mut self, on: bool) -> Self {
        self.editable = on;
        self
    }
}

impl TableModel for VecTableModel {
    fn row_count(&self) -> usize {
        self.rows.len()
    }
    fn col_count(&self) -> usize {
        self.headers.len().max(self.rows.iter().map(|r| r.len()).max().unwrap_or(0))
    }
    fn cell(&self, row: usize, col: usize) -> String {
        self.rows.get(row).and_then(|r| r.get(col)).cloned().unwrap_or_default()
    }
    fn header(&self, col: usize) -> String {
        self.headers.get(col).cloned().unwrap_or_else(|| column_letters(col))
    }
    fn editable(&self, _row: usize, _col: usize) -> bool {
        self.editable
    }
    fn set_cell(&mut self, row: usize, col: usize, value: &str) -> bool {
        match self.rows.get_mut(row) {
            Some(r) => {
                if r.len() <= col {
                    r.resize(col + 1, String::new());
                }
                r[col] = value.to_string();
                true
            }
            None => false,
        }
    }
    fn sort(&mut self, col: usize, ascending: bool) -> bool {
        // Устойчивая сортировка: равные строки сохраняют порядок.
        self.rows.sort_by(|a, b| {
            let o = natural_cmp(a.get(col).map(|s| s.as_str()).unwrap_or(""), b.get(col).map(|s| s.as_str()).unwrap_or(""));
            if ascending {
                o
            } else {
                o.reverse()
            }
        });
        true
    }
}

/// «Виртуальная» модель: ячейки вычисляются функцией по запросу.
/// Подходит для очень больших таблиц — данные не хранятся в памяти.
pub struct FnTableModel<F: Fn(usize, usize) -> String> {
    pub rows: usize,
    pub cols: usize,
    pub headers: Vec<String>,
    pub f: F,
}

impl<F: Fn(usize, usize) -> String> FnTableModel<F> {
    pub fn new(rows: usize, cols: usize, f: F) -> Self {
        FnTableModel { rows, cols, headers: Vec::new(), f }
    }
    /// Задать заголовки столбцов.
    pub fn headers(mut self, h: &[&str]) -> Self {
        self.headers = h.iter().map(|s| s.to_string()).collect();
        self
    }
}

impl<F: Fn(usize, usize) -> String> TableModel for FnTableModel<F> {
    fn row_count(&self) -> usize {
        self.rows
    }
    fn col_count(&self) -> usize {
        self.cols
    }
    fn cell(&self, row: usize, col: usize) -> String {
        (self.f)(row, col)
    }
    fn header(&self, col: usize) -> String {
        self.headers.get(col).cloned().unwrap_or_else(|| column_letters(col))
    }
}

// ----------------------------------------------------------------------
// Настройки
// ----------------------------------------------------------------------

/// Выравнивание текста в столбце.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

/// Описание столбца.
#[derive(Clone, Debug, PartialEq)]
pub struct TableColumn {
    /// Свой заголовок (`None` — заголовок из модели).
    pub title: Option<String>,
    /// Ширина в ячейках экрана.
    pub width: i32,
    pub min_width: i32,
    pub max_width: i32,
    pub align: Align,
    /// Скрытый столбец не показывается и пропускается курсором.
    pub visible: bool,
    /// Можно менять ширину (мышью или Ctrl+←/→).
    pub resizable: bool,
    /// Можно сортировать щелчком по заголовку.
    pub sortable: bool,
}

impl Default for TableColumn {
    fn default() -> Self {
        TableColumn { title: None, width: 12, min_width: 1, max_width: 200, align: Align::Left, visible: true, resizable: true, sortable: true }
    }
}

impl TableColumn {
    /// Столбец заданной ширины.
    pub fn new(width: i32) -> Self {
        TableColumn { width: width.max(1), ..Default::default() }
    }
    pub fn title(mut self, t: &str) -> Self {
        self.title = Some(t.to_string());
        self
    }
    pub fn align(mut self, a: Align) -> Self {
        self.align = a;
        self
    }
    pub fn limits(mut self, min: i32, max: i32) -> Self {
        self.min_width = min.max(1);
        self.max_width = max.max(self.min_width);
        self.width = self.width.clamp(self.min_width, self.max_width);
        self
    }
    pub fn visible(mut self, v: bool) -> Self {
        self.visible = v;
        self
    }
    pub fn resizable(mut self, v: bool) -> Self {
        self.resizable = v;
        self
    }
    pub fn sortable(mut self, v: bool) -> Self {
        self.sortable = v;
        self
    }
}

/// Что выбирает курсор.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SelectionMode {
    /// Одна ячейка (←/→ двигают курсор по столбцам).
    #[default]
    Cell,
    /// Строка целиком (←/→ прокручивают столбцы).
    Row,
}

/// Свои цвета таблицы (готовые атрибуты, без палитры).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableColors {
    pub normal: Attr,
    /// Чётные строки при включённой «зебре».
    pub zebra: Attr,
    pub header: Attr,
    /// Заголовок столбца, по которому отсортировано.
    pub header_sorted: Attr,
    /// Курсор, когда таблица в фокусе.
    pub cursor: Attr,
    /// Курсор, когда таблица не в фокусе.
    pub cursor_inactive: Attr,
    /// Выделенные строки.
    pub selected: Attr,
    /// Линии сетки.
    pub grid: Attr,
    /// Номера строк.
    pub row_number: Attr,
}

/// Цвета на случай, если у владельца нет подходящей палитры.
pub const DEFAULT_TABLE_COLORS: TableColors = TableColors {
    normal: 0x1E,
    zebra: 0x1F,
    header: 0x70,
    header_sorted: 0x74,
    cursor: 0x2F,
    cursor_inactive: 0x30,
    selected: 0x3E,
    grid: 0x17,
    row_number: 0x13,
};

/// «Зебра» по умолчанию: тот же цвет с чуть другим фоном
/// (см. [`crate::color::shade_bg`]; работает и для 256-цветного фона).
fn zebra_of(a: Attr) -> Attr {
    crate::color::shade_bg(a)
}

// ----------------------------------------------------------------------
// Таблица
// ----------------------------------------------------------------------

/// Видимый столбец на экране.
#[derive(Clone, Copy, Debug)]
struct Slot {
    /// Номер в списке видимых столбцов.
    vis: usize,
    x: i32,
    /// Видимая ширина (последний столбец может быть обрезан).
    w: i32,
    /// Полная ширина столбца.
    full: i32,
}

/// Правка ячейки на месте.
#[derive(Clone, Debug)]
struct Edit {
    row: usize,
    col: usize,
    text: Vec<char>,
    pos: usize,
}

/// Перетаскивание границы столбца.
#[derive(Clone, Copy, Debug)]
struct Resize {
    col: usize,
    start_x: i32,
    start_w: i32,
}

pub struct Table {
    pub vs: ViewState,
    model: Box<dyn TableModel>,
    /// Описания столбцов (по одному на столбец модели).
    pub columns: Vec<TableColumn>,
    pub v_scroll: Option<ViewId>,
    pub h_scroll: Option<ViewId>,

    // --- настройки ---
    pub show_header: bool,
    /// Вертикальные линии между столбцами и линия под заголовками.
    pub show_grid: bool,
    pub show_row_numbers: bool,
    /// Номер первой строки в столбце номеров (обычно 1).
    pub row_number_base: usize,
    /// Сколько первых видимых столбцов закрепить слева.
    pub frozen_cols: usize,
    pub selection_mode: SelectionMode,
    /// Разрешено выделение нескольких строк.
    pub multi_select: bool,
    /// Разрешена правка (если модель её поддерживает).
    pub allow_edit: bool,
    pub allow_sort: bool,
    pub allow_resize: bool,
    pub zebra: bool,
    /// Свои цвета (`None` — по палитре владельца).
    pub colors: Option<TableColors>,

    // --- состояние ---
    cur_row: usize,
    /// Курсор: номер в списке видимых столбцов.
    cur_vis: usize,
    top_row: usize,
    /// Первый показанный незакреплённый столбец (номер в списке видимых).
    left_vis: usize,
    /// Сортировка: (столбец модели, по возрастанию).
    sort: Option<(usize, bool)>,
    selected: BTreeSet<usize>,
    /// Начало выделения Shift+стрелками.
    anchor: Option<usize>,
    edit: Option<Edit>,
    resize: Option<Resize>,
    last_sent: Option<(usize, usize, usize, usize, i32)>,
    last_cursor: (usize, usize),
}

impl Table {
    /// Таблица по модели. `v_scroll`, `h_scroll` — id полос прокрутки
    /// (горизонтальная прокручивает столбцы).
    pub fn new(bounds: Rect, model: Box<dyn TableModel>, v_scroll: Option<ViewId>, h_scroll: Option<ViewId>) -> Self {
        let mut vs = ViewState::new(bounds);
        vs.options |= of::SELECTABLE | of::FIRST_CLICK;
        let columns = (0..model.col_count()).map(|_| TableColumn::default()).collect();
        Table {
            vs,
            model,
            columns,
            v_scroll,
            h_scroll,
            show_header: true,
            show_grid: true,
            show_row_numbers: false,
            row_number_base: 1,
            frozen_cols: 0,
            selection_mode: SelectionMode::Cell,
            multi_select: false,
            allow_edit: false,
            allow_sort: true,
            allow_resize: true,
            zebra: false,
            colors: None,
            cur_row: 0,
            cur_vis: 0,
            top_row: 0,
            left_vis: 0,
            sort: None,
            selected: BTreeSet::new(),
            anchor: None,
            edit: None,
            resize: None,
            last_sent: None,
            last_cursor: (0, 0),
        }
    }

    // --- настройка цепочкой вызовов ---

    pub fn with_columns(mut self, cols: Vec<TableColumn>) -> Self {
        self.set_columns(cols);
        self
    }
    pub fn header(mut self, on: bool) -> Self {
        self.show_header = on;
        self
    }
    pub fn grid(mut self, on: bool) -> Self {
        self.show_grid = on;
        self
    }
    pub fn row_numbers(mut self, on: bool) -> Self {
        self.show_row_numbers = on;
        self
    }
    pub fn frozen(mut self, n: usize) -> Self {
        self.frozen_cols = n;
        self
    }
    pub fn selection(mut self, m: SelectionMode) -> Self {
        self.selection_mode = m;
        self
    }
    pub fn multi_select(mut self, on: bool) -> Self {
        self.multi_select = on;
        self
    }
    pub fn editable(mut self, on: bool) -> Self {
        self.allow_edit = on;
        self
    }
    pub fn sortable(mut self, on: bool) -> Self {
        self.allow_sort = on;
        self
    }
    pub fn resizable(mut self, on: bool) -> Self {
        self.allow_resize = on;
        self
    }
    pub fn striped(mut self, on: bool) -> Self {
        self.zebra = on;
        self
    }
    pub fn with_colors(mut self, c: TableColors) -> Self {
        self.colors = Some(c);
        self
    }

    // --- модель и столбцы ---

    pub fn model(&self) -> &dyn TableModel {
        self.model.as_ref()
    }
    pub fn model_mut(&mut self) -> &mut dyn TableModel {
        self.model.as_mut()
    }

    /// Заменить модель (столбцы пересоздаются, курсор — в начало).
    pub fn set_model(&mut self, model: Box<dyn TableModel>) {
        self.model = model;
        self.columns = (0..self.model.col_count()).map(|_| TableColumn::default()).collect();
        self.cur_row = 0;
        self.cur_vis = 0;
        self.top_row = 0;
        self.left_vis = 0;
        self.sort = None;
        self.selected.clear();
        self.edit = None;
        self.last_sent = None;
    }

    /// Задать описания столбцов. Недостающие дополняются стандартными.
    pub fn set_columns(&mut self, mut cols: Vec<TableColumn>) {
        let n = self.model.col_count();
        cols.resize(n.max(cols.len()), TableColumn::default());
        self.columns = cols;
        self.refresh();
    }

    /// Подобрать ширину столбцов по содержимому первых `sample` строк
    /// (и заголовку), в пределах `min_width..=max_width`.
    pub fn auto_width(&mut self, sample: usize) {
        let rows = self.model.row_count().min(sample);
        for c in 0..self.columns.len() {
            let mut w = str_width(&self.title_of(c)) as i32 + 2;
            for r in 0..rows {
                w = w.max(str_width(&self.model.cell(r, c)) as i32);
            }
            let col = &mut self.columns[c];
            col.width = w.clamp(col.min_width, col.max_width);
        }
    }

    /// После изменения данных модели: поправить курсор и прокрутку.
    pub fn refresh(&mut self) {
        let n = self.model.col_count();
        if self.columns.len() < n {
            self.columns.resize(n, TableColumn::default());
        }
        let rows = self.model.row_count();
        self.cur_row = self.cur_row.min(rows.saturating_sub(1));
        self.top_row = self.top_row.min(self.cur_row);
        let nv = self.visible_cols().len();
        self.cur_vis = self.cur_vis.min(nv.saturating_sub(1));
        self.selected.retain(|&r| r < rows);
        self.last_sent = None;
        self.track_cursor();
    }

    // --- курсор и выделение ---

    /// Курсор: (строка, столбец модели).
    pub fn cursor(&self) -> (usize, usize) {
        (self.cur_row, self.vis_to_col(self.cur_vis))
    }

    /// Поставить курсор на ячейку (столбец — номер в модели).
    pub fn set_cursor(&mut self, row: usize, col: usize) {
        self.cur_row = row.min(self.model.row_count().saturating_sub(1));
        if let Some(v) = self.visible_cols().iter().position(|&c| c == col) {
            self.cur_vis = v;
        }
        self.track_cursor();
    }

    /// Выделенные строки (по возрастанию). Без выделения — пусто.
    pub fn selected_rows(&self) -> Vec<usize> {
        self.selected.iter().copied().collect()
    }

    /// Выделить строки.
    pub fn select_rows(&mut self, rows: &[usize]) {
        let n = self.model.row_count();
        self.selected = rows.iter().copied().filter(|&r| r < n).collect();
    }

    pub fn clear_selection(&mut self) {
        self.selected.clear();
    }

    /// Текущая сортировка: (столбец модели, по возрастанию).
    pub fn sort_state(&self) -> Option<(usize, bool)> {
        self.sort
    }

    /// Отсортировать по столбцу (если модель умеет).
    pub fn sort_by(&mut self, col: usize, ascending: bool) -> bool {
        self.edit = None;
        if self.model.sort(col, ascending) {
            self.sort = Some((col, ascending));
            self.selected.clear();
            self.last_sent = None;
            true
        } else {
            false
        }
    }

    /// Идёт ли правка ячейки.
    pub fn is_editing(&self) -> bool {
        self.edit.is_some()
    }

    // --- вспомогательное ---

    fn title_of(&self, c: usize) -> String {
        self.columns.get(c).and_then(|x| x.title.clone()).unwrap_or_else(|| self.model.header(c))
    }

    /// Номера видимых столбцов модели.
    fn visible_cols(&self) -> Vec<usize> {
        (0..self.model.col_count()).filter(|&c| self.columns.get(c).map(|x| x.visible).unwrap_or(true)).collect()
    }

    fn vis_to_col(&self, v: usize) -> usize {
        self.visible_cols().get(v).copied().unwrap_or(0)
    }

    fn col_width(&self, c: usize) -> i32 {
        self.columns.get(c).map(|x| x.width).unwrap_or(12).max(1)
    }

    fn sep(&self) -> i32 {
        if self.show_grid {
            1
        } else {
            0
        }
    }

    /// Строк заголовка (заголовок + линия под ним).
    fn header_rows(&self) -> i32 {
        if !self.show_header {
            0
        } else if self.show_grid {
            2
        } else {
            1
        }
    }

    fn page_rows(&self) -> usize {
        (self.vs.size.y - self.header_rows()).max(1) as usize
    }

    /// Ширина столбца номеров строк (с разделителем).
    fn row_num_width(&self) -> i32 {
        if !self.show_row_numbers {
            return 0;
        }
        let last = self.model.row_count() + self.row_number_base;
        last.max(1).to_string().len() as i32 + 1 + self.sep()
    }

    fn frozen_count(&self) -> usize {
        self.frozen_cols.min(self.visible_cols().len())
    }

    /// Раскладка видимых столбцов при текущей прокрутке.
    fn layout(&self) -> Vec<Slot> {
        let vis = self.visible_cols();
        let width = self.vs.size.x;
        let mut x = self.row_num_width();
        let mut out = Vec::new();
        let frozen = self.frozen_count();
        let order = (0..frozen).chain((frozen + self.left_vis)..vis.len());
        for v in order {
            if x >= width {
                break;
            }
            let full = self.col_width(vis[v]);
            let w = full.min(width - x);
            out.push(Slot { vis: v, x, w, full });
            x += full + self.sep();
        }
        out
    }

    /// Прокрутить так, чтобы курсор был виден.
    fn track_cursor(&mut self) {
        let page = self.page_rows();
        if self.cur_row < self.top_row {
            self.top_row = self.cur_row;
        } else if self.cur_row >= self.top_row + page {
            self.top_row = self.cur_row + 1 - page;
        }
        let nv = self.visible_cols().len();
        let frozen = self.frozen_count();
        let scrollable = nv.saturating_sub(frozen);
        self.left_vis = self.left_vis.min(scrollable.saturating_sub(1));
        if self.selection_mode == SelectionMode::Row || self.cur_vis < frozen {
            return;
        }
        let target = self.cur_vis - frozen;
        if target < self.left_vis {
            self.left_vis = target;
        }
        // Сдвигаем вправо, пока столбец курсора не поместится целиком.
        while self.left_vis < target {
            let fits = self.layout().iter().any(|s| s.vis == self.cur_vis && s.w == s.full);
            if fits {
                break;
            }
            self.left_vis += 1;
        }
    }

    /// Цвета: свои или по палитре владельца.
    fn palette_colors(&self, dc: &DrawCtx) -> TableColors {
        if let Some(c) = self.colors {
            return c;
        }
        let pick = |n: [u8; 7]| {
            let a: Vec<Attr> = n.iter().map(|&i| dc.color(i)).collect();
            TableColors {
                normal: a[0],
                zebra: zebra_of(a[0]),
                header: a[1],
                header_sorted: a[2],
                cursor: a[3],
                cursor_inactive: a[4],
                selected: a[5],
                grid: a[6],
                row_number: a[6],
            }
        };
        // У таблицы пустая палитра: номера цветов идут прямо к владельцу.
        // [обычный, заголовок, отсортированный, курсор, курсор без фокуса, выделение, сетка]
        match dc.owner_palette_len() {
            // Окно: 6 — текст, 7 — выделенный текст, 2 — активная рамка,
            // 3 — значки рамки, 4–5 — полоса прокрутки, 1 — пассивная рамка.
            8 => pick([6, 2, 3, 7, 4, 5, 1]),
            // Диалог: 26–29 — список, 7–8 — метка.
            32 => pick([26, 7, 8, 27, 28, 28, 29]),
            // Прямо на рабочем столе: номера синего окна в палитре приложения.
            n if n >= 100 => pick([13, 9, 10, 14, 11, 12, 8]),
            _ => DEFAULT_TABLE_COLORS,
        }
    }

    /// Текст, вписанный ровно в `w` ячеек экрана (с «…» при обрезке).
    fn fit(s: &str, w: usize, align: Align) -> String {
        let sw = str_width(s);
        if sw > w {
            if w == 0 {
                return String::new();
            }
            let mut out = String::new();
            let mut used = 0;
            for c in s.chars() {
                let cw = char_width(c);
                if used + cw > w - 1 {
                    break;
                }
                out.push(c);
                used += cw;
            }
            out.push('…');
            used += 1;
            out.extend(std::iter::repeat_n(' ', w - used));
            return out;
        }
        let pad = w - sw;
        let (l, r) = match align {
            Align::Left => (0, pad),
            Align::Right => (pad, 0),
            Align::Center => (pad / 2, pad - pad / 2),
        };
        format!("{}{}{}", " ".repeat(l), s, " ".repeat(r))
    }

    /// Сообщить полосам прокрутки текущие значения.
    fn sync_scroll(&mut self, ctx: &mut Ctx) {
        let rows = self.model.row_count();
        let nv = self.visible_cols().len();
        let scrollable = nv.saturating_sub(self.frozen_count());
        let key = (self.cur_row, rows, self.left_vis, scrollable, self.vs.size.y);
        if self.last_sent != Some(key) {
            self.last_sent = Some(key);
            if let Some(id) = self.v_scroll {
                let page = (self.page_rows() as i32 - 1).max(1);
                ctx.message(
                    cm::SCROLL_BAR_SET,
                    Info::Scroll { id, value: self.cur_row as i32, min: 0, max: rows.saturating_sub(1) as i32, page, arrow: 1 },
                );
            }
            if let Some(id) = self.h_scroll {
                ctx.message(
                    cm::SCROLL_BAR_SET,
                    Info::Scroll { id, value: self.left_vis as i32, min: 0, max: scrollable.saturating_sub(1) as i32, page: 1, arrow: 1 },
                );
            }
        }
        let cur = self.cursor();
        if cur != self.last_cursor {
            self.last_cursor = cur;
            ctx.message(cm::TABLE_CURSOR_MOVED, Info::Id(self.vs.id));
        }
    }

    fn move_cursor(&mut self, row: i64, vis: i64, extend: bool) {
        let rows = self.model.row_count();
        let nv = self.visible_cols().len();
        if rows == 0 || nv == 0 {
            return;
        }
        let new_row = row.clamp(0, rows as i64 - 1) as usize;
        if extend && self.multi_select {
            let a = *self.anchor.get_or_insert(self.cur_row);
            self.selected = (a.min(new_row)..=a.max(new_row)).collect();
        } else if !extend {
            self.anchor = None;
        }
        self.cur_row = new_row;
        if self.selection_mode == SelectionMode::Cell {
            self.cur_vis = vis.clamp(0, nv as i64 - 1) as usize;
        }
        self.track_cursor();
    }

    /// Прокрутка столбцов (в режиме строк и для колеса с Shift).
    fn scroll_cols(&mut self, d: i64) {
        let scrollable = self.visible_cols().len().saturating_sub(self.frozen_count());
        self.left_vis = (self.left_vis as i64 + d).clamp(0, scrollable.saturating_sub(1) as i64) as usize;
    }

    /// Изменить ширину столбца модели `col` на `d`.
    pub fn resize_column(&mut self, col: usize, d: i32) {
        if let Some(c) = self.columns.get_mut(col) {
            if c.resizable {
                c.width = (c.width + d).clamp(c.min_width, c.max_width);
            }
        }
    }

    /// Текст для копирования: выделенные строки (через табуляцию) или ячейка.
    pub fn copy_text(&self) -> String {
        let vis = self.visible_cols();
        if !self.selected.is_empty() {
            let lines: Vec<String> =
                self.selected.iter().map(|&r| vis.iter().map(|&c| self.model.cell(r, c)).collect::<Vec<_>>().join("\t")).collect();
            return lines.join("\n");
        }
        if self.model.row_count() == 0 || vis.is_empty() {
            return String::new();
        }
        let (r, c) = self.cursor();
        if self.selection_mode == SelectionMode::Row {
            vis.iter().map(|&c| self.model.cell(r, c)).collect::<Vec<_>>().join("\t")
        } else {
            self.model.cell(r, c)
        }
    }

    // --- правка ---

    fn can_edit(&self) -> bool {
        if !self.allow_edit || self.model.row_count() == 0 || self.visible_cols().is_empty() {
            return false;
        }
        let (r, c) = self.cursor();
        self.model.editable(r, c)
    }

    fn start_edit(&mut self, initial: Option<char>) {
        let (row, col) = self.cursor();
        let text: Vec<char> = match initial {
            Some(ch) => vec![ch],
            None => self.model.cell(row, col).chars().collect(),
        };
        let pos = text.len();
        // Столбец правки должен быть виден.
        if self.selection_mode == SelectionMode::Row {
            if let Some(v) = self.visible_cols().iter().position(|&c| c == col) {
                self.cur_vis = v;
            }
        }
        self.edit = Some(Edit { row, col, text, pos });
        self.vs.state |= sf::CURSOR_VIS;
    }

    fn finish_edit(&mut self, commit: bool, ctx: &mut Ctx) {
        if let Some(e) = self.edit.take() {
            self.vs.state &= !sf::CURSOR_VIS;
            if commit {
                let s: String = e.text.iter().collect();
                if self.model.set_cell(e.row, e.col, &s) {
                    ctx.message(cm::TABLE_CELL_EDITED, Info::Id(self.vs.id));
                }
            }
        }
    }

    /// Клавиши во время правки. `true` — клавиша обработана.
    fn edit_key(&mut self, k: crate::event::KeyEvent, ctx: &mut Ctx) -> bool {
        let e = match self.edit.as_mut() {
            Some(e) => e,
            None => return false,
        };
        match (k.key, k.mods.ctrl, k.mods.alt) {
            (Key::Enter, false, false) => {
                self.finish_edit(true, ctx);
            }
            (Key::Esc, _, _) => {
                self.finish_edit(false, ctx);
            }
            (Key::Up, false, false) | (Key::Down, false, false) => {
                self.finish_edit(true, ctx);
                let d = if k.key == Key::Up { -1 } else { 1 };
                self.move_cursor(self.cur_row as i64 + d, self.cur_vis as i64, false);
            }
            (Key::Tab, false, false) | (Key::BackTab, false, false) => {
                // Tab — сохранить и выйти: дальше Tab обработает окно.
                self.finish_edit(true, ctx);
                return false;
            }
            (Key::Left, false, false) => e.pos = e.pos.saturating_sub(1),
            (Key::Right, false, false) => e.pos = (e.pos + 1).min(e.text.len()),
            (Key::Home, false, false) => e.pos = 0,
            (Key::End, false, false) => e.pos = e.text.len(),
            (Key::Backspace, false, false) => {
                if e.pos > 0 {
                    e.pos -= 1;
                    e.text.remove(e.pos);
                }
            }
            (Key::Del, false, false) => {
                if e.pos < e.text.len() {
                    e.text.remove(e.pos);
                }
            }
            (Key::Char(c), true, false) if c.eq_ignore_ascii_case(&'v') => {
                let s = ctx.clipboard.get();
                for ch in s.chars().filter(|c| *c != '\n' && *c != '\r') {
                    e.text.insert(e.pos, ch);
                    e.pos += 1;
                }
            }
            (Key::Ins, false, false) if k.mods.shift => {
                let s = ctx.clipboard.get();
                for ch in s.chars().filter(|c| *c != '\n' && *c != '\r') {
                    e.text.insert(e.pos, ch);
                    e.pos += 1;
                }
            }
            _ => match k.text_char() {
                Some(c) => {
                    e.text.insert(e.pos, c);
                    e.pos += 1;
                }
                None => return true, // прочие сочетания во время правки глотаем
            },
        }
        true
    }

    /// Ячейка под точкой (локальные координаты): (строка или None для
    /// заголовка, номер видимого столбца или None).
    fn hit(&self, p: Point) -> (Option<usize>, Option<usize>) {
        let slot = self.layout().into_iter().find(|s| p.x >= s.x && p.x < s.x + s.w).map(|s| s.vis);
        let hr = self.header_rows();
        if p.y < hr {
            return (None, slot);
        }
        let row = self.top_row + (p.y - hr) as usize;
        (Some(row), slot)
    }

    /// Граница столбца под точкой заголовка (для изменения ширины).
    fn border_at(&self, x: i32) -> Option<usize> {
        let vis = self.visible_cols();
        self.layout().into_iter().find(|s| x == s.x + s.full || (self.sep() == 0 && x == s.x + s.full - 1)).map(|s| vis[s.vis])
    }
}

impl View for Table {
    crate::impl_view_basics!();

    fn draw(&self, dc: &mut DrawCtx) {
        let colors = self.palette_colors(dc);
        let w = self.vs.size.x.max(0) as usize;
        let focused = self.vs.is_focused();
        let vis = self.visible_cols();
        let slots = self.layout();
        let rn_w = self.row_num_width();
        let sep = self.sep();
        let rows = self.model.row_count();
        let mut y = 0;

        // Заголовок.
        if self.show_header {
            let mut b = DrawBuffer::new(w);
            b.move_char(0, ' ', colors.header, w);
            for s in &slots {
                let c = vis[s.vis];
                let mut title = self.title_of(c);
                let sorted = self.sort.filter(|(sc, _)| *sc == c);
                if let Some((_, asc)) = sorted {
                    title.push(if asc { '▲' } else { '▼' });
                }
                let attr = if sorted.is_some() { colors.header_sorted } else { colors.header };
                let align = if self.columns.get(c).map(|x| x.align) == Some(Align::Right) { Align::Right } else { Align::Center };
                b.move_str(s.x as usize, &Self::fit(&title, s.w as usize, align), attr);
                if sep == 1 && s.x + s.full < w as i32 {
                    b.move_char((s.x + s.full) as usize, '│', colors.header, 1);
                }
            }
            if rn_w > 0 {
                b.move_str(0, &" ".repeat((rn_w - sep) as usize), colors.header);
                if sep == 1 {
                    b.move_char((rn_w - 1) as usize, '│', colors.header, 1);
                }
            }
            dc.write_line(0, y, self.vs.size.x, 1, &b);
            y += 1;
            if self.show_grid {
                let mut b = DrawBuffer::new(w);
                b.move_char(0, '─', colors.grid, w);
                if rn_w > 0 {
                    b.move_char((rn_w - 1) as usize, '┼', colors.grid, 1);
                }
                for s in &slots {
                    if s.x + s.full < w as i32 {
                        b.move_char((s.x + s.full) as usize, '┼', colors.grid, 1);
                    }
                }
                dc.write_line(0, y, self.vs.size.x, 1, &b);
                y += 1;
            }
        }

        // Строки.
        let mut row = self.top_row;
        while y < self.vs.size.y {
            let mut b = DrawBuffer::new(w);
            if row >= rows {
                b.move_char(0, ' ', colors.normal, w);
                if rows == 0 && row == 0 {
                    b.move_str(1, crate::locale::tr(crate::widgets::listbox::EMPTY_TEXT), colors.normal);
                }
                // Продолжить линии сетки до низа.
                if sep == 1 {
                    if rn_w > 0 {
                        b.move_char((rn_w - 1) as usize, '│', colors.grid, 1);
                    }
                    for s in &slots {
                        if s.x + s.full < w as i32 {
                            b.move_char((s.x + s.full) as usize, '│', colors.grid, 1);
                        }
                    }
                }
                dc.write_line(0, y, self.vs.size.x, 1, &b);
                y += 1;
                row += 1;
                continue;
            }
            let base = if self.zebra && row % 2 == 1 { colors.zebra } else { colors.normal };
            let row_sel = self.selected.contains(&row);
            let row_attr = if row_sel { colors.selected } else { base };
            let is_cur_row = row == self.cur_row;
            let cursor_attr = if focused { colors.cursor } else { colors.cursor_inactive };
            let line_attr = if is_cur_row && self.selection_mode == SelectionMode::Row { cursor_attr } else { row_attr };
            b.move_char(0, ' ', line_attr, w);
            if rn_w > 0 {
                let n = (row + self.row_number_base).to_string();
                b.move_str(0, &Self::fit(&n, (rn_w - sep) as usize, Align::Right), colors.row_number);
                if sep == 1 {
                    b.move_char((rn_w - 1) as usize, '│', colors.grid, 1);
                }
            }
            for s in &slots {
                let c = vis[s.vis];
                let cell_cur = is_cur_row && self.selection_mode == SelectionMode::Cell && s.vis == self.cur_vis;
                let attr = if cell_cur { cursor_attr } else { line_attr };
                let editing = self.edit.as_ref().filter(|e| e.row == row && e.col == c);
                let text = match editing {
                    Some(e) => {
                        // Текст правки: прокручиваем так, чтобы курсор был виден.
                        let start = e.pos.saturating_sub(s.w.max(1) as usize - 1);
                        e.text[start..].iter().collect::<String>()
                    }
                    None => self.model.cell(row, c),
                };
                let align = if editing.is_some() { Align::Left } else { self.columns.get(c).map(|x| x.align).unwrap_or_default() };
                let shown = if editing.is_some() {
                    // Без «…»: при правке видна часть текста у курсора.
                    let t: String = text.chars().collect();
                    let t = crate::text::truncate_width(&t, s.w as usize).to_string();
                    let pad = s.w as usize - str_width(&t);
                    format!("{}{}", t, " ".repeat(pad))
                } else {
                    Self::fit(&text, s.w as usize, align)
                };
                b.move_str(s.x as usize, &shown, if editing.is_some() { colors.cursor } else { attr });
                if sep == 1 && s.x + s.full < w as i32 {
                    b.move_char((s.x + s.full) as usize, '│', if line_attr == base { colors.grid } else { line_attr }, 1);
                }
            }
            dc.write_line(0, y, self.vs.size.x, 1, &b);
            y += 1;
            row += 1;
        }
    }

    fn handle_event(&mut self, ev: &mut Event, ctx: &mut Ctx) {
        default_handle_event(&self.vs, ev, ctx);
        let id = self.vs.id;
        let focused = self.vs.is_focused();
        let page = self.page_rows() as i64;
        match ev {
            Event::MouseDown(m) => {
                let p = self.vs.make_local(m.pos);
                if self.edit.is_some() {
                    self.finish_edit(true, ctx);
                }
                // Граница столбца в заголовке — изменение ширины.
                if self.allow_resize && self.show_header && p.y == 0 {
                    if let Some(col) = self.border_at(p.x) {
                        if self.columns.get(col).map(|c| c.resizable).unwrap_or(false) {
                            self.resize = Some(Resize { col, start_x: p.x, start_w: self.col_width(col) });
                            ctx.grab_mouse(id);
                            ev.clear();
                            return;
                        }
                    }
                }
                match self.hit(p) {
                    (None, Some(v)) => {
                        // Щелчок по заголовку — сортировка.
                        let c = self.vis_to_col(v);
                        if self.allow_sort && self.columns.get(c).map(|x| x.sortable).unwrap_or(true) {
                            let asc = !matches!(self.sort, Some((sc, true)) if sc == c);
                            self.sort_by(c, asc);
                        }
                    }
                    (Some(r), v) if r < self.model.row_count() => {
                        let vis = v.unwrap_or(self.cur_vis) as i64;
                        if m.mods.ctrl && self.multi_select {
                            if !self.selected.remove(&r) {
                                self.selected.insert(r);
                            }
                            self.move_cursor(r as i64, vis, false);
                        } else {
                            self.move_cursor(r as i64, vis, m.mods.shift);
                            if !m.mods.shift {
                                self.selected.clear();
                            }
                        }
                        if m.double {
                            if self.can_edit() {
                                self.start_edit(None);
                            } else {
                                ctx.message(cm::LIST_ITEM_SELECTED, Info::Id(id));
                            }
                        }
                    }
                    _ => {}
                }
                ev.clear();
            }
            Event::MouseMove(m) | Event::MouseAuto(m) if ctx.mouse_grabbed_by(id) => {
                if let Some(r) = self.resize {
                    let p = self.vs.make_local(m.pos);
                    let nw = r.start_w + (p.x - r.start_x);
                    if let Some(c) = self.columns.get_mut(r.col) {
                        c.width = nw.clamp(c.min_width, c.max_width);
                    }
                }
                ev.clear();
            }
            Event::MouseUp(_) if ctx.mouse_grabbed_by(id) => {
                self.resize = None;
                ctx.release_mouse();
                ev.clear();
            }
            Event::MouseWheel(m) => {
                if m.mods.shift {
                    self.scroll_cols(m.wheel as i64);
                } else {
                    let d = m.wheel as i64 * 3;
                    self.move_cursor(self.cur_row as i64 + d, self.cur_vis as i64, false);
                }
                ev.clear();
            }
            Event::KeyDown(k) if focused => {
                let k = *k;
                if self.edit.is_some() {
                    if self.edit_key(k, ctx) {
                        ev.clear();
                    }
                    self.sync_scroll(ctx);
                    return;
                }
                let (r, v) = (self.cur_row as i64, self.cur_vis as i64);
                let rows = self.model.row_count() as i64;
                let nv = self.visible_cols().len() as i64;
                let row_mode = self.selection_mode == SelectionMode::Row;
                let shift = k.mods.shift;
                let handled = match (k.key, k.mods.ctrl, k.mods.alt) {
                    (Key::Up, false, false) => {
                        self.move_cursor(r - 1, v, shift);
                        true
                    }
                    (Key::Down, false, false) => {
                        self.move_cursor(r + 1, v, shift);
                        true
                    }
                    (Key::PgUp, false, false) => {
                        self.move_cursor(r - page, v, shift);
                        true
                    }
                    (Key::PgDn, false, false) => {
                        self.move_cursor(r + page, v, shift);
                        true
                    }
                    (Key::PgUp | Key::Home, true, false) => {
                        self.move_cursor(0, v, shift);
                        true
                    }
                    (Key::PgDn | Key::End, true, false) => {
                        self.move_cursor(rows - 1, v, shift);
                        true
                    }
                    (Key::Left, false, false) => {
                        if row_mode {
                            self.scroll_cols(-1);
                        } else {
                            self.move_cursor(r, v - 1, false);
                        }
                        true
                    }
                    (Key::Right, false, false) => {
                        if row_mode {
                            self.scroll_cols(1);
                        } else {
                            self.move_cursor(r, v + 1, false);
                        }
                        true
                    }
                    (Key::Home, false, false) => {
                        if row_mode {
                            self.left_vis = 0;
                        } else {
                            self.move_cursor(r, 0, false);
                        }
                        true
                    }
                    (Key::End, false, false) => {
                        if row_mode {
                            self.scroll_cols(nv);
                        } else {
                            self.move_cursor(r, nv - 1, false);
                        }
                        true
                    }
                    (Key::Left | Key::Right, true, false) if self.allow_resize => {
                        let c = self.vis_to_col(self.cur_vis);
                        self.resize_column(c, if k.key == Key::Left { -1 } else { 1 });
                        self.track_cursor();
                        true
                    }
                    (Key::Char(c), true, false) if c.eq_ignore_ascii_case(&'a') && self.multi_select => {
                        self.selected = (0..self.model.row_count()).collect();
                        true
                    }
                    (Key::Char(c), true, false) if c.eq_ignore_ascii_case(&'c') => {
                        ctx.clipboard.set(&self.copy_text());
                        true
                    }
                    (Key::Ins, true, false) => {
                        ctx.clipboard.set(&self.copy_text());
                        true
                    }
                    (Key::F(2), false, false) if self.can_edit() => {
                        self.start_edit(None);
                        true
                    }
                    (Key::Enter, false, false) => {
                        // Клавиша не поглощается: в диалоге Enter ещё нажмёт
                        // кнопку по умолчанию.
                        ctx.message(cm::LIST_ITEM_SELECTED, Info::Id(id));
                        false
                    }
                    (Key::Char(' '), false, false) if self.multi_select => {
                        let row = self.cur_row;
                        if !self.selected.remove(&row) {
                            self.selected.insert(row);
                        }
                        self.move_cursor(r + 1, v, false);
                        true
                    }
                    _ => match k.text_char() {
                        // Буква начинает правку с этой буквы (как в электронных таблицах).
                        Some(c) if self.can_edit() && c != ' ' => {
                            self.start_edit(Some(c));
                            true
                        }
                        _ => false,
                    },
                };
                if handled {
                    ev.clear();
                }
            }
            Event::Broadcast(m) => match (&m.command, &m.info) {
                (&cm::SCROLL_BAR_CHANGED, Info::Scroll { id: sb, value, .. }) if Some(*sb) == self.v_scroll => {
                    let v = self.cur_vis as i64;
                    self.move_cursor((*value).max(0) as i64, v, false);
                    self.last_sent = None;
                }
                (&cm::SCROLL_BAR_CHANGED, Info::Scroll { id: sb, value, .. }) if Some(*sb) == self.h_scroll => {
                    let scrollable = self.visible_cols().len().saturating_sub(self.frozen_count());
                    self.left_vis = ((*value).max(0) as usize).min(scrollable.saturating_sub(1));
                    // Курсор ячейки остаётся в видимой части.
                    if self.selection_mode == SelectionMode::Cell && self.cur_vis >= self.frozen_count() {
                        let slots = self.layout();
                        if !slots.iter().any(|s| s.vis == self.cur_vis) {
                            if let Some(s) = slots.iter().find(|s| s.vis >= self.frozen_count()) {
                                self.cur_vis = s.vis;
                            }
                        }
                    }
                }
                (&cm::SCROLL_BAR_CLICKED, Info::Id(sb)) if Some(*sb) == self.v_scroll || Some(*sb) == self.h_scroll => {
                    if !self.vs.is_selected() {
                        ctx.select(id);
                    }
                }
                _ => {}
            },
            _ => {}
        }
        // Курсор терминала — в ячейке правки.
        if let Some(e) = &self.edit {
            let vis = self.visible_cols();
            if let Some(s) = self.layout().into_iter().find(|s| vis[s.vis] == e.col) {
                let start = e.pos.saturating_sub(s.w.max(1) as usize - 1);
                let x: usize = e.text[start..e.pos].iter().map(|c| char_width(*c)).sum();
                let y = self.header_rows() + (e.row as i32 - self.top_row as i32);
                self.vs.set_cursor(s.x + x as i32, y);
            }
        }
        self.sync_scroll(ctx);
    }

    fn set_state(&mut self, flag: u16, enable: bool) {
        if enable {
            self.vs.state |= flag;
        } else {
            self.vs.state &= !flag;
        }
        // Потеря фокуса заканчивает правку (с сохранением нельзя — нет ctx).
        if flag & sf::FOCUSED != 0 && !enable && self.edit.is_some() {
            if let Some(e) = self.edit.take() {
                let s: String = e.text.iter().collect();
                self.model.set_cell(e.row, e.col, &s);
            }
            self.vs.state &= !sf::CURSOR_VIS;
        }
    }

    fn change_bounds(&mut self, bounds: Rect) {
        self.vs.set_bounds(bounds);
        self.last_sent = None;
        self.track_cursor();
    }

    fn has_data(&self) -> bool {
        true
    }

    fn get_data(&self) -> Value {
        Value::Int(self.cur_row as i64)
    }

    fn set_data(&mut self, value: &Value) {
        if let Value::Int(r) = value {
            let c = self.cursor().1;
            self.set_cursor((*r).max(0) as usize, c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn people() -> VecTableModel {
        VecTableModel::new(
            &["Имя", "Город", "Возраст"],
            vec![
                vec!["Иван".into(), "Тверь".into(), "30".into()],
                vec!["Анна".into(), "Москва".into(), "25".into()],
                vec!["Пётр".into(), "Казань".into(), "100".into()],
            ],
        )
    }

    #[test]
    fn letters() {
        assert_eq!(column_letters(0), "A");
        assert_eq!(column_letters(25), "Z");
        assert_eq!(column_letters(26), "AA");
        assert_eq!(column_letters(27), "AB");
        assert_eq!(column_letters(701), "ZZ");
        assert_eq!(column_letters(702), "AAA");
    }

    #[test]
    fn fit_text() {
        assert_eq!(Table::fit("abc", 5, Align::Left), "abc  ");
        assert_eq!(Table::fit("abc", 5, Align::Right), "  abc");
        assert_eq!(Table::fit("abc", 5, Align::Center), " abc ");
        assert_eq!(Table::fit("abcdef", 4, Align::Left), "abc…");
        assert_eq!(str_width(&Table::fit("日本語テキスト", 6, Align::Left)), 6);
    }

    #[test]
    fn natural_sort() {
        let mut m = people();
        m.sort(2, true);
        let ages: Vec<String> = (0..3).map(|r| m.cell(r, 2)).collect();
        assert_eq!(ages, ["25", "30", "100"]); // числа — как числа, а не как текст
        m.sort(0, false);
        assert_eq!(m.cell(0, 0), "Пётр");
    }

    #[test]
    fn huge_virtual_model() {
        // Миллион строк × тысяча столбцов: таблица спрашивает только видимое.
        let m = FnTableModel::new(1_000_000, 1000, |r, c| format!("{}:{}", r, c));
        let mut t = Table::new(Rect::new(0, 0, 60, 20), Box::new(m), None, None);
        t.set_cursor(999_999, 999);
        assert_eq!(t.cursor(), (999_999, 999));
        assert!(t.top_row > 999_000);
        assert!(t.layout().iter().any(|s| s.vis == 999));
    }

    #[test]
    fn frozen_and_hidden_columns() {
        let m = FnTableModel::new(10, 50, |r, c| format!("{}{}", r, c));
        let mut cols: Vec<TableColumn> = (0..50).map(|_| TableColumn::new(8)).collect();
        cols[1] = TableColumn::new(8).visible(false);
        let mut t = Table::new(Rect::new(0, 0, 40, 10), Box::new(m), None, None).with_columns(cols).frozen(1);
        t.set_cursor(0, 30);
        let l = t.layout();
        // Первый слот — закреплённый столбец 0, скрытый столбец 1 не показан.
        assert_eq!(t.visible_cols()[l[0].vis], 0);
        assert!(!l.iter().any(|s| t.visible_cols()[s.vis] == 1));
        assert!(l.iter().any(|s| t.visible_cols()[s.vis] == 30));
    }

    #[test]
    fn copy_selection() {
        let mut t = Table::new(Rect::new(0, 0, 40, 10), Box::new(people()), None, None).multi_select(true);
        t.select_rows(&[0, 2]);
        assert_eq!(t.copy_text(), "Иван\tТверь\t30\nПётр\tКазань\t100");
        t.clear_selection();
        t.set_cursor(1, 1);
        assert_eq!(t.copy_text(), "Москва");
    }
}
