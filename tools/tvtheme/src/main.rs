//! Редактор тем для tvision-rs.
//!
//! ```text
//! tvtheme                 новая тема (на основе классической палитры)
//! tvtheme тема.tvtheme    открыть тему (или создать файл при сохранении)
//! ```

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a.starts_with('-')) || args.len() > 1 {
        eprintln!("Редактор тем tvision-rs\n\n  tvtheme [тема.tvtheme]");
        std::process::exit(2);
    }
    if let Err(e) = tvtheme::editor::run(args.first().map(std::path::PathBuf::from)) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
