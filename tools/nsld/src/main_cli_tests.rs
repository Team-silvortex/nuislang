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
fn parses_final_executable_readiness_input_and_json_flag() {
    let command = parse_args(
        vec![
            "final-executable-readiness".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::FinalExecutableReadiness {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_final_executable_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-final-executable".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitFinalExecutable {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_final_executable_emit_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-final-executable-emit".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyFinalExecutableEmit {
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

#[test]
fn parses_container_plan_input_and_json_flag() {
    let command = parse_args(
        vec![
            "container-plan".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::ContainerPlan {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_container_plan_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-container-plan".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitContainerPlan {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_container_plan_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-container-plan".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyContainerPlan {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_container_input_and_json_flag() {
    let command = parse_args(
        vec![
            "container".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::Container {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_container_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-container".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitContainer {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_container_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-container".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyContainer {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_bundle_input_and_json_flag() {
    let command =
        parse_args(vec!["bundle".to_owned(), "out".to_owned(), "--json".to_owned()].into_iter());
    assert_eq!(
        command,
        Ok(Command::Bundle {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_bundle_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-bundle".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitBundle {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_bundle_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-bundle".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyBundle {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_units_input_and_json_flag() {
    let command =
        parse_args(vec!["units".to_owned(), "out".to_owned(), "--json".to_owned()].into_iter());
    assert_eq!(
        command,
        Ok(Command::Units {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_units_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-units".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitUnits {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_units_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-units".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyUnits {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_inputs_input_and_json_flag() {
    let command =
        parse_args(vec!["inputs".to_owned(), "out".to_owned(), "--json".to_owned()].into_iter());
    assert_eq!(
        command,
        Ok(Command::Inputs {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_inputs_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-inputs".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitInputs {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_inputs_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-inputs".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyInputs {
            input: PathBuf::from("out"),
            json: true
        })
    );
}
