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
