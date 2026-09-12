use super::*;
use helpers::{edge, helper, push_node, root_call};

const SCOPED: &str = include_str!("scoped_loops.ns");

fn rejected(module: &YirModule, message: &str) {
    let error = emit_registered(module, "counter").unwrap_err();
    assert!(error.contains(message), "expected {message}: {error}");
}

// A zero-trip scoped edge still belongs to the admitted closure. A dead runtime
// path is not permission to hide recursion, an unbounded graph or side effects.
fn scoped_helper(module: &mut YirModule, name: &str, callee: &str) {
    helper(module, name, &[], 2);
    let initial = format!("{name}_initial");
    let step = format!("{name}_step");
    push_node(module, name, &initial, "const_i64", vec!["0".to_owned()]);
    push_node(module, name, &step, "const_i64", vec!["1".to_owned()]);
    let loop_name = format!("{name}_v0");
    module
        .nodes
        .iter_mut()
        .find(|n| n.name == loop_name)
        .unwrap()
        .op = yir_core::Operation::parse(
        "cpu.loop_while_i64_effect",
        vec![
            initial.clone(),
            initial.clone(),
            step.clone(),
            "lt".to_owned(),
            "add".to_owned(),
            "cpu".to_owned(),
            "scoped_call".to_owned(),
            "1".to_owned(),
            callee.to_owned(),
        ],
    )
    .unwrap();
    edge(module, &initial, &loop_name);
    edge(module, &step, &loop_name);
    module
        .functions
        .iter_mut()
        .find(|f| f.name == name)
        .unwrap()
        .body_nodes
        .extend([initial, step]);
}

#[test]
fn scoped_call_edges_share_recursion_depth_and_function_bounds() {
    let project = Project::new();
    let base = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    for mutual in [false, true] {
        let mut module = base.clone();
        scoped_helper(
            &mut module,
            "scoped_a",
            if mutual { "scoped_b" } else { "scoped_a" },
        );
        if mutual {
            helper(&mut module, "scoped_b", &["scoped_a".to_owned()], 2);
        }
        assert!(!emit_registered(&module, "counter")
            .unwrap()
            .llvm_ir
            .contains("@nuis_fn_scoped_"));
        root_call(&mut module, "scoped_a");
        rejected(&module, "recursive helper call cycles");
    }
    for count in [31, 32] {
        let mut module = base.clone();
        for index in 0..count {
            let name = format!("scoped_{index}");
            if index + 1 == count {
                helper(&mut module, &name, &[], 2);
            } else {
                scoped_helper(&mut module, &name, &format!("scoped_{}", index + 1));
            }
        }
        root_call(&mut module, "scoped_0");
        if count == 31 {
            emit_registered(&module, "counter").unwrap();
        } else {
            rejected(&module, "call-depth bound");
        }
    }
    for count in [60, 61] {
        let mut module = base.clone();
        for index in 0..count {
            let name = format!("scoped_{index}");
            scoped_helper(&mut module, &name, "leaf");
            root_call(&mut module, &name);
        }
        helper(&mut module, "leaf", &[], 2);
        if count == 60 {
            emit_registered(&module, "counter").unwrap();
        } else {
            rejected(&module, "reachable-function bound");
        }
    }
}

#[test]
fn scoped_actions_reject_target_payload_and_result_drift() {
    let project = Project::with_source(SCOPED);
    let base = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    let index = base
        .nodes
        .iter()
        .position(|n| n.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carry"))
        .unwrap();
    for mutation in [
        "unknown",
        "arity",
        "return-kind",
        "duplicate-carry",
        "copy",
        "action",
    ] {
        let mut module = base.clone();
        let expected = match mutation {
            "unknown" => {
                module.nodes[index].op.args[8] = "unknown_scoped_helper".to_owned();
                "unknown helper"
            }
            "arity" => {
                let args = &mut module.nodes[index].op.args;
                args.push("$current".to_owned());
                args[7] = (args.len() - 8).to_string();
                "helper signature drift"
            }
            "return-kind" => {
                let callee = &module.nodes[index].op.args[8];
                module
                    .functions
                    .iter_mut()
                    .find(|f| &f.name == callee)
                    .unwrap()
                    .result
                    .as_mut()
                    .unwrap()
                    .ty = "bool".to_owned();
                "helper signature drift"
            }
            "duplicate-carry" => {
                let args = &mut module.nodes[index].op.args;
                let current = args.iter().position(|arg| arg == "$current").unwrap();
                args[current] = "$carry".to_owned();
                "scoped_call_i64_carry payload"
            }
            "copy" => {
                let args = &mut module.nodes[index].op.args;
                let last = args.len() - 1;
                args[last] = format!("copy_owned:{}", args[last]);
                "scoped_call_i64_carry payload"
            }
            "action" => {
                let args = &mut module.nodes[index].op.args;
                let initial = args[0].clone();
                args.truncate(8);
                args[6] = "owned_bytes_copy_drop".to_owned();
                args[7] = "1".to_owned();
                args.push(initial);
                "does not admit this scoped action"
            }
            _ => unreachable!(),
        };
        rejected(&module, expected);
    }
}

#[test]
fn scoped_captures_require_exact_types_and_the_callers_own_lane() {
    let project = Project::with_source(SCOPED);
    let base = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    let index = base
        .nodes
        .iter()
        .position(|n| {
            n.op.args.get(6).map(String::as_str) == Some("scoped_call") && n.op.args.len() == 14
        })
        .unwrap();
    for mutation in ["current-kind", "capture-kind", "foreign-lane"] {
        let mut module = base.clone();
        let expected = match mutation {
            "current-kind" => {
                let callee = &module.nodes[index].op.args[8];
                let function = module
                    .functions
                    .iter_mut()
                    .find(|f| &f.name == callee)
                    .unwrap();
                function.parameters[0].ty = "i32".to_owned();
                let node = function.parameters[0].node.clone();
                module
                    .nodes
                    .iter_mut()
                    .find(|n| n.name == node)
                    .unwrap()
                    .op
                    .instruction = "param_i32".to_owned();
                "exact i64 loop-state parameters"
            }
            "capture-kind" => {
                module.nodes[index].op.args[10] = module.nodes[index].op.args[11].clone();
                "declared scalar kind"
            }
            "foreign-lane" => {
                let root = &module.application_sessions[0].open;
                let capture = module
                    .functions
                    .iter()
                    .find(|f| &f.name == root)
                    .unwrap()
                    .parameters
                    .iter()
                    .find(|p| p.ty == "bool")
                    .unwrap()
                    .node
                    .clone();
                let loop_name = module.nodes[index].name.clone();
                module.nodes[index].op.args[11] = capture.clone();
                edge(&mut module, &capture, &loop_name);
                "external initialization or cross-function values"
            }
            _ => unreachable!(),
        };
        rejected(&module, expected);
    }
    let mut module = base.clone();
    let index = module
        .nodes
        .iter()
        .position(|n| n.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carry"))
        .unwrap();
    let boolean = module.nodes[index].op.args.last().unwrap().clone();
    module.nodes[index].op.args[9] = boolean;
    rejected(&module, "declared scalar kind");
}

#[test]
fn scoped_callee_effects_are_checked_even_for_discarded_and_zero_trip_calls() {
    let source = SCOPED
        .replace(
            "fn ping() -> i64 { return 0; }",
            "fn ping() -> i64 { print(777); return 0; }",
        )
        .replace("idle(3)", "idle(0)");
    let project = Project::with_source(&source);
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    rejected(&module, "does not admit cpu.print");
}
