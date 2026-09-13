use super::*;
use std::collections::BTreeMap;
use yir_core::Operation;

fn node(carries: &[&str]) -> Node {
    Node {
        name: "guarded".into(),
        resource: "cpu0".into(),
        op: Operation::parse(
            "cpu.loop_while_scalar_cond_chain",
            ["initial", "limit", "step", "lt", "add"]
                .into_iter()
                .chain(carries.iter().copied())
                .map(str::to_owned)
                .collect(),
        )
        .unwrap(),
    }
}

#[test]
fn conditional_metadata_preserves_selected_and_previous_state_contracts() {
    let good = [
        "seed",
        "current_gt",
        "threshold",
        "add_current",
        "mul_prev_current",
        "seed",
        "carry0_ge",
        "threshold",
        "add_carry0",
        "keep",
        "seed",
        "always",
        "initial",
        "mul_carry1",
        "mul_carry1",
    ];
    for instruction in ["loop_while_i64_cond_chain", "loop_while_scalar_cond_chain"] {
        let mut n = node(&good);
        n.op.instruction = instruction.into();
        assert_eq!(parse(&n).unwrap().len(), 3);
        super::super::validate(&n, &BTreeMap::new()).unwrap();
    }
    for condition in ["prev_current_le", "prev_carry0_ne"] {
        let n = node(&[
            "seed",
            condition,
            "threshold",
            "mul_prev_carry0",
            "keep_prev_carry",
        ]);
        assert_eq!(parse(&n).unwrap().len(), 1);
    }
    assert!(parse(&node(&["seed", "always", "add_current", "keep"])).is_ok());
    let mut named = node(&["seed", "always", "add_current", "keep"]);
    named.op.args[0] = "add_current".into();
    assert!(
        parse(&named).is_ok(),
        "a value name is not a metadata placeholder"
    );
}

#[test]
fn conditional_metadata_rejects_hidden_payloads_forward_state_and_ignored_tokens() {
    for carries in [
        vec!["seed", "current_gt", "threshold", "add_carry0", "keep"],
        vec!["seed", "carry0_gt", "threshold", "add_current", "keep"],
        vec!["seed", "prev_carry1_gt", "threshold", "add_current", "keep"],
        vec!["seed", "current_gt", "threshold", "add_prev_carry1", "keep"],
        vec![
            "seed",
            "current_gt",
            "threshold",
            "add_invariant",
            "hidden",
            "keep",
        ],
        vec!["seed", "always", "hidden", "add_current", "keep"],
        vec![
            "seed",
            "always",
            "initial",
            "add_current",
            "scoped_call",
            "foreign",
        ],
        vec![
            "seed",
            "xor",
            "current_gt",
            "threshold",
            "current_lt",
            "threshold",
            "add_current",
            "keep",
        ],
        vec!["seed", "current_gt", "threshold", "add_current"],
        vec![
            "seed",
            "always",
            "initial",
            "add_current",
            "keep",
            "trailing",
        ],
    ] {
        assert!(parse(&node(&carries)).is_err(), "{carries:?}");
    }
    let huge = vec!["and"; 10000];
    assert!(parse(&node(&huge)).unwrap_err().contains("shape/bound"));
}

#[test]
fn conditional_metadata_enforces_slot_and_induction_bounds() {
    for count in [1, 3, 7, 64, 65] {
        let mut carries = Vec::new();
        for _ in 0..count {
            carries.extend(["seed", "always", "add_current", "keep"]);
        }
        assert_eq!(parse(&node(&carries)).is_ok(), count <= 64);
    }
    let mut n = node(&["seed", "always", "add_current", "keep"]);
    n.op.args[3] = "always".into();
    assert!(super::super::validate(&n, &BTreeMap::new()).is_err());
    n.op.args[3] = "lt".into();
    n.op.args[4] = "call".into();
    assert!(super::super::validate(&n, &BTreeMap::new()).is_err());
}

#[test]
fn compound_metadata_preserves_nested_state_and_opaque_rhs_names() {
    let n = node(&[
        "seed",
        "and",
        "current_gt",
        "or",
        "or",
        "prev_carry0_le",
        "and",
        "always",
        "add_current",
        "keep",
    ]);
    let carries = parse(&n).unwrap();
    assert!(matches!(carries[0].condition, LoopCondExpr::Binary { .. }));
    super::super::validate(&n, &BTreeMap::new()).unwrap();
    for invalid in ["carry0_le", "prev_carry1_le", "xor", "current_bad"] {
        let mut changed = n.clone();
        changed.op.args[10] = invalid.into();
        assert!(parse(&changed).is_err(), "{invalid}");
    }
}

#[test]
fn compound_metadata_bounds_depth_and_nodes_before_recursive_parsing() {
    let wrap = |expr: Vec<String>| {
        let mut n = node(&[]);
        n.op.args.push("seed".into());
        n.op.args.extend(expr);
        n.op.args.extend(["add_current".into(), "keep".into()]);
        n
    };
    let leaf = || vec!["current_gt".to_owned(), "threshold".to_owned()];
    let mut deep = leaf();
    for depth in 2..=shape::MAX_CONDITION_DEPTH + 1 {
        deep = [vec!["and".into()], deep, leaf()].concat();
        let result = parse(&wrap(deep.clone()));
        assert_eq!(
            result.is_ok(),
            depth <= shape::MAX_CONDITION_DEPTH,
            "{depth}"
        );
        if depth > shape::MAX_CONDITION_DEPTH {
            assert!(result.unwrap_err().contains("shape/bound"));
        }
    }
    let mut wide = leaf();
    for _ in 0..6 {
        wide = [vec!["or".into()], wide.clone(), wide].concat();
    }
    assert_eq!(parse(&wrap(wide.clone())).unwrap().len(), 1);
    let oversized = [vec!["and".into()], wide, leaf()].concat();
    assert!(parse(&wrap(oversized)).unwrap_err().contains("shape/bound"));
    for bad in [
        vec!["and".into(), "always".into()],
        vec!["or".into(), "always".into(), "ignored".into()],
    ] {
        assert!(parse(&wrap(bad)).is_err());
    }
}
