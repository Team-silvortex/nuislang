use super::*;
use crate::frontend::parse_nuis_module;
use crate::lowering::loop_types::{PreparedCarryUpdateKind, PreparedLoopFlowCondition};

const UPDATE: &str = "let total: i64 = total + index;";
const KEEP: &str = "let total: i64 = total;";

fn source(arm: &str) -> String {
    carries_tests::SOURCE.replace(UPDATE, arm).replace(
        "let checksum: i64 = checksum * total;",
        "if total < bound { if index > initial { let checksum: i64 = checksum * total; } }",
    )
}

#[test]
fn nested_update_trees_reuse_shared_collapse_and_transitive_catalog() {
    for arm in [
        format!("if index > initial {{ if index < bound {{ {UPDATE} }} }}"),
        format!("if index > initial {{ if index < bound {{ {UPDATE} }} else {{}} }} else {{ {KEEP} }}"),
        format!("if index > initial {{ {UPDATE} }} else {{ if index < bound {{ {UPDATE} }} }}"),
        format!("if index > initial {{ if index < bound {{ {UPDATE} }} else {{ let total: i64 = total * index; }} }} else {{ let total: i64 = total * index; }}"),
        format!("if initial < index || index == bound {{ if index < bound {{ {UPDATE} }} else {{ if initial != index {{ {UPDATE} }} }} }}"),
    ] {
        let mut module = parse_nuis_module(&source(&arm)).unwrap();
        let layouts = control_values::layouts(&module);
        for _ in 0..2 {
            let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
            for name in ["walk", "wrap", "choose"] {
                assert!(catalog.get(name).is_some_and(|f| f.may_loop), "{arm}: {name}");
            }
            assert_eq!(scalar_helpers::collect(&module).keys().collect::<Vec<_>>(), ["main"]);
            module.functions.reverse();
        }
        let walk = module.functions.iter().find(|f| f.name == "walk").unwrap();
        let NirStmt::While { condition, body } = walk.body.iter().find(|s| matches!(s, NirStmt::While { .. })).unwrap() else { unreachable!() };
        let prepared = prepare_chained_while(condition, body, &BTreeSet::new(), &BTreeMap::new(), &BTreeMap::new()).unwrap();
        assert_eq!(prepared.carries.len(), 2);
        for carry in &prepared.carries {
            assert!(matches!(carry.kind, PreparedCarryUpdateKind::Conditional { condition: PreparedLoopFlowCondition::Compound { .. }, .. }));
        }
        let outlined = outline_buffer_loops(&mut module).unwrap();
        assert!(outlined.functions.contains("walk"));
        assert!(outlined.functions.contains("choose"));
    }
}

#[test]
fn nested_conditions_cannot_hide_stale_forward_or_fallible_inputs() {
    for inner in [
        "index < total",
        "total < bound",
        "checksum < bound",
        "index < (bound / initial)",
        "index < wrap(initial, bound, step)",
        "index < bound || index == total",
        "index < bound && checksum > initial",
    ] {
        for arm in [
            format!("if index > initial {{ if {inner} {{ {UPDATE} }} }}"),
            format!("if index > initial {{ {UPDATE} }} else {{ if {inner} {{ {UPDATE} }} }}"),
        ] {
            let module = parse_nuis_module(&source(&arm)).unwrap();
            let catalog =
                scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
            assert!(!catalog.contains_key("walk"), "{arm}");
            assert!(!catalog.contains_key("choose"), "{arm}");
        }
    }
    let arm = format!("if index > initial {{ if index < bound {{ {UPDATE} }} }}");
    let module =
        parse_nuis_module(&source(&arm).replace("if total < bound", "if total < index")).unwrap();
    let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
    assert!(!catalog.contains_key("choose"));
}

#[test]
fn nested_arms_keep_shape_types_and_multi_outcome_boundaries_fail_closed() {
    for inner in [
        "if index < bound {}",
        "if index < bound { let fresh: i64 = index; }",
        "if index < bound { let total: i64 = total / index; }",
        "if index < bound { let total: i64 = total + checksum; }",
        "if index < bound { let total: i64 = total + index; print(total); }",
        "if index < bound { let total: i64 = total + index; let total: i64 = total * index; }",
        "if index < bound { let total: i64 = total + index; } else { let checksum: i64 = checksum + index; }",
        "if index < bound { let total: i64 = total + index; } else { let index: i64 = index + stride; }",
        "if index < bound { while total < bound { let total: i64 = total + 1; } }",
        "if index < bound { break; }",
        "if index < bound { continue; }",
        "if index < bound { let total: i64 = total + index; } else { let total: i64 = total * index; }",
        "if index < bound { let total: i64 = total + index; } else { let total: i64 = total + index; }",
        "if index < bound { if index > initial { let total: i64 = total + index; } } else { if initial != index { let total: i64 = total + index; } }",
    ] {
        // Three-outcome and noncollapsible two-outcome trees stay rejected.
        let arm = format!("if index > initial {{ {inner} }}");
        let module = parse_nuis_module(&source(&arm)).unwrap();
        let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert!(!catalog.contains_key("walk"), "{arm}");
        assert!(!catalog.contains_key("choose"), "{arm}");
    }
    let valid = source(&format!(
        "if index > initial {{ if index < bound {{ {UPDATE} }} }}"
    ));
    for invalid in [
        valid.replace("let total: i64 = initial;", ""),
        valid.replace(UPDATE, "let total: i32 = total + index;"),
        valid.replace("index < bound", "index < 1.5"),
    ] {
        assert!(parse_nuis_module(&invalid).is_err());
    }
    let module = parse_nuis_module(
        &valid.replace("let total: i64 = initial;", "const total: i64 = initial;"),
    )
    .unwrap();
    let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
    assert!(!catalog.contains_key("choose"));
}

#[test]
fn nested_source_catalog_does_not_copy_native_tree_budgets() {
    for depth in [2, 8, 40] {
        let mut arm = UPDATE.to_owned();
        for level in 0..depth {
            arm = if level % 2 == 0 {
                format!("if index > initial {{ {arm} }}")
            } else {
                format!("if index < bound {{ {UPDATE} }} else {{ {arm} }}")
            };
        }
        let module = parse_nuis_module(&source(&arm)).unwrap();
        let catalog =
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert!(catalog.get("choose").is_some_and(|f| f.may_loop), "{depth}");
    }
}
