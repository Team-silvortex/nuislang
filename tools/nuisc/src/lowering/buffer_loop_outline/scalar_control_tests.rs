use super::*;
use crate::frontend::parse_nuis_module;

fn outlined(source: &str) -> (NirModule, Vec<NirFunction>, BTreeSet<String>) {
    let mut module = parse_nuis_module(source).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    for function in &module.functions {
        assert!(catalog.contains_key(&function.name), "{}", function.name);
    }
    let retained = catalog.keys().cloned().collect();
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let mut helpers = Vec::new();
    let mut guarded = BTreeSet::new();
    outline(
        &mut module,
        &retained,
        &mut names,
        &mut helpers,
        &mut guarded,
        &catalog,
        &layouts,
    );
    let mut normalized = module.clone();
    normalized.functions.extend(helpers.clone());
    crate::nir_verify::verify_nir_module(&normalized).unwrap();
    (module, helpers, guarded)
}

#[test]
fn ready_scalar_and_record_returns_need_no_private_branch_or_continuation() {
    let (_, helpers, guarded) = outlined("mod cpu Main {
        struct Pair { a: i64, b: i64 }
        fn integer(flag: bool, value: i64) -> i64 { if flag { return value; } return 3; }
        fn boolean(flag: bool, value: bool) -> bool { if flag { return value; } return false; }
        fn record(flag: bool, a: Pair, b: Pair) -> Pair { if flag { return a; } else { return b; } }
        fn record_fallthrough(flag: bool, a: Pair, b: Pair) -> Pair { if flag { return a; } return b; }
        fn main() -> i64 { return 0; }
    }");
    assert!(helpers.is_empty());
    assert!(guarded.is_empty());
}

#[test]
fn nontrivial_suffixes_remain_shared_and_branch_work_stays_guarded() {
    let (_, helpers, guarded) = outlined(
        "mod cpu Main {
        fn divide(value: i64, divisor: i64) -> i64 { return value / divisor; }
        fn choose(flag: bool, inner: bool, value: i64, divisor: i64) -> i64 {
            if flag { if inner { return value; } }
            return divide(value, divisor);
        }
        fn main() -> i64 { return 0; }
    }",
    );
    assert_eq!(helpers.len(), 4);
    assert_eq!(guarded.len(), 3);
    assert!(helpers
        .iter()
        .any(|f| f.name.starts_with("__nuis_scalar_continue")));
    assert!(helpers
        .iter()
        .filter(|f| guarded.contains(&f.name))
        .all(|f| matches!(f.body.first(), Some(NirStmt::If { .. }))));
    assert_eq!(
        helpers
            .iter()
            .filter(|f| matches!(f.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee.starts_with("__nuis_scalar_continue")))
            .count(),
        2
    );
}

#[test]
fn atomic_return_elision_never_moves_predicates_projections_or_constructors() {
    let (module, helpers, guarded) = outlined("mod cpu Main {
        struct Pair { a: i64, b: i64 }
        fn predicate(value: i64) -> bool { return 9 / value > 0; }
        fn choose(value: i64) -> i64 { if predicate(value) { return 7; } return 7; }
        fn project(flag: bool, a: Pair) -> i64 { if flag { return a.a; } return 4; }
        fn create(flag: bool, value: i64) -> Pair { if flag { return Pair { a: value, b: 0 }; } return Pair { a: 0, b: value }; }
        fn main() -> i64 { return 0; }
    }");
    let choose = module
        .functions
        .iter()
        .find(|f| f.name == "choose")
        .unwrap();
    assert!(
        matches!(&choose.body[0], NirStmt::Let { value: NirExpr::Call { callee, .. }, .. } if callee == "predicate")
    );
    assert_eq!(helpers.len(), 3);
    assert_eq!(guarded.len(), 3);
    assert!(helpers.iter().any(|f| f
        .body
        .iter()
        .any(|s| matches!(s, NirStmt::Return(Some(NirExpr::FieldAccess { .. }))))));
}

