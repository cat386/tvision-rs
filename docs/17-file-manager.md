# 17. Файловые панели и операции

Для файловых менеджеров в библиотеке есть:

* `FilePanel` — файловая панель с вкладками, фильтрами, сортировкой,
  выделением, быстрым поиском и «виртуальной панелью»;
* `DualPanel` — две панели рядом: Tab, режим одной панели, обмен местами;
* модуль `fileops` — копирование, перенос и удаление в отдельном потоке
  с прогрессом и отменой.

Возможности и клавиши взяты из консольного файлового менеджера
[lfm](https://inigo.katxi.org/devel/lfm/) (Last File Manager, Iñigo
Serna). Полный пример — `cargo run --example lfm`.

## Две панели на рабочем столе

```rust
use std::path::Path;
use tvision::*;

/// Две панели во весь рабочий стол. Возвращает id `DualPanel`.
fn panels(app: &mut Program, left: &Path, right: &Path) -> ViewId {
    let r = app.desktop().group.vs.extent();
    app.insert_window(DualPanel::new(r, left, right))
}

/// Каталог и файл под курсором в активной панели.
fn current(app: &mut Program, id: ViewId) -> Option<(String, String)> {
    let dp = app.find_mut::<DualPanel>(id)?;
    let p = dp.active();
    let file = p.current()?.name.clone();
    Some((p.path().display().to_string(), file))
}
```

`DualPanel` — группа из двух `FilePanel`. Сама она обрабатывает только Tab
(другая панель); остальное — панели.

| Метод `DualPanel` | Что делает |
|-------------------|-----------|
| `active()`, `active_mut()`, `other()`, `other_mut()` | панель в фокусе и другая |
| `left_id()`, `right_id()`, `active_id()`, `other_id()`, `panel_mut(id)` | панели по id |
| `toggle_single()`, поле `single` | одна или две панели (`.` в lfm) |
| `swap()` | поменять панели местами (Ctrl+U) |
| `sync_other()` | открыть в другой панели тот же каталог (`=`) |
| `switch(ctx)` | переключить фокус (Tab) |

`FilePanel` можно поставить и отдельно — в окно или диалог: цвета она
подбирает по палитре владельца, как таблица.

## Что умеет панель

| Возможность | Клавиши в панели | Методы |
|-------------|------------------|--------|
| движение | ↑ ↓ PgUp PgDn Home End, Ctrl+↑/↓ (на 10) | `cursor()`, `set_cursor(i)`, `select_name(n)`, `select_path(p)` |
| каталоги | ← / Backspace / Ctrl+PgUp — наверх, → / Ctrl+PgDn — внутрь, Enter | `path()`, `set_path(p)`, `change_dir(p, Some(имя))`, `go_up()`, `enter()`, `refresh()` (Ctrl+R) |
| файл под курсором | Enter на файле — команда `cm::FILE_PANEL_EXECUTE` | `current()`, `entries()` |
| выделение | Ins, пробел, правая кнопка мыши | `toggle_mark()`, `mark_glob("*.rs,*.toml", true)`, `invert_marks()`, `clear_marks()`, `marked()`, `targets()` |
| быстрый поиск | Ctrl+S и набор (повторный Ctrl+S — следующее совпадение), Alt+буква | `quick_find(текст)` |
| вкладки | щелчок по вкладке в рамке | `new_tab()`, `close_tab()`, `next_tab()`, `prev_tab()`, `select_tab(i)`, `tab_count()`, `tab_paths()` |
| история | — | `history()` — посещённые каталоги вкладки |
| фильтр вкладки | — | `set_filter("*.o,*.tmp,!keep*")`, `filter()` |
| скрытые файлы | — | `toggle_hidden()`, поле `show_hidden` |
| сортировка | — | `set_sort(SortMode::Size, true)`; поля `sort_mix_dirs`, `sort_case_sensitive` |
| размеры каталогов | — | `calc_dir_sizes()`, `dir_size(p)` |
| виртуальная панель | ← или `..` — выход | `set_listing(заголовок, корень, пути)`, `listing_title()` |
| сообщение внизу | — | `set_message(текст)`, `error()` |

`targets()` — пути для операции: выделенные файлы, а без выделения — файл
под курсором (кроме `..`). Так работают F5–F8 в lfm, MC и DN.

