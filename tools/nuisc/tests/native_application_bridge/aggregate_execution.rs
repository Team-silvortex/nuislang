use super::*;

pub(super) fn source(slots: usize, comparison: &str) -> String {
    let params = (0..slots)
        .rev()
        .map(|i| format!("c{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let args = (0..slots)
        .rev()
        .map(|i| format!("c{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let second = (0..slots)
        .rev()
        .map(|i| format!("first.carry{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let returned = (0..slots)
        .map(|i| format!("carry{i}: second.carry{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    multi_execution::source(slots, comparison)
        .replace("fn advance(", "@noinline fn branch(")
        .replace("fn main()", &format!("@noinline
          fn advance({params}, index: i64, tag: i32, flag: bool, gain: f32, scale: f64) -> Carries {{
            let first: Carries = branch({args}, index, tag, flag, gain, scale);
            let second: Carries = branch({second}, index, tag, flag, gain, scale);
            return Carries {{ {returned} }};
          }} fn main()"))
}

#[test]
fn ordinary_flat_returns_preserve_exact_captures_order_and_nested_drop_balance() {
    for slots in [2, 3, 7] {
        for (comparison, ranges) in [
            (
                "<",
                [(0_i64, 4_i64, 1_i64), (5, 2, 0), (-5, 1, 2), (3, 4, 1)],
            ),
            (">", [(4, 0, -1), (2, 5, 0), (5, -1, -2), (4, 3, -1)]),
        ] {
            let source = source(slots, comparison);
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
                        let mut carry = (0..slots).map(|i| i64::MAX - i as i64).collect::<Vec<_>>();
                        let captures = [-17_i64 as u64, u64::from(flag), u64::from(gain), scale];
                        let mut args = vec![initial as u64, limit as u64, stride as u64];
                        args.extend(carry.iter().map(|v| *v as u64));
                        args.extend(captures);
                        cases.push(args);
                        let mut index = initial;
                        while if comparison == "<" {
                            index < limit
                        } else {
                            index > limit
                        } {
                            for _ in 0..2 {
                                expected.push(0); // Previous nested return is already released.
                                expected.extend(carry.iter().rev().map(|v| *v as u64));
                                expected.push(index as u64);
                                expected.extend(captures);
                                if flag {
                                    let mut delta = index;
                                    for value in &mut carry {
                                        *value = value.wrapping_add(delta);
                                        delta = *value;
                                    }
                                }
                                allocations += 1;
                            }
                            allocations += 1; // Scoped iteration return.
                            index += stride;
                        }
                        allocations += 1; // Callback State.
                        expected.push(0);
                        expected.extend(carry.iter().map(|v| *v as u64));
                        expected.extend([allocations, allocations]);
                    }
                }
            }
            let run = multi_execution::execute_observed(
                &source,
                slots,
                &cases,
                |module| {
                    assert_eq!(
                        module
                            .nodes
                            .iter()
                            .filter(|n| n.op.instruction == "call_owned_struct"
                                && n.op.args[0] == "branch")
                            .count(),
                        2
                    );
                    module.nodes.reverse();
                    module.functions.reverse();
                    for function in &mut module.functions {
                        function.body_nodes.reverse();
                    }
                },
                Some("branch"),
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
            for (word, (actual, expected)) in actual.iter().zip(&expected).enumerate() {
                assert_eq!(
                    actual, expected,
                    "slots={slots} comparison={comparison} word={word}"
                );
            }
        }
    }
}
