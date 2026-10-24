use super::*;

fn function(source: &str) -> NirFunction {
    crate::frontend::parse_nuis_module(&format!("mod cpu Main {{ {source} }}"))
        .unwrap()
        .functions
        .remove(0)
}

#[test]
fn source_join_normalization_is_idempotent_and_keeps_strict_write_admission() {
    let mut module = crate::frontend::parse_nuis_module(
        "mod cpu Main {
        fn choose(gate: bool, a: i64, b: i64) -> i64 {
            print(7); let x: i64 = if gate { a } else { b }; return x;
        }
    }",
    )
    .unwrap();
    let layouts = control_values::layouts(&module);
    assert!(!scalar_helpers::collect_iteration_calls(&module, &layouts).contains_key("choose"));
    let values = control_values::TypedLayouts::collect(&module);
    let catalog = scalar_helpers::collect_capture_values(&module, &values);
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated =
        conditional_values::outline_declarations(&mut module, &catalog, &values, &mut names);
    assert_eq!(generated.len(), 1);
    assert!(scalar_helpers::collect_iteration_calls(&module, &layouts).contains_key("choose"));
    assert!(!scalar_helpers::collect_with_layouts(&module, &layouts).contains_key("choose"));
    let before = module.clone();
    assert!(
        conditional_values::outline_declarations(&mut module, &catalog, &values, &mut names)
            .is_empty()
    );
    assert_eq!(module, before);
    let mut denied = module.clone();
    denied.functions.retain(|f| !generated.contains(&f.name));
    assert!(!scalar_helpers::collect_iteration_calls(&denied, &layouts).contains_key("choose"));
}

#[test]
fn fresh_typed_joins_share_ready_value_proof_without_mutating_source() {
    for (kind, yes, no) in [
        ("i64", "a", "b"),
        ("i32", "a", "b"),
        ("bool", "a", "false"),
        ("f32", "a", "b"),
        ("f64", "a", "b"),
    ] {
        let f = function(&format!("fn choose(gate: bool, a: {kind}, b: {kind}) -> {kind} {{ let x: {kind} = if gate {{ {yes} }} else {{ {no} }}; return x; }}"));
        let before = f.clone();
        let scope = f
            .params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        let join = declaration(&f.body[0], &scope, &f.body[1..]).unwrap();
        assert_eq!(join.name, "x");
        assert_eq!(join.ty, &scalar_type(kind));
        assert_eq!(f, before);
    }
}

#[test]
fn typed_joins_preserve_selected_raw_scalar_words_in_reversed_yir() {
    let manifest =
        crate::registry::load_manifest(std::path::Path::new("nustar-packages"), "official.cpu")
            .unwrap();
    for kind in ["bool", "i32", "i64", "f32", "f64"] {
        let source = format!(
            "mod cpu Main {{
            @noinline fn choose(gate: bool, yes: {kind}, no: {kind}) -> {kind} {{
                let selected: {kind} = if gate {{ yes }} else {{ no }};
                return selected;
            }}
            fn main() -> i64 {{ return 0; }}
        }}"
        );
        let pairs: &[(i64, i64)] = match kind {
            "bool" => &[(0, 1), (1, 0)],
            "i32" => &[(i32::MIN as i64, i32::MAX as i64), (-1, 0)],
            "i64" => &[(i64::MIN, i64::MAX), (-1, 0)],
            "f32" => &[(0, 0x80000000), (1, 0x807fffff), (0x7fc00123, 0xffc45678)],
            "f64" => &[
                (0, i64::MIN),
                (1, 0x800fffffffffffffu64 as i64),
                (0x7ff8000000000123, 0xfff8000000000456u64 as i64),
            ],
            _ => unreachable!(),
        };
        let encode = |value: NirExpr| match kind {
            "bool" => NirExpr::CastBoolToI64(Box::new(value)),
            "i32" => NirExpr::CastI32ToI64(Box::new(value)),
            "f32" => NirExpr::PackF32Word(Box::new(value)),
            "f64" => NirExpr::PackF64Word(Box::new(value)),
            _ => value,
        };
        let decode = |word| match kind {
            "bool" => NirExpr::Bool(word != 0),
            "i32" => NirExpr::CastI64ToI32(Box::new(NirExpr::Int(word))),
            "f32" => NirExpr::UnpackF32Word(Box::new(NirExpr::Int(word))),
            "f64" => NirExpr::UnpackF64Word(Box::new(NirExpr::Int(word))),
            _ => NirExpr::Int(word),
        };
        for &(yes, no) in pairs {
            for gate in [false, true] {
                let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
                let main = module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "main")
                    .unwrap();
                main.body = vec![NirStmt::Return(Some(encode(NirExpr::Call {
                    callee: "choose".into(),
                    args: vec![NirExpr::Bool(gate), decode(yes), decode(no)],
                })))];
                let yir = crate::lowering::lower_nir_to_yir(&module, &manifest, None).unwrap();
                for reversed in [false, true] {
                    let mut yir = yir.clone();
                    if reversed {
                        yir.functions.reverse();
                        yir.nodes.reverse();
                        for f in &mut yir.functions {
                            f.body_nodes.reverse();
                        }
                    }
                    let result = yir
                        .functions
                        .iter()
                        .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                        .unwrap()
                        .result
                        .as_ref()
                        .unwrap()
                        .node
                        .clone();
                    let trace = yir_runtime_host::execute_module_source_with_registry(
                        &crate::render::render_yir(&yir),
                        &yir_verify::default_registry(),
                    )
                    .unwrap();
                    assert_eq!(
                        trace.values[&result],
                        yir_core::Value::Int(if gate { yes } else { no }),
                        "{kind}"
                    );
                }
            }
        }
    }
}

