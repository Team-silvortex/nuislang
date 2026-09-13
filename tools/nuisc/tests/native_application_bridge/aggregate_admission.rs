use super::*;
use helpers::{edge, push_node};

fn compiled() -> YirModule {
    let source = aggregate_execution::source(2, "<");
    let project = Project::with_source(&source);
    let mut module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    let function = module
        .functions
        .iter()
        .find(|f| f.name == "branch")
        .unwrap();
    let returned = function.result.as_ref().unwrap().node.clone();
    let flag = function
        .parameters
        .iter()
        .find(|p| p.ty == "bool")
        .unwrap()
        .node
        .clone();
    let result = module.nodes.iter().find(|n| n.name == returned).unwrap();
    let value = result.op.args[0].clone();
    let layout = result.op.args[1].clone();
    push_node(
        &mut module,
        "branch",
        "aggregate_guard",
        "guard_return",
        vec![flag.clone(), value.clone(), layout],
    );
    module
        .functions
        .iter_mut()
        .find(|f| f.name == "branch")
        .unwrap()
        .body_nodes
        .push("aggregate_guard".to_owned());
    edge(&mut module, &flag, "aggregate_guard");
    edge(&mut module, &value, "aggregate_guard");
    edge(&mut module, "aggregate_guard", &returned);
    module
}

fn call_index(module: &YirModule) -> usize {
    module
        .nodes
        .iter()
        .position(|n| n.op.instruction == "call_owned_struct" && n.op.args[0] == "branch")
        .unwrap()
}

fn rejected(module: &YirModule, expected: &str) {
    let error = emit_registered(module, "counter").unwrap_err();
    assert!(error.contains(expected), "expected {expected}: {error}");
}

#[test]
fn flat_aggregate_calls_require_exact_call_result_and_parameter_contracts() {
    let base = compiled();
    emit_registered(&base, "counter").unwrap();
    let index = call_index(&base);
    for (slot, value, error) in [
        (0, "missing_helper", "unknown helper"),
        (1, "Wrong{carry0:i64;carry1:i64}", "signature drift"),
        (1, "Carries{carry0:i64}", "signature drift"),
        (
            1,
            "Carries{carry0:i64;carry1:bool}",
            "flat i64 carry layout",
        ),
        (
            1,
            "Carries{carry0:i64;carry1:Nested{x:i64}}",
            "flat i64 carry layout",
        ),
        (
            1,
            "Carries{carry0:i64;carry1:Bytes}",
            "flat i64 carry layout",
        ),
    ] {
        let mut drift = base.clone();
        drift.nodes[index].op.args[slot] = value.to_owned();
        rejected(&drift, error);
    }
    for mutation in [
        "missing-layout",
        "arity",
        "result-ownership",
        "result-type",
        "parameter-kind",
        "parameter-ownership",
        "capture",
        "return-layout",
        "guard-layout",
        "return-field",
        "guard-field",
    ] {
        let mut drift = base.clone();
        let function = drift
            .functions
            .iter_mut()
            .find(|f| f.name == "branch")
            .unwrap();
        let returned = function.result.as_ref().unwrap().node.clone();
        let flag = function
            .parameters
            .iter()
            .find(|p| p.ty == "bool")
            .unwrap()
            .node
            .clone();
        let expected = match mutation {
            "missing-layout" => {
                drift.nodes[index].op.args.truncate(1);
                "missing target/layout"
            }
            "arity" => {
                drift.nodes[index].op.args.pop();
                "signature drift"
            }
            "result-ownership" => {
                function.result.as_mut().unwrap().ownership = yir_core::YirValueOwnership::Value;
                "return layout drift"
            }
            "result-type" => {
                function.result.as_mut().unwrap().ty = "Wrong".to_owned();
                "return layout drift"
            }
            "parameter-ownership" => {
                function.parameters[0].ownership = yir_core::YirValueOwnership::Owned;
                "parameter node/signature drift"
            }
            "parameter-kind" => {
                function.parameters[0].ty = "bool".to_owned();
                let param = &function.parameters[0].node;
                drift
                    .nodes
                    .iter_mut()
                    .find(|n| &n.name == param)
                    .unwrap()
                    .op
                    .instruction = "param_bool".to_owned();
                "declared scalar kind"
            }
            "capture" => {
                let caller_flag = drift.nodes[index].op.args[6].clone();
                drift.nodes[index].op.args[2] = caller_flag;
                "declared scalar kind"
            }
            "return-layout" | "guard-layout" => {
                let (name, arg) = if mutation == "return-layout" {
                    (returned.as_str(), 1)
                } else {
                    ("aggregate_guard", 2)
                };
                drift
                    .nodes
                    .iter_mut()
                    .find(|n| n.name == name)
                    .unwrap()
                    .op
                    .args[arg] = "Wrong{carry0:i64;carry1:i64}".to_owned();
                "layout"
            }
            "return-field" | "guard-field" => {
                let name = if mutation == "return-field" {
                    returned.as_str()
                } else {
                    "aggregate_guard"
                };
                let result = drift.nodes.iter().find(|n| n.name == name).unwrap();
                let value = result.op.args[usize::from(mutation == "guard-field")].clone();
                let leaf = "invalid_aggregate_return";
                push_node(
                    &mut drift,
                    "branch",
                    leaf,
                    "struct",
                    vec![
                        "Carries".to_owned(),
                        format!("carry0={flag}"),
                        format!("carry1={flag}"),
                    ],
                );
                drift
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "branch")
                    .unwrap()
                    .body_nodes
                    .push(leaf.to_owned());
                drift
                    .nodes
                    .iter_mut()
                    .find(|n| n.name == name)
                    .unwrap()
                    .op
                    .args[usize::from(mutation == "guard-field")] = leaf.to_owned();
                edge(&mut drift, &flag, leaf);
                edge(&mut drift, &value, leaf);
                edge(&mut drift, leaf, name);
                "does not match"
            }
            _ => unreachable!(),
        };
        rejected(&drift, expected);
    }
}

