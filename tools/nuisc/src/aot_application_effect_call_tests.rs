use super::*;
use crate::aot_native_session::{emit_for_packaging_mode, literal_print_packaging_mode};

#[path = "aot_application_effect_call_regions_tests.rs"]
mod regions;

#[path = "aot_application_effect_call_staged_tests.rs"]
mod staged;

#[path = "aot_application_effect_call_continued_tests.rs"]
mod continued;

#[path = "aot_application_effect_call_repeated_tests.rs"]
mod repeated;

#[path = "aot_application_effect_call_ordered_tests.rs"]
mod ordered;

#[path = "aot_application_effect_call_logical_tests.rs"]
mod logical;

#[path = "aot_application_effect_call_computed_logical_tests.rs"]
mod computed_logical;

#[path = "aot_application_effect_call_tree_logical_tests.rs"]
mod tree_logical;

#[path = "aot_application_effect_call_staging_logical_tests.rs"]
mod staging_logical;

#[path = "aot_application_effect_call_update_logical_tests.rs"]
mod update_logical;

const SOURCE: &str = include_str!("../tests/native_application_bridge/effectful_selected_calls.ns");

fn compiled(source: &str) -> (Fixture, yir_core::YirModule, String) {
    let seed = fixture_for_mode("native-session-aot-bundle:counter");
    fs::write(seed.0.join("main.ns"), source).unwrap();
    let module = crate::pipeline::compile_project(&seed.0).unwrap().yir;
    let sites = module
        .nodes
        .iter()
        .filter(|node| matches!(node.op.instruction.as_str(), "print" | "guard_print"))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let mode = literal_print_packaging_mode("counter", sites, 0, 100).unwrap();
    (seed, module, mode)
}

#[test]
fn native_effectful_selected_calls_keep_source_order_and_guard_arguments() {
    let (_, module, mode) = compiled(SOURCE);
    let bridge = emit_for_packaging_mode(&module, &mode).unwrap();
    assert_eq!(
        module
            .functions
            .iter()
            .filter(|f| f.name.starts_with("__nuis_effect_call_arm_"))
            .count(),
        2
    );
    for function in module
        .functions
        .iter()
        .filter(|f| f.name.starts_with("__nuis_effect_call_arm_"))
    {
        let guard = function
            .body_nodes
            .iter()
            .find(|name| {
                module
                    .nodes
                    .iter()
                    .any(|node| &node.name == *name && node.op.instruction == "guard_return")
            })
            .unwrap();
        let calls = function
            .body_nodes
            .iter()
            .filter(|name| {
                module
                    .nodes
                    .iter()
                    .any(|node| &node.name == *name && node.op.instruction == "call_i64")
            })
            .collect::<Vec<_>>();
        assert_eq!(calls.len(), 3);
        let ordered = [guard, calls[0], calls[1], calls[2]];
        for pair in ordered.windows(2) {
            assert!(module.edges.iter().any(|edge| edge.from == *pair[0]
                && edge.to == *pair[1]
                && matches!(edge.kind, yir_core::EdgeKind::Effect)));
        }
    }
    assert!(bridge.llvm_ir.contains("@nuis_debug_print_i64"));
}

#[test]
fn native_effectful_selected_calls_preserve_profile_checks_and_reject_removed_order() {
    let (_, mut module, mode) = compiled(SOURCE);
    assert!(emit_for_packaging_mode(&module, "native-session-aot-bundle:counter").is_err());
    let empty = literal_print_packaging_mode("counter", Vec::<String>::new(), 0, 100).unwrap();
    assert!(emit_for_packaging_mode(&module, &empty).is_err());
    module
        .edges
        .retain(|edge| !matches!(edge.kind, yir_core::EdgeKind::Effect));
    let error = emit_for_packaging_mode(&module, &mode).unwrap_err();
    assert!(error.contains("lacks dependency order"), "{error}");
}

#[test]
fn native_effectful_selected_calls_bind_source_free_checkpoint() {
    let (_, _, mode) = compiled(SOURCE);
    let fixture = fixture_with_source(&mode, Some(SOURCE));
    crate::aot::verify_nuis_compiled_artifact(&fixture.0.join("nuis.compiled.artifact")).unwrap();
    let path = fixture.0.join("nuis.build.manifest.toml");
    let source = fs::read_to_string(&path).unwrap();
    let rows = parse_artifact_hash_blocks(&source, &path).unwrap();
    verify_sources(&source, &path, &rows).unwrap();
}

