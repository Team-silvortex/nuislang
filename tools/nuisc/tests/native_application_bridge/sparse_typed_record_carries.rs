use super::*;

#[path = "sparse_typed_return_carries.rs"]
mod returns;

#[path = "sparse_record_fixture.rs"]
mod fixture;

#[test]
fn typed_materialized_loop_snapshots_keep_current_words_and_complete_64_word_state() {
    check_loop_snapshots(None);
}

#[test]
fn typed_materialized_loop_snapshots_keep_inner_break_and_continue_local() {
    check_loop_snapshots(Some("break"));
    check_loop_snapshots(Some("continue"));
}

fn check_loop_snapshots(exit: Option<&str>) {
    for width in [9, 64] {
        let mut source = fixture::loop_source(width);
        if let Some(exit) = exit {
            source = source.replace("i % 3", "i % 4").replace(
                "let previous = selected;",
                &format!("let previous = selected; if j == 2 {{ {exit}; }}"),
            );
        }
        let compiled = nuisc::pipeline::compile_source(&source).unwrap();
        let call = compiled
            .yir
            .nodes
            .iter()
            .filter_map(|node| {
                yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args)
                    .ok()
                    .flatten()
            })
            .find(|call| call.seeds.len() == width)
            .unwrap();
        assert_eq!(call.operands.len(), 6);
        let cases = [0_u64, 1, 2, 3, 4]
            .into_iter()
            .map(|limit| {
                let mut initial = vec![
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
                initial.resize(width, 27);
                let mut event = initial.clone();
                let mut value = -1.5_f64;
                let mut gain = 2.5_f32;
                for i in 1..=limit {
                    let inner_limit = i % if exit.is_some() { 4 } else { 3 };
                    for j in 1..=inner_limit {
                        if j == 2 {
                            match exit {
                                Some("break") => break,
                                Some("continue") => continue,
                                _ => {}
                            }
                        }
                        value += if j == 1 { 0.5 } else { 1.0 };
                        gain += if j == 1 { 0.25 } else { 0.5 };
                    }
                }
                if limit > 0 {
                    event[0] = value.to_bits();
                    event[1] = u64::from(gain.to_bits());
                    event[3] = limit % 2;
                    event[4] = 3.5_f64.to_bits();
                    event[5] = u64::from(4.5_f32.to_bits());
                    event[6] = 9;
                    event[7] = 0;
                    event[9..].fill(99);
                }
                (
                    vec![initial[0], initial[1], initial[2], limit],
                    vec![initial, event.clone(), event],
                )
            })
            .collect();
        typed_record_inputs::check_flattened(&source, cases);
    }
}

#[test]
fn typed_sparse_branch_snapshots_keep_current_words_and_complete_64_word_state() {
    check_alternating(fixture::branch_source);
}

#[test]
fn typed_materialized_join_snapshots_keep_current_words_and_complete_64_word_state() {
    check_alternating(fixture::join_source);
}

fn check_alternating(make_source: fn(usize) -> String) {
    for width in [9, 64] {
        let source = make_source(width);
        let compiled = nuisc::pipeline::compile_source(&source).unwrap();
        let call = compiled
            .yir
            .nodes
            .iter()
            .find_map(|node| {
                yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args)
                    .ok()
                    .flatten()
            })
            .unwrap();
        assert_eq!(call.seeds.len(), width);
        assert_eq!(call.operands.len(), 6);
        let cases = [0_u64, 1, 2, 3, 4]
            .into_iter()
            .map(|limit| {
                let mut initial = vec![
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
                initial.resize(width, 27);
                let mut event = initial.clone();
                if limit > 0 {
                    let odd = limit.div_ceil(2) as f64;
                    let even = (limit / 2) as f64;
                    event[0] = (-1.5_f64 + odd * 0.5 + even).to_bits();
                    event[1] =
                        u64::from((2.5_f32 + odd as f32 * 0.25 + even as f32 * 0.5).to_bits());
                    event[3] = limit % 2;
                    event[4] = 3.5_f64.to_bits();
                    event[5] = u64::from(4.5_f32.to_bits());
                    event[6] = 9;
                    event[7] = 0;
                    event[9..].fill(99);
                }
                (
                    vec![initial[0], initial[1], initial[2], limit],
                    vec![initial, event.clone(), event],
                )
            })
            .collect();
        typed_record_inputs::check_flattened(&source, cases);
    }
}

#[test]
fn typed_sparse_record_carries_preserve_full_zero_trip_state_and_raw_leaf_bits() {
    check(9);
}

#[test]
fn typed_sparse_record_carries_separate_64_seed_words_from_six_iteration_arguments() {
    check(64);
}

