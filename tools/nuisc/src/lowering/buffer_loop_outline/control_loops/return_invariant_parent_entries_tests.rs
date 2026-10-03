use super::super::tests::fixture;
use super::*;

pub(super) fn entry_mask(prefix: &str, parent: &str, child: &str) -> BTreeMap<String, Vec<bool>> {
    let (module, scope, body) = fixture(&format!("{prefix} while flag {{ {parent} }}"));
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let mut budget = Budget(MAX_WORK);
    let mut snapshots = Snapshots::new(&scope, &layouts, &catalog, &mut budget).unwrap();
    let mut clock = 0;
    let (last, prefix_body) = body.split_last().unwrap();
    for stmt in prefix_body {
        snapshots
            .bind(stmt, &layouts, &catalog, &mut budget, &mut clock, 0)
            .unwrap();
    }
    let NirStmt::While { body, .. } = last else {
        unreachable!()
    };
    let entries = snapshots
        .loop_facts(body, &layouts, &catalog, &mut budget, 0)
        .unwrap()
        .at_boundary(&mut budget, &mut clock)
        .unwrap();
    let (_, _, child) = fixture(&format!("{prefix} {child}"));
    let scope = entries
        .env
        .iter()
        .map(|(name, value)| (name.clone(), value.ty.clone()))
        .collect();
    analyze_at(
        &child[prefix_body.len()..],
        &scope,
        &layouts,
        &catalog,
        &mut budget,
        Some(&entries),
    )
    .unwrap()
    .into_iter()
    .map(|(name, (_, mask))| (name, mask))
    .collect()
}

#[test]
fn return_invariant_parent_entries_keep_only_per_write_stable_fields() {
    let result = entry_mask("let carry = a; let saved = a;",
        "let carry = Nest { left: Pair { x: carry.left.x + 1, tag: saved.left.tag }, right: carry.right };
         while flag { let carry = carry; break; }",
        "let carry = Nest { left: Pair { x: carry.left.x + 1, tag: saved.left.tag }, right: carry.right };");
    assert_eq!(result["carry"], [false, true, true, true]);
}

#[test]
fn return_invariant_parent_entries_include_later_trips_and_intermediate_exits() {
    for parent in [
        "let carry = delayed; let delayed = b;",
        "let carry = delayed; let delayed = second; let second = b;",
        "let carry = b; if flag { break; } let carry = a;",
        "let carry = b; if flag { continue; } let carry = a;",
        "while flag { let carry = b; break; } let carry = a;",
        "let carry = relay(a);",
    ] {
        let result = entry_mask("let carry = a; let delayed = a; let second = a;", parent,
            "let carry = Nest { left: Pair { x: carry.left.x + 1, tag: a.left.tag }, right: carry.right };");
        assert_eq!(result["carry"], [false, false, true, true], "{parent}");
    }
}

#[test]
fn return_invariant_parent_entries_do_not_share_varying_fields_or_export_locals() {
    let result = entry_mask("let carry = a; let saved = a;",
        "let local = a;
         let carry = Nest { left: Pair { x: carry.left.x + 1, tag: carry.left.tag + 1 }, right: carry.right };
         let saved = carry;",
        "let carry = Nest { left: Pair { x: carry.left.x + 1, tag: saved.left.tag }, right: carry.right };");
    assert_eq!(result["carry"], [false, false, true, true]);
    let (module, scope, body) = fixture("let local = a;");
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let mut budget = Budget(MAX_WORK);
    let snapshots = Snapshots::new(&scope, &layouts, &catalog, &mut budget).unwrap();
    let entries = snapshots
        .loop_facts(&body, &layouts, &catalog, &mut budget, 0)
        .unwrap()
        .at_boundary(&mut budget, &mut 0)
        .unwrap();
    assert!(!entries.env.contains_key("local"));
}

#[test]
fn return_invariant_parent_entries_reject_exhaustion_and_preserve_the_input() {
    let (module, scope, body) = fixture("let a = relay(a);");
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let mut budget = Budget(MAX_WORK);
    let snapshots = Snapshots::new(&scope, &layouts, &catalog, &mut budget).unwrap();
    let before = snapshots.env.clone();
    assert!(snapshots
        .loop_facts(&body, &layouts, &catalog, &mut Budget(1), 0)
        .is_none());
    assert!(snapshots
        .loop_facts(&body, &layouts, &catalog, &mut budget, MAX_DEPTH)
        .is_none());
    let mut exhausted_clock = usize::MAX;
    assert!(snapshots
        .loop_facts(&body, &layouts, &catalog, &mut budget, 0)
        .unwrap()
        .at_boundary(&mut budget, &mut exhausted_clock)
        .is_none());
    assert!(snapshots.env == before);
}
