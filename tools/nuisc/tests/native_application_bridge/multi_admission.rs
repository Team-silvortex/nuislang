use super::*;
use helpers::{edge, push_node};

fn compiled(breaking: bool) -> YirModule {
    let source = multi_execution::source(2, "<").replace(
        "if !flag { return Carries { carry0: c0, carry1: c1 }; }",
        "",
    );
    let project = Project::with_source(&source);
    let mut module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    let name = module.nodes[loop_index(&module)].op.args[8].clone();
    let function = module.functions.iter().find(|f| f.name == name).unwrap();
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
    // Retain an explicit guard instead of depending on NIR select folding.
    push_node(
        &mut module,
        &name,
        "carry_guard",
        "guard_return",
        vec![flag.clone(), value.clone(), layout],
    );
    module
        .functions
        .iter_mut()
        .find(|f| f.name == name)
        .unwrap()
        .body_nodes
        .push("carry_guard".to_owned());
    edge(&mut module, &flag, "carry_guard");
    edge(&mut module, &value, "carry_guard");
    edge(&mut module, "carry_guard", &returned);
    if breaking {
        let index = loop_index(&module);
        module.nodes[index].op.args[6] = "scoped_call_i64_carries_break".to_owned();
    }
    module
}

fn loop_index(module: &YirModule) -> usize {
    module
        .nodes
        .iter()
        .position(|node| {
            matches!(
                node.op.args.get(6).map(String::as_str),
                Some("scoped_call_i64_carries" | "scoped_call_i64_carries_break")
            )
        })
        .unwrap()
}

fn rejected(module: &YirModule, expected: &str) {
    let error = emit_registered(module, "counter").unwrap_err();
    assert!(error.contains(expected), "expected {expected}: {error}");
}

#[test]
fn multi_carry_admission_rejects_layout_ownership_marker_and_action_drift() {
    check_layout_drift(false);
}

pub(super) fn check_layout_drift(breaking: bool) {
    let base = compiled(breaking);
    emit_registered(&base, "counter").unwrap();
    let index = loop_index(&base);
    for (arg, value, error) in [
        (9, "Wrong{carry0:i64;carry1:i64}", "signature drift"),
        (
            9,
            "Carries{carry0:i64;carry1:bool}",
            "scoped_call_i64_carries payload",
        ),
        (
            9,
            "Carries{carry0:i64;carry0:i64}",
            "scoped_call_i64_carries payload",
        ),
        (
            9,
            "Carries{carry0:Inner{x:i64};carry1:i64}",
            "scoped_call_i64_carries payload",
        ),
        (
            10,
            "$owned_struct_carry:0:missing_seed",
            "scoped_call_i64_carries payload",
        ),
        (
            10,
            "$owned_struct_carry:999:missing_seed",
            "scoped_call_i64_carries payload",
        ),
        (10, "$carry", "scoped_call_i64_carries payload"),
        (10, "copy_owned:seed", "scoped_call_i64_carries payload"),
        (10, "move_owned:seed", "scoped_call_i64_carries payload"),
        (
            6,
            "scoped_call_i64_carries_unknown",
            "unregistered loop action",
        ),
        (8, "missing_helper", "unknown helper"),
    ] {
        let mut drift = base.clone();
        drift.nodes[index].op.args[arg] = value.to_owned();
        rejected(&drift, error);
    }
    for mutation in ["ownership", "type", "layout", "arity"] {
        let mut drift = base.clone();
        let name = drift.nodes[index].op.args[8].clone();
        let callee = drift.functions.iter_mut().find(|f| f.name == name).unwrap();
        let result = callee.result.as_mut().unwrap();
        match mutation {
            "ownership" => result.ownership = yir_core::YirValueOwnership::Value,
            "type" => result.ty = "Wrong".to_owned(),
            "layout" => {
                let node = drift
                    .nodes
                    .iter_mut()
                    .find(|n| n.name == result.node)
                    .unwrap();
                node.op.args[1] = "Carries{carry0:bool;carry1:i64}".to_owned();
            }
            "arity" => {
                drift.nodes[index].op.args.push("$current".to_owned());
                drift.nodes[index].op.args[7] = (drift.nodes[index].op.args.len() - 8).to_string();
            }
            _ => unreachable!(),
        }
        rejected(
            &drift,
            if mutation == "arity" {
                "signature drift"
            } else {
                "return layout drift"
            },
        );
    }
}

