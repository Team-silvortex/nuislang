use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str) -> String {
    carries_tests::SOURCE.replace(
        "let total: i64 = total + index;\n      let checksum: i64 = checksum * total;",
        body,
    )
}

const BODY: &str = "
    let delta: i64 = index * 2;
    let total: i64 = total + delta;
    let snapshot = total;
    let gate = snapshot < bound || (index > initial && snapshot != bound);
    let total: i64 = total + 1;
    if gate {
        let local: i64 = snapshot + index;
        let total: i64 = total + local;
        if local > snapshot { let delta: i64 = delta + local; }
    } else {
        let local: i64 = snapshot - index;
        let checksum: i64 = checksum + local;
    }
    let total: i64 = total + snapshot + delta;
";

#[test]
fn temporaries_are_iteration_local_not_driver_carries_or_captures() {
    for body in [
        BODY,
        "let delta = index * 2; let total: i64 = total + delta;",
        "let delta: i64 = index; let delta: i64 = delta + 1; let total: i64 = total + delta;",
        "if index > initial { let fresh: i64 = index; }",
        "let fresh: i64 = index;",
        "let flag: bool = index < bound; if flag { let total: i64 = total + index; }",
        "let flag = true; if flag == false { let total: i64 = total + index; }",
        "let flag = true; let saved = flag; let flag: bool = false; if saved { let total: i64 = total + index; }",
        "let flag: bool = true; if flag { let flag = false; } if flag { let total: i64 = total + index; }",
        "let flag = true; let other = false; if flag { let flag = false; let other = true; let total: i64 = total + index; } else { let flag = true; } if other { let checksum: i64 = checksum + index; }",
        "let flag = true; if flag { let local = false; if flag { let local = true; } let flag = local; } else { let local: i64 = index; let total: i64 = total + local; }",
        "if index < bound { let local: i64 = index; let total: i64 = total + local; } else { let local: bool = index > bound; if local { let total: i64 = total + 1; } }",
    ] {
        for reversed in [false, true] {
            let mut module = parse_nuis_module(&source(body)).unwrap();
            if reversed { module.functions.reverse(); }
            let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
            for name in ["walk", "wrap", "choose"] {
                assert!(catalog.get(name).is_some_and(|f| f.may_loop), "{name}: {body}");
            }
            assert_eq!(scalar_helpers::collect(&module).keys().collect::<Vec<_>>(), ["main"]);
            outline_buffer_loops(&mut module).unwrap();
            let iterations = module.functions.iter().filter(|f| f.name.starts_with("__nuis_scalar_iteration_")).collect::<Vec<_>>();
            assert_eq!(iterations.len(), 1, "{body}");
            let iteration = iterations[0];
            for name in ["delta", "snapshot", "gate", "local", "fresh", "flag"] {
                assert!(!iteration.params.iter().any(|p| p.name == name), "temporary captured as prior-iteration state: {name}");
            }
            let returned = iteration.body.last().unwrap();
            let expected = usize::from(body.contains("let total:")) + usize::from(body.contains("let checksum:"));
            match (expected, returned) {
                (0, NirStmt::Return(Some(NirExpr::Int(0)))) => {},
                (1, NirStmt::Return(Some(NirExpr::Var(name)))) => assert!(name == "total" || name == "checksum"),
                (2, NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))) => assert_eq!(fields.len(), 2),
                _ => panic!("temporaries must not escape: {returned:?}"),
            }
        }
    }
}

#[test]
fn temporary_scope_and_definite_initialization_are_fail_closed() {
    for body in [
        "let temp: i64 = temp + 1; let total: i64 = total + temp;",
        "let temp: i64 = later; let later: i64 = index; let total: i64 = total + temp;",
        "if index < bound { let temp: i64 = index; } let total: i64 = total + temp;",
        "if index < bound { let temp: i64 = index; } else { let total: i64 = total + temp; }",
        "if index < bound { let temp: i64 = index; } else { let temp: i64 = index + 1; } let total: i64 = total + temp;",
        "let temp: bool = index; let total: i64 = total + index;",
        "let temp: i64 = true; let total: i64 = total + index;",
        "let temp: i64 = index; let temp: bool = true; let total: i64 = total + index;",
        "let flag = true; let flag: i64 = index; let total: i64 = total + index;",
        "let flag = true; let flag = index; let total: i64 = total + index;",
        "let flag: bool = flag; let total: i64 = total + index;",
        "if index < bound { let flag = true; } else { let flag = false; } if flag { let total: i64 = total + index; }",
        "if index < bound { let flag = true; } else { let copy = flag; let total: i64 = total + index; }",
        "let temp: i64 = index; let limit: i64 = limit + 1;",
        "let temp: i64 = index; let stride: i64 = stride + 1;",
        "let temp: i64 = index; let index: i64 = index + 1;",
        "let temp: i64 = index; let bound: i64 = bound + 1;",
        "let temp: i64 = wrap(index, limit, stride); let total: i64 = total + temp;",
        "let temp: i64 = index; print(temp);",
        "let temp: i64 = index; const fixed: i64 = temp;",
        // A temporary does not launder an unavailable sibling read.
        "let temp: i64 = checksum; let total: i64 = total + temp; let checksum: i64 = checksum + index;",
        // Do not hide compound boolean evaluation under an eager equality.
        "let flag: bool = (index < bound && index > initial) == false; let total: i64 = total + index;",
    ] {
        let module = match parse_nuis_module(&source(body)) {
            Ok(module) => module,
            Err(error) => {
                assert!(error.contains("unknown value") || error.contains("type"), "unexpected frontend error for {body}: {error}");
                continue;
            }
        };
        let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert!(!catalog.contains_key("walk"), "{body}");
        assert!(!catalog.contains_key("choose"), "{body}");
    }
}

