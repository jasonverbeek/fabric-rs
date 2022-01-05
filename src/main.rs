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

    // basic args parsing
    // no need for clap crate
    // 0 args = help
    // any args = attempt execute all args
    if args.len() == 0 {
        println!("Available commands:");
        println!("{} - show this help page", "fabric".bold());
        for instr in &fabric.fabrics {
            // do not show help for private fabrics
            if &instr.private == &true {
                continue;
            }
            println!(
                "{} {} - {}",
                "fabric".bold(),
                &instr.name.bold(),
                instr.explain()
            );
        }
        // exit so we do not print the "done" message
        exit(0);
    } else {
        // start executing all args in order
        fabric.execute_all(&args)?;
    }
    Ok(())
}

fn main() {
    println!("Fabric version {}", VERSION);
    println!();

    // attempt to fabricate the project and collect/handle errors in one place
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
