use super::*;

pub(in crate::lowering::buffer_loop_outline::conditional_returns) fn assert_selected_effect_not_pure(
    source: &str,
) {
    let mut module = crate::frontend::parse_nuis_module(source).unwrap();
    let layouts = control_values::TypedLayouts::collect(&module);
    let carries = control_values::layouts(&module);
    let control = scalar_helpers::collect_with_layouts(&module, &carries);
    let catalog = scalar_helpers::collect_typed_values(&module, &layouts, &control);
    let checked = speculation::collect_checked_arithmetic(&module);
    let event = module.functions.iter().find(|f| f.name == "event").unwrap();
    let scope = event
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect::<Scope>();
    let (condition, yes, no) = event
        .body
        .iter()
        .find_map(|stmt| match stmt {
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } if condition == &NirExpr::Var("outer".into()) => {
                Some((condition, then_body, else_body))
            }
            _ => None,
        })
        .unwrap();
    let result = event.return_type.as_ref().unwrap();
    assert!(
        prepare(condition, yes, no, result, &scope, &catalog, &layouts, &checked).is_none(),
        "nested prints must not become pure-tail authority"
    );
    assert!(
        effects::prepare(condition, yes, no, result, &scope, &catalog, &layouts, &checked)
            .is_none(),
        "nested prints must not become ordinary print-prefix authority"
    );
    let mut bindings = scope.keys().cloned().collect();
    branches::collect_bindings(&event.body, &mut bindings);
    assert!(
        effects::regions::prepare(
            condition, yes, no, result, &scope, &catalog, &layouts, &checked, &bindings
        )
        .is_some(),
        "selected-region proof is required"
    );
    assert!(!outline_test(&mut module).is_empty());
    crate::nir_verify::verify_nir_module(&module).unwrap();
    let once = module.clone();
    assert!(outline_test(&mut module).is_empty());
    assert_eq!(module, once);
}

const ARMS: [&str; 8] = [
    "let first = nested; if first { print(88); return helper(produce(divisor)); }",
    "let selected = divisor > 0; if selected { print(88); return helper(produce(divisor)); }",
    "if divisor > 0 { print(88); return helper(produce(divisor)); }",
    "if nested { return helper(produce(divisor)); } else { print(88); }",
    "if helper(produce(divisor)) { print(88); return false; } else { return false; }",
    "if nested { return helper(produce(divisor)); } else { print(88); return false; }",
    "if nested { return helper(produce(divisor)); } else { let selected = gate; print(88); return selected; }",
    "if nested { return helper(produce(divisor)); } print(88);",
];

fn case(
    index: usize,
    outer: bool,
    nested: bool,
    gate: bool,
    divisor: i64,
) -> (String, i64, Vec<i64>, usize) {
    let mut prints = vec![99];
    let mut calls = 0;
    let mut returned = false;
    let mut value = false;
    if outer {
        match index {
            0..=2 => {
                let selected = if index == 0 { nested } else { divisor > 0 };
                if selected {
                    prints.push(88);
                    calls = 1;
                    returned = true;
                    value = divisor > 0;
                }
            }
            3 | 7 => {
                if nested {
                    calls = 1;
                    returned = true;
                    value = divisor > 0;
                } else {
                    prints.push(88);
                }
            }
            4 => {
                calls = 1;
                returned = true;
                if divisor > 0 {
                    prints.push(88);
                }
            }
            5 | 6 => {
                returned = true;
                if nested {
                    calls = 1;
                    value = divisor > 0;
                } else {
                    prints.push(88);
                    value = index == 6 && gate;
                }
            }
            _ => unreachable!(),
        }
    }
    if !returned {
        prints.push(77);
    }
    let result = if value { 11 } else { 19 };
    prints.push(result);
    let arm = ARMS[index];
    let text = format!("mod cpu Main {{
        struct Packet {{ unused: i64, value: i64 }}
        @noinline fn produce(divisor: i64) -> Packet {{ return Packet {{ unused: 10 / divisor, value: divisor }}; }}
        @noinline fn helper(packet: Packet) -> bool {{ return packet.value > 0; }}
        @noinline fn event(outer: bool, nested: bool, gate: bool, divisor: i64) -> bool {{
            print(99); if outer {{ {arm} }} print(77); return false;
        }}
        fn main() -> i64 {{ let value = event({outer}, {nested}, {gate}, {divisor});
            if value {{ print(11); return 11; }} print(19); return 19;
        }}
    }}");
    (text, result, prints, calls)
}

#[test]
fn conditional_return_effect_boundaries_keep_pure_vetoes_and_selected_print_semantics() {
    let mut cases = 0;
    for index in 0..ARMS.len() {
        for (outer, nested, gate, divisor) in [
            (false, true, true, 0),
            (false, false, false, 0),
            (true, true, true, 2),
            (true, false, true, 2),
            (true, true, false, -2),
            (true, false, false, -2),
            (true, true, false, 2),
            (true, false, true, -2),
        ] {
            let (text, result, prints, calls) = case(index, outer, nested, gate, divisor);
            assert_selected_effect_not_pure(&text);
            execute(&text, Some(result), &prints, calls);
            cases += 1;
        }
    }
    assert_eq!(cases, 64);
}

#[test]
fn conditional_return_effect_boundaries_execute_original_native_selected_prints_and_continuations()
{
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for index in 0..ARMS.len() {
        for nested in [false, true] {
            let (text, _, prints, _) = case(index, true, nested, true, 2);
            let text = text
                .replace("print(11); return 11;", "print(11); return 0;")
                .replace("print(19); return 19;", "print(19); return 0;");
            let expected = prints
                .into_iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            native::run(&text, Some(&expected));
        }
    }
}
