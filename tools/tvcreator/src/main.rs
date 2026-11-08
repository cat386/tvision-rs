//! TVCreator — дизайнер форм для tvision-rs.
//!
//! ```text
//! tvcreator                      открыть среду с новой формой
//! tvcreator форма.tvui           открыть форму
//! tvcreator --gen форма.tvui [-o файл.rs]
//!                                только сгенерировать код (как uic) — для build.rs и скриптов
//! ```

use std::path::{Path, PathBuf};

use tvcreator::codegen;
use tvcreator::model::FormDoc;

fn usage() -> ! {
    eprintln!("TVCreator — дизайнер форм для tvision-rs\n");
    eprintln!("  tvcreator [форма.tvui]                    среда разработки");
    eprintln!("  tvcreator --gen форма.tvui [-o файл.rs]   сгенерировать код Rust");
    std::process::exit(2)
}

fn generate(input: &Path, output: Option<PathBuf>) -> Result<PathBuf, String> {
    let doc = FormDoc::load(input)?;
    let problems = doc.problems();
    if !problems.is_empty() {
        return Err(format!("{}:\n  {}", input.display(), problems.join("\n  ")));
    }
    let out = output.unwrap_or_else(|| codegen::output_path(input, &doc));
    let src = input.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    std::fs::write(&out, codegen::generate(&doc, &src)).map_err(|e| format!("{}: {}", out.display(), e))?;
    Ok(out)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--gen") {
        let Some(input) = args.get(1) else { usage() };
        let output = match (args.get(2).map(String::as_str), args.get(3)) {
            (Some("-o"), Some(o)) => Some(PathBuf::from(o)),
            (None, _) => None,
            _ => usage(),
        };
        match generate(Path::new(input), output) {
            Ok(out) => println!("{}", out.display()),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        return;
    }
    if args.iter().any(|a| a.starts_with('-')) {
        usage();
    }
    if let Err(e) = tvcreator::ide::run(args.first().map(PathBuf::from)) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