#[test]
fn native_effectful_selected_calls_keep_exact_scalar_guard_and_callback_kinds() {
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        for shape in ["paired", "then", "else", "nested_then", "nested_else"] {
            let selection = match shape {
            "paired" => format!("let selected: {kind} = if decide(gate) {{ work(value) }} else {{ work(fallback) }};"),
            "then" => format!("let selected: {kind} = fallback; if decide(gate) {{ let selected: {kind} = work(value); }}"),
            "else" => format!("let selected: {kind} = fallback; if decide(gate) {{}} else {{ let selected: {kind} = work(value); }}"),
            "nested_then" => format!("let selected: {kind} = fallback; if decide(gate) {{ if decide(gate) {{ let selected: {kind} = work(value); }} }}"),
            "nested_else" => format!("let selected: {kind} = fallback; if decide(gate) {{}} else {{ if decide(gate) {{}} else {{ let selected: {kind} = work(value); }} }}"),
            _ => unreachable!(),
        };
            let source = format!("mod cpu Main {{ struct State {{ result: {kind} }}
            @noinline fn work(value: {kind}) -> {kind} {{ print(70); return value; }}
            @noinline fn decide(gate: bool) -> bool {{ print(98); return gate; }}
            fn start(gate: bool, value: {kind}, fallback: {kind}) -> State {{
                {selection}
                return State {{ result: selected }};
            }}
            fn step(state: State, gate: bool) -> State {{ return start(gate, state.result, state.result); }}
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
fn native_effectful_scalar_rebindings_keep_retained_arms_and_guarded_argument_order() {
    let source = include_str!("../tests/native_application_bridge/effectful_scalar_rebindings.ns");
    let (_, module, mode) = compiled(source);
    emit_for_packaging_mode(&module, &mode).unwrap();
    let arms = module
        .functions
        .iter()
        .filter(|f| f.name.starts_with("__nuis_effect_call_arm_"))
        .collect::<Vec<_>>();
    assert_eq!(arms.len(), 4);
    let mut retained = 0;
    for function in arms {
        let instruction = |name: &String| {
            module
                .nodes
                .iter()
                .find(|node| &node.name == name)
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
        if calls.is_empty() {
            retained += 1;
        } else {
            assert_eq!(calls.len(), 3);
            for pair in [guard, calls[0], calls[1], calls[2]].windows(2) {
                assert!(module.edges.iter().any(|edge| edge.from == *pair[0]
                    && edge.to == *pair[1]
                    && matches!(edge.kind, yir_core::EdgeKind::Effect)));
            }
        }
    }
    assert_eq!(retained, 2);
}

#[test]
fn native_effectful_scalar_rebindings_preserve_policy_order_and_source_free_checkpoint_checks() {
    let source = include_str!("../tests/native_application_bridge/effectful_scalar_rebindings.ns");
    let (_, mut module, mode) = compiled(source);
    assert!(emit_for_packaging_mode(&module, "native-session-aot-bundle:counter").is_err());
    let empty = literal_print_packaging_mode("counter", Vec::<String>::new(), 0, 100).unwrap();
    assert!(emit_for_packaging_mode(&module, &empty).is_err());
    let fixture = fixture_with_source(&mode, Some(source));
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

#[test]
fn native_nested_effectful_scalar_selections_order_predicates_after_each_ancestor_guard() {
    let source =
        include_str!("../tests/native_application_bridge/nested_effectful_scalar_selections.ns");
    let (_, module, mode) = compiled(source);
    emit_for_packaging_mode(&module, &mode).unwrap();
    let arms = module
        .functions
        .iter()
        .filter(|f| f.name.starts_with("__nuis_effect_call_arm_"))
        .collect::<Vec<_>>();
    assert_eq!(arms.len(), 8);
    let mut predicate_carriers = 0;
    for function in arms {
        let instruction = |name: &String| {
            module
                .nodes
                .iter()
                .find(|node| &node.name == name)
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
            .filter(|name| instruction(name).starts_with("call_"))
            .collect::<Vec<_>>();
        predicate_carriers +=
            usize::from(calls.iter().any(|name| instruction(name) == "call_bool"));
        let ordered = std::iter::once(guard).chain(calls).collect::<Vec<_>>();
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
    assert_eq!(predicate_carriers, 3);
}

#[test]
fn native_nested_effectful_scalar_selections_keep_policy_order_and_checkpoint_vetoes() {
    let source =
        include_str!("../tests/native_application_bridge/nested_effectful_scalar_selections.ns");
    let (_, mut module, mode) = compiled(source);
    assert!(emit_for_packaging_mode(&module, "native-session-aot-bundle:counter").is_err());
    let empty = literal_print_packaging_mode("counter", Vec::<String>::new(), 0, 100).unwrap();
    assert!(emit_for_packaging_mode(&module, &empty).is_err());
    let fixture = fixture_with_source(&mode, Some(source));
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
