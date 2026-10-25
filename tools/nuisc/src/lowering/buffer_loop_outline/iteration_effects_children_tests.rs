use super::*;
use crate::frontend::parse_nuis_module;
#[allow(dead_code)]
#[path = "../../../tests/native_application_bridge/state_loop_children_cases.rs"]
mod cases;

#[test]
fn iteration_effects_children_oracle_distinguishes_current_child_and_parent_indices() {
    assert_eq!(cases::cases().len(), 100);
    for leading in [false, true] {
        let failures = cases::failures(leading);
        assert_eq!(failures.len(), 16);
        assert_eq!(
            failures
                .iter()
                .filter(|(_, bad)| bad.is_none() || *bad == Some(6))
                .count(),
            10
        );
        assert!(failures.iter().any(|(call, bad)| bad.is_none()
            && call.suffix == 2
            && call
                .execute(None, false)
                .unwrap_err()
                .iter()
                .filter(|n| **n == 22)
                .count()
                == 1));
    }
    for (leading, gate, total, result, current) in [
        (false, false, 605, false, 582),
        (false, true, 809, false, 786),
        (true, false, 423, false, 400),
        (true, true, 762, true, 739),
    ] {
        let call = cases::invocation(
            cases::Case {
                seed: 8,
                gate,
                flag: false,
                trips: 3,
                cut: 1,
                rounds: 2,
                marker: 0x8000000000000000,
            },
            None,
            leading,
            false,
        );
        assert_eq!(call.next().words(), [i64::from(result), total, i64::MIN]);
        let trace = call.trace(true);
        assert_eq!(
            &trace[trace.len() - 8..],
            &[31, current, 2, 35, total, 0, 1, 40]
        );
        let effects = call.trace(false);
        for marker in [8, 21, 22] {
            assert_eq!(effects.iter().filter(|n| **n == marker).count(), 2);
        }
        for marker in [12, 13] {
            assert_eq!(effects.iter().filter(|n| **n == marker).count(), 1);
        }
        assert!(call.execute_with_budget(None, false, Some(8)).is_ok());
        let failed = call.execute_with_budget(None, false, Some(7)).unwrap_err();
        assert_eq!(failed.last(), Some(&8));
        assert_eq!(failed.iter().filter(|n| **n == 22).count(), 1);
    }
}

#[test]
fn iteration_effects_children_keep_control_local_and_recover_only_child_indices() {
    for variant in cases::variants() {
        let mut module = parse_nuis_module(&variant.source()).unwrap();
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        assert!(!pure.contains_key("walk"));
        assert!(
            Plan::collect(&module, &layouts, &pure).contains("walk"),
            "{variant:?}"
        );
        let outlined = outline_buffer_loops(&mut module).unwrap();
        assert_eq!(outlined.break_controls.len(), 1);
        let mut names = BTreeSet::new();
        for function in &module.functions {
            branches::collect_bindings(&function.body, &mut names);
        }
        assert_eq!(
            names
                .iter()
                .filter(|n| n.starts_with("__nuis_advanced_index_"))
                .count(),
            usize::from(variant.leading)
        );
        assert!(
            !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("walk")
        );
    }
}

#[test]
fn iteration_effects_children_cannot_write_parent_induction_or_headers() {
    for effect in [
        "outer = outer + 1;",
        "rounds = rounds + 1;",
        "trips = trips + 1;",
        "print(true);",
        "index = index + 1; print(99); continue;",
    ] {
        let source = cases::SOURCE.replace(
            "print(13);\n                    index = index + 1;\n                    continue;",
            effect,
        );
        assert_ne!(source, cases::SOURCE);
        let Ok(module) = parse_nuis_module(&source) else {
            continue;
        };
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        assert!(
            !Plan::collect(&module, &layouts, &pure).contains("walk"),
            "{effect}"
        );
    }
}
