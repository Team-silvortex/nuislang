use super::*;

const MIXED: &str = "mod cpu Main {
    struct Slots { carry0: i64, carry1: i64 }
    struct Words { carry0: i64, carry1: i64 }
    struct Packet { value: i64, enabled: bool }
    fn action(input: Words) -> Slots { return Slots { carry0: input.carry0, carry1: input.carry1 }; }
    fn main() -> i64 { return 0; }
}";

fn reconstruction() -> Vec<NirStmt> {
    vec![NirStmt::Let {
        name: "packet".into(),
        ty: Some(ty("Packet")),
        value: NirExpr::StructLiteral {
            type_name: "Packet".into(),
            type_args: vec![],
            fields: vec![
                ("value".into(), word(0)),
                ("enabled".into(), NirExpr::CastI64ToBool(Box::new(word(1)))),
            ],
        },
    }]
}

fn seed() -> NirExpr {
    NirExpr::StructLiteral {
        type_name: "Words".into(),
        type_args: vec![],
        fields: vec![
            (
                "carry0".into(),
                mixed_words::source_word("packet", "value", &ty("i64")),
            ),
            (
                "carry1".into(),
                mixed_words::source_word("packet", "enabled", &ty("bool")),
            ),
        ],
    }
}

#[test]
fn f64_record_word_maps_require_bit_encoding_and_reject_numeric_casts() {
    let module = parse_nuis_module(&MIXED.replace("enabled: bool", "enabled: f64")).unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    let mut body = reconstruction();
    let NirStmt::Let {
        value: NirExpr::StructLiteral { fields, .. },
        ..
    } = &mut body[0]
    else {
        unreachable!()
    };
    fields[1].1 = NirExpr::UnpackF64Word(Box::new(word(1)));
    let projections = projected_bindings("result", &ty("Slots"), &body, &definitions).unwrap();
    let mut input = seed();
    let NirExpr::StructLiteral { fields, .. } = &mut input else {
        unreachable!()
    };
    fields[1].1 = mixed_words::source_word("packet", "enabled", &ty("f64"));
    let function = &module.functions[0];
    validate_seeds(
        &projections,
        false,
        function,
        &[input.clone()],
        "action",
        &definitions,
    )
    .unwrap();
    for invalid in [
        NirExpr::CastF64ToI64(Box::new(NirExpr::FieldAccess {
            base: Box::new(NirExpr::Var("packet".into())),
            field: "enabled".into(),
        })),
        mixed_words::source_word("packet", "enabled", &ty("i64")),
        mixed_words::source_word("packet", "enabled", &ty("bool")),
        mixed_words::source_word("packet", "enabled", &ty("f32")),
        mixed_words::source_word("saved", "enabled", &ty("f64")),
        mixed_words::source_word("packet", "value", &ty("f64")),
    ] {
        let mut input = input.clone();
        let NirExpr::StructLiteral { fields, .. } = &mut input else {
            unreachable!()
        };
        fields[1].1 = invalid;
        assert!(validate_seeds(
            &projections,
            false,
            function,
            &[input],
            "action",
            &definitions
        )
        .is_err());
    }
    for invalid in [
        NirExpr::CastI64ToF64(Box::new(word(1))),
        word(1),
        NirExpr::UnpackF32Word(Box::new(word(1))),
        NirExpr::CastI64ToBool(Box::new(word(1))),
        NirExpr::UnpackF64Word(Box::new(word(0))),
    ] {
        let mut body = body.clone();
        let NirStmt::Let {
            value: NirExpr::StructLiteral { fields, .. },
            ..
        } = &mut body[0]
        else {
            unreachable!()
        };
        fields[1].1 = invalid;
        assert!(projected_bindings("result", &ty("Slots"), &body, &definitions).is_none());
    }
}

