use super::*;

#[test]
fn native_return_stateful_fields_keeps_guard_first_effect_order_and_explicit_print_policy() {
    #[allow(dead_code)]
    #[path = "../tests/native_application_bridge/return_stateful_fields_cases.rs"]
    mod cases;
    for source in cases::source_variants() {
        let (_, mut module, mode) = compiled(&source);
        let bridge = emit_for_packaging_mode(&module, &mode).unwrap();
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
        let mut helpers = 0;
        let mut entries = 0;
        for function in module.functions.iter().filter(|f| {
            f.name.starts_with("__nuis_effect_call_") || matches!(f.name.as_str(), "start" | "step")
        }) {
            let helper = function.name.starts_with("__nuis_effect_call_");
            if helper {
                helpers += 1;
            } else {
                entries += 1;
            }
            let sensitive = function
                .body_nodes
                .iter()
                .filter_map(|name| {
                    let n = module.nodes.iter().find(|n| &n.name == name).unwrap();
                    (n.op.instruction.starts_with("call_")
                        || matches!(n.op.instruction.as_str(), "guard_return" | "div" | "rem"))
                    .then_some(n)
                })
                .collect::<Vec<_>>();
            if helper {
                assert_eq!(sensitive[0].op.instruction, "guard_return");
            } else {
                let calls = sensitive
                    .iter()
                    .filter(|n| n.op.instruction.starts_with("call_"))
                    .map(|n| n.op.instruction.as_str())
                    .collect::<Vec<_>>();
                assert!(calls.len() >= 2);
                assert_eq!(calls[0], "call_i64");
                assert!(calls[1..].iter().all(|call| *call == "call_bool"));
            }
            for pair in sensitive.windows(2) {
                let mut seen = std::collections::BTreeSet::new();
                let mut pending = vec![pair[0].name.as_str()];
                while let Some(from) = pending.pop() {
                    for edge in module.edges.iter().filter(|e| {
                        e.from == from
                            && matches!(e.kind, yir_core::EdgeKind::Effect)
                            && function.body_nodes.contains(&e.to)
                    }) {
                        if seen.insert(edge.to.as_str()) {
                            pending.push(edge.to.as_str());
                        }
                    }
                }
                assert!(seen.contains(pair[1].name.as_str()));
            }
        }
        assert!(helpers > 0);
        assert_eq!(entries, 2);
        assert!(emit_for_packaging_mode(&module, "native-session-aot-bundle:counter").is_err());
        module
            .edges
            .retain(|e| !matches!(e.kind, yir_core::EdgeKind::Effect));
        assert!(emit_for_packaging_mode(&module, &mode)
            .unwrap_err()
            .contains("lacks dependency order"));
    }
}
