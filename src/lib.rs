//! # tvision-rs
//!
//! Порт библиотеки текстового интерфейса **Turbo Vision**
//! (современная версия — <https://github.com/magiblot/tvision>) с C++ на Rust.
//!
//! ## Соответствие классов C++ и типов Rust
//!
//! | C++ (tvision)        | Rust (этот крейт)                      |
//! |----------------------|----------------------------------------|
//! | `TPoint`, `TRect`    | [`Point`], [`Rect`]                    |
//! | `TDrawBuffer`        | [`DrawBuffer`]                         |
//! | `TEvent`             | [`Event`]                              |
//! | `TView`              | трейт [`View`] + структура [`ViewState`] |
//! | `TGroup`             | [`Group`]                              |
//! | `TProgram`/`TApplication` | [`Program`] + трейт [`AppHandler`] |
//! | `TWindow`, `TFrame`  | [`Window`], [`Frame`]                  |
//! | `TDesktop`, `TBackground` | [`Desktop`], [`Background`]       |
//! | `TMenuBar`, `TMenuBox`, `TMenu`, `TMenuItem` | [`MenuBar`], [`Menu`], [`MenuItem`] |
//! | `TStatusLine`, `TStatusDef`, `TStatusItem` | [`StatusLine`], [`StatusDef`], [`StatusItem`] |
//! | `TDialog`            | [`Dialog`]                             |
//! | `TButton`            | [`Button`]                             |
//! | `TStaticText`, `TLabel` | [`StaticText`], [`Label`]           |
//! | `TInputLine`         | [`InputLine`]                          |
//! | `TCheckBoxes`, `TRadioButtons` | [`CheckBoxes`], [`RadioButtons`] |
//! | `TScrollBar`         | [`ScrollBar`]                          |
//! | `TListViewer`, `TListBox` | [`ListBox`]                       |
//! | `messageBox()`, `inputBox()` | [`message_box`], [`input_box`] |
//! | `TValidator` и наследники | трейт [`Validator`], [`PictureValidator`], [`RangeValidator`] ... |
//! | `THistory`, `THistoryWindow` | [`History`], [`HistoryWindow`] |
//! | `TScroller`, `TTerminal` | [`Scroller`], [`Terminal`] |
//! | `TEditor`, `TMemo`, `TFileEditor`, `TEditWindow`, `TIndicator` | [`Editor`], [`Memo`], [`FileEditor`], [`EditWindow`], [`Indicator`] |
//! | `TFileDialog`, `TChDirDialog` | [`FileDialog`], [`ChDirDialog`] |
//!
//! ## Главное архитектурное отличие от C++
//!
//! В C++ каждое представление (`TView`) хранит сырой указатель на владельца
//! (`owner`) и может напрямую вызывать его методы: `owner->setCurrent(this)`,
//! `message(owner, ...)`, `destroy(this)` и т.п. В Rust такая циклическая
//! структура ссылок неудобна и небезопасна, поэтому:
//!
//! * представления **не знают своего владельца**;
//! * всё, что в C++ делалось через `owner`, в Rust делается через контекст
//!   [`Ctx`], который передаётся в `handle_event`. Например, `ctx.select(id)`
//!   ставит в «почтовый ящик» запрос, который владелец обработает сразу после
//!   возврата из `handle_event` дочернего представления;
//! * отрисовка всегда идёт целиком в теневой буфер, а в терминал выводятся
//!   только изменившиеся ячейки — это заменяет сложную систему
//!   `drawView()/exposed()` оригинала;
//! * вложенные модальные циклы (`execView`) запускаются из [`Program`]
//!   (обычно из обработчика приложения [`AppHandler`]).

pub mod color;
pub mod drawbuf;
pub mod event;
pub mod geometry;
pub mod group;
pub mod platform;
pub mod program;
pub mod text;
pub mod view;
pub mod widgets;

pub use color::{Attr, AttrPair};
pub use drawbuf::{Cell, DrawBuffer, ScreenBuffer};
pub use event::{cm, kb, Event, Info, Key, KeyEvent, Message, Mods, MouseEvent};
pub use geometry::{Point, Rect};
pub use group::Group;
pub use program::{AppHandler, Program};
pub use view::{dm, gf, of, sf, Ctx, DrawCtx, Phase, Value, View, ViewId, ViewState};
pub use widgets::*;
