use super::super::tests::{fixture, masks};
use super::*;

#[test]
fn return_invariant_loops_keep_only_per_write_identity_through_nested_children() {
    let result = masks("while flag {
        let before = a;
        while flag {
            let a = Nest { left: Pair { x: before.left.x + 1, tag: before.left.tag }, right: before.right };
            if flag { continue; }
            break;
        }
        if flag { break; }
    } let a = a;");
    assert_eq!(result["a"], [false, true, true, true]);
    assert_eq!(masks("while flag { let a = a; }")["a"], [true; 4]);
}

#[test]
fn return_invariant_loops_reach_a_fixed_point_for_delayed_aliases() {
    // The first iteration preserves a; the second sees b through delayed.
    assert!(masks(
        "let delayed = a;
        while flag { let a = delayed; let delayed = b; }"
    )
    .is_empty());
    assert!(masks(
        "let first = a; let second = a;
        while flag { let a = first; let first = second; let second = b; }"
    )
    .is_empty());
}

#[test]
fn return_invariant_loops_do_not_forget_zero_trips_or_restored_exit_snapshots() {
    for exit in ["break", "continue"] {
        let body = format!(
            "let snapshot = a;
            while flag {{
                let snapshot = b;
                if flag {{ {exit}; }}
                let snapshot = a;
            }}
            let a = snapshot;"
        );
        assert!(masks(&body).is_empty(), "{body}");
    }
    assert!(masks(
        "let snapshot = b;
        while flag { let snapshot = a; } let a = snapshot;"
    )
    .is_empty());
    assert!(masks(
        "let snapshot = a;
        while flag { while flag { let snapshot = b; break; } let snapshot = a; }
        let a = snapshot;"
    )
    .is_empty());
}

#[test]
fn return_invariant_loops_reject_escaping_locals_bad_conditions_and_exhaustion() {
    let (module, scope, mut body) = fixture("while flag { let local = a; } let a = a;");
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let original = body.clone();
    let NirStmt::Let { value, .. } = body.last_mut().unwrap() else {
        unreachable!()
    };
    *value = NirExpr::Var("local".into());
    assert!(analyze(&body, &scope, &layouts, &catalog, &mut Budget(MAX_WORK)).is_none());
    body = original.clone();
    let NirStmt::While { condition, .. } = &mut body[0] else {
        unreachable!()
    };
    *condition = NirExpr::Int(1);
    assert!(analyze(&body, &scope, &layouts, &catalog, &mut Budget(MAX_WORK)).is_none());
    let mut measured = Budget(MAX_WORK);
    analyze(&original, &scope, &layouts, &catalog, &mut measured).unwrap();
    let mut budget = Budget(MAX_WORK - measured.0 - 1);
    assert!(analyze(&original, &scope, &layouts, &catalog, &mut budget).is_none());
    assert_eq!(
        original,
        fixture("while flag { let local = a; } let a = a;").2
    );
    let mut nested = original;
    for _ in 0..MAX_DEPTH {
        nested = vec![NirStmt::While {
            condition: NirExpr::Var("flag".into()),
            body: nested,
        }];
    }
    assert!(analyze(&nested, &scope, &layouts, &catalog, &mut Budget(MAX_WORK)).is_none());
}
