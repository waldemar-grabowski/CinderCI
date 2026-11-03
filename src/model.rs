use std::fmt::Debug;

/// Represents the events that can trigger a pipeline execution.
///
/// Allows pipelines to define when they should be executed.
#[derive(Debug)]
pub enum Trigger {
    Push,
    PullRequest,
    #[allow(dead_code)]
    Manual,
}

/// Represents the current execution status of a pipeline step.
///
/// Provides a consistent way to track the progress and outcome
/// of individual steps during pipeline execution.
#[derive(Debug, PartialEq, Clone)]
pub enum StepStatus {
    Pending,
    Running,
    Succeeded(std::time::Duration),
    Failed(i32),
    Skipped(String),
}

/// Represents an individual task within a pipeline.
///
/// Defines the command to execute, its dependencies, and optional
/// conditions that determine whether the step should run.
#[derive(Debug)]
pub struct Step {
    pub(crate) name: String,
    pub(crate) command: String,
    pub(crate) dependencies: Vec<String>,
    pub(crate) condition: Option<String>,
}

impl Step {
    /// Creates a new pipeline step.
    ///
    /// Provides a starting point for defining individual pipeline tasks.
    fn new(step_name: &str, step_command: &str) -> Step {
        Step {
            name: step_name.to_string(),
            command: step_command.to_string(),
            dependencies: vec![],
            condition: None,
        }
    }

    /// Determines whether the step can run independently.
    ///
    /// Identifies steps that do not depend on other pipeline tasks.
    pub(crate) fn is_root(&self) -> bool {
        self.dependencies.is_empty()
    }

    /// Determines whether the step has an execution condition.
    fn is_condition(&self) -> bool {
        self.condition.is_some()
    }
}

/// Represents a complete CI/CD pipeline.
///
/// Groups related steps, their dependencies, and execution triggers
/// into a single workflow.
#[derive(Debug)]
pub struct Pipeline {
    pub(crate) name: String,
    pub(crate) steps: Vec<Step>,
    pub(crate) trigger: Vec<Trigger>,
}

impl Pipeline {
    /// Creates the default CinderCI pipeline.
    ///
    /// Provides a predefined workflow for formatting, linting,
    /// testing, and building the project.
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

    /// Creates an example pipeline demonstrating available features.
    ///
    /// Provides a reference workflow with dependencies,
    /// multiple triggers, and conditional execution.
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

    /// Returns the total number of steps in the pipeline.
    ///
    /// Provides information about the size of the workflow.
    pub(crate) fn steps_amount(&self) -> usize {
        let mut amount = 0;
        for _step in &self.steps {
            amount += 1;
        }
        amount
    }

    /// Returns the number of steps without dependencies.
    ///
    /// Identifies tasks that can start independently.
    pub(crate) fn roots_amount(&self) -> usize {
        let mut amount = 0;
        for step in &self.steps {
            if step.is_root() {
                amount += 1
            };
        }
        amount
    }

    /// Returns the number of steps with execution conditions.
    ///
    /// Provides insight into how much of the workflow
    /// depends on conditional execution.
    pub(crate) fn conditions_amount(&self) -> usize {
        let mut amount = 0;
        for step in &self.steps {
            if step.is_condition() {
                amount += 1
            };
        }
        amount
    }

    /// Finds a pipeline step by its name.
    ///
    /// Allows other parts of the application to access
    /// a specific task within the workflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use cinderci::model::Pipeline;
    ///
    /// let pipeline = Pipeline::cinderci();
    /// let step = pipeline.find_step("test");
    ///
    /// assert!(step.is_some());
    /// ```
    ///
    /// # Returns
    ///
    /// `Some(&Step)` if the step exists, otherwise `None`.
    #[allow(clippy::manual_find)]
    pub fn find_step(&self, name: &str) -> Option<&Step> {
        for step in &self.steps {
            if step.name == name {
                return Some(step);
            }
        }
        None
    }

    /// Returns all step names as a comma-separated string.
    ///
    /// Provides a convenient overview of the tasks
    /// belonging to the pipeline.
    pub fn step_names(&self) -> String {
        self.steps
            .iter()
            .map(|step| step.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Returns the names of steps depending on a given step.
    ///
    /// Allows the application to identify which tasks
    /// are affected by another step's execution.
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
    /// Formats a step status into a human-readable representation.
    ///
    /// Makes execution results easier to understand and display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Pending => write!(f, "⏳ Pending"),
            Self::Running => write!(f, "▶ Running"),
            Self::Succeeded(s) => write!(f, "✓ Success ({:.1}s)", s.as_secs_f32()),
            Self::Failed(e) => write!(f, "✗ Failed ({})", e),
            Self::Skipped(e) => write!(f, "○ Skipped ({})", e),
        }
    }
}

impl std::fmt::Display for Trigger {
    /// Formats a pipeline trigger into its string representation.
    ///
    /// Provides consistent identifiers for pipeline trigger events.
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
        let pipe = Pipeline::cinderci();

        assert_eq!(pipe.roots_amount(), 3);
        assert_eq!(pipe.conditions_amount(), 0);
    }

    #[test]
    fn both_types() {
        let pipe = Pipeline::example();

        assert_eq!(pipe.roots_amount(), 3);
        assert_eq!(pipe.conditions_amount(), 1);
    }

    #[test]
    fn test_find_step() {
        let pipe = Pipeline::cinderci();
        let step = pipe.find_step("test");

        assert!(step.is_some());
    }

    #[test]
    fn test_find_non_existent() {
        let pipe = Pipeline::cinderci();
        let step = pipe.find_step("nope");

        assert!(step.is_none());
    }

    #[test]
    fn dependents_of_test() {
        let pipe = Pipeline::cinderci();
        let step = pipe.dependents_of("test");

        assert_eq!(step, vec!["build"]);
    }
}