#[test]
fn f32_record_word_maps_require_bit_encoding_and_reject_numeric_casts() {
    let module = parse_nuis_module(&MIXED.replace("enabled: bool", "enabled: f32")).unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    let mut body = reconstruction();
    let NirStmt::Let {
        value: NirExpr::StructLiteral { fields, .. },
        ..
    } = &mut body[0]
    else {
        unreachable!()
    };
    fields[1].1 = NirExpr::UnpackF32Word(Box::new(word(1)));
    let projections = projected_bindings("result", &ty("Slots"), &body, &definitions).unwrap();
    let mut input = seed();
    let NirExpr::StructLiteral { fields, .. } = &mut input else {
        unreachable!()
    };
    fields[1].1 = mixed_words::source_word("packet", "enabled", &ty("f32"));
    let function = &module.functions[0];
    validate_seeds(
        &projections,
        false,
        function,
        &[input.clone()],
        "action",
        &definitions,
    )
    .unwrap();
    for invalid in [
        NirExpr::CastF32ToI64(Box::new(NirExpr::FieldAccess {
            base: Box::new(NirExpr::Var("packet".into())),
            field: "enabled".into(),
        })),
        mixed_words::source_word("packet", "enabled", &ty("i64")),
        mixed_words::source_word("packet", "enabled", &ty("bool")),
        mixed_words::source_word("saved", "enabled", &ty("f32")),
        mixed_words::source_word("packet", "value", &ty("f32")),
    ] {
        let mut input = input.clone();
        let NirExpr::StructLiteral { fields, .. } = &mut input else {
            unreachable!()
        };
        fields[1].1 = invalid;
        assert!(validate_seeds(
            &projections,
            false,
            function,
            &[input],
            "action",
            &definitions
        )
        .is_err());
    }
    for invalid in [
        NirExpr::CastI64ToF32(Box::new(word(1))),
        word(1),
        NirExpr::CastI64ToBool(Box::new(word(1))),
        NirExpr::UnpackF32Word(Box::new(word(0))),
    ] {
        let mut body = body.clone();
        let NirStmt::Let {
            value: NirExpr::StructLiteral { fields, .. },
            ..
        } = &mut body[0]
        else {
            unreachable!()
        };
        fields[1].1 = invalid;
        assert!(projected_bindings("result", &ty("Slots"), &body, &definitions).is_none());
    }
}

#[test]
fn mixed_record_word_maps_require_exact_types_slots_and_source_identity() {
    let module = parse_nuis_module(MIXED).unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    let body = reconstruction();
    let projections = projected_bindings("result", &ty("Slots"), &body, &definitions).unwrap();
    let function = &module.functions[0];
    validate_seeds(
        &projections,
        false,
        function,
        &[seed()],
        "action",
        &definitions,
    )
    .unwrap();
    assert!(projections[0].whole_record_seed(&function.params[0], &seed(), &definitions));
    for change in [
        "raw",
        "source",
        "constant",
        "duplicate",
        "order",
        "missing",
        "nominal",
        "reference",
        "optional",
        "field_type",
    ] {
        let mut function = function.clone();
        let mut args = vec![seed()];
        let NirExpr::StructLiteral {
            type_name, fields, ..
        } = &mut args[0]
        else {
            unreachable!()
        };
        match change {
            "raw" => fields[1].1 = mixed_words::source_word("packet", "enabled", &ty("i64")),
            "source" => fields[1].1 = mixed_words::source_word("saved", "enabled", &ty("bool")),
            "constant" => fields[1].1 = NirExpr::CastBoolToI64(Box::new(NirExpr::Bool(true))),
            "duplicate" => fields[1].1 = fields[0].1.clone(),
            "order" => fields.swap(0, 1),
            "missing" => {
                fields.pop();
            }
            "nominal" => *type_name = "Slots".into(),
            "reference" => function.params[0].ty.is_ref = true,
            "optional" => function.params[0].ty.is_optional = true,
            "field_type" => {
                function.params[0].ty = ty("Packet");
                *type_name = "Packet".into();
            }
            _ => unreachable!(),
        }
        assert!(
            validate_seeds(
                &projections,
                false,
                &function,
                &args,
                "action",
                &definitions
            )
            .is_err(),
            "{change}"
        );
    }
    let mut duplicate = function.clone();
    duplicate.params.push(duplicate.params[0].clone());
    assert!(validate_seeds(
        &projections,
        false,
        &duplicate,
        &[seed(), seed()],
        "action",
        &definitions
    )
    .unwrap_err()
    .contains("duplicate"));
}

