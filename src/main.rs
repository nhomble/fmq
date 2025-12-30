use clap::Parser;
use std::fs;
use std::io::{self, Read};
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

    let markdown = match read_input(args.file.as_ref()) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

    match fmq::fmq(&args.expr, &markdown) {
        Ok(output) => print!("{output}"),
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    }
}

fn read_input(file: Option<&PathBuf>) -> io::Result<String> {
    match file {
        Some(path) => fs::read_to_string(path),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}
