//use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::process::exit;
use std::{fmt, io::Read};

use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json;

const VERSION: &'static str = "0.0.1";

type Result<T> = std::result::Result<T, FabricError>;

enum FabricError {
    NoFabricProject,
    NoAccessToFile,
    InvalidFabric,
}

impl FabricError {
    pub fn value(&self) -> &'static str {
        match *self {
            Self::NoFabricProject => "Current directory is not a fabric project",
            Self::NoAccessToFile => ".fabric file exists but cannot be read.",
            Self::InvalidFabric => ".fabric file cannot be parsed. JSON not in expected structure.",
            _ => "Something went so wrong that I don't event know what!",
        }
    }
}

impl fmt::Display for FabricError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", "ERR".red().bold(), self.value())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Instruction {
    name: String, // name of command to be available in project: fabric <name>
    #[serde(default = "bool::default")]
    private: bool, // wether or not to show the command in help. default false
    command: Option<String>, // command to run
    args: Option<Vec<String>>, // args to that command
    subfabrics: Option<Vec<String>>, // combine multiple Instructions, used instead of command+args
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Fabric {
    fabrics: Vec<Instruction>,
}

impl Fabric {
    fn _execute(_instructions: &Instruction) {}

    fn find_by_name(&self, name: &String) -> Option<&Instruction> {
        self.fabrics.iter().find(|f| f.name.eq(name))
    }
}

fn load_fabric_project() -> Result<Fabric> {
    if !Path::new("./.fabric").exists() {
        return Err(FabricError::NoFabricProject);
    }

    let mut file = File::open("./.fabric").or(Err(FabricError::NoAccessToFile))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .or(Err(FabricError::NoAccessToFile))?;

    let fabric: Fabric =
        serde_json::from_str(content.as_str()).or(Err(FabricError::InvalidFabric))?;

    let mut expected_fabrics: Vec<&Instruction> = Vec::new();
    for f in &fabric.fabrics {
        if let Some(subfabrics) = &f.subfabrics {
            for sf in subfabrics {
                if let Some(instr) = fabric.find_by_name(sf) {
                    expected_fabrics.push(instr);
                }
            }
        }
    }

    println!("{:#?}", expected_fabrics);

    for f in &fabric.fabrics {
        println!("FABRIC: {}", f.name);
    }

    Ok(fabric)
}

fn main() {
    println!("Fabric {}", VERSION);
    let _fabric: Fabric = match load_fabric_project() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}", e);
            exit(1)
        }
    };
}
