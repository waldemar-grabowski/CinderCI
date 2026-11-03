
use std::collections::HashMap;

use crate::model::Pipeline;
use crate::model::StepStatus;

/// Represents the execution state of a pipeline.
///
/// Keeps track of individual step statuses and provides
/// a shared representation of the pipeline's execution progress.
#[derive(Debug)]
pub struct RunState {
    pub statuses: HashMap<String, StepStatus>,
}

impl RunState {
    /// Creates an initial execution state for a pipeline.
    ///
    /// Ensures that every pipeline step has a status
    /// before execution begins.
    ///
    /// # Arguments
    /// * `pipeline` - The pipeline whose execution state will be initialized.
    pub(crate) fn new(pipeline: &Pipeline) -> RunState {
        let mut map = HashMap::new();

        for step in &pipeline.steps {
            map.insert(step.name.clone(), StepStatus::Pending);
        }

        RunState { statuses: map }
    }
}

/// Summarizes the execution results of pipeline steps.
///
/// Provides an overview of the pipeline outcome by counting
/// successful, failed, and skipped steps.
///
/// # Arguments
/// * `steps` - A slice containing the statuses of pipeline steps.
///
/// # Returns
/// A tuple containing:
/// * `bool` - Whether the execution completed successfully.
/// * `i32` - Number of successful steps.
/// * `i32` - Number of failed steps.
/// * `i32` - Number of skipped steps.
///
/// # Examples
///
/// ```
/// use cinderci::model::StepStatus;
/// use cinderci::state::summarize;
///
/// let results = vec![
///     StepStatus::Succeeded(std::time::Duration::from_secs(1)),
///     StepStatus::Succeeded(std::time::Duration::from_secs(2)),
///     StepStatus::Failed(1),
/// ];
///
/// assert_eq!(summarize(&results), (false, 2, 1, 0));
/// ```
pub fn summarize(steps: &[StepStatus]) -> (bool, i32, i32, i32) {
    let mut success = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for step in steps {
        match step {
            StepStatus::Succeeded(_s) => success += 1,
            StepStatus::Failed(_i) => failed += 1,
            StepStatus::Skipped(_s) => skipped += 1,
            StepStatus::Running | StepStatus::Pending => (),
        }
    }

    let status = !(failed > 0 || skipped > 0);

    (status, success, failed, skipped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_result_pass() {
        let results: Vec<StepStatus> = vec![
            StepStatus::Succeeded(std::time::Duration::from_secs(1)),
            StepStatus::Succeeded(std::time::Duration::from_secs(1)),
            StepStatus::Succeeded(std::time::Duration::from_secs(1)),
            StepStatus::Succeeded(std::time::Duration::from_secs(1)),
        ];

        assert_eq!(summarize(&results), (true, 4, 0, 0));
    }

    #[test]
    fn test_run_result_fail() {
        let results: Vec<StepStatus> = vec![
            StepStatus::Succeeded(std::time::Duration::from_secs(1)),
            StepStatus::Succeeded(std::time::Duration::from_secs(1)),
            StepStatus::Failed(2137),
            StepStatus::Skipped("foo".to_string()),
        ];

        assert_eq!(summarize(&results), (false, 2, 1, 1));
    }

    #[test]
    fn failure_on_no_clean_exit() {
        let results: Vec<StepStatus> = vec![
            StepStatus::Skipped("foo".to_string()),
            StepStatus::Skipped("foo".to_string()),
            StepStatus::Skipped("foo".to_string()),
            StepStatus::Skipped("foo".to_string()),
        ];

        assert_eq!(summarize(&results), (false, 0, 0, 4));
    }
}
