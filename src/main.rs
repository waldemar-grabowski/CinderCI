use std::{env, process::ExitCode};

use cinderci::cli::*;
use cinderci::model::*;
use cinderci::sim::*;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    match read_commands(&args) {
        Command::Help => print_help(),
        Command::Version => println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        Command::Plan(Some(name)) => match name.as_str() {
            "example" => print_plan(&Pipeline::example()),
            _ => {
                eprintln!("Error: Unknown pipeline: '{}' (available: cinderci, example)", name);
                return ExitCode::from(2);
            }
        },
        Command::Show(name) => {
            let pipeline = Pipeline::cinderci();

            match pipeline.find_step(&name) {
                Some(step) => print_step(step, &pipeline),

                None => {
                    eprintln!(
                        "Error: no step '{}' (available: {})",
                        name,
                        pipeline.step_names()
                    );
                    return ExitCode::from(2);
                }
            }
        }
        Command::Simulate(None) => {
            let pipeline = Pipeline::cinderci();
            let state = simulate(&pipeline, None);
            print_simulation(&pipeline, &state);
        }
        Command::Simulate(Some(name)) => {
            let pipeline = Pipeline::cinderci();
            if pipeline.find_step(&name).is_none() {
                eprintln!(
                    "Error: no step '{}' (available: {})",
                    name,
                    pipeline.step_names()
                );
                return ExitCode::from(2);
            } else {
                let state = simulate(&pipeline, Some(&name));
                print_simulation(&pipeline, &state);
            }
        }
        Command::Invalid(command) => {
            eprintln!("{}", command);
            return ExitCode::from(2);
        }
        Command::Locked(text, _number) => {
            for (name, _description, stage, available) in HELP_COMMANDS {
                if text == name && !available {
                    eprintln!(
                        "Command {} not available, will be implemented at stage {}.",
                        name, stage
                    );
                    return ExitCode::from(2);
                }
            }
        }
        Command::Plan(None) => print_plan(&Pipeline::cinderci()),
    }

    ExitCode::SUCCESS
}
