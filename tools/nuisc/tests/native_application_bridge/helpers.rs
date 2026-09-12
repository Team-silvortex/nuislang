use super::*;

const HELPERS: &str = include_str!("helpers.ns");

#[test]
fn acyclic_scalar_helpers_execute_with_typed_native_reference_parity() {
    assert_native_parity(HELPERS, true);
}

fn rejected(module: &YirModule, expected: &str) {
    let error = emit_registered(module, "counter").unwrap_err();
    assert!(error.contains(expected), "expected {expected}: {error}");
}

#[test]
fn reachable_helpers_reject_signature_lane_effect_and_dependency_drift() {
    let project = Project::with_source(HELPERS);
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    let call = module
        .nodes
        .iter()
        .find(|n| n.op.instruction == "call_f32")
        .unwrap();
    let callee = module
        .functions
        .iter()
        .find(|f| f.name == call.op.args[0])
        .unwrap();
    let parameter = &callee.parameters[0].node;
    for mutation in [
        "unknown",
        "call-kind",
        "arity",
        "parameter",
        "index",
        "lane",
        "global",
    ] {
        let mut drift = module.clone();
        let expected = match mutation {
            "unknown" => {
                drift
                    .nodes
                    .iter_mut()
                    .find(|n| n.name == call.name)
                    .unwrap()
                    .op
                    .args[0] = "missing_helper".to_owned();
                "unknown helper"
            }
            "call-kind" => {
                drift
                    .nodes
                    .iter_mut()
                    .find(|n| n.name == call.name)
                    .unwrap()
                    .op
                    .instruction = "call_f64".to_owned();
                "signature drift"
            }
            "arity" => {
                drift
                    .nodes
                    .iter_mut()
                    .find(|n| n.name == call.name)
                    .unwrap()
                    .op
                    .args
                    .pop();
                "signature drift"
            }
            "parameter" => {
                drift
                    .functions
                    .iter_mut()
                    .find(|f| f.name == callee.name)
                    .unwrap()
                    .parameters[0]
                    .ty = "f64".to_owned();
                "parameter node/signature drift"
            }
            "index" => {
                drift
                    .nodes
                    .iter_mut()
                    .find(|n| &n.name == parameter)
                    .unwrap()
                    .op
                    .args[0] = "1".to_owned();
                "parameter node/signature drift"
            }
            "lane" => {
                drift
                    .node_lanes
                    .insert(parameter.clone(), "fn:unrelated".to_owned());
                "does not admit"
            }
            "global" => {
                let main = module
                    .functions
                    .iter()
                    .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                    .unwrap();
                edge(&mut drift, &main.result.as_ref().unwrap().node, parameter);
                "requires external initialization"
            }
            _ => unreachable!(),
        };
        rejected(&drift, expected);
    }
    fs::write(
        project.0.join("main.ns"),
        HELPERS.replace("return left + right", "print(left); return left + right"),
    )
    .unwrap();
    let effects = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    rejected(&effects, "does not admit cpu.print");
}

pub(super) fn edge(module: &mut YirModule, from: &str, to: &str) {
    module.edges.push(yir_core::Edge {
        kind: yir_core::EdgeKind::Dep,
        from: from.to_owned(),
        to: to.to_owned(),
    });
}

pub(super) fn push_node(
    module: &mut YirModule,
    function: &str,
    name: &str,
    instruction: &str,
    args: Vec<String>,
) {
    let resource = module
        .nodes
        .iter()
        .find(|n| n.op.module == "cpu")
        .unwrap()
        .resource
        .clone();
    module.nodes.push(yir_core::Node {
        name: name.to_owned(),
        resource,
        op: yir_core::Operation::parse(&format!("cpu.{instruction}"), args).unwrap(),
    });
    module
        .node_lanes
        .insert(name.to_owned(), format!("fn:{function}"));
}

