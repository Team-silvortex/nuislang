use super::*;

const SOURCE: &str = include_str!(
    "../tests/native_application_bridge/return_i32_pair_mixed_logical_effectful_scalar_selections.ns"
);

#[test]
fn native_return_i32_pair_mixed_logical_effectful_scalar_selections_keep_guarded_transitive_update_stage_return_order_and_exact_state(
) {
    let (_, module, mode) = compiled(SOURCE);
    assert_eq!(emit_for_packaging_mode(&module,&mode).unwrap().state_layout.source(),"State{seed:i64;first:bool;second:bool;outer:i64;init:i64;child_init:i64;cond:i64;child_rhs:i64;parent_rhs:i64;after:i64;unused:i64;answer:i64;result:bool}");
    let node = |name: &String| module.nodes.iter().find(|n| &n.name == name).unwrap();
    let mut checked_functions = 0;
    let mut guarded_calls = 0;
    for function in module
        .functions
        .iter()
        .filter(|f| f.name.starts_with("__nuis_effect_call_"))
    {
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
        checked_functions += usize::from(
            sensitive
                .iter()
                .any(|name| node(name).op.instruction == "div"),
        );
        guarded_calls += sensitive
            .iter()
            .filter(|name| node(name).op.instruction.starts_with("call_"))
            .count();
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
    assert!(checked_functions >= 9);
    assert!(module
        .nodes
        .iter()
        .any(|n| n.op.instruction == "cast_i32_to_i64"));
    assert!(module
        .nodes
        .iter()
        .any(|n| n.op.instruction == "cast_i64_to_i32"));
    assert!(guarded_calls >= 16);
    for inverted in [false, true] {
        let selected="let local = saved;if first {let child = i32_from_i64(7);if saved {let local = child == i32_from_i64(7);let child = relay(child);} else {let child = relay(child);let local = first;} if local {let child = relay(child);let local = child == i32_from_i64(7);} const seal: bool = local || false;let local = local && seal;} return local && decide(first);";
        let (yes, no) = if inverted {
            ("return saved;", selected)
        } else {
            (selected, "return saved;")
        };
        let text=format!("mod cpu Main {{ struct State {{ result: bool }}
            @noinline fn decide(value: bool) -> bool {{ print(98); return value; }}
            @noinline fn relay(value: i32) -> i32 {{ print(96); return value; }}
            fn observe(gate: bool, first: bool, saved: bool) -> bool {{ if gate {{ {yes} }} else {{ {no} }} print(80); return false; }}
            fn start(gate: bool, first: bool, saved: bool) -> State {{ return State {{ result: observe(gate,first,saved) }}; }}
            fn step(state: State, gate: bool) -> State {{ return start(gate,state.result,state.result); }}
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
fn native_return_i32_pair_mixed_logical_effectful_scalar_selections_preserve_source_free_policy_and_removed_update_order_vetoes(
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
