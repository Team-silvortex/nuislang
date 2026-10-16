use super::*;
#[allow(dead_code)]
#[path = "../tests/native_application_bridge/state_loop_exits_cases.rs"]
mod cases;

#[test]
fn native_state_loop_exits_require_exact_grants_and_transitive_effect_order() {
    for variant in cases::variants() {
        let (_, mut module, _) = compiled(&variant.source());
        let sites = module
            .nodes
            .iter()
            .filter(|n| matches!(n.op.instruction.as_str(), "print" | "guard_print"))
            .map(|n| n.name.clone())
            .collect::<Vec<_>>();
        let mode =
            literal_print_packaging_mode("counter", sites.clone(), cases::LOOP_LIMIT, 100).unwrap();
        let bridge = emit_for_packaging_mode(&module, &mode).unwrap();
        assert_eq!(bridge.loop_work_limit, 3);
        assert_eq!(bridge.helper_entry_limit, 100);
        assert_eq!(
            bridge.state_layout.source(),
            "State{result:bool;total:i64;marker:f64}"
        );
        assert!(bridge.llvm_ir.contains("loop_while_i64_body"));
        assert!(!bridge
            .llvm_ir
            .contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
        assert!(emit_for_packaging_mode(&module, "native-session-aot-bundle:counter").is_err());
        for site in &sites {
            let restricted = literal_print_packaging_mode(
                "counter",
                sites.iter().filter(|s| *s != site).cloned(),
                3,
                100,
            )
            .unwrap();
            assert!(
                emit_for_packaging_mode(&module, &restricted).is_err(),
                "missing grant {site} was accepted"
            );
        }
        module
            .edges
            .retain(|e| !matches!(e.kind, yir_core::EdgeKind::Effect));
        assert!(emit_for_packaging_mode(&module, &mode)
            .unwrap_err()
            .contains("lacks dependency order"));
    }
}
