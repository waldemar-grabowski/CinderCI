use std::collections::HashMap;

use crate::model::Pipeline;
use crate::model::StepStatus;

#[derive(Debug)]
pub struct RunState {
    pub statuses: HashMap<String, StepStatus>,
}

impl RunState {
    pub(crate) fn new(pipeline: &Pipeline) -> RunState {
        let mut map = HashMap::new();
        for step in &pipeline.steps {
            map.insert(step.name.clone(), StepStatus::Pending);
        }
        RunState { statuses: map }
    }
}

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
