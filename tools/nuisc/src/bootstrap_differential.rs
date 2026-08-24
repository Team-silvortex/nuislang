use std::{fs, path::PathBuf};

use nuis_artifact::{
    compare_compiler_component_paths, parse_compiler_component_differential,
    render_compiler_component_differential,
};

pub(crate) fn run_bootstrap_diff(
    stage0_record: PathBuf,
    candidate_record: PathBuf,
    report_path: PathBuf,
) -> Result<(), String> {
    if stage0_record == candidate_record {
        return Err("bootstrap-diff requires two distinct component record paths".to_owned());
    }
    if report_path == stage0_record || report_path == candidate_record {
        return Err(
            "bootstrap-diff report must not overwrite an input component record".to_owned(),
        );
    }
    let report = compare_compiler_component_paths(&stage0_record, &candidate_record)
        .map_err(|error| format!("failed to compare compiler components: {error}"))?;
    if let Some(parent) = report_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create differential report directory `{}`: {error}",
                parent.display()
            )
        })?;
    }
    fs::write(
        &report_path,
        render_compiler_component_differential(&report),
    )
    .map_err(|error| {
        format!(
            "failed to write compiler differential report `{}`: {error}",
            report_path.display()
        )
    })?;
    let verified = parse_compiler_component_differential(&report_path)
        .map_err(|error| format!("failed to verify compiler differential report: {error}"))?;

    println!("bootstrap differential: {}", verified.verdict);
    println!("  protocol: {}", verified.protocol);
    println!("  component: {}", verified.component_id);
    println!("  comparisons: {}", verified.comparison_count);
    println!("  equivalent: {}", verified.equivalent_count);
    println!(
        "  deterministic_artifact_equivalent: {}",
        verified.deterministic_artifact_equivalent
    );
    println!(
        "  replacement_authorized: {}",
        verified.replacement_authorized
    );
    println!("  report_sha256: {}", verified.report_sha256);
    println!("  report: {}", report_path.display());
    if verified.deterministic_artifact_equivalent {
        Ok(())
    } else {
        Err(format!(
            "compiler component drift blocked replacement; audit report written to `{}`",
            report_path.display()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differential_report_cannot_overwrite_an_input_record() {
        let error = run_bootstrap_diff(
            PathBuf::from("stage0.toml"),
            PathBuf::from("candidate.toml"),
            PathBuf::from("stage0.toml"),
        )
        .expect_err("output collision must fail before reading inputs");
        assert!(error.contains("must not overwrite"));
    }
}
