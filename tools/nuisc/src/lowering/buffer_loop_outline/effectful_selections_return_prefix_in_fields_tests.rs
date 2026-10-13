use super::*;
use crate::frontend::parse_nuis_module;

#[allow(dead_code)]
#[path = "../../../tests/native_application_bridge/return_prefix_in_fields_oracle.rs"]
mod oracle;
use oracle::SOURCE;

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
fn return_prefix_in_fields_retains_prefix_then_guarded_suffix_without_control_in_records() {
    for source in [SOURCE.to_owned(), oracle::mirror_source()] {
        let (module, generated) = rewrite(&source);
        assert!(!generated.is_empty());
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let records = module
            .structs
            .iter()
            .filter(|s| s.name.starts_with("__nuis_effect_fields_value"))
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 4);
        for record in records {
            assert_eq!(
                record
                    .fields
                    .iter()
                    .map(|f| f.ty.name.as_str())
                    .collect::<Vec<_>>(),
                ["i64", "i32", "f32", "f64", "bool", "bool"]
            );
            assert!(record
                .fields
                .iter()
                .all(|f| !["exit", "result", "returned"].contains(&f.name.as_str())));
        }
        let continuations=module.functions.iter().filter(|f|generated.contains(&f.name))
            .filter(|f|f.body.iter().any(|s|matches!(s,NirStmt::Let {value:NirExpr::Call {callee,..},..} if callee=="after_work")))
            .collect::<Vec<_>>();
        assert_eq!(continuations.len(), 1);
        assert_eq!(continuations[0].return_type, Some(scalar_type("bool")));
        assert!(
            matches!(&continuations[0].body[0],NirStmt::If {then_body,..}
            if then_body==&[NirStmt::Return(Some(NirExpr::Bool(false)))])
        );
        for name in ["local", "a", "b", "c", "d", "e"] {
            assert!(continuations[0].params.iter().any(|p| p.name == name));
        }
        assert_eq!(module.functions.iter().flat_map(|f|&f.body).filter(|s|
            matches!(s,NirStmt::Let {value:NirExpr::Call {callee,..},..} if callee=="condition")).count(),1);
        assert_eq!(
            module
                .functions
                .iter()
                .flat_map(|f| &f.body)
                .filter(
                    |s| matches!(s, NirStmt::Let {value: NirExpr::Call {callee, ..}, ..}
                if callee == "nested_work" || callee == "prefix_work")
                )
                .count(),
            2
        );
        assert!(module
            .functions
            .iter()
            .filter(|f| generated.contains(&f.name))
            .all(|f| f.params.len() <= 32));
    }
    let source = SOURCE.replace(
        "let a = small_seed;",
        "let __nuis_effect_field_exit_gate_0 = first; let a = small_seed;",
    );
    let (module, generated) = rewrite(&source);
    assert!(!generated.is_empty());
    crate::nir_verify::verify_nir_module(&module).unwrap();
    assert!(module
        .functions
        .iter()
        .flat_map(|f| &f.body)
        .any(|s| matches!(s,
        NirStmt::Let {name,value:NirExpr::Call {callee,..},..}
            if name=="__nuis_effect_field_exit_gate_1" && callee=="condition")));
}
#[test]
fn return_prefix_in_fields_keeps_atomic_authority_type_and_shape_vetoes() {
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
fn return_prefix_in_fields_rejects_nested_exits_leaks_and_uncertified_writes_atomically() {
    for source in [
        SOURCE.replace("let e = prefix_peer_work(e, 1 / prefix_peer);", "return e;"),
        SOURCE.replace(
            "let e = prefix_peer_work(e, 1 / prefix_peer);",
            "if first { return e; }",
        ),
        SOURCE.replace(
            "let e = prefix_peer_work(e, 1 / prefix_peer);",
            "let seed = initialize(seed, 1 / init);",
        ),
        SOURCE.replace("let c = wide_seed;", "const c: f64 = wide_seed;"),
        SOURCE.replace(
            "let c = prefix_wide_work(c, b, d, 1 / prefix_wide);",
            "let c: f32 = b;",
        ),
        SOURCE.replace(
            "let c = prefix_wide_work(c, b, d, 1 / prefix_wide);",
            "let ignored = wide_work(c, b, d, 1 / wide_rhs);",
        ),
        SOURCE.replace(
            "if exit_work",
            "if d { let b = narrow_work(b, 1 / narrow_rhs); return d; } if exit_work",
        ),
        SOURCE.replace(
            "let ignored = after_work",
            "if first { let d = true; let e = false; } let ignored = after_work",
        ),
    ] {
        reject(&source);
    }
    let leaked = SOURCE
        .replace(
            "let c = prefix_wide_work(c, b, d, 1 / prefix_wide);",
            "let leaked = wide_work(c, b, d, 1 / wide_rhs);",
        )
        .replace(
            "if exit_work(local, a, b, c, d, e, 1 / exit_guard)",
            "if exit_work(local, a, b, leaked, d, e, 1 / exit_guard)",
        );
    // The frontend already rejects lexical leakage, before outlining can install.
    let error = parse_nuis_module(&leaked).unwrap_err();
    assert!(error.contains("unknown value `leaked`"), "{error}");
}

#[test]
fn return_prefix_in_fields_charges_original_work_once_without_budget_or_depth_reset() {
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
    let mut exact = budget(23, 256, 32);
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
        "nested",
        "prefix",
        "prefix_wide",
        "prefix_peer",
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
        budget(22, 256, 32),
        budget(23, used - 1, 32),
        budget(23, 256, 2),
    ] {
        assert!(prove(then_body, &scope, &signatures, &mut limited, 1).is_none());
    }
    assert!(prove(then_body, &scope, &signatures, &mut budget(23, used, 3), 5).is_some());
    assert!(prove(then_body, &scope, &signatures, &mut budget(23, 256, 32), 6).is_none());
    let deeper = SOURCE.replace(
        "let c = prefix_wide_work(c, b, d, 1 / prefix_wide);",
        "if b < 0.0 { let c = prefix_wide_work(c, b, d, 1 / prefix_wide); }",
    );
    let module = parse_nuis_module(&deeper).unwrap();
    let function = module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap();
    let NirStmt::If { then_body, .. } = &function.body[1] else {
        panic!()
    };
    let mut exact = budget(24, 256, 32);
    assert!(prove(then_body, &scope, &signatures, &mut exact, 4).is_some());
    assert_eq!(exact.nodes, 0);
    assert!(prove(then_body, &scope, &signatures, &mut budget(24, 256, 32), 5).is_none());
    let (module, generated) = rewrite(&deeper);
    assert!(!generated.is_empty());
    crate::nir_verify::verify_nir_module(&module).unwrap();
}
