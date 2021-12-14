//use std::error::Error;
mod fabric;

use std::env;
use std::process::exit;

use colored::Colorize;
use fabric::{Fabric, Result};

const VERSION: &'static str = "0.0.1";

fn fabricate() -> Result<()> {
    let fabric: Fabric = Fabric::load_project("./.fabric")?;
    let args: Vec<String> = env::args().skip(1).collect();

    let x = fabric.execute_all(&args);
    let z = x?;
    Ok(())
}

fn main() {
    println!("Fabric {}", VERSION);
    match fabricate() {
        Ok(_) => {
            println!("{}: Done!", "FIN".green().bold());
        }
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    }
}
