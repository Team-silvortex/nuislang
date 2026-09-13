use super::*;
use crate::frontend::parse_nuis_module;
use crate::lowering::loop_types::{PreparedCarryBranchSource, PreparedCarryUpdateKind};

fn source(arm: &str) -> String {
    carries_tests::SOURCE
        .replace("let total: i64 = total + index;", arm)
        .replace(
            "let checksum: i64 = checksum * total;",
            "if total < bound { let checksum: i64 = checksum * total; }",
        )
}

#[test]
fn missing_carry_arms_use_shared_keep_and_preserve_transitive_catalog_scope() {
    for (arm, empty_then) in [
        (
            "if index > initial { let total: i64 = total + index; }",
            false,
        ),
        (
            "if index > initial { let total: i64 = total + index; } else {}",
            false,
        ),
        (
            "if index > initial {} else { let total: i64 = total + index; }",
            true,
        ),
    ] {
        let mut module = parse_nuis_module(&source(arm)).unwrap();
        let layouts = control_values::layouts(&module);
        for _ in 0..2 {
            let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
            for name in ["walk", "wrap", "choose"] {
                assert!(
                    catalog.get(name).is_some_and(|helper| helper.may_loop),
                    "{arm}: {name}"
                );
            }
            assert_eq!(
                scalar_helpers::collect(&module).keys().collect::<Vec<_>>(),
                ["main"]
            );
            module.functions.reverse();
        }
        let walk = module.functions.iter().find(|f| f.name == "walk").unwrap();
        let NirStmt::While { condition, body } = walk
            .body
            .iter()
            .find(|stmt| matches!(stmt, NirStmt::While { .. }))
            .unwrap()
        else {
            unreachable!()
        };
        let prepared = prepare_chained_while(
            condition,
            body,
            &BTreeSet::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .unwrap();
        assert_eq!(prepared.carries.len(), 2);
        assert_eq!(prepared.carries[0].binding_name, "total");
        assert_eq!(prepared.carries[1].binding_name, "checksum");
        let PreparedCarryUpdateKind::Conditional {
            then_source,
            else_source,
            ..
        } = &prepared.carries[0].kind
        else {
            panic!("missing conditional carry");
        };
        assert_eq!(
            matches!(**then_source, PreparedCarryBranchSource::KeepCurrentValue),
            empty_then
        );
        assert_eq!(
            matches!(**else_source, PreparedCarryBranchSource::KeepCurrentValue),
            !empty_then
        );
        let PreparedCarryUpdateKind::Conditional { else_source, .. } = &prepared.carries[1].kind
        else {
            panic!("missing dependent conditional carry");
        };
        assert!(matches!(
            **else_source,
            PreparedCarryBranchSource::KeepCurrentValue
        ));
        let outlined = outline_buffer_loops(&mut module).unwrap();
        assert!(outlined.functions.contains("walk"));
        assert!(outlined.functions.contains("choose"));
    }
}

#[test]
fn empty_or_malformed_carry_arms_cannot_create_or_erase_state() {
    for arm in [
        "if index > initial {}",
        "if index > initial {} else {}",
        "if index > initial { let fresh: i64 = index; }",
        "if index > initial { let total: i64 = total + index; print(total); }",
        "if index > initial { print(total); } else { let total: i64 = total + index; }",
        "if index > initial { let total: i64 = total + index; let total: i64 = total * index; }",
        "if index > initial { let total: i64 = total + index; } else { let checksum: i64 = checksum + index; }",
        "if index > initial { let total: i64 = total / index; }",
        "if index > initial { let total: i64 = total + checksum; }",
        "if index > total { let total: i64 = total + index; }",
        "if checksum > initial { let total: i64 = total + index; }",
        "if index > initial && index < (bound / initial) { let total: i64 = total + index; }",
        "if index > initial { if index < bound { let total: i64 = total + index; } }",
        "if index > initial { break; }",
        "if index > initial { let index: i64 = index + stride; }",
    ] {
        let module = parse_nuis_module(&source(arm)).unwrap();
        let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert!(!catalog.contains_key("walk"), "{arm}");
        assert!(!catalog.contains_key("choose"), "{arm}");
    }
}

#[test]
fn one_sided_updates_require_existing_exact_i64_mutable_seeds_and_invariants() {
    let source = source("if index > initial { let total: i64 = total + index; }");
    for (from, to) in [
        ("let total: i64 = initial;", "const total: i64 = initial;"),
        ("total", "limit"),
        ("total", "stride"),
        ("total < bound", "total < index"),
        ("index > initial", "index > (bound / initial)"),
        ("index > initial", "index > wrap(initial, bound, step)"),
    ] {
        let module = parse_nuis_module(&source.replace(from, to)).unwrap();
        let catalog =
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert!(!catalog.contains_key("choose"), "{from} => {to}");
    }
}

#[test]
fn invalid_one_sided_carry_seeds_and_types_are_rejected_before_lowering() {
    let source = source("if index > initial { let total: i64 = total + index; }");
    for (from, to, expected) in [
        ("let total: i64 = initial;", "", "unknown value `total`"),
        (
            "let total: i64 = total + index;",
            "let total: i32 = total + index;",
            "binding `total` expected type `i32`, found `i64`",
        ),
    ] {
        let error = parse_nuis_module(&source.replace(from, to)).unwrap_err();
        assert!(error.contains(expected), "{error}");
    }
}
