use std::env;
use std::fs;
use std::path::Path;

use clap::Parser;
use colored::Colorize;

// TODO:
// instead of panic do nice colored prints.
// perhaps also use clap in fabric and combine funcionality
//      however may prove difficult with dynamic arguments

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const TEMPLATE: &'static str = include_str!("../template.fabric.json");
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Arguments {
    #[clap(default_value = "./")]
    filename: String,
}

fn main() {
    println!("{} version {}", "mk-fabric".green(), VERSION.yellow());
    println!();
    let args = Arguments::parse();
    let path = Path::new(&args.filename);
    if !path.exists() {
        eprintln!("{} '{}' doesn't exist", "ERR".red(), args.filename);
        return;
    }
    if path.is_file() {
        eprintln!(
            "{} '{}', is a file, not a directory",
            "ERR".red(),
            args.filename
        );
        return;
    }
    let fabric_file = path.join(".fabric"); // format!("{}/.fabric", &args.filename);
    let fabric_file_str = fabric_file.to_str().unwrap();
    if Path::new(&fabric_file).exists() {
        eprintln!("{} This directory already has a .fabric file", "ERR".red());
        return;
    }
    fs::write(&fabric_file, TEMPLATE).unwrap_or_else(|_| {
        eprintln!("{} Unable to write to '{}'", "ERR".red(), fabric_file_str);
        return;
    });
    println!(
        "{} Created .farbic file at '{}'",
        "OK".green().bold(),
        fabric_file_str
    );
}
