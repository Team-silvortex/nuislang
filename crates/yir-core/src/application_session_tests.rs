use super::*;
use crate::YirFunctionResult;

fn fixture() -> YirModule {
    let mut module = YirModule::new("0.1");
    for (name, parameters) in [
        ("start", vec!["seed"]),
        ("step", vec!["state.count", "input"]),
        ("stop", vec!["state.count"]),
    ] {
        module.functions.push(YirFunction {
            name: name.to_owned(),
            domain: "cpu".to_owned(),
            role: YirFunctionRole::Helper,
            parameters: parameters
                .iter()
                .enumerate()
                .map(|(index, name)| YirFunctionParameter {
                    name: (*name).to_owned(),
                    ty: "i64".to_owned(),
                    ownership: YirValueOwnership::Value,
                    node: format!("arg{index}"),
                })
                .collect(),
            result: Some(YirFunctionResult {
                ty: "Counter".to_owned(),
                ownership: YirValueOwnership::Owned,
                node: "result".to_owned(),
            }),
            body_nodes: Vec::new(),
        });
    }
    module.application_sessions.push(
        YirApplicationSession::from_fields(&[
            "primary",
            APPLICATION_SESSION_CONTRACT,
            "start",
            "step",
            "stop",
            "state",
        ])
        .unwrap(),
    );
    module
}

#[test]
fn registry_has_explicit_ids_and_no_default_application() {
    let mut module = fixture();
    let mut second = module.application_sessions[0].clone();
    second.id = "independent".to_owned();
    module.application_sessions.push(second);
    assert_eq!(
        registered_application_session(&module, "independent")
            .unwrap()
            .event,
        "step"
    );
    assert!(registered_application_session(&module, "missing").is_err());
    let signature =
        ApplicationSessionSignature::bind(&module, module.application_sessions[0].entries())
            .unwrap();
    assert_eq!(signature.state_parameters.len(), 1);
    assert_eq!(signature.state_type, "Counter");
    module.application_sessions[1].id = "primary".to_owned();
    assert!(validate_application_sessions(&module)
        .unwrap_err()
        .contains("duplicate"));
}

#[test]
fn signature_drift_is_rejected_by_the_shared_contract() {
    for case in 0..7 {
        let mut module = fixture();
        match case {
            0 => module.application_sessions[0].event = "missing".to_owned(),
            1 => module.functions[1].role = YirFunctionRole::Entry,
            2 => module.functions[2].parameters[0].ty = "i32".to_owned(),
            3 => module.functions[1].parameters[0].ownership = YirValueOwnership::Borrowed,
            4 => module.functions[2].result.as_mut().unwrap().ty = "Other".to_owned(),
            5 => module.functions.push(module.functions[0].clone()),
            _ => module.functions[1].parameters[1].name = "state.count".to_owned(),
        }
        assert!(
            validate_application_sessions(&module).is_err(),
            "accepted drift {case}"
        );
    }
}

#[test]
fn registration_shape_version_and_bounds_are_checked() {
    for fields in [
        vec!["id", "v2", "start", "step", "stop", "state"],
        vec![
            "id",
            APPLICATION_SESSION_CONTRACT,
            "start",
            "start",
            "stop",
            "state",
        ],
        vec![
            "id",
            APPLICATION_SESSION_CONTRACT,
            "start",
            "step",
            "stop",
            "state.field",
        ],
        vec!["id", APPLICATION_SESSION_CONTRACT, "start", "step", "stop"],
        vec![
            "bad id",
            APPLICATION_SESSION_CONTRACT,
            "start",
            "step",
            "stop",
            "state",
        ],
    ] {
        assert!(YirApplicationSession::from_fields(&fields).is_err());
    }
    let mut module = fixture();
    module.application_sessions = (0..65)
        .map(|id| {
            let mut registration = module.application_sessions[0].clone();
            registration.id = format!("session{id}");
            registration
        })
        .collect();
    assert!(validate_application_sessions(&module)
        .unwrap_err()
        .contains("64"));
}
