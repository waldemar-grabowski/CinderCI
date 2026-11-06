use std::time::Duration;

use cinderci::{
    cli::format_simulation,
    model::{Pipeline, StepStatus},
    sim::simulate,
};

// Verifies that pipeline simulation correctly handles failures
// and that the resulting execution summary is formatted correctly.
#[test]
fn pipeline() {
    let pipe = Pipeline::cinderci();
    let sim = simulate(&pipe, Some("build"));

    assert!(
        matches!(sim.statuses.get("test"), Some(StepStatus::Succeeded(success)) if *success == Duration::from_secs(1))
    );

    let plan = format_simulation(&pipe, &sim);

    assert!(plan.contains("build   ✗ Failed (1)"));
    assert!(plan.contains("Result: failure · 3 ✓ · 1 ✗ · 0 ○"));
}
