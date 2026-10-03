use super::*;

#[path = "sparse_typed_return_probe.rs"]
mod probe;

#[path = "sparse_typed_return_child_exits.rs"]
mod child_exits;

#[path = "sparse_typed_return_literals.rs"]
mod literals;

#[test]
fn typed_sparse_nested_return_capture_geometry() {
    let mut actual = Vec::new();
    for width in [9, 57, 58, 59, 60, 61, 62, 63, 64] {
        let compiled = nuisc::pipeline::compile_source(&fixture::return_source(width)).unwrap();
        let whole = compiled
            .yir
            .nodes
            .iter()
            .any(|node| node.op.instruction == "param_value_struct");
        actual.push((width, whole, carried_widths(&compiled.yir)));
    }
    assert_eq!(
        actual,
        [
            (9, false, vec![5, 8]),
            (57, false, vec![53, 56]),
            (58, false, vec![54, 57]),
            (59, false, vec![55, 58]),
            (60, false, vec![56, 59]),
            (61, true, vec![57, 60]),
            (62, true, vec![58, 61]),
            (63, true, vec![59, 62]),
            (64, true, vec![60, 63]),
        ]
    );
}

#[test]
fn typed_sparse_nested_returns_share_work_budgets_without_partial_publication() {
    for width in [9, 63, 64] {
        // Both 3 and 4 trips return in the second outer iteration. Reservations
        // still include every planned outer trip, not just the executed bodies.
        let cases = [
            (0, 0, 1, Some(0), [0, 0]),
            (1, 2, 10, Some(0), [0, 0]),
            (3, 6, 24, Some(0), [0, 0]),
            (4, 7, 24, Some(0), [0, 0]),
            (3, 5, 64, None, [1, 53]),
            (4, 6, 64, None, [1, 53]),
            (3, 6, 23, None, [0, 0]),
            (3, 0, 64, None, [0, 63]),
            (0, 0, 0, None, [0, 0]),
        ]
        .map(|(limit, loops, entries, status, remaining)| probe::Case {
            limit,
            loops,
            entries,
            status,
            remaining,
            corrupt: None,
            alias: false,
            semantic_trap: false,
        });
        probe::check(&fixture::return_source(width), width, &with_aliases(&cases));
    }
}

#[test]
fn typed_sparse_nested_returns_publish_current_snapshots_and_skip_dead_suffixes() {
    for width in [9, 30, 31, 57, 58, 59, 60, 61, 62, 63, 64] {
        let source = fixture::return_source(width);
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        assert_eq!(carried_widths(&compiled.yir), [width - 4, width - 1]);
        let cases = (0..=4)
            .map(|limit| {
                let initial = initial(width, limit);
                let event = event(&initial, limit);
                (
                    vec![initial[0], initial[1], initial[2], limit],
                    vec![initial, event.clone(), event],
                )
            })
            .collect();
        if width >= 61 {
            typed_record_inputs::check_compact(&source, cases, width - 7);
        } else {
            typed_record_inputs::check_flattened(&source, cases);
        }
    }
}

#[test]
fn typed_sparse_nested_returns_reject_expanded_private_state_before_native_emission() {
    for (width, rejected) in [(64, 65)] {
        // Mutating both tag and count removes the remaining invariants. Keep
        // the return/control word and reject an actually incompressible state.
        let source = fixture::return_source(width)
            .replace(
                "tag: selected.tag, enabled: !enabled",
                "tag: i32_from_i64(17), enabled: !enabled",
            )
            .replace("count: limit", "count: limit + 1");
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        assert_eq!(carried_widths(&compiled.yir), [width - 3, width + 1]);
        let error = emit_registered(&compiled.yir, "counter").unwrap_err();
        assert!(
            error.contains(&format!(
                "{} carried words exceed the 64-word native limit",
                rejected
            )),
            "{error}"
        );
        assert!(
            error.contains("including private return/control state"),
            "{error}"
        );
    }
}

#[test]
fn typed_sparse_nested_returns_execute_changed_outer_tags_with_preheader_snapshots() {
    for width in [9, 63, 64] {
        let source = fixture::return_source(width).replace(
            "tag: selected.tag, enabled: !enabled",
            "tag: i32_from_i64(17), enabled: !enabled",
        );
        let compiled = nuisc::pipeline::compile_source(&source).unwrap();
        assert_eq!(carried_widths(&compiled.yir), [width - 4, width]);
        let cases = [0, 1, 3, 4]
            .into_iter()
            .map(|limit| {
                let initial = initial(width, limit);
                let mut event = event(&initial, limit);
                if limit > 0 {
                    event[2] = 17;
                }
                (
                    vec![initial[0], initial[1], initial[2], limit],
                    vec![initial, event.clone(), event],
                )
            })
            .collect();
        if width == 9 {
            typed_record_inputs::check_flattened(&source, cases);
        } else {
            typed_record_inputs::check_compact(&source, cases, width - 7);
        }
    }
}

