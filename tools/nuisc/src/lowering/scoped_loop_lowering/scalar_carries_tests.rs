use super::*;
use crate::frontend::parse_nuis_module;

#[path = "scalar_carry_seed_maps_tests.rs"]
mod seed_maps;

#[path = "scalar_carry_seed_storage_tests.rs"]
mod seed_storage;

const SOURCE: &str = "mod cpu Main {
    struct Slots { carry0: i64, carry1: i64, carry2: i64, carry3: i64 }
    struct Tiny { carry0: i64 }
    struct Pair { left: i64, right: i64 }
    struct Other { left: i64, right: i64 }
    struct Marker { value: i64 }
    fn action(marker: Marker, total: i64, packet: Pair) -> Slots {
        return Slots { carry0: packet.left, carry1: packet.right, carry2: total, carry3: marker.value };
    }
    fn main() -> i64 { return 0; }
}";

fn ty(name: &str) -> NirTypeRef {
    NirTypeRef {
        name: name.to_owned(),
        is_ref: false,
        is_optional: false,
        generic_args: vec![],
    }
}

fn word(slot: usize) -> NirExpr {
    NirExpr::FieldAccess {
        base: Box::new(NirExpr::Var("result".into())),
        field: format!("carry{slot}"),
    }
}

fn record(name: &str, record: &str, values: &[(&str, usize)]) -> NirStmt {
    NirStmt::Let {
        name: name.into(),
        ty: Some(ty(record)),
        value: NirExpr::StructLiteral {
            type_name: record.into(),
            type_args: vec![],
            fields: values
                .iter()
                .map(|(name, slot)| ((*name).into(), word(*slot)))
                .collect(),
        },
    }
}

fn body() -> Vec<NirStmt> {
    vec![
        record("packet", "Pair", &[("left", 0), ("right", 1)]),
        NirStmt::Let {
            name: "total".into(),
            ty: Some(ty("i64")),
            value: word(2),
        },
        record("marker", "Marker", &[("value", 3)]),
    ]
}

#[test]
fn flat_projection_slots_follow_layouts_not_capture_order() {
    let module = parse_nuis_module(SOURCE).unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|definition| (definition.name.as_str(), definition))
        .collect();
    let body = body();
    let projections = projected_bindings("result", &ty("Slots"), &body, &definitions).unwrap();
    assert_eq!(
        projections
            .iter()
            .map(|binding| (binding.name, binding.width()))
            .collect::<Vec<_>>(),
        [("packet", 2), ("total", 1), ("marker", 1)]
    );
    let function = &module.functions[0];
    let args = function
        .params
        .iter()
        .map(|param| NirExpr::Var(param.name.clone()))
        .collect::<Vec<_>>();
    validate_seeds(&projections, false, function, &args, "action").unwrap();
    let result = ScopedLoopResult::Scalars {
        separate_seeds: false,
        bindings: projections,
        layout: String::new(),
        breaking: false,
    };
    for (index, expected) in [(0, (3, 1)), (1, (2, 1)), (2, (0, 2))] {
        assert_eq!(
            argument_index(&result, &function.params[index], &args[index]),
            Some(expected)
        );
    }
    let singleton = vec![record("marker", "Marker", &[("value", 0)])];
    assert_eq!(
        projected_bindings("result", &ty("Tiny"), &singleton, &definitions).unwrap()[0].width(),
        1
    );
    let scalar = vec![NirStmt::Let {
        name: "value".into(),
        ty: Some(ty("i64")),
        value: word(0),
    }];
    assert!(projected_bindings("result", &ty("Tiny"), &scalar, &definitions).is_none());
}

#[test]
fn flat_projection_rejects_noncanonical_reconstruction_without_evaluation() {
    let module = parse_nuis_module(SOURCE).unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|definition| (definition.name.as_str(), definition))
        .collect();
    for change in [
        "nominal",
        "missing",
        "duplicate",
        "order",
        "base",
        "call",
        "extra",
        "reference",
        "optional",
        "generic",
        "alias",
    ] {
        let mut body = body();
        let NirStmt::Let {
            name,
            ty: annotation,
            value: NirExpr::StructLiteral {
                type_name, fields, ..
            },
        } = &mut body[0]
        else {
            unreachable!()
        };
        match change {
            "nominal" => *type_name = "Other".into(),
            "missing" => {
                fields.pop();
            }
            "duplicate" => fields[1].1 = word(0),
            "order" => fields.swap(0, 1),
            "base" => {
                fields[1].1 = NirExpr::FieldAccess {
                    base: Box::new(NirExpr::Var("previous".into())),
                    field: "carry1".into(),
                }
            }
            "call" => {
                fields[1].1 = NirExpr::Call {
                    callee: "fallible".into(),
                    args: vec![word(1)],
                }
            }
            "extra" => fields.push(("extra".into(), word(2))),
            "reference" => annotation.as_mut().unwrap().is_ref = true,
            "optional" => annotation.as_mut().unwrap().is_optional = true,
            "generic" => annotation.as_mut().unwrap().generic_args.push(ty("i64")),
            "alias" => *name = "result".into(),
            _ => unreachable!(),
        }
        assert!(
            projected_bindings("result", &ty("Slots"), &body, &definitions).is_none(),
            "{change}"
        );
    }
    let mut duplicate = body();
    let NirStmt::Let { name, .. } = &mut duplicate[2] else {
        unreachable!()
    };
    *name = "packet".into();
    assert!(projected_bindings("result", &ty("Slots"), &duplicate, &definitions).is_none());
    assert!(projected_bindings("result", &ty("Tiny"), &body(), &definitions).is_none());
}

