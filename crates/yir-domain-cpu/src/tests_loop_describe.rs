use super::*;
use yir_core::{Operation, ResourceKind};

fn cpu_resource() -> Resource {
    Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    }
}

fn flow_node(args: Vec<&str>) -> Node {
    Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_flow_chain",
            args.into_iter().map(str::to_owned).collect(),
        )
        .unwrap(),
    }
}

#[test]
fn flow_chain_without_carries_has_bounded_semantics() {
    let node = flow_node(vec![
        "initial",
        "limit",
        "step",
        "lt",
        "add",
        "current_ge",
        "control_rhs",
        "break",
    ]);
    let semantics = describe_cpu_node(&node, &cpu_resource()).unwrap();
    assert!(semantics.has_effect);
    assert_eq!(
        semantics.dependencies,
        ["initial", "limit", "step", "control_rhs"]
    );
}

#[test]
fn flow_chain_collects_each_carry_initial() {
    let node = flow_node(vec![
        "initial",
        "limit",
        "step",
        "lt",
        "add",
        "current_ge",
        "control_rhs",
        "continue",
        "carry_initial",
        "add_current",
    ]);
    let semantics = describe_cpu_node(&node, &cpu_resource()).unwrap();
    assert_eq!(
        semantics.dependencies,
        ["initial", "limit", "step", "control_rhs", "carry_initial"]
    );
}
