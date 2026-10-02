use super::*;

const NESTED: &str = "mod cpu Main {
    struct Slots { carry0: i64, carry1: i64, carry2: i64, carry3: i64 }
    struct Words { carry0: i64, carry1: i64, carry2: i64, carry3: i64 }
    struct Leaf { value: i64, enabled: bool }
    struct Other { value: i64, enabled: bool }
    struct Root { left: Leaf, right: Leaf }
    fn action(input: Words) -> Slots {
        return Slots { carry0: input.carry0, carry1: input.carry1,
            carry2: input.carry2, carry3: input.carry3 };
    }
    fn main() -> i64 { return 0; }
}";

fn reconstruction() -> Vec<NirStmt> {
    vec![NirStmt::Let {
        name: "packet".into(),
        ty: Some(ty("Root")),
        value: NirExpr::StructLiteral {
            type_name: "Root".into(),
            type_args: vec![],
            fields: ["left", "right"]
                .into_iter()
                .enumerate()
                .map(|(index, name)| {
                    (
                        name.into(),
                        NirExpr::StructLiteral {
                            type_name: "Leaf".into(),
                            type_args: vec![],
                            fields: vec![
                                ("value".into(), word(index * 2)),
                                (
                                    "enabled".into(),
                                    NirExpr::CastI64ToBool(Box::new(word(index * 2 + 1))),
                                ),
                            ],
                        },
                    )
                })
                .collect(),
        },
    }]
}

fn path(parent: &str, field: &str) -> NirExpr {
    NirExpr::FieldAccess {
        base: Box::new(NirExpr::FieldAccess {
            base: Box::new(NirExpr::Var("packet".into())),
            field: parent.into(),
        }),
        field: field.into(),
    }
}

fn seed() -> NirExpr {
    NirExpr::StructLiteral {
        type_name: "Words".into(),
        type_args: vec![],
        fields: vec![
            ("carry0".into(), path("left", "value")),
            (
                "carry1".into(),
                NirExpr::CastBoolToI64(Box::new(path("left", "enabled"))),
            ),
            ("carry2".into(), path("right", "value")),
            (
                "carry3".into(),
                NirExpr::CastBoolToI64(Box::new(path("right", "enabled"))),
            ),
        ],
    }
}

#[test]
fn nested_record_word_maps_require_exact_segment_paths_and_typed_codecs() {
    let module = parse_nuis_module(NESTED).unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    let body = reconstruction();
    let bindings = projected_bindings("result", &ty("Slots"), &body, &definitions).unwrap();
    assert_eq!(bindings[0].width(), 4);
    assert!(has_encoded_fields(&bindings[0]));
    validate_seeds(
        &bindings,
        false,
        &module.functions[0],
        &[seed()],
        "action",
        &definitions,
    )
    .unwrap();
    for change in [
        "parent",
        "field",
        "codec",
        "missing",
        "duplicate",
        "order",
        "nominal",
        "binding",
    ] {
        let mut input = seed();
        let NirExpr::StructLiteral {
            type_name, fields, ..
        } = &mut input
        else {
            unreachable!()
        };
        match change {
            "parent" => fields[2].1 = path("left", "value"),
            "field" => fields[2].1 = path("right", "enabled"),
            "codec" => fields[3].1 = NirExpr::CastI32ToI64(Box::new(path("right", "enabled"))),
            "missing" => {
                fields.pop();
            }
            "duplicate" => fields[2] = fields[0].clone(),
            "order" => fields.swap(0, 2),
            "nominal" => *type_name = "Slots".into(),
            "binding" => {
                fields[0].1 = crate::lowering::scalar_record_shape::source_value(
                    "saved",
                    &["left".into(), "value".into()],
                )
            }
            _ => unreachable!(),
        }
        assert!(
            validate_seeds(
                &bindings,
                false,
                &module.functions[0],
                &[input],
                "action",
                &definitions
            )
            .is_err(),
            "{change}"
        );
    }
}

