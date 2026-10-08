use super::*;

const SOURCE: &str =
    include_str!("../tests/native_application_bridge/logical_effectful_scalar_selections.ns");

#[test]
fn native_logical_effectful_scalar_selections_order_guards_divisions_rhs_calls_and_actual_merges() {
    let (_, module, mode) = compiled(SOURCE);
    emit_for_packaging_mode(&module, &mode).unwrap();
    let predicates = module
        .functions
        .iter()
        .filter(|f| f.name.starts_with("__nuis_effect_call_predicate_"))
        .collect::<Vec<_>>();
    assert_eq!(predicates.len(), 4);
    for function in predicates {
        assert_eq!(function.result.as_ref().unwrap().ty, "bool");
        let node = |name: &String| module.nodes.iter().find(|n| &n.name == name).unwrap();
        let ordered = function
            .body_nodes
            .iter()
            .filter(|name| {
                matches!(
                    node(name).op.instruction.as_str(),
                    "guard_return" | "div" | "call_bool"
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(ordered.len(), 3);
        assert_eq!(node(ordered[0]).op.instruction, "guard_return");
        assert_eq!(node(ordered[1]).op.instruction, "div");
        assert_eq!(node(ordered[2]).op.instruction, "call_bool");
        for pair in ordered.windows(2) {
            assert!(module.edges.iter().any(|edge| edge.from == *pair[0]
                && edge.to == *pair[1]
                && matches!(edge.kind, yir_core::EdgeKind::Effect)));
        }
    }
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for op in ["&&", "||"] {
            for inverted in [false, true] {
                let leaf = format!("let saved: {kind} = work(saved);");
                let child = if inverted {
                    format!("if first {op} decide(saved) {{}} else {{ {leaf} }}")
                } else {
                    format!("if first {op} decide(saved) {{ {leaf} }}")
                };
                let source = format!("mod cpu Main {{ struct State {{ result: {kind} }}
                    @noinline fn work(value: {kind}) -> {kind} {{ print(70); return value; }}
                    @noinline fn decide(value: {kind}) -> bool {{ print(98); return value == value; }}
                    fn start(gate: bool, first: bool, second: bool, value: {kind}) -> State {{
                        let saved = value; if gate {{ {child} if second {{ {leaf} }} }} return State {{ result: saved }}; }}
                    fn step(state: State, gate: bool, first: bool, second: bool) -> State {{ return start(gate,first,second,state.result); }}
                    fn stop(state: State) -> State {{ return state; }} fn main() -> i64 {{ return 0; }} }}");
                let (_, module, mode) = compiled(&source);
                let bridge = emit_for_packaging_mode(&module, &mode).unwrap();
                assert_eq!(
                    bridge.state_layout.source(),
                    format!("State{{result:{kind}}}")
                );
                assert_eq!(
                    module
                        .functions
                        .iter()
                        .filter(|f| f.name.starts_with("__nuis_effect_call_predicate_"))
                        .count(),
                    1
                );
            }
        }
    }
}

#[test]
fn native_logical_effectful_scalar_selections_preserve_default_policy_checkpoint_and_removed_order_vetoes(
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
