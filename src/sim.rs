use crate::model::{Pipeline, StepStatus};
use crate::state::RunState;

pub fn simulate(pipeline: &Pipeline, name: Option<&str>) -> RunState {
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn no_fails_all_passes() {
        let pipe = Pipeline::cinderci();
        let pass = simulate(&pipe, None);
        for step in pass.statuses.values() {
            assert!(matches!(step, StepStatus::Succeeded(_)));
        }
    }

    #[test]
    fn time_is_predictable() {
        let pipe = Pipeline::cinderci();
        let sim = simulate(&pipe, None);

        assert!(matches!(sim.statuses.get("test"), Some(StepStatus::Succeeded(success)) if *success == Duration::from_secs(1)));
    }

    #[test]
    fn fail_skips_dependents() {
        let pipe = Pipeline::cinderci();
        let sim = simulate(&pipe, Some("test"));
        assert!(matches!(sim.statuses.get("test"), Some(StepStatus::Failed(1))));
        assert!(matches!(sim.statuses.get("build"), Some(StepStatus::Skipped(_))));
        assert!(matches!(sim.statuses.get("fmt"), Some(StepStatus::Succeeded(_))));
        assert!(matches!(sim.statuses.get("lint"), Some(StepStatus::Succeeded(_))));
    }
}
