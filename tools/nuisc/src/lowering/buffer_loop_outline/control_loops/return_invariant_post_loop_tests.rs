use super::*;
use crate::frontend::parse_nuis_module;

#[test]
fn return_invariants_post_loop_copies_exits_and_checked_work_match_independent_storage() {
    for warm_limit in [0, 1, 3] {
        for limit in [0, 1, 3] {
            for exit in ["let noop = warm;", "break;", "continue;"] {
                for changed_tag in [false, true] {
                    for trap in [false, true] {
                        // An independent invariant forces changing-tag cases to
                        // revalidate an optimized plan, not merely fall back.
                        let source = format!(
                            "mod cpu Main {{
                                struct State {{ value: i64, tag: i64, identity: i64 }}
                                fn step(seed: State, warm_limit: i64, limit: i64) -> State {{
                                    let carry = seed; let saved = seed; let warm = 0;
                                    while warm < warm_limit {{
                                        let warm = warm + 1; let carry = carry; let previous = carry;
                                        let carry = State {{ value: previous.value + 10, tag: {}, identity: previous.identity }};
                                        if warm == 2 {{ {exit} }}
                                        let checked = {};
                                        let carry = State {{ value: carry.value + 100, tag: seed.tag, identity: carry.identity }};
                                    }}
                                    let after = carry; let i = 0;
                                    while i < limit {{
                                        let i = i + 1; let carry = carry; let selected = carry;
                                        let j = 0; let child_limit = i % 3;
                                        while j < child_limit {{
                                            let j = j + 1; let selected = selected; let previous = selected;
                                            let selected = State {{ value: previous.value + 1, tag: seed.tag, identity: previous.identity }};
                                            if j == 2 {{ return State {{
                                                value: selected.value + after.value + saved.value,
                                                tag: selected.tag + after.tag + warm * 100, identity: selected.identity
                                            }}; }}
                                        }}
                                        let carry = State {{ value: selected.value + after.value, tag: selected.tag, identity: selected.identity }};
                                    }}
                                    return State {{ value: carry.value + saved.value, tag: carry.tag + after.tag + warm * 100, identity: carry.identity }};
                                }}
                                fn main() -> i64 {{
                                    let result = step(State {{ value: 10, tag: 7, identity: 9 }}, {warm_limit}, {limit});
                                    return result.value * 1000 + result.tag + result.identity;
                                }}
                            }}",
                            if changed_tag { "previous.tag + 100" } else { "previous.tag" },
                            if trap { "1 / (warm - 2)" } else { "warm" },
                        );
                        tests::promoted(&source);
                        let module = parse_nuis_module(&source).unwrap();
                        let layouts = control_values::layouts(&module);
                        let mut independent = module.clone();
                        for function in &mut independent.functions {
                            if let Some(body) = normalize(function, &layouts).unwrap() {
                                function.body = body;
                            }
                        }
                        let actual = nested_tests::execute(&module);
                        assert_eq!(actual, nested_tests::execute(&independent), "{source}");
                        let expected = (|| {
                            let mut value = 10;
                            let mut tag = 7;
                            let mut warm = 0;
                            for trip in 1..=warm_limit {
                                warm = trip;
                                value += 10;
                                if changed_tag {
                                    tag += 100;
                                }
                                if trip == 2 {
                                    if exit == "break;" {
                                        break;
                                    }
                                    if exit == "continue;" {
                                        continue;
                                    }
                                    if trap {
                                        return Err(());
                                    }
                                }
                                value += 100;
                                tag = 7;
                            }
                            let (after_value, after_tag) = (value, tag);
                            for i in 1..=limit {
                                for j in 1..=i % 3 {
                                    value += 1;
                                    tag = 7;
                                    if j == 2 {
                                        return Ok((value + after_value + 10) * 1000
                                            + tag
                                            + after_tag
                                            + warm * 100
                                            + 9);
                                    }
                                }
                                value += after_value;
                            }
                            Ok((value + 10) * 1000 + tag + after_tag + warm * 100 + 9)
                        })();
                        assert_eq!(actual, expected, "{source}");
                    }
                }
            }
        }
    }
}
