use super::*;
use crate::frontend::parse_nuis_module;

pub(super) fn fixture(body: &str) -> (NirModule, Scope, Vec<NirStmt>) {
    let module = parse_nuis_module(&format!(
        "mod cpu Main {{
            struct Pair {{ x: i64, tag: i64 }}
            struct Nest {{ left: Pair, right: Pair }}
            @noinline fn relay(value: Nest) -> Nest {{ return value; }}
            fn work(a: Nest, b: Nest, flag: bool) -> Nest {{ {body} return a; }}
            fn main() -> i64 {{ return 0; }}
        }}"
    ))
    .unwrap();
    let work = module.functions.iter().find(|f| f.name == "work").unwrap();
    let scope = work
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    let mut body = work.body.clone();
    body.pop();
    (module, scope, body)
}

pub(super) fn masks(body: &str) -> BTreeMap<String, Vec<bool>> {
    let (module, scope, body) = fixture(body);
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    analyze(&body, &scope, &layouts, &catalog, &mut Budget(MAX_WORK))
        .unwrap()
        .into_iter()
        .map(|(name, (_, mask))| (name, mask))
        .collect()
}

#[test]
fn return_invariant_facts_follow_nested_identity_and_constructor_names() {
    let result = masks(
        "let before = a; let a = Nest {
        right: before.right,
        left: Pair { tag: before.left.tag, x: before.left.x + 1 }
    };",
    );
    assert_eq!(result["a"], [false, true, true, true]);
    assert_eq!(masks("let a = a;")["a"], [true; 4]);
}

#[test]
fn return_invariant_facts_check_every_write_not_only_the_backedge() {
    let result = masks("let before = a;
        let a = Nest { left: Pair { x: before.left.x, tag: before.left.tag + 1 }, right: before.right };
        let a = before;");
    assert_eq!(result["a"], [true, false, true, true]);
    let result = masks(
        "let before = a;
        let a = Nest { left: before.right, right: before.left };
        let a = before;",
    );
    assert!(result.is_empty(), "temporary swaps are observable");
}

#[test]
fn return_invariant_facts_merge_both_arms_without_leaking_local_aliases() {
    let result = masks("if flag {
        let local = a;
        let a = Nest { left: Pair { x: local.left.x + 1, tag: local.left.tag }, right: local.right };
    } else {
        let local = a;
        let a = Nest { left: local.left, right: Pair { x: local.right.x, tag: local.right.tag + 1 } };
    }
    let a = a;");
    assert_eq!(result["a"], [false, true, true, false]);

    let (module, scope, mut body) = fixture("if flag { let local = a; } let a = a;");
    let NirStmt::Let { value, .. } = body.last_mut().unwrap() else {
        unreachable!()
    };
    *value = NirExpr::Var("local".into());
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    assert!(analyze(&body, &scope, &layouts, &catalog, &mut Budget(MAX_WORK)).is_none());
}

#[test]
fn return_invariant_facts_do_not_equate_roots_calls_or_algebra() {
    for body in [
        "let saved = a; let a = b; let b = saved;",
        "let a = relay(a);",
        "let a = Nest { left: Pair { x: a.left.x + 0, tag: a.left.tag + 0 }, right: Pair { x: a.right.x + 0, tag: a.right.tag + 0 } };",
    ] {
        assert!(masks(body).is_empty(), "{body}");
    }
}

#[test]
fn return_invariant_facts_reject_unknown_shapes_types_and_exhausted_proofs() {
    let (module, scope, body) = fixture("let a = a;");
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let before = body.clone();
    assert!(analyze(&body, &scope, &layouts, &catalog, &mut Budget(1)).is_none());
    assert_eq!(body, before);
    for value in [
        NirExpr::Text("unknown".into()),
        NirExpr::Await(Box::new(NirExpr::Int(0))),
        NirExpr::Int(0),
    ] {
        let body = [binding("a", &scalar_type("Nest"), value)];
        assert!(analyze(&body, &scope, &layouts, &catalog, &mut Budget(MAX_WORK)).is_none());
    }
    let mut deep = body;
    for _ in 0..MAX_DEPTH {
        deep = vec![NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: deep,
            else_body: vec![],
        }];
    }
    assert!(analyze(&deep, &scope, &layouts, &catalog, &mut Budget(MAX_WORK)).is_none());

    let mut layouts = layouts;
    layouts.insert(
        "Wide".into(),
        (0..65)
            .map(|i| (format!("f{i}"), scalar_type("i64")))
            .collect(),
    );
    let scope = [("a".into(), scalar_type("Wide"))].into_iter().collect();
    assert!(analyze(&[], &scope, &layouts, &catalog, &mut Budget(MAX_WORK)).is_none());
    layouts.insert("Cycle".into(), vec![("child".into(), scalar_type("Cycle"))]);
    let scope = [("a".into(), scalar_type("Cycle"))].into_iter().collect();
    assert!(analyze(&[], &scope, &layouts, &catalog, &mut Budget(MAX_WORK)).is_none());
}