**Фильтр** перечисляет, что *скрыть*: `*.o,*.tmp` прячет объектные и
временные файлы, `*,!*.rs` оставляет только `.rs`, `*.jpg,!*shot*` прячет
JPEG, кроме снимков экрана. Каталоги фильтр не прячет, иначе по ним
нельзя было бы ходить. Шаблоны — `*`, `?`, `[abc]`, `[a-z]`, `[!x]`
(функция `glob_match`). На Windows регистр не различается.

**Рамка** панели: в фокусе — двойная. Вверху — путь или вкладки
(`1:src 2:docs`) и флаги: буква сортировки (`n` имя, `e` расширение,
`s` размер, `t` время, `u` без сортировки; заглавная — обратный порядок),
`.` — скрытые видны, `f` — есть фильтр, `p` — виртуальная панель.
Внизу — число файлов и их объём. Над нижней рамкой — строка сведений:
имя, размер, права и время файла под курсором или итог выделения.

**Метки** перед именем: `/` — каталог, `@` — ссылка, `*` — программа,
`|` — устройство или канал, `√` — выделенный файл.

### Пример: команда над файлом под курсором

Enter на файле не открывает его сам: панель ставит в очередь команду
`cm::FILE_PANEL_EXECUTE`, а программа решает, что делать.

```rust
use tvision::*;

struct App {
    panels: ViewId,
}

impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(cm::FILE_PANEL_EXECUTE) {
            let file = app.find::<DualPanel>(self.panels).and_then(|d| d.active().current().map(|e| e.path.clone()));
            if let Some(f) = file {
                message_box(app, &format!("Открыть {}?", f.display()), mf::CONFIRMATION | mf::YES_NO_CANCEL);
            }
            ev.clear();
        }
    }
}
```

О смене каталога панель сообщает соседям по группе сообщением
`cm::FILE_PANEL_DIR_CHANGED` (`Info::Id` панели).

### Записи

`entries()`, `current()`, `marked()` возвращают `PanelEntry`:

| Поле / метод | Что |
|--------------|-----|
| `name` | показываемое имя (в виртуальной панели — путь от её корня) |
| `path` | полный путь |
| `kind` | `FileKind::Parent` (`..`), `Dir`, `File`, `Link { to_dir }`, `Other` |
| `size`, `modified`, `mode`, `executable`, `hidden` | размер, время, права, программа, скрытый |
| `is_dir()`, `is_parent()`, `extension()`, `class()`, `perms_string()` | помощники |

`class()` — класс для цвета: каталог, программа, ссылка, архив,
изображение, звук, видео, документ, исходник, данные, временный (по
расширению, функция `class_by_name`).

### Цвета

Без настройки панель берёт цвета владельца (как `Table`): синего окна —
на рабочем столе, окна — в окне, списка — в диалоге. Типы файлов в
256-цветной палитре (gruvbox) раскрашиваются цветами gruvbox, в
классической — цветами BIOS. Свои цвета —
`FilePanelColors::derive(рамка, рамка_активная, текст, курсор)` с
правкой полей (`classes`, `marked`, `cursor`…) и `panel.colors = Some(…)`.

## Файловые операции: `fileops`

lfm выполнял долгие операции в дочернем процессе (`fork`) и останавливал
его сигналами — это работает только в UNIX. Здесь операция идёт в потоке,
а интерфейс читает общее состояние, так что всё работает и на Windows.

```rust
use std::path::PathBuf;
use tvision::fileops::{self, Conflict, FileJob, JobKind};
use tvision::*;

/// Скопировать файлы в каталог с окном прогресса и кнопкой «Стоп».
fn copy(app: &mut Program, sources: Vec<PathBuf>, dest: PathBuf) -> fileops::JobProgress {
    // Уже существующие файлы: заменить или пропустить — спросить заранее.
    let conflict = if fileops::conflicts(&sources, &dest).is_empty() { Conflict::Overwrite } else { Conflict::Skip };
    let job = FileJob::start(JobKind::Copy, sources, Some(dest), conflict);
    let mut d = Dialog::centered(50, 8, "Копирование");
    let bar = d.add(ProgressBar::new(Rect::new(2, 2, 48, 3), 100));
    d.add(Button::new(Rect::new(19, 4, 31, 6), "~С~топ", cm::CANCEL, bf::DEFAULT));
    let (c, _) = app.exec_dialog_idle(d, |app, _| {
        let p = job.progress();
        if let Some(b) = app.find_mut::<ProgressBar>(bar) {
            b.set_value(p.percent() as i64);
        }
        p.finished.then_some(cm::OK)
    });
    let mut job = job;
    if c != cm::OK {
        job.cancel();
    }
    job.wait() // итог: ошибки, пропущенные, отмена
}
```

