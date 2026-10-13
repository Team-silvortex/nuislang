use super::*;
use crate::frontend::parse_nuis_module;

const SOURCE: &str =
    include_str!("../../../tests/native_application_bridge/return_early_fields.ns");

fn rewrite(source: &str) -> (NirModule, BTreeSet<String>) {
    let mut module = parse_nuis_module(source).unwrap();
    let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated = outline(&mut module, &mut names);
    (module, generated)
}

fn reject(source: &str) {
    let mut module = parse_nuis_module(source).unwrap();
    let before = module.clone();
    let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
    let before_names = names.clone();
    assert!(outline(&mut module, &mut names).is_empty());
    assert_eq!(module, before);
    assert_eq!(names, before_names);
}

#[test]
fn return_early_fields_retains_current_fields_and_one_guarded_continuation() {
    let (module, generated) = rewrite(SOURCE);
    assert!(!generated.is_empty());
    crate::nir_verify::verify_nir_module(&module).unwrap();
    assert_eq!(
        module
            .structs
            .iter()
            .filter(|s| s.name.starts_with("__nuis_effect_fields_value"))
            .count(),
        1
    );
    let record = module
        .structs
        .iter()
        .find(|s| s.name.starts_with("__nuis_effect_fields_value"))
        .unwrap();
    assert_eq!(
        record
            .fields
            .iter()
            .map(|f| f.ty.name.as_str())
            .collect::<Vec<_>>(),
        ["i64", "i32", "f32", "f64", "bool", "bool"]
    );
    let continuation = module.functions.iter().filter(|f| generated.contains(&f.name))
        .filter(|f|f.body.iter().any(|s|matches!(s, NirStmt::Let { value: NirExpr::Call { callee, .. }, .. } if callee=="after_work")))
        .collect::<Vec<_>>();
    assert_eq!(continuation.len(), 1);
    assert_eq!(continuation[0].return_type, Some(scalar_type("bool")));
    assert!(
        matches!(&continuation[0].body[0], NirStmt::If { then_body, .. }
        if then_body == &[NirStmt::Return(Some(NirExpr::Bool(false)))])
    );
    for name in ["local", "a", "b", "c", "d", "e"] {
        assert!(continuation[0].params.iter().any(|p| p.name == name));
    }
    assert!(module
        .functions
        .iter()
        .filter(|f| generated.contains(&f.name))
        .all(|f| f.params.len() <= 32));
}

#[test]
fn return_early_fields_keeps_atomic_authority_type_and_shape_vetoes() {
    for source in [
        SOURCE
            .replace("let local = initialize(seed, 1 / init);", "")
            .replace("local", "seed"),
        SOURCE.replace("let a = small_seed;", "const a: i32 = small_seed;"),
        SOURCE.replace(
            "return d && answer_work(local, seed, 1 / answer);",
            "return local;",
        ),
        SOURCE.replace(
            "return d && answer_work(local, seed, 1 / answer);",
            "return;",
        ),
        SOURCE.replace(
            "let ignored = after_work",
            "let a = small_work(a, 1 / small_rhs); let ignored = after_work",
        ),
        SOURCE.replace(
            "let ignored = after_work",
            "if second { return d; } let ignored = after_work",
        ),
        SOURCE.replace(
            "if exit_work(local, a, b, c, d, e, 1 / exit_guard)",
            "if exit_work(local, a, b, c, d, e, 1 / exit_guard) { return e; } if first",
        ),
        SOURCE.replace(
            "let e = e && peer_work(d, 1 / peer_rhs);",
            "let e = e && peer_work(d, 1 / peer_rhs); print(78);",
        ),
        SOURCE.replace(
            "let a = small_work(a, 1 / small_rhs);",
            "let small_seed = small_work(a, 1 / small_rhs);",
        ),
        SOURCE.replace("let a = small_seed;", "let a: i64 = seed;"),
    ] {
        // Parser-level vetoes and proof-level vetoes both leave no installation.
        if parse_nuis_module(&source).is_ok() {
            reject(&source);
        }
    }
}

#[test]
fn return_early_fields_charges_original_work_once_without_budget_or_depth_reset() {
    let module = parse_nuis_module(SOURCE).unwrap();
    let function = module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap();
    let scope = function
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    let signatures = module
        .functions
        .iter()
        .filter_map(|f| {
            Some((
                f.name.clone(),
                (
                    f.params.iter().map(|p| p.ty.clone()).collect(),
                    f.return_type.clone()?,
                ),
            ))
        })
        .collect();
    let NirStmt::If { then_body, .. } = &function.body[1] else {
        panic!()
    };
    let budget = |nodes, expressions, logical_edges| nested::Budget {
        nodes,
        expressions,
        logical_edges,
    };
    let mut exact = budget(18, 256, 32);
    let result = prove(then_body, &scope, &signatures, &mut exact, 1).unwrap();
    assert_eq!(exact.nodes, 0);
    assert_eq!(exact.logical_edges, 29);
    for name in [
        "seed",
        "small_seed",
        "narrow_seed",
        "wide_seed",
        "bool_seed",
        "peer_seed",
        "second",
        "init",
        "cond",
        "small_rhs",
        "narrow_rhs",
        "wide_rhs",
        "bool_rhs",
        "peer_rhs",
        "parent_rhs",
        "exit_guard",
        "after",
        "answer",
    ] {
        assert!(result.inputs.contains(name), "lost lexical input {name}");
    }
    assert!(["local", "a", "b", "c", "d", "e", "ignored"]
        .iter()
        .all(|n| !result.inputs.contains(*n)));
    let used = 256 - exact.expressions;
    for mut limited in [
        budget(17, 256, 32),
        budget(18, used - 1, 32),
        budget(18, 256, 2),
    ] {
        assert!(prove(then_body, &scope, &signatures, &mut limited, 1).is_none());
    }
    assert!(prove(then_body, &scope, &signatures, &mut budget(18, used, 3), 6).is_some());
    assert!(prove(then_body, &scope, &signatures, &mut budget(18, 256, 32), 7).is_none());
}