#[test]
fn bool_write_authority_does_not_escape_the_iteration_or_widen_buffer_effects() {
    for declaration in ["let", "const"] {
        let text =
            source("let local = true; let outer: bool = local; let total: i64 = total + index;")
                .replace(
                    "while index < limit",
                    &format!("{declaration} outer: bool = false; while index < limit"),
                );
        assert!(text.contains(&format!("{declaration} outer:")));
        let module = parse_nuis_module(&text).unwrap();
        assert!(
            !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("walk")
        );
    }
    let scope = Scope::from([("flag".to_owned(), scalar_type("bool"))]);
    let mutations = MutationScope {
        writable: BTreeSet::from(["flag".to_owned()]),
        protected: BTreeSet::new(),
    };
    let body = vec![NirStmt::Let {
        name: "flag".to_owned(),
        ty: None,
        value: NirExpr::Bool(false),
    }];
    assert!(validate_effects(
        &body,
        &mut scope.clone(),
        &mut BTreeSet::new(),
        validation::EffectTypes::Buffer(&ScalarHelpers::new()),
        &mutations,
        &mut Vec::new(),
    )
    .is_none());
}

#[test]
fn bool_branch_transport_uses_canonical_words_without_new_loop_state() {
    let mut module = parse_nuis_module(&source(
        "let flag = true; let other = false;
        if flag { let flag = false; let other = true; let total: i64 = total + index; }
        if other { let flag = true; }
        if flag { let checksum: i64 = checksum + index; }",
    ))
    .unwrap();
    outline_buffer_loops(&mut module).unwrap();
    let iteration = module
        .functions
        .iter()
        .find(|f| f.name.starts_with("__nuis_scalar_iteration_"))
        .unwrap();
    assert!(!iteration
        .params
        .iter()
        .any(|p| p.name == "flag" || p.name == "other"));
    assert!(iteration.body.iter().any(|s| matches!(s, NirStmt::Let { ty: Some(ty), value: NirExpr::CastI64ToBool(_), .. } if ty == &scalar_type("bool"))));
    for layout in &module.structs {
        if layout.name.starts_with("__nuis_scalar_carries_") {
            assert!(layout.fields.iter().all(|f| f.ty == scalar_type("i64")));
        }
    }
    assert!(module.functions.iter().any(|f| {
        f.name.starts_with("__nuis_buffer_branch_")
            && matches!(
                f.body.last(),
                Some(NirStmt::Return(Some(NirExpr::CastBoolToI64(_))))
            )
    }));
}

#[test]
fn temporaries_cannot_escape_loop_or_borrow_sibling_branch_initialization() {
    for body in [
        "let temp = index; let total: i64 = total + temp;",
        "if index < bound { let temp = index; let total: i64 = total + temp; }",
    ] {
        let text = source(body).replace("return index + total + checksum;", "return temp;");
        let error = parse_nuis_module(&text)
            .err()
            .expect("loop-local variable must be unknown after the loop");
        assert!(error.contains("unknown value `temp`"), "{error}");
    }
    let text = source("let temp = index; if temp < bound { let checksum: i64 = checksum + temp; } else { let total: i64 = total + checksum; }");
    let module = parse_nuis_module(&text).unwrap();
    assert!(
        !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
            .contains_key("walk")
    );
}

#[test]
fn temporary_catalog_rechecks_forged_nir_initializers_and_depth() {
    for depth in [0, 64, 65] {
        let mut module = parse_nuis_module(&source(
            "let temp: i64 = index; let total: i64 = total + temp;",
        ))
        .unwrap();
        let walk = module
            .functions
            .iter_mut()
            .find(|f| f.name == "walk")
            .unwrap();
        let body = walk
            .body
            .iter_mut()
            .find_map(|stmt| {
                if let NirStmt::While { body, .. } = stmt {
                    Some(body)
                } else {
                    None
                }
            })
            .unwrap();
        let NirStmt::Let { value, .. } = &mut body[1] else {
            panic!("temporary")
        };
        for _ in 0..depth {
            *value = NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(value.clone()),
                rhs: Box::new(NirExpr::Int(1)),
            };
        }
        let catalog =
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert_eq!(catalog.contains_key("walk"), depth <= 64, "depth {depth}");
        let walk = module
            .functions
            .iter_mut()
            .find(|f| f.name == "walk")
            .unwrap();
        let body = walk
            .body
            .iter_mut()
            .find_map(|stmt| {
                if let NirStmt::While { body, .. } = stmt {
                    Some(body)
                } else {
                    None
                }
            })
            .unwrap();
        let NirStmt::Let { value, .. } = &mut body[1] else {
            panic!("temporary")
        };
        *value = NirExpr::Var("not_initialized".to_owned());
        assert!(
            !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("walk")
        );
    }
}
