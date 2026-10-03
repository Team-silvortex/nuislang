use super::super::tests::fixture;
use super::*;

#[test]
fn return_invariant_post_loop_boundaries_keep_stable_fields_but_freshen_each_unknown() {
    let (module, scope, body) = fixture(
        "let a = Nest { left: Pair { x: a.left.x + 1, tag: a.left.tag }, right: a.right };
         let b = a; let local = a;",
    );
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let mut budget = Budget(MAX_WORK);
    let snapshots = Snapshots::new(&scope, &layouts, &catalog, &mut budget).unwrap();
    let before = snapshots.env.clone();
    let facts = snapshots
        .loop_facts(&body, &layouts, &catalog, &mut budget, 0)
        .unwrap();
    let mut clock = 0;
    let entry = facts.at_boundary(&mut budget, &mut clock).unwrap();
    let exit = facts.at_boundary(&mut budget, &mut clock).unwrap();
    for name in ["a", "b"] {
        assert!(entry.env[name].words.iter().all(Option::is_some));
        assert!(exit.env[name].words.iter().all(Option::is_some));
        assert!(entry.env[name].words[0] != before[name].words[0]);
        assert!(entry.env[name].words[0] != exit.env[name].words[0]);
    }
    assert!(entry.env["a"].words[0] != entry.env["b"].words[0]);
    assert!(exit.env["a"].words[0] != exit.env["b"].words[0]);
    assert!(exit.env["a"].words[1..] == before["a"].words[1..]);
    assert!(!exit.env.contains_key("local"));
    assert!(snapshots.env == before);
    assert!(facts.env["a"].words[0].is_none());
}

#[test]
fn return_invariant_post_loop_materialization_is_atomic_on_budget_and_clock_exhaustion() {
    let (module, scope, body) = fixture("let a = relay(a); let b = relay(b);");
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let mut budget = Budget(MAX_WORK);
    let snapshots = Snapshots::new(&scope, &layouts, &catalog, &mut budget).unwrap();
    let facts = snapshots
        .loop_facts(&body, &layouts, &catalog, &mut budget, 0)
        .unwrap();
    let before = facts.env.clone();
    let mut clock = usize::MAX - 1;
    assert!(facts.at_boundary(&mut budget, &mut clock).is_none());
    assert_eq!(clock, usize::MAX - 1);
    let mut clock = 0;
    assert!(facts.at_boundary(&mut Budget(9), &mut clock).is_none());
    assert_eq!(clock, 0);
    assert!(facts.env == before);
    assert!(facts.at_boundary(&mut budget, &mut clock).is_some());
}

#[test]
fn return_invariant_post_loop_snapshots_do_not_recover_restored_or_delayed_origins() {
    for body in [
        "let carry = b; if flag { break; } let carry = a;",
        "let carry = b; if flag { continue; } let carry = a;",
        "let carry = delayed; let delayed = second; let second = b;",
        "if flag { let carry = relay(carry); }",
        "while flag { let carry = b; break; } let carry = a;",
    ] {
        let result = super::tests::entry_mask(
            "let carry = a; let delayed = a; let second = a;",
            body,
            "let carry = Nest { left: Pair { x: carry.left.x + 1, tag: a.left.tag }, right: carry.right };",
        );
        assert_eq!(result["carry"], [false, false, true, true], "{body}");
    }
}
