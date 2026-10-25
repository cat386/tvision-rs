//! Стандартные представления Turbo Vision.

pub mod background;
pub mod button;
pub mod cluster;
pub mod colorsel;
pub mod combo;
pub mod desktop;
pub mod dialog;
pub mod editor;
pub mod filedlg;
pub mod frame;
pub mod help;
pub mod history;
pub mod inputline;
pub mod inputlong;
pub mod listbox;
pub mod menu;
pub mod msgbox;
pub mod outline;
pub mod progress;
pub mod scrollbar;
pub mod scroller;
pub mod statictext;
pub mod statusline;
pub mod table;
pub mod validate;
pub mod window;

pub use background::Background;
pub use button::{bf, Button};
pub use cluster::{cf, CheckBoxes, Cluster, ClusterKind, MultiCheckBoxes, RadioButtons};
pub use colorsel::{
    standard_color_groups, Color256Dialog, Color256Selector, ColorDialog, ColorDisplay, ColorGroup, ColorItem, ColorSel, ColorSelector,
};
pub use combo::{cbx, Combo, ComboWindow};
pub use desktop::Desktop;
pub use dialog::{Dialog, DialogPalette};
pub use editor::{ef, EditWindow, Editor, FileEditor, Indicator, Memo, Pos};
pub use filedlg::{cd, fd, ChDirDialog, DirListBox, FileDialog, FileEntry, FileInfoPane, FileList};
pub use frame::Frame;
pub use help::{HelpFile, HelpTopic, HelpViewer, HelpWindow};
pub use history::{history_add, history_count, history_str, History, HistoryWindow};
pub use inputline::InputLine;
pub use inputlong::{il, parse_long, InputLong};
pub use listbox::ListBox;
pub use menu::{Menu, MenuBar, MenuItem};
pub use msgbox::{build_message_box, input_box, message_box, message_box_rect, mf};
pub use outline::{Node, Outline};
pub use progress::{ProgressBar, ProgressStyle, PROGRESS_PALETTE};
pub use scrollbar::ScrollBar;
pub use scroller::{Scroller, Terminal};
pub use statictext::{ColoredText, Label, StaticText};
pub use statusline::{StatusDef, StatusItem, StatusLine};
pub use table::{
    column_letters, natural_cmp, Align, FnTableModel, SelectionMode, Table, TableColors, TableColumn, TableModel, VecTableModel, DEFAULT_TABLE_COLORS,
};
pub use validate::{FilterValidator, FnValidator, PictureValidator, RangeValidator, StringLookupValidator, Validator};
pub use window::{wf, Window};
