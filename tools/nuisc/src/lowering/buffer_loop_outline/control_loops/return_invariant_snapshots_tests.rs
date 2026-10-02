use super::super::tests::fixture;
use super::*;

fn masks(prefix: &str, effects: &str) -> BTreeMap<String, Vec<bool>> {
    let (module, scope, body) = fixture(&format!("{prefix} while flag {{ {effects} }}"));
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let mut budget = Budget(MAX_WORK);
    let mut snapshots = Snapshots::new(&scope, &layouts, &catalog, &mut budget).unwrap();
    let mut clock = 0;
    let (last, prefix) = body.split_last().unwrap();
    prefix_snapshots(
        prefix,
        &mut snapshots,
        &layouts,
        &catalog,
        &mut budget,
        &mut clock,
    );
    let NirStmt::While { body, .. } = last else {
        panic!("fixture must end in a loop");
    };
    let scope = snapshots
        .env
        .iter()
        .map(|(name, value)| (name.clone(), value.ty.clone()))
        .collect();
    analyze_at(
        body,
        &scope,
        &layouts,
        &catalog,
        &mut budget,
        Some(&snapshots),
    )
    .unwrap()
    .into_iter()
    .map(|(name, (_, fields))| (name, fields))
    .collect()
}

fn prefix_snapshots(
    body: &[NirStmt],
    snapshots: &mut Snapshots,
    layouts: &control_values::CarryLayouts,
    catalog: &ScalarHelpers,
    budget: &mut Budget,
    clock: &mut usize,
) {
    for stmt in body {
        match stmt {
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                let mut left = snapshots.clone();
                let mut right = snapshots.clone();
                prefix_snapshots(then_body, &mut left, layouts, catalog, budget, clock);
                prefix_snapshots(else_body, &mut right, layouts, catalog, budget, clock);
                snapshots.join(&left, &right, budget, clock).unwrap();
            }
            NirStmt::While { body, .. } => snapshots.forget_writes(body, budget, clock, 0).unwrap(),
            _ => snapshots
                .bind(stmt, layouts, catalog, budget, clock, 0)
                .unwrap(),
        }
    }
}

#[test]
fn return_invariant_snapshot_joins_keep_correlated_arm_values_and_nested_aliases() {
    for prefix in [
        "let carry = a; let tag = carry.left.tag; if flag { let tag = carry.left.tag; }",
        "let carry = a; let tag = carry.left.tag;
         if flag { let carry = b; let tag = carry.left.tag; }
         else { let carry = a; let tag = carry.left.tag; }",
        "let carry = a; let tag = carry.left.tag;
         if flag { let local = relay(b); let carry = local; let tag = local.left.tag; }
         else { let local = relay(a); let carry = local; let tag = local.left.tag; }",
        "let carry = a; let tag = carry.left.tag;
         if flag {
            if flag { let carry = relay(b); let tag = carry.left.tag; }
            else { let carry = relay(a); let tag = carry.left.tag; }
         } else { let carry = b; let tag = carry.left.tag; }",
    ] {
        let result = masks(prefix,
            "let carry = Nest { left: Pair { x: carry.left.x + 1, tag: tag }, right: carry.right };");
        assert_eq!(result["carry"], [false, true, true, true], "{prefix}");
    }
}

#[test]
fn return_invariant_snapshot_joins_do_not_equate_swapped_arms_or_separate_calls() {
    for prefix in [
        "let carry = a; let tag = carry.left.tag;
         if flag { let carry = b; let tag = a.left.tag; }
         else { let carry = a; let tag = b.left.tag; }",
        "let carry = a; let tag = carry.left.tag;
         if flag { let carry = relay(b); let tag = relay(b).left.tag; }
         else { let carry = relay(a); let tag = relay(a).left.tag; }",
        "let carry = a; let tag = carry.left.tag;
         if flag { let tag = carry.left.tag + 0; }",
        "let carry = a; let tag = carry.left.tag;
         if flag { let carry = b; let tag = carry.left.tag; }
         let carry = relay(carry);",
    ] {
        let result = masks(prefix,
            "let carry = Nest { left: Pair { x: carry.left.x + 1, tag: tag }, right: carry.right };");
        assert_eq!(result["carry"], [false, false, true, true], "{prefix}");
    }
}

