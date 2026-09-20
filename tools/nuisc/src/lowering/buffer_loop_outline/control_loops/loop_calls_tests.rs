use super::*;
use crate::frontend::parse_nuis_module;

const HELPERS: &str = "
    struct Packet { value: i64, count: i64 }
    fn checked(value: i64, divisor: i64) -> i64 { return value / divisor; }
    fn fold(value: i64, limit: i64, stride: i64) -> i64 {
        let index: i64 = 0;
        let total: i64 = value;
        while index < limit {
            let index: i64 = index + stride;
            let total: i64 = checked(total, stride) + index;
        }
        return total;
    }
    fn relay(value: i64, limit: i64, stride: i64) -> i64 { return fold(value, limit, stride); }
    fn record(value: i64, limit: i64, stride: i64) -> Packet {
        return Packet { value: relay(value, limit, stride), count: limit };
    }
    fn predicate(value: i64, limit: i64, stride: i64) -> bool { return fold(value, limit, stride) > 0; }
    fn broken(value: i64, limit: i64, stride: i64) -> i64 {
        let index: i64 = 0;
        while index < limit { let index: i64 = index + index; }
        return index;
    }
    fn broken_wrapper(value: i64, limit: i64, stride: i64) -> i64 { return broken(value, limit, stride); }
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
fn loop_call_catalog_validates_callees_before_iteration_callers() {
    for body in [
        "let total: i64 = relay(total, index, stride);",
        "let total: i64 = total; let local = record(total, index, stride); let total: i64 = local.value + local.count;",
        "if index == bound || predicate(index, index, stride) { let total: i64 = total + index; }",
        "if index < bound { let local = fold(index, index, stride); let total: i64 = total + local; }",
        "let unused = fold(index, index, stride);",
    ] {
        for reversed in [false, true] {
            let mut module = parse_nuis_module(&source(body)).unwrap();
            if reversed { module.functions.reverse(); }
            let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
            for name in ["fold", "relay", "record", "predicate", "walk", "wrap", "choose"] {
                assert!(catalog[name].may_loop, "{name}: {body}");
                assert!(!scalar_helpers::collect(&module).contains_key(name));
            }
            assert!(!catalog["checked"].may_loop);
            let outlined = outline_buffer_loops(&mut module).unwrap();
            for name in ["walk", "fold", "relay", "checked"] {
                assert!(outlined.functions.contains(name), "{name}: {body}");
            }
        }
    }
}

#[test]
fn loop_call_catalog_blocks_invalid_dependencies_and_recursive_components() {
    let valid = source("let total: i64 = relay(total, index, stride);");
    for variant in 0..6 {
        let mut module = parse_nuis_module(&valid).unwrap();
        let leaf = module
            .functions
            .iter_mut()
            .find(|f| f.name == "checked")
            .unwrap();
        match variant {
            0 => leaf.body.insert(0, NirStmt::Print(NirExpr::Int(1))),
            1 => leaf.is_async = true,
            2 => leaf.params[0].ty.is_ref = true,
            3 => leaf.return_type = Some(scalar_type("bool")),
            4 => {
                leaf.body = vec![NirStmt::Return(Some(NirExpr::Call {
                    callee: "checked".into(),
                    args: vec![NirExpr::Var("value".into()), NirExpr::Var("divisor".into())],
                }))]
            }
            5 => {
                leaf.body = vec![NirStmt::Return(Some(NirExpr::Call {
                    callee: "missing".into(),
                    args: vec![],
                }))]
            }
            _ => unreachable!(),
        }
        let catalog =
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        for name in [
            "checked",
            "fold",
            "relay",
            "record",
            "predicate",
            "walk",
            "wrap",
            "choose",
        ] {
            assert!(!catalog.contains_key(name), "variant={variant}, {name}");
        }
    }
    for body in [
        "let total: i64 = broken_wrapper(total, index, stride);",
        "let total: i64 = wrap(index, bound, stride);",
        "let local: i64 = record(index, index, stride);",
        "let local = fold(checksum, index, stride); let checksum: i64 = checksum + 1; let total: i64 = total + local;",
    ] {
        if let Ok(module) = parse_nuis_module(&source(body)) {
            assert!(!scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module)).contains_key("walk"), "{body}");
        }
    }
    for text in [
        valid.replacen("index < limit", "index < fold(limit, bound, stride)", 1),
        valid.replacen("index + stride", "index + fold(stride, bound, stride)", 1),
    ] {
        let module = parse_nuis_module(&text).unwrap();
        assert!(
            !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("walk")
        );
    }
}

#[test]
fn loop_call_dependency_discovery_is_iterative_and_not_a_two_level_profile() {
    let mut module = parse_nuis_module("mod cpu Main { fn main() -> i64 { return 0; } fn template(v: i64) -> i64 { let index: i64 = 0; let total: i64 = v; while index < v { let index: i64 = index + 1; let total: i64 = total + index; } return total; } }").unwrap();
    let template = module.functions.pop().unwrap();
    assert_eq!(template.name, "template");
    for slot in 0..2048 {
        let mut function = template.clone();
        function.name = format!("level_{slot}");
        if slot != 0 {
            let NirStmt::While { body, .. } = &mut function.body[2] else {
                unreachable!()
            };
            let NirStmt::Let { value, .. } = &mut body[1] else {
                unreachable!()
            };
            *value = NirExpr::Call {
                callee: format!("level_{}", slot - 1),
                args: vec![NirExpr::Var("total".into())],
            };
        }
        module.functions.push(function);
    }
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    assert_eq!(catalog.len(), 2049);
    assert!(catalog["level_2047"].may_loop);
    assert_eq!(scalar_helpers::collect(&module).len(), 1);
    module.functions.reverse();
    assert!(scalar_helpers::collect_with_layouts(&module, &layouts)
        .keys()
        .eq(catalog.keys()));
}