#[test]
fn multi_carry_admission_requires_exact_seed_capture_parameter_and_return_kinds() {
    check_scalar_kinds(false);
}

pub(super) fn check_scalar_kinds(breaking: bool) {
    let base = compiled(breaking);
    let index = loop_index(&base);
    let caller = &base.application_sessions[0].open;
    let flag = base
        .functions
        .iter()
        .find(|f| &f.name == caller)
        .unwrap()
        .parameters
        .iter()
        .find(|p| p.ty == "bool")
        .unwrap()
        .node
        .clone();
    for mutation in [
        "seed",
        "capture",
        "parameter",
        "return",
        "guard-return",
        "guard-layout",
    ] {
        let mut drift = base.clone();
        let name = drift.nodes[index].op.args[8].clone();
        let callee = drift.functions.iter_mut().find(|f| f.name == name).unwrap();
        let expected = match mutation {
            "seed" => {
                drift.nodes[index].op.args[10] = yir_core::encode_loop_owned_struct_carry(1, &flag);
                "declared scalar kind"
            }
            "capture" => {
                drift.nodes[index].op.args[13] = flag.clone();
                "declared scalar kind"
            }
            "parameter" => {
                callee.parameters[0].ty = "i32".to_owned();
                let parameter = &callee.parameters[0].node;
                drift
                    .nodes
                    .iter_mut()
                    .find(|n| &n.name == parameter)
                    .unwrap()
                    .op
                    .instruction = "param_i32".to_owned();
                "declared scalar kind"
            }
            "return" | "guard-return" => {
                let returned = if mutation == "return" {
                    &callee.result.as_ref().unwrap().node
                } else {
                    &drift
                        .nodes
                        .iter()
                        .find(|n| {
                            callee.body_nodes.contains(&n.name)
                                && n.op.instruction == "guard_return"
                        })
                        .unwrap()
                        .name
                };
                let returned = drift.nodes.iter().find(|n| &n.name == returned).unwrap();
                let value = returned.op.args[usize::from(mutation == "guard-return")].clone();
                let flag = callee
                    .parameters
                    .iter()
                    .find(|p| p.ty == "bool")
                    .unwrap()
                    .node
                    .clone();
                drift
                    .nodes
                    .iter_mut()
                    .find(|n| n.name == value)
                    .unwrap()
                    .op
                    .args[1] = format!("carry0={flag}");
                edge(&mut drift, &flag, &value);
                "return layout"
            }
            "guard-layout" => {
                let guard = drift
                    .nodes
                    .iter_mut()
                    .find(|n| {
                        callee.body_nodes.contains(&n.name) && n.op.instruction == "guard_return"
                    })
                    .unwrap();
                guard.op.args[2] = "Wrong{carry0:i64;carry1:i64}".to_owned();
                "return layout"
            }
            _ => unreachable!(),
        };
        rejected(&drift, expected);
    }
}

#[test]
fn multi_carry_edges_reject_recursive_closures_foreign_lanes_and_hidden_effects() {
    check_edges(false);
}