pub(super) fn helper(module: &mut YirModule, name: &str, callees: &[String], nodes: usize) {
    let mut body = Vec::<String>::new();
    for index in 0..nodes - 1 {
        let node = format!("{name}_v{index}");
        let (instruction, args) = if let Some(callee) = callees.get(index) {
            ("call_i64", vec![callee.clone()])
        } else {
            ("const_i64", vec!["0".to_owned()])
        };
        push_node(module, name, &node, instruction, args);
        if let Some(previous) = body.last() {
            edge(module, previous, &node);
        }
        body.push(node);
    }
    let returned = format!("{name}_return");
    let value = body.last().unwrap();
    push_node(module, name, &returned, "return_i64", vec![value.clone()]);
    edge(module, value, &returned);
    body.push(returned.clone());
    module.functions.push(yir_core::YirFunction {
        name: name.to_owned(),
        domain: "cpu".to_owned(),
        role: yir_core::YirFunctionRole::Helper,
        parameters: Vec::new(),
        result: Some(yir_core::YirFunctionResult {
            ty: "i64".to_owned(),
            ownership: yir_core::YirValueOwnership::Value,
            node: returned,
        }),
        body_nodes: body,
    });
}

pub(super) fn root_call(module: &mut YirModule, callee: &str) -> String {
    let name = module.application_sessions[0].open.clone();
    let call = format!("root_call_{callee}");
    push_node(module, &name, &call, "call_i64", vec![callee.to_owned()]);
    let function = module
        .functions
        .iter_mut()
        .find(|f| f.name == name)
        .unwrap();
    function.body_nodes.push(call.clone());
    let result = function.result.as_ref().unwrap().node.clone();
    edge(module, &call, &result);
    call
}

#[test]
fn selected_call_graph_is_bounded_acyclic_and_ignores_unreachable_effects() {
    let project = Project::new();
    let base = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    for mutual in [false, true] {
        let mut module = base.clone();
        helper(
            &mut module,
            "cycle_a",
            &[if mutual { "cycle_b" } else { "cycle_a" }.to_owned()],
            2,
        );
        if mutual {
            helper(&mut module, "cycle_b", &["cycle_a".to_owned()], 2);
        }
        // An unselected recursive helper must not leak into the bridge.
        assert!(!emit_registered(&module, "counter")
            .unwrap()
            .llvm_ir
            .contains("@nuis_fn_cycle_"));
        root_call(&mut module, "cycle_a");
        rejected(&module, "recursive helper call cycles");
    }
    for count in [31, 32] {
        let mut module = base.clone();
        for i in 0..count {
            let callees = if i + 1 < count {
                vec![format!("chain_{}", i + 1)]
            } else {
                Vec::new()
            };
            helper(&mut module, &format!("chain_{i}"), &callees, 2);
        }
        root_call(&mut module, "chain_0");
        if count == 31 {
            emit_registered(&module, "counter").unwrap();
        } else {
            rejected(&module, "call-depth bound");
        }
    }
    for count in [61, 62] {
        let mut module = base.clone();
        for i in 0..count {
            let name = format!("wide_{i}");
            helper(&mut module, &name, &[], 2);
            root_call(&mut module, &name);
        }
        if count == 61 {
            emit_registered(&module, "counter").unwrap();
        } else {
            rejected(&module, "reachable-function bound");
        }
    }
}

