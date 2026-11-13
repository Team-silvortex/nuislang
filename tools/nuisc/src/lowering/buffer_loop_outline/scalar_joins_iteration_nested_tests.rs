use super::*;

fn source(ty: &str) -> String {
    format!(
        "mod cpu Main {{
        @noinline fn leaf(value: {ty}, divisor: i64) -> {ty} {{
            print(41); let check = 0 / divisor; return value;
        }}
        fn owner(outer: bool, gate: bool, inner: bool, left: {ty}, right: {ty}, divisor: i64) -> {ty} {{
            if outer {{
                let joined: {ty} = if gate {{
                    let staged: {ty} = if inner {{ let local = leaf(left, divisor); print(51); local }} else {{ let local = right; print(52); local }};
                    const marker: i64 = 1; print(marker);
                    let checked = marker / divisor;
                    staged
                }} else {{
                    let staged: {ty} = if inner {{ let local = right; print(53); local }} else {{ let local = left; print(54); local }};
                    let checked = 0 / divisor; print(divisor);
                    staged
                }};
                let later = joined; print(42); return later;
            }}
            let joined: {ty} = if gate {{
                let staged: {ty} = if inner {{ let local = leaf(right, divisor); print(51); local }} else {{ let local = left; print(52); local }};
                let checked = 0 / divisor; print(divisor);
                staged
            }} else {{
                let staged: {ty} = if inner {{ let local = left; print(53); local }} else {{ let local = right; print(54); local }};
                let checked = 0 / divisor; print(divisor);
                staged
            }};
            let later = joined; print(43); return later;
        }}
        fn main() -> i64 {{ return 0; }}
    }}"
    )
}

#[test]
fn iteration_nested_join_exports_only_exact_result_and_keeps_colliding_locals_private() {
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
            assert_eq!(joins.len(), 6);
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
fn iteration_nested_join_rejects_leaks_rebinding_forward_reads_and_unregistered_effects() {
    let original = source("i64");
    for (from, to) in [
        ("let later = joined", "let later = local"),
        ("let later = joined", "let later = staged"),
        ("let local = right;", "let local = joined;"),
        ("let local = right;", "let local = staged;"),
        ("let local = right;", "let local = checked;"),
        ("let local = right;", "let local = false;"),
        ("let local = right;", "let left = right;"),
        ("print(51);", "print(local == left);"),
        ("print(51);", "print(leaf(local, divisor));"),
        ("print(51);", "print(staged);"),
        ("if inner {", "if divisor {"),
        (
            "let checked = 0 / divisor;",
            "let checked: i64 = gate && gate;",
        ),
        ("leaf(left, divisor)", "missing(left, divisor)"),
    ] {
        let text = original.replace(from, to);
        assert!(text != original, "missing mutation target: {from}");
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

fn result(name: &str, value: NirExpr) -> NirStmt {
    NirStmt::Let {
        name: name.into(),
        ty: Some(scalar_type("i64")),
        value,
    }
}
fn pair(_name: &str, yes: Vec<NirStmt>, no: Vec<NirStmt>) -> NirStmt {
    NirStmt::If {
        condition: NirExpr::Var("gate".into()),
        then_body: yes,
        else_body: no,
    }
}
fn tree(depth: usize, name: &str) -> NirStmt {
    let yes = if depth == 0 {
        vec![
            NirStmt::Print(NirExpr::Int(1)),
            result(name, NirExpr::Var("left".into())),
        ]
    } else {
        let child = format!("nested_{depth}");
        vec![tree(depth - 1, &child), result(name, NirExpr::Var(child))]
    };
    pair(name, yes, vec![result(name, NirExpr::Var("right".into()))])
}

#[test]
fn iteration_nested_join_shares_statement_expression_and_depth_budgets() {
    let module = crate::frontend::parse_nuis_module(&source("i64")).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let owner = module.functions.iter().find(|f| f.name == "owner").unwrap();
    let scope = owner
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    let suffix = vec![NirStmt::Return(Some(NirExpr::Var("joined".into())))];
    let accepts =
        |stmt: &NirStmt| prefixed_declaration(stmt, &scope, &suffix, &catalog, &layouts).is_some();
    for (depth, expected) in [(8, true), (9, false)] {
        assert_eq!(accepts(&tree(depth, "joined")), expected, "depth {depth}");
    }
    let arm = |name: &str, count: usize, value: &NirExpr| {
        let mut body = vec![NirStmt::Print(value.clone()); count - 1];
        body.push(result(name, NirExpr::Var("left".into())));
        body
    };
    let wrap =
        |name: &str, child: NirStmt| vec![child, result("joined", NirExpr::Var(name.into()))];
    let mut bounded = pair(
        "joined",
        wrap(
            "yes",
            pair(
                "yes",
                arm("yes", 31, &NirExpr::Int(1)),
                arm("yes", 31, &NirExpr::Int(1)),
            ),
        ),
        wrap(
            "no",
            pair(
                "no",
                arm("no", 31, &NirExpr::Int(1)),
                arm("no", 31, &NirExpr::Int(1)),
            ),
        ),
    );
    assert!(accepts(&bounded), "128 shared statements");
    let NirStmt::If { then_body, .. } = &mut bounded else {
        panic!()
    };
    let NirStmt::If { then_body, .. } = &mut then_body[0] else {
        panic!()
    };
    then_body.insert(0, NirStmt::Print(NirExpr::Int(1)));
    assert!(!accepts(&bounded), "129 shared statements");

    let mut value = NirExpr::Int(1);
    for _ in 0..6 {
        value = NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(value.clone()),
            rhs: Box::new(value),
        };
    }
    let heavy = |name: &str| pair(name, arm(name, 16, &value), arm(name, 16, &value));
    let one = pair(
        "joined",
        wrap("yes", heavy("yes")),
        vec![result("joined", NirExpr::Int(1))],
    );
    assert!(accepts(&one));
    let both = pair("joined", wrap("yes", heavy("yes")), wrap("no", heavy("no")));
    assert!(
        !accepts(&both),
        "nested expression work must not reset per child"
    );

    let mut ancestor = tree(1, "joined");
    let NirStmt::If { then_body, .. } = &mut ancestor else {
        panic!()
    };
    then_body[0] = pair(
        "joined",
        vec![result("joined", NirExpr::Int(1))],
        vec![result("joined", NirExpr::Int(2))],
    );
    assert!(
        !accepts(&ancestor),
        "child cannot publish an ancestor destination"
    );
}

#[test]
fn iteration_nested_join_keeps_repeated_value_boundaries_linear() {
    let mut text = String::from("mod cpu Main { @noinline fn leaf(v: i64) -> i64 { print(v); return v; } fn owner(gate: bool, value: i64) -> i64 {");
    for i in 0..16 {
        text.push_str(&format!("let joined_{i}: i64 = if gate {{ let staged: i64 = if gate {{ let local = leaf(value); print(local); local }} else {{ let local = value; print(local); local }}; print(staged); staged }} else {{ let local = value; print(local); local }}; print(joined_{i});"));
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
        32
    );
    assert!(helpers.len() <= 96);
    assert_eq!(
        module
            .functions
            .iter()
            .find(|f| f.name == "owner")
            .unwrap()
            .body
            .len(),
        33
    );
    module.functions.extend(helpers);
    crate::nir_verify::verify_nir_module(&module).unwrap();
}
