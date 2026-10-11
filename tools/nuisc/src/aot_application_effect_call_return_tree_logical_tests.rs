use super::*;

const SOURCE: &str = include_str!(
    "../tests/native_application_bridge/return_tree_logical_effectful_scalar_selections.ns"
);

#[test]
fn native_return_tree_logical_effectful_scalar_selections_keep_transitive_ancestor_condition_leaf_check_call_order(
) {
    let (_, module, mode) = compiled(SOURCE);
    let bridge = emit_for_packaging_mode(&module, &mode).unwrap();
    assert_eq!(bridge.state_layout.source(), "State{seed:bool;first:bool;second:bool;reply:bool;outer:i64;ycond:i64;ncond:i64;a:i64;ar:i64;b:i64;br:i64;c:i64;cr:i64;d:i64;dr:i64;result:bool}");
    let node = |name: &String| module.nodes.iter().find(|n| &n.name == name).unwrap();
    let mut predicates = 0;
    let mut guarded_checks = 0;
    let mut child_carriers = 0;
    for function in module
        .functions
        .iter()
        .filter(|f| f.name.starts_with("__nuis_effect_call_"))
    {
        predicates += usize::from(function.name.starts_with("__nuis_effect_call_predicate_"));
        let sensitive = function
            .body_nodes
            .iter()
            .filter(|name| {
                let instruction = node(name).op.instruction.as_str();
                instruction.starts_with("call_")
                    || matches!(instruction, "guard_return" | "div" | "rem")
            })
            .collect::<Vec<_>>();
        assert_eq!(node(sensitive[0]).op.instruction, "guard_return");
        guarded_checks += usize::from(
            sensitive
                .iter()
                .any(|name| node(name).op.instruction == "div"),
        );
        child_carriers += usize::from(
            function.name.starts_with("__nuis_effect_call_arm")
                && sensitive.iter().any(|name| {
                    let op = &node(name).op;
                    op.instruction == "call_bool"
                        && op
                            .args
                            .first()
                            .is_some_and(|name| name.starts_with("__nuis_effect_call_arm"))
                }),
        );
        for pair in sensitive.windows(2) {
            let mut seen = std::collections::BTreeSet::new();
            let mut pending = vec![pair[0].as_str()];
            while let Some(from) = pending.pop() {
                for edge in module.edges.iter().filter(|edge| {
                    edge.from == from
                        && matches!(edge.kind, yir_core::EdgeKind::Effect)
                        && function.body_nodes.contains(&edge.to)
                }) {
                    if seen.insert(edge.to.as_str()) {
                        pending.push(edge.to.as_str());
                    }
                }
            }
            assert!(
                seen.contains(pair[1].as_str()),
                "{} lacks an Effect path: {pair:?}",
                function.name
            );
        }
    }
    assert_eq!(predicates, 8);
    assert_eq!(guarded_checks, 10);
    assert_eq!(child_carriers, 2);
    for inverted in [false, true] {
        let child = "if decide(saved) { return saved && gate; } else { return false; }";
        let (yes, no) = if inverted {
            ("return saved;", child)
        } else {
            (child, "return saved;")
        };
        let text = format!("mod cpu Main {{ struct State {{ result: bool }}
            @noinline fn decide(v: bool) -> bool {{ print(98); return v; }}
            fn observe(gate: bool, saved: bool) -> bool {{ if gate {{ {yes} }} else {{ {no} }} print(80); return false; }}
            fn start(gate: bool, value: bool) -> State {{ return State {{ result: observe(gate,value) }}; }}
            fn step(state: State, gate: bool) -> State {{ return start(gate,state.result); }}
            fn stop(state: State) -> State {{ return state; }} fn main() -> i64 {{ return 0; }} }}");
        let (_, module, mode) = compiled(&text);
        assert_eq!(
            emit_for_packaging_mode(&module, &mode)
                .unwrap()
                .state_layout
                .source(),
            "State{result:bool}"
        );
    }
}

#[test]
fn native_return_tree_logical_effectful_scalar_selections_preserve_source_free_policy_and_removed_effect_order_vetoes(
) {
    let (_, mut module, mode) = compiled(SOURCE);
    assert!(emit_for_packaging_mode(&module, "native-session-aot-bundle:counter").is_err());
    let empty = literal_print_packaging_mode("counter", Vec::<String>::new(), 0, 100).unwrap();
    assert!(emit_for_packaging_mode(&module, &empty).is_err());
    let fixture = fixture_with_source(&mode, Some(SOURCE));
    crate::aot::verify_nuis_compiled_artifact(&fixture.0.join("nuis.compiled.artifact")).unwrap();
    module
        .edges
        .retain(|edge| !matches!(edge.kind, yir_core::EdgeKind::Effect));
    let error = emit_for_packaging_mode(&module, &mode).unwrap_err();
    assert!(error.contains("lacks dependency order"), "{error}");
}