#[test]
fn typed_sparse_nested_returns_execute_joined_preheader_snapshots_at_full_width() {
    for width in [9, 63, 64] {
        let source = fixture::joined_return_source(width);
        let compiled = nuisc::pipeline::compile_source(&source).unwrap();
        assert_eq!(carried_widths(&compiled.yir), [width - 4, width]);
        let cases = [0, 1, 3, 4]
            .into_iter()
            .map(|limit| {
                let initial = initial(width, limit);
                let mut event = event(&initial, limit);
                if limit > 0 {
                    event[2] = 17;
                }
                (
                    vec![initial[0], initial[1], initial[2], limit],
                    vec![initial, event.clone(), event],
                )
            })
            .collect();
        if width == 9 {
            typed_record_inputs::check_flattened(&source, cases);
        } else {
            typed_record_inputs::check_compact(&source, cases, width - 7);
        }
    }
}

#[test]
fn typed_sparse_nested_returns_preserve_parent_entry_fields_and_reject_first_trip_aliases() {
    for width in [9, 63, 64] {
        for changed_tag in [false, true] {
            let mut source = fixture::parent_return_source(width);
            if changed_tag {
                source = source.replace(
                    "tag: selected.tag, enabled: !enabled",
                    "tag: i32_from_i64(17), enabled: !enabled",
                );
            }
            let compiled = nuisc::pipeline::compile_source(&source).unwrap();
            assert_eq!(
                carried_widths(&compiled.yir),
                [
                    width - if changed_tag { 2 } else { 4 },
                    width - usize::from(!changed_tag)
                ]
            );
            let cases = [0, 1, 3, 4]
                .into_iter()
                .map(|limit| {
                    let initial = initial(width, limit);
                    let mut event = event(&initial, limit);
                    if changed_tag && limit == 1 {
                        event[2] = 17;
                    }
                    (
                        vec![initial[0], initial[1], initial[2], limit],
                        vec![initial, event.clone(), event],
                    )
                })
                .collect();
            if width == 9 {
                typed_record_inputs::check_flattened(&source, cases);
            } else {
                typed_record_inputs::check_compact(
                    &source,
                    cases,
                    width - if changed_tag { 6 } else { 7 },
                );
            }
        }
    }
}

#[test]
fn typed_sparse_nested_returns_keep_post_loop_fields_and_current_snapshot_bits() {
    for width in [9, 64] {
        for exit in ["", "break;", "continue;"] {
            let source = fixture::post_loop_return_source(width, exit);
            let compiled = nuisc::pipeline::compile_source(&source).unwrap();
            let widths = carried_widths(&compiled.yir);
            assert_eq!(
                widths,
                [if exit == "break;" { 2 } else { 1 }, width - 4, width - 1],
                "{exit}"
            );
            let cases = [0, 1, 3, 4]
                .into_iter()
                .map(|limit| {
                    let initial = initial(width, limit);
                    let event = event(&initial, limit);
                    (
                        vec![initial[0], initial[1], initial[2], limit],
                        vec![initial, event.clone(), event],
                    )
                })
                .collect();
            if width == 9 {
                typed_record_inputs::check_flattened(&source, cases);
            } else {
                typed_record_inputs::check_compact(&source, cases, width - 7);
            }
        }
    }
}

#[test]
fn typed_sparse_post_loop_unknown_fields_keep_the_native_carry_limit() {
    let source = fixture::post_loop_return_source(64, "")
        .replace("left: carry.left,", "left: Leaf { value: carry.left.value, gain: carry.left.gain, tag: i32_from_i64(17), enabled: carry.left.enabled },")
        .replace("count: carry.count", "count: carry.count + 1");
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    assert!(carried_widths(&compiled.yir)
        .iter()
        .any(|width| *width > 64));
    let error = emit_registered(&compiled.yir, "counter").unwrap_err();
    assert!(
        error.contains("carried words exceed the 64-word native limit"),
        "{error}"
    );
}

#[test]
fn typed_sparse_nested_return_joins_reject_one_changed_arm_before_native_emission() {
    let source = fixture::joined_return_source(64)
        .replace("let bounds = carry;", "let bounds = relay(carry);");
    let compiled = nuisc::pipeline::compile_source(&source).unwrap();
    assert_eq!(carried_widths(&compiled.yir), [61, 65]);
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let error = emit_registered(&compiled.yir, "counter").unwrap_err();
    assert!(
        error.contains("65 carried words exceed the 64-word native limit"),
        "{error}"
    );
}