#[test]
fn single_use_terminal_values_stay_inside_guards_without_forwarders() {
    let (_, helpers, guarded) = outlined(
        "mod cpu Main {
        struct Pair { a: i64, b: i64 }
        fn divide(value: i64, divisor: i64) -> i64 { return value / divisor; }
        fn integer(flag: bool, value: i64, divisor: i64) -> i64 {
            if flag { return value; } return divide(value, divisor);
        }
        fn boolean(flag: bool, value: i64, divisor: i64) -> bool {
            if flag { return true; } return divide(value, divisor) > 0;
        }
        fn record(flag: bool, saved: Pair, divisor: i64) -> Pair {
            if flag { return saved; }
            return Pair { a: divide(saved.a, divisor), b: saved.b };
        }
        fn project(flag: bool, saved: Pair) -> i64 {
            if flag { return 7; } return saved.a;
        }
        fn main() -> i64 { return 0; }
    }",
    );
    assert_eq!(helpers.len(), 4);
    assert_eq!(guarded.len(), 4);
    assert!(helpers.iter().all(|f| guarded.contains(&f.name)));
    assert!(helpers
        .iter()
        .all(|f| matches!(f.body.first(), Some(NirStmt::If { .. }))));
}

#[test]
fn terminal_fallthroughs_count_generated_uses_not_runtime_branch_paths() {
    let (_, helpers, guarded) = outlined(
        "mod cpu Main {
        fn divide(value: i64, divisor: i64) -> i64 { return value / divisor; }
        fn choose(flag: bool, inner: bool, value: i64, divisor: i64) -> i64 {
            if flag { return value; } else {
                if inner { let ignored = divide(value, divisor); }
                let local = value + 1;
            }
            return divide(value, divisor);
        }
        fn main() -> i64 { return 0; }
    }",
    );
    // Both inner paths share the suffix containing `local`, so the terminal
    // expression has one generated use even though two runtime paths reach it.
    assert_eq!(helpers.len(), 4);
    assert_eq!(guarded.len(), 3);
    assert_eq!(
        helpers
            .iter()
            .filter(|f| f.name.starts_with("__nuis_scalar_continue"))
            .count(),
        1
    );
}

#[test]
fn nontrivial_terminal_expression_is_never_duplicated_by_return_chains() {
    let mut source = String::from(
        "mod cpu Main {
        fn divide(value: i64, divisor: i64) -> i64 { return value / divisor; }
        fn choose(value: i64, divisor: i64) -> i64 {",
    );
    for i in 0..32 {
        source.push_str(&format!("if value == {i} {{ return value; }}"));
    }
    source.push_str("return divide(value, divisor); } fn main() -> i64 { return 0; } }");
    let (_, helpers, guarded) = outlined(&source);
    assert_eq!(helpers.len(), 32);
    assert_eq!(guarded.len(), 32);
    assert_eq!(
        helpers
            .iter()
            .filter(|f| matches!(f.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. }))) if callee == "divide"))
            .count(),
        1
    );
}

#[test]
fn single_use_statement_suffixes_stay_guarded_and_keep_unused_calls() {
    let (_, helpers, guarded) = outlined(
        "mod cpu Main {
        struct Pair { a: i64, b: i64 }
        fn divide(value: i64, divisor: i64) -> i64 { return value / divisor; }
        fn integer(flag: bool, value: i64, divisor: i64) -> i64 {
            if flag { return value; }
            let unused = divide(value, divisor); return value;
        }
        fn boolean(flag: bool, value: i64, divisor: i64) -> bool {
            if flag { return true; }
            let next = divide(value, divisor); return next > 0;
        }
        fn record(flag: bool, saved: Pair, divisor: i64) -> Pair {
            if flag { return saved; }
            let next = divide(saved.a, divisor);
            return Pair { a: next, b: saved.b };
        }
        fn main() -> i64 { return 0; }
    }",
    );
    assert_eq!(helpers.len(), 3);
    assert_eq!(guarded.len(), 3);
    assert!(helpers.iter().all(|f| guarded.contains(&f.name)));
    assert!(helpers
        .iter()
        .all(|f| matches!(f.body.first(), Some(NirStmt::If { .. }))));
    assert_eq!(
        helpers
            .iter()
            .filter(|f| f.body.iter().any(|stmt| matches!(stmt,
                NirStmt::Let { value: NirExpr::Call { callee, .. }, .. } if callee == "divide"
            )))
            .count(),
        3
    );
}

