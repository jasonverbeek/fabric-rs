//use std::error::Error;
mod fabric;

use std::env;
use std::process::exit;

use colored::Colorize;
use fabric::{Fabric, Result};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn fabricate() -> Result<()> {
    let fabric: Fabric = Fabric::load_project("./.fabric")?;
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() == 0 {
        println!("Available commands:");
        println!("{} - show this help page", "fabric".bold());
        for instr in &fabric.fabrics {
            println!(
                "{} {} - {}",
                "fabric".bold(),
                &instr.name.bold(),
                instr.explain()
            );
        }
        exit(0);
    }

    fabric.execute_all(&args)?;
    Ok(())
}

fn main() {
    println!("Fabric version {}", VERSION);
    println!();
    match fabricate() {
        Ok(_) => {
            println!("{}: All fabrics completed!", "FIN".green().bold());
        }
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    }
}
