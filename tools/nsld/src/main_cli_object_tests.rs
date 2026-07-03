use super::{parse_args, Command};
use std::path::PathBuf;

#[test]
fn parses_object_plan_input_and_json_flag() {
    let command = parse_args(
        vec![
            "object-plan".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::ObjectPlan {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_object_plan_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-object-plan".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitObjectPlan {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_object_plan_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-object-plan".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyObjectPlan {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_object_writer_readiness_input_and_json_flag() {
    let command = parse_args(
        vec![
            "object-writer-readiness".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::ObjectWriterReadiness {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_object_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-object".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitObject {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_object_writer_input_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-object-writer-input".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyObjectWriterInput {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_object_writer_dry_run_input_and_json_flag() {
    let command = parse_args(
        vec![
            "object-writer-dry-run".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::ObjectWriterDryRun {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_object_writer_dry_run_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-object-writer-dry-run".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitObjectWriterDryRun {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_object_writer_dry_run_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-object-writer-dry-run".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyObjectWriterDryRun {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_object_byte_layout_input_and_json_flag() {
    let command = parse_args(
        vec![
            "object-byte-layout".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::ObjectByteLayout {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_emit_object_byte_layout_input_and_json_flag() {
    let command = parse_args(
        vec![
            "emit-object-byte-layout".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::EmitObjectByteLayout {
            input: PathBuf::from("out"),
            json: true
        })
    );
}

#[test]
fn parses_verify_object_byte_layout_input_and_json_flag() {
    let command = parse_args(
        vec![
            "verify-object-byte-layout".to_owned(),
            "out".to_owned(),
            "--json".to_owned(),
        ]
        .into_iter(),
    );
    assert_eq!(
        command,
        Ok(Command::VerifyObjectByteLayout {
            input: PathBuf::from("out"),
            json: true
        })
    );
}
