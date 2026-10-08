use super::*;

const SOURCE: &str = include_str!(
    "../tests/native_application_bridge/update_logical_effectful_scalar_selections.ns"
);

#[test]
fn native_update_logical_effectful_scalar_selections_keep_transitive_guard_check_call_order_and_exact_types(
) {
    let (_, module, mode) = compiled(SOURCE);
    let bridge = emit_for_packaging_mode(&module, &mode).unwrap();
    assert_eq!(bridge.state_layout.source(), "State{seed:bool;enabled:bool;first:bool;second:bool;reply:bool;da:i64;db:i64;dc:i64;dd:i64;first_leaf:i64;second_leaf:i64;result:bool}");
    let node = |name: &String| module.nodes.iter().find(|n| &n.name == name).unwrap();
    let mut predicates = 0;
    let mut carriers = 0;
    for function in module
        .functions
        .iter()
        .filter(|f| f.name.starts_with("__nuis_effect_call_"))
    {
        let predicate = function.name.starts_with("__nuis_effect_call_predicate_");
        predicates += usize::from(predicate);
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
        if !predicate
            && sensitive
                .iter()
                .any(|name| node(name).op.instruction == "div")
        {
            carriers += 1;
        }
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
    assert_eq!(predicates, 4);
    assert!(carriers >= 1);
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for inverted in [false, true] {
            let stage = "let local = first; let local = local && decide(saved);
                if local { let saved = work(saved); }
                let local = local || decide(saved);
                if second { let saved = work(saved); }
                let local = local && decide(saved); let saved = work(saved);";
            let body = if inverted {
                format!("if gate {{}} else {{ {stage} }}")
            } else {
                format!("if gate {{ {stage} }}")
            };
            let text = format!("mod cpu Main {{ struct State {{ result: {kind} }}
                @noinline fn work(value: {kind}) -> {kind} {{ print(70); return value; }}
                @noinline fn decide(value: {kind}) -> bool {{ print(98); return value == value; }}
                fn start(gate: bool, first: bool, second: bool, value: {kind}) -> State {{
                    let saved = value; {body} return State {{ result: saved }}; }}
                fn step(state: State, gate: bool, first: bool, second: bool) -> State {{ return start(gate,first,second,state.result); }}
                fn stop(state: State) -> State {{ return state; }} fn main() -> i64 {{ return 0; }} }}");
            let (_, module, mode) = compiled(&text);
            assert_eq!(
                emit_for_packaging_mode(&module, &mode)
                    .unwrap()
                    .state_layout
                    .source(),
                format!("State{{result:{kind}}}")
            );
            assert_eq!(
                module
                    .functions
                    .iter()
                    .filter(|f| f.name.starts_with("__nuis_effect_call_predicate_"))
                    .count(),
                3
            );
        }
    }
}

#[test]
fn native_update_logical_effectful_scalar_selections_preserve_policy_checkpoint_and_removed_order_vetoes(
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
