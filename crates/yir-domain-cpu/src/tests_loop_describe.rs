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

#[test]
fn post_flow_chain_accepts_an_unbounded_entry_condition() {
    let node = Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_post_flow_chain",
            [
                "initial",
                "unused_limit",
                "step",
                "always",
                "add",
                "carry0_ge",
                "control_rhs",
                "break",
                "carry_initial",
                "add_current",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        )
        .unwrap(),
    };
    let semantics = describe_cpu_node(&node, &cpu_resource()).unwrap();
    assert!(semantics.has_effect);
    assert_eq!(
        semantics.dependencies,
        [
            "initial",
            "unused_limit",
            "step",
            "control_rhs",
            "carry_initial",
        ]
    );
}

#[test]
fn post_flow_chain_accepts_an_invariant_pattern_gate() {
    let node = Node {
        name: "loop".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            "cpu.loop_while_scalar_post_flow_chain",
            [
                "initial",
                "pattern_condition",
                "step",
                "invariant_true",
                "add",
                "current_ge",
                "control_rhs",
                "break",
                "carry_initial",
                "add_current",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        )
        .unwrap(),
    };
    let semantics = describe_cpu_node(&node, &cpu_resource()).unwrap();
    assert!(semantics.has_effect);
    assert!(semantics
        .dependencies
        .iter()
        .any(|dependency| dependency == "pattern_condition"));
}

#[test]
fn guarded_loop_continue_effects_preserve_their_inputs() {
    for (instruction, args, expected) in [
        (
            "cpu.guard_loop_continue",
            vec!["condition"],
            vec!["condition"],
        ),
        (
            "cpu.guard_loop_print_continue",
            vec!["condition", "shown"],
            vec!["condition", "shown"],
        ),
    ] {
        let node = Node {
            name: "repeat".to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation::parse(instruction, args.into_iter().map(str::to_owned).collect())
                .unwrap(),
        };
        let semantics = describe_cpu_node(&node, &cpu_resource()).unwrap();
        assert!(semantics.has_effect);
        assert_eq!(semantics.dependencies, expected);
    }
}
