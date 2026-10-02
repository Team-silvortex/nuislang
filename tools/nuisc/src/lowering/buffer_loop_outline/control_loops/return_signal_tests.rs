use super::*;
use crate::frontend::parse_nuis_module;

#[path = "../../../../tests/control_flow_syntax_native/scoped_return_signal_cases.rs"]
mod fixture;

#[test]
fn return_signal_only_reuses_minted_signals_and_keeps_the_canonical_slot_last() {
    for (case, breaks, shared) in fixture::CASES {
        for leading in [false, true] {
            let source = fixture::source(case, leading, 4, 3);
            let mut module = parse_nuis_module(&source).unwrap();
            let layouts = control_values::layouts(&module);
            let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
            assert!(catalog.contains_key("work"), "{case} leading={leading}");
            let function = module.functions.iter().find(|f| f.name == "work").unwrap();
            let plan = returns::prepare(function, &layouts, &catalog, &BTreeSet::new())
                .unwrap()
                .unwrap();
            assert_ne!(plan.signal.name(), "__nuis_return_pending_0");
            let outlines = outline_buffer_loops(&mut module).unwrap();
            assert_eq!(
                outlines.break_controls.len(),
                breaks,
                "{case} leading={leading}"
            );
            assert_eq!(
                outlines
                    .break_controls
                    .values()
                    .filter(|name| name.as_str() == plan.signal.name())
                    .count(),
                shared,
                "{case} leading={leading}"
            );
            for (helper, flag) in outlines.break_controls {
                let function = module.functions.iter().find(|f| f.name == helper).unwrap();
                assert_eq!(function.params.iter().filter(|p| p.name == flag).count(), 1);
                let Some(NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))) =
                    function.body.last()
                else {
                    panic!("canonical break return");
                };
                assert_eq!(fields.last().unwrap().1, NirExpr::Var(flag));
            }
        }
    }
}

#[test]
fn return_signal_matches_independent_lowering_and_source_exit_oracles() {
    let manifest =
        crate::registry::load_manifest(std::path::Path::new("nustar-packages"), "official.cpu")
            .unwrap();
    for (case, _, _) in fixture::CASES {
        for leading in [false, true] {
            for limit in 0..=4 {
                for stop in 0..=4 {
                    let source = fixture::source(case, leading, limit, stop);
                    let module = parse_nuis_module(&source).unwrap();
                    let layouts = control_values::layouts(&module);
                    let mut independent = module.clone();
                    for function in &mut independent.functions {
                        if let Some(body) = returns::normalize(function, &layouts).unwrap() {
                            function.body = body;
                        }
                    }
                    // Pre-normalized input has no provenance from this outline
                    // pass, even though its generated binding names look alike.
                    let mut check = independent.clone();
                    let outlines = outline_buffer_loops(&mut check).unwrap();
                    assert!(outlines
                        .break_controls
                        .values()
                        .all(|name| !name.starts_with("__nuis_return_pending")));
                    let execute = |module: &NirModule| {
                        let mut yir =
                            crate::lowering::lower_nir_to_yir(module, &manifest, None).unwrap();
                        yir.nodes.reverse();
                        yir.functions.reverse();
                        for function in &mut yir.functions {
                            function.body_nodes.reverse();
                        }
                        yir_verify::verify_module(&yir).unwrap();
                        let trace = yir_runtime_host::execute_module_source_with_registry(
                            &crate::render::render_yir(&yir),
                            &yir_verify::default_registry(),
                        )
                        .unwrap();
                        let entry = yir
                            .functions
                            .iter()
                            .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                            .unwrap();
                        trace.values[&entry.result.as_ref().unwrap().node].clone()
                    };
                    let actual = execute(&module);
                    assert_eq!(
                        actual,
                        execute(&independent),
                        "{case} leading={leading} limit={limit} stop={stop}"
                    );
                    assert_eq!(
                        actual,
                        yir_core::Value::Int(fixture::expected(case, leading, limit, stop)),
                        "{source}"
                    );
                }
            }
        }
    }
}
