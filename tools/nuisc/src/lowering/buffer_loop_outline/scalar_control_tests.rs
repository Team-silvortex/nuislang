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
        fn choose(flag: bool, value: i64, divisor: i64) -> i64 {
            if flag { return value; }
            return divide(value, divisor);
        }
        fn main() -> i64 { return 0; }
    }",
    );
    assert_eq!(helpers.len(), 2);
    assert_eq!(guarded.len(), 1);
    assert!(helpers
        .iter()
        .any(|f| f.name.starts_with("__nuis_scalar_continue")));
    let arm = helpers.iter().find(|f| guarded.contains(&f.name)).unwrap();
    assert!(matches!(arm.body.first(), Some(NirStmt::If { .. })));
    assert!(
        matches!(arm.body.last(), Some(NirStmt::Return(Some(NirExpr::Call { callee, .. }))) if callee.starts_with("__nuis_scalar_continue"))
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
    assert_eq!(helpers.len(), 4);
    assert_eq!(guarded.len(), 3);
    assert!(helpers.iter().any(|f| f
        .body
        .iter()
        .any(|s| matches!(s, NirStmt::Return(Some(NirExpr::FieldAccess { .. }))))));
}
