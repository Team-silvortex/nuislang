use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str) -> String {
    carries_tests::SOURCE.replace(
        "let total: i64 = total + index;\n      let checksum: i64 = checksum * total;",
        body,
    )
}

#[test]
fn checked_iteration_expressions_use_scoped_helpers_with_or_without_temporaries() {
    for body in [
        "let total: i64 = total / index;",
        "let total: i64 = total % index;",
        "let total: i64 = total + (index / stride); let checksum: i64 = checksum % total;",
        "let quotient = index / stride; let remainder: i64 = index % stride; let total: i64 = total + quotient + remainder;",
        "if index < bound { let total: i64 = total / stride; } else { let total: i64 = total % stride; }",
        "if index > initial || index < (bound / initial) { let total: i64 = total + index; }",
        "if index > initial && (bound % initial) < index { let total: i64 = total + index; }",
        "let valid = stride != 0 && index / stride < bound; if valid { let total: i64 = total + 1; }",
        "if stride == 0 || index % stride == 0 { let total: i64 = total + 1; }",
        "if index > initial { let local = index / stride; if local > 0 { let total: i64 = total + local; } }",
        "let unused = index / stride;",
    ] {
        for reversed in [false, true] {
            let mut module = parse_nuis_module(&source(body)).unwrap();
            if reversed { module.functions.reverse(); }
            let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
            for name in ["walk", "wrap", "choose"] {
                assert!(catalog.get(name).is_some_and(|f| f.may_loop), "{name}: {body}");
            }
            assert_eq!(scalar_helpers::collect(&module).keys().collect::<Vec<_>>(), ["main"]);
            let outlined = outline_buffer_loops(&mut module).unwrap();
            let iteration = module.functions.iter().find(|f| f.name.starts_with("__nuis_scalar_iteration_")).unwrap();
            assert!(outlined.functions.contains(&iteration.name));
            for name in ["quotient", "remainder", "local", "unused", "valid"] {
                assert!(!iteration.params.iter().any(|p| p.name == name), "{body}");
            }
        }
    }
}

#[test]
fn checked_arithmetic_does_not_weaken_header_scope_type_or_effect_admission() {
    for body in [
        "let local = checksum / index; let total: i64 = total + local; let checksum: i64 = checksum + 1;",
        "let local = index / stride; let index: i64 = index + 1;",
        "let local = index / stride; let limit: i64 = limit + 1;",
        "let local = index / stride; let stride: i64 = stride + 1;",
        "let local = index / stride; let bound: i64 = bound + 1;",
        "let local = index / wrap(index, bound, stride); let total: i64 = total + local;",
        "let local = index / stride; print(local);",
        "let local = index / stride; while index < bound { let index: i64 = index + 1; }",
        "if index < bound { let local = index / stride; } let total: i64 = total + local;",
        "let local: bool = index / stride;",
        "let local: i64 = true / false;",
        "let local: i64 = index / 1.5;",
    ] {
        if let Ok(module) = parse_nuis_module(&source(body)) {
            let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
            assert!(!catalog.contains_key("choose"), "{body}");
        }
    }
    let valid = source("let local = index / stride; let total: i64 = total + local;");
    for text in [
        valid.replace("index < limit", "index < (limit / stride)"),
        valid.replace("index + stride", "index + (stride / 2)"),
        valid.replace("let total: i64 = initial;", "const total: i64 = initial;"),
    ] {
        let module = parse_nuis_module(&text).unwrap();
        let catalog =
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert!(!catalog.contains_key("choose"), "{text}");
    }
}

#[test]
fn checked_scope_rejects_forged_operand_kinds_even_in_an_unreached_arm() {
    let text =
        source("if index < bound { let local = index / stride; let total: i64 = total + local; }");
    let mut module = parse_nuis_module(&text).unwrap();
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
    let NirStmt::If {
        condition,
        then_body,
        ..
    } = &mut body[1]
    else {
        unreachable!()
    };
    *condition = NirExpr::Bool(false);
    let NirStmt::Let {
        value: NirExpr::Binary { rhs, .. },
        ..
    } = &mut then_body[0]
    else {
        unreachable!()
    };
    **rhs = NirExpr::Bool(false);
    let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
    assert!(!catalog.contains_key("walk"));
    assert!(!catalog.contains_key("choose"));
}
