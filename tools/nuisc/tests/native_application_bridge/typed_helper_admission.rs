use super::*;

fn compiled() -> YirModule {
    let project = Project::with_source(&typed_helper_values::source(6));
    nuisc::pipeline::compile_project(&project.0).unwrap().yir
}

fn reject(module: &YirModule, expected: &str) {
    let error = emit_registered(module, "counter").unwrap_err();
    assert!(error.contains(expected), "expected {expected}: {error}");
}

#[test]
fn typed_helper_calls_require_exact_nested_layouts_and_scalar_arguments() {
    let base = compiled();
    emit_registered(&base, "counter").unwrap();
    let call = base
        .nodes
        .iter()
        .position(|n| n.op.instruction == "call_owned_struct" && n.op.args[0] == "packet")
        .unwrap();
    let encoded = &base.nodes[call].op.args[1];
    for (from, to, error) in [
        ("Packet{", "Other{", "signature drift"),
        ("Payload{", "Other{", "signature drift"),
        ("f1:i32", "f1:i64", "signature drift"),
        ("f3:f32", "f3:f64", "signature drift"),
        ("f0:bool;f1:i32", "f1:i32;f0:bool", "signature drift"),
        ("f0:bool", "renamed:bool", "signature drift"),
        ("f0:bool", "f1:bool", "duplicate field names"),
        ("f0:bool", "f0:String", "cannot carry resources"),
        ("f0:bool", "f0:Bytes", "cannot carry resources"),
        ("f0:bool", "f0:Empty{}", "empty state structs"),
        ("f0:bool", "f0:N{x:i64}", "signature drift"),
    ] {
        let mut drift = base.clone();
        assert!(encoded.contains(from), "{encoded}");
        drift.nodes[call].op.args[1] = encoded.replace(from, to);
        reject(&drift, error);
    }
    let mut oversized = base.clone();
    oversized.nodes[call].op.args[1] = format!(
        "Packet{{payload:Payload{{{}}}}}",
        (0..65)
            .map(|i| format!("f{i}:f32"))
            .collect::<Vec<_>>()
            .join(";")
    );
    reject(&oversized, "slot bounds");
    let mut deep = base.clone();
    deep.nodes[call].op.args[1] = format!(
        "{}Leaf{{value:bool}}{}",
        "N{child:".repeat(64),
        "}".repeat(64)
    );
    reject(&deep, "maximum depth");
    let mut duplicate = base.clone();
    duplicate.nodes[call].op.args[1] = "Packet{payload:A{x:i64};payload:B{y:bool}}".into();
    reject(&duplicate, "duplicate field names");

    for mutation in ["arity", "kind", "ownership"] {
        let mut drift = base.clone();
        let expected = match mutation {
            "arity" => {
                drift.nodes[call].op.args.pop();
                "signature drift"
            }
            "kind" => {
                drift.nodes[call].op.args[2] = drift.nodes[call].op.args[3].clone();
                "declared scalar kind"
            }
            "ownership" => {
                drift
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "packet")
                    .unwrap()
                    .result
                    .as_mut()
                    .unwrap()
                    .ownership = yir_core::YirValueOwnership::Value;
                "return layout drift"
            }
            _ => unreachable!(),
        };
        reject(&drift, expected);
    }
}

#[test]
fn typed_helper_values_reject_spoofed_nested_values_and_hidden_effects() {
    let base = compiled();
    for mutation in ["nominal", "kind", "duplicate", "effect"] {
        let mut drift = base.clone();
        let expected = if mutation == "effect" {
            let function = drift.functions.iter().find(|f| f.name == "packet").unwrap();
            let argument = function.parameters[2].node.clone();
            let returned = function.result.as_ref().unwrap().node.clone();
            helpers::push_node(
                &mut drift,
                "packet",
                "hidden_print",
                "print",
                vec![argument.clone()],
            );
            drift
                .functions
                .iter_mut()
                .find(|f| f.name == "packet")
                .unwrap()
                .body_nodes
                .push("hidden_print".into());
            helpers::edge(&mut drift, &argument, "hidden_print");
            helpers::edge(&mut drift, "hidden_print", &returned);
            "does not admit cpu.print"
        } else {
            let node = drift
                .nodes
                .iter_mut()
                .find(|n| {
                    drift
                        .node_lanes
                        .get(&n.name)
                        .is_some_and(|lane| lane == "fn:packet")
                        && n.op.instruction == "struct"
                        && n.op.args[0] == "Payload"
                })
                .unwrap();
            match mutation {
                "nominal" => node.op.args[0] = "Other".into(),
                "kind" => {
                    let flag = node
                        .op
                        .args
                        .iter()
                        .find_map(|a| a.strip_prefix("f0="))
                        .unwrap()
                        .to_owned();
                    *node
                        .op
                        .args
                        .iter_mut()
                        .find(|a| a.starts_with("f1="))
                        .unwrap() = format!("f1={flag}");
                }
                "duplicate" => {
                    let field = node
                        .op
                        .args
                        .iter_mut()
                        .find(|a| a.starts_with("f0="))
                        .unwrap();
                    *field = field.replacen("f0=", "f1=", 1);
                }
                _ => unreachable!(),
            }
            "native value return"
        };
        reject(&drift, expected);
    }
}

#[test]
fn typed_helper_early_returns_keep_fallible_suffix_after_guard() {
    let source = typed_helper_values::source(6).replace("f2: p2 + 1", "f2: p2 / p2");
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let bridge = emit_registered(&compiled.yir, "counter").unwrap();
    let start = aggregate_values::definition(&bridge.llvm_ir, "packet");
    let body = &bridge.llvm_ir[start..start + bridge.llvm_ir[start..].find("\n}\n").unwrap()];
    assert!(body.find("\nguard_return_struct_cont.").unwrap() < body.find(" = sdiv i64").unwrap());
    assert!(body.contains("ret [6 x i64]"));
}

#[test]
fn mixed_nested_local_rebinding_uses_guarded_native_value_returns() {
    let project = Project::with_source("mod cpu Main {
        struct Payload { flag: bool, value: i64 }
        struct Packet { payload: Payload }
        struct State { value: i64 }
        @noinline fn choose(seed: i64, divisor: i64, flag: bool) -> Packet {
            let result = Packet { payload: Payload { flag: flag, value: seed } };
            if flag { let result = Packet { payload: Payload { flag: flag, value: seed / divisor } }; }
            else { let result = Packet { payload: Payload { flag: flag, value: seed } }; }
            return result;
        }
        fn start(seed: i64, divisor: i64, flag: bool) -> State {
            let packet = choose(seed, divisor, flag);
            return State { value: packet.payload.value };
        }
        fn step(state: State) -> State { return state; }
        fn stop(state: State) -> State { return state; }
        fn main() -> i64 { return 0; }
    }");
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let bridge = emit_registered(&compiled.yir, "counter").unwrap();
    assert!(bridge
        .llvm_ir
        .contains("call [2 x i64] @nuis_fn___nuis_conditional_value"));
    assert!(!bridge
        .llvm_ir
        .contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
}
