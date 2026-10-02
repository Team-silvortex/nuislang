use super::*;

#[test]
fn nested_guard_defaults_require_a_pure_parameter_tree_not_just_a_pure_leaf() {
    let source = "mod cpu Main {
        struct Leaf { value: f64 }
        struct Root { left: Leaf, right: Leaf }
        fn branch(packet: Root) -> i64 { return 0; }
    }";
    for change in [
        "valid",
        "sibling_resource",
        "child_reference",
        "child_optional",
        "cycle",
        "wrong_path",
        "wrong_codec",
    ] {
        let mut module = crate::frontend::parse_nuis_module(source).unwrap();
        let root = module
            .structs
            .iter_mut()
            .find(|d| d.name == "Root")
            .unwrap();
        match change {
            "sibling_resource" => root.fields[1].ty.name = "Buffer".into(),
            "child_reference" => root.fields[0].ty.is_ref = true,
            "child_optional" => root.fields[0].ty.is_optional = true,
            "cycle" => root.fields[1].ty.name = "Root".into(),
            _ => {}
        }
        let field = crate::lowering::scalar_record_shape::source_value(
            "packet",
            &[
                "left".into(),
                if change == "wrong_path" {
                    "missing".into()
                } else {
                    "value".into()
                },
            ],
        );
        let encoded = if change == "wrong_codec" {
            NirExpr::PackF32Word(Box::new(field))
        } else {
            NirExpr::PackF64Word(Box::new(field))
        };
        let definitions = module
            .structs
            .iter()
            .map(|d| (d.name.as_str(), d))
            .collect();
        assert_eq!(
            is_pass_through_guard_seed(&module.functions[0], &encoded, &definitions),
            change == "valid",
            "{change}"
        );
    }
}

#[test]
fn f64_guard_defaults_require_bit_packing_of_exact_parameter_fields() {
    let module = crate::frontend::parse_nuis_module(
        "mod cpu Main {
        struct Packet { value: f64, word: i64 }
        fn branch(packet: Packet, scalar: f64) -> i64 { return 0; }
    }",
    )
    .unwrap();
    let structs = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    let field = |name: &str| NirExpr::FieldAccess {
        base: Box::new(NirExpr::Var("packet".into())),
        field: name.into(),
    };
    for (value, valid) in [
        (field("value"), true),
        (NirExpr::Var("scalar".into()), true),
        (field("word"), false),
        (field("absent"), false),
        (NirExpr::F64("0.0".into()), false),
        (
            NirExpr::Call {
                callee: "work".into(),
                args: vec![],
            },
            false,
        ),
    ] {
        assert_eq!(
            is_pass_through_guard_seed(
                &module.functions[0],
                &NirExpr::PackF64Word(Box::new(value)),
                &structs
            ),
            valid
        );
    }
    assert!(!is_pass_through_guard_seed(
        &module.functions[0],
        &NirExpr::PackF32Word(Box::new(field("value"))),
        &structs
    ));
    assert!(!is_pass_through_guard_seed(
        &module.functions[0],
        &NirExpr::CastF64ToI64(Box::new(field("value"))),
        &structs
    ));
}

#[test]
fn f32_guard_defaults_require_bit_packing_of_exact_parameter_fields() {
    let module = crate::frontend::parse_nuis_module(
        "mod cpu Main {
        struct Packet { value: f32, word: i64 }
        fn branch(packet: Packet, scalar: f32) -> i64 { return 0; }
    }",
    )
    .unwrap();
    let structs = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    let field = |name: &str| NirExpr::FieldAccess {
        base: Box::new(NirExpr::Var("packet".into())),
        field: name.into(),
    };
    for (value, valid) in [
        (field("value"), true),
        (NirExpr::Var("scalar".into()), true),
        (field("word"), false),
        (field("absent"), false),
        (NirExpr::F32("0.0".into()), false),
        (
            NirExpr::Call {
                callee: "work".into(),
                args: vec![],
            },
            false,
        ),
    ] {
        assert_eq!(
            is_pass_through_guard_seed(
                &module.functions[0],
                &NirExpr::PackF32Word(Box::new(value)),
                &structs
            ),
            valid
        );
    }
    assert!(!is_pass_through_guard_seed(
        &module.functions[0],
        &NirExpr::CastF32ToI64(Box::new(field("value"))),
        &structs
    ));
}

