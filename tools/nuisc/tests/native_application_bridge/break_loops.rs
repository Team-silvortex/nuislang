use super::*;

const BREAKS: &str = include_str!("break_loops.ns");

#[test]
fn guarded_break_scoped_calls_preserve_native_reference_and_typed_session_parity() {
    let project = Project::with_source(BREAKS);
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    let loops = module
        .nodes
        .iter()
        .filter(|n| n.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carries_break"))
        .collect::<Vec<_>>();
    assert_eq!(loops.len(), 2);
    let bridge = emit_registered(&module, "counter").unwrap();
    for node in loops {
        assert!(bridge
            .llvm_ir
            .contains(&format!(" = call i64 @nuis_fn_{}(", node.op.args[8])));
    }
    assert!(bridge.llvm_ir.contains("loop_break_control_invalid"));
    assert_native_parity(BREAKS, true);
}

#[test]
fn guarded_break_keeps_shared_layout_kinds_closure_and_general_call_rejections() {
    multi_admission::check_layout_drift(true);
    multi_admission::check_scalar_kinds(true);
    multi_admission::check_edges(true);
    multi_admission::check_general_calls(true);
}

#[test]
fn guarded_break_reference_fuel_failure_keeps_accepted_state() {
    let project = Project::with_source(BREAKS);
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
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
        .event_budgeted(vec![Value::Int(3), Value::Bool(false)], 1)
        .unwrap_err();
    assert!(error.contains("budget"), "{error}");
    assert_eq!(session.state(), &accepted);
    session.close(vec![Value::Int(0)]).unwrap();
    assert!(session.completion_status().is_err());
}

#[test]
fn scalar_break_source_preserves_provenance_and_rejects_unmodeled_updates() {
    for bad in [
        "let index: i64 = index + 1; break;",
        "break; let total: i64 = 99;",
        "return total;",
    ] {
        let source = BREAKS.replace(
            "if index == stop { break; }",
            &format!("if index == stop {{ {bad} }}"),
        );
        let project = Project::with_source(&source);
        assert!(
            nuisc::pipeline::compile_project(&project.0).is_err(),
            "{bad}"
        );
    }
    // A user's aggregate field is not a private compiler-generated control slot.
    let source = multi_execution::source(2, "<").replace(
        "let s1: i64 = returned.carry1;",
        "let s1: i64 = returned.carry1; if s1 == 1 { break; }",
    );
    let project = Project::with_source(&source);
    assert!(nuisc::pipeline::compile_project(&project.0).is_err());
}
