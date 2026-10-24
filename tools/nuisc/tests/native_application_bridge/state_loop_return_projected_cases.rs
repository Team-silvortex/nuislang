use super::State;

fn replace(source: String, old: &str, new: &str) -> String {
    assert_eq!(source.matches(old).count(), 1, "{old}");
    source.replace(old, new)
}

pub(super) fn source(mut source: String) -> String {
    source = replace(
        source,
        "    @noinline fn work(",
        &format!(
            "{}\n    @noinline fn work(",
            include_str!("state_loop_return_projected_helpers.ns")
        ),
    );
    for (record, other, early, late, old_fault, old_delayed, argument, delayed) in [
        ("recent", "frozen", 1, 10, 12, 18, 20, 24),
        ("frozen", "recent", 6, 11, 15, 19, 22, 26),
    ] {
        let old = format!(
            "let payload_{record}_early = mode == {early} || (mode >= {old_fault} && mode <= {});",
            old_fault + 2
        );
        source = replace(source, &old, &format!("let payload_{record}_early = mode == {early} || (mode >= {old_fault} && mode <= {}) || (mode >= {argument} && mode <= {});", old_fault + 2, argument + 1));
        let old = format!("let payload_{record}_late = mode == {late} || mode == {old_delayed};");
        source = replace(source, &old, &format!("let payload_{record}_late = mode == {late} || mode == {old_delayed} || (mode >= {delayed} && mode <= {});", delayed + 1));
        let old = format!("return State {{ marker: field_marker({other}.marker), result: field_result(carry.result != true), gain: field_gain({record}.gain), total: field_total(carry.total +");
        source = replace(source, &old, &format!(
            "return State {{
                marker: field_marker(project_record({other}, project_marker_arg(1), 1).marker),
                result: field_result(project_record(carry, project_result_arg(2), 1).result != true),
                gain: field_gain(project_record({record}, project_gain_arg(3 + 0 / ((mode - {argument}) * (mode - {delayed}))), (mode - {}) * (mode - {})).gain),
                total: field_total(project_record(carry, project_total_arg(4), 1).total +", argument + 1, delayed + 1
        ));
    }
    source
}

pub(super) fn returning(mode: i64, child: usize, outer: i64) -> bool {
    let argument = if child == 0 { 20 } else { 22 };
    (argument..=argument + 1).contains(&mode)
        || (outer == 2 && (argument + 4..=argument + 5).contains(&mode))
}

pub(super) fn payload(
    live: State,
    older: State,
    newer: State,
    mode: i64,
    child: usize,
    trace: &mut Vec<i64>,
) -> Option<State> {
    // Model source occurrences independently of the generated Nuis expression.
    trace.extend([985, 986, 987, 980, 988, 986, 987, 981]);
    let argument = if child == 0 { 20 } else { 22 };
    if mode == argument || mode == argument + 4 {
        return None;
    }
    trace.extend([989, 986]);
    if mode == argument + 1 || mode == argument + 5 {
        return None;
    }
    trace.extend([987, 982, 990, 986, 987, 984]);
    let (gain, marker) = if child == 0 {
        (newer.gain, older.marker)
    } else {
        (older.gain, newer.marker)
    };
    super::payload(
        State {
            result: !live.result,
            total: live.total
                + 4
                + older.total * 100
                + newer.total * 10
                + if live.result { 1000 } else { 2000 },
            gain: gain ^ (1 << 31),
            marker: marker ^ (1 << 63),
        },
        mode,
        child,
        trace,
    )
}
