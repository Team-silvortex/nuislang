use super::*;
use crate::frontend::parse_nuis_module;

#[allow(dead_code)]
#[path = "../../../tests/native_application_bridge/return_quint_tail_in_fields_oracle.rs"]
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
fn return_quint_tail_in_fields_retains_prefix_then_guarded_suffix_without_control_in_records() {
    for source in [SOURCE.to_owned(), oracle::mirror_source()] {
        let (module, generated) = rewrite(&source);
        assert!(!generated.is_empty());
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let records = module
            .structs
            .iter()
            .filter(|s| s.name.starts_with("__nuis_effect_fields_value"))
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 7);
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
                if callee == "nested_work" || callee == "prefix_work" || callee == "sibling_work")
                )
                .count(),
            3
        );
        assert!(module
            .functions
            .iter()
            .filter(|f| generated.contains(&f.name))
            .all(|f| f.params.len() <= 32));
        for name in [
            "tail_work",
            "tail_yes_work",
            "tail_no_work",
            "tail_child_work",
            "tail_pair_work",
            "tail_third_work",
            "tail_fourth_work",
            "tail_fifth_work",
        ] {
            assert_eq!(module.functions.iter().flat_map(|f| &f.body).filter(|s|
                        matches!(s,
                            NirStmt::Let { value: NirExpr::Call { callee, .. }, .. }
                            | NirStmt::Return(Some(NirExpr::Call { callee, .. })) if callee == name)
            ).count(), 1, "copied common-tail work: {name}");
        }
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
fn return_quint_tail_in_fields_keeps_atomic_authority_type_and_shape_vetoes() {
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
fn return_quint_tail_in_fields_rejects_sibling_exits_leaks_and_uncertified_writes_atomically() {
    for source in [
        SOURCE.replace(
            "let a = sibling_small_work(a, 1 / sibling_small);",
            "return e;",
        ),
        SOURCE.replace(
            "let a = sibling_small_work(a, 1 / sibling_small);",
            "if first { return e; }",
        ),
        SOURCE.replace(
            "let a = sibling_small_work(a, 1 / sibling_small);",
            "let seed = initialize(seed, 1 / init);",
        ),
        SOURCE.replace("let c = wide_seed;", "const c: f64 = wide_seed;"),
        SOURCE.replace(
            "let c = sibling_wide_work(c, b, d, 1 / sibling_wide);",
            "let c: f32 = b;",
        ),
        SOURCE.replace(
            "let c = sibling_wide_work(c, b, d, 1 / sibling_wide);",
            "let ignored = wide_work(c, b, d, 1 / wide_rhs);",
        ),
        SOURCE.replace(
            "let local = sibling_owner_work(local, a, b, c, d, e, 1 / sibling_owner);",
            "let a = sibling_small_work(a, 1 / sibling_small);",
        ),
        SOURCE.replace(
            "if exit_work",
            "if d { let b = narrow_work(b, 1 / narrow_rhs); return d; } if exit_work",
        ),
        SOURCE.replace(
            "let ignored = after_work",
            "if first { let a = small_work(a, 1 / small_rhs); let d = true; let e = false; let seed = local; } let ignored = after_work",
        ),
    ] {
        reject(&source);
    }
    let leaked = SOURCE
        .replace(
            "let c = sibling_wide_work(c, b, d, 1 / sibling_wide);",
            "let leaked = wide_work(c, b, d, 1 / wide_rhs);",
        )
        .replace(
            "let local = sibling_owner_work(local, a, b, c, d, e, 1 / sibling_owner);",
            "let local = sibling_owner_work(local, a, b, leaked, d, e, 1 / sibling_owner);",
        );
    // The frontend already rejects lexical leakage, before outlining can install.
    let error = parse_nuis_module(&leaked).unwrap_err();
    assert!(error.contains("unknown value `leaked`"), "{error}");
}

#[test]
fn return_quint_tail_in_fields_charges_original_work_once_without_budget_or_depth_reset() {
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
    let mut exact = budget(37, 256, 32);
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
        "sibling",
        "sibling_small",
        "sibling_wide",
        "sibling_owner",
        "after",
        "answer",
        "tail_cond",
        "tail_pair",
        "tail_no",
        "tail_child",
    ] {
        assert!(result.inputs.contains(name), "lost lexical input {name}");
    }
    assert!(["local", "a", "b", "c", "d", "e", "ignored"]
        .iter()
        .all(|n| !result.inputs.contains(*n)));
    let used = 256 - exact.expressions;
    for mut limited in [
        budget(36, 256, 32),
        budget(37, used - 1, 32),
        budget(37, 256, 2),
    ] {
        assert!(prove(then_body, &scope, &signatures, &mut limited, 1).is_none());
    }
    assert!(prove(then_body, &scope, &signatures, &mut budget(37, used, 3), 4).is_some());
    assert!(prove(then_body, &scope, &signatures, &mut budget(37, 256, 32), 5).is_none());
    let deeper = SOURCE.replace(
        "let a = sibling_small_work(a, 1 / sibling_small);",
        "if d { let a = sibling_small_work(a, 1 / sibling_small); }",
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
    let mut exact = budget(38, 256, 32);
    assert!(prove(then_body, &scope, &signatures, &mut exact, 4).is_some());
    assert_eq!(exact.nodes, 0);
    assert!(prove(then_body, &scope, &signatures, &mut budget(38, 256, 32), 5).is_none());
    let (module, generated) = rewrite(&deeper);
    assert!(!generated.is_empty());
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn return_quint_tail_in_fields_confines_nested_writes_to_certificate_and_five_children() {
    for source in [SOURCE.to_owned(), oracle::mirror_source()] {
        for (before, after) in [
            (
                "let local = tail_yes_work(local, 1);",
                "let d = true; let e = false; let seed = local;",
            ),
            (
                "let local = tail_yes_work(local, 1);",
                "let a = small_work(a, 1 / small_rhs); let e = false; let seed = local;",
            ),
            (
                "let local = tail_yes_work(local, 1);",
                "let seed = tail_yes_work(local, 1);",
            ),
            (
                "let local = tail_yes_work(local, 1);",
                "const local: i64 = tail_yes_work(local, 1);",
            ),
            ("let local = tail_yes_work(local, 1);", "return d;"),
            ("let local = tail_yes_work(local, 1);", "if d { return e; }"),
            (
                "let local = tail_yes_work(local, 1);",
                "let ignored = initialize(local, 1 / init);",
            ),
            (
                "let local = initialize(seed, 1 / init);",
                "const local: i64 = initialize(seed, 1 / init);",
            ),
            (
                "let local = tail_yes_work(local, 1);",
                "const private: i64 = 1; let local = tail_yes_work(local, 1);",
            ),
        ] {
            reject(&source.replace(before, after));
        }
        for binding in [
            "let a = small_work(a, 1 / small_rhs);",
            "let b = narrow_work(b, 1 / narrow_rhs);",
            "let d = bool_work(d, 1 / bool_rhs);",
            "let e = peer_work(d, 1 / peer_rhs);",
        ] {
            let alternate = source.replace("let c = tail_child_work(c, 1 / tail_child);", binding);
            let (module, generated) = rewrite(&alternate);
            assert!(!generated.is_empty());
            crate::nir_verify::verify_nir_module(&module).unwrap();
        }
        for extra in [
            "if first { let e = false; let seed = local; } let ignored = after_work",
            "if first { let seed = local; } let ignored = after_work",
            "let c = wide_work(c, b, d, 1 / wide_rhs); let ignored = after_work",
        ] {
            reject(&source.replace("let ignored = after_work", extra));
        }
        for prefix in ["let private = local;", "const private: bool = d && e;"] {
            let with_private = source.replace(
                "let local = tail_yes_work(local, 1);",
                &format!("{prefix} let local = tail_yes_work(local, 1);"),
            );
            let (module, generated) = rewrite(&with_private);
            assert!(!generated.is_empty());
            crate::nir_verify::verify_nir_module(&module).unwrap();
        }
        let separate = source
            .replace("let e = tail_fifth_work(e, b);", "")
            .replace(
                "let ignored = after_work",
                "if d { let e = tail_fifth_work(e, b); } let ignored = after_work",
            );
        let (module, generated) = rewrite(&separate);
        assert!(!generated.is_empty());
        crate::nir_verify::verify_nir_module(&module).unwrap();
    }
    let leaked = SOURCE
        .replace(
            "let local = tail_yes_work(local, 1);",
            "let private = tail_yes_work(local, 1);",
        )
        .replace(
            "return answer_work(local, seed, 1 / answer) && e;",
            "return answer_work(private, seed, 1 / answer) && e;",
        );
    assert!(parse_nuis_module(&leaked)
        .unwrap_err()
        .contains("unknown value `private`"));
}