#[test]
fn flat_projection_requires_exact_nominal_single_seed_and_never_a_break_record() {
    let module = parse_nuis_module(SOURCE).unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|definition| (definition.name.as_str(), definition))
        .collect();
    let body = body();
    let projections = projected_bindings("result", &ty("Slots"), &body, &definitions).unwrap();
    for change in ["nominal", "missing", "duplicate", "expression"] {
        let mut function = module.functions[0].clone();
        let mut args = function
            .params
            .iter()
            .map(|param| NirExpr::Var(param.name.clone()))
            .collect::<Vec<_>>();
        match change {
            "nominal" => function.params[2].ty = ty("Other"),
            "missing" => args[2] = NirExpr::Var("saved".into()),
            "duplicate" => {
                function.params.push(function.params[2].clone());
                args.push(args[2].clone());
            }
            "expression" => {
                args[2] = NirExpr::Call {
                    callee: "make".into(),
                    args: vec![],
                }
            }
            _ => unreachable!(),
        }
        assert!(
            validate_seeds(&projections, false, &function, &args, "action").is_err(),
            "{change}"
        );
    }
    // A same-width record cannot impersonate the normalizer's i64 control seed.
    let function = &module.functions[0];
    let args = vec![
        NirExpr::Int(0),
        NirExpr::Var("total".into()),
        NirExpr::Var("packet".into()),
    ];
    assert!(!is_scalar_i64(projections.last().unwrap().ty));
    assert!(!break_guard(None, "marker"));
    assert!(validate_seeds(&projections, false, function, &args, "action").is_err());
    assert!(validate_seeds(&projections, true, function, &args, "action").is_err());
}

#[test]
fn bool_projection_requires_explicit_typed_seed_and_decode() {
    let module = parse_nuis_module(SOURCE).unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    let body = vec![NirStmt::Let {
        name: "flag".into(),
        ty: Some(ty("bool")),
        value: NirExpr::CastI64ToBool(Box::new(word(0))),
    }];
    let projections = projected_bindings("result", &ty("Tiny"), &body, &definitions).unwrap();
    let mut function = module.functions[0].clone();
    function.params = vec![NirParam {
        name: "private_word".into(),
        ty: ty("i64"),
    }];
    let args = vec![NirExpr::CastBoolToI64(Box::new(NirExpr::Var(
        "flag".into(),
    )))];
    validate_seeds(&projections, false, &function, &args, "action").unwrap();
    assert!(validate_seeds(&projections, true, &function, &args, "action").is_err());
    for change in ["missing", "raw", "expression", "type", "duplicate"] {
        let mut function = function.clone();
        let mut args = args.clone();
        match change {
            "missing" => args[0] = NirExpr::CastBoolToI64(Box::new(NirExpr::Var("saved".into()))),
            "raw" => args[0] = NirExpr::Var("flag".into()),
            "expression" => args[0] = NirExpr::CastBoolToI64(Box::new(NirExpr::Bool(true))),
            "type" => function.params[0].ty = ty("bool"),
            "duplicate" => {
                function.params.push(function.params[0].clone());
                args.push(args[0].clone());
            }
            _ => unreachable!(),
        }
        assert!(
            validate_seeds(&projections, false, &function, &args, "action").is_err(),
            "{change}"
        );
    }
    for value in [
        word(0),
        NirExpr::CastI64ToBool(Box::new(word(1))),
        NirExpr::CastBoolToI64(Box::new(word(0))),
        NirExpr::CastI64ToBool(Box::new(NirExpr::Int(1))),
    ] {
        let mut invalid = body.clone();
        let NirStmt::Let { value: slot, .. } = &mut invalid[0] else {
            unreachable!()
        };
        *slot = value;
        assert!(projected_bindings("result", &ty("Tiny"), &invalid, &definitions).is_none());
    }
    let result = ScopedLoopResult::Scalars {
        separate_seeds: false,
        bindings: projections,
        layout: String::new(),
        breaking: false,
    };
    assert_eq!(
        argument_index(&result, &function.params[0], &args[0]),
        Some((0, 1))
    );
}