#[test]
fn typed_sparse_record_carries_read_current_float_words_each_trip() {
    let source = fixture::source(9)
        .replace("value: saved.left.value", "value: saved.left.value + 0.5")
        .replace("gain: saved.left.gain", "gain: saved.left.gain + 0.25");
    let cases = [0_u64, 1, 3]
        .into_iter()
        .map(|limit| {
            let initial = vec![
                (-1.5_f64).to_bits(),
                u64::from(2.5_f32.to_bits()),
                (-17_i64) as u64,
                0,
                1.5_f64.to_bits(),
                u64::from(2.5_f32.to_bits()),
                7,
                1,
                limit,
            ];
            let mut event = initial.clone();
            if limit > 0 {
                event[0] = (-1.5_f64 + limit as f64 * 0.5).to_bits();
                event[1] = u64::from((2.5_f32 + limit as f32 * 0.25).to_bits());
                event[3] = limit % 2;
                event[4] = 3.5_f64.to_bits();
                event[5] = u64::from(4.5_f32.to_bits());
                event[6] = 9;
                event[7] = 0;
            }
            (
                vec![initial[0], initial[1], initial[2], limit],
                vec![initial, event.clone(), event],
            )
        })
        .collect();
    typed_record_inputs::check_flattened(&source, cases);
}

#[test]
fn typed_sparse_record_carries_can_omit_every_input_word_without_losing_zero_trip_state() {
    let source = fixture::source(9)
        .replace("saved.left.value", "1.5")
        .replace("saved.left.gain", "2.5")
        .replace("saved.left.tag", "i32_from_i64(7)")
        .replace("saved.left.enabled", "false");
    let compiled = nuisc::pipeline::compile_source(&source).unwrap();
    let call = compiled
        .yir
        .nodes
        .iter()
        .find_map(|node| {
            yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args)
                .ok()
                .flatten()
        })
        .unwrap();
    assert_eq!(call.seeds.len(), 9);
    assert_eq!(call.operands.len(), 2);
    assert!(!call
        .operands
        .iter()
        .any(|arg| arg.starts_with("$owned_struct_carry:")));
    let cases = [0_u64, 1, 3]
        .into_iter()
        .map(|limit| {
            let initial = vec![
                0xfff8_0000_0000_1234,
                0x8000_0000,
                i32::MIN as i64 as u64,
                0,
                1.5_f64.to_bits(),
                u64::from(2.5_f32.to_bits()),
                7,
                1,
                limit,
            ];
            let event = if limit == 0 {
                initial.clone()
            } else {
                vec![
                    1.5_f64.to_bits(),
                    u64::from(2.5_f32.to_bits()),
                    7,
                    1,
                    3.5_f64.to_bits(),
                    u64::from(4.5_f32.to_bits()),
                    9,
                    0,
                    limit,
                ]
            };
            (
                vec![initial[0], initial[1], initial[2], limit],
                vec![initial, event.clone(), event],
            )
        })
        .collect();
    typed_record_inputs::check_flattened(&source, cases);
}

fn check(width: usize) {
    let source = fixture::source(width);
    let compiled = nuisc::pipeline::compile_source(&source).unwrap();
    let call = compiled
        .yir
        .nodes
        .iter()
        .find_map(|node| {
            yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args)
                .ok()
                .flatten()
        })
        .unwrap();
    assert_eq!(call.seeds.len(), width);
    assert_eq!(call.operands.len(), 6);
    let mut cases = Vec::new();
    for (value, gain, tag) in [
        (0x8000_0000_0000_0000, 0x8000_0000, i32::MIN),
        (0xfff8_0000_0000_1234, 0x7fc0_5678, i32::MAX),
        (1.5_f64.to_bits(), u64::from(2.5_f32.to_bits()), -17),
    ] {
        for limit in [0_u64, 1, 2, 3] {
            let mut initial = vec![
                value,
                gain,
                tag as i64 as u64,
                0,
                1.5_f64.to_bits(),
                u64::from(2.5_f32.to_bits()),
                7,
                1,
                limit,
            ];
            initial.resize(width, 27);
            let mut event = initial.clone();
            if limit > 0 {
                event[3] = limit % 2;
                event[4] = 3.5_f64.to_bits();
                event[5] = u64::from(4.5_f32.to_bits());
                event[6] = 9;
                event[7] = 0;
                event[9..].fill(99);
            }
            cases.push((
                vec![value, gain, tag as i64 as u64, limit],
                vec![initial, event.clone(), event],
            ));
        }
    }
    typed_record_inputs::check_flattened(&source, cases);
}