#[test]
fn strict_native_helpers_reject_implicit_scalar_coercions_and_unmaterialized_values() {
    let project = Project::new();
    let base = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    for return_value in [false, true] {
        let mut module = base.clone();
        helper(&mut module, "typed", &[], 2);
        let call = root_call(&mut module, "typed");
        let input = module
            .nodes
            .iter_mut()
            .find(|n| n.name == "typed_v0")
            .unwrap();
        if return_value {
            input.op =
                yir_core::Operation::parse("cpu.const_bool", vec!["true".to_owned()]).unwrap();
        } else {
            input.op = yir_core::Operation::parse("cpu.param_i64", vec!["0".to_owned()]).unwrap();
            module
                .functions
                .iter_mut()
                .find(|f| f.name == "typed")
                .unwrap()
                .parameters
                .push(yir_core::YirFunctionParameter {
                    name: "input".to_owned(),
                    ty: "i64".to_owned(),
                    ownership: yir_core::YirValueOwnership::Value,
                    node: "typed_v0".to_owned(),
                });
            let root = &module.application_sessions[0].open;
            let boolean = module
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
            module
                .nodes
                .iter_mut()
                .find(|n| n.name == call)
                .unwrap()
                .op
                .args
                .push(boolean.clone());
            edge(&mut module, &boolean, &call);
        }
        yir_verify::verify_module(&module).unwrap();
        rejected(&module, "exactly match its declared scalar kind");
    }
    for mutation in ["valid", "condition", "value", "layout"] {
        let mut module = base.clone();
        helper(&mut module, "guarded", &[], 2);
        root_call(&mut module, "guarded");
        let (instruction, literal) = if mutation == "condition" {
            ("const_f32", "0.5")
        } else {
            ("const_bool", "false")
        };
        push_node(
            &mut module,
            "guarded",
            "guard_condition",
            instruction,
            vec![literal.to_owned()],
        );
        let mut args = vec![
            "guard_condition".to_owned(),
            if mutation == "value" {
                "guard_condition"
            } else {
                "guarded_v0"
            }
            .to_owned(),
        ];
        if mutation == "layout" {
            args.push("State{value:i64}".to_owned());
        }
        push_node(&mut module, "guarded", "guarded_exit", "guard_return", args);
        module
            .functions
            .iter_mut()
            .find(|f| f.name == "guarded")
            .unwrap()
            .body_nodes
            .extend(["guard_condition".to_owned(), "guarded_exit".to_owned()]);
        edge(&mut module, "guard_condition", "guarded_exit");
        edge(&mut module, "guarded_v0", "guarded_exit");
        edge(&mut module, "guarded_exit", "guarded_return");
        yir_verify::verify_module(&module).unwrap();
        match mutation {
            "valid" => {
                assert!(emit_registered(&module, "counter")
                    .unwrap()
                    .llvm_ir
                    .contains("guard_return_then"));
            }
            "condition" => rejected(&module, "requires a bool or i64 condition"),
            "value" => rejected(&module, "exactly match its declared scalar kind"),
            "layout" => rejected(&module, "cannot carry an aggregate layout"),
            _ => unreachable!(),
        }
    }
    let mut module = base;
    helper(&mut module, "partial", &[], 2);
    root_call(&mut module, "partial");
    push_node(
        &mut module,
        "partial",
        "partial_field",
        "field",
        vec!["partial_v0".to_owned(), "missing".to_owned()],
    );
    module
        .functions
        .iter_mut()
        .find(|f| f.name == "partial")
        .unwrap()
        .body_nodes
        .push("partial_field".to_owned());
    edge(&mut module, "partial_v0", "partial_field");
    edge(&mut module, "partial_field", "partial_return");
    yir_verify::verify_module(&module).unwrap();
    rejected(&module, "did not materialize value");
}

#[test]
fn reachable_helper_node_limits_apply_to_each_body_and_the_whole_closure() {
    let project = Project::new();
    let base = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    for nodes in [4096, 4097] {
        let mut module = base.clone();
        helper(&mut module, "body_bound", &[], nodes);
        root_call(&mut module, "body_bound");
        if nodes == 4096 {
            emit_registered(&module, "counter").unwrap();
        } else {
            rejected(&module, "function/argument bounds");
        }
    }
    let mut module = base;
    for i in 0..4 {
        let name = format!("total_{i}");
        helper(&mut module, &name, &[], 4096);
        root_call(&mut module, &name);
    }
    rejected(&module, "total-node bound");
}