#[test]
fn nested_record_backedges_reject_child_nominal_slot_and_codec_drift() {
    let module = parse_nuis_module(NESTED).unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    for change in [
        "nominal",
        "duplicate",
        "slot",
        "codec",
        "order",
        "sparse",
        "base",
    ] {
        let mut body = reconstruction();
        let NirStmt::Let {
            value: NirExpr::StructLiteral { fields, .. },
            ..
        } = &mut body[0]
        else {
            unreachable!()
        };
        let NirExpr::StructLiteral {
            type_name, fields, ..
        } = &mut fields[1].1
        else {
            unreachable!()
        };
        match change {
            "nominal" => *type_name = "Other".into(),
            "duplicate" => fields[0].1 = word(0),
            "slot" => fields[0].1 = word(3),
            "codec" => fields[1].1 = NirExpr::CastI64ToI32(Box::new(word(3))),
            "order" => fields.swap(0, 1),
            "sparse" => {
                fields.pop();
            }
            "base" => {
                fields[0].1 = NirExpr::FieldAccess {
                    base: Box::new(NirExpr::Var("saved".into())),
                    field: "carry2".into(),
                }
            }
            _ => unreachable!(),
        }
        assert!(
            projected_bindings("result", &ty("Slots"), &body, &definitions).is_none(),
            "{change}"
        );
    }
}

#[test]
fn nested_i64_maps_still_require_complete_path_seeds() {
    let module = parse_nuis_module(&NESTED.replace("enabled: bool", "enabled: i64")).unwrap();
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
    for (index, (_, child)) in fields.iter_mut().enumerate() {
        let NirExpr::StructLiteral { fields, .. } = child else {
            unreachable!()
        };
        fields[1].1 = word(index * 2 + 1);
    }
    let bindings = projected_bindings("result", &ty("Slots"), &body, &definitions).unwrap();
    assert!(has_encoded_fields(&bindings[0]));
    let mut input = seed();
    let NirExpr::StructLiteral { fields, .. } = &mut input else {
        unreachable!()
    };
    fields[1].1 = path("left", "enabled");
    fields[3].1 = path("right", "enabled");
    validate_seeds(
        &bindings,
        false,
        &module.functions[0],
        &[input],
        "action",
        &definitions,
    )
    .unwrap();
    assert!(validate_seeds(
        &bindings,
        false,
        &module.functions[0],
        &[NirExpr::Var("packet".into())],
        "action",
        &definitions
    )
    .is_err());
}

#[test]
fn sparse_nested_word_seeds_match_typed_paths_and_reject_duplicate_coverage() {
    let module = parse_nuis_module(NESTED).unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    let body = reconstruction();
    let bindings = projected_bindings("result", &ty("Slots"), &body, &definitions).unwrap();
    let binding = &bindings[0];
    let param = NirParam {
        name: "word".into(),
        ty: ty("i64"),
    };
    assert_eq!(
        seed_range(binding, &param, &path("right", "value"), &definitions),
        Some((2, 1))
    );
    let flag = NirExpr::CastBoolToI64(Box::new(path("right", "enabled")));
    assert_eq!(
        seed_range(binding, &param, &flag, &definitions),
        Some((3, 1))
    );
    for arg in [
        path("right", "enabled"),
        NirExpr::CastI32ToI64(Box::new(path("right", "enabled"))),
        NirExpr::CastBoolToI64(Box::new(path("right", "value"))),
        NirExpr::CastBoolToI64(Box::new(NirExpr::Var("other".into()))),
    ] {
        assert_eq!(seed_range(binding, &param, &arg, &definitions), None);
    }
    let mut function = module.functions[0].clone();
    function.params = vec![param.clone()];
    assert!(needs_separate_seeds(
        &bindings,
        false,
        &function,
        &[flag.clone()],
        "action",
        &[],
        &definitions
    )
    .unwrap());
    function.params.push(param);
    assert!(needs_separate_seeds(
        &bindings,
        false,
        &function,
        &[flag.clone(), flag],
        "action",
        &[],
        &definitions
    )
    .unwrap_err()
    .contains("duplicate seed coverage"));
    function.params.clear();
    assert!(needs_separate_seeds(
        &bindings,
        false,
        &function,
        &[],
        "action",
        &[],
        &definitions
    )
    .is_err());
    assert!(needs_separate_seeds(
        &bindings,
        false,
        &function,
        &[],
        "action",
        &[binding.record_seed(0)],
        &definitions
    )
    .unwrap());
}
