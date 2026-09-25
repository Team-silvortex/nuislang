use super::*;

fn field(name: &str, field: &str) -> NirExpr {
    NirExpr::FieldAccess {
        base: Box::new(NirExpr::Var(name.into())),
        field: field.into(),
    }
}

fn scalar(name: &str) -> NirParam {
    NirParam {
        name: name.into(),
        ty: ty("i64"),
    }
}

#[test]
fn carried_field_seeds_follow_result_layout_not_parameter_order() {
    let module = parse_nuis_module(SOURCE).unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    let body = body();
    let projections = projected_bindings("result", &ty("Slots"), &body, &definitions).unwrap();
    let mut function = module.functions[0].clone();
    function.params = vec![
        scalar("right"),
        scalar("stamp"),
        scalar("left"),
        scalar("sum"),
    ];
    let args = vec![
        field("packet", "right"),
        field("marker", "value"),
        field("packet", "left"),
        NirExpr::Var("total".into()),
    ];
    validate_seeds(&projections, false, &function, &args, "action").unwrap();
    let result = ScopedLoopResult::Scalars {
        separate_seeds: false,
        bindings: projections,
        layout: String::new(),
        breaking: false,
    };
    for ((param, arg), index) in function.params.iter().zip(&args).zip([1, 3, 0, 2]) {
        assert_eq!(argument_index(&result, param, arg), Some((index, 1)));
    }
}

#[test]
fn carried_field_seeds_require_complete_unique_exact_typed_coverage() {
    let module = parse_nuis_module(SOURCE).unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    let body = body();
    let projections = projected_bindings("result", &ty("Slots"), &body, &definitions).unwrap();
    for case in [
        "missing",
        "duplicate",
        "whole_and_field",
        "unknown",
        "root",
        "nested",
        "computed",
        "wrong_type",
        "reference",
        "optional",
        "generic",
        "arity",
    ] {
        let mut function = module.functions[0].clone();
        function.params = vec![
            scalar("right"),
            scalar("stamp"),
            scalar("left"),
            scalar("sum"),
        ];
        let mut args = vec![
            field("packet", "right"),
            field("marker", "value"),
            field("packet", "left"),
            NirExpr::Var("total".into()),
        ];
        match case {
            "missing" => {
                function.params.remove(2);
                args.remove(2);
            }
            "duplicate" => args[2] = args[0].clone(),
            "whole_and_field" => {
                function.params.push(NirParam {
                    name: "whole".into(),
                    ty: ty("Pair"),
                });
                args.push(NirExpr::Var("packet".into()));
            }
            "unknown" => args[2] = field("packet", "missing"),
            "root" => args[2] = field("saved", "left"),
            "nested" => {
                args[2] = NirExpr::FieldAccess {
                    base: Box::new(args[2].clone()),
                    field: "left".into(),
                }
            }
            "computed" => {
                args[2] = NirExpr::Call {
                    callee: "make".into(),
                    args: vec![],
                }
            }
            "wrong_type" => function.params[2].ty = ty("i32"),
            "reference" => function.params[2].ty.is_ref = true,
            "optional" => function.params[2].ty.is_optional = true,
            "generic" => function.params[2].ty.generic_args.push(ty("i64")),
            "arity" => {
                args.pop();
            }
            _ => unreachable!(),
        }
        assert!(
            validate_seeds(&projections, false, &function, &args, "action").is_err(),
            "{case}"
        );
    }
}

#[test]
fn carried_field_seeds_preserve_bool_and_break_word_identities() {
    let module = parse_nuis_module(SOURCE).unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    let body = vec![
        record("packet", "Pair", &[("left", 0), ("right", 1)]),
        NirStmt::Let {
            name: "flag".into(),
            ty: Some(ty("bool")),
            value: NirExpr::CastI64ToBool(Box::new(word(2))),
        },
        NirStmt::Let {
            name: "stop".into(),
            ty: Some(ty("i64")),
            value: word(3),
        },
    ];
    let projections = projected_bindings("result", &ty("Slots"), &body, &definitions).unwrap();
    let mut function = module.functions[0].clone();
    function.params = vec![
        scalar("right"),
        scalar("word"),
        scalar("left"),
        scalar("stop"),
    ];
    let args = vec![
        field("packet", "right"),
        NirExpr::CastBoolToI64(Box::new(NirExpr::Var("flag".into()))),
        field("packet", "left"),
        NirExpr::Int(0),
    ];
    validate_seeds(&projections, true, &function, &args, "action").unwrap();
    let result = ScopedLoopResult::Scalars {
        separate_seeds: false,
        bindings: projections,
        layout: String::new(),
        breaking: true,
    };
    for ((param, arg), index) in function.params.iter().zip(&args).zip([1, 2, 0, 3]) {
        assert_eq!(argument_index(&result, param, arg), Some((index, 1)));
    }
    let ScopedLoopResult::Scalars { bindings, .. } = result else {
        panic!()
    };
    for drift in [false, true] {
        let mut args = args.clone();
        let mut function = function.clone();
        if drift {
            function.params[3].name = "other".into();
        } else {
            args[3] = NirExpr::Int(1);
        }
        assert!(validate_seeds(&bindings, true, &function, &args, "action").is_err());
    }
}

