use super::*;
use yir_core::{ExecutionState, Operation, RegisteredMod, Resource, ResourceKind, StructValue};

fn parameter(args: &[&str]) -> Node {
    Node {
        name: "input".into(),
        resource: "cpu0".into(),
        op: Operation::parse(
            "cpu.param_value_struct",
            args.iter().map(|arg| (*arg).into()).collect(),
        )
        .unwrap(),
    }
}

fn record(name: &str, fields: Vec<(&str, Value)>) -> Value {
    Value::Struct(StructValue {
        type_name: name.into(),
        fields: fields
            .into_iter()
            .map(|(name, value)| (name.into(), value))
            .collect(),
    })
}

#[test]
fn value_parameter_accepts_only_bounded_resource_free_nominal_layouts() {
    for slots in [1, 64] {
        let fields = (0..slots)
            .map(|i| format!("f{i}:i64"))
            .collect::<Vec<_>>()
            .join(";");
        assert!(parse(&parameter(&[
            "0",
            &format!("State{{payload:Payload{{{fields}}}}}")
        ]))
        .unwrap()
        .is_some());
    }
    let oversized = format!(
        "State{{{}}}",
        (0..65)
            .map(|i| format!("f{i}:i64"))
            .collect::<Vec<_>>()
            .join(";")
    );
    for layout in [
        "State{}",
        "State{x:text}",
        "State{x:ptr}",
        "State{x:i64;x:i32}",
        &oversized,
    ] {
        assert!(parse(&parameter(&["0", layout])).is_err(), "{layout}");
    }
    for args in [
        vec![],
        vec!["0"],
        vec!["-1", "State{x:i64}"],
        vec!["0", "State{x:i64}", "extra"],
    ] {
        assert!(parse(&parameter(&args)).is_err());
    }
}

#[test]
fn value_parameter_requires_exact_nested_types_field_sets_and_scalar_kinds() {
    let contract = parse(&parameter(&[
        "2",
        "State{tag:bool;payload:Payload{x:i32;y:f32;z:f64}}",
    ]))
    .unwrap()
    .unwrap();
    let payload = || {
        record(
            "Payload",
            vec![
                ("z", Value::F64(f64::from_bits(0x7ff8_0000_0000_1234))),
                ("y", Value::F32(-0.0)),
                ("x", Value::I32(-17)),
            ],
        )
    };
    let good = record(
        "State",
        vec![("payload", payload()), ("tag", Value::Bool(true))],
    );
    contract.validate(&good).unwrap();
    for value in [
        record(
            "Other",
            vec![("tag", Value::Bool(true)), ("payload", payload())],
        ),
        record(
            "State",
            vec![("tag", Value::Int(1)), ("payload", payload())],
        ),
        record(
            "State",
            vec![
                ("tag", Value::Bool(true)),
                (
                    "payload",
                    record(
                        "Payload",
                        vec![
                            ("x", Value::Int(-17)),
                            ("y", Value::F32(-0.0)),
                            ("z", Value::F64(1.0)),
                        ],
                    ),
                ),
            ],
        ),
        record(
            "State",
            vec![
                ("tag", Value::Bool(true)),
                (
                    "payload",
                    record(
                        "Other",
                        vec![
                            ("x", Value::I32(-17)),
                            ("y", Value::F32(-0.0)),
                            ("z", Value::F64(1.0)),
                        ],
                    ),
                ),
            ],
        ),
        record(
            "State",
            vec![("tag", Value::Bool(true)), ("tag", Value::Bool(false))],
        ),
        record("State", vec![("tag", Value::Bool(true))]),
        record(
            "State",
            vec![
                ("tag", Value::Bool(true)),
                ("payload", payload()),
                ("extra", Value::Int(0)),
            ],
        ),
        Value::Int(0),
    ] {
        assert!(contract.validate(&value).is_err());
    }
}

#[test]
fn value_parameter_has_no_variant_conversion_or_unbound_default() {
    let node = parameter(&["0", "__nuis_variant_union__State{x:i64}"]);
    let contract = parse(&node).unwrap().unwrap();
    contract
        .validate(&record(
            "__nuis_variant_union__State",
            vec![("x", Value::Int(7))],
        ))
        .unwrap();
    assert!(contract
        .validate(&record("Other", vec![("x", Value::Int(7))]))
        .is_err());
    let error = crate::CpuMod
        .execute(
            &node,
            &Resource {
                name: "cpu0".into(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            &mut ExecutionState::default(),
        )
        .unwrap_err();
    assert!(
        error.contains("requires a function argument binding"),
        "{error}"
    );
}
