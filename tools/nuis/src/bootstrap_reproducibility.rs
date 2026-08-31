use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use nuis_artifact::{
    build_compiler_component_reproducibility_from_paths,
    build_compiler_component_reproducibility_v2_from_paths,
    read_compiler_component_reproducibility, read_compiler_component_reproducibility_v2,
    render_compiler_component_reproducibility, render_compiler_component_reproducibility_v2,
    CompilerComponentReproducibilityRootInput, COMPILER_COMPONENT_REPRODUCIBILITY_FILE,
    COMPILER_COMPONENT_REPRODUCIBILITY_V2_FILE,
};

use crate::{
    bootstrap_candidate_build::handle_bootstrap_clean_candidate_build, digest_sha256::sha256_hex,
};

const RUN_IDS: [&str; 2] = ["clean-build-0", "clean-build-1"];
const INVOCATION_WITNESS_CONTRACT: &str = "nuis-bootstrap-reproducibility-invocation-v1";
const RUN_WITNESS_CONTRACT: &str = "nuis-bootstrap-clean-run-witness-v1";

pub(crate) fn handle_bootstrap_reproducibility(
    input: PathBuf,
    output_dir: PathBuf,
) -> Result<(), String> {
    prepare_empty_output_root(&output_dir)?;
    let invocation_witness = invocation_witness(&output_dir)?;
    let roots = RUN_IDS
        .iter()
        .map(|run_id| output_dir.join(run_id))
        .collect::<Vec<_>>();
    let witnesses = RUN_IDS
        .iter()
        .enumerate()
        .map(|(ordinal, run_id)| run_witness(&invocation_witness, ordinal, run_id))
        .collect::<Vec<_>>();

    for root in &roots {
        if root.exists() {
            return Err(format!(
                "bootstrap reproducibility run root `{}` must not exist before its build",
                root.display()
            ));
        }
        handle_bootstrap_clean_candidate_build(input.clone(), root.clone())?;
    }

    let root_inputs = RUN_IDS
        .iter()
        .zip(&witnesses)
        .zip(&roots)
        .map(
            |((run_id, witness), root)| CompilerComponentReproducibilityRootInput {
                run_id,
                clean_root_witness_sha256: witness,
                root,
            },
        )
        .collect::<Vec<_>>();
    let report = build_compiler_component_reproducibility_from_paths(&root_inputs)
        .map_err(|error| format!("failed to aggregate clean compiler builds: {error}"))?;
    let report_source = render_compiler_component_reproducibility(&report);
    let report_v2 =
        build_compiler_component_reproducibility_v2_from_paths(&report, &report_source, &roots)
            .map_err(|error| format!("failed to bind clean representation sidecars: {error}"))?;
    let report_v2_source = render_compiler_component_reproducibility_v2(&report_v2);
    let report_path = output_dir.join(COMPILER_COMPONENT_REPRODUCIBILITY_FILE);
    fs::write(&report_path, report_source).map_err(|error| {
        format!(
            "failed to write compiler reproducibility aggregate `{}`: {error}",
            report_path.display()
        )
    })?;
    let report_v2_path = output_dir.join(COMPILER_COMPONENT_REPRODUCIBILITY_V2_FILE);
    fs::write(&report_v2_path, report_v2_source).map_err(|error| {
        format!(
            "failed to write compiler reproducibility v2 `{}`: {error}",
            report_v2_path.display()
        )
    })?;
    let verified = read_compiler_component_reproducibility(&report_path, &roots)
        .map_err(|error| format!("failed to verify clean compiler builds: {error}"))?;
    let verified_v2 =
        read_compiler_component_reproducibility_v2(&report_v2_path, &report_path, &roots)
            .map_err(|error| format!("failed to verify clean representation sidecars: {error}"))?;

    println!("bootstrap compiler reproducibility: verified");
    println!("  clean_runs: {}", verified.run_count);
    println!(
        "  candidate_reproducible_build_sha256: {}",
        verified.candidate_reproducible_build_sha256
    );
    println!("  differential: {}", verified.differential_verdict);
    println!("  verdict: {}", verified.verdict);
    println!(
        "  selected_representations: {}/{}",
        verified_v2.equivalent_representation_count, verified_v2.representation_comparison_count
    );
    println!(
        "  sidecars_individually_bound: {}",
        verified_v2.sidecars_individually_bound
    );
    println!("  replacement_authorized: false");
    println!("  aggregate: {}", report_path.display());
    println!("  aggregate_v2: {}", report_v2_path.display());
    Ok(())
}

fn prepare_empty_output_root(output_dir: &Path) -> Result<(), String> {
    match fs::symlink_metadata(output_dir) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(format!(
                    "bootstrap reproducibility output `{}` must be a real directory",
                    output_dir.display()
                ));
            }
            let mut entries = fs::read_dir(output_dir).map_err(|error| {
                format!(
                    "failed to inspect bootstrap reproducibility output `{}`: {error}",
                    output_dir.display()
                )
            })?;
            if entries.next().is_some() {
                return Err(format!(
                    "bootstrap reproducibility output `{}` must be empty",
                    output_dir.display()
                ));
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            fs::create_dir_all(output_dir).map_err(|error| {
                format!(
                    "failed to create bootstrap reproducibility output `{}`: {error}",
                    output_dir.display()
                )
            })?;
        }
        Err(error) => {
            return Err(format!(
                "failed to inspect bootstrap reproducibility output `{}`: {error}",
                output_dir.display()
            ));
        }
    }
    Ok(())
}

fn invocation_witness(output_dir: &Path) -> Result<String, String> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock cannot seed clean build witness: {error}"))?
        .as_nanos();
    let seed = format!(
        "{INVOCATION_WITNESS_CONTRACT}\0{}\0{nanos}\0{}",
        process::id(),
        output_dir.to_string_lossy()
    );
    Ok(sha256_hex(seed.as_bytes()))
}

fn run_witness(invocation: &str, ordinal: usize, run_id: &str) -> String {
    sha256_hex(format!("{RUN_WITNESS_CONTRACT}\0{invocation}\0{ordinal}\0{run_id}").as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_witnesses_are_distinct_and_path_free() {
        let first = run_witness(&"a".repeat(64), 0, RUN_IDS[0]);
        let second = run_witness(&"a".repeat(64), 1, RUN_IDS[1]);
        assert_ne!(first, second);
        assert_eq!(first.len(), 64);
        assert!(!first.contains('/'));
        assert!(!second.contains('/'));
    }
}
