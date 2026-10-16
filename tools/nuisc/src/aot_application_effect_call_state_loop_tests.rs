use super::*;
#[allow(dead_code)]
#[path = "../tests/native_application_bridge/state_loop_cases.rs"]
mod cases;

#[test]
fn native_state_loop_preserves_runtime_carry_guard_and_explicit_budget_identity() {
    for source in cases::source_variants() {
        let (_, mut module, old_mode) = compiled(&source);
        let sites = module
            .nodes
            .iter()
            .filter(|n| matches!(n.op.instruction.as_str(), "print" | "guard_print"))
            .map(|n| n.name.clone())
            .collect::<Vec<_>>();
        let mode = literal_print_packaging_mode("counter", sites, cases::LOOP_LIMIT, 100).unwrap();
        assert_ne!(old_mode, mode);
        let bridge = emit_for_packaging_mode(&module, &mode).unwrap();
        assert_eq!(bridge.loop_work_limit, 3);
        assert_eq!(bridge.helper_entry_limit, 100);
        assert_eq!(
            bridge.state_layout.source(),
            "State{result:bool;total:i64;marker:f64}"
        );
        assert!(!bridge
            .llvm_ir
            .contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
        assert!(!bridge
            .llvm_ir
            .contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
        // A runtime loop must survive outlining, not just three copied source calls.
        assert!(module
            .nodes
            .iter()
            .any(|n| n.op.instruction.starts_with("loop_while")));
        // This unoptimized emitter uses stack-carried values, not SSA phis.
        assert!(bridge.llvm_ir.contains("loop_while_i64_body"));
        assert!(
            bridge
                .llvm_ir
                .matches("br label %loop_while_i64_cond")
                .count()
                >= 2
        );
        assert!(bridge.llvm_ir.contains(" = load i64, ptr "));
        assert!(bridge.llvm_ir.contains("store i64"));
        assert!(bridge.llvm_ir.contains("%nuis_loop_work"));
        assert!(module
            .functions
            .iter()
            .any(|f| f.name.starts_with("__nuis_scalar_iteration_")));
        assert!(module
            .nodes
            .iter()
            .any(|n| n.op.instruction == "guard_return"));
        assert!(emit_for_packaging_mode(&module, "native-session-aot-bundle:counter").is_err());
        module
            .edges
            .retain(|e| !matches!(e.kind, yir_core::EdgeKind::Effect));
        assert!(emit_for_packaging_mode(&module, &mode)
            .unwrap_err()
            .contains("lacks dependency order"));
    }
}

#[test]
fn native_state_loop_keeps_transitive_iteration_prints_out_of_pure_admission() {
    for (before, after) in [
        (
            "{ return value == (proof == 1); }",
            "{ print(10); return value == (proof == 1); }",
        ),
        (
            "{ return value * 2 + proof + index; }",
            "{ print(20); return value * 2 + proof + index; }",
        ),
    ] {
        let source = cases::SOURCE.replace(before, after);
        assert_ne!(source, cases::SOURCE);
        let seed = fixture_for_mode("native-session-aot-bundle:counter");
        fs::write(seed.0.join("main.ns"), source).unwrap();
        let module = crate::pipeline::compile_project(&seed.0).unwrap().yir;
        let error = emit_for_packaging_mode(&module, "native-session-aot-bundle:counter")
            .err()
            .expect("ungranted effectful iteration must reject");
        assert!(
            error.contains("print") || error.contains("effect"),
            "{error}"
        );
    }
}
