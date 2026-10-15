use super::*;

#[test]
fn native_return_exit_prefix_keeps_single_tail_and_guard_first_effect_order() {
    let source = include_str!("../tests/native_application_bridge/return_exit_prefix.ns");
    for source in [
        source.to_owned(),
        source.replace("select(pick,", "select(pick != true,"),
    ] {
        let (_, mut module, mode) = compiled(&source);
        let bridge = emit_for_packaging_mode(&module, &mode).unwrap();
        assert!(!bridge
            .llvm_ir
            .contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
        assert!(!bridge
            .llvm_ir
            .contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
        assert_eq!(
            module
                .nodes
                .iter()
                .filter(|n| n.op.instruction == "call_i64" && n.op.args[0] == "tail_work")
                .count(),
            1
        );
        for function in module
            .functions
            .iter()
            .filter(|f| f.name.starts_with("__nuis_effect_call_"))
        {
            let sensitive = function
                .body_nodes
                .iter()
                .filter_map(|name| {
                    let node = module.nodes.iter().find(|n| &n.name == name).unwrap();
                    (node.op.instruction.starts_with("call_")
                        || matches!(node.op.instruction.as_str(), "guard_return" | "div" | "rem"))
                    .then_some(node)
                })
                .collect::<Vec<_>>();
            assert_eq!(sensitive[0].op.instruction, "guard_return");
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
        assert!(emit_for_packaging_mode(&module, "native-session-aot-bundle:counter").is_err());
        module
            .edges
            .retain(|e| !matches!(e.kind, yir_core::EdgeKind::Effect));
        assert!(emit_for_packaging_mode(&module, &mode)
            .unwrap_err()
            .contains("lacks dependency order"));
    }
}
