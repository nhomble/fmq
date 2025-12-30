use clap::Parser;
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::PathBuf;
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
        if !fmq::is_mutation(&args.expr) {
            eprintln!("error: --in-place requires a mutation expression (use = or |= or del)");
            process::exit(1);
        }

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

        let output = fmq::fmq(&args.expr, &content, args.init).unwrap_or_else(|e| {
            eprintln!("error: {e}");
            process::exit(1);
        });

        fs::write(path, output).unwrap_or_else(|e| {
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
