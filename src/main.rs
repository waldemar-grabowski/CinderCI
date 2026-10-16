use std::{env, fmt::Debug, process::ExitCode, vec};

#[derive(Debug)]
enum Trigger {
    Push,
    PullRequest,
    Manual,
}

#[derive(Debug)]
pub enum StepStatus {
    Pending,
    Running,
    Succeeded(std::time::Duration),
    Failed(i32),
    Skipped(String),
}

#[derive(Debug)]
struct Step {
    name: String,
    command: String,
    dependencies: Vec<String>,
    condition: Option<String>,
}

impl Step {
    fn new(step_name: String, step_command: String) -> Step {
        Step {
            name: step_name,
            command: step_command,
            dependencies: vec![],
            condition: None,
        }
    }

    // Short if
    pub fn is_root(&self) -> bool {
        self.dependencies.is_empty()
    }

    // Long if
    pub fn is_condition(&self) -> bool {
        if self.condition.iter().len() > 0 {
            true
        } else {
            false
        }
    }
}

#[derive(Debug)]
struct Pipeline {
    name: String,
    steps: Vec<Step>,
    trigger: Vec<Trigger>,
}

impl Pipeline {
    fn cinderci() -> Pipeline {
        let mut cinderci = vec![];
        cinderci.push(Step::new(
            "fmt".to_string(),
            "cargo fmt --check".to_string(),
        ));
        cinderci.push(Step::new(
            "lint".to_string(),
            "cargo clippy -- -D warnings".to_string(),
        ));
        cinderci.push(Step::new("test".to_string(), "cargo test".to_string()));
        let mut build = Step::new("build".to_string(), "cargo build --release".to_string());
        build.dependencies = vec!["fmt".to_string(), "lint".to_string(), "test".to_string()];
        cinderci.push(build);
        Pipeline {
            name: "cinderci".to_string(),
            steps: cinderci,
            trigger: vec![Trigger::Push],
        }
    }

    fn example() -> Pipeline {
        let mut example = vec![];
        example.push(Step::new(
            "fmt".to_string(),
            "cargo fmt --check".to_string(),
        ));
        example.push(Step::new(
            "lint".to_string(),
            "cargo clippy -- -D warnings".to_string(),
        ));
        example.push(Step::new(
            "build".to_string(),
            "cargo build --release".to_string(),
        ));
        let mut test = Step::new("test".to_string(), "cargo test".to_string());
        test.dependencies = vec!["build".to_string()];
        example.push(test);
        let mut deploy = Step::new("deploy".to_string(), "./deploy.sh".to_string());
        deploy.dependencies = vec!["test".to_string(), "lint".to_string(), "fmt".to_string()];
        deploy.condition = Some("branch == \"main\" && event == \"push\"".to_string());
        example.push(deploy);
        Pipeline {
            name: "build-and-test".to_string(),
            steps: example,
            trigger: vec![Trigger::Push, Trigger::PullRequest],
        }
    }

    fn steps_amount(&self) -> usize {
        let mut amount = 0;
        for _step in &self.steps {
            amount += 1;
        }
        amount
    }

    fn roots_amount(&self) -> usize {
        let mut amount = 0;
        for step in &self.steps {
            if step.is_root() {
                amount += 1
            };
        }
        amount
    }

    fn conditions_amount(&self) -> usize {
        let mut amount = 0;
        for step in &self.steps {
            if step.is_condition() {
                amount += 1
            };
        }
        amount
    }
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Pending => write!(f, "⏳ Pending"),
            Self::Running => write!(f, "▶ Running "),
            Self::Succeeded(s) => write!(f, "✓ Success ({}s)", s.as_secs_f32().round()),
            Self::Failed(e) => write!(f, "✗ Failed ({})", e),
            Self::Skipped(e) => write!(f, "○ Skipped ({})", e),
        }
    }
}

impl std::fmt::Display for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Manual => write!(f, "manual"),
            Self::PullRequest => write!(f, "pull_request"),
            Self::Push => write!(f, "push"),
        }
    }
}

const HELP_COMMANDS: [(&str, &str, u8, bool); 7] = [
    ("help", "this help", 0, true),
    ("version", "program version", 0, true),
    ("plan", "pipeline plan", 1, true),
    ("validate", "validate cinder.yaml", 5, false),
    ("graph", "dependecy graph", 6, false),
    ("run", "start pipeline", 7, false),
    ("tui", "fullscreen tui", 17, false),
];

fn print_help() {
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

fn print_plan(pipeline: &Pipeline) {
    println!();
    let mut triggers: Vec<String> = vec![];
    for t in &pipeline.trigger {
        triggers.push(t.to_string());
    }
    println!(
        "Pipeline: {}   (trigger: {})",
        pipeline.name,
        triggers.join(", ")
    );

    for s in &pipeline.steps {
        println!("⏳  {:<5}     {}", s.name, s.command);
        if !Step::is_root(s) {
            println!("              ↳ needs: {}", s.dependencies.join(", "))
        };
    }

    println!();
    println!(
        "{} steps · {} without dependencies · {} conditional",
        Pipeline::steps_amount(pipeline),
        Pipeline::roots_amount(pipeline),
        Pipeline::conditions_amount(pipeline)
    );
}

fn main() -> ExitCode {
    let command: Option<String> = env::args().nth(1);
    let arg1: Option<String> = env::args().nth(2);

    match command.as_deref() {
        Some("-V" | "version" | "--version") => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            return ExitCode::from(0);
        }
        Some("-h" | "help" | "--help") | None => print_help(),
        Some("-p" | "plan" | "--plan") => match arg1.as_deref() {
            Some("example") => print_plan(&Pipeline::example()),
            None => print_plan(&Pipeline::cinderci()),
            Some(&_) => return ExitCode::from(2),
        },
        Some(command) => {
            for (name, _desc, etap, avail) in HELP_COMMANDS {
                if command == name && !avail {
                    eprintln!(
                        "Command {} not available, will be implemented at stage {}.",
                        name, etap
                    );
                    return ExitCode::from(2);
                }
            }

            eprintln!(
                "Command {} not available, please use 'cinderci help' to see available commands.",
                command
            );
            return ExitCode::from(2);
        }
    }

    ExitCode::SUCCESS
}
