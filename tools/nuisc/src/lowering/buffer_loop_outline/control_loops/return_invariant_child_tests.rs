use super::*;
use crate::frontend::parse_nuis_module;

#[test]
fn return_invariants_revalidate_full_width_preheader_snapshots_with_changed_tags() {
    let base = include_str!(
        "../../../../tests/control_flow_syntax_native/scoped_return_typed_record_carries.ns"
    );
    for width in [9, 64] {
        let extra = |value: &str| {
            (9..width)
                .map(|i| format!(", extra{i}: {value}"))
                .collect::<String>()
        };
        let source = base
            .replace("count: i64", &format!("count: i64{}", extra("i64")))
            .replacen("count: limit", &format!("count: limit{}", extra("27")), 1)
            .replace("count: limit\n", &format!("count: limit{}\n", extra("99")))
            .replace(
                "tag: selected.tag, enabled: !enabled",
                "tag: i32_from_i64(17), enabled: !enabled",
            );
        tests::promoted(&source);
        let compiled = crate::pipeline::compile_source(&source).unwrap();
        let mut widths = compiled
            .yir
            .nodes
            .iter()
            .filter_map(|node| {
                yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args)
                    .unwrap()
                    .map(|call| call.seeds.len())
            })
            .collect::<Vec<_>>();
        widths.sort_unstable();
        assert_eq!(widths, [width - 4, width]);
    }
}

#[test]
fn return_invariants_do_not_treat_delayed_entry_aliases_as_loop_invariants() {
    let source = "mod cpu Main {
        struct State { value: i64, tag: i64 }
        fn step(seed: State, other: State, limit: i64) -> State {
            let carry = seed; let delayed = seed; let i = 0;
            while i < limit {
                let i = i + 1; let carry = carry; let delayed = delayed;
                let carry = delayed; let delayed = other;
                if i == 2 { return carry; }
            }
            return carry;
        }
        fn main() -> i64 {
            let result = step(State { value: 37, tag: 5 }, State { value: 99, tag: 7 }, 3);
            return result.value * 100 + result.tag;
        }
    }";
    let module = parse_nuis_module(source).unwrap();
    assert_eq!(nested_tests::execute(&module), Ok(9907));
    let layouts = control_values::layouts(&module);
    let mut independent = module.clone();
    for function in &mut independent.functions {
        if let Some(body) = normalize(function, &layouts).unwrap() {
            function.body = body;
        }
    }
    assert_eq!(nested_tests::execute(&independent), Ok(9907));
}

#[test]
fn return_invariants_promote_child_checked_record_snapshots_at_full_width() {
    let base =
        include_str!("../../../../tests/control_flow_syntax_native/scoped_return_child_exits.ns");
    for width in [9, 64] {
        let extra = |value: &str| {
            (9..width)
                .map(|i| format!(", extra{i}: {value}"))
                .collect::<String>()
        };
        let fields = (9..width)
            .map(|i| format!(", extra{i}: carry.extra{i}"))
            .collect::<String>();
        let source = base
            .replace("count: i64", &format!("count: i64{}", extra("i64")))
            .replacen("count: limit", &format!("count: limit{}", extra("27")), 1)
            .replace("count: limit\n", &format!("count: limit{}\n", extra("99")))
            .replace("if k == 1 { break; }", &format!(
                "let snapshot = State {{ left: carry.left, right: carry.right, count: 10 / (carry.count - carry.count){fields} }}; let observed = snapshot.right.tag; if k == 1 {{ break; }}"
            ));
        tests::promoted(&source);
    }
}

#[test]
fn return_invariants_preserve_ordinary_child_mutations_snapshots_and_exit_indices() {
    for leading in [false, true] {
        for limit in [0, 1, 3] {
            for child_limit in [0, 1, 3] {
                for exit in ["", "break;", "continue;"] {
                    for trap in [false, true] {
                        let step = "let j = j + 1;";
                        let exit_body = if exit.is_empty() {
                            "let noop = j;".to_owned()
                        } else if !leading && exit == "continue;" {
                            format!("{step} {exit}")
                        } else {
                            exit.to_owned()
                        };
                        let source = format!("mod cpu Main {{
                            struct State {{ value: i64, tag: i64 }}
                            fn step(seed: State, limit: i64, child_limit: i64) -> State {{
                                let carry = seed; let i = 0;
                                while i < limit {{
                                    let i = i + 1; let carry = carry; let old = carry;
                                    let j = 0;
                                    while j < child_limit {{
                                        {}
                                        let carry = carry; let previous = carry;
                                        let carry = State {{ value: previous.value + j, tag: previous.tag }};
                                        if j == 2 {{ {exit_body} }}
                                        let checked = {};
                                        let carry = State {{ value: carry.value + 10, tag: carry.tag }};
                                        {}
                                    }}
                                    if i == 2 {{ return State {{ value: carry.value, tag: carry.tag + j * 100 }}; }}
                                    let carry = State {{ value: carry.value + old.value, tag: carry.tag }};
                                }}
                                return carry;
                            }}
                            fn main() -> i64 {{
                                let result = step(State {{ value: 10, tag: 7 }}, {limit}, {child_limit});
                                return result.value * 1000 + result.tag;
                            }}
                        }}", if leading { step } else { "" },
                            if trap { "1 / (j - 2)" } else { "j" },
                            if leading { "" } else { step });
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
                                let old = value;
                                let mut observed = 0;
                                for trip in 0..child_limit {
                                    let j = trip + i64::from(leading);
                                    observed = j;
                                    value += j;
                                    if j == 2 {
                                        if exit == "break;" {
                                            break;
                                        }
                                        if exit == "continue;" {
                                            observed = trip + 1;
                                            continue;
                                        }
                                        if trap {
                                            return Err(());
                                        }
                                    }
                                    value += 10;
                                    observed = trip + 1;
                                }
                                if i == 2 {
                                    return Ok(value * 1000 + 7 + observed * 100);
                                }
                                value += old;
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
