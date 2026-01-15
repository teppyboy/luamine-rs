use clap::Parser;
use std::{fs::read_to_string, path::PathBuf};
pub mod minifier;

/// An experimental Lua(u) minifier built using full-moon
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to lua file
    #[arg(short, long)]
    file: String,
    /// Path to the output file, if not provided, prints to stdout
    #[arg(short, long)]
    output: Option<String>,
}

fn main() {
    println!("!!! NOT READY FOR PRODUCTION USE !!!");
    println!("Lumine is cute :3");
    let args = Args::parse();
    println!("Reading file {}...", args.file);
    let file = PathBuf::from(args.file);
    let text = read_to_string(file).expect("read input file error");
    let mut minifier = minifier::Minifier::new(&text);
    let result = minifier.minify();
    //println!("{:#?}", new_ast);
    println!("\n=== SCRIPT GENERATED ===\n");
    match args.output {
        Some(output_path) => {
            std::fs::write(output_path, result).expect("write output file error");
            println!("Written to output file.");
        }
        None => {
            println!("{}", result);
        }
    }
}
