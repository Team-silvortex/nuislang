use super::*;

#[path = "capture_caller_records_native_tests.rs"]
mod native;

const RECORDS: &str = "struct Pair { x: i64, y: i64 }
    struct State { a: Pair, b: Pair, unused: i64 }";
const ARGUMENT: &str =
    "State { unused: state.unused, b: state.a, a: Pair { y: state.b.y, x: state.b.x } }";

fn module(body: &str) -> NirModule {
    crate::frontend::parse_nuis_module(&format!("mod cpu Main {{ {RECORDS} {body} }}")).unwrap()
}

fn fixture(argument: &str) -> NirModule {
    module(&format!(
        "fn helper(state: State, flag: bool) -> i64 {{
            if flag {{ return state.a.x + state.a.y; }} return state.b.y;
        }} fn entry(state: State, flag: bool) -> i64 {{ return helper({argument}, flag); }}"
    ))
}

fn run(module: &mut NirModule) -> bool {
    let layouts = control_values::TypedLayouts::collect(module);
    project_module(module, &layouts)
}

fn project_module(module: &mut NirModule, layouts: &impl ValueLayouts) -> bool {
    super::super::project(
        module,
        &BTreeSet::from(["helper".into()]),
        &BTreeSet::new(),
        layouts,
        &BTreeMap::new(),
    )
    .changed
}

fn function<'a>(module: &'a NirModule, name: &str) -> &'a NirFunction {
    module.functions.iter().find(|f| f.name == name).unwrap()
}

fn argument_mut(module: &mut NirModule) -> &mut NirExpr {
    let entry = module
        .functions
        .iter_mut()
        .find(|f| f.name == "entry")
        .unwrap();
    let NirStmt::Return(Some(NirExpr::Call { args, .. })) = &mut entry.body[0] else {
        panic!()
    };
    &mut args[0]
}

fn nest_return(module: &mut NirModule) {
    let entry = module
        .functions
        .iter_mut()
        .find(|f| f.name == "entry")
        .unwrap();
    let NirStmt::Return(Some(value)) = &mut entry.body[0] else {
        panic!()
    };
    *value = NirExpr::Binary {
        op: nuis_semantics::model::NirBinaryOp::Add,
        lhs: Box::new(NirExpr::Int(0)),
        rhs: Box::new(value.clone()),
    };
}

#[test]
fn caller_record_constructors_select_nested_fields_without_copying_whole_constructors() {
    for reverse in [false, true] {
        let mut module = fixture(ARGUMENT);
        if reverse {
            module.functions.reverse();
            module.structs.reverse();
        }
        crate::nir_verify::verify_nir_module(&module).unwrap();
        assert!(run(&mut module));
        crate::nir_verify::verify_nir_module(&module).unwrap();
        assert_eq!(function(&module, "helper").params.len(), 4);
        let NirStmt::Return(Some(NirExpr::Call { args, .. })) = &function(&module, "entry").body[0]
        else {
            panic!()
        };
        for (arg, (record, field)) in args[..3].iter().zip([("b", "x"), ("b", "y"), ("a", "y")]) {
            assert_eq!(access(arg).unwrap(), ["state", record, field]);
        }
        assert_eq!(args[3], NirExpr::Var("flag".into()));
        let before = module.clone();
        assert!(!run(&mut module));
        assert_eq!(module, before);
    }
}

#[test]
fn caller_record_constructors_keep_subrecord_kinds_and_scalar_operand_order() {
    let mut module = module(&format!(
        "fn checked(value: i64) -> i64 {{ return 10 / value; }}
        fn consume(pair: Pair) -> i64 {{ return pair.x; }}
        fn helper(before: i64, state: State, after: i64) -> i64 {{
            return before + consume(state.a) + after;
        }} fn entry(state: State, divisor: i64) -> i64 {{
            return helper(checked(divisor), {ARGUMENT}, 20 / divisor);
        }}"
    ));
    let original = function(&module, "entry").body.clone();
    assert!(run(&mut module));
    crate::nir_verify::verify_nir_module(&module).unwrap();
    assert_eq!(function(&module, "helper").params[1].ty.name, "Pair");
    let NirStmt::Return(Some(NirExpr::Call { args: original, .. })) = &original[0] else {
        panic!()
    };
    let NirStmt::Return(Some(NirExpr::Call { args, .. })) = &function(&module, "entry").body[0]
    else {
        panic!()
    };
    assert_eq!(args[0], original[0]);
    assert_eq!(args[2], original[2]);
    assert!(matches!(&args[1], NirExpr::StructLiteral { type_name, .. } if type_name == "Pair"));
}

