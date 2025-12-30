use clap::Parser;
use std::fs::File;
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
}

fn main() {
    let args = Args::parse();

    let result = match &args.file {
        Some(path) => {
            let file = File::open(path).unwrap_or_else(|e| {
                eprintln!("error: {e}");
                process::exit(1);
            });
            fmq::fmq_reader(&args.expr, BufReader::new(file))
        }
        None => {
            let stdin = io::stdin().lock();
            fmq::fmq_reader(&args.expr, stdin)
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