#[test]
fn carried_field_seeds_mix_whole_and_field_mapped_records() {
    let module = parse_nuis_module(SOURCE).unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    let body = vec![
        record("packet", "Pair", &[("left", 0), ("right", 1)]),
        record("other", "Other", &[("left", 2), ("right", 3)]),
    ];
    let projections = projected_bindings("result", &ty("Slots"), &body, &definitions).unwrap();
    let mut function = module.functions[0].clone();
    function.params = vec![
        scalar("right"),
        NirParam {
            name: "whole".into(),
            ty: ty("Other"),
        },
        scalar("left"),
    ];
    let args = vec![
        field("packet", "right"),
        NirExpr::Var("other".into()),
        field("packet", "left"),
    ];
    validate_seeds(&projections, false, &function, &args, "action").unwrap();
    let result = ScopedLoopResult::Scalars {
        separate_seeds: false,
        bindings: projections,
        layout: String::new(),
        breaking: false,
    };
    for ((param, arg), range) in function
        .params
        .iter()
        .zip(&args)
        .zip([(1, 1), (2, 2), (0, 1)])
    {
        assert_eq!(argument_index(&result, param, arg), Some(range));
    }
}

const FIELD_SOURCE: &str =
    include_str!("../../../tests/control_flow_syntax_native/scoped_field_seeds.ns");

#[test]
fn carried_field_seeds_execute_updated_values_zero_trips_and_old_snapshots() {
    check_execution(FIELD_SOURCE);
}

fn check_execution(source: &str) {
    let compiled = crate::pipeline::compile_source(source).unwrap();
    let mut yir = compiled.yir;
    yir.nodes.reverse();
    yir.functions.reverse();
    for function in &mut yir.functions {
        function.body_nodes.reverse();
    }
    yir_verify::verify_module(&yir).unwrap();
    yir_lower_llvm::emit_module(&yir).unwrap();
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(&yir),
        &yir_verify::default_registry(),
    )
    .unwrap();
    let entry = yir
        .functions
        .iter()
        .find(|f| f.role == yir_core::YirFunctionRole::Entry)
        .unwrap();
    assert_eq!(
        trace.values[&entry.result.as_ref().unwrap().node],
        yir_core::Value::Int(171)
    );
}

#[test]
fn carried_field_seeds_reject_incomplete_and_computed_source_maps() {
    for args in [
        "packet.right, packet.right, divisor",
        "packet.right, packet.left + 0, divisor",
        "packet.right, packet.unknown, divisor",
    ] {
        let source = FIELD_SOURCE.replace(
            "advance(packet.right, packet.left, divisor)",
            &format!("advance({args})"),
        );
        assert!(crate::pipeline::compile_source(&source).is_err(), "{args}");
    }
}

#[test]
fn carried_field_seeds_preserve_nominal_origins_through_calls_and_fields() {
    for nominal in ["Packet", "Other"] {
        let value = format!("{nominal} {{ left: 12, right: 33 }}");
        let definitions = format!(
            "struct Other {{ left: i64, right: i64 }}
             struct Envelope {{ packet: {nominal} }}
             @noinline fn seed() -> {nominal} {{ return {value}; }}"
        );
        for initial in [
            format!("let packet = {value};"),
            "let packet = seed();".into(),
            format!("let input = {value}; let packet = input;"),
            format!("let outer = Envelope {{ packet: {value} }}; let packet = outer.packet;"),
        ] {
            let source = FIELD_SOURCE
                .replace("struct Slots", &format!("{definitions}\n    struct Slots"))
                .replace("let packet = Packet { left: 12, right: 33 };", &initial);
            assert!(source.contains(&initial));
            if nominal == "Packet" {
                check_execution(&source);
            } else {
                let error = crate::pipeline::compile_source(&source)
                    .err()
                    .unwrap_or_else(|| panic!("admitted nominal drift: {initial}"));
                assert!(error.contains("scoped field seed"), "{initial}: {error}");
                assert!(error.contains("found Other"), "{initial}: {error}");
            }
        }
    }
}
