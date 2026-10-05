use super::*;

#[path = "capture_caller_spills_native_tests.rs"]
mod native;

#[path = "capture_caller_spills_return_tests.rs"]
mod returns;

#[path = "capture_caller_spills_condition_tests.rs"]
mod conditions;

#[path = "capture_caller_spills_gated_tests.rs"]
mod gated;

const RECORDS: &str = "struct Pair { x: i64, y: i64 }
    struct State { a: Pair, b: Pair, unused: i64 }";
const CONSTRUCTOR: &str = "State { unused: checked(divisor), b: state.b, a: state.a }";

fn module(argument: &str, body: &str) -> NirModule {
    crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{ {RECORDS}
        @noinline fn checked(value: i64) -> i64 {{ return 10 / value; }}
        @noinline fn produce(state: State, divisor: i64) -> State {{ return {CONSTRUCTOR}; }}
        @noinline fn helper(before: i64, state: State, after: i64, flag: bool) -> i64 {{
            if flag {{ return before + state.a.x + after; }} return before + state.b.y + after;
        }} fn entry(state: State, divisor: i64, flag: bool) -> i64 {{ {} }} }}",
        body.replace("ARGUMENT", argument)
    ))
    .unwrap()
}

const BODY: &str =
    "let result = helper(checked(divisor), ARGUMENT, 20 / divisor, flag); return result;";

fn function<'a>(module: &'a NirModule, name: &str) -> &'a NirFunction {
    module.functions.iter().find(|f| f.name == name).unwrap()
}

fn function_mut<'a>(module: &'a mut NirModule, name: &str) -> &'a mut NirFunction {
    module
        .functions
        .iter_mut()
        .find(|f| f.name == name)
        .unwrap()
}

fn run(module: &mut NirModule) -> bool {
    let layouts = control_values::TypedLayouts::collect(module);
    super::super::project(
        module,
        &BTreeSet::from(["helper".into()]),
        &BTreeSet::new(),
        &layouts,
        &BTreeMap::new(),
    )
    .changed
}

fn original_args(module: &NirModule) -> Vec<NirExpr> {
    let NirStmt::Let {
        value: NirExpr::Call { args, .. },
        ..
    } = &function(module, "entry").body[0]
    else {
        panic!()
    };
    args.clone()
}

#[test]
fn caller_spills_preserve_complete_operands_in_exact_source_order_and_independent_bindings() {
    for argument in ["produce(state, divisor)", CONSTRUCTOR] {
        for constant in [false, true] {
            for reverse in [false, true] {
                let body = if constant {
                    BODY.replace("let result =", "const result: i64 =")
                } else {
                    BODY.into()
                };
                let mut module = module(argument, &body);
                let original = match &function(&module, "entry").body[0] {
                    NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => {
                        let NirExpr::Call { args, .. } = value else {
                            panic!()
                        };
                        args.clone()
                    }
                    _ => panic!(),
                };
                if reverse {
                    module.functions.reverse();
                    module.structs.reverse();
                }
                assert!(run(&mut module));
                crate::nir_verify::verify_nir_module(&module).unwrap();
                assert_eq!(function(&module, "helper").params.len(), 5);
                let body = &function(&module, "entry").body;
                assert_eq!(body.len(), 6);
                let mut names = Vec::new();
                for ((stmt, before), kind) in body[..4]
                    .iter()
                    .zip(&original)
                    .zip(["i64", "State", "i64", "bool"])
                {
                    let NirStmt::Let { name, ty, value } = stmt else {
                        panic!()
                    };
                    assert_eq!(value, before);
                    assert_eq!(ty.as_ref().unwrap(), &scalar_type(kind));
                    names.push(name);
                }
                let value = match &body[4] {
                    NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => value,
                    _ => panic!(),
                };
                let NirExpr::Call { args, .. } = value else {
                    panic!()
                };
                assert_eq!(args[0], NirExpr::Var(names[0].clone()));
                assert_eq!(access(&args[1]).unwrap(), [names[1].as_str(), "a", "x"]);
                assert_eq!(access(&args[2]).unwrap(), [names[1].as_str(), "b", "y"]);
                assert_eq!(args[3], NirExpr::Var(names[2].clone()));
                assert_eq!(args[4], NirExpr::Var(names[3].clone()));
                assert_eq!(
                    function(&module, "produce").return_type,
                    Some(scalar_type("State"))
                );
                let before = module.clone();
                assert!(!run(&mut module));
                assert_eq!(module, before);
            }
        }
    }
    let mut module = module("produce(state, divisor)", BODY);
    let helper = function_mut(&mut module, "helper");
    helper.params.push(NirParam {
        name: "other".into(),
        ty: scalar_type("State"),
    });
    let field = |root: &str, parent: &str, leaf: &str| NirExpr::FieldAccess {
        base: Box::new(NirExpr::FieldAccess {
            base: Box::new(NirExpr::Var(root.into())),
            field: parent.into(),
        }),
        field: leaf.into(),
    };
    helper.body = vec![NirStmt::Return(Some(NirExpr::Binary {
        op: nuis_semantics::model::NirBinaryOp::Add,
        lhs: Box::new(field("state", "a", "x")),
        rhs: Box::new(field("other", "b", "y")),
    }))];
    let NirStmt::Let {
        value: NirExpr::Call { args, .. },
        ..
    } = &mut function_mut(&mut module, "entry").body[0]
    else {
        panic!()
    };
    args.push(args[1].clone());
    assert!(run(&mut module));
    crate::nir_verify::verify_nir_module(&module).unwrap();
    let body = &function(&module, "entry").body;
    let NirStmt::Let {
        name: first,
        value: first_call,
        ..
    } = &body[1]
    else {
        panic!()
    };
    let NirStmt::Let {
        name: second,
        value: second_call,
        ..
    } = &body[4]
    else {
        panic!()
    };
    assert_eq!(first_call, second_call);
    assert_ne!(first, second);
    let NirStmt::Let {
        value: NirExpr::Call { args, .. },
        ..
    } = &body[5]
    else {
        panic!()
    };
    assert_eq!(access(&args[1]).unwrap(), [first.as_str(), "a", "x"]);
    assert_eq!(access(&args[4]).unwrap(), [second.as_str(), "b", "y"]);
}

