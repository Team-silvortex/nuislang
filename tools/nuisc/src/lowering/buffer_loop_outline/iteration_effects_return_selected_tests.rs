use super::*;
use crate::frontend::parse_nuis_module;
#[allow(dead_code)]
#[path = "../../../tests/native_application_bridge/state_loop_return_selected_cases.rs"]
mod cases;

#[test]
fn iteration_effects_return_selected_admits_typed_returns_without_pure_authority() {
    for variant in cases::variants() {
        let mut module = parse_nuis_module(&cases::source(variant)).unwrap();
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
        assert!(!pure.contains_key("work"));
        assert!(!pure.contains_key("extent"));
        assert!(catalog.contains_key("work"), "{variant:?}");
        assert!(Plan::collect(&module, &layouts, &pure).contains("work"));
        let work = module.functions.iter().find(|f| f.name == "work").unwrap();
        assert_eq!(work.return_type.as_ref().unwrap().name, variant.kind.ty());
        let plan = control_loops::returns::prepare_effects(work, &layouts, &catalog)
            .unwrap()
            .unwrap();
        assert!(
            scalar_helpers::validate_iteration_body(work, &plan.body, &catalog, &layouts).is_some()
        );
        assert!(
            scalar_helpers::validate_control_body(work, &plan.body, &catalog, &layouts).is_none()
        );
        let outlined = outline_buffer_loops(&mut module).unwrap();
        for helper in ["work", "extent", "returned"] {
            assert!(outlined.functions.contains(helper), "{variant:?}: {helper}");
        }
    }
}

#[test]
fn iteration_effects_return_selected_oracle_checks_demand_bounds_and_exact_return_prefixes() {
    for variant in cases::variants() {
        let call = cases::base(variant);
        let (state, trace) = call.execute(false, Some(5)).unwrap();
        assert_eq!(trace, [5, 8, 6, 16, 7, 10, 11, 12, 30, 35]);
        assert!(!state.result);
        assert_eq!(state.gain, call.input.gain);
        assert_eq!(state.marker, call.input.marker);
        assert_eq!(
            state.total,
            if variant.kind == cases::Kind::Record {
                if variant.leading {
                    120
                } else {
                    109
                }
            } else {
                5
            }
        );
        assert_eq!(call.execute(false, Some(4)).unwrap_err(), [5, 8, 6, 16, 7]);
        let late = cases::late(variant);
        let (state, trace) = late.execute(false, Some(5)).unwrap();
        assert_eq!(
            trace,
            [5, 8, 9, 23, 21, 22, 8, 6, 16, 7, 10, 11, 12, 30, 35]
        );
        assert_eq!(
            state.total,
            if variant.kind == cases::Kind::Record {
                if variant.leading {
                    224
                } else {
                    213
                }
            } else {
                4
            }
        );
        assert_eq!(
            late.execute(false, Some(4)).unwrap_err(),
            [5, 8, 9, 23, 21, 22, 8, 6, 16, 7]
        );
        for (divisor, trips) in [(0, i64::MAX), (-1, i64::MIN)] {
            for inactive in cases::inactive_sequence(variant, divisor, trips) {
                assert_eq!(inactive.trace(false), [5, 8, 9, 23, 21, 22, 31, 35]);
                assert!(inactive.execute(true, Some(1)).is_ok());
            }
            let mut selected = call;
            selected.divisor = divisor;
            selected.trips = trips;
            assert_eq!(selected.execute(false, None).unwrap_err(), [5, 8, 6, 16]);
        }
        for bad in cases::failures(variant) {
            assert!(
                cases::after_prefix(bad).execute(true, None).is_err(),
                "{bad:?}"
            );
        }
        for open in cases::cases(variant) {
            assert_eq!(cases::sequence(open).len(), 4);
            for event in cases::sequence(open).into_iter().skip(1) {
                assert_ne!(event.input, event.prior());
            }
        }
    }
}

#[test]
fn iteration_effects_return_selected_rejects_scope_effect_and_step_authority_leaks() {
    for variant in cases::variants() {
        for (from, to) in [
            ("print(7);", "print(7); width = width + 1;"),
            ("print(7);", "print(7); rounds = rounds + 1;"),
            ("print(7);", "print(7); outer = outer + 1;"),
            ("print(23);", "let escaped = width; print(23);"),
            ("print(12);", "print(current);"),
            ("print(16);", "print(trips);"),
            (
                "return returned(choice, current, narrow, wide, index, outer, 1 / root);",
                "return 1;",
            ),
            ("continue;", "continue; print(99);"),
        ] {
            let source = cases::source(variant);
            assert!(source.contains(from));
            let Ok(module) = parse_nuis_module(&source.replace(from, to)) else {
                continue;
            };
            let layouts = control_values::layouts(&module);
            let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
            assert!(
                !Plan::collect(&module, &layouts, &pure).contains("work"),
                "{variant:?}: {to}"
            );
        }
    }
}
