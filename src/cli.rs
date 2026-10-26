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

pub fn format_help() -> String {
    let mut text: Vec<String> = vec![];
    text.push("cinderci - local CI/CD, which is building itself 🔥.".to_string());
    text.push("".to_string());
    text.push("Commands:".to_string());
    for (name, desc, etap, avail) in HELP_COMMANDS {
        if avail {
            text.push(format!("   {:<10}    {:<30} ({:<02})", name, desc,  etap));
        } else {
            text.push(format!("   {:<10} 🔒 {:<30} ({:<02})", name, desc, etap));
        }
    }
    text.join("\n")
}

pub fn format_plan(pipeline: &Pipeline) -> String {
    let mut text: Vec<String> = vec![];
    let mut triggers: Vec<String> = vec![];
    for t in &pipeline.trigger {
        triggers.push(t.to_string());
    }
    text.push(format!(
        "Pipeline: {}   (trigger: {})",
        pipeline.name,
        triggers.join(", ")
    ));
    text.push("".to_string());

    let pending = StepStatus::Pending;
    let running = StepStatus::Running;
    let succeed = StepStatus::Succeeded(std::time::Duration::from_millis(12_400));
    let failed = StepStatus::Failed(1);
    let skipped = StepStatus::Skipped("dependency 'test' crashed".to_string());

    for s in &pipeline.steps {
        text.push(format!("{}   {:<8}     {}", pending, s.name, s.command));
        if !Step::is_root(s) {
            text.push(format!("                 ↳ needs: {}", s.dependencies.join(", ")));
            if let Some(condition) = &s.condition {
                text.push(format!("                 ↳ condition:{} ", condition));
            }
        };
    }

    text.push("".to_string());
    text.push(format!(
        "{} steps · {} without dependencies · {} conditional",
        Pipeline::steps_amount(pipeline),
        Pipeline::roots_amount(pipeline),
        Pipeline::conditions_amount(pipeline)
    ));

    text.push(format!(
        "Legends: {} · {} · {} · {} · {}",
        pending, running, succeed, failed, skipped
    ));
    text.join("\n")
}

pub fn format_step(step: &Step, pipeline: &Pipeline) -> String{
    let mut text: Vec<String> = vec![];
    text.push(format!("Step: {}", step.name));
    text.push(format!("  command:      {}", step.command));

    if step.is_root() {
        text.push("  needs:        - (root)".to_string());
    } else {
        text.push(format!("  needs:        {}", step.dependencies.join(", ")));
    }

    let dependents = pipeline.dependents_of(&step.name);

    text.push(format!(
        "  needed by:    {}",
        if dependents.is_empty() {
            "-".to_string()
        } else {
            dependents.join(", ")
        }
    ));

    if let Some(condition) = &step.condition {
        text.push(format!("  condition:    {}", condition));
    }

    text.join("\n")
}

pub fn format_simulation(pipeline: &Pipeline, state: &RunState) -> String{
    let mut text: Vec<String> = vec![];
    let mut success = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for step in &pipeline.steps {
        if let Some(status) = state.statuses.get(&step.name) {
            text.push(format!("  {:<8}{}", step.name, status));
            match status {
                StepStatus::Succeeded(_s) => success += 1,
                StepStatus::Failed(_i) => failed += 1,
                StepStatus::Skipped(_s) => skipped += 1,
                StepStatus::Running | StepStatus::Pending => (),
            }
        }
    }
    text.push("".to_string());
    text.push(format!(
        "✓ Succeeded: {}, ✗ Failed: {}, ○ Skipped: {}",
        success, failed, skipped
    ));
    text.join("\n")
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
