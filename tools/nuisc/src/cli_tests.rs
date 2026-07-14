use super::{parse_args, CommandKind};
use std::path::PathBuf;

#[test]
fn parse_compile_with_packaging_mode() {
    let command = parse_args(
        vec![
            "compile".to_owned(),
            "--packaging-mode".to_owned(),
            "nuis-self-contained-image".to_owned(),
            "main.ns".to_owned(),
            "out".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::Compile {
            input: PathBuf::from("main.ns"),
            output_dir: PathBuf::from("out"),
            verbose_cache: false,
            cpu_abi: None,
            target: None,
            packaging_mode: Some("nuis-self-contained-image".to_owned()),
        }
    );
}

#[test]
fn parse_pack_envelope_command() {
    let command = parse_args(
        vec![
            "pack-envelope".to_owned(),
            "nuis.build.manifest.toml".to_owned(),
            "out.nenv".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::PackEnvelope {
            input: PathBuf::from("nuis.build.manifest.toml"),
            output: PathBuf::from("out.nenv"),
        }
    );
}

#[test]
fn parse_unpack_envelope_command() {
    let command = parse_args(
        vec![
            "unpack-envelope".to_owned(),
            "artifact.nenv".to_owned(),
            "out.toml".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::UnpackEnvelope {
            input: PathBuf::from("artifact.nenv"),
            output: PathBuf::from("out.toml"),
        }
    );
}

#[test]
fn parse_inspect_artifact_command() {
    let command = parse_args(
        vec![
            "inspect-artifact".to_owned(),
            "nuis.compiled.artifact".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::InspectArtifact {
            input: PathBuf::from("nuis.compiled.artifact"),
            json: false,
        }
    );
}

#[test]
fn parse_verify_artifact_command() {
    let command = parse_args(
        vec![
            "verify-artifact".to_owned(),
            "nuis.compiled.artifact".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::VerifyArtifact {
            input: PathBuf::from("nuis.compiled.artifact"),
            json: false,
        }
    );
}

#[test]
fn parse_inspect_execution_command() {
    let command = parse_args(
        vec![
            "inspect-execution".to_owned(),
            "nuis.compiled.artifact".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::InspectExecution {
            input: PathBuf::from("nuis.compiled.artifact"),
            json: false,
        }
    );
}

#[test]
fn parse_inspect_execution_json_command() {
    let command = parse_args(
        vec![
            "inspect-execution".to_owned(),
            "--json".to_owned(),
            "nuis.compiled.artifact".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::InspectExecution {
            input: PathBuf::from("nuis.compiled.artifact"),
            json: true,
        }
    );
}

#[test]
fn parse_artifact_report_json_command() {
    let command = parse_args(
        vec![
            "artifact-report".to_owned(),
            "--json".to_owned(),
            "nuis.compiled.artifact".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::ArtifactReport {
            input: PathBuf::from("nuis.compiled.artifact"),
            json: true,
            summary: false,
        }
    );
}

#[test]
fn parse_artifact_report_summary_command() {
    let command = parse_args(
        vec![
            "artifact-report".to_owned(),
            "--summary".to_owned(),
            "nuis.compiled.artifact".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::ArtifactReport {
            input: PathBuf::from("nuis.compiled.artifact"),
            json: false,
            summary: true,
        }
    );
}

#[test]
fn parse_inspect_artifact_json_command() {
    let command = parse_args(
        vec![
            "inspect-artifact".to_owned(),
            "--json".to_owned(),
            "nuis.compiled.artifact".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::InspectArtifact {
            input: PathBuf::from("nuis.compiled.artifact"),
            json: true,
        }
    );
}

#[test]
fn parse_verify_artifact_json_command() {
    let command = parse_args(
        vec![
            "verify-artifact".to_owned(),
            "--json".to_owned(),
            "nuis.compiled.artifact".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::VerifyArtifact {
            input: PathBuf::from("nuis.compiled.artifact"),
            json: true,
        }
    );
}

#[test]
fn parse_verify_build_manifest_json_command() {
    let command = parse_args(
        vec![
            "verify-build-manifest".to_owned(),
            "--json".to_owned(),
            "nuis.build.manifest.toml".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::VerifyBuildManifest {
            manifest: PathBuf::from("nuis.build.manifest.toml"),
            json: true,
        }
    );
}

#[test]
fn parse_inspect_benchmarks_command() {
    let command =
        parse_args(vec!["inspect-benchmarks".to_owned(), "main.ns".to_owned()].into_iter())
            .unwrap();
    assert_eq!(
        command,
        CommandKind::InspectBenchmarks {
            input: PathBuf::from("main.ns"),
            json: false,
        }
    );
}

#[test]
fn parse_inspect_benchmarks_json_command() {
    let command = parse_args(
        vec![
            "inspect-benchmarks".to_owned(),
            "--json".to_owned(),
            "main.ns".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::InspectBenchmarks {
            input: PathBuf::from("main.ns"),
            json: true,
        }
    );
}

#[test]
fn parse_inspect_docs_command() {
    let command =
        parse_args(vec!["inspect-docs".to_owned(), "main.ns".to_owned()].into_iter()).unwrap();
    assert_eq!(
        command,
        CommandKind::InspectDocs {
            input: PathBuf::from("main.ns"),
            json: false,
            output: None,
        }
    );
}

#[test]
fn parse_inspect_docs_json_command() {
    let command = parse_args(
        vec![
            "inspect-docs".to_owned(),
            "--json".to_owned(),
            "nuis.toml".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::InspectDocs {
            input: PathBuf::from("nuis.toml"),
            json: true,
            output: None,
        }
    );
}

#[test]
fn parse_inspect_docs_json_output_command() {
    let command = parse_args(
        vec![
            "inspect-docs".to_owned(),
            "--json".to_owned(),
            "--output".to_owned(),
            "docs.json".to_owned(),
            "nuis.toml".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::InspectDocs {
            input: PathBuf::from("nuis.toml"),
            json: true,
            output: Some(PathBuf::from("docs.json")),
        }
    );
}

#[test]
fn parse_inspect_galaxy_docs_command() {
    let command =
        parse_args(vec!["inspect-galaxy-docs".to_owned(), "pixelmagic".to_owned()].into_iter())
            .unwrap();
    assert_eq!(
        command,
        CommandKind::InspectGalaxyDocs {
            galaxy: "pixelmagic".to_owned(),
            json: false,
        }
    );
}

#[test]
fn parse_inspect_galaxy_docs_json_command() {
    let command = parse_args(
        vec![
            "inspect-galaxy-docs".to_owned(),
            "--json".to_owned(),
            "pixelmagic".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::InspectGalaxyDocs {
            galaxy: "pixelmagic".to_owned(),
            json: true,
        }
    );
}

#[test]
fn parse_inspect_stdlib_docs_command() {
    let command = parse_args(vec!["inspect-stdlib-docs".to_owned()].into_iter()).unwrap();
    assert_eq!(command, CommandKind::InspectStdlibDocs { json: false });
}

#[test]
fn parse_inspect_stdlib_docs_json_command() {
    let command =
        parse_args(vec!["inspect-stdlib-docs".to_owned(), "--json".to_owned()].into_iter())
            .unwrap();
    assert_eq!(command, CommandKind::InspectStdlibDocs { json: true });
}

#[test]
fn parse_inspect_project_metadata_command() {
    let command = parse_args(
        vec![
            "inspect-project-metadata".to_owned(),
            "examples/projects/tooling/benchmark_report_file_demo".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::InspectProjectMetadata {
            input: PathBuf::from("examples/projects/tooling/benchmark_report_file_demo"),
            json: false,
            summary: false,
            paths_only: false,
        }
    );
}

#[test]
fn parse_inspect_project_metadata_json_command() {
    let command = parse_args(
        vec![
            "inspect-project-metadata".to_owned(),
            "--json".to_owned(),
            "build/nuis.build.manifest.toml".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::InspectProjectMetadata {
            input: PathBuf::from("build/nuis.build.manifest.toml"),
            json: true,
            summary: false,
            paths_only: false,
        }
    );
}

#[test]
fn parse_inspect_project_metadata_summary_command() {
    let command = parse_args(
        vec![
            "inspect-project-metadata".to_owned(),
            "--summary".to_owned(),
            "examples/projects/tooling/benchmark_report_file_demo".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::InspectProjectMetadata {
            input: PathBuf::from("examples/projects/tooling/benchmark_report_file_demo"),
            json: false,
            summary: true,
            paths_only: false,
        }
    );
}

#[test]
fn parse_inspect_project_metadata_paths_only_command() {
    let command = parse_args(
        vec![
            "inspect-project-metadata".to_owned(),
            "--paths-only".to_owned(),
            "build/nuis.build.manifest.toml".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::InspectProjectMetadata {
            input: PathBuf::from("build/nuis.build.manifest.toml"),
            json: false,
            summary: false,
            paths_only: true,
        }
    );
}

#[test]
fn parse_repair_project_metadata_command() {
    let command = parse_args(
        vec![
            "repair-project-metadata".to_owned(),
            "build/nuis.build.manifest.toml".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::RepairProjectMetadata {
            input: PathBuf::from("build/nuis.build.manifest.toml"),
            dry_run: false,
        }
    );
}

#[test]
fn parse_repair_project_metadata_dry_run_command() {
    let command = parse_args(
        vec![
            "repair-project-metadata".to_owned(),
            "--dry-run".to_owned(),
            "build/nuis.build.manifest.toml".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        command,
        CommandKind::RepairProjectMetadata {
            input: PathBuf::from("build/nuis.build.manifest.toml"),
            dry_run: true,
        }
    );
}
