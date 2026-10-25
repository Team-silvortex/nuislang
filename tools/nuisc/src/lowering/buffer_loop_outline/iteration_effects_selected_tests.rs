use super::*;
use crate::frontend::parse_nuis_module;
#[allow(dead_code)]
#[path = "../../../tests/native_application_bridge/state_loop_selected_cases.rs"]
mod cases;

#[test]
fn iteration_effects_selected_children_guard_bound_math_and_share_reservations() {
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
        assert!(
            !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("walk")
        );
    }
}

#[test]
fn iteration_effects_selected_children_oracle_checks_actual_selection_and_poison() {
    assert_eq!(cases::cases().len(), 100);
    for leading in [false, true] {
        let failures = cases::failures(leading);
        assert_eq!(failures.len(), 14);
        assert_eq!(
            failures
                .iter()
                .filter(|(_, slot)| slot.is_none() || *slot == Some(6))
                .count(),
            9
        );
        assert!(failures
            .iter()
            .any(|(call, bad)| *bad == Some(4)
                && call.execute(*bad, false).unwrap_err() == [5, 8, 6]));
        assert!(failures.iter().any(|(call, bad)| call.root == -1
            && bad.is_none()
            && call.execute(None, false).unwrap_err() == [5, 8, 6, 7]));
        if !leading {
            let (call, _) = failures
                .iter()
                .find(|(call, bad)| call.hole == 2 && bad.is_none())
                .unwrap();
            let trace = call.execute(None, false).unwrap_err();
            assert_eq!(trace.last(), Some(&19));
            assert_eq!(trace.iter().filter(|n| **n == 6).count(), 2);
            assert_eq!(trace.iter().filter(|n| **n == 22).count(), 1);
        }
        let mut selections = BTreeSet::new();
        let mut inactive_poison = 0;
        for open in cases::cases() {
            for call in cases::sequence(open, leading, true) {
                let trace = call.trace(false);
                selections.insert((trace.contains(&6), trace.contains(&9)));
                if call.root == 0 && call.input.rounds > 0 {
                    assert!(!trace.contains(&6));
                    assert!(trace.contains(&9));
                    inactive_poison += 1;
                }
            }
        }
        assert_eq!(selections.len(), 4);
        assert!(inactive_poison > 0);
        assert!(cases::failures(leading)
            .iter()
            .any(|(c, b)| c.root == -1 && b.is_none()));
    }
}

#[test]
fn iteration_effects_selected_children_preserve_branch_local_scope_and_parent_authority() {
    for (before, after) in [
        ("print(7);", "print(7); rounds = rounds + 1;"),
        ("print(7);", "print(7); outer = outer + 1;"),
        ("print(7);", "print(7); width = width + 1;"),
        ("print(21);", "let escaped = width; print(21);"),
        (
            "current = current + outer;",
            "current = refine(current, index, outer);",
        ),
        ("print(13);", "print(true);"),
    ] {
        let source = cases::SOURCE.replace(before, after);
        assert_ne!(source, cases::SOURCE);
        let Ok(module) = parse_nuis_module(&source) else {
            continue;
        };
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        assert!(
            !Plan::collect(&module, &layouts, &pure).contains("walk"),
            "{after}"
        );
    }
}

#[test]
fn iteration_effects_selected_children_manual_current_and_index_witnesses() {
    for leading in [false, true] {
        let skipped = cases::unselected_sequence(leading, i64::MAX, 0)[0];
        assert_eq!(skipped.next().words(), [0, 339, i64::MIN]);
        assert_eq!(
            skipped.trace(false),
            [5, 8, 9, 21, 22, 8, 9, 21, 22, 31, 35, 40]
        );
        assert!(skipped.execute_with_budget(None, false, Some(2)).is_ok());
        let call = cases::invocation(
            cases::Case {
                seed: -3,
                gate: false,
                flag: false,
                trips: 3,
                cut: 2,
                rounds: 2,
                marker: i64::MIN as u64,
            },
            None,
            leading,
            false,
        );
        let (total, current) = if leading { (510, 487) } else { (770, 747) };
        assert_eq!(call.next().words(), [1, total, i64::MIN]);
        let trace = call.trace(true);
        assert_eq!(
            &trace[trace.len() - 8..],
            &[31, current, 2, 35, total, 0, 1, 40]
        );
        assert!(call.execute_with_budget(None, false, Some(8)).is_ok());
        let failed = call.execute_with_budget(None, false, Some(5)).unwrap_err();
        assert_eq!(failed.last(), Some(&7));
        assert_eq!(failed.iter().filter(|n| **n == 22).count(), 1);
    }
}
