use super::*;
use crate::frontend::parse_nuis_module;

#[test]
fn return_invariant_literals_match_independent_returns_exits_and_selected_failures() {
    for limit in [0, 1, 3] {
        for exit in ["let noop = j;", "break;", "continue;"] {
            for changed in [false, true] {
                for branch in [false, true] {
                    for trap in [false, true] {
                        let entry = "let carry = State { value: seed.value, tag: 7,
                            narrow: i32_from_i64(4294967303), flag: true, small: 1.5, wide: 2.5 };";
                        let source = format!(
                            "mod cpu Main {{
                                struct State {{ value: i64, tag: i64, narrow: i32,
                                    flag: bool, small: f32, wide: f64 }}
                                fn step(seed: State, limit: i64, branch: bool) -> State {{
                                    {entry} if branch {{ {entry} }} else {{ {entry} }} let i = 0;
                                    while i < limit {{
                                        let i = i + 1; let carry = carry; let selected = carry; let j = 0;
                                        let inner_limit = i % 3;
                                        while j < inner_limit {{
                                            let j = j + 1; let selected = selected; let old = selected;
                                            let selected = State {{ value: old.value + 1, tag: {},
                                                narrow: i32_from_i64(7), flag: true, small: 1.5, wide: 2.5 }};
                                            if branch {{ if j == 2 {{ return selected; }} }}
                                            if j == 1 {{ {exit} }}
                                            let checked = {};
                                            let selected = State {{ value: selected.value + 10, tag: 7,
                                                narrow: i32_from_i64(7), flag: true, small: 1.5, wide: 2.5 }};
                                        }}
                                        let carry = State {{ value: selected.value + 100, tag: 7,
                                            narrow: i32_from_i64(7), flag: true, small: 1.5, wide: 2.5 }};
                                    }}
                                    return carry;
                                }}
                                fn main() -> i64 {{
                                    let result = step(State {{ value: 10, tag: 3, narrow: i32_from_i64(3),
                                        flag: false, small: 0.5, wide: 0.5 }}, {limit}, {branch});
                                    if result.narrow != i32_from_i64(7) {{ return 201; }}
                                    if !result.flag {{ return 202; }}
                                    if result.small != 1.5 {{ return 203; }}
                                    if result.wide != 2.5 {{ return 204; }}
                                    return result.value * 1000 + result.tag;
                                }}
                            }}",
                            if changed { 8 } else { 7 },
                            if trap { "1 / (j - 2)" } else { "j" },
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
                            for i in 1..=limit {
                                for j in 1..=i % 3 {
                                    value += 1;
                                    if branch && j == 2 {
                                        return Ok(value * 1000 + if changed { 8 } else { 7 });
                                    }
                                    if j == 1 {
                                        if exit == "break;" {
                                            break;
                                        }
                                        if exit == "continue;" {
                                            continue;
                                        }
                                    }
                                    if trap && j == 2 {
                                        return Err(());
                                    }
                                    value += 10;
                                }
                                value += 100;
                            }
                            Ok(value * 1000 + 7)
                        })();
                        assert_eq!(actual, expected, "{source}");
                    }
                }
            }
        }
    }
}
