use super::*;
use crate::frontend::parse_nuis_module;
#[allow(dead_code)]
#[path = "../../../tests/native_application_bridge/state_loop_parents_cases.rs"]
mod cases;

#[test]
fn iteration_effects_parents_admit_scoped_child_and_parent_exits_without_pure_leakage() {
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
        assert_eq!(outlined.break_controls.len(), 2);
        let mut names = BTreeSet::new();
        for function in &module.functions {
            branches::collect_bindings(&function.body, &mut names);
        }
        assert_eq!(
            names
                .iter()
                .filter(|n| n.starts_with("__nuis_advanced_index_"))
                .count(),
            1 + usize::from(variant.leading)
        );
        assert!(
            !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("walk")
        );
    }
}

#[test]
fn iteration_effects_parents_oracle_retains_child_work_and_distinguishes_parent_exits() {
    assert_eq!(cases::cases().len(), 100);
    for leading in [false, true] {
        let mut exits = BTreeSet::new();
        for open in cases::cases() {
            for poisoned in [false, true] {
                for call in cases::sequence(open, leading, poisoned) {
                    let trace = call.trace(false);
                    exits.insert((trace.contains(&25), trace.contains(&26)));
                }
            }
        }
        assert!(exits.contains(&(true, false)));
        assert!(exits.contains(&(false, true)));
        assert!(exits.contains(&(false, false)));
        let failures = cases::failures(leading);
        assert_eq!(failures.len(), if leading { 11 } else { 15 });
        assert_eq!(
            failures
                .iter()
                .filter(|(_, slot)| slot.is_none() || *slot == Some(6))
                .count(),
            if leading { 5 } else { 9 }
        );
        assert!(failures
            .iter()
            .all(|(c, bad)| c.execute(*bad, false).is_err()));
    }
}

#[test]
fn iteration_effects_parents_reject_terminal_suffixes_and_child_authority_leaks() {
    for (before, after) in [
        (
            "if gate { print(25); break; }",
            "if gate { break; print(25); }",
        ),
        (
            "print(26);\n                continue;",
            "continue; print(26);",
        ),
        ("print(26);", "rounds = rounds + 1; print(26);"),
        ("print(26);", "outer = outer + 1; print(26);"),
        ("print(26);", "print(current);"),
        ("print(23);", "let escaped = width; print(23);"),
    ] {
        assert!(cases::SOURCE.contains(before));
        let source = cases::SOURCE.replace(before, after);
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
fn iteration_effects_parents_manual_witnesses_observe_committed_child_and_recovered_parent() {
    for leading in [false, true] {
        for (breaking, total, current, outer, limit) in [
            (
                true,
                if leading { 47 } else { 69 },
                if leading { 34 } else { 56 },
                1,
                5,
            ),
            (false, 215, 182, 3, 9),
        ] {
            let call = cases::Invocation {
                input: cases::Case {
                    seed: if breaking { 8 } else { -3 },
                    gate: breaking,
                    flag: false,
                    trips: 3,
                    cut: if breaking {
                        2
                    } else if leading {
                        3
                    } else {
                        1
                    },
                    rounds: if breaking { 2 } else { 3 },
                    marker: i64::MIN as u64,
                },
                previous: None,
                leading,
                root: 1,
                hole: -1,
                suffix: 1,
            };
            assert_eq!(call.next().words(), [0, total, i64::MIN]);
            let trace = call.trace(true);
            assert_eq!(
                &trace[trace.len() - 8..],
                &[31, current, outer, 35, total, 0, 1, 40]
            );
            let effects = call.trace(false);
            assert!(!effects.contains(&21));
            assert!(!effects.contains(&22));
            assert_eq!(effects.iter().filter(|n| **n == 23).count(), outer as usize);
            assert_eq!(
                effects
                    .iter()
                    .filter(|n| **n == if breaking { 25 } else { 26 })
                    .count(),
                outer as usize
            );
            assert!(call.execute_with_budget(None, false, Some(limit)).is_ok());
            let failed = call
                .execute_with_budget(None, false, Some(if breaking { 4 } else { 6 }))
                .unwrap_err();
            assert_eq!(failed.last(), Some(&7));
            if !breaking {
                assert_eq!(failed.iter().filter(|n| **n == 26).count(), 2);
                assert_eq!(failed.iter().filter(|n| **n == 8).count(), 3);
            }
        }
        for breaking in [false, true] {
            assert_eq!(cases::parent_sequence(leading, breaking).len(), 4);
        }
    }
}
