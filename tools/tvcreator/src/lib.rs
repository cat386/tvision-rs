//! TVCreator — дизайнер форм для tvision-rs в духе Qt Creator / Qt Designer.
//!
//! * [`model`] — документ формы и файл `.tvui`;
//! * [`build`] — сборка формы «на лету» (холст, предпросмотр);
//! * [`codegen`] — генерация Rust-кода (как `uic`);
//! * [`props`] — свойства узлов для редактора свойств;
//! * [`themes`] — темы среды и формы, настройки;
//! * [`ide`] — сама среда: палитра, дерево объектов, холст, свойства.

pub mod build;
pub mod canvas;
pub mod codegen;
pub mod ide;
pub mod model;
pub mod props;
pub mod themes;
