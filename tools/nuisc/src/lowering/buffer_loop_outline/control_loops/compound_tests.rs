use super::*;
use crate::frontend::parse_nuis_module;
use crate::lowering::loop_types::{PreparedCarryUpdateKind, PreparedLoopFlowCondition};

fn source(condition: &str) -> String {
    carries_tests::SOURCE
        .replace("let total: i64 = total + index;", &format!("if {condition} {{ let total: i64 = total + index; }}"))
        .replace("let checksum: i64 = checksum * total;", "if (total < bound || initial > index) && index != bound { let checksum: i64 = checksum * total; } else { let checksum: i64 = checksum; }")
}

#[test]
fn compound_predicates_reuse_shared_preparation_without_widening_buffer_catalog() {
    for condition in [
        "index > initial && index < bound",
        "initial <= index || bound != index",
        "(index > initial && index < bound) || index == initial",
        "index > initial && (index < bound || index == initial)",
    ] {
        let mut module = parse_nuis_module(&source(condition)).unwrap();
        let layouts = control_values::layouts(&module);
        for _ in 0..2 {
            let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
            for name in ["walk", "wrap", "choose"] {
                assert!(catalog[name].may_loop, "{condition}: {name}");
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
            .find(|s| matches!(s, NirStmt::While { .. }))
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
        for carry in &prepared.carries {
            assert!(matches!(
                carry.kind,
                PreparedCarryUpdateKind::Conditional {
                    condition: PreparedLoopFlowCondition::Compound { .. },
                    ..
                }
            ));
        }
        let outlined = outline_buffer_loops(&mut module).unwrap();
        assert!(outlined.functions.contains("walk"));
        assert!(outlined.functions.contains("choose"));
    }
}

#[test]
fn every_compound_leaf_requires_exact_state_order_and_invariant_atoms() {
    for condition in [
        "index > initial && index < total",
        "index > initial || checksum < bound",
        "index > initial && (index < bound || total > initial)",
        "index > initial || index < (bound / initial)",
        "index > initial && (bound % initial) < index",
        "index > initial || index < wrap(initial, bound, step)",
        "index > initial && (bound + initial) < index",
        "index > initial || index < 1.5",
    ] {
        let parsed = parse_nuis_module(&source(condition));
        if condition.ends_with("1.5") {
            assert!(parsed.is_err());
            continue;
        }
        let module = parsed.unwrap();
        let catalog =
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert!(!catalog.contains_key("walk"), "{condition}");
        assert!(!catalog.contains_key("choose"), "{condition}");
    }
}

#[test]
fn source_condition_catalog_does_not_duplicate_native_profile_budgets() {
    let condition = std::iter::repeat_n("index > initial", 70)
        .collect::<Vec<_>>()
        .join(" || ");
    let module = parse_nuis_module(&source(&condition)).unwrap();
    let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
    assert!(catalog["choose"].may_loop);
}
