use super::{parse_args, Command};
use std::path::PathBuf;

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
