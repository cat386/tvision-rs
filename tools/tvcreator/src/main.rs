//! TVCreator — дизайнер форм для tvision-rs.
//!
//! ```text
//! tvcreator                      открыть среду с новой формой
//! tvcreator форма.tvui           открыть форму
//! tvcreator --gen форма.tvui [-o файл.rs]
//!                                только сгенерировать код (как uic) — для build.rs и скриптов
//! tvcreator --gen-include форма.tvui [-o файл.rs]
//!                                вариант для include! (без #![…] и //!)
//! ```

use std::path::{Path, PathBuf};

use tvcreator::codegen;
use tvcreator::model::FormDoc;

fn usage() -> ! {
    eprintln!("TVCreator — дизайнер форм для tvision-rs\n");
    eprintln!("  tvcreator [форма.tvui]                    среда разработки");
    eprintln!("  tvcreator --gen форма.tvui [-o файл.rs]           сгенерировать код Rust");
    eprintln!("  tvcreator --gen-include форма.tvui [-o файл.rs]   вариант для include!");
    std::process::exit(2)
}

fn load_form(input: &Path) -> Result<FormDoc, String> {
    let doc = FormDoc::load(input)?;
    if !doc.problems().is_empty() {
        return Err(format!("{}:\n  {}", input.display(), doc.problems().join("\n  ")));
    }
    Ok(doc)
}

fn parse_gen_args(args: &[String]) -> (&str, Option<PathBuf>) {
    let input = args.get(1).map(String::as_str).unwrap_or("");
    let output = match (args.get(2).map(String::as_str), args.get(3)) {
        (Some("-o"), Some(o)) => Some(PathBuf::from(o)),
        _ => None,
    };
    (input, output)
}

fn generate(input: &Path, output: Option<PathBuf>, for_include: bool) -> Result<PathBuf, String> {
    let doc = load_form(input)?;
    let out = output.unwrap_or_else(|| codegen::output_path(input, &doc));
    let src = input.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let code = if for_include { codegen::generate_for_include(&doc, &src) } else { codegen::generate(&doc, &src) };
    std::fs::write(&out, code).map_err(|e| format!("{}: {}", out.display(), e))?;
    Ok(out)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if matches!(args.first().map(String::as_str), Some("--gen") | Some("--gen-include")) {
        let for_include = args.first().map(String::as_str) == Some("--gen-include");
        let (input, output) = parse_gen_args(&args);
        if input.is_empty() {
            usage();
        }
        match generate(Path::new(input), output, for_include) {
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
