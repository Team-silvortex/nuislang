use super::*;

const SOURCE: &str =
    include_str!("../tests/native_application_bridge/repeated_effectful_scalar_selections.ns");

#[test]
fn native_repeated_effectful_scalar_selections_order_sibling_stages_and_keep_exact_types() {
    let (_, module, mode) = compiled(SOURCE);
    emit_for_packaging_mode(&module, &mode).unwrap();
    let mut carriers = 0;
    for function in module
        .functions
        .iter()
        .filter(|f| f.name.starts_with("__nuis_effect_call_arm_"))
    {
        let node = |name: &String| module.nodes.iter().find(|n| &n.name == name).unwrap();
        let ordered = function
            .body_nodes
            .iter()
            .filter(|name| {
                matches!(
                    node(name).op.instruction.as_str(),
                    "guard_return" | "call_i64" | "call_bool" | "div"
                )
            })
            .collect::<Vec<_>>();
        if ordered.len() != 11 {
            continue;
        }
        carriers += 1;
        assert_eq!(node(ordered[0]).op.instruction, "guard_return");
        assert_eq!(node(ordered[2]).op.instruction, "call_bool");
        assert_eq!(node(ordered[6]).op.instruction, "call_bool");
        for pair in ordered.windows(2) {
            assert!(
                module.edges.iter().any(|edge| edge.from == *pair[0]
                    && edge.to == *pair[1]
                    && matches!(edge.kind, yir_core::EdgeKind::Effect)),
                "{}: {pair:?}",
                function.name
            );
        }
    }
    assert_eq!(carriers, 2);
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for prefix in [false, true] {
            for mask in 0..8 {
                let child = |gate: &str, inverted| {
                    let leaf = format!("let selected: {kind} = work(selected);");
                    if inverted {
                        format!("if decide({gate}) {{}} else {{ {leaf} }}")
                    } else {
                        format!("if decide({gate}) {{ {leaf} }}")
                    }
                };
                let prefix = if prefix {
                    "let staged = work(selected);"
                } else {
                    ""
                };
                let body = format!(
                    "{prefix} {} let staged = work(selected); {}
                    let selected: {kind} = work(selected);",
                    child("first", mask & 1 != 0),
                    child("second", mask & 2 != 0)
                );
                let body = if mask & 4 != 0 {
                    format!("if decide(gate) {{}} else {{ {body} }}")
                } else {
                    format!("if decide(gate) {{ {body} }}")
                };
                let source = format!("mod cpu Main {{ struct State {{ result: {kind} }}
                    @noinline fn work(v: {kind}) -> {kind} {{ print(70); return v; }}
                    @noinline fn decide(v: bool) -> bool {{ print(98); return v; }}
                    fn start(gate: bool, first: bool, second: bool, value: {kind}) -> State {{
                        let selected = value; {body} return State {{ result: selected }}; }}
                    fn step(state: State, gate: bool, first: bool, second: bool) -> State {{
                        return start(gate,first,second,state.result); }}
                    fn stop(state: State) -> State {{ return state; }} fn main() -> i64 {{ return 0; }} }}");
                let (_, module, mode) = compiled(&source);
                let bridge = emit_for_packaging_mode(&module, &mode).unwrap();
                assert_eq!(
                    bridge.state_layout.source(),
                    format!("State{{result:{kind}}}")
                );
                let helpers = module
                    .functions
                    .iter()
                    .filter(|f| f.name.starts_with("__nuis_effect_call_arm_"))
                    .collect::<Vec<_>>();
                assert_eq!(helpers.len(), 6);
                for function in helpers {
                    assert_eq!(function.result.as_ref().unwrap().ty, kind);
                    assert!(function
                        .body_nodes
                        .iter()
                        .any(|name| module.nodes.iter().any(
                            |node| &node.name == name && node.op.instruction == "guard_return"
                        )));
                }
            }
        }
    }
}

#[test]
fn native_repeated_effectful_scalar_selections_preserve_policy_checkpoint_and_removed_order_vetoes()
{
    let (_, mut module, mode) = compiled(SOURCE);
    assert!(emit_for_packaging_mode(&module, "native-session-aot-bundle:counter").is_err());
    let empty = literal_print_packaging_mode("counter", Vec::<String>::new(), 0, 100).unwrap();
    assert!(emit_for_packaging_mode(&module, &empty).is_err());
    let fixture = fixture_with_source(&mode, Some(SOURCE));
    crate::aot::verify_nuis_compiled_artifact(&fixture.0.join("nuis.compiled.artifact")).unwrap();
    let path = fixture.0.join("nuis.build.manifest.toml");
    let text = fs::read_to_string(&path).unwrap();
    verify_sources(
        &text,
        &path,
        &parse_artifact_hash_blocks(&text, &path).unwrap(),
    )
    .unwrap();
    module
        .edges
        .retain(|edge| !matches!(edge.kind, yir_core::EdgeKind::Effect));
    let error = emit_for_packaging_mode(&module, &mode).unwrap_err();
    assert!(error.contains("lacks dependency order"), "{error}");
}
