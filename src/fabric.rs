use std::{fmt, fs::File, io::Read, path::Path, process::Command};

use colored::Colorize;
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, FabricError>;

pub enum FabricError {
    NoFabricProject,
    NoAccessToFile,
    InvalidFabric,
    InvalidSubFabric,
    ExecutionError,
}

impl FabricError {
    // map enum to a basic error message
    // NOTE that this doesn't allow for adding variables or custom text
    pub fn value(&self) -> &'static str {
        match *self {
            Self::NoFabricProject => "Current directory is not a fabric project",
            Self::NoAccessToFile => ".fabric file exists but cannot be read.",
            Self::InvalidFabric => ".fabric file cannot be parsed. JSON not in expected structure.",
            Self::InvalidSubFabric => "One or more of the subfabrics in .fabric doesnt exist",
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
    pub private: bool, // wether or not to show the command in help. default false
    pub command: Option<String>, // command to run
    pub args: Option<Vec<String>>, // args to that command
    pub subfabrics: Option<Vec<String>>, // combine multiple Instructions, used instead of command+args
}

impl Instruction {
    // simple helper to do a check
    pub fn is_composed(&self) -> bool {
        self.subfabrics.is_some()
    }

    pub fn explain(&self) -> String {
        // helper to attempt to explain a fabric
        // the fact I need to check vars to determine what kind it is somewhat wrong, there has to be something for this in Rust
        if let Some(subfabrics) = &self.subfabrics {
            return format!("composition of commands: {}", subfabrics.join(", "));
        } else if let (Some(command), Some(args)) = (&self.command, &self.args) {
            return format!("{} {}", command, args.join(" "));
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
        // check if .fabric file exists
        if !Path::new(project_file).exists() {
            return Err(FabricError::NoFabricProject);
        }

        // read .fabric file into memory
        let mut file = File::open(project_file).or(Err(FabricError::NoAccessToFile))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .or(Err(FabricError::NoAccessToFile))?;

        // decode json to a Fabric object/instance
        let fabric: Fabric =
            serde_json::from_str(content.as_str()).or(Err(FabricError::InvalidFabric))?;

        // Validate subfabrics on .fabric file
        for f in &fabric.fabrics {
            if let Some(subfabrics) = &f.subfabrics {
                for sf in subfabrics {
                    if let None = fabric.find_by_name(sf) {
                        return Err(FabricError::InvalidSubFabric);
                    }
                }
            }
        }

        Ok(fabric)
    }

    pub fn find_by_name(&self, name: &String) -> Option<&Instruction> {
        // search through all fabrics returning an Option with the matching instruction if found
        self.fabrics.iter().find(|f| f.name.eq(name))
    }

    fn expand_all(&self, tasks: &Vec<String>) -> Vec<&Instruction> {
        // This function loops over all instructions and expands those that are subfabrics in the order of the args/tasks
        // turning `subfabrics: ["a", "b"]` into 2 seperate tasks (a and b)

        // map args/tasks into their respective Instructions
        let raw_instructions = tasks
            .into_iter()
            .filter_map(|t| -> Option<&Instruction> { return self.find_by_name(t) });

        // build a vec in the order of the args/tasks with expanded subfabrics
        let mut expanded_instructions: Vec<&Instruction> = Vec::new();
        for ri in raw_instructions {
            if !ri.is_composed() {
                expanded_instructions.push(ri);
            } else {
                if let Some(instructions) = self.expand(ri) {
                    expanded_instructions.extend(instructions)
                }
            }
        }
        //return that build vec
        expanded_instructions
    }

    pub fn expand(&self, instruction: &Instruction) -> Option<Vec<&Instruction>> {
        // expand a single instruction which is a collection of instructions into a ordered list of that collection

        // if instruction does not have subfrabrics return None because nothing can be expanded
        let subfabrics = match &instruction.subfabrics {
            None => return None,
            Some(subfabrics) => subfabrics,
        };
        // build expanded vec of instruction based on the names of the subfabrics
        let mut expanded: Vec<&Instruction> = Vec::new();
        for sf in subfabrics {
            if let Some(instr) = self.find_by_name(&sf) {
                expanded.push(instr);
            }
        }
        // return result
        return Some(expanded);
    }

    pub fn execute_instruction(instruction: &Instruction) -> Result<()> {
        // execute a single Instruction on the system
        if let (Some(command), Some(args)) = (&instruction.command, &instruction.args) {
            // spawn process
            let mut cmd = Command::new(command)
                .args(args)
                .spawn()
                .or(Err(FabricError::ExecutionError))?;
            // check wether exit was successful, if not stop execution of all Instructions
            let exitcode = cmd.wait().or(Err(FabricError::ExecutionError))?;
            if !exitcode.success() {
                return Err(FabricError::ExecutionError);
            }
        }
        // execution finished
        Ok(())
    }

    pub fn execute_all(&self, raw_tasks: &Vec<String>) -> Result<()> {
        // execute all instructions gathered based on the args/Tasks
        let tasks: Vec<&Instruction> = self.expand_all(raw_tasks);
        for task in tasks {
            println!(
                "{} {} ({})",
                "RUN".blue().bold(),
                &task.name,
                task.explain().italic()
            );
            Self::execute_instruction(task)?;
        }
        Ok(())
    }
}
