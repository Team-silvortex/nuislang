use super::sibling_bound_tests::{cases, run_entry};
use super::tests::plans;
use super::*;

#[test]
fn effect_return_storage_conditional_payload_full_helpers_fit_native_function_bound() {
    for variant in cases::conditional_payload_variants() {
        let (mut module, _, _) = plans(&cases::source(variant), "work");
        let outlined = outline_buffer_loops(&mut module).unwrap();
        assert!(module.functions.len() <= 64, "{variant:?}");
        assert_eq!(
            outlined
                .guarded_functions
                .iter()
                .filter(|name| name.starts_with("__nuis_scalar_predicate_"))
                .count(),
            2,
            "only the two effectful field predicates require helpers"
        );
    }
}

#[test]
fn effect_return_storage_conditional_payload_oracle_covers_lazy_truth_table_and_fault_stages() {
    let mut seen = std::collections::BTreeSet::new();
    for variant in cases::conditional_payload_variants() {
        for calls in cases::healthy(variant) {
            for call in calls {
                let (state, trace) = call.execute(true, None).unwrap();
                let mut previous = call;
                previous.variant.conditional_payload = false;
                assert_eq!(state, previous.execute(true, None).unwrap().0);
                for (i, effect) in trace.iter().enumerate() {
                    if !matches!(effect, 991 | 995) {
                        continue;
                    }
                    assert_eq!(trace[i + 1], 997);
                    let rhs = trace[i + 2] == 998;
                    let branch = if rhs {
                        assert_eq!(trace[i + 3], 999);
                        trace[i + 4]
                    } else {
                        trace[i + 2]
                    };
                    assert!(if *effect == 991 {
                        matches!(branch, 1000 | 1001)
                    } else {
                        matches!(branch, 1002 | 1003)
                    });
                    seen.insert((*effect, rhs, branch));
                }
            }
        }
        assert_eq!(cases::failures(variant).len(), 31);
        for (_, call) in cases::failures(variant)
            .into_iter()
            .filter(|(_, c)| c.mode >= 28)
        {
            let trace = call.execute(true, None).unwrap_err();
            assert!(!trace.contains(&982));
            assert!(!trace.contains(&990));
            assert!(!trace.contains(&949));
            assert_eq!(
                trace.iter().filter(|&&n| n == 941).count(),
                if call.mode >= 32 { 2 } else { 1 }
            );
            assert!(matches!(trace.last(), Some(998 | 1001 | 1002)));
        }
    }
    assert_eq!(
        seen,
        [
            (991, false, 1001),
            (991, true, 1000),
            (991, true, 1001),
            (995, false, 1002),
            (995, true, 1002),
            (995, true, 1003)
        ]
        .into_iter()
        .collect()
    );
}

#[test]
fn effect_return_storage_conditional_payload_requires_transitive_predicate_authority() {
    super::field_payload_tests::check_authority(cases::conditional_payload_variants());
    for variant in cases::conditional_payload_variants() {
        let (module, layouts, catalog) = plans(&cases::source(variant), "work");
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        assert!(pure.contains_key("record_select"));
        for effectful in ["record_gate", "record_rhs", "record_marker", "record_total"] {
            assert!(!pure.contains_key(effectful));
        }
        for leaf in ["record_gate", "record_rhs", "record_select"] {
            for parent in ["record_marker", "record_total"] {
                assert!(catalog[parent].dependencies.contains(leaf));
            }
            let mut denied = module.clone();
            denied.functions.retain(|function| function.name != leaf);
            let denied = scalar_helpers::collect_iteration_calls(&denied, &layouts);
            for name in [
                leaf,
                "record_marker",
                "record_total",
                "project_record",
                "work",
            ] {
                assert!(!denied.contains_key(name), "{name}");
            }
        }
    }
}

#[test]
fn effect_return_storage_conditional_payload_keeps_existing_scalar_writes_outside_admission() {
    let source = cases::source(cases::conditional_payload_variants()[0]);
    let source = source.replace(
        "let selected_root = record_select(choice, root, 0);",
        "let mut selected_root: i64 = 0; if choice { selected_root = root; } else { selected_root = 0; }",
    );
    let module = crate::frontend::parse_nuis_module(&source).unwrap();
    let before = module.clone();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    assert!(catalog.contains_key("record_total"));
    for name in ["record_marker", "project_record", "work"] {
        assert!(!catalog.contains_key(name));
    }
    assert_eq!(module, before);
}

#[test]
fn effect_return_storage_conditional_payload_matches_independent_execution_and_carry_width() {
    super::payload_call_tests::check_execution(cases::conditional_payload_variants());
}

#[test]
fn effect_return_storage_conditional_payload_rejects_caller_reordering_and_guard_bypasses() {
    super::projected_payload_tests::check_certificate(cases::conditional_payload_variants());
    for variant in cases::conditional_payload_variants() {
        let (module, layouts, catalog) = plans(&cases::source(variant), "work");
        let function = module.functions.iter().find(|f| f.name == "work").unwrap();
        let independent =
            rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
        for mutation in [6, 7] {
            let mut shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
            assert!(super::projected_payload_tests::mutate_body(
                &mut shared.body,
                mutation
            ));
            assert!(scalar_helpers::validate_iteration_body(
                function,
                &shared.body,
                &catalog,
                &layouts
            )
            .is_some());
            assert!(verify(function, &independent, &shared, "carry", &layouts, &catalog).is_none());
        }
    }
}

#[test]
fn effect_return_storage_conditional_payload_detects_eager_and_pruned_rhs() {
    use super::execution_tests::lowered;
    use super::unprojected_payload_tests::invoke;
    let variant = cases::conditional_payload_variants()[0];
    let source = cases::source(variant);
    let mut call = cases::base(variant);
    call.mode = 1;
    let (state, expected) = call.execute(false, None).unwrap();
    let original = lowered(&invoke(&source, call), false);
    yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(&original),
        &yir_verify::default_registry(),
    )
    .expect("unmodified lazy-field program must execute");
    for op in ["&&", "||"] {
        let old = format!("record_gate(choice) {op} record_rhs(selected_root)");
        assert_eq!(source.matches(&old).count(), 1);
        for eager in [false, true] {
            // Alternative source programs test oracle sensitivity, not rewrite authority.
            let new = if eager {
                format!("record_rhs(selected_root) {op} record_gate(choice)")
            } else {
                "record_gate(choice)".into()
            };
            let changed = source.replace(&old, &new);
            for independent in [false, true] {
                let yir = lowered(&invoke(&changed, call), independent);
                for reversed in [false, true] {
                    let actual = run_entry(yir.clone(), reversed);
                    if eager {
                        assert!(actual.is_err(), "{op}");
                    } else {
                        let (value, trace) = actual.unwrap();
                        assert_eq!(value, yir_core::Value::Int(state.total));
                        assert_ne!(trace, expected, "{op}");
                    }
                }
            }
        }
    }
}
