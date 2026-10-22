use std::{env, process::ExitCode};

use cinderci::cli::*;
use cinderci::model::Pipeline;
use cinderci::sim::*;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    eprintln!("{:?}", read_commands(&args));

    let mut args = env::args().skip(1);

    let command = args.next();
    let arg2 = args.next();
    let arg3 = args.next();

    match command.as_deref() {
        Some("-V" | "version" | "--version") => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        }

        Some("-h" | "help" | "--help") | None => {
            print_help();
        }

        Some("-p" | "plan" | "--plan") => match arg2.as_deref() {
            Some("example") => {
                print_plan(&Pipeline::example());
            }

            None => {
                print_plan(&Pipeline::cinderci());
            }

            Some(pipeline_name) => {
                eprintln!(
                    "Error: Unknown pipeline: '{}' (available: cinderci, example)",
                    pipeline_name
                );
                return ExitCode::from(2);
            }
        },

        Some("-s" | "show" | "--show") => {
            let pipeline = Pipeline::cinderci();

            match arg2.as_deref() {
                Some(step_name) => match pipeline.find_step(step_name) {
                    Some(step) => print_step(step, &pipeline),

                    None => {
                        eprintln!(
                            "Error: no step '{}' (available: {})",
                            step_name,
                            pipeline.step_names()
                        );
                        return ExitCode::from(2);
                    }
                },

                None => {
                    eprintln!("Usage: cinderci show <step>");
                    return ExitCode::from(2);
                }
            }
        }

        Some("simulate" | "--simulate") => {
            let pipeline = Pipeline::cinderci();

            match arg2.as_deref() {
                None => {
                    let state = simulate(&pipeline, None);
                    print_simulation(&pipeline, &state);
                }

                Some("fail" | "--fail") => match arg3.as_deref() {
                    Some(step_name) => {
                        if pipeline.find_step(step_name).is_none() {
                            eprintln!(
                                "Error: no step '{}' (available: {})",
                                step_name,
                                pipeline.step_names()
                            );
                            return ExitCode::from(2);
                        }

                        let state = simulate(&pipeline, Some(step_name));
                        print_simulation(&pipeline, &state);
                    }

                    None => {
                        eprintln!("Usage: cinderci simulate --fail <step>");
                        return ExitCode::from(2);
                    }
                },

                Some(option) => {
                    eprintln!("Error: Unknown option '{}'", option);
                    eprintln!("Usage: cinderci simulate [--fail <step>]");
                    return ExitCode::from(2);
                }
            }
        }

        Some(command_name) => {
            for (name, _description, stage, available) in HELP_COMMANDS {
                if command_name == name && !available {
                    eprintln!(
                        "Command {} not available, will be implemented at stage {}.",
                        name, stage
                    );
                    return ExitCode::from(2);
                }
            }

            eprintln!(
                "Command '{}' not available. Use 'cinderci help' to see available commands.",
                command_name
            );
            return ExitCode::from(2);
        }
    }

    ExitCode::SUCCESS
}
