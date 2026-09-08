use super::*;
use yir_core::{Operation, ResourceKind};

fn resource() -> Resource {
    Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    }
}

fn node(name: &str, instruction: &str, args: &[&str]) -> Node {
    Node {
        name: name.to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            instruction,
            args.iter().map(|arg| (*arg).to_owned()).collect(),
        )
        .unwrap(),
    }
}

#[test]
fn logical_operations_preserve_bool_values() {
    let mut state = ExecutionState::default();
    state.bind_value("truthy", Value::Bool(true));
    state.bind_value("falsy", Value::Bool(false));

    assert_eq!(
        CpuMod
            .execute(
                &node("both", "cpu.and", &["truthy", "falsy"]),
                &resource(),
                &mut state,
            )
            .unwrap(),
        Value::Bool(false)
    );
    assert_eq!(
        CpuMod
            .execute(
                &node("either", "cpu.or", &["truthy", "falsy"]),
                &resource(),
                &mut state,
            )
            .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        CpuMod
            .execute(
                &node("inverse", "cpu.not", &["falsy"]),
                &resource(),
                &mut state,
            )
            .unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn bitwise_operations_keep_their_i64_contract() {
    let mut state = ExecutionState::default();
    state.bind_value("lhs", Value::Int(6));
    state.bind_value("rhs", Value::Int(3));

    assert_eq!(
        CpuMod
            .execute(
                &node("masked", "cpu.and", &["lhs", "rhs"]),
                &resource(),
                &mut state,
            )
            .unwrap(),
        Value::Int(2)
    );
}

#[test]
fn invalid_integer_divisors_report_errors_without_panicking() {
    for instruction in ["cpu.div", "cpu.rem", "cpu.div_i32"] {
        for (lhs, rhs, diagnostic) in [(7, 0, "zero"), (i64::MIN, -1, "overflow")] {
            let mut state = ExecutionState::default();
            if instruction == "cpu.div_i32" {
                state.bind_value("lhs", Value::I32(if rhs == 0 { 7 } else { i32::MIN }));
                state.bind_value("rhs", Value::I32(rhs as i32));
            } else {
                state.bind_value("lhs", Value::Int(lhs));
                state.bind_value("rhs", Value::Int(rhs));
            }
            let error = CpuMod
                .execute(
                    &node("invalid", instruction, &["lhs", "rhs"]),
                    &resource(),
                    &mut state,
                )
                .unwrap_err();
            assert!(error.contains(diagnostic), "{error}");
        }
    }
}

#[test]
fn generic_comparisons_feed_logical_operations_as_bool_values() {
    let mut state = ExecutionState::default();
    state.bind_value("low", Value::Int(2));
    state.bind_value("high", Value::Int(7));

    let below = CpuMod
        .execute(
            &node("below", "cpu.lt", &["low", "high"]),
            &resource(),
            &mut state,
        )
        .unwrap();
    state.bind_value("below", below);
    let distinct = CpuMod
        .execute(
            &node("distinct", "cpu.ne", &["low", "high"]),
            &resource(),
            &mut state,
        )
        .unwrap();
    state.bind_value("distinct", distinct);

    assert_eq!(
        CpuMod
            .execute(
                &node("valid", "cpu.and", &["below", "distinct"]),
                &resource(),
                &mut state,
            )
            .unwrap(),
        Value::Bool(true)
    );
}
