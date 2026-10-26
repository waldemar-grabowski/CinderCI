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