| Функция / тип | Что делает |
|---------------|-----------|
| `FileJob::start(вид, источники, назначение, конфликт)` | начать операцию в потоке (`JobKind::Copy`, `Move`, `Delete`) |
| `job.progress()` | снимок `JobProgress`: `counting`, байты и файлы (всего / готово), `current`, `errors`, `skipped`, `finished`, `cancelled`, `percent()` |
| `job.cancel()`, `job.wait()` | остановить (после текущего блока в 1 МБ), дождаться |
| `FileJob::run_blocking(…)` | то же в текущем потоке (тесты, мелочи) |
| `conflicts(источники, назначение)` | какие файлы назначения уже существуют |
| `target_path(src, dest)` | куда попадёт источник |
| `dir_size(путь)` | (байт, файлов) |
| `is_inside(a, b)`, `remove_any(путь)` | лежит ли `a` внутри `b`; удалить файл или каталог |

Назначение — как у `cp`/`mv`: существующий каталог — источники кладутся
внутрь; иначе (один источник) — это новое имя. Перенос сначала пробует
быстрое переименование, на другой диск — копирует и удаляет. При копировании
сохраняются права и время изменения; недописанный (при ошибке или
остановке) файл удаляется; каталог в самого себя не копируется. Ссылки на
Unix копируются как ссылки, на Windows — то, на что они указывают.

## Пример `lfm`

`cargo run --example lfm [левый] [правый] [--path-file ФАЙЛ]` — файловый
менеджер на этих виджетах с клавишами lfm:

* F2–F8 — имя, просмотр (текст или шестнадцатеричный вид), правка
  (встроенный редактор), копирование, перенос, каталог, удаление — с
  окном прогресса;
* вкладки (`:` `!` `<` `>`), одна/две панели (`.`), `=`, Ctrl+U, фильтр
  (Ctrl+F), скрытые (Ctrl+H), сортировка (`s`), размеры (`#`);
* выделение `+` `-` `*`, закладки `b`/`B` + 0–9, a–z (хранятся в
  `%APPDATA%\tvision-lfm` или `~/.config/tvision-lfm`), Ctrl+D — список,
  Ctrl+Y — история, `g` — перейти, Ctrl+T — дерево, Alt+F1/F2 — диски;
* `/` — поиск файлов и текста в потоке → виртуальная панель;
* **PowerCLI** (Ctrl+X) — командная строка с подстановками `$f $F $E $e
  $p $o $s $a $i $b1 $tm $dn` и циклом по выделенным; суффиксы `&` (фон)
  и `%` (интерактивная программа, терминал отдаётся ей через
  `Program::suspended`).

В отличие от lfm, подставленные значения сразу заключаются в кавычки
оболочки. Поэтому имя файла с кавычкой или `$(…)` не выполнится как
команда. Вставок Python `{…}` нет. Архивы как каталоги (VFS lfm) пример
не открывает — Enter на архиве открывает его программой системы.

Для выхода с переходом оболочки в текущий каталог (как функция `lfm` из
README оригинала) в `.bashrc`:

```text
lfm() {
    command lfm-rs --path-file /tmp/lfm-$$.path "$@"
    [ -f /tmp/lfm-$$.path ] && cd "$(cat /tmp/lfm-$$.path)" && rm -f /tmp/lfm-$$.path
}
```

(`lfm-rs` — собранный пример: `cargo build --release --example lfm`,
файл `target/release/examples/lfm`.)

## Нужное в `Program`

| Метод | Зачем |
|-------|-------|
| `app.focus(id)` | передать фокус представлению (например, из панелей в строку PowerCLI и обратно) |
| `app.suspended(\|\| …)` | временно вернуть терминал в обычный режим и запустить интерактивную программу; после — перерисовка |
| `app.exec_dialog_idle(d, f)` | модальное окно прогресса, пока работает поток |