#[test]
fn single_use_statement_suffix_merges_into_shared_inner_tail_once() {
    let (_, helpers, guarded) = outlined(
        "mod cpu Main {
        fn divide(value: i64, divisor: i64) -> i64 { return value / divisor; }
        fn choose(flag: bool, inner: bool, value: i64, divisor: i64) -> i64 {
            if flag { return value; } else {
                if inner { let ignored = value + 1; }
                let local = value + 2;
            }
            let result = divide(value, divisor); return result;
        }
        fn main() -> i64 { return 0; }
    }",
    );
    assert_eq!(helpers.len(), 4);
    assert_eq!(guarded.len(), 3);
    let suffixes = helpers
        .iter()
        .filter(|f| !guarded.contains(&f.name))
        .collect::<Vec<_>>();
    assert_eq!(suffixes.len(), 1);
    assert!(suffixes[0]
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Let { name, .. } if name == "local")));
    assert!(suffixes[0]
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Let { name, .. } if name == "result")));
}

#[test]
fn statement_suffixes_keep_boundaries_for_multiple_uses() {
    for body in [
        "if flag { if inner { return value; } }",
        "if flag { let result = true; } else { let result = false; }",
        "if flag { return value; } else { if inner { let result = true; } }",
    ] {
        let (_, helpers, guarded) = outlined(&format!(
            "mod cpu Main {{
            fn divide(value: i64, divisor: i64) -> i64 {{ return value / divisor; }}
            fn choose(flag: bool, inner: bool, value: i64, divisor: i64) -> i64 {{
                {body}
                let result = divide(value, divisor); return result;
            }}
            fn main() -> i64 {{ return 0; }}
        }}"
        ));
        let suffixes = helpers
            .iter()
            .filter(|f| !guarded.contains(&f.name))
            .collect::<Vec<_>>();
        assert_eq!(suffixes.len(), 1, "{body}");
        assert!(matches!(&suffixes[0].body[0], NirStmt::Let { name, .. } if name == "result"));
    }
}

#[test]
fn single_use_collisions_rename_locals_without_touching_fields_or_callees() {
    let (_, helpers, guarded) = outlined(
        "mod cpu Main {
        struct Packet { result: i64, marker: i64 }
        fn result(value: i64) -> i64 { return value; }
        fn choose(flag: bool, value: i64) -> i64 {
            let __nuis_scalar_local_0 = value + 3;
            if flag { return value; } else {
                let result = Packet { result: result(value), marker: __nuis_scalar_local_0 };
                let unused = result.result + result.marker;
            }
            let result = result(value + 1); return result + __nuis_scalar_local_0;
        }
        fn main() -> i64 { return 0; }
    }",
    );
    assert_eq!(helpers.len(), 1);
    assert_eq!(guarded.len(), 1);
    let body = &helpers[0].body;
    assert!(body.iter().any(|stmt| matches!(stmt, NirStmt::Let { name, value: NirExpr::StructLiteral { type_name, fields, .. }, .. }
        if name == "__nuis_scalar_local_1" && type_name == "Packet" && fields[0].0 == "result"
        && matches!(&fields[0].1, NirExpr::Call { callee, .. } if callee == "result"))));
    assert!(body.iter().any(
        |stmt| matches!(stmt, NirStmt::Let { name, value: NirExpr::Binary { lhs, .. }, .. }
        if name == "unused" && matches!(lhs.as_ref(), NirExpr::FieldAccess { base, field }
            if field == "result" && base.as_ref() == &NirExpr::Var("__nuis_scalar_local_1".into())))
    ));
    assert!(body.iter().any(
        |stmt| matches!(stmt, NirStmt::Let { name, value: NirExpr::Call { callee, .. }, .. }
        if name == "result" && callee == "result")
    ));
}

#[test]
fn sibling_constants_keep_distinct_bindings_when_the_outer_suffix_folds() {
    let (_, helpers, guarded) = outlined(
        "mod cpu Main {
        fn identity(value: i64) -> i64 { return value; }
        fn choose(flag: bool, inner: bool, value: i64) -> i64 {
            if flag { return value; } else {
                if inner {
                    const result: i64 = 3;
                    if result == value { return identity(result); }
                } else {
                    const result: bool = true;
                    if result { return identity(value); }
                }
                let unused = identity(value);
            }
            let result = identity(value + 1); return result;
        }
        fn main() -> i64 { return 0; }
    }",
    );
    let mut constants = BTreeSet::new();
    for function in &helpers {
        for stmt in &function.body {
            if let NirStmt::Const { name, .. } = stmt {
                assert!(name.starts_with("__nuis_scalar_local_"));
                assert!(constants.insert(name.clone()));
            }
        }
    }
    assert_eq!(constants.len(), 2);
    assert_eq!(helpers.len() - guarded.len(), 1);
}
