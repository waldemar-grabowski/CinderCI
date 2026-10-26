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
