use super::*;

pub(super) fn source(ty: &str) -> String {
    format!(
        "mod cpu Main {{
        struct Inner {{ value: {ty} }}
        struct Packet {{ inner: Inner, tag: i64, flag: bool }}
        struct Other {{ inner: Inner, tag: i64, flag: bool }}
        @noinline fn leaf(value: {ty}, divisor: i64) -> Packet {{
            print(41); let checked = 0 / divisor;
            return Packet {{ inner: Inner {{ value: value }}, tag: checked, flag: true }};
        }}
        fn owner(outer: bool, gate: bool, inner: bool, left: {ty}, right: {ty}, divisor: i64) -> {ty} {{
            if outer {{
                let joined: Packet = if gate {{
                    let staged: Packet = if inner {{
                        let local = leaf(left, divisor); print(local.tag); local
                    }} else {{
                        Packet {{ inner: Inner {{ value: right }}, tag: 0 / divisor, flag: false }}
                    }};
                    print(staged.tag); staged
                }} else {{
                    let staged: Packet = if inner {{
                        Packet {{ inner: Inner {{ value: left }}, tag: 0, flag: true }}
                    }} else {{
                        Packet {{ inner: Inner {{ value: right }}, tag: 0, flag: false }}
                    }};
                    print(staged.tag); staged
                }};
                let later = joined; print(42); return later.inner.value;
            }}
            let staged = leaf(right, divisor);
            let joined = staged;
            let later = joined; print(43); return later.inner.value;
        }}
        fn main() -> i64 {{ return 0; }}
    }}"
    )
}

#[test]
fn iteration_record_join_exports_exact_owned_layouts_and_keeps_hygiene() {
    for ty in ["bool", "i32", "i64", "f32", "f64"] {
        for reversed in [false, true] {
            let mut module = crate::frontend::parse_nuis_module(&source(ty)).unwrap();
            if reversed {
                module.functions.reverse();
                module.structs.reverse();
            }
            let layouts = control_values::layouts(&module);
            let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
            assert!(catalog.contains_key("owner"), "{ty}");
            assert!(!scalar_helpers::collect_with_layouts(&module, &layouts).contains_key("owner"));
            let owner = module.functions.iter().find(|f| f.name == "owner").unwrap();
            let scope = owner
                .params
                .iter()
                .map(|p| (p.name.clone(), p.ty.clone()))
                .collect();
            let NirStmt::If { then_body, .. } = &owner.body[0] else {
                panic!("outer")
            };
            assert!(prefixed::declaration(
                &then_body[0],
                &scope,
                &then_body[1..],
                &catalog,
                &layouts
            )
            .is_none());
            let retained = catalog.keys().cloned().collect();
            let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
            let mut helpers = Vec::new();
            let mut guarded = BTreeSet::new();
            scalar_control::outline(
                &mut module,
                &retained,
                &mut names,
                &mut helpers,
                &mut guarded,
                &catalog,
                &layouts,
            );
            let joins = helpers
                .iter()
                .filter(|f| f.name.starts_with("__nuis_scalar_value_join"))
                .collect::<Vec<_>>();
            assert_eq!(joins.len(), 3);
            assert!(joins
                .iter()
                .all(|f| f.return_type == Some(scalar_type("Packet"))));
            assert!(helpers.iter().all(|f| f
                .params
                .iter()
                .all(|p| p.name != "staged" && p.name != "local")));
            module.functions.extend(helpers);
            crate::nir_verify::verify_nir_module(&module).unwrap();
        }
    }
}

#[test]
fn iteration_record_join_rejects_nominal_shape_scope_and_effect_leaks() {
    let original = source("i64");
    for (from, to) in [
        (
            "let later = joined; print(42)",
            "let later = staged; print(42)",
        ),
        ("value: right", "value: local.inner.value"),
        ("value: right", "value: joined.inner.value"),
        ("value: right", "value: staged.inner.value"),
        ("tag: 0 / divisor, flag: false", "tag: 0 / divisor, flag: 0"),
        ("tag: 0 / divisor, flag: false", "tag: 0 / divisor"),
        (
            "tag: 0 / divisor, flag: false",
            "tag: 0 / divisor, tag: 1, flag: false",
        ),
        (
            "Packet { inner: Inner { value: right }, tag: 0 / divisor, flag: false }",
            "Other { inner: Inner { value: right }, tag: 0 / divisor, flag: false }",
        ),
        ("let staged: Packet", "let staged: Other"),
        ("print(staged.tag)", "print(staged.flag)"),
        ("print(staged.tag)", "print(leaf(left, divisor).tag)"),
        ("print(staged.tag)", "print(joined.tag)"),
        (
            "let local = leaf(left, divisor)",
            "let left = leaf(left, divisor)",
        ),
    ] {
        let text = original.replace(from, to);
        assert_ne!(text, original, "{from}");
        if let Ok(module) = crate::frontend::parse_nuis_module(&text) {
            let before = module.clone();
            let layouts = control_values::layouts(&module);
            assert!(
                !scalar_helpers::collect_iteration_calls(&module, &layouts).contains_key("owner"),
                "{from} -> {to}"
            );
            assert_eq!(before, module);
        }
    }
    for ty in ["&i64", "&mut i64", "string"] {
        if let Ok(module) = crate::frontend::parse_nuis_module(&source(ty)) {
            let layouts = control_values::layouts(&module);
            assert!(!layouts.contains_key("Packet"));
            assert!(
                !scalar_helpers::collect_iteration_calls(&module, &layouts).contains_key("owner")
            );
        }
    }
}

#[test]
fn iteration_record_join_bounds_all_synthetic_record_arms_before_cloning() {
    let binding = |name: &str, ty: &str, value: NirExpr| NirStmt::Let {
        name: name.into(),
        ty: Some(scalar_type(ty)),
        value,
    };
    let child = NirStmt::If {
        condition: NirExpr::Var("gate".into()),
        then_body: vec![binding("staged", "Wide", NirExpr::Var("left".into()))],
        else_body: vec![binding("staged", "Wide", NirExpr::Var("right".into()))],
    };
    let arm = vec![
        child.clone(),
        binding(
            "joined",
            "i64",
            NirExpr::FieldAccess {
                base: Box::new(NirExpr::Var("staged".into())),
                field: "f0".into(),
            },
        ),
    ];
    let join = NirStmt::If {
        condition: NirExpr::Var("gate".into()),
        then_body: arm.clone(),
        else_body: arm,
    };
    let scope = BTreeMap::from([
        ("gate".into(), scalar_type("bool")),
        ("left".into(), scalar_type("Wide")),
        ("right".into(), scalar_type("Wide")),
    ]);
    let suffix = [NirStmt::Return(Some(NirExpr::Var("joined".into())))];
    let catalog = BTreeMap::new();
    for (width, expected) in [(1023, true), (1024, false)] {
        let layouts = BTreeMap::from([(
            "Wide".into(),
            (0..width)
                .map(|i| (format!("f{i}"), scalar_type("i64")))
                .collect::<Vec<_>>(),
        )]);
        assert!(
            prefixed_declaration(
                &child,
                &scope,
                &[NirStmt::Print(NirExpr::FieldAccess {
                    base: Box::new(NirExpr::Var("staged".into())),
                    field: "f0".into(),
                })],
                &catalog,
                &layouts
            )
            .is_some(),
            "one child fits"
        );
        assert_eq!(
            prefixed_declaration(&join, &scope, &suffix, &catalog, &layouts).is_some(),
            expected,
            "four neutral records share one 4096-node budget: width {width}"
        );
    }
}
