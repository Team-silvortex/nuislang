use super::*;
#[allow(dead_code)]
#[path = "../tests/native_application_bridge/state_loop_return_parent_steps_cases.rs"]
mod cases;

#[test]
fn native_state_loop_return_parent_steps_keep_exact_layout_grants_and_effect_order() {
    for variant in cases::cli_variants() {
        let (_, mut module, _) = compiled(&cases::source(variant));
        let sites = module
            .nodes
            .iter()
            .filter(|n| matches!(n.op.instruction.as_str(), "print" | "guard_print"))
            .map(|n| n.name.clone())
            .collect::<Vec<_>>();
        let mode =
            literal_print_packaging_mode("counter", sites.clone(), cases::LOOP_LIMIT, 100).unwrap();
        let bridge = emit_for_packaging_mode(&module, &mode).unwrap();
        assert_eq!(
            bridge.state_layout.source(),
            "State{result:bool;total:i64;gain:f32;marker:f64}"
        );
        assert_eq!(bridge.loop_work_limit, cases::LOOP_LIMIT);
        assert!(!bridge
            .llvm_ir
            .contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
        assert!(emit_for_packaging_mode(&module, "native-session-aot-bundle:counter").is_err());
        for site in &sites {
            let restricted = literal_print_packaging_mode(
                "counter",
                sites.iter().filter(|s| *s != site).cloned(),
                cases::LOOP_LIMIT,
                100,
            )
            .unwrap();
            assert!(emit_for_packaging_mode(&module, &restricted).is_err());
        }
        module
            .edges
            .retain(|e| !matches!(e.kind, yir_core::EdgeKind::Effect));
        assert!(emit_for_packaging_mode(&module, &mode)
            .unwrap_err()
            .contains("lacks dependency order"));
    }
}
