//! Стандартные представления Turbo Vision.

pub mod background;
pub mod button;
pub mod cluster;
pub mod desktop;
pub mod dialog;
pub mod frame;
pub mod inputline;
pub mod listbox;
pub mod menu;
pub mod msgbox;
pub mod scrollbar;
pub mod statictext;
pub mod statusline;
pub mod window;

pub use background::Background;
pub use button::{bf, Button};
pub use cluster::{CheckBoxes, Cluster, ClusterKind, RadioButtons};
pub use desktop::Desktop;
pub use dialog::Dialog;
pub use frame::Frame;
pub use inputline::InputLine;
pub use listbox::ListBox;
pub use menu::{Menu, MenuBar, MenuItem};
pub use msgbox::{input_box, message_box, message_box_rect, mf};
pub use scrollbar::ScrollBar;
pub use statictext::{Label, StaticText};
pub use statusline::{StatusDef, StatusItem, StatusLine};
pub use window::{wf, Window};
