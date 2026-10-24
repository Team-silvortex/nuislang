use super::*;

fn flat(fields: usize, i32_tail: bool) -> control_values::CarryLayouts {
    let mut fields = (0..fields)
        .map(|i| (format!("f{i}"), scalar_type("i64")))
        .collect::<Vec<_>>();
    if i32_tail {
        fields.push(("cast".into(), scalar_type("i32")));
    }
    BTreeMap::from([("Wide".into(), fields)])
}

#[test]
fn discarded_record_neutral_expansion_has_exact_node_and_cast_bounds() {
    for (fields, cast, expected) in [
        (4095, false, true),
        (4096, false, false),
        (4093, true, true),
        (4094, true, false),
    ] {
        let layouts = flat(fields, cast);
        assert!(control_values::supported_type(
            &scalar_type("Wide"),
            &layouts
        ));
        let mut work = 4096;
        assert_eq!(neutral(&scalar_type("Wide"), &layouts, &mut work), expected);
        if expected {
            assert_eq!(work, 0);
            assert!(conditional_values::prefix::expression(
                &control_values::zero_value(&scalar_type("Wide"), &layouts)
            ));
        }
    }
}

#[test]
fn discarded_record_neutral_depth_cycles_and_invalid_fields_are_rejected_before_synthesis() {
    for (depth, leaf, expected) in [
        (63, "i64", true),
        (64, "i64", false),
        (62, "i32", true),
        (63, "i32", false),
    ] {
        let layouts: control_values::CarryLayouts = (0..depth)
            .map(|i| {
                let next = if i + 1 == depth {
                    leaf.to_owned()
                } else {
                    format!("R{}", i + 1)
                };
                (format!("R{i}"), vec![("next".into(), scalar_type(&next))])
            })
            .collect();
        assert_eq!(neutral(&scalar_type("R0"), &layouts, &mut 4096), expected);
        if expected {
            assert!(conditional_values::prefix::expression(
                &control_values::zero_value(&scalar_type("R0"), &layouts)
            ));
        }
    }
    for mut ty in [
        scalar_type("Wide"),
        scalar_type("unknown"),
        scalar_type("i64"),
    ] {
        if ty.name == "i64" {
            ty.is_ref = true;
        }
        let layouts = BTreeMap::from([("Wide".into(), vec![("field".into(), ty)])]);
        assert!(!neutral(&scalar_type("Wide"), &layouts, &mut 4096));
    }
}

fn discarded(ty: &str) -> NirStmt {
    NirStmt::If {
        condition: NirExpr::Bool(true),
        then_body: vec![NirStmt::Let {
            name: "unused".into(),
            ty: Some(scalar_type(ty)),
            value: NirExpr::Var("input".into()),
        }],
        else_body: vec![],
    }
}

#[test]
fn discarded_record_neutrals_share_one_declaration_budget_without_narrowing_flat_layouts() {
    let layouts = flat(2047, false);
    assert!(bounded(
        &[discarded("Wide")],
        &[discarded("Wide")],
        &layouts
    ));
    assert!(!bounded(
        &[discarded("Wide"), discarded("Wide")],
        &[discarded("Wide")],
        &layouts
    ));
    assert!(!bounded(
        &[discarded("Wide")],
        &[discarded("Wide")],
        &flat(2048, false)
    ));
    let mut module =
        crate::frontend::parse_nuis_module("mod cpu Main { struct Wide { value:i64 } }").unwrap();
    module.structs[0].fields = (0..4096)
        .map(|i| {
            let mut field = module.structs[0].fields[0].clone();
            field.name = format!("f{i}");
            field
        })
        .collect();
    let layouts = control_values::TypedLayouts::collect(&module);
    assert!(control_values::supported_type(
        &scalar_type("Wide"),
        &layouts
    ));
    assert!(!bounded(&[discarded("Wide")], &[], &layouts));
}
