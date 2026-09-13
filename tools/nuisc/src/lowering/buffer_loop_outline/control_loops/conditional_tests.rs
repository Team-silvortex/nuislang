use super::*;
use crate::frontend::parse_nuis_module;

fn source() -> String {
    carries_tests::SOURCE
        .replace("let total: i64 = total + index;", "if index > initial { let total: i64 = total + index; } else { let total: i64 = total * index; }")
        .replace("let checksum: i64 = checksum * total;", "if total < bound { let checksum: i64 = checksum * total; } else { let checksum: i64 = checksum; }")
}

#[test]
fn conditional_carries_reuse_ordered_preparation_and_keep_buffer_catalog_narrow() {
    let mut module = parse_nuis_module(&source()).unwrap();
    let layouts = control_values::layouts(&module);
    for _ in 0..2 {
        let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
        for name in ["walk", "wrap", "choose"] {
            assert!(catalog[name].may_loop);
        }
        assert_eq!(
            scalar_helpers::collect(&module).keys().collect::<Vec<_>>(),
            ["main"]
        );
        module.functions.reverse();
    }
    let outlined = outline_buffer_loops(&mut module).unwrap();
    assert!(outlined.functions.contains("walk"));
    assert!(outlined.functions.contains("choose"));
    assert_eq!(outlined.guarded_functions.len(), 2);
}

#[test]
fn conditional_comparisons_support_invariant_atoms_and_reversed_operands() {
    for condition in [
        "index == initial",
        "index != initial",
        "index <= 2",
        "index >= initial",
        "initial < index",
        "initial > index",
    ] {
        let module = parse_nuis_module(&source().replace("index > initial", condition)).unwrap();
        let catalog =
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert!(
            catalog.get("choose").is_some_and(|entry| entry.may_loop),
            "{condition}"
        );
    }
}

#[test]
fn reversed_condition_preparation_does_not_hoist_expressions() {
    for condition in ["(bound / initial) < index", "(bound + initial) < index"] {
        let module = parse_nuis_module(&source().replace("index > initial", condition)).unwrap();
        let walk = module.functions.iter().find(|f| f.name == "walk").unwrap();
        let NirStmt::While { condition, body } = walk
            .body
            .iter()
            .find(|stmt| matches!(stmt, NirStmt::While { .. }))
            .unwrap()
        else {
            unreachable!()
        };
        assert!(prepare_chained_while(
            condition,
            body,
            &BTreeSet::new(),
            &BTreeMap::new(),
            &BTreeMap::new()
        )
        .is_none());
    }
}

#[test]
fn conditional_carries_reject_stale_state_fallible_predicates_and_body_effects() {
    for (from, to) in [
        ("index > initial", "index > total"),
        ("index > initial", "checksum > initial"),
        ("index > initial", "total > initial"),
        ("total < bound", "total < index"),
        ("index > initial", "index > (bound / initial)"),
        ("index > initial", "index > wrap(initial, bound, step)"),
        ("index > initial", "index > initial && index < bound"),
        ("let total: i64 = total * index;", "let checksum: i64 = checksum * index;"),
        ("let total: i64 = total * index;", "let total: i64 = total / index;"),
        ("let total: i64 = total * index;", "let total: i64 = total + checksum;"),
        ("let total: i64 = total * index;", "let total: i64 = total + index; print(total);"),
        ("let total: i64 = total * index;", "let total: i64 = total + index; let total: i64 = total * index;"),
        ("let total: i64 = total * index;", "if index > initial { let total: i64 = total * index; } else { let total: i64 = total; }"),
        ("let total: i64 = initial;", "const total: i64 = initial;"),
        ("let checksum: i64 = checksum;", ""),
    ] {
        let module = parse_nuis_module(&source().replace(from, to)).unwrap();
        let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert!(!catalog.contains_key("walk"), "{to}");
        assert!(!catalog.contains_key("choose"), "{to}");
    }
}

#[test]
fn conditional_arms_cannot_mutate_the_header_or_change_scalar_kind() {
    let source = source()
        .replace("total", "limit")
        .replace("let limit: i64 = initial;", "");
    let module = parse_nuis_module(&source).unwrap();
    let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
    assert!(!catalog.contains_key("choose"));
    let mut module = parse_nuis_module(&self::source()).unwrap();
    let function = module
        .functions
        .iter_mut()
        .find(|f| f.name == "walk")
        .unwrap();
    let NirStmt::While { body, .. } = function
        .body
        .iter_mut()
        .find(|s| matches!(s, NirStmt::While { .. }))
        .unwrap()
    else {
        unreachable!()
    };
    let NirStmt::If { else_body, .. } = &mut body[1] else {
        unreachable!()
    };
    let NirStmt::Let { ty, .. } = &mut else_body[0] else {
        unreachable!()
    };
    *ty = Some(scalar_type("i32"));
    let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
    assert!(!catalog.contains_key("choose"));
}