#[test]
fn joins_reject_outer_writes_checks_calls_and_incomplete_branches() {
    let f = function("fn choose(gate: bool, a: i64, b: i64) -> i64 { let x: i64 = if gate { a } else { b }; return x; }");
    let scope = f
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect::<Scope>();
    for mutation in 0..8 {
        let mut stmt = f.body[0].clone();
        let mut scope = scope.clone();
        let NirStmt::If {
            condition,
            then_body,
            else_body,
        } = &mut stmt
        else {
            panic!()
        };
        match mutation {
            0 => {
                scope.insert("x".into(), scalar_type("i64"));
            }
            1 => else_body.clear(),
            2 => then_body.push(NirStmt::Print(NirExpr::Int(7))),
            3 => {
                *condition = NirExpr::Call {
                    callee: "gate".into(),
                    args: vec![],
                }
            }
            _ => {
                let NirStmt::Let { name, ty, value } = &mut then_body[0] else {
                    panic!()
                };
                match mutation {
                    4 => *name = "different".into(),
                    5 => *ty = None,
                    6 => {
                        *value = NirExpr::Call {
                            callee: "leaf".into(),
                            args: vec![],
                        }
                    }
                    7 => {
                        *value = NirExpr::Binary {
                            op: NirBinaryOp::Div,
                            lhs: Box::new(NirExpr::Int(0)),
                            rhs: Box::new(NirExpr::Var("a".into())),
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
        assert!(
            declaration(&stmt, &scope, &f.body[1..]).is_none(),
            "{mutation}"
        );
    }
}

#[test]
fn branch_locals_are_not_exported_through_later_or_nested_redeclarations() {
    let f = function("fn choose(gate: bool, a: i64, b: i64) -> i64 { let x: i64 = if gate { a } else { b }; return x; }");
    let scope = f
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    let define = NirStmt::Let {
        name: "x".into(),
        ty: Some(scalar_type("i64")),
        value: NirExpr::Int(7),
    };
    let returned = NirStmt::Return(Some(NirExpr::Var("x".into())));
    assert!(declaration(&f.body[0], &scope, &[define.clone(), returned.clone()]).is_none());
    let nested = NirStmt::If {
        condition: NirExpr::Bool(true),
        then_body: vec![define.clone(), returned.clone()],
        else_body: vec![define, returned.clone()],
    };
    assert!(declaration(&f.body[0], &scope, &[nested]).is_none());
    assert!(declaration(
        &f.body[0],
        &scope,
        &[NirStmt::Print(NirExpr::Int(9)), returned]
    )
    .is_some());
    let mut deep = NirExpr::Var("x".into());
    for _ in 0..64 {
        deep = NirExpr::CastI32ToI64(Box::new(deep));
    }
    assert!(declaration(&f.body[0], &scope, &[NirStmt::Return(Some(deep))]).is_none());
    let mut large = vec![NirStmt::Print(NirExpr::Int(0)); 4096];
    large.push(NirStmt::Return(Some(NirExpr::Var("x".into()))));
    assert!(declaration(&f.body[0], &scope, &large).is_none());
    let mut args = vec![NirExpr::Int(0); 4096];
    args.push(NirExpr::Var("x".into()));
    let wide = NirStmt::Return(Some(NirExpr::Call {
        callee: "wide".into(),
        args,
    }));
    assert!(declaration(&f.body[0], &scope, &[wide]).is_none());
}
