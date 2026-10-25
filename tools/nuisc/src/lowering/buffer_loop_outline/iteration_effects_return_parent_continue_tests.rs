use super::*;
use crate::frontend::parse_nuis_module;
#[allow(dead_code)]
#[path = "../../../tests/native_application_bridge/state_loop_return_parent_continue_cases.rs"]
mod cases;

#[test]
fn iteration_effects_return_parent_continue_admits_both_parent_steps_without_pure_authority() {
    assert_eq!(cases::variants().len(), 32);
    assert_eq!(cases::cli_variants().len(), 4);
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
        for stmt in &work.body {
            if let NirStmt::While { condition, body } = stmt {
                assert_eq!(
                    control_loops::induction::parse(condition, body)
                        .unwrap()
                        .leading,
                    variant.parent_leading
                );
            }
        }
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
fn iteration_effects_return_parent_continue_oracle_checks_steps_skipped_tails_and_later_reservations(
) {
    for variant in cases::variants() {
        let early = cases::base(variant);
        let (_, trace) = early.execute(false, Some(5)).unwrap();
        assert_eq!(trace, [5, 8, 6, 16, 7, 10, 11, 12, 30, 35]);
        let late = cases::late(variant);
        let (state, trace) = late.execute(false, Some(5)).unwrap();
        assert_eq!(trace, [5, 8, 9, 25, 8, 6, 16, 7, 10, 11, 12, 30, 35]);
        assert_eq!(
            state.total,
            if variant.kind == cases::Kind::Record {
                111 + 102 * i64::from(variant.parent_leading) + 11 * i64::from(variant.leading)
            } else {
                late.input.total + 1
            }
        );
        assert_eq!(
            late.execute(false, Some(4)).unwrap_err(),
            [5, 8, 9, 25, 8, 6, 16, 7]
        );
        for (divisor, trips) in [(0, i64::MAX), (-1, i64::MIN)] {
            for inactive in cases::inactive_sequence(variant, divisor, trips) {
                assert_eq!(inactive.trace(false), [5, 8, 9, 25, 31, 35]);
                assert!(inactive.execute(true, Some(1)).is_ok());
                assert_eq!(inactive.execute(false, Some(0)).unwrap_err(), [5]);
            }
        }
        let mut child_continue = cases::completed(variant);
        child_continue.input.total += 1;
        child_continue.cut = i64::from(variant.leading);
        child_continue.select_first(true);
        let trace = child_continue.trace(false);
        let child_exit = trace.iter().position(|n| *n == 13).unwrap();
        let parent_exit = trace.iter().position(|n| *n == 25).unwrap();
        assert!(child_exit < parent_exit, "{variant:?}: {trace:?}");
        for bad in cases::failures(variant) {
            assert!(
                cases::after_prefix(bad).execute(true, None).is_err(),
                "{bad:?}"
            );
        }
        for calls in cases::sequences(variant) {
            assert_eq!(calls.len(), 4);
            for event in calls.into_iter().skip(1) {
                assert_ne!(event.input, event.prior());
            }
        }
    }
}

#[test]
fn iteration_effects_return_parent_continue_rejects_step_scope_and_effect_authority_leaks() {
    for variant in cases::variants() {
        for (from, to) in [
            ("print(7);", "print(7); width = width + 1;"),
            ("print(7);", "print(7); rounds = rounds + 1;"),
            ("print(7);", "print(7); outer = outer + 1;"),
            ("print(23);", "let escaped = width; print(23);"),
            ("print(25);", "print(true);"),
            ("print(16);", "print(true);"),
            ("continue;", "continue; print(99);"),
            (
                "current = current + outer;\n                ",
                "current = current + outer;\n                outer = outer + 2; ",
            ),
        ] {
            let source = cases::source(variant);
            assert!(source.contains(from), "{variant:?}: {from}");
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
        if !variant.parent_leading {
            let source = cases::source(variant);
            for replacement in ["continue;", "outer = outer + 2;\n                continue;"] {
                let source =
                    source.replace("outer = outer + 1;\n                continue;", replacement);
                let module = parse_nuis_module(&source).unwrap();
                let layouts = control_values::layouts(&module);
                let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
                assert!(
                    !Plan::collect(&module, &layouts, &pure).contains("work"),
                    "{variant:?}: {replacement}"
                );
            }
        }
    }
}
