# tvision-rs

**Русский:** [README.ru.md](README.ru.md)

A Rust port of the **Turbo Vision** text UI library ([magiblot/tvision](https://github.com/magiblot/tvision)), originally written in C++.

* **No external crate dependencies** — terminal I/O is implemented inside the library, like the original `source/platform`;
* **Linux / macOS / BSD** (termios + ANSI) and **Windows 10+** (Console API + VT sequences);
* Mouse, Unicode (Cyrillic, wide characters), 16-color palettes from the original plus **256 xterm colors** (`color::attr256`, themes, 256-color picker in `ColorDialog`);
* Default theme: [gruvbox](https://github.com/morhetz/gruvbox) (`color::palettes::GRUVBOX`; classic `APP_COLOR` is used on 16-color terminals);
* Source comments are mostly in Russian; UI strings can be switched with `locale::set_language`.

## Documentation

The **[user guide](docs/guide/README.md)** ([single PDF](docs/tvision-rs-guide.pdf)) is written in Russian: fifteen lessons from a blank program to JSON/SQLite/CSV apps, Qt-style layouts, the TVCreator form designer, and themes — each with a runnable `cargo run --example learn_…` program.

Broader topic chapters live under [`docs/`](docs/README.md): quick start, architecture, events, custom views, windows, menus, dialogs and controls, validation, the editor, standard dialogs, colors, platform, locale, testing, porting from C++/Pascal, constants, file panels, and `TreeView`. Per-type API reference: `cargo doc --open`.

## Running examples

```
cargo run --example hello   # port of hello.cpp
cargo run --example demo    # windows, menu, dialog with all controls, list, context menu
cargo run --example tvedit  # port of tvedit: multi-window file editor
cargo run --example tvhc -- examples/demohelp.txt hc.rs   # help compiler check + HC_* constants
cargo run --example progress  # progress bars: copy, background thread, all bar styles
cargo run --example json_parse  # JSON in editor: validate, jump to error, format, minify
cargo run --example json_table -- examples/data/books.json  # JSON file in a table: sort, record dialog
cargo run --example json_load   # large JSON load in a thread with ProgressBar and cancel
cargo run --example table   # table: virtual million rows, edit, sort, dialog
cargo run --example colors  # 256 colors: xterm palette, gruvbox/classic themes, ColorDialog, 16 vs 256 mode
cargo run --example learn_01_hello   # … learn_08_todo — programs for the tutorial lessons
cargo run --example guide_contacts  # tutorial app: table, validated dialog, files
cargo run --example mvc_contacts    # same on MVC scaffolding: JSON records (or SQLite: --features sqlite -- file.db)
cargo run --example mvc_todo        # tasks: custom model commands (F5 done), summary in title
cargo run --example mvc_books       # books: custom window, status line, Ctrl+F search, F9 stats
cargo run --example mvc_csv         # warehouse in CSV: custom Storage impl
cargo run --example sqlite_query --features sqlite  # SQL console: query on top, table below
cargo run --example json_settings   # app settings in JSON: defaults, preserve unknown keys
cargo run --example layout_demo     # Qt-style layouts: HBox, VBox, Grid, Form, stretches — no manual coords
cargo run --example themes          # themes from themes/: menu, F3 preview picker
cargo run --example treeview  # TreeView: lazy FS tree, edit, JSON, checkboxes
cargo run --example lfm     # dual-panel file manager (lfm-inspired): tabs, F5/F6/F8 + progress, PowerCLI
cargo run --example tvdemo  # full tvdemo port: calculator, calendar, puzzle, ASCII, file viewer…
cargo test --all-targets    # all tests, including tvdemo example tests
cargo test --all-targets --features sqlite  # plus SQLite (system libsqlite3 required)
cargo test -p tvcreator     # TVCreator form designer
cargo test -p tvtheme       # theme editor
cargo test                  # headless terminal tests
```

The **[Qwen Agent](examples/qwen-agent/README.md)** example is a chat UI for Qwen via Model Studio API: conversation list, message history, multi-line input, connection settings. Requires `curl` and `DASHSCOPE_API_KEY`. From the repo root:

    cargo run --release --manifest-path examples/qwen-agent/Cargo.toml

## Ported from C++

| C++ (tvision) | Rust (this crate) |
|---------------|-------------------|
| `TPoint`, `TRect` | `Point`, `Rect` |
| `TDrawBuffer`, `TScreenCell` | `DrawBuffer`, `Cell`, `ScreenBuffer` |
| `TEvent`, `kbXXX`, `cmXXX` | `Event`, `KeyEvent`, modules `kb`, `cm` |
| `TView` | trait `View` + `ViewState` |
| `TGroup` | `Group` |
| `TProgram`, `TApplication` | `Program` + trait `AppHandler` |
| `TDesktop`, `TBackground` | `Desktop`, `Background` (pattern, cascade) |
| `TWindow`, `TFrame` | `Window`, `Frame` |
| `TMenuBar`, `TMenuBox`, `TMenuPopup` | `MenuBar`, `MenuSession`, `Menu`, `MenuItem` |
| `TStatusLine`, `TStatusDef`, `TStatusItem` | `StatusLine`, `StatusDef`, `StatusItem` |
| `TDialog` | `Dialog` |
| `TButton` | `Button` |
| `TStaticText`, `TLabel` | `StaticText`, `Label` |
| `TInputLine` | `InputLine` (selection, clipboard, validators) |
| `TCheckBoxes`, `TRadioButtons` | `Cluster::check_boxes`, `Cluster::radio_buttons` |
| `TScrollBar` | `ScrollBar` |
| `TListViewer`, `TListBox` | `ListBox` (multi-column) |
| — (extension) | `Table` — data model, edit, sort, frozen columns |
| — (extension) | `ProgressBar`; `Program::exec_dialog_idle` — modal UI + background work |
| — (extension) | `TreeView` — lazy load, tri-state checks, quick search, in-place edit |
| — (extension) | `FilePanel`, `DualPanel`, `fileops` — file panels (lfm-style) and threaded ops |
| `messageBox`, `inputBox` | `message_box`, `message_box_rect`, `input_box` |
| `platform/*` | `platform::{unix, windows, ansi}` |
| `TValidator`, … | trait `Validator`, `PictureValidator`, `FilterValidator`, `RangeValidator`, `StringLookupValidator` |
| `THistory`, `THistoryWindow`, `historyAdd` | `History`, `HistoryWindow`, `history_add` |
| `TScroller`, `TTerminal` | `Scroller`, `Terminal` |
| `TEditor`, `TMemo`, `TFileEditor` | `Editor`, `Memo`, `FileEditor` (undo stack, find/replace) |
| `TEditWindow`, `TIndicator` | `EditWindow`, `Indicator` |
| `TFileDialog`, `TFileList`, `TFileInfoPane` | `FileDialog`, `FileList`, `FileInfoPane` |
| `TChDirDialog`, `TDirListBox` | `ChDirDialog`, `DirListBox` |
| `TOutlineViewer`, `TOutline`, `TNode` | `Outline`, `Node` |
| `TColorDialog`, … | `ColorDialog`, `ColorSelector`, `ColorDisplay`, `ColorGroup`, `ColorItem` |
| `THelpFile`, … | `HelpFile`, `HelpTopic`, `HelpViewer`, `HelpWindow` |
| `tvhc` tool | `tvhc` example (check + `HC_*` constants) |
| `TMultiCheckBoxes` | `Cluster::multi_check_boxes` |
| `TInputLong`, `TColoredText`, `TCombo` | `InputLong`, `StaticText::colored`, `Combo` |
| `TColorAttr`, `TColorXTerm` | `Attr` (`u32`): BIOS byte + 256 xterm colors |

## Data: JSON, SQLite, MVC

Extensions beyond original Turbo Vision:

* `tvision::json` — parse/write JSON (line/column errors, parse progress for progress bars);
* `tvision::sqlite` — SQLite via system `libsqlite3` (feature flag `sqlite`):
  `tvision-rs = { git = "…", features = ["sqlite"] }`;
* `tvision::mvc` — model, table window, controller: define `Schema` / `Field`, get table + edit dialog with validation and per-change persistence to JSON or SQLite (`open_storage` picks backend by file extension). Lessons: [11. MVC](docs/guide/11-mvc.md), [12. Recipes](docs/guide/12-data-recipes.md) (Russian).

Still **zero external crates**.

## Qt-style layouts

Module `tvision::layout`: `HBox`, `VBox`, `Grid`, `Form`, stretches (`stretch`, `Spacer`), size policies (`Fixed`, `Preferred`, `Expanding`, …), margins and spacing. Widgets use `Rect::default()`; `Window::set_layout` places and reflows them on resize; `adjust_size` sizes the window to content. Custom views implement `size_hint` and `size_policy`. Lesson: [13. Layouts](docs/guide/13-layouts.md).

## TVCreator — form designer

[`tools/tvcreator`](tools/tvcreator/README.md) is a Qt Creator / Designer–style IDE: widget palette, object tree, live canvas, property editor, undo, preview. Forms are `.tvui` (JSON) and codegen to Rust (like `.ui` + `uic`): struct with widget ids and a `create()` that builds the window with layouts.

```text
cargo run -p tvcreator                                     # IDE
cargo run -p tvcreator -- tools/tvcreator/forms/login.tvui # open sample
cargo run -p tvcreator -- --gen form.tvui -o form.rs       # codegen only
cargo run -p tvcreator -- --gen-include form.tvui -o include.rs  # for include!
cargo run -p tvcreator --example generated_forms           # forms in an app
```

Lesson: [14. TVCreator](docs/guide/14-tvcreator.md).

## TV Studio

**[TV Studio](https://github.com/cat386/tvstudio)** — a full visual IDE for tvision-rs (Turbo Pascal / Delphi style): paint forms with the mouse, edit the application tree and menus, generate Rust, edit code in the built-in editor, build and run from the IDE. Projects are stored in `.tvsp` files — this repository includes [`tvision-rs.tvsp`](tvision-rs.tvsp) for experimenting with the library as a project. TV Creator above is a focused `.tvui` designer and codegen tool; TV Studio adds the whole project workflow (workspace, build integration, forms, and sources in one environment).

## Themes and tvtheme

A theme is the app palette (135 entries) in a `.tvtheme` file.
[`tools/tvtheme`](tools/tvtheme/README.md) edits it with live preview: color groups, 16- and 256-color grids, undo, Rust export. In apps: `app.load_theme("theme.tvtheme")`, `theme::choose_theme` for interactive pick, or env `TVISION_THEME`. Samples in `themes/`.

```text
cargo run -p tvtheme -- themes/sea.tvtheme                     # editor
TVISION_THEME=themes/matrix.tvtheme cargo run --example themes # theme without code
```

Lesson: [15. Themes](docs/guide/15-themes.md).

### Application settings

`tvision::settings` provides a standard Settings dialog (theme with live preview, terminal colors, desktop pattern, language, window layout at startup, mouse) and a `<program>.settings.json` file **next to the executable**. Load at startup, capture window positions on exit:

```rust,ignore
let path = Settings::default_path();
let mut settings = Settings::load_or_default(&path);
settings.apply_language();
let mut app = Program::with_terminal(Some(menu()), Some(status()))?;
settings.apply(&mut app).ok();
open_windows(&mut app);
settings.restore_windows(&mut app);
app.run(&mut handler);
settings.capture_windows(&app);
settings.save(&path).ok();
```

Example: `cargo run --example app_settings`. Lesson: [16. Settings](docs/guide/16-settings.md).

## Not yet ported

* `TMonoSelector` (monochrome color picker);
* `TStreamable` / resource streams;
* TrueColor (24-bit RGB; 256 xterm colors already supported);
* non–UTF-8 encodings in the editor.

## System clipboard

`Ctx::clipboard` talks to the OS clipboard (input lines and editor: Ctrl+Ins / Shift+Ins / Shift+Del):

| OS | Mechanism |
|----|-----------|
| Windows | WinAPI `CF_UNICODETEXT` |
| macOS | `pbcopy` / `pbpaste` |
| Linux / BSD, Wayland | `wl-copy` / `wl-paste` |
| Linux / BSD, X11 | `xclip` or `xsel` |
| none of the above (e.g. SSH) | copy via terminal OSC 52; paste from internal fallback buffer |

## tvdemo

`examples/tvdemo/` ports the full demo: ≡ menu (Alt+Space), About, Puzzle, Calendar, ASCII table, Calculator, Event Viewer (Alt+0), file viewer, change directory, DOS shell, mouse/color/pattern settings, desktop save/restore, clock and memory meter, F1 context help from `demohelp.txt`. Differences: “Video mode” switches palettes (color / B&W / mono), not 25/43 rows; desktop state is a simple text format, not `TStreamable` streams.

Library additions for this demo: `Program::dos_shell`, `double_delay`, `reverse_buttons`, `event_monitor` + `View::monitor_event`, `Ctx::with_program`, `Desktop::set_pattern`, `platform::time`, `Menu::submenu_key`, `Menu::help`.

## Help files

Help sources use the `tvhc` text format (`examples/demohelp.txt`) — no binary `.hlp`:

```rust
let help = Rc::new(HelpFile::parse(include_str!("demohelp.txt"))?);
let ctx = app.help_ctx();
app.exec_dialog(HelpWindow::new(help.clone(), ctx));
```

`Program::palette` is a plain `Vec<u8>`; persist it however you like (e.g. a file).

## Modal dialogs from views

In C++, views called `owner->execView(dialog)` or `messageBox(...)` from `handleEvent`. Here use `ctx.exec(Box::new(dialog), Some(self.vs.id), tag)`; the result arrives in `View::exec_result(tag, cmd, view, ctx)`. Used for validator prompts, history window, “Save changes?”, editor find/replace. `View::valid(cmd, ctx)` receives `Ctx` too.

## Main difference from C++

C++ views hold an `owner` pointer and call it directly (`setCurrent`, `message`, `destroy`). Rust views **do not know their owner**: they enqueue requests on `Ctx` (`ctx.select(id)`, `ctx.message(cmd, info)`, `ctx.close(id)`), and the owner handles them after `handle_event` returns. Modal dialogs are usually started from `AppHandler`: `let (cmd, dlg) = app.exec_dialog(dlg);`.

## Minimal example

```rust
use tvision::*;

struct App;
impl AppHandler for App {
    fn handle_event(&mut self, app: &mut Program, ev: &mut Event) {
        if ev.command() == Some(cm::NEW) {
            message_box(app, "\x03Hello!", mf::INFORMATION | mf::OK_BUTTON);
            ev.clear();
        }
    }
}

fn main() -> std::io::Result<()> {
    let menu = Menu::new().submenu("~F~ile", Menu::new()
        .item_key("~H~ello", cm::NEW, kb::F2, "F2")
        .item_key("E~x~it", cm::QUIT, kb::ALT_X, "Alt-X"));
    let status = vec![StatusDef::all(vec![StatusItem::new("~Alt-X~ Exit", kb::ALT_X, cm::QUIT)])];
    Program::with_terminal(Some(menu), Some(status))?.run(&mut App);
    Ok(())
}
```

Hotkeys work across keyboard layouts: on Windows, Alt+letter arrives as the physical key’s Latin letter; menus, labels, buttons, and clusters match hotkeys using QWERTY ↔ ЙЦУКЕН pairs. Built-in library strings (message boxes, file dialogs) can be shown in Russian with `locale::set_language(locale::Language::Ru)`.

### Agent Harness

[Full agent example](examples/agent-harness/README.md): Qwen / OpenAI-compatible API chat, tools for files, shell, web, Playwright, Windows UI Automation, action confirmations, SQLite run log.

```bash
cargo run --manifest-path examples/agent-harness/Cargo.toml
```

### Markdown viewer

`MarkdownViewer` renders Markdown with wrapping, vertical scroll, and styling for headings, quotes, lists, code, and links.

```bash
cargo run --example markdown -- README.md
```

Without a path, a built-in demo document opens. F3 opens a `.md` file (UTF-8, up to 4 MiB); arrows and PgUp/PgDn scroll; Home/End jump to ends; the window is resizable and reflows text.

```rust
use tvision::*;
let mut window = Window::new(Rect::new(0, 0, 80, 24), "Document", 0);
let scroll = window.standard_scroll_bar(true);
let viewer = MarkdownViewer::new(Rect::new(1, 1, 79, 23), "# Title\n**Markdown** text", Some(scroll));
window.add(viewer);
```

`set_markdown(&str)` replaces the document and resets scroll; `markdown()` returns source after stripping control characters. Supported subset: ATX headings, lists, block quotes, fenced code, horizontal rules, basic emphasis, inline links (URL shown next to label). Not supported: nested inline markup, tables, setext headings, reference links; HTML/images are not executed; links are not opened.