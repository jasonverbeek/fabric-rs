use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use colored::Colorize;
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, FabricError>;

pub enum FabricError {
    NoFabricProject,
    NoAccessToFile,
    InvalidFabric,
    ExecutionError,
}

impl FabricError {
    pub fn value(&self) -> &'static str {
        match *self {
            Self::NoFabricProject => "Current directory is not a fabric project",
            Self::NoAccessToFile => ".fabric file exists but cannot be read.",
            Self::InvalidFabric => ".fabric file cannot be parsed. JSON not in expected structure.",
            Self::ExecutionError => "Command failed to execute",
        }
    }
}

impl fmt::Display for FabricError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", "ERR".red().bold(), self.value())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Instruction {
    pub name: String, // name of command to be available in project: fabric <name>
    #[serde(default = "bool::default")]
    private: bool, // wether or not to show the command in help. default false
    pub command: Option<String>, // command to run
    pub args: Option<Vec<String>>, // args to that command
    pub subfabrics: Option<Vec<String>>, // combine multiple Instructions, used instead of command+args
}

impl Instruction {
    pub fn is_composed(&self) -> bool {
        if let Some(_) = &self.subfabrics {
            return true;
        }
        return false;
    }

    pub fn explain(&self) -> String {
        if let Some(subfabrics) = &self.subfabrics {
            return format!("(composition: {})", subfabrics.join(" "));
        } else if let (Some(command), Some(args)) = (&self.command, &self.args) {
            return format!("({} {})", command, args.join(" "));
        } else {
            return String::from("(unknown)");
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fabric {
    pub fabrics: Vec<Instruction>,
}

impl Fabric {
    pub fn load_project(project_file: &'_ str) -> Result<Self> {
        if !Path::new("./.fabric").exists() {
            return Err(FabricError::NoFabricProject);
        }

        let mut file = File::open(project_file).or(Err(FabricError::NoAccessToFile))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .or(Err(FabricError::NoAccessToFile))?;

        let fabric: Fabric =
            serde_json::from_str(content.as_str()).or(Err(FabricError::InvalidFabric))?;

        // Validate subfabrics on .fabric file
        for f in &fabric.fabrics {
            if let Some(subfabrics) = &f.subfabrics {
                for sf in subfabrics {
                    if let None = fabric.find_by_name(sf) {
                        return Err(FabricError::InvalidFabric);
                    }
                }
            }
        }

        Ok(fabric)
    }

    pub fn find_by_name(&self, name: &String) -> Option<&Instruction> {
        self.fabrics.iter().find(|f| f.name.eq(name))
    }

    fn expand_all(&self, tasks: &Vec<String>) -> Vec<&Instruction> {
        let raw_instructions = tasks
            .into_iter()
            .filter_map(|t| -> Option<&Instruction> { return self.find_by_name(&t.to_string()) });

        let mut expanded_instructions: Vec<&Instruction> = Vec::new();
        for ri in raw_instructions {
            if !ri.is_composed() {
                expanded_instructions.push(ri);
                continue;
            } else {
                if let Some(instructions) = self.expand(ri) {
                    expanded_instructions.extend(instructions)
                }
            }
        }
        expanded_instructions
    }

    pub fn expand(&self, instruction: &Instruction) -> Option<Vec<&Instruction>> {
        let subfabrics = match &instruction.subfabrics {
            None => return None,
            Some(subfabrics) => subfabrics,
        };
        let mut expanded: Vec<&Instruction> = Vec::new();
        for sf in subfabrics {
            if let Some(instr) = self.find_by_name(&sf) {
                expanded.push(instr);
            }
        }
        return Some(expanded);
    }

    pub fn execute_instruction(instruction: &Instruction) -> Result<()> {
        Ok(())
    }
    pub fn execute_all(&self, raw_tasks: &Vec<String>) -> Result<()> {
        let tasks: Vec<&Instruction> = self.expand_all(raw_tasks);
        for task in tasks {
            println!(
                "{}: {} {}",
                "RUN".green().bold(),
                &task.name,
                task.explain().italic()
            );
            Self::execute_instruction(task)?;
        }
        Ok(())
    }
}
