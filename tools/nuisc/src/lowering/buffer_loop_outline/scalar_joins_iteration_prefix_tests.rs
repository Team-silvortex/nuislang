use super::*;

fn source(ty: &str) -> String {
    format!(
        "mod cpu Main {{
        @noinline fn leaf(value: {ty}, divisor: i64) -> {ty} {{
            print(41); let check = 0 / divisor; return value;
        }}
        fn owner(outer: bool, gate: bool, left: {ty}, right: {ty}, divisor: i64) -> {ty} {{
            if outer {{
                let joined: {ty} = if gate {{
                    let staged = leaf(left, divisor);
                    const marker: i64 = 1;
                    let checked = marker / divisor;
                    staged
                }} else {{
                    let staged = right;
                    let checked = 0 / divisor;
                    staged
                }};
                let later = joined; print(42); return later;
            }}
            let joined: {ty} = if gate {{
                let staged = leaf(right, divisor);
                let checked = 0 / divisor;
                staged
            }} else {{
                let staged = left;
                let checked = 0 / divisor;
                staged
            }};
            let later = joined; print(43); return later;
        }}
        fn main() -> i64 {{ return 0; }}
    }}"
    )
}

#[test]
fn iteration_prefix_join_exports_only_exact_result_and_keeps_colliding_locals_private() {
    for ty in ["bool", "i32", "i64", "f32", "f64"] {
        for reversed in [false, true] {
            let mut module = crate::frontend::parse_nuis_module(&source(ty)).unwrap();
            if reversed {
                module.functions.reverse();
            }
            let layouts = control_values::layouts(&module);
            let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
            assert!(catalog.contains_key("owner"), "{ty}");
            assert!(!scalar_helpers::collect_with_layouts(&module, &layouts).contains_key("owner"));
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
            assert!(joins.iter().all(|f| f.return_type == Some(scalar_type(ty))));
            assert!(helpers.iter().all(|f| f
                .params
                .iter()
                .all(|p| p.name != "staged" && p.name != "checked" && p.name != "marker")));
            module.functions.extend(helpers);
            crate::nir_verify::verify_nir_module(&module).unwrap();
        }
    }
}

#[test]
fn iteration_prefix_join_rejects_leaks_rebinding_forward_reads_and_unregistered_effects() {
    let original = source("i64");
    for (from, to) in [
        ("let later = joined", "let later = staged"),
        ("let staged = right", "let staged = checked"),
        ("let checked = 0 / divisor;", "let staged = 0 / divisor;"),
        ("const marker: i64 = 1;", "const marker: bool = true;"),
        (
            "let staged = leaf(left, divisor);",
            "let left = leaf(left, divisor);",
        ),
        ("let checked = 0 / divisor;", "print(44);"),
        ("let checked = 0 / divisor;", "let checked = gate && gate;"),
        ("leaf(left, divisor)", "missing(left, divisor)"),
        ("if gate {", "if divisor {"),
        ("let staged = right;", "let staged = false;"),
    ] {
        let text = original.replace(from, to);
        assert_ne!(text, original);
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
}

#[test]
fn iteration_prefix_join_shares_arm_root_budgets_before_inference() {
    let module = crate::frontend::parse_nuis_module(&source("i64")).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let owner = module.functions.iter().find(|f| f.name == "owner").unwrap();
    let scope = owner
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    let NirStmt::If {
        then_body: outer, ..
    } = &owner.body[0]
    else {
        panic!("outer")
    };
    let mut join = outer[0].clone();
    let arm = |count: usize, value: NirExpr| {
        let mut result = (0..count - 1)
            .map(|i| NirStmt::Let {
                name: format!("slot_{i}"),
                ty: None,
                value: value.clone(),
            })
            .collect::<Vec<_>>();
        result.push(NirStmt::Let {
            name: "joined".into(),
            ty: Some(scalar_type("i64")),
            value: NirExpr::Var("left".into()),
        });
        result
    };
    let accepts = |stmt: &NirStmt| {
        prefixed_declaration(stmt, &scope, &outer[1..], &catalog, &layouts).is_some()
    };
    for (count, expected) in [(32, true), (33, false)] {
        let NirStmt::If {
            then_body,
            else_body,
            ..
        } = &mut join
        else {
            panic!("join")
        };
        *then_body = arm(count, NirExpr::Int(1));
        *else_body = arm(1, NirExpr::Int(2));
        assert_eq!(accepts(&join), expected);
    }
    let mut tree = NirExpr::Int(1);
    for _ in 0..6 {
        tree = NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(tree.clone()),
            rhs: Box::new(tree),
        };
    }
    let NirStmt::If {
        then_body,
        else_body,
        ..
    } = &mut join
    else {
        panic!("join")
    };
    *then_body = arm(32, tree.clone());
    *else_body = arm(1, NirExpr::Int(1));
    assert!(accepts(&join));
    let NirStmt::If { else_body, .. } = &mut join else {
        panic!("join")
    };
    *else_body = arm(32, tree);
    assert!(
        !accepts(&join),
        "two separately valid arms must share one source budget"
    );
    for (depth, expected) in [(63, true), (64, false)] {
        let mut value = NirExpr::Int(1);
        for _ in 0..depth {
            value = NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(value),
                rhs: Box::new(NirExpr::Int(1)),
            };
        }
        let NirStmt::If {
            then_body,
            else_body,
            ..
        } = &mut join
        else {
            panic!("join")
        };
        *then_body = arm(2, value);
        *else_body = arm(1, NirExpr::Int(1));
        assert_eq!(accepts(&join), expected, "depth {depth}");
    }
}

#[test]
fn iteration_prefix_join_keeps_repeated_value_boundaries_linear() {
    let mut text = String::from("mod cpu Main { @noinline fn leaf(value: i64) -> i64 { print(value); return value; } fn owner(gate: bool, value: i64, divisor: i64) -> i64 {");
    for i in 0..64 {
        text.push_str(&format!("let joined_{i}: i64 = if gate {{ let staged = leaf(value); let checked = 0 / divisor; staged }} else {{ let staged = value; let checked = 0 / divisor; staged }}; print(joined_{i});"));
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
