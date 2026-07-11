use super::{parse_args, Command};
use std::path::PathBuf;

#[test]
fn parses_status_by_default() {
    assert_eq!(
        parse_args(Vec::<String>::new().into_iter()),
        Ok(Command::Status)
    );
}

#[test]
fn parses_plan_input_and_json_flag() {
    let command =
        parse_args(vec!["plan".to_owned(), "out".to_owned(), "--json".to_owned()].into_iter());
    assert_eq!(
        command,
        Ok(Command::Plan {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_check_input_and_json_flag() {
    let command = parse_args(
        vec![
            "check".to_owned(),
            "nuis.build.manifest.toml".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::Check {
            input: PathBuf::from("nuis.build.manifest.toml"),
            json: true
        })
    );
}

#[test]
fn parses_check_next_action_input_and_json_flag() {
    let command = parse_args(
        vec![
            "check-next-action".to_owned(),
            "nuis.build.manifest.toml".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::CheckNextAction {
            input: PathBuf::from("nuis.build.manifest.toml"),
            json: true
        })
    );
}

#[test]
fn parses_drive_input_and_json_flag() {
    let command = parse_args(
        vec![
            "drive".to_owned(),
            "nuis.build.manifest.toml".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::Drive {
            input: PathBuf::from("nuis.build.manifest.toml"),
            json: true,
            apply: false,
            until_clean: false
        })
    );
}

#[test]
fn parses_drive_apply_input_and_json_flag() {
    let command = parse_args(
        vec![
            "drive".to_owned(),
            "nuis.build.manifest.toml".to_owned(),
            "--apply".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::Drive {
            input: PathBuf::from("nuis.build.manifest.toml"),
            json: true,
            apply: true,
            until_clean: false
        })
    );
}

#[test]
fn parses_drive_apply_until_clean_flag() {
    let command = parse_args(
        vec![
            "drive".to_owned(),
            "nuis.build.manifest.toml".to_owned(),
            "--apply".to_owned(),
            "--until-clean".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::Drive {
            input: PathBuf::from("nuis.build.manifest.toml"),
            json: true,
            apply: true,
            until_clean: true
        })
    );
}

#[test]
fn rejects_drive_until_clean_without_apply() {
    let error = parse_args(
        vec![
            "drive".to_owned(),
            "nuis.build.manifest.toml".to_owned(),
            "--until-clean".to_owned(),
        ]
        .into_iter(),
    )
    .unwrap_err();

    assert!(error.contains("requires `--apply`"));
}

#[test]
fn parses_artifact_chain_input_and_json_flag() {
    let command = parse_args(
        vec![
            "artifact-chain".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::ArtifactChain {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_closure_input_and_json_flag() {
    let command =
        parse_args(vec!["closure".to_owned(), "out".to_owned(), "--json".to_owned()].into_iter());
    assert_eq!(
        command,
        Ok(Command::Closure {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_closure_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-closure".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitClosure {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_closure_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-closure".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyClosure {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_final_stage_plan_input_and_json_flag() {
    let command = parse_args(
        vec![
            "final-stage-plan".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::FinalStagePlan {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_final_stage_plan_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-final-stage-plan".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitFinalStagePlan {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_final_stage_plan_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-final-stage-plan".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyFinalStagePlan {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_prepare_input_and_json_flag() {
    let command =
        parse_args(vec!["prepare".to_owned(), "out".to_owned(), "--json".to_owned()].into_iter());
    assert_eq!(
        command,
        Ok(Command::Prepare {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_assemble_plan_input_and_json_flag() {
    let command = parse_args(
        vec![
            "assemble-plan".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::AssemblePlan {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_assemble_plan_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-assemble-plan".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitAssemblePlan {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_assemble_plan_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-assemble-plan".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyAssemblePlan {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_section_manifest_input_and_json_flag() {
    let command = parse_args(
        vec![
            "section-manifest".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::SectionManifest {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_section_manifest_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-section-manifest".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitSectionManifest {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_section_manifest_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-section-manifest".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifySectionManifest {
            input: PathBuf::from("out"),
            json: true
        })
    );
}
