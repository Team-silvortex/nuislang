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
            "and",
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
