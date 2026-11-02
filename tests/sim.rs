#[cfg(test)]
mod tests {
    use std::time::Duration;

    use cinderci::{
        cli::format_simulation,
        model::{Pipeline, StepStatus},
        sim::simulate,
    };

    #[test]
    fn pipeline() {
        let pipe = Pipeline::cinderci();
        let sim = simulate(&pipe, Some("build"));

        assert!(
            matches!(sim.statuses.get("test"), Some(StepStatus::Succeeded(success)) if *success == Duration::from_secs(1))
        );
        let plan = format_simulation(&pipe, &sim);
        println!("{}", plan);

        assert!(plan.contains("build   ✗ Failed (1)"));
    }
}
