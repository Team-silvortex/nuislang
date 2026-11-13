use super::*;

fn source(ty: &str) -> String {
    format!("mod cpu Main {{
        @noinline fn leaf(value: {ty}, divisor: i64) -> {ty} {{
            print(41); let checked = 0 / divisor; return value;
        }}
        @noinline fn owner(outer: bool, gate: bool, left: {ty}, right: {ty}, divisor: i64) -> {ty} {{
            if outer {{
                let joined: {ty} = if gate {{ leaf(left, divisor) }} else {{ right }};
                let later = joined; print(42); return later;
            }}
            return right;
        }}
        fn main() -> i64 {{ return 0; }}
    }}")
}

#[test]
fn iteration_value_join_retains_exact_owned_types_and_private_return_boundaries() {
    for ty in ["bool", "i32", "i64", "f32", "f64"] {
        for reversed in [false, true] {
            let mut module = crate::frontend::parse_nuis_module(&source(ty)).unwrap();
            if reversed {
                module.functions.reverse();
            }
            let layouts = control_values::layouts(&module);
            let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
            let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
            assert!(!pure.contains_key("owner"));
            assert!(catalog.contains_key("owner"), "{ty}");
            let before = module.clone();
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
            assert_eq!(joins.len(), 1);
            assert_eq!(joins[0].return_type, Some(scalar_type(ty)));
            assert!(!guarded.is_empty());
            assert_eq!(
                module.functions.iter().find(|f| f.name == "leaf"),
                before.functions.iter().find(|f| f.name == "leaf")
            );
            module.functions.extend(helpers);
            crate::nir_verify::verify_nir_module(&module).unwrap();
        }
    }
}

#[test]
fn iteration_value_join_rejects_incomplete_mismatched_or_preexisting_destinations() {
    for mutation in [
        "one-arm",
        "name",
        "type",
        "undeclared",
        "prefix",
        "constant",
        "outer",
        "missing-leaf",
    ] {
        let mut module = crate::frontend::parse_nuis_module(&source("i64")).unwrap();
        if mutation == "missing-leaf" {
            module.functions.retain(|f| f.name != "leaf");
        } else {
            let owner = module
                .functions
                .iter_mut()
                .find(|f| f.name == "owner")
                .unwrap();
            let NirStmt::If {
                then_body: outer, ..
            } = &mut owner.body[0]
            else {
                panic!("outer branch")
            };
            let NirStmt::If {
                then_body,
                else_body,
                ..
            } = &mut outer[0]
            else {
                panic!("joined value")
            };
            match mutation {
                "one-arm" => else_body.clear(),
                "prefix" => then_body.insert(0, NirStmt::Print(NirExpr::Bool(true))),
                "constant" => {
                    let NirStmt::Let { name, ty, value } = else_body[0].clone() else {
                        panic!("let")
                    };
                    else_body[0] = NirStmt::Const {
                        name,
                        ty: ty.unwrap(),
                        value,
                    };
                }
                "outer" => {}
                _ => {
                    let NirStmt::Let { name, ty, value } = &mut else_body[0] else {
                        panic!("let")
                    };
                    match mutation {
                        "name" => *name = "other".into(),
                        "type" => {
                            *ty = Some(scalar_type("bool"));
                            *value = NirExpr::Bool(false);
                        }
                        "undeclared" => *value = NirExpr::Var("not_ready".into()),
                        _ => unreachable!(),
                    }
                }
            }
            if mutation == "outer" {
                owner.params.push(NirParam {
                    name: "joined".into(),
                    ty: scalar_type("i64"),
                });
            }
        }
        let before = module.clone();
        let layouts = control_values::layouts(&module);
        assert!(
            !scalar_helpers::collect_iteration_calls(&module, &layouts).contains_key("owner"),
            "{mutation}"
        );
        assert_eq!(module, before);
    }
}

#[test]
fn iteration_value_join_keeps_shared_suffix_growth_linear() {
    let mut text = String::from("mod cpu Main { @noinline fn leaf(value: i64) -> i64 { print(value); return value; } fn owner(gate: bool, value: i64) -> i64 {");
    for i in 0..64 {
        text.push_str(&format!(
            "let joined_{i}: i64 = if gate {{ leaf(value) }} else {{ value }}; print(joined_{i});"
        ));
    }
    text.push_str("return value; } fn main() -> i64 { return 0; } }");
    let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    assert!(catalog.contains_key("owner"));
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
    assert_eq!(
        helpers
            .iter()
            .filter(|f| f.name.starts_with("__nuis_scalar_value_join"))
            .count(),
        64
    );
    assert!(helpers.len() <= 64 * 3);
    assert_eq!(
        module
            .functions
            .iter()
            .find(|f| f.name == "owner")
            .unwrap()
            .body
            .len(),
        129
    );
    module.functions.extend(helpers);
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn iteration_value_join_preserves_one_identity_when_moving_a_colliding_suffix() {
    let text = "mod cpu Main {
        @noinline fn leaf(value: i64) -> i64 { print(value); return value; }
        fn owner(outer: bool, gate: bool, left: i64, right: i64) -> bool {
            if outer {
                let next = leaf(left);
                let joined: i64 = if gate { leaf(next) } else { next + 1 };
                let later = joined ^ 4294967296; print(later); return later != 0;
            }
            let next = leaf(right);
            let joined: i64 = if gate { leaf(next) } else { next - 1 };
            let later = joined ^ 8589934592; print(later); return later != 0;
        }
        fn main() -> i64 { return 0; }
    }";
    let mut module = crate::frontend::parse_nuis_module(text).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    assert!(catalog.contains_key("owner"));
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
    assert_eq!(joins.len(), 2);
    assert!(joins
        .iter()
        .all(|f| f.return_type == Some(scalar_type("i64"))));
    assert!(helpers
        .iter()
        .filter(|f| f.name.starts_with("__nuis_scalar_branch"))
        .any(|f| f.return_type == Some(scalar_type("bool"))));
    module.functions.extend(helpers);
    crate::nir_verify::verify_nir_module(&module).unwrap();
}
