use super::*;

fn source(slots: usize, compare: &str, signal: &str) -> String {
    let last = slots - 1;
    multi_execution::source(slots, compare)
        .replace(
            &format!("let v{last}: i64 = sum(c{last}, v{});", last - 1),
            &format!("let v{last}: i64 = signal(index);"),
        )
        .replace(
            "fn main()",
            &format!("fn signal(index: i64) -> i64 {{ {signal} }} fn main()"),
        )
}

fn enable_break(module: &mut YirModule) {
    let node = module
        .nodes
        .iter_mut()
        .find(|n| n.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carries"))
        .unwrap();
    // Exercise the serialized YIR boundary independently of source normalization.
    node.op.args[6] = "scoped_call_i64_carries_break".to_owned();
}

fn assert_trap(run: &std::process::Output) {
    assert!(!run.status.success(), "invalid native control must trap");
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(run.status.signal().is_some(), "not a callback error status");
    }
}

#[test]
fn guarded_break_native_calls_commit_carries_and_release_aggregates_before_exit() {
    for slots in [2, 3, 7] {
        for (compare, ranges) in [
            (
                "<",
                [
                    (2_i64, 6_i64, 1_i64),
                    (0, 5, 1),
                    (0, 3, 1),
                    (3, 6, 1),
                    (5, 2, 0),
                ],
            ),
            (
                ">",
                [(2, -2, -1), (4, 0, -1), (4, 1, -1), (5, 2, -1), (2, 5, 0)],
            ),
        ] {
            let mut cases = Vec::new();
            let mut expected = Vec::new();
            let mut allocations = 0_u64;
            for (gain, scale) in [
                (1.5_f32.to_bits(), (-2.25_f64).to_bits()),
                (0x8000_0000, 0x8000_0000_0000_0000),
                (0x7fc0_1234, 0x7ff8_0000_0000_4321),
            ] {
                for flag in [false, true] {
                    for (initial, limit, stride) in ranges {
                        let mut carry = (0..slots)
                            .map(|i| {
                                if i + 1 == slots {
                                    0
                                } else {
                                    i64::MAX - i as i64
                                }
                            })
                            .collect::<Vec<_>>();
                        let captures = [-17_i64 as u64, u64::from(flag), u64::from(gain), scale];
                        let mut case = vec![initial as u64, limit as u64, stride as u64];
                        case.extend(carry.iter().map(|v| *v as u64));
                        case.extend(captures);
                        cases.push(case);
                        let mut index = initial;
                        while if compare == "<" {
                            index < limit
                        } else {
                            index > limit
                        } {
                            expected.push(0); // No previous returned aggregate remains live.
                            expected.extend(carry.iter().rev().map(|v| *v as u64));
                            expected.push(index as u64);
                            expected.extend(captures);
                            if flag {
                                for i in 0..slots - 1 {
                                    carry[i] = carry[i].wrapping_add(if i == 0 {
                                        index
                                    } else {
                                        carry[i - 1]
                                    });
                                }
                                carry[slots - 1] = i64::from(index == 2);
                            }
                            allocations += 1;
                            if carry[slots - 1] == 1 {
                                break;
                            }
                            index += stride;
                        }
                        allocations += 1; // Callback State is unpacked and released as well.
                        expected.push(0); // Native callback status.
                        expected.extend(carry.iter().map(|v| *v as u64));
                        expected.extend([allocations, allocations]);
                    }
                }
            }
            let run = multi_execution::execute_with(
                &source(slots, compare, "if index == 2 { return 1; } return 0;"),
                slots,
                &cases,
                |module| {
                    enable_break(module);
                    module.nodes.reverse();
                    module.functions.reverse();
                    for function in &mut module.functions {
                        function.body_nodes.reverse();
                    }
                },
            );
            assert!(
                run.status.success(),
                "{}",
                String::from_utf8_lossy(&run.stderr)
            );
            let actual = String::from_utf8(run.stdout)
                .unwrap()
                .lines()
                .map(|line| line.parse::<i64>().unwrap() as u64)
                .collect::<Vec<_>>();
            assert_eq!(actual.len(), expected.len());
            for (index, (actual, expected)) in actual.iter().zip(&expected).enumerate() {
                assert_eq!(
                    actual, expected,
                    "slots={slots} comparison={compare} word={index}"
                );
            }
        }
    }
}

#[test]
fn guarded_break_rejects_nonzero_control_seeds_even_on_zero_trips() {
    for seed in [1_i64, 2, -1] {
        for (initial, limit) in [(0_i64, 4_i64), (4, 0)] {
            let args = vec![initial as u64, limit as u64, 1, 10, seed as u64, 0, 1, 0, 0];
            let run = multi_execution::execute_with(
                &source(2, "<", "return 1;"),
                2,
                &[args],
                enable_break,
            );
            assert_trap(&run);
            assert!(
                run.stdout.is_empty(),
                "seed validation must precede the helper"
            );
        }
    }
}

#[test]
fn guarded_break_rejects_invalid_returned_controls_without_exporting_state() {
    for control in [2, -1] {
        let args = vec![0, 4, 1, 10, 0, 0, 1, 0, 0];
        let source = source(2, "<", &format!("return {control};"));
        let run = multi_execution::execute_with(&source, 2, &[args], enable_break);
        assert_trap(&run);
        let output = String::from_utf8(run.stdout).unwrap();
        assert_eq!(
            output.lines().count(),
            8,
            "one helper entry, no callback return: {output}"
        );
        let project = Project::with_source(&source);
        let mut module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
        enable_break(&mut module);
        let llvm = emit_registered(&module, "counter").unwrap().llvm_ir;
        let caller = &module.application_sessions[0].open;
        let start = llvm
            .find(&format!("define i64 @nuis_fn_{caller}("))
            .unwrap();
        let end = start + llvm[start..].find("\n}").unwrap();
        let llvm = &llvm[start..end];
        let drop = llvm
            .find("call void @nuis_scheduler_owned_aggregate_drop_v1(")
            .unwrap();
        let control_check = drop + llvm[drop..].find(" = icmp ule i64 ").unwrap();
        let accepted = control_check
            + llvm[control_check..]
                .find("loop_break_control_valid")
                .unwrap();
        assert!(
            !llvm[drop..accepted].contains("store i64"),
            "validate before committing any carry"
        );
    }
}

#[test]
fn guarded_break_does_not_bypass_the_full_induction_preflight() {
    for (initial, limit, stride) in [
        (2_i64, 7_i64, 0_i64),
        (2, 1_000_000, 1),
        (i64::MAX - 1, i64::MAX, 2),
    ] {
        let args = vec![
            initial as u64,
            limit as u64,
            stride as u64,
            10,
            0,
            0,
            1,
            0,
            0,
        ];
        let run =
            multi_execution::execute_with(&source(2, "<", "return 1;"), 2, &[args], enable_break);
        assert_trap(&run);
        assert!(
            run.stdout.is_empty(),
            "preflight must precede the first helper"
        );
    }
}
