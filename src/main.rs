mod file_ops;

use clap::Parser;
use std::{fs::File, io::Write};

#[derive(Debug, Parser)]
#[clap(name = "sql-merge", version = "0.1.0", author = "kaito.suzuki")]
#[command(
    about = "-d で指定したディレクトリ直下のSQLファイル(exec_*.sql は除く)をマージして -o で指定したファイルに出力します"
)]
struct Args {
    #[arg(short, long, help = "directory path")]
    directory: String,

    #[arg(short, long, help = "output file path")]
    output_file_path: String,
}

fn main() {
    let args = Args::parse();

    let merged = match file_ops::merge_files(args.directory.as_str()) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error merging files: {}", e);
            return;
        }
    };

    let mut output_file =
        File::create(args.output_file_path).expect("Could not create output file");
    output_file
        .write_all(merged.as_bytes())
        .expect("Could not write to output file");
}