#[test]
fn caller_record_constructors_preserve_exact_mixed_scalar_kinds() {
    let mut module = crate::frontend::parse_nuis_module(
        "mod cpu Main {
        struct Leaf { value: f64, gain: f32, tag: i32, enabled: bool }
        struct Packet { leaf: Leaf, ignored: i64 }
        fn helper(value: Packet) -> f64 { return value.leaf.value; }
        fn entry(value: Packet) -> f64 {
            return helper(Packet { ignored: value.ignored, leaf: Leaf {
                enabled: value.leaf.enabled, tag: value.leaf.tag,
                gain: value.leaf.gain, value: value.leaf.value } });
        } }",
    )
    .unwrap();
    assert!(run(&mut module));
    crate::nir_verify::verify_nir_module(&module).unwrap();
    assert_eq!(function(&module, "helper").params[0].ty.name, "f64");
}

#[test]
fn caller_record_constructors_never_drop_computed_unused_fields_or_inline_calls() {
    for argument in [
        ARGUMENT.replace("state.unused", "10 / state.unused"),
        ARGUMENT.replace("state.unused", "checked(state.unused)"),
        ARGUMENT.replace("state.b.x", "checked(state.b.x)"),
        "computed(state)".into(),
        "wrapped(state).payload".into(),
    ] {
        let mut module = module(&format!(
            "struct Envelope {{ payload: State }}
            fn checked(value: i64) -> i64 {{ return 10 / value; }}
            fn computed(state: State) -> State {{ return state; }}
            fn wrapped(state: State) -> Envelope {{ return Envelope {{ payload: state }}; }}
            fn helper(state: State, flag: bool) -> i64 {{
                if flag {{ return state.a.x + state.a.y; }} return state.b.y;
            }} fn entry(state: State, flag: bool) -> i64 {{ return 0 + helper({argument}, flag); }}"
        ));
        let before = module.clone();
        assert!(!run(&mut module), "{argument}");
        assert_eq!(module, before, "{argument}");
    }
    let mut codec = fixture(ARGUMENT);
    let NirExpr::StructLiteral { fields, .. } = argument_mut(&mut codec) else {
        panic!()
    };
    fields[0].1 = NirExpr::CastI32ToI64(Box::new(NirExpr::CastI64ToI32(Box::new(
        fields[0].1.clone(),
    ))));
    // A separate root-return proof preserves the complete codec expression;
    // total constructor projection alone still cannot elide it inline.
    let original = argument_mut(&mut codec).clone();
    let mut stored = codec.clone();
    assert!(run(&mut stored));
    crate::nir_verify::verify_nir_module(&stored).unwrap();
    assert!(
        matches!(&function(&stored, "entry").body[0], NirStmt::Let { value, .. } if *value == original)
    );
    nest_return(&mut codec);
    let before = codec.clone();
    assert!(!run(&mut codec));
    assert_eq!(codec, before);
    let mut module = module(&format!(
        "fn helper(state: State, flag: bool) -> bool {{ return flag; }}
        fn entry(state: State, flag: bool) -> bool {{ return helper({ARGUMENT}, flag); }}"
    ));
    assert!(run(&mut module));
    crate::nir_verify::verify_nir_module(&module).unwrap();
    assert_eq!(function(&module, "helper").params.len(), 1);
    // A new unobserved aggregate argument still needs the complete proof.
    let mut bad = fixture(&ARGUMENT.replace("state.unused", "10 / state.unused"));
    bad.functions
        .iter_mut()
        .find(|f| f.name == "helper")
        .unwrap()
        .body = vec![NirStmt::Return(Some(NirExpr::Int(7)))];
    nest_return(&mut bad);
    let before = bad.clone();
    assert!(!run(&mut bad));
    assert_eq!(bad, before);
}

#[test]
fn caller_record_constructors_require_nominal_fields_kinds_and_immutable_input_paths() {
    for mutation in 0..10 {
        let mut module = fixture(ARGUMENT);
        let NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } = argument_mut(&mut module)
        else {
            panic!()
        };
        match mutation {
            0 => *type_name = "Pair".into(),
            1 => type_args.push(scalar_type("i64")),
            2 => {
                fields.pop();
            }
            3 => fields[0].0 = "missing".into(),
            4 => fields[0].0 = fields[1].0.clone(),
            5 => fields[0].1 = NirExpr::Bool(true),
            6 => fields[0].1 = NirExpr::Var("local".into()),
            7 => {
                module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "entry")
                    .unwrap()
                    .body
                    .insert(
                        0,
                        NirStmt::Let {
                            name: "state".into(),
                            ty: None,
                            value: NirExpr::Var("state".into()),
                        },
                    );
            }
            8 => {
                module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "entry")
                    .unwrap()
                    .params[0]
                    .ty
                    .is_ref = true;
            }
            9 => {
                module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "entry")
                    .unwrap()
                    .body
                    .push(NirStmt::If {
                        condition: NirExpr::Var("flag".into()),
                        then_body: vec![NirStmt::Let {
                            name: "state".into(),
                            ty: None,
                            value: NirExpr::Var("state".into()),
                        }],
                        else_body: vec![],
                    });
            }
            _ => unreachable!(),
        }
        let before = module.clone();
        assert!(!run(&mut module), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
}

