use super::*;
use crate::frontend::parse_nuis_module;
#[allow(dead_code)]
#[path = "../../../tests/native_application_bridge/state_loop_return_parent_mixed_cases.rs"]
mod cases;

#[test]
fn iteration_effects_return_parent_mixed_keeps_local_break_and_return_signals_independent() {
    assert_eq!(cases::variants().len(), 32);
    assert_eq!(cases::cli_variants().len(), 4);
    for variant in cases::variants() {
        let mut module = parse_nuis_module(&cases::source(variant)).unwrap();
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
        assert!(!pure.contains_key("work"));
        assert!(!pure.contains_key("extent"));
        assert!(
            Plan::collect(&module, &layouts, &pure).contains("work"),
            "{variant:?}"
        );
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
        let pending = plan.signal.name().to_owned();
        assert!(
            scalar_helpers::validate_iteration_body(work, &plan.body, &catalog, &layouts).is_some()
        );
        assert!(
            scalar_helpers::validate_control_body(work, &plan.body, &catalog, &layouts).is_none()
        );
        let outlined = outline_buffer_loops(&mut module).unwrap();
        let work = module.functions.iter().find(|f| f.name == "work").unwrap();
        let parent_helper = work
            .body
            .iter()
            .find_map(|s| match s {
                NirStmt::While { body, .. } => body.iter().find_map(|s| match s {
                    NirStmt::Let {
                        value: NirExpr::Call { callee, .. },
                        ..
                    } if outlined.break_controls.contains_key(callee) => Some(callee),
                    _ => None,
                }),
                _ => None,
            })
            .unwrap();
        assert_ne!(
            outlined.break_controls[parent_helper], pending,
            "{variant:?}"
        );
        assert!(outlined.break_controls.len() >= 2);
        for helper in ["work", "extent", "returned", "checkpoint"] {
            assert!(outlined.functions.contains(helper), "{variant:?}: {helper}");
        }
    }
}

#[test]
fn iteration_effects_return_parent_mixed_oracle_keeps_break_continue_and_return_distinct() {
    for variant in cases::variants() {
        let early = cases::base(variant);
        assert_eq!(early.finish, 0);
        assert_eq!(early.trace(false), [5, 8, 6, 16, 7, 10, 11, 12, 30, 35]);
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
        let mixed = cases::continued_then_break(variant);
        assert_eq!(
            mixed.execute(false, Some(2)).unwrap().1,
            [5, 8, 9, 25, 8, 6, 16, 7, 25, 31, 33, 32, 35]
        );
        let observed = mixed.trace(true);
        let fallback = observed.iter().position(|n| *n == 31).unwrap();
        assert_eq!(
            observed[fallback + 1],
            i64::from(variant.parent_leading) + 1
        );
        assert_eq!(mixed.execute(false, Some(1)).unwrap_err(), [5]);
        for (divisor, trips) in [(0, i64::MAX), (-1, i64::MIN)] {
            let call = cases::skipped(variant, divisor, trips);
            assert_eq!(call.skip, call.resume);
            assert_eq!(call.trace(false), [5, 8, 9, 25, 31, 33, 32, 35]);
            let observed = call.trace(true);
            let fallback = observed.iter().position(|n| *n == 31).unwrap();
            assert_eq!(observed[fallback + 1], i64::from(variant.parent_leading));
        }
        let trace = cases::child_continued(variant).trace(false);
        let child = trace.iter().position(|n| *n == 13).unwrap();
        let parent = trace.iter().position(|n| *n == 25).unwrap();
        let fallback = trace.iter().position(|n| *n == 31).unwrap();
        assert!(child < parent && parent < fallback);
        assert_eq!(trace.iter().filter(|n| **n == 25).count(), 2);
        for bad in cases::failures(variant) {
            assert!(
                cases::after_prefix(bad).execute(true, None).is_err(),
                "{bad:?}"
            );
        }
        assert_eq!(cases::sequences(variant).len(), 32);
        for calls in cases::sequences(variant) {
            assert_eq!(calls.len(), 4);
            for call in calls.into_iter().skip(1) {
                assert_ne!(call.input, call.prior());
            }
        }
    }
}

#[test]
fn iteration_effects_return_parent_mixed_admits_ready_i64_prints_for_each_variant() {
    for variant in cases::variants() {
        for (from, to) in [
            ("print(25);", "print(current);"),
            ("print(32);", "print(finish);"),
        ] {
            let source = cases::source(variant);
            assert!(source.contains(from));
            let module = parse_nuis_module(&source.replace(from, to)).unwrap();
            let layouts = control_values::layouts(&module);
            let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
            assert!(!pure.contains_key("work"));
            assert!(Plan::collect(&module, &layouts, &pure).contains("work"));
        }
    }
}

#[test]
fn iteration_effects_return_parent_mixed_rejects_step_scope_and_effect_authority_leaks() {
    for variant in cases::variants() {
        for (from, to) in [
            ("break;", "break; print(99);"),
            ("break;", "outer = outer + 1; break;"),
            ("print(7);", "print(7); width = width + 1;"),
            ("print(7);", "print(7); rounds = rounds + 1;"),
            ("print(23);", "let escaped = width; print(23);"),
            ("print(25);", "print(true);"),
            ("print(32);", "print(1.0);"),
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
