use super::*;

fn ty(name: &str) -> NirTypeRef {
    NirTypeRef {
        name: name.into(),
        is_ref: false,
        is_optional: false,
        generic_args: vec![],
    }
}

fn layouts() -> BTreeMap<String, Vec<(String, NirTypeRef)>> {
    BTreeMap::from([
        (
            "Leaf".into(),
            vec![("value".into(), ty("i64")), ("flag".into(), ty("bool"))],
        ),
        (
            "Root".into(),
            vec![("left".into(), ty("Leaf")), ("right".into(), ty("Leaf"))],
        ),
    ])
}

#[test]
fn pure_record_shapes_keep_segment_paths_and_exact_nested_nominal_constructors() {
    let shape = Shape::from_layouts(&ty("Root"), &layouts()).unwrap();
    let leaves = shape.leaves();
    assert_eq!(
        leaves
            .iter()
            .map(|(path, ty)| (path.join("/"), ty.name.as_str()))
            .collect::<Vec<_>>(),
        [
            ("left/value".into(), "i64"),
            ("left/flag".into(), "bool"),
            ("right/value".into(), "i64"),
            ("right/flag".into(), "bool")
        ]
    );
    let mut slot = 0;
    let value = shape.reconstruct(&mut |_| {
        slot += 1;
        NirExpr::Int(slot)
    });
    assert_eq!(
        shape.values(&value).unwrap(),
        [
            &NirExpr::Int(1),
            &NirExpr::Int(2),
            &NirExpr::Int(3),
            &NirExpr::Int(4)
        ]
    );
    assert_ne!(
        source_value("root", &["left.value".into()]),
        source_value("root", &["left".into(), "value".into()])
    );
    for change in [
        "nominal",
        "order",
        "missing",
        "extra",
        "duplicate",
        "generic",
        "expression",
    ] {
        let mut invalid = value.clone();
        let NirExpr::StructLiteral { fields, .. } = &mut invalid else {
            unreachable!()
        };
        let NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } = &mut fields[0].1
        else {
            unreachable!()
        };
        match change {
            "nominal" => *type_name = "Other".into(),
            "order" => fields.swap(0, 1),
            "missing" => {
                fields.pop();
            }
            "extra" => fields.push(("extra".into(), NirExpr::Int(3))),
            "duplicate" => fields[1].0 = fields[0].0.clone(),
            "generic" => type_args.push(ty("i64")),
            "expression" => {
                let NirExpr::StructLiteral { fields, .. } = &mut invalid else {
                    unreachable!()
                };
                fields[0].1 = NirExpr::Var("other".into());
            }
            _ => unreachable!(),
        }
        assert!(shape.values(&invalid).is_none(), "{change}");
    }
}

#[test]
fn pure_record_shapes_reject_resources_cycles_and_qualified_types_transitively() {
    for change in [
        "resource",
        "cycle",
        "empty",
        "duplicate",
        "reference",
        "optional",
        "generic",
    ] {
        let mut layouts = layouts();
        let fields = layouts.get_mut("Leaf").unwrap();
        match change {
            "resource" => fields[1].1 = ty("Buffer"),
            "cycle" => fields[1].1 = ty("Root"),
            "empty" => fields.clear(),
            "duplicate" => fields[1].0 = fields[0].0.clone(),
            "reference" => fields[1].1.is_ref = true,
            "optional" => fields[1].1.is_optional = true,
            "generic" => fields[1].1.generic_args.push(ty("i64")),
            _ => unreachable!(),
        }
        assert!(
            Shape::from_layouts(&ty("Root"), &layouts).is_none(),
            "{change}"
        );
    }
}

#[test]
fn pure_record_shapes_bound_recursive_expansion_without_restricting_explicit_flat_width() {
    for (width, accepted, rejected) in [(1, 63, 64), (2, 10, 11)] {
        let mut layouts = BTreeMap::new();
        for level in 0..=rejected {
            layouts.insert(
                format!("R{level}"),
                (0..width)
                    .map(|slot| {
                        (
                            format!("f{slot}"),
                            if level == 0 {
                                ty("bool")
                            } else {
                                ty(&format!("R{}", level - 1))
                            },
                        )
                    })
                    .collect(),
            );
        }
        assert!(Shape::from_layouts(&ty(&format!("R{accepted}")), &layouts).is_some());
        assert!(Shape::from_layouts(&ty(&format!("R{rejected}")), &layouts).is_none());
    }
    let layouts = BTreeMap::from([(
        "Wide".into(),
        (0..5000).map(|i| (format!("f{i}"), ty("i64"))).collect(),
    )]);
    assert_eq!(
        Shape::from_layouts(&ty("Wide"), &layouts)
            .unwrap()
            .leaves()
            .len(),
        5000
    );
}
