use std::fmt::Debug;

#[derive(Debug)]
pub enum Trigger {
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
pub struct Step {
    pub(crate) name: String,
    pub(crate) command: String,
    pub(crate) dependencies: Vec<String>,
    pub(crate) condition: Option<String>,
}

impl Step {
    fn new(step_name: &str, step_command: &str) -> Step {
        Step {
            name: step_name.to_string(),
            command: step_command.to_string(),
            dependencies: vec![],
            condition: None,
        }
    }

    pub(crate) fn is_root(&self) -> bool {
        self.dependencies.is_empty()
    }

    fn is_condition(&self) -> bool {
        self.condition.is_some()
    }
}

#[derive(Debug)]
pub struct Pipeline {
    pub(crate) name: String,
    pub(crate) steps: Vec<Step>,
    pub(crate) trigger: Vec<Trigger>,
}

impl Pipeline {
    pub fn cinderci() -> Pipeline {
        let mut cinderci = vec![];
        cinderci.push(Step::new("fmt", "cargo fmt --check"));
        cinderci.push(Step::new("lint", "cargo clippy -- -D warnings"));
        cinderci.push(Step::new("test", "cargo test"));
        let mut build = Step::new("build", "cargo build --release");
        build.dependencies = vec!["fmt".to_string(), "lint".to_string(), "test".to_string()];
        cinderci.push(build);
        Pipeline {
            name: "cinderci".to_string(),
            steps: cinderci,
            trigger: vec![Trigger::Push],
        }
    }

    pub fn example() -> Pipeline {
        let mut example = vec![];
        example.push(Step::new("fmt", "cargo fmt --check"));
        example.push(Step::new("lint", "cargo clippy -- -D warnings"));
        example.push(Step::new("build", "cargo build --release"));
        let mut test = Step::new("test", "cargo test");
        test.dependencies = vec!["build".to_string()];
        example.push(test);
        let mut deploy = Step::new("deploy", "./deploy.sh");
        deploy.dependencies = vec!["test".to_string(), "lint".to_string(), "fmt".to_string()];
        deploy.condition = Some("branch == \"main\" && event == \"push\"".to_string());
        example.push(deploy);
        Pipeline {
            name: "build-and-test".to_string(),
            steps: example,
            trigger: vec![Trigger::Push, Trigger::PullRequest],
        }
    }

    pub(crate) fn steps_amount(&self) -> usize {
        let mut amount = 0;
        for _step in &self.steps {
            amount += 1;
        }
        amount
    }

    pub(crate) fn roots_amount(&self) -> usize {
        let mut amount = 0;
        for step in &self.steps {
            if step.is_root() {
                amount += 1
            };
        }
        amount
    }

    pub(crate) fn conditions_amount(&self) -> usize {
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

    pub fn step_names(&self) -> String {
        self.steps
            .iter()
            .map(|step| step.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub(crate) fn dependents_of(&self, name: &str) -> Vec<&str> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_has_no_deps() {
        let step = Step::new("fmt", "cargo fmt --check");
        assert!(step.is_root());
    }

    #[test]
    fn just_roots() {
        let mut root = 0;
        let mut cond = 0;
        let pipe = Pipeline::cinderci();
        for step in pipe.steps {
            if step.is_root() {
                root += 1;
            } else if step.is_condition() {
                cond += 1;
            }
        }
        assert_eq!(root, 3);
        assert_eq!(cond, 0);
    }

    #[test]
    fn both_types() {
        let mut root = 0;
        let mut cond = 0;
        let pipe = Pipeline::example();
        for step in pipe.steps {
            if step.is_root() {
                root += 1;
            } else if step.is_condition() {
                cond += 1;
            }
        }
        assert_eq!(root, 3);
        assert_eq!(cond, 1);
    }

    #[test]
    fn test_find_step() {
        let pipe = Pipeline::cinderci();
        let step = pipe.find_step("test");

        match step {
            None => assert!(false),
            _ => assert!(true),
        }
    }

    #[test]
    fn test_find_non_existent() {
        let pipe = Pipeline::cinderci();
        let step = pipe.find_step("nope");

        match step {
            None => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn dependents_of_test() {
        let pipe = Pipeline::cinderci();
        let step = pipe.dependents_of("test");

        assert_eq!(step, vec!["build"]);
    }
}