#[test]
fn typed_sparse_nested_returns_preserve_invariant_negative_zero_and_nan_payloads() {
    for width in [9, 63, 64] {
        let source = fixture::return_source(width)
            .replace(
                "left: Leaf { value: value, gain: gain, tag: tag, enabled: false }",
                "left: Leaf { value: 0.0, gain: 1.0, tag: tag, enabled: false }",
            )
            .replace(
                "right: Leaf { value: 1.5, gain: 2.5, tag: i32_from_i64(7), enabled: true }",
                "right: Leaf { value: value, gain: gain, tag: tag, enabled: true }",
            )
            .replace(
                "right: Leaf { value: 3.5, gain: 4.5, tag: i32_from_i64(9), enabled: false }",
                "right: carry.right",
            );
        let mut cases = Vec::new();
        for (value, gain) in [
            (0x8000_0000_0000_0000, 0x8000_0000),
            (0x7ff8_0000_0000_4321, 0x7fc0_1234),
        ] {
            for limit in [0, 3] {
                let mut initial = initial(width, limit);
                initial[0] = 0_f64.to_bits();
                initial[1] = u64::from(1_f32.to_bits());
                let tag = initial[2];
                initial[4..8].copy_from_slice(&[value, gain, tag, 1]);
                let mut result = initial.clone();
                if limit > 0 {
                    result[0] = 2_f64.to_bits();
                    result[1] = u64::from(2_f32.to_bits());
                    result[3] = 1;
                    result[9..].fill(99);
                }
                cases.push((
                    vec![value, gain, initial[2], limit],
                    vec![initial, result.clone(), result],
                ));
            }
        }
        if width == 9 {
            typed_record_inputs::check_flattened(&source, cases);
        } else {
            typed_record_inputs::check_compact(&source, cases, width - 7);
        }
    }
}

fn carried_widths(yir: &yir_core::YirModule) -> Vec<usize> {
    let mut widths = yir
        .nodes
        .iter()
        .filter_map(|node| {
            yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args).unwrap()
        })
        .map(|call| call.seeds.len())
        .collect::<Vec<_>>();
    widths.sort_unstable();
    widths
}

#[test]
fn typed_sparse_nested_loops_keep_64_word_budget_failures_atomic() {
    let cases = [
        (0, 0, 1, Some(0), [0, 0]),
        (1, 2, 5, Some(0), [0, 0]),
        (3, 6, 13, Some(0), [0, 0]),
        (3, 5, 64, None, [1, 58]),
        (3, 6, 12, None, [0, 0]),
    ]
    .map(|(limit, loops, entries, status, remaining)| probe::Case {
        limit,
        loops,
        entries,
        status,
        remaining,
        corrupt: None,
        alias: false,
        semantic_trap: false,
    });
    probe::check(&fixture::loop_source(64), 64, &with_aliases(&cases));
}

#[test]
fn typed_sparse_nested_return_payload_failures_leave_the_last_state_untouched() {
    for width in [9, 63, 64] {
        let source = if width == 64 {
            fixture::return_source(width).replace(
                "right: carry.right, count: limit",
                "right: carry.right, count: 10 / (2 - j)",
            )
        } else {
            fixture::return_source(width).replace("return State { left: selected,",
                "return State { left: Leaf { value: selected.value, gain: selected.gain, tag: i32_from_i64(10 / (2 - j)), enabled: selected.enabled },")
        };
        let cases =
            [(0, [64, 63]), (1, [62, 54]), (3, [58, 43])].map(|(limit, remaining)| probe::Case {
                limit,
                loops: 64,
                entries: 64,
                status: if limit == 3 { None } else { Some(0) },
                remaining,
                corrupt: None,
                alias: false,
                semantic_trap: limit == 3,
            });
        probe::check(&source, width, &with_aliases(&cases));
    }
}

#[test]
fn typed_sparse_nested_returns_reject_unused_noncanonical_inputs_before_work() {
    for width in [9, 63, 64] {
        // These right-hand leaves are overwritten before the selected return.
        // Removing their decode must not bypass public callback validation.
        let cases = [(5, 1_u64 << 32), (6, 0x8000_0000), (7, 2)].map(|corrupt| probe::Case {
            limit: 3,
            loops: 0,
            entries: 0,
            status: Some(2),
            remaining: [-1, -1],
            corrupt: Some(corrupt),
            alias: false,
            semantic_trap: false,
        });
        probe::check(&fixture::return_source(width), width, &with_aliases(&cases));
    }
}

fn with_aliases(cases: &[probe::Case]) -> Vec<probe::Case> {
    cases
        .iter()
        .flat_map(|case| {
            [false, true].map(|alias| probe::Case {
                alias,
                ..case.clone()
            })
        })
        .collect()
}

fn initial(width: usize, limit: u64) -> Vec<u64> {
    let mut words = vec![
        (-1.5_f64).to_bits(),
        u64::from(2.5_f32.to_bits()),
        i32::MIN as i64 as u64,
        0,
        1.5_f64.to_bits(),
        u64::from(2.5_f32.to_bits()),
        7,
        1,
        limit,
    ];
    words.resize(width, 27);
    words
}

fn event(initial: &[u64], limit: u64) -> Vec<u64> {
    let mut result = initial.to_vec();
    if limit > 0 {
        let delta = if limit == 1 { 0.5 } else { 2.0 };
        result[0] = (f64::from_bits(initial[0]) + delta).to_bits();
        result[1] = u64::from((f32::from_bits(initial[1] as u32) + delta as f32 / 2.0).to_bits());
        result[3] ^= 1;
        result[4..8].copy_from_slice(&[3.5_f64.to_bits(), u64::from(4.5_f32.to_bits()), 9, 0]);
        result[9..].fill(99);
    }
    result
}