#[test]
fn caller_spills_retain_unused_checked_arguments_and_exact_subrecord_kinds() {
    let mut module = module("produce(state, divisor)", BODY);
    function_mut(&mut module, "helper").body =
        vec![NirStmt::Return(Some(NirExpr::Var("flag".into())))];
    function_mut(&mut module, "helper").return_type = Some(scalar_type("bool"));
    function_mut(&mut module, "entry").return_type = Some(scalar_type("bool"));
    let original = original_args(&module);
    assert!(run(&mut module));
    crate::nir_verify::verify_nir_module(&module).unwrap();
    assert_eq!(function(&module, "helper").params.len(), 3);
    for (stmt, value) in function(&module, "entry").body[..4].iter().zip(original) {
        assert!(matches!(stmt, NirStmt::Let { value: stored, .. } if *stored == value));
    }
    let source = format!(
        "mod cpu Main {{ {RECORDS}
        fn consume(pair: Pair) -> i64 {{ return pair.x; }}
        fn make(state: State) -> State {{ return state; }}
        fn helper(state: State) -> i64 {{ return consume(state.a); }}
        fn entry(state: State) -> i64 {{ let result = helper(make(state)); return result; }} }}"
    );
    let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
    assert!(run(&mut module));
    crate::nir_verify::verify_nir_module(&module).unwrap();
    assert_eq!(
        function(&module, "helper").params[0].ty,
        scalar_type("Pair")
    );
}

#[test]
fn caller_spills_roll_back_every_caller_if_any_site_or_catalog_is_unproven() {
    for reverse in [false, true] {
        let mut module = module("produce(state, divisor)", BODY);
        let mut bad = function(&module, "entry").clone();
        bad.name = "unproven".into();
        let NirStmt::Let { value, .. } = &bad.body[0] else {
            panic!()
        };
        bad.body = vec![NirStmt::Return(Some(NirExpr::Binary {
            op: nuis_semantics::model::NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Int(1)),
            rhs: Box::new(value.clone()),
        }))];
        module.functions.push(bad);
        if reverse {
            module.functions.reverse();
        }
        let before = module.clone();
        assert!(!run(&mut module));
        assert_eq!(module, before);
    }
    for mutation in 0..8 {
        let mut module = module("produce(state, divisor)", BODY);
        let producer = function_mut(&mut module, "produce");
        match mutation {
            0 => producer.is_async = true,
            1 => producer.return_type.as_mut().unwrap().is_ref = true,
            2 => producer.return_type.as_mut().unwrap().is_optional = true,
            3 => producer.body.insert(0, NirStmt::Print(NirExpr::Int(1))),
            4 => {
                producer.body = vec![NirStmt::Return(Some(NirExpr::Call {
                    callee: "produce".into(),
                    args: vec![NirExpr::Var("state".into()), NirExpr::Var("divisor".into())],
                }))]
            }
            5 => function_mut(&mut module, "checked")
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(1))),
            6 => {
                function_mut(&mut module, "entry").params[0].ty.is_ref = true;
            }
            7 => function_mut(&mut module, "entry").body.push(NirStmt::Let {
                name: "state".into(),
                ty: None,
                value: NirExpr::Var("state".into()),
            }),
            _ => unreachable!(),
        }
        let before = module.clone();
        assert!(!run(&mut module), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
}

