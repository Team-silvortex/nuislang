use std::{fs, path::PathBuf};

use nuis_artifact::{
    compare_compiler_component_representation_paths, parse_compiler_component_differential,
    parse_compiler_component_representation_differential, render_compiler_component_differential,
    render_compiler_component_representation_differential,
    COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE,
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
    let representation_report_path =
        report_path.with_file_name(COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE);
    if representation_report_path == report_path
        || representation_report_path == stage0_record
        || representation_report_path == candidate_record
    {
        return Err(
            "bootstrap-diff representation report must not overwrite another artifact".to_owned(),
        );
    }
    let (report, representation_report) =
        compare_compiler_component_representation_paths(&stage0_record, &candidate_record)
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
    fs::write(
        &representation_report_path,
        render_compiler_component_representation_differential(&representation_report),
    )
    .map_err(|error| {
        format!(
            "failed to write compiler representation differential `{}`: {error}",
            representation_report_path.display()
        )
    })?;
    let verified = parse_compiler_component_differential(&report_path)
        .map_err(|error| format!("failed to verify compiler differential report: {error}"))?;
    let verified_representation = parse_compiler_component_representation_differential(
        &representation_report_path,
    )
    .map_err(|error| format!("failed to verify compiler representation differential: {error}"))?;
    if verified_representation != representation_report {
        return Err(
            "compiler representation differential changed during its disk round trip".to_owned(),
        );
    }

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
    println!(
        "  selected_representations: {}",
        verified_representation.comparison_count
    );
    println!(
        "  selected_representations_equivalent: {}",
        verified_representation.equivalent_count
    );
    println!(
        "  representation_report_sha256: {}",
        verified_representation.report_sha256
    );
    println!(
        "  representation_report: {}",
        representation_report_path.display()
    );
    if verified.deterministic_artifact_equivalent
        && verified_representation.all_representations_equivalent
    {
        Ok(())
    } else {
        Err(format!(
            "compiler component or selected representation drift blocked replacement; audit reports written to `{}` and `{}`",
            report_path.display(),
            representation_report_path.display()
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

    #[test]
    fn representation_report_cannot_alias_the_requested_base_report() {
        let error = run_bootstrap_diff(
            PathBuf::from("stage0.toml"),
            PathBuf::from("candidate.toml"),
            PathBuf::from(COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE),
        )
        .expect_err("base and representation reports must remain distinct");
        assert!(error.contains("must not overwrite another artifact"));
    }
}
