use crate::model::{Pipeline, Step, StepStatus};
use crate::state::RunState;

#[derive(Debug, PartialEq)]
pub enum Command {
    Help,
    Version,
    Plan(Option<String>),
    Show(String),
    Simulate(Option<String>),
    Invalid(String),
    Locked(String, u8),
}

pub const HELP_COMMANDS: [(&str, &str, u8, bool); 7] = [
    ("help", "this help", 0, true),
    ("version", "program version", 0, true),
    ("plan", "pipeline plan", 1, true),
    ("validate", "validate cinder.yaml", 5, false),
    ("graph", "dependecy graph", 6, false),
    ("run", "start pipeline", 7, false),
    ("tui", "fullscreen tui", 17, false),
];

pub fn print_help() {
    println!("cinderci - local CI/CD, which is building itself 🔥.\n");
    println!("Commands:");
    for (name, desc, etap, avail) in HELP_COMMANDS {
        if avail {
            println!("   {:<10}    {:<30} ({:<02})", name, desc, etap);
        } else {
            println!("   {:<10} 🔒 {:<30} ({:<02})", name, desc, etap)
        }
    }
}

pub fn print_plan(pipeline: &Pipeline) {
    let mut triggers: Vec<String> = vec![];
    for t in &pipeline.trigger {
        triggers.push(t.to_string());
    }
    println!(
        "Pipeline: {}   (trigger: {})",
        pipeline.name,
        triggers.join(", ")
    );
    println!();

    let pending = StepStatus::Pending;
    let running = StepStatus::Running;
    let succeed = StepStatus::Succeeded(std::time::Duration::from_millis(12_400));
    let failed = StepStatus::Failed(1);
    let skipped = StepStatus::Skipped("dependency 'test' crashed".to_string());

    for s in &pipeline.steps {
        println!("{}   {:<8}     {}", pending, s.name, s.command);
        if !Step::is_root(s) {
            println!("                 ↳ needs: {}", s.dependencies.join(", "));
            if let Some(condition) = &s.condition {
                println!("                 ↳ condition:{} ", condition);
            }
        };
    }

    println!();
    println!(
        "{} steps · {} without dependencies · {} conditional",
        Pipeline::steps_amount(pipeline),
        Pipeline::roots_amount(pipeline),
        Pipeline::conditions_amount(pipeline)
    );

    println!(
        "Legends: {} · {} · {} · {} · {}",
        pending, running, succeed, failed, skipped
    );
}

pub fn print_step(step: &Step, pipeline: &Pipeline) {
    println!("Step: {}", step.name);
    println!("  command:      {}", step.command);

    if step.is_root() {
        println!("  needs:        - (root)");
    } else {
        println!("  needs:        {}", step.dependencies.join(", "));
    }

    let dependents = pipeline.dependents_of(&step.name);

    println!(
        "  needed by:    {}",
        if dependents.is_empty() {
            "-".to_string()
        } else {
            dependents.join(", ")
        }
    );

    if let Some(condition) = &step.condition {
        println!("  condition:    {}", condition);
    }
}

pub fn print_simulation(pipeline: &Pipeline, state: &RunState) {
    let mut success = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for step in &pipeline.steps {
        if let Some(status) = state.statuses.get(&step.name) {
            println!("  {:<8}{}", step.name, status);
            match status {
                StepStatus::Succeeded(_s) => success += 1,
                StepStatus::Failed(_i) => failed += 1,
                StepStatus::Skipped(_s) => skipped += 1,
                StepStatus::Running | StepStatus::Pending => (),
            }
        }
    }
    println!();
    println!(
        "✓ Succeeded: {}, ✗ Failed: {}, ○ Skipped: {}",
        success, failed, skipped
    );
}

pub fn read_commands(command: &[String]) -> Command {
    match command.first().map(String::as_str) {
        Some("-h" | "help" | "--help") | None => Command::Help,
        Some("-V" | "version" | "--version") => Command::Version,
        Some("-p" | "plan" | "--plan") => match command.get(1).map(String::as_str) {
            None => Command::Plan(None),
            Some(name) => Command::Plan(Some(name.to_string())),
        },
        Some("-s" | "show" | "--show") => match command.get(1).map(String::as_str) {
            Some(name) => Command::Show(name.to_string()),
            None => Command::Invalid("Usage: cinderci show <step>".to_string()),
        },
        Some("simulate" | "--simulate") => match command.get(1).map(String::as_str) {
            Some("fail" | "--fail") => match command.get(2).map(String::as_str) {
                Some(name) => Command::Simulate(Some(name.to_string())),
                None => Command::Invalid("Usage: cinderci simulate --fail <step>".to_string()),
            },
            None => Command::Simulate(None),
            Some(name) => Command::Invalid(format!(
                "Error: Unknown option '{}'\nUsage: cinderci simulate [--fail <step>]",
                name
            )),
        },
        Some(command_name) => {
            for (name, _description, stage, available) in HELP_COMMANDS {
                if command_name == name && !available {
                    return Command::Locked(name.to_string(), stage);
                }
            }

            Command::Invalid(format!(
                "Command '{}' not available. Use 'cinderci help' to see available commands.",
                command_name
            ))
        }
    }
}
