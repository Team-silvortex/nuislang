use super::*;
use crate::frontend::parse_nuis_module;

const HELPERS: &str = "
    fn leaf(value: i64, divisor: i64) -> i64 { return value / divisor; }
    fn relay(value: i64, divisor: i64) -> i64 { return leaf(value, divisor); }
    fn gate(value: i64, enabled: bool) -> bool { if enabled { return value > 0; } return false; }
    fn pick(enabled: bool, value: i64) -> i64 { if enabled { return value; } return 0; }
    fn ignore(value: i64) -> i64 { return 17; }
    fn impure(value: i64) -> i64 { print(value); return value; }
    fn effect_wrapper(value: i64) -> i64 { return impure(value); }
    fn recursive(value: i64) -> i64 { return recursive(value); }
    fn cycle_a(value: i64) -> i64 { return cycle_b(value); }
    fn cycle_b(value: i64) -> i64 { return cycle_a(value); }
    fn looped(value: i64) -> i64 { let index: i64 = 0;
        while index < value { let index: i64 = index + 1; } return index; }
    fn loop_wrapper(value: i64) -> i64 { return looped(value); }
    struct CallRecord { first: i64, second: i64 }
    fn record(value: i64) -> CallRecord { return CallRecord { first: value, second: value }; }
    fn record_reader(value: CallRecord) -> i64 { return value.first; }
";

fn source(body: &str) -> String {
    carries_tests::SOURCE
        .replace("fn main()", &format!("{HELPERS} fn main()"))
        .replace(
            "let total: i64 = total + index;\n      let checksum: i64 = checksum * total;",
            body,
        )
}

#[test]
fn iteration_calls_use_completed_scalar_closure_and_scoped_arguments() {
    for body in [
        "let total: i64 = relay(total, stride);",
        "let local = relay(index, stride); let total: i64 = total + local;",
        "if gate(relay(index, stride), true) { let total: i64 = total + index; }",
        "if gate(relay(index, stride), true) == true { let total: i64 = total + index; }",
        "let selected = gate(index, true) && relay(index, stride) < bound; if selected { let total: i64 = total + index; }",
        "let local = pick(stride == 0 || gate(relay(index, stride), true), index); let total: i64 = total + local;",
        "let total: i64 = total + pick(stride != 0 && relay(index, stride) > bound, index);",
        "let unused = ignore(relay(index, stride));",
        "let local = loop_wrapper(index); let total: i64 = total + local;",
    ] {
        for reversed in [false, true] {
            let mut module = parse_nuis_module(&source(body)).unwrap();
            if reversed { module.functions.reverse(); }
            let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
            assert!(catalog.contains_key("walk"), "{body}");
            assert!(!scalar_helpers::collect(&module).contains_key("walk"));
            let outlined = outline_buffer_loops(&mut module).unwrap();
            for name in ["walk", "wrap", "choose", "relay", "leaf"] {
                assert!(outlined.functions.contains(name), "{name}: {body}");
            }
            let iteration = module.functions.iter().find(|f| f.name.starts_with("__nuis_scalar_iteration_")).unwrap();
            assert!(outlined.functions.contains(&iteration.name));
            assert!(!iteration.params.iter().any(|p| ["local", "selected", "unused"].contains(&p.name.as_str())));
        }
    }
}

#[test]
fn iteration_calls_reject_transitive_effects_cycles_and_wrong_kinds() {
    for expr in [
        "impure(index)",
        "effect_wrapper(index)",
        "recursive(index)",
        "cycle_a(index)",
        "wrap(index, bound, stride)",
        "record(index)",
        "record_reader(true)",
        "relay(index)",
        "relay(index, true)",
        "relay(true, stride)",
        "gate(index, index)",
        "missing(index)",
    ] {
        let text = source(&format!(
            "if false {{ let local = {expr}; let total: i64 = total + local; }}"
        ));
        if let Ok(module) = parse_nuis_module(&text) {
            let catalog =
                scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
            assert!(!catalog.contains_key("walk"), "{expr}");
            assert!(!catalog.contains_key("choose"), "{expr}");
        }
    }
    for body in [
        "let local = relay(checksum, stride); let total: i64 = total + local; let checksum: i64 = checksum + 1;",
        "let index: i64 = ignore(index);", "let stride: i64 = ignore(stride);",
        "let bound: i64 = ignore(bound);", "let local: bool = relay(index, stride);",
        "if index < bound { let local = relay(index, stride); } let total: i64 = total + local;",
        "let local = gate(index, true); let local: bool = gate(index, false);",
    ] {
        if let Ok(module) = parse_nuis_module(&source(body)) {
            assert!(!scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module)).contains_key("walk"), "{body}");
        }
    }
}

#[test]
fn iteration_call_arguments_retain_expression_depth_and_header_atom_limits() {
    for (depth, admitted) in [(64, true), (65, false)] {
        // Source parsing and NIR expression admission have independent budgets.
        let mut module = parse_nuis_module(&source(
            "let local = index; let total: i64 = total + local;",
        ))
        .unwrap();
        let walk = module
            .functions
            .iter_mut()
            .find(|f| f.name == "walk")
            .unwrap();
        let NirStmt::While { body, .. } = walk
            .body
            .iter_mut()
            .find(|s| matches!(s, NirStmt::While { .. }))
            .unwrap()
        else {
            unreachable!()
        };
        let NirStmt::Let { value, .. } = &mut body[1] else {
            unreachable!()
        };
        for _ in 0..depth {
            *value = NirExpr::Call {
                callee: "ignore".into(),
                args: vec![value.clone()],
            };
        }
        assert_eq!(
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("walk"),
            admitted
        );
    }
    let valid = source("let local = relay(index, stride); let total: i64 = total + local;");
    for text in [
        valid.replace("index < limit", "index < ignore(limit)"),
        valid.replace("index + stride", "index + ignore(stride)"),
    ] {
        let module = parse_nuis_module(&text).unwrap();
        assert!(
            !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("walk")
        );
    }
}
