use super::*;

const LOOPS: &str = include_str!("loops.ns");

#[test]
fn counted_scalar_loops_compose_with_native_helpers_and_reference_state() {
    let project = Project::with_source(LOOPS);
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    let loops = module
        .nodes
        .iter()
        .filter(|node| node.op.instruction.starts_with("loop_while_"))
        .collect::<Vec<_>>();
    assert!(loops.len() >= 4, "must retain native loops: {loops:?}");
    assert!(loops
        .iter()
        .any(|node| node.op.instruction == "loop_while_i64"));
    assert!(
        loops
            .iter()
            .any(|node| node.op.args.contains(&"add_carry0".to_owned())),
        "{loops:?}"
    );
    let bridge = emit_registered(&module, "counter").unwrap();
    assert!(bridge.llvm_ir.contains("loop_while_i64_cond"));
    assert!(bridge.llvm_ir.contains("loop_while_scalar_chain_cond"));
    assert!(!bridge.llvm_ir.contains("native_loop_preflight"));
    assert_native_parity(LOOPS, true);
}

#[test]
fn native_counted_loops_reject_hidden_effects() {
    let source = LOOPS.replace("return total - 6;", "print(total); return total - 6;");
    let project = Project::with_source(&source);
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    let error = emit_registered(&module, "counter").unwrap_err();
    assert!(error.contains("does not admit"), "{error}");
}

#[test]
fn native_counted_loops_reject_induction_and_carry_drift() {
    let project = Project::with_source(LOOPS);
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    let original = module
        .nodes
        .iter()
        .find(|node| {
            node.op.instruction == "loop_while_scalar_chain"
                && node.op.args.contains(&"add_carry0".to_owned())
        })
        .unwrap();
    for mutation in [
        "zero-step",
        "direction",
        "too-many",
        "forward-carry",
        "float-carry",
    ] {
        let mut drift = module.clone();
        let mut changed = original.clone();
        let expected = match mutation {
            "direction" => {
                changed.op.args[4] = "sub".to_owned();
                "finite non-wrapping induction"
            }
            "forward-carry" => {
                changed.op.args[6] = "add_carry1".to_owned();
                "scalar carry source"
            }
            _ => {
                let (index, instruction, literal, expected) = match mutation {
                    "zero-step" => (2, "const_i64", "0", "finite non-wrapping induction"),
                    "too-many" => (1, "const_i64", "65537", "65536-iteration bound"),
                    "float-carry" => (5, "const_f64", "1.0", "declared scalar kind"),
                    _ => unreachable!(),
                };
                let name = format!("{}_drift", original.name);
                let mut constant = original.clone();
                constant.name = name.clone();
                constant.op = yir_core::Operation::parse(
                    &format!("cpu.{instruction}"),
                    vec![literal.to_owned()],
                )
                .unwrap();
                drift.nodes.push(constant);
                let function = drift
                    .functions
                    .iter_mut()
                    .find(|f| f.body_nodes.contains(&original.name))
                    .unwrap();
                function.body_nodes.push(name.clone());
                drift
                    .node_lanes
                    .insert(name.clone(), format!("fn:{}", function.name));
                drift.edges.push(yir_core::Edge {
                    kind: yir_core::EdgeKind::Dep,
                    from: name.clone(),
                    to: original.name.clone(),
                });
                changed.op.args[index] = name;
                expected
            }
        };
        *drift
            .nodes
            .iter_mut()
            .find(|node| node.name == original.name)
            .unwrap() = changed;
        let error = emit_registered(&drift, "counter").unwrap_err();
        assert!(error.contains(expected), "{mutation}: {error}");
    }
}

#[test]
fn plain_loop_reference_fuel_exhaustion_retains_the_accepted_session_state() {
    let source = LOOPS.replacen("index < 4", "index < 1000", 1);
    let project = Project::with_source(&source);
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    emit_registered(&module, "counter").unwrap();
    let registry = yir_verify::default_registry();
    let (mut session, _) = ApplicationSession::open_registered(
        &module,
        &registry,
        "counter",
        vec![
            Value::Int(10),
            Value::I32(-17),
            Value::Bool(true),
            Value::F32(1.5),
            Value::F64(-2.25),
        ],
    )
    .unwrap();
    let accepted = session.state().clone();
    let error = session
        .event_budgeted(vec![Value::Int(3), Value::Bool(false)], 100)
        .unwrap_err();
    assert!(error.contains("budget"), "{error}");
    assert_eq!(session.state(), &accepted);
    session.close(vec![Value::Int(0)]).unwrap();
    assert!(
        session.completion_status().is_err(),
        "cleanup must retain the event failure"
    );
}
