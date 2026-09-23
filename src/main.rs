use clap::Parser;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process;

#[derive(Parser)]
#[command(name = "fmq")]
#[command(about = "jq for markdown frontmatter")]
struct Args {
    /// jq expression
    expr: String,

    /// Input file (reads stdin if omitted)
    file: Option<PathBuf>,

    /// Treat missing frontmatter as empty (allows initializing frontmatter)
    #[arg(long)]
    init: bool,

    /// Edit file in place
    #[arg(short, long)]
    in_place: bool,
}

fn main() {
    let args = Args::parse();

    if args.in_place {
        let path = match &args.file {
            Some(p) => p,
            None => {
                eprintln!("error: --in-place requires a file");
                process::exit(1);
            }
        };

        let content = fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("error: {e}");
            process::exit(1);
        });

        let output = fmq::fmq_document(&args.expr, &content, args.init).unwrap_or_else(|e| {
            eprintln!("error: {e}");
            process::exit(1);
        });

        write_atomic(path, output.as_bytes()).unwrap_or_else(|e| {
            eprintln!("error: {e}");
            process::exit(1);
        });
    } else {
        let result = match &args.file {
            Some(path) => {
                let file = File::open(path).unwrap_or_else(|e| {
                    eprintln!("error: {e}");
                    process::exit(1);
                });
                fmq::fmq_reader(&args.expr, BufReader::new(file), args.init)
            }
            None => {
                let stdin = io::stdin().lock();
                fmq::fmq_reader(&args.expr, stdin, args.init)
            }
        };

        match result {
            Ok(output) => print!("{output}"),
            Err(e) => {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }
}

/// Write `contents` to `path` atomically: write a sibling temp file, fsync,
/// then rename over the target. A crash mid-write leaves the original intact.
fn write_atomic(path: &Path, contents: &[u8]) -> io::Result<()> {
    let target = fs::canonicalize(path)?;
    let dir = target.parent().unwrap_or_else(|| Path::new("."));
    let name = target.file_name().unwrap_or_default().to_string_lossy();
    let tmp = dir.join(format!(".{name}.fmq-tmp.{}", process::id()));

    let result = (|| {
        let perms = fs::metadata(&target)?.permissions();
        let mut f = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
        f.write_all(contents)?;
        f.sync_all()?;
        drop(f);
        fs::set_permissions(&tmp, perms)?;
        fs::rename(&tmp, &target)
    })();

    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}
