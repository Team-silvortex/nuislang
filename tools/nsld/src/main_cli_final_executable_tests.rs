use super::{parse_args, Command};
use std::path::PathBuf;

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
fn parses_final_executable_writer_plan_input_and_json_flag() {
    let command = parse_args(
        vec![
            "final-executable-writer-plan".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::FinalExecutableWriterPlan {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_final_executable_writer_input_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-final-executable-writer-input".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitFinalExecutableWriterInput {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_final_executable_writer_input_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-final-executable-writer-input".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyFinalExecutableWriterInput {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_final_executable_host_dry_run_input_and_json_flag() {
    let command = parse_args(
        vec![
            "final-executable-host-dry-run".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::FinalExecutableHostDryRun {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_final_executable_host_invoke_plan_input_and_json_flag() {
    let command = parse_args(
        vec![
            "final-executable-host-invoke-plan".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::FinalExecutableHostInvokePlan {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_final_executable_host_invoke_plan_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-final-executable-host-invoke-plan".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitFinalExecutableHostInvokePlan {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_final_executable_host_invoke_plan_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-final-executable-host-invoke-plan".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyFinalExecutableHostInvokePlan {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_final_executable_layout_input_and_json_flag() {
    let command = parse_args(
        vec![
            "final-executable-layout".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::FinalExecutableLayout {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_final_executable_layout_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-final-executable-layout".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitFinalExecutableLayout {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_final_executable_layout_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-final-executable-layout".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyFinalExecutableLayout {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_final_executable_image_dry_run_input_and_json_flag() {
    let command = parse_args(
        vec![
            "final-executable-image-dry-run".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::FinalExecutableImageDryRun {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_final_executable_image_dry_run_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-final-executable-image-dry-run".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitFinalExecutableImageDryRun {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_final_executable_image_dry_run_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-final-executable-image-dry-run".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyFinalExecutableImageDryRun {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_final_executable_pipeline_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-final-executable-pipeline".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitFinalExecutablePipeline {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_final_executable_pipeline_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-final-executable-pipeline".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyFinalExecutablePipeline {
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
fn parses_final_executable_output_input_and_json_flag() {
    let command = parse_args(
        vec![
            "final-executable-output".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::FinalExecutableOutput {
            input: PathBuf::from("out"),
            json: true
        })
    );
}