#[test]
fn mixed_record_word_reconstruction_rejects_missing_and_wrong_decodes() {
    let module = parse_nuis_module(MIXED).unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    for value in [
        word(1),
        NirExpr::CastI64ToBool(Box::new(word(0))),
        NirExpr::Bool(true),
    ] {
        let mut body = reconstruction();
        let NirStmt::Let {
            value: NirExpr::StructLiteral { fields, .. },
            ..
        } = &mut body[0]
        else {
            unreachable!()
        };
        fields[1].1 = value;
        assert!(projected_bindings("result", &ty("Slots"), &body, &definitions).is_none());
    }
    for unsupported in [
        "i32",
        "f32",
        "f64",
        "reference",
        "optional",
        "generic",
        "Buffer",
    ] {
        let mut module = parse_nuis_module(MIXED).unwrap();
        let field = &mut module
            .structs
            .iter_mut()
            .find(|d| d.name == "Packet")
            .unwrap()
            .fields[1];
        match unsupported {
            "reference" => field.ty.is_ref = true,
            "optional" => field.ty.is_optional = true,
            "generic" => field.ty.generic_args.push(ty("i64")),
            kind => field.ty.name = kind.into(),
        }
        let definitions = module
            .structs
            .iter()
            .map(|d| (d.name.as_str(), d))
            .collect();
        assert!(
            projected_bindings("result", &ty("Slots"), &reconstruction(), &definitions).is_none(),
            "{unsupported}"
        );
    }
}

#[test]
fn i32_record_word_maps_require_signed_encoding_and_exact_decoding() {
    let module = parse_nuis_module(&MIXED.replace("enabled: bool", "enabled: i32")).unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    let mut body = reconstruction();
    let NirStmt::Let {
        value: NirExpr::StructLiteral { fields, .. },
        ..
    } = &mut body[0]
    else {
        unreachable!()
    };
    fields[1].1 = NirExpr::CastI64ToI32(Box::new(word(1)));
    let projections = projected_bindings("result", &ty("Slots"), &body, &definitions).unwrap();
    let mut input = seed();
    let NirExpr::StructLiteral { fields, .. } = &mut input else {
        unreachable!()
    };
    fields[1].1 = mixed_words::source_word("packet", "enabled", &ty("i32"));
    let function = &module.functions[0];
    validate_seeds(
        &projections,
        false,
        function,
        &[input.clone()],
        "action",
        &definitions,
    )
    .unwrap();
    for invalid in [
        mixed_words::source_word("packet", "enabled", &ty("i64")),
        mixed_words::source_word("packet", "enabled", &ty("bool")),
        mixed_words::source_word("saved", "enabled", &ty("i32")),
        mixed_words::source_word("packet", "value", &ty("i32")),
    ] {
        let mut input = input.clone();
        let NirExpr::StructLiteral { fields, .. } = &mut input else {
            unreachable!()
        };
        fields[1].1 = invalid;
        assert!(validate_seeds(
            &projections,
            false,
            function,
            &[input],
            "action",
            &definitions
        )
        .is_err());
    }
    for invalid in [
        word(1),
        NirExpr::CastI64ToBool(Box::new(word(1))),
        NirExpr::CastI64ToI32(Box::new(word(0))),
    ] {
        let mut body = body.clone();
        let NirStmt::Let {
            value: NirExpr::StructLiteral { fields, .. },
            ..
        } = &mut body[0]
        else {
            unreachable!()
        };
        fields[1].1 = invalid;
        assert!(projected_bindings("result", &ty("Slots"), &body, &definitions).is_none());
    }
}
