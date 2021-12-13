use std::io::Read;
use std::process::{exit, Command};
use std::{env, fs::File, path::Path};

use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json;

const VERSION: &'static str = "0.0.0.1";

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

fn check_fabric() -> Result<Fabric, String> {
    if !Path::new("./.fabric").exists() {
        return Err(format!(
            "{}: Current directory is not a Fabric project",
            "ERR".red().bold()
        ));
    }

    println!("{}: Found .fabric", "OK ".green().bold());

    let mut file = match File::open("./.fabric") {
        Ok(v) => v,
        _ => return Err(format!("{}: Could not open .fabric", "ERR".red().bold())),
    };

    println!("{}: Opened .fabric", "OK ".green().bold());

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => println!("{}: Read .fabric", "OK ".green().bold()),
        Err(_) => return Err(format!("{}: Could not read .fabric", "ERR".red().bold())),
    };
    match serde_json::from_str(contents.as_str()) {
        Ok(fabric) => {
            println!("{}: Serialized .fabric", "OK ".green().bold());
            Ok(fabric)
        }
        _ => Err(format!(
            "{}: Could not serialize .fabric.",
            "ERR".red().bold()
        )),
    }
}

fn process_instruction(fabric: &Fabric, command: &Instruction) {
    if command.command.is_some() && command.args.is_some() {
        let instruction = command.clone();
        println!();
        let mut cmd: std::process::Child = match Command::new(instruction.command.unwrap())
            .args(instruction.args.unwrap())
            .spawn()
        {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        };
        cmd.wait().unwrap();
    } else if command.subfabrics.is_some() {
        // TODO: subfabrics from the fabric object
    } else {
        println!();
        println!(
            "{}: Fabric '{}' is neither a command or collection of fabrics.",
            "ERR".red().bold(),
            command.name
        );
        exit(1)
    }
}

fn main() {
    println!("Fabric {}", VERSION);
    println!();
    let fabric: Fabric = match check_fabric() {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            exit(1)
        }
    };
    let all_args: Vec<String> = env::args().collect();
    let mut args: Vec<String> = Vec::from(&all_args[1..])
        .iter()
        .map(|i| i.to_lowercase())
        .collect();
    let available_cmds: Vec<Instruction> = fabric
        .clone()
        .fabrics
        .into_iter()
        .filter(|i| i.private == false)
        .collect();

    if args.len() == 0 {
        args.push("help".to_string());
    }

    if args.len() == 1 && args[0].eq("help") {
        println!();
        println!("Fabric commands:");
        println!("\t{}\t\t\t\tshow this info", "help".bold());
        //TODO: println!("\t{}\tshow what would be executed", "describe <project command>".bold());

        println!();
        println!("Available commands for this project:");
        available_cmds
            .clone()
            .into_iter()
            .map(|i| i.name)
            .for_each(|i| println!("\t{}", i.bold()));
        exit(0);
    }

    let unavailable_cmds: Vec<String> = args
        .clone()
        .into_iter()
        .filter(|arg| {
            !available_cmds
                .clone()
                .into_iter()
                .any(|ac| arg.eq(&ac.name))
        })
        .collect();

    if unavailable_cmds.len() > 0 {
        println!();
        println!(
            "{}: One or more commands are not accepted in this project:",
            "ERR".red().bold()
        );
        unavailable_cmds
            .iter()
            .for_each(|uac| println!("\t{}", uac.red().italic()));
        println!();
        println!(
            "{}: See `fabric help` for a list of available commands in this project.",
            "INF".cyan().bold()
        );
        exit(1);
    }

    let tasks: Vec<&Instruction> = args
        .clone()
        .into_iter()
        .filter_map(
            |a| match available_cmds.clone().iter().position(|ac| a.eq(&ac.name)) {
                Some(index) => Some(&available_cmds[index]),
                None => None,
            },
        )
        .collect();

    let mut tasks2: Vec<&Instruction> = Vec::new();
    args.clone().iter().for_each(|a| {
        match available_cmds.clone().iter().position(|ac| a.eq(&ac.name)) {
            Some(index) => {
                let instr: &Instruction = &available_cmds[index];
                if instr.command.is_some() && instr.args.is_some() {
                    tasks2.push(instr);
                } else {
                    // parse subfabrics if any, otherwise invalid .fabric file
                }
            }
            None => (),
        }
    });

    println!("TASKS: {:#?}", tasks);
    tasks
        .clone()
        .iter()
        .for_each(|task| process_instruction(&fabric, &task));
}