pub(super) fn check_edges(breaking: bool) {
    let base = compiled(breaking);
    let index = loop_index(&base);
    let target = base.nodes[index].op.args[8].clone();
    let callee = base.functions.iter().find(|f| f.name == target).unwrap();
    let returned = callee.result.as_ref().unwrap().node.clone();
    let lane = format!("fn:{target}");
    let mut recursive = base.clone();
    let step = "recursive_step";
    let current = &callee.parameters[2].node;
    push_node(
        &mut recursive,
        &target,
        step,
        "const_i64",
        vec!["1".to_owned()],
    );
    let mut args = base.nodes[index].op.args.clone();
    args[0] = current.clone();
    args[1] = current.clone(); // Even a zero-trip self-call must remain a graph edge.
    args[2] = step.to_owned();
    for (operand, param) in args[10..].iter_mut().zip(&callee.parameters) {
        *operand =
            if let Some((slot, _)) = yir_core::parse_loop_owned_struct_carry(operand).unwrap() {
                yir_core::encode_loop_owned_struct_carry(slot, &param.node)
            } else if operand == "$current" {
                operand.clone()
            } else {
                param.node.clone()
            };
    }
    push_node(
        &mut recursive,
        &target,
        "recursive_loop",
        "loop_while_i64_effect",
        args,
    );
    recursive
        .functions
        .iter_mut()
        .find(|f| f.name == target)
        .unwrap()
        .body_nodes
        .extend([step.to_owned(), "recursive_loop".to_owned()]);
    for param in &callee.parameters {
        edge(&mut recursive, &param.node, "recursive_loop");
    }
    edge(&mut recursive, step, "recursive_loop");
    edge(&mut recursive, "recursive_loop", &returned);
    rejected(&recursive, "recursive helper call cycles");

    let mut foreign = base.clone();
    let capture = callee.parameters[3].node.clone();
    foreign.nodes[index].op.args[13] = capture.clone();
    let loop_name = foreign.nodes[index].name.clone();
    edge(&mut foreign, &capture, &loop_name);
    rejected(&foreign, "cross-function values");

    let mut effect = base.clone();
    push_node(
        &mut effect,
        &target,
        "hidden_print",
        "print",
        vec![current.clone()],
    );
    effect
        .functions
        .iter_mut()
        .find(|f| f.name == target)
        .unwrap()
        .body_nodes
        .push("hidden_print".to_owned());
    assert_eq!(effect.node_lanes["hidden_print"], lane);
    edge(&mut effect, current, "hidden_print");
    edge(&mut effect, "hidden_print", &returned);
    rejected(&effect, "does not admit cpu.print");
}

#[test]
fn multi_carry_returns_reject_discarded_layout_drift_and_resource_aggregate_calls() {
    check_general_calls(false);
}

pub(super) fn check_general_calls(breaking: bool) {
    let base = compiled(breaking);
    let index = loop_index(&base);
    let mut discarded = base.clone();
    let args = &mut discarded.nodes[index].op.args;
    args[6] = "scoped_call".to_owned();
    args.remove(9);
    for operand in &mut args[9..] {
        if let Some((_, seed)) = yir_core::parse_loop_owned_struct_carry(operand).unwrap() {
            *operand = seed.to_owned();
        }
    }
    args[7] = (args.len() - 8).to_string();
    rejected(&discarded, "signature drift");

    let mut direct = base.clone();
    let args = direct.nodes[index].op.args.clone();
    let mut direct_args = vec![args[8].clone(), args[9].clone()];
    direct_args.extend(args[10..].iter().map(|operand| {
        yir_core::parse_loop_owned_struct_carry(operand)
            .unwrap()
            .map(|(_, seed)| seed.to_owned())
            .unwrap_or_else(|| {
                if operand == "$current" {
                    args[0].clone()
                } else {
                    operand.clone()
                }
            })
    }));
    direct.nodes[index].op.instruction = "call_owned_struct".to_owned();
    direct.nodes[index].op.args = direct_args;
    direct.nodes[index].op.args[1] = "Carries{carry0:i64;carry1:Nested{x:i64}}".to_owned();
    rejected(&direct, "requires a flat i64 value layout");
}

#[test]
fn unsupported_outer_updates_are_not_erased_by_counted_loop_fallback() {
    let project = Project::with_source(
        "mod cpu Main {
        struct State { value: i64 }
        @noinline
        fn apply(value: i64, index: i64) -> i64 { return value + index; }
        fn start(seed: i64, limit: i64) -> State {
          let total: i64 = seed; let index: i64 = 0;
          while index < limit {
            let temporary: i64 = apply(total, index);
            let total: i64 = temporary;
            let index: i64 = index + 1;
          }
          return State { value: total };
        }
        fn step(state: State) -> State { return state; }
        fn stop(state: State) -> State { return state; }
        fn main() -> i64 { return 0; }
    }",
    );
    let error = nuisc::pipeline::compile_project(&project.0)
        .err()
        .expect("must not erase carried updates");
    assert!(
        error.contains("cannot discard an unsupported outer-state update"),
        "{error}"
    );
}
