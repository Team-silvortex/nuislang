use super::*;

const MULTI: &str = include_str!("multi_loops.ns");

#[test]
fn multi_carry_scoped_calls_preserve_native_reference_and_typed_session_parity() {
    let project = Project::with_source(MULTI);
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    let loops = module
        .nodes
        .iter()
        .filter(|node| node.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carries"))
        .collect::<Vec<_>>();
    assert_eq!(
        loops.len(),
        2,
        "{:?}",
        module
            .nodes
            .iter()
            .filter(|n| n.op.instruction.starts_with("loop_"))
            .map(|n| &n.op)
            .collect::<Vec<_>>()
    );
    let bridge = emit_registered(&module, "counter").unwrap();
    for node in loops {
        let carries = yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args)
            .unwrap()
            .unwrap();
        assert!(bridge.llvm_ir.contains(&format!(
            " = call [{} x i64] @nuis_fn_{}(",
            carries.layout.fields.len(),
            node.op.args[8]
        )));
    }
    assert_native_parity(MULTI, true);
}

#[test]
fn extra_carries_cannot_disappear_behind_a_single_scoped_call() {
    let source = MULTI.replace(
        "let returned: Carries = advance(total, index, checksum, enabled);\n      let total: i64 = returned.carry0;\n      let checksum: i64 = returned.carry1;",
        "let total: i64 = accumulate(total, index, enabled);\n      let checksum: i64 = sum(checksum, total);",
    );
    // The value outliner now admits both writes. Losing checksum would corrupt
    // the independent lifecycle totals even if reference/native made the same bug.
    assert_native_parity(&source, true);
    // An effectful callee remains outside that catalog. The fallback must reject
    // this body instead of silently dropping its extra outer-state update.
    let effectful = source.replace(
        "let checksum: i64 = sum(checksum, total);",
        "let checksum: i64 = audit(checksum, total);",
    ).replace(
        "mod cpu Main {",
        "mod cpu Main { @noinline fn audit(left: i64, right: i64) -> i64 { print(left); return left + right; }",
    );
    let project = Project::with_source(&effectful);
    let error = nuisc::pipeline::compile_project(&project.0)
        .err()
        .expect("must retain every carried update or reject");
    assert!(
        error.contains("structured `while` lowering recognized loop state `index`")
            && error.contains("first body binding `total` is not a supported temp/step prefix"),
        "{error}"
    );
}

#[test]
fn multi_carry_reference_fuel_failure_preserves_the_accepted_state() {
    let source = MULTI.replace("delta + 4", "delta + 1000");
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
    assert!(session.completion_status().is_err());
}