#[test]
fn caller_spills_keep_mixed_kinds_branch_locality_and_exact_constructor_fields() {
    let source = "mod cpu Main {
        struct Mixed { value: f64, gain: f32, tag: i32, enabled: bool, unused: i64 }
        struct TwinMixed { value: f64, gain: f32, tag: i32, enabled: bool, unused: i64 }
        fn checked(value: i64) -> i64 { return 10 / value; }
        fn helper(before: f32, packet: Mixed, after: i32) -> f64 { return packet.value; }
        fn entry(packet: Mixed, divisor: i64, flag: bool) -> f64 {
            if flag {
                const result: f64 = helper(packet.gain, Mixed {
                    unused: checked(divisor), enabled: packet.enabled, tag: packet.tag,
                    gain: packet.gain, value: packet.value }, packet.tag);
                return result;
            } return packet.value;
        } }";
    let original = crate::frontend::parse_nuis_module(source).unwrap();
    let mut module = original.clone();
    assert!(run(&mut module));
    crate::nir_verify::verify_nir_module(&module).unwrap();
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = &function(&module, "entry").body[0]
    else {
        panic!()
    };
    assert_eq!(condition, &NirExpr::Var("flag".into()));
    assert!(else_body.is_empty());
    assert_eq!(then_body.len(), 5);
    for (stmt, kind) in then_body[..3].iter().zip(["f32", "Mixed", "i32"]) {
        assert!(matches!(stmt, NirStmt::Let { ty: Some(ty), .. } if *ty == scalar_type(kind)));
    }
    assert_eq!(function(&module, "helper").params[1].ty, scalar_type("f64"));
    for mutation in 0..6 {
        let mut module = original.clone();
        let NirStmt::If { then_body, .. } = &mut function_mut(&mut module, "entry").body[0] else {
            panic!()
        };
        let NirStmt::Const {
            value: NirExpr::Call { args, .. },
            ..
        } = &mut then_body[0]
        else {
            panic!()
        };
        let NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } = &mut args[1]
        else {
            panic!()
        };
        match mutation {
            0 => *type_name = "Missing".into(),
            1 => type_args.push(scalar_type("i64")),
            2 => {
                fields.pop();
            }
            3 => fields[0].0 = fields[1].0.clone(),
            4 => fields[1].1 = NirExpr::Int(1),
            5 => *type_name = "TwinMixed".into(),
            _ => unreachable!(),
        }
        let before = module.clone();
        assert!(!run(&mut module), "{mutation}");
        assert_eq!(module, before);
    }
}

#[test]
fn caller_spills_keep_nested_expressions_conditions_and_loop_returns_conservative() {
    for body in [
        "let result = 1 + helper(checked(divisor), ARGUMENT, 20 / divisor, flag); return result;",
        "return 1 + helper(checked(divisor), ARGUMENT, 20 / divisor, flag);",
        "if helper(checked(divisor), ARGUMENT, 20 / divisor, flag) > 0 { return 1; } return 0;",
        "let i = 0; while i < 1 { let result = helper(checked(divisor), ARGUMENT, 20 / divisor, flag); let i = i + 1; } return i;",
        "while flag { return helper(checked(divisor), ARGUMENT, 20 / divisor, flag); } return 0;",
    ] {
        let mut module = module("produce(state, divisor)", body);
        let before = module.clone();
        assert!(!run(&mut module), "{body}");
        assert_eq!(module, before);
    }
    let mut module = module("produce(state, divisor)", BODY);
    let NirStmt::Let {
        value: NirExpr::Call { args, .. },
        ..
    } = &mut function_mut(&mut module, "entry").body[0]
    else {
        panic!()
    };
    args[1] = NirExpr::FieldAccess {
        base: Box::new(args[1].clone()),
        field: "a".into(),
    };
    let before = module.clone();
    assert!(!run(&mut module));
    assert_eq!(module, before);
}

