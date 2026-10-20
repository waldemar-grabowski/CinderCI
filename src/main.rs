use std::{collections::HashMap, env, fmt::Debug, process::ExitCode, vec};

#[derive(Debug)]
enum Trigger {
    Push,
    PullRequest,
    #[allow(dead_code)]
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
struct RunState {
    statuses: HashMap<String, StepStatus>,
}

impl RunState {
    fn new(pipeline: &Pipeline) -> RunState {
        let mut map = HashMap::new();
        for step in &pipeline.steps {
            map.insert(step.name.clone(), StepStatus::Pending);
        }
        RunState { statuses: map }
    }
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

    pub fn is_root(&self) -> bool {
        self.dependencies.is_empty()
    }

    pub fn is_condition(&self) -> bool {
        self.condition.is_some()
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

    #[allow(clippy::manual_find)]
    pub fn find_step(&self, name: &str) -> Option<&Step> {
        for step in &self.steps {
            if step.name == name {
                return Some(step);
            }
        }
        None
    }

    fn step_names(&self) -> String {
        self.steps
            .iter()
            .map(|step| step.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn dependents_of(&self, name: &str) -> Vec<&str> {
        let mut result: Vec<&str> = vec![];
        for s in &self.steps {
            for d in &s.dependencies {
                if d == name {
                    result.push(&s.name);
                }
            }
        }
        result
    }
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Pending => write!(f, "⏳ Pending"),
            Self::Running => write!(f, "▶ Running"),
            Self::Succeeded(s) => write!(f, "✓ Success ({}s)", s.as_secs_f32()),
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

fn print_step(step: &Step, pipeline: &Pipeline) {
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

fn simulate(pipeline: &Pipeline, name: Option<&str>) -> RunState {
    let mut state = RunState::new(pipeline);

    for step in &pipeline.steps {
        let mut reason: Option<String> = None;
        for dependency in &step.dependencies {
            if let Some(StepStatus::Failed(_) | StepStatus::Skipped(_)) =
                state.statuses.get(dependency)
                && reason.is_none()
            {
                reason = format!("dependency '{dependency}' failed").into();
            };
        }

        let status = if let Some(text) = reason {
            StepStatus::Skipped(text)
        } else if name == Some(step.name.as_str()) {
            StepStatus::Failed(1)
        } else {
            StepStatus::Succeeded(std::time::Duration::from_millis(
                step.command.len() as u64 * 100,
            ))
        };
        state.statuses.insert(step.name.clone(), status);
    }
    state
}

fn print_simulation(pipeline: &Pipeline, state: &RunState) {
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

fn main() -> ExitCode {
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