#[test]
fn mixed_record_guard_defaults_only_read_exact_typed_parameter_fields() {
    let module = crate::frontend::parse_nuis_module(
        "mod cpu Main {
        struct Packet { value: i64, enabled: bool }
        struct Words { carry0: i64, carry1: i64 }
        fn branch(packet: Packet) -> Words { return Words { carry0: 0, carry1: 0 }; }
    }",
    )
    .unwrap();
    let field = |name: &str| NirExpr::FieldAccess {
        base: Box::new(NirExpr::Var("packet".into())),
        field: name.into(),
    };
    for change in [
        "valid",
        "raw_bool",
        "cast_integer",
        "computed",
        "call",
        "missing",
        "reference",
        "optional",
        "generic",
        "resource",
        "nested",
    ] {
        let mut module = module.clone();
        let mut value = NirExpr::StructLiteral {
            type_name: "Words".into(),
            type_args: vec![],
            fields: vec![
                ("carry0".into(), field("value")),
                (
                    "carry1".into(),
                    NirExpr::CastBoolToI64(Box::new(field("enabled"))),
                ),
            ],
        };
        let NirExpr::StructLiteral { fields, .. } = &mut value else {
            unreachable!()
        };
        match change {
            "valid" => {}
            "raw_bool" => fields[1].1 = field("enabled"),
            "cast_integer" => fields[1].1 = NirExpr::CastBoolToI64(Box::new(field("value"))),
            "computed" => {
                fields[0].1 = NirExpr::Binary {
                    op: NirBinaryOp::Div,
                    lhs: Box::new(field("value")),
                    rhs: Box::new(NirExpr::Int(0)),
                }
            }
            "call" => {
                fields[0].1 = NirExpr::Call {
                    callee: "work".into(),
                    args: vec![field("value")],
                }
            }
            "missing" => fields[0].1 = field("absent"),
            "reference" => module.functions[0].params[0].ty.is_ref = true,
            "optional" => module.functions[0].params[0].ty.is_optional = true,
            "generic" => module.functions[0].params[0]
                .ty
                .generic_args
                .push(module.structs[0].fields[0].ty.clone()),
            "resource" => module.structs[0].fields[1].ty.name = "Buffer".into(),
            "nested" => module.structs[0].fields[1].ty.name = "Words".into(),
            _ => unreachable!(),
        }
        let structs = module
            .structs
            .iter()
            .map(|d| (d.name.as_str(), d))
            .collect();
        assert_eq!(
            is_pass_through_guard_seed(&module.functions[0], &value, &structs),
            change == "valid",
            "{change}"
        );
    }
}

#[test]
fn i32_guard_defaults_require_total_signed_parameter_projection() {
    let module = crate::frontend::parse_nuis_module(
        "mod cpu Main {
        struct Packet { narrow: i32, wide: i64 }
        fn branch(packet: Packet, scalar: i32) -> i64 { return 0; }
    }",
    )
    .unwrap();
    let structs = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    let field = |name: &str| NirExpr::FieldAccess {
        base: Box::new(NirExpr::Var("packet".into())),
        field: name.into(),
    };
    for (value, valid) in [
        (field("narrow"), true),
        (NirExpr::Var("scalar".into()), true),
        (field("wide"), false),
        (field("absent"), false),
        (
            NirExpr::Call {
                callee: "work".into(),
                args: vec![],
            },
            false,
        ),
        (NirExpr::CastI64ToI32(Box::new(NirExpr::Int(1))), false),
    ] {
        let encoded = NirExpr::CastI32ToI64(Box::new(value));
        assert_eq!(
            is_pass_through_guard_seed(&module.functions[0], &encoded, &structs),
            valid
        );
    }
    assert!(!is_pass_through_guard_seed(
        &module.functions[0],
        &field("narrow"),
        &structs
    ));
}
