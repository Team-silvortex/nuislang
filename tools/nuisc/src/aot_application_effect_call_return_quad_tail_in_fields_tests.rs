use super::*;

#[test]
fn native_return_quad_tail_in_fields_keeps_typed_fields_guarded_continuation_and_policy_vetoes() {
    #[allow(dead_code)]
    #[path = "../tests/native_application_bridge/return_quad_tail_in_fields_oracle.rs"]
    mod oracle;
    for source in [oracle::SOURCE.to_owned(), oracle::mirror_source()] {
        let (_, mut module, mode) = compiled(&source);
        let bridge = emit_for_packaging_mode(&module, &mode).unwrap();
        assert!(bridge.state_layout.source().ends_with(";result:bool}"));
        assert!(module
            .nodes
            .iter()
            .any(|n| n.op.instruction == "call_owned_struct"
                && n.op.args.get(1).is_some_and(|layout| layout.ends_with(
                    "{owner:i64;child_0:i32;child_1:f32;child_2:f64;child_3:bool;child_4:bool}"
                ))));
        let continuations = module
            .functions
            .iter()
            .filter(|f| f.name.starts_with("__nuis_effect_call_"))
            .filter(|f| {
                f.body_nodes.iter().any(|name| {
                    module.nodes.iter().any(|n| {
                        &n.name == name
                            && n.op.instruction == "call_i64"
                            && n.op.args[0] == "after_work"
                    })
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(continuations.len(), 1);
        assert_eq!(continuations[0].result.as_ref().unwrap().ty, "bool");
        for function in module
            .functions
            .iter()
            .filter(|f| f.name.starts_with("__nuis_effect_call_"))
        {
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
        assert!(!bridge
            .llvm_ir
            .contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
        assert!(!bridge
            .llvm_ir
            .contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
        assert!(emit_for_packaging_mode(&module, "native-session-aot-bundle:counter").is_err());
        let empty = literal_print_packaging_mode("counter", Vec::<String>::new(), 0, 100).unwrap();
        assert!(emit_for_packaging_mode(&module, &empty).is_err());
        module
            .edges
            .retain(|e| !matches!(e.kind, yir_core::EdgeKind::Effect));
        let error = emit_for_packaging_mode(&module, &mode).unwrap_err();
        assert!(error.contains("lacks dependency order"), "{error}");
    }
}