#[test]
fn return_invariant_snapshot_joins_are_atomic_typed_scoped_and_budgeted() {
    let (module, scope, _) = fixture("");
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let mut budget = Budget(MAX_WORK);
    let mut snapshots = Snapshots::new(&scope, &layouts, &catalog, &mut budget).unwrap();
    let left = snapshots.clone();
    let mut right = snapshots.clone();
    right.env.insert("local".into(), Rc::clone(&right.env["a"]));
    let mut clock = 0;
    snapshots
        .join(&left, &right, &mut budget, &mut clock)
        .unwrap();
    assert!(!snapshots.env.contains_key("local"));
    let before = snapshots.env.clone();
    assert!(snapshots
        .join(&left, &right, &mut Budget(1), &mut clock)
        .is_none());
    assert!(snapshots.env == before);
    assert!(snapshots
        .join(&left, &right, &mut Budget(8), &mut clock)
        .is_none());
    assert!(
        snapshots.env == before,
        "late exhaustion must not publish the first binding"
    );
    let mut missing = right.clone();
    missing.env.remove("b");
    assert!(snapshots
        .join(&left, &missing, &mut budget, &mut clock)
        .is_none());
    assert!(snapshots.env == before);
    for invalid in [0, 1, 2] {
        let mut wrong = right.clone();
        let entry = &wrong.env["b"];
        let mut ty = entry.ty.clone();
        let mut words = entry.words.clone();
        match invalid {
            0 => ty.is_ref = true,
            1 => {
                words.pop();
            }
            _ => ty.name = "Pair".into(),
        }
        wrong.env.insert("b".into(), Rc::new(Value { ty, words }));
        assert!(snapshots
            .join(&left, &wrong, &mut budget, &mut clock)
            .is_none());
        assert!(snapshots.env == before);
    }
    let mut exhausted_clock = usize::MAX;
    assert!(snapshots
        .join(&left, &right, &mut budget, &mut exhausted_clock)
        .is_none());
    assert!(snapshots.env == before);
}

#[test]
fn return_invariant_snapshot_joins_never_share_unknown_leaves() {
    let (module, scope, _) = fixture("");
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let mut budget = Budget(MAX_WORK);
    let mut snapshots = Snapshots::new(&scope, &layouts, &catalog, &mut budget).unwrap();
    for name in ["a", "b"] {
        snapshots.env.insert(
            name.into(),
            Rc::new(Value {
                ty: scalar_type("Nest"),
                words: vec![None; 4],
            }),
        );
    }
    let arms = snapshots.clone();
    snapshots.join(&arms, &arms, &mut budget, &mut 0).unwrap();
    let origins = snapshots
        .env
        .values()
        .flat_map(|v| v.words.iter())
        .filter_map(Clone::clone)
        .collect::<BTreeSet<_>>();
    assert_eq!(origins.len(), 9);
}

#[test]
fn return_invariant_snapshots_prove_cross_binding_field_and_computed_snapshots() {
    let result = masks(
        "let saved = a.left; let tag = saved.tag; let carry = a;",
        "let carry = Nest { left: Pair { x: carry.left.x + 1, tag: tag }, right: carry.right };",
    );
    assert_eq!(result["carry"], [false, true, true, true]);
    let result = masks(
        "let carry = relay(a); const tag: i64 = carry.left.tag; let saved = carry.right;",
        "let carry = Nest { left: Pair { x: carry.left.x + 1, tag: tag }, right: saved };",
    );
    assert_eq!(result["carry"], [false, true, true, true]);
}

