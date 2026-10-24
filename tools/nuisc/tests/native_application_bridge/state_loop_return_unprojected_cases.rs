use super::State;

fn replace(source: String, old: &str, new: &str, count: usize) -> String {
    assert_eq!(source.matches(old).count(), count, "{old}");
    source.replace(old, new)
}

pub(super) fn source(mut source: String) -> String {
    let old = include_str!("state_loop_return_projected_helpers.ns");
    let arguments = old.split_once("@noinline fn project_record").unwrap().0;
    source = replace(
        source,
        old,
        &format!(
            "{arguments}{}",
            include_str!("state_loop_return_unprojected_helpers.ns")
        ),
        1,
    );
    for (record, argument, delayed, before, later) in
        [("recent", 20, 24, 28, 32), ("frozen", 22, 26, 30, 34)]
    {
        for (old_mode, new_mode) in [(argument, before), (delayed, later)] {
            let old = format!("(mode >= {old_mode} && mode <= {});", old_mode + 1);
            source = replace(
                source,
                &old,
                &format!(
                    "(mode >= {old_mode} && mode <= {}) || (mode >= {new_mode} && mode <= {});",
                    old_mode + 1,
                    new_mode + 1
                ),
                1,
            );
        }
        let old = format!("(mode - {}) * (mode - {})).gain", argument + 1, delayed + 1);
        source = replace(source, &old, &format!("(mode - {}) * (mode - {}), (mode - {before}) * (mode - {later}), (mode - {}) * (mode - {})).gain", argument + 1, delayed + 1, before + 1, later + 1), 1);
        let old = format!("project_record({record}, project_marker_arg(1), 1).marker");
        source = replace(
            source,
            &old,
            &format!("project_record({record}, project_marker_arg(1), 1, 1, 1).marker"),
            1,
        );
    }
    for (argument, field) in [
        ("project_result_arg(2)", "result"),
        ("project_total_arg(4)", "total"),
    ] {
        let old = format!("project_record(carry, {argument}, 1).{field}");
        source = replace(
            source,
            &old,
            &format!("project_record(carry, {argument}, 1, 1, 1).{field}"),
            2,
        );
    }
    source
}

pub(super) fn returning(mode: i64, child: usize, outer: i64) -> bool {
    let before = if child == 0 { 28 } else { 30 };
    (before..=before + 1).contains(&mode)
        || (outer == 2 && (before + 4..=before + 5).contains(&mode))
}

pub(super) fn payload(
    live: State,
    older: State,
    newer: State,
    mode: i64,
    child: usize,
    trace: &mut Vec<i64>,
) -> Option<State> {
    let before = if child == 0 { 28 } else { 30 };
    let marker_fault = mode == before || mode == before + 4;
    let total_fault = mode == before + 1 || mode == before + 5;
    // Reuse the independent value model, then model complete callee-field work.
    let mut projected = vec![];
    let result = super::projected::payload(live, older, newer, mode, child, &mut projected);
    let mut calls = 0;
    for effect in projected {
        trace.push(effect);
        if effect == 987 {
            calls += 1;
            trace.push(991);
            if calls == 3 && marker_fault {
                return None;
            }
            trace.extend([992, 993, 994, 995]);
            if calls == 3 && total_fault {
                return None;
            }
            trace.push(996);
        }
    }
    result
}
