use super::*;
use crate::frontend::parse_nuis_module;
#[allow(dead_code)]
#[path = "../../../tests/native_application_bridge/state_loop_exits_cases.rs"]
mod cases;

#[test]
fn iteration_effects_exits_oracle_distinguishes_step_positions_and_control_paths() {
    assert_eq!(cases::cases().len(), 96);
    for (leading, count, cli_count, last) in [(false, 11, 5, 2), (true, 10, 4, 3)] {
        let failures = cases::failures(leading);
        assert_eq!(failures.len(), count);
        assert_eq!(
            failures
                .iter()
                .filter(|(_, bad)| bad.is_none() || *bad == Some(6))
                .count(),
            cli_count
        );
        assert!(failures.iter().any(|(call, bad)| bad.is_none()
            && call.hole == last
            && call.execute(None, false).unwrap_err().contains(&13)));
    }
    for (leading, gate, total, result, current, index) in [
        (false, false, 81, false, 48, 3),
        (true, false, 86, true, 53, 3),
        (false, true, 35, false, 22, 1),
        (true, true, 23, false, 10, 1),
    ] {
        let call = cases::invocation(
            cases::Case {
                seed: 8,
                gate,
                flag: false,
                trips: 3,
                cut: 1,
                marker: 0x8000000000000000,
            },
            None,
            leading,
            false,
        );
        let state = call.next();
        assert_eq!(state.words(), [i64::from(result), total, i64::MIN]);
        let trace = call.trace(true);
        assert_eq!(
            &trace[trace.len() - 8..],
            &[31, current, index, 35, total, 0, 1, 40]
        );
        assert!(cases::Invocation { hole: 1, ..call }
            .execute(None, false)
            .is_ok());
    }
}

#[test]
fn iteration_effects_exits_keep_pure_isolation_and_observed_index_recovery() {
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
fn iteration_effects_exits_reject_unstepped_continue_and_effects_after_terminal_control() {
    for (from, to) in [
        ("index = index + 1;\n                continue;", "continue;"),
        (
            "index = index + 1;\n                continue;",
            "index = index + 2; continue;",
        ),
        (
            "index = index + 1;\n                continue;",
            "index = index + 1; print(99); continue;",
        ),
        ("print(12); break;", "break; print(12);"),
        ("print(13);", "print(index);"),
        ("print(12); break;", "trips = trips + 1; print(12); break;"),
    ] {
        assert!(cases::SOURCE.contains(from));
        let text = cases::SOURCE.replace(from, to);
        let Ok(module) = parse_nuis_module(&text) else {
            continue;
        };
        let layouts = control_values::layouts(&module);
        let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
        assert!(
            !Plan::collect(&module, &layouts, &pure).contains("walk"),
            "{to}"
        );
    }
}