#[test]
fn return_invariant_snapshots_keep_old_versions_after_rebinding() {
    for prefix in [
        "let carry = a; let tag = carry.left.tag; let carry = b;",
        "let carry = relay(a); let tag = carry.left.tag; let carry = relay(a);",
        "let carry = a; let tag = carry.left.tag + 1;",
    ] {
        let result = masks(
            prefix,
            "let carry = Nest { left: Pair { x: carry.left.x + 1, tag: tag }, right: carry.right };",
        );
        assert_eq!(result["carry"], [false, false, true, true], "{prefix}");
    }
    let result = masks(
        "let saved = a; let a = b; let tag = saved.left.tag; let carry = saved;",
        "let carry = Nest { left: Pair { x: carry.left.x + 1, tag: tag }, right: carry.right };",
    );
    assert_eq!(result["carry"], [false, true, true, true]);
}

#[test]
fn return_invariant_snapshots_invalidate_preheader_branch_and_loop_writes() {
    for prefix in [
        "let carry = a; let tag = carry.left.tag; if flag { let carry = b; }",
        "let carry = a; let tag = carry.left.tag; while flag { let carry = b; break; }",
        "let carry = a; let tag = carry.left.tag; if flag { let tag = b.left.tag; }",
    ] {
        let result = masks(
            prefix,
            "let carry = Nest { left: Pair { x: carry.left.x + 1, tag: tag }, right: carry.right };",
        );
        assert_eq!(result["carry"], [false, false, true, true], "{prefix}");
    }
    let result = masks(
        "let carry = a; while flag { let carry = b; } let tag = carry.left.tag;",
        "let carry = Nest { left: Pair { x: carry.left.x + 1, tag: tag }, right: carry.right };",
    );
    assert_eq!(result["carry"], [false, true, true, true]);
}

#[test]
fn return_invariant_snapshots_check_later_trips_and_restored_intermediate_writes() {
    for effects in [
        "let carry = delayed; let delayed = b;",
        "let carry = delayed; let delayed = second; let second = b;",
        "let carry = relay(carry); let carry = delayed;",
    ] {
        assert!(
            masks("let carry = a; let delayed = a; let second = a;", effects).is_empty(),
            "{effects}"
        );
    }
    for exit in ["break", "continue"] {
        let result = masks(
            "let carry = a; let tag = carry.left.tag;",
            &format!("let carry = Nest {{ left: Pair {{ x: carry.left.x + 1, tag: tag + 1 }}, right: carry.right }};
                if flag {{ {exit}; }}
                let carry = Nest {{ left: Pair {{ x: carry.left.x, tag: tag }}, right: carry.right }};"),
        );
        assert_eq!(result["carry"], [false, false, true, true]);
    }
}

#[test]
fn return_invariant_snapshots_reject_qualifiers_and_exhausted_mutation_scans() {
    let (module, scope, body) = fixture("let saved = a; let a = relay(a);");
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let mut budget = Budget(MAX_WORK);
    let mut snapshots = Snapshots::new(&scope, &layouts, &catalog, &mut budget).unwrap();
    let mut clock = 0;
    snapshots
        .bind(&body[0], &layouts, &catalog, &mut budget, &mut clock, 0)
        .unwrap();
    let before = snapshots.env.clone();
    assert!(snapshots
        .forget_writes(&body[1..], &mut Budget(1), &mut clock, 0)
        .is_none());
    assert!(snapshots.env == before);
    let mut qualified = body[1].clone();
    let NirStmt::Let { ty, .. } = &mut qualified else {
        unreachable!()
    };
    let mut reference = scalar_type("Nest");
    reference.is_ref = true;
    *ty = Some(reference);
    assert!(snapshots
        .bind(&qualified, &layouts, &catalog, &mut budget, &mut clock, 0)
        .is_none());
    assert!(snapshots.env == before);
}