#[test]
fn caller_record_constructors_require_every_caller_before_rewriting_any_signature() {
    for bad in [false, true] {
        for reverse in [false, true] {
            let mut module = fixture(ARGUMENT);
            let mut second = function(&module, "entry").clone();
            second.name = "second".into();
            if bad {
                let NirStmt::Return(Some(NirExpr::Call { args, .. })) = &mut second.body[0] else {
                    panic!()
                };
                args[0] = NirExpr::Call {
                    callee: "computed".into(),
                    args: vec![NirExpr::Var("state".into())],
                };
            }
            module.functions.push(second);
            if reverse {
                module.functions.reverse();
            }
            let before = module.clone();
            assert_eq!(run(&mut module), !bad);
            if bad {
                assert_eq!(module, before);
            } else {
                crate::nir_verify::verify_nir_module(&module).unwrap();
            }
        }
    }
}

#[test]
fn caller_record_constructors_keep_bounded_work_and_depth() {
    let mut module = fixture(ARGUMENT);
    let layouts = control_values::TypedLayouts::collect(&module);
    let plan = super::super::plan(function(&module, "helper"), None, &layouts).unwrap();
    let caller = function(&module, "entry");
    let mut accepted = 0;
    let mut rejected = 0;
    for budget in 1..1024 {
        if valid_with_budget(caller, "helper", &plan, &layouts, budget).is_some() {
            accepted += 1;
        } else {
            rejected += 1;
        }
    }
    assert!(accepted > 0 && rejected > 0);
    let mut deep = caller.clone();
    for _ in 0..70 {
        deep.body = vec![NirStmt::If {
            condition: NirExpr::Var("flag".into()),
            then_body: deep.body,
            else_body: vec![],
        }];
    }
    assert!(valid_with_budget(&deep, "helper", &plan, &layouts, 65_536).is_none());
    let NirExpr::StructLiteral { fields, .. } = argument_mut(&mut module) else {
        panic!()
    };
    for _ in 0..70 {
        fields[0].1 = NirExpr::FieldAccess {
            base: Box::new(fields[0].1.clone()),
            field: "x".into(),
        };
    }
    let before = module.clone();
    assert!(!project_module(&mut module, &layouts));
    assert_eq!(module, before);
}

#[test]
fn caller_record_constructors_preserve_kept_checks_and_guarded_timing_with_an_oracle() {
    let execute = |module: &NirModule| -> Result<yir_core::Value, ()> {
        let mut yir = crate::lowering::lower_nir_to_yir_builtin_cpu(module).unwrap();
        yir.nodes.reverse();
        yir.functions.reverse();
        for function in &mut yir.functions {
            function.body_nodes.reverse();
        }
        let entry = yir
            .functions
            .iter()
            .find(|f| f.role == yir_core::YirFunctionRole::Entry)
            .unwrap();
        let trace = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&yir),
            &yir_verify::default_registry(),
        )
        .map_err(|_| ())?;
        Ok(trace.values[&entry.result.as_ref().unwrap().node].clone())
    };
    for first in 0..4 {
        for last in 0..4 {
            for guard in 0..3 {
                let mut module = module(&format!(
                    "@noinline fn checked(value: i64, divisor: i64) -> i64 {{ return value / divisor; }}
                    @noinline fn helper(before: i64, state: State, after: i64, flag: bool) -> i64 {{
                        if flag {{ return before + state.a.x + after; }} return before + state.b.y + after;
                    }} fn entry(state: State, first: i64, last: i64, guard: i64) -> i64 {{
                        if guard == 0 {{ return 0; }}
                        return helper(checked(10, first), {ARGUMENT}, checked(20, last), guard == 1);
                    }} fn main() -> i64 {{ return entry(State {{ a: Pair {{ x: 11, y: 13 }}, b: Pair {{ x: 17, y: 19 }}, unused: 23 }}, {first}, {last}, {guard}); }}"
                ));
                let expected = if guard == 0 {
                    Ok(yir_core::Value::Int(0))
                } else if first == 0 || last == 0 {
                    Err(())
                } else {
                    Ok(yir_core::Value::Int(
                        10 / first + 20 / last + if guard == 1 { 17 } else { 13 },
                    ))
                };
                assert_eq!(execute(&module), expected, "before {first}/{last}/{guard}");
                assert!(run(&mut module));
                crate::nir_verify::verify_nir_module(&module).unwrap();
                assert_eq!(execute(&module), expected, "after {first}/{last}/{guard}");
            }
        }
    }
}
