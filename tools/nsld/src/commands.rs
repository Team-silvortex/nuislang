use crate::context::load_link_input_context;
use std::path::Path;

pub(crate) fn run_status_command() {
    println!("Nsld linker front-door");
    println!("  tool: nsld");
    println!("  phase: alpha-0.8.0 binary-linking convergence");
    println!(
        "  current_role: link-plan inspection, artifact-chain diagnosis, and final executable readiness"
    );
    println!("  implementation: reuses nuisc::linker while linker ownership is split out");
    println!(
        "  final_link_status: final executable emission is still blocked before real binary linking"
    );
}

pub(crate) fn run_plan_command(input: &Path, json: bool) -> Result<(), String> {
    let ctx = load_link_input_context(input)?;
    if json {
        println!("{}", nuisc::linker::render_link_plan_json(&ctx.plan));
    } else {
        println!("Nsld link plan");
        println!("  input: {}", ctx.input.display());
        println!("  manifest: {}", ctx.manifest.display());
        println!("  role: alpha-0.8.0 binary-linking convergence front-door");
        for line in nuisc::linker::render_link_plan_summary(&ctx.plan) {
            println!("  {line}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::run_plan_command;
    use std::{env, fs};

    #[test]
    fn plan_command_reports_missing_manifest_directory() {
        let dir = env::temp_dir().join(format!(
            "nsld-plan-command-missing-manifest-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();

        let error = run_plan_command(&dir, false).unwrap_err();
        fs::remove_dir_all(dir).unwrap();

        assert!(error.contains("does not contain `nuis.build.manifest.toml`"));
    }
}