#[test]
fn flat_aggregate_edges_reject_hidden_effects_foreign_values_and_recursive_helpers() {
    let base = compiled();
    let index = call_index(&base);
    let function = base.functions.iter().find(|f| f.name == "branch").unwrap();
    let returned = &function.result.as_ref().unwrap().node;
    for mutation in ["self-cycle", "discarded-effect", "foreign-capture"] {
        let mut drift = base.clone();
        let expected = match mutation {
            "self-cycle" => {
                let mut args = drift.nodes[index].op.args[..2].to_vec();
                args.extend(function.parameters.iter().map(|p| p.node.clone()));
                push_node(
                    &mut drift,
                    "branch",
                    "recursive_aggregate",
                    "call_owned_struct",
                    args,
                );
                drift
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "branch")
                    .unwrap()
                    .body_nodes
                    .push("recursive_aggregate".to_owned());
                for param in &function.parameters {
                    edge(&mut drift, &param.node, "recursive_aggregate");
                }
                edge(&mut drift, "recursive_aggregate", returned);
                "recursive helper call cycles"
            }
            "discarded-effect" => {
                push_node(
                    &mut drift,
                    "branch",
                    "hidden_aggregate_print",
                    "print",
                    vec![function.parameters[0].node.clone()],
                );
                drift
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "branch")
                    .unwrap()
                    .body_nodes
                    .push("hidden_aggregate_print".to_owned());
                edge(
                    &mut drift,
                    &function.parameters[0].node,
                    "hidden_aggregate_print",
                );
                edge(&mut drift, "hidden_aggregate_print", returned);
                "does not admit cpu.print"
            }
            "foreign-capture" => {
                drift.nodes[index].op.args[2] = function.parameters[0].node.clone();
                let call = drift.nodes[index].name.clone();
                edge(&mut drift, &function.parameters[0].node, &call);
                "cross-function values"
            }
            _ => unreachable!(),
        };
        rejected(&drift, expected);
    }
}
