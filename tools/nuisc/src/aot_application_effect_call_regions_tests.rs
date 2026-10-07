use super::*;

const SOURCE: &str =
    include_str!("../tests/native_application_bridge/sequential_effectful_scalar_regions.ns");

#[test]
fn native_sequential_effectful_scalar_regions_keep_guard_to_staging_order_and_exact_types() {
    let (_, module, mode) = compiled(SOURCE);
    emit_for_packaging_mode(&module, &mode).unwrap();
    let mut regions = 0;
    for function in module
        .functions
        .iter()
        .filter(|f| f.name.starts_with("__nuis_effect_call_arm_"))
    {
        let instruction = |name: &String| {
            module
                .nodes
                .iter()
                .find(|n| &n.name == name)
                .unwrap()
                .op
                .instruction
                .as_str()
        };
        let guard = function
            .body_nodes
            .iter()
            .find(|name| instruction(name) == "guard_return")
            .unwrap();
        let calls = function
            .body_nodes
            .iter()
            .filter(|name| instruction(name) == "call_i64")
            .collect::<Vec<_>>();
        if calls.len() == 4 {
            regions += 1;
            for pair in [guard, calls[0], calls[1], calls[2], calls[3]].windows(2) {
                assert!(
                    module.edges.iter().any(|edge| edge.from == *pair[0]
                        && edge.to == *pair[1]
                        && matches!(edge.kind, yir_core::EdgeKind::Effect)),
                    "{}: {pair:?}",
                    function.name
                );
            }
        }
    }
    assert_eq!(regions, 2);
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for shape in ["then", "else", "nested_then", "nested_else"] {
            let region = format!(
                "let staged: {kind} = work(selected); let selected: {kind} = work(staged);"
            );
            let region = if shape.starts_with("nested") {
                format!("if decide(gate) {{ {region} }}")
            } else {
                region
            };
            let body = if shape.ends_with("else") {
                format!("if decide(gate) {{}} else {{ {region} }}")
            } else {
                format!("if decide(gate) {{ {region} }}")
            };
            let source = format!("mod cpu Main {{ struct State {{ result: {kind} }}
                @noinline fn work(v: {kind}) -> {kind} {{ print(70); return v; }}
                @noinline fn decide(gate: bool) -> bool {{ print(98); return gate; }}
                fn start(gate: bool, value: {kind}) -> State {{
                    let selected = value; {body} return State {{ result: selected }};
                }}
                fn step(state: State, gate: bool) -> State {{ return start(gate, state.result); }}
                fn stop(state: State) -> State {{ return state; }} fn main() -> i64 {{ return 0; }} }}");
            let (_, module, mode) = compiled(&source);
            let bridge = emit_for_packaging_mode(&module, &mode).unwrap();
            assert_eq!(
                bridge.state_layout.source(),
                format!("State{{result:{kind}}}")
            );
            for function in module
                .functions
                .iter()
                .filter(|f| f.name.starts_with("__nuis_effect_call_arm_"))
            {
                assert_eq!(function.result.as_ref().unwrap().ty, kind);
                assert!(function.body_nodes.iter().any(|name| module
                    .nodes
                    .iter()
                    .any(|node| &node.name == name && node.op.instruction == "guard_return")));
            }
        }
    }
}

#[test]
fn native_sequential_effectful_scalar_regions_preserve_policy_order_and_checkpoint_vetoes() {
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