#[test]
fn caller_spills_bound_work_depth_names_and_registered_controls_before_installation() {
    let mut module = module("produce(state, divisor)", BODY);
    function_mut(&mut module, "entry").params.push(NirParam {
        name: "__nuis_caller_operand_0".into(),
        ty: scalar_type("i64"),
    });
    let layouts = control_values::TypedLayouts::collect(&module);
    let mut catalog = scalar_helpers::collect_capture_values(&module, &layouts);
    catalog.remove("helper");
    let plan = super::super::plan(function(&module, "helper"), None, &layouts).unwrap();
    let caller = function(&module, "entry");
    let mut accepted = 0;
    let mut rejected = 0;
    for budget in 1..2048 {
        if prepare_with_budget(
            caller,
            "helper",
            &plan,
            &layouts,
            &catalog,
            &BTreeSet::new(),
            budget,
        )
        .is_some()
        {
            accepted += 1;
        } else {
            rejected += 1;
        }
    }
    assert!(accepted > 0 && rejected > 0);
    assert!(prepare(
        caller,
        "helper",
        &plan,
        &layouts,
        &catalog,
        &BTreeSet::from(["state".into()])
    )
    .is_none());
    let mut deep = caller.clone();
    for _ in 0..70 {
        deep.body = vec![NirStmt::If {
            condition: NirExpr::Var("flag".into()),
            then_body: deep.body,
            else_body: vec![],
        }];
    }
    assert!(prepare(&deep, "helper", &plan, &layouts, &catalog, &BTreeSet::new()).is_none());
    assert!(run(&mut module));
    crate::nir_verify::verify_nir_module(&module).unwrap();
    let NirStmt::Let { name, .. } = &function(&module, "entry").body[0] else {
        panic!()
    };
    assert_ne!(name, "__nuis_caller_operand_0");
}

#[test]
fn caller_spills_preserve_guarded_unused_checks_with_an_independent_oracle() {
    let execute = |module: &NirModule| -> Result<yir_core::Value, ()> {
        let mut yir = crate::lowering::lower_nir_to_yir_builtin_cpu(module).unwrap();
        yir.nodes.reverse();
        yir.functions.reverse();
        for function in &mut yir.functions {
            function.body_nodes.reverse();
        }
        let trace = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&yir),
            &yir_verify::default_registry(),
        )
        .map_err(|_| ())?;
        let main = yir
            .functions
            .iter()
            .find(|f| f.role == yir_core::YirFunctionRole::Entry)
            .unwrap();
        Ok(trace.values[&main.result.as_ref().unwrap().node].clone())
    };
    for first in 0..4 {
        for unused in 0..4 {
            for guard in 0..3 {
                let mut module = module("produce(state, divisor)",
                    "if divisor == 0 { return 0; } let result = helper(checked(divisor), ARGUMENT, 20 / divisor, flag); return result;");
                let entry = function_mut(&mut module, "entry");
                entry.params.extend([
                    NirParam {
                        name: "unused".into(),
                        ty: scalar_type("i64"),
                    },
                    NirParam {
                        name: "guard".into(),
                        ty: scalar_type("i64"),
                    },
                ]);
                let NirStmt::If { condition, .. } = &mut entry.body[0] else {
                    panic!()
                };
                *condition = NirExpr::Binary {
                    op: nuis_semantics::model::NirBinaryOp::Eq,
                    lhs: Box::new(NirExpr::Var("guard".into())),
                    rhs: Box::new(NirExpr::Int(0)),
                };
                let NirStmt::Let {
                    value: NirExpr::Call { args, .. },
                    ..
                } = &mut entry.body[1]
                else {
                    panic!()
                };
                args[1] = NirExpr::Call {
                    callee: "produce".into(),
                    args: vec![NirExpr::Var("state".into()), NirExpr::Var("unused".into())],
                };
                let mut main = function(&module, "entry").clone();
                main.name = "main".into();
                main.params.clear();
                main.body = vec![NirStmt::Return(Some(NirExpr::Call {
                    callee: "entry".into(),
                    args: vec![
                        NirExpr::StructLiteral {
                            type_name: "State".into(),
                            type_args: vec![],
                            fields: vec![
                                ("a".into(), pair(11, 13)),
                                ("b".into(), pair(17, 19)),
                                ("unused".into(), NirExpr::Int(23)),
                            ],
                        },
                        NirExpr::Int(first),
                        NirExpr::Bool(guard == 1),
                        NirExpr::Int(unused),
                        NirExpr::Int(guard),
                    ],
                }))];
                module.functions.push(main);
                let expected = if guard == 0 {
                    Ok(yir_core::Value::Int(0))
                } else if first == 0 || unused == 0 {
                    Err(())
                } else {
                    Ok(yir_core::Value::Int(
                        10 / first + 20 / first + if guard == 1 { 11 } else { 19 },
                    ))
                };
                assert_eq!(
                    execute(&module),
                    expected,
                    "before {first}/{unused}/{guard}"
                );
                assert!(run(&mut module));
                crate::nir_verify::verify_nir_module(&module).unwrap();
                assert_eq!(execute(&module), expected, "after {first}/{unused}/{guard}");
            }
        }
    }
}

fn pair(x: i64, y: i64) -> NirExpr {
    NirExpr::StructLiteral {
        type_name: "Pair".into(),
        type_args: vec![],
        fields: vec![("x".into(), NirExpr::Int(x)), ("y".into(), NirExpr::Int(y))],
    }
}
