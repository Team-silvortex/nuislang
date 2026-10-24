use super::*;

fn module() -> NirModule {
    crate::frontend::parse_nuis_module(include_str!(
        "../../../tests/native_application_bridge/guarded_scalar_join.ns"
    ))
    .unwrap()
}

#[test]
fn guarded_scalar_join_requires_completed_pure_dependencies_and_is_idempotent() {
    let mut module = module();
    let layouts = control_values::TypedLayouts::collect(&module);
    let pure = scalar_helpers::collect_capture_values(&module, &layouts);
    assert!(pure.contains_key("quotient"));
    assert!(!pure.contains_key("packet"));
    let mut reordered = module.clone();
    reordered.functions.reverse();
    let reordered_pure = scalar_helpers::collect_capture_values(&reordered, &layouts);
    let mut reordered_names = reordered.functions.iter().map(|f| f.name.clone()).collect();
    assert_eq!(
        conditional_values::outline_declarations(
            &mut reordered,
            &reordered_pure,
            &layouts,
            &mut reordered_names
        )
        .len(),
        1
    );
    let f = module
        .functions
        .iter()
        .find(|f| f.name == "packet")
        .unwrap();
    let scope = f
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect::<Scope>();
    assert!(declaration(&f.body[1], &scope, &f.body[2..]).is_none());
    assert!(guarded_declaration(&f.body[1], &scope, &f.body[2..], &pure, &layouts).is_some());
    let mut missing = pure.clone();
    missing.remove("quotient");
    assert!(guarded_declaration(&f.body[1], &scope, &f.body[2..], &missing, &layouts).is_none());
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated =
        conditional_values::outline_declarations(&mut module, &pure, &layouts, &mut names);
    assert_eq!(generated.len(), 1);
    let carries = control_values::layouts(&module);
    assert!(scalar_helpers::collect_iteration_calls(&module, &carries).contains_key("packet"));
    assert!(!scalar_helpers::collect_capture_values(&module, &layouts).contains_key("packet"));
    let before = module.clone();
    assert!(
        conditional_values::outline_declarations(&mut module, &pure, &layouts, &mut names)
            .is_empty()
    );
    assert_eq!(module, before);
    module.functions.retain(|f| f.name != "quotient");
    assert!(!scalar_helpers::collect_iteration_calls(&module, &carries).contains_key("packet"));
}

#[test]
fn guarded_scalar_join_bounds_syntax_and_keeps_scope_gate_and_effect_vetoes() {
    let module = module();
    let layouts = control_values::TypedLayouts::collect(&module);
    let pure = scalar_helpers::collect_capture_values(&module, &layouts);
    let f = module
        .functions
        .iter()
        .find(|f| f.name == "packet")
        .unwrap();
    let scope = f
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect::<Scope>();
    for mutation in 0..10 {
        let mut stmt = f.body[1].clone();
        let mut scope = scope.clone();
        let NirStmt::If {
            condition,
            then_body,
            else_body,
        } = &mut stmt
        else {
            panic!()
        };
        match mutation {
            0 => {
                scope.insert("selected".into(), scalar_type("i64"));
            }
            1 => {
                *condition = NirExpr::Binary {
                    op: NirBinaryOp::Eq,
                    lhs: Box::new(NirExpr::Int(1)),
                    rhs: Box::new(NirExpr::Int(1)),
                }
            }
            2 => then_body.insert(0, NirStmt::Print(NirExpr::Int(88))),
            3 => else_body.clear(),
            _ => {
                let NirStmt::Let { ty, value, .. } = &mut then_body[0] else {
                    panic!()
                };
                match mutation {
                    4 => *ty = None,
                    5 => {
                        *value = NirExpr::Call {
                            callee: "packet".into(),
                            args: vec![],
                        }
                    }
                    6 => {
                        *value = NirExpr::Call {
                            callee: "quotient".into(),
                            args: vec![NirExpr::Int(1); 4097],
                        }
                    }
                    7 => {
                        for _ in 0..64 {
                            *value = NirExpr::Binary {
                                op: NirBinaryOp::Add,
                                lhs: Box::new(value.clone()),
                                rhs: Box::new(NirExpr::Int(0)),
                            };
                        }
                    }
                    8 => {
                        *value = NirExpr::Binary {
                            op: NirBinaryOp::And,
                            lhs: Box::new(NirExpr::Bool(false)),
                            rhs: Box::new(NirExpr::Bool(true)),
                        }
                    }
                    9 => {
                        let mut tree = NirExpr::Int(1);
                        for _ in 0..11 {
                            tree = NirExpr::Binary {
                                op: NirBinaryOp::Add,
                                lhs: Box::new(tree.clone()),
                                rhs: Box::new(tree),
                            };
                        }
                        *value = tree.clone();
                        let NirStmt::Let { value, .. } = &mut else_body[0] else {
                            panic!()
                        };
                        *value = tree;
                    }
                    _ => unreachable!(),
                }
            }
        }
        assert!(
            guarded_declaration(&stmt, &scope, &f.body[2..], &pure, &layouts).is_none(),
            "{mutation}"
        );
    }
}

#[test]
fn guarded_scalar_join_uses_shared_scalar_types_for_both_arms() {
    for (input, output, expression) in [
        ("f32", "f32", "a + b"),
        ("f64", "f64", "a * b"),
        ("i32", "bool", "a < b"),
        ("i64", "bool", "a != b"),
        ("bool", "bool", "a == b"),
    ] {
        let source = format!("mod cpu Main {{ fn pick(gate: bool, a: {input}, b: {input}, fallback: {output}) -> {output} {{ let value: {output} = if gate {{ {expression} }} else {{ fallback }}; return value; }} }}");
        let module = crate::frontend::parse_nuis_module(&source).unwrap();
        let layouts = control_values::TypedLayouts::collect(&module);
        let pure = scalar_helpers::collect_capture_values(&module, &layouts);
        let f = &module.functions[0];
        let scope = f
            .params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        assert!(
            guarded_declaration(&f.body[0], &scope, &f.body[1..], &pure, &layouts).is_some(),
            "{input}"
        );
    }
    let module = crate::frontend::parse_nuis_module("mod cpu Main { fn pick(gate: bool, a: bool, b: bool) -> bool { let value: bool = if gate { a && b } else { false }; return value; } }").unwrap();
    let layouts = control_values::TypedLayouts::collect(&module);
    let pure = scalar_helpers::collect_capture_values(&module, &layouts);
    let f = &module.functions[0];
    let scope = f
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    assert!(
        guarded_declaration(&f.body[0], &scope, &f.body[1..], &pure, &layouts).is_none(),
        "well-typed logical arms need their own guard composition"
    );
}

#[test]
fn guarded_scalar_join_keeps_checked_work_inside_selected_helper_arms() {
    for (op, left, right, expected) in [
        ("+", 7, 3, 10),
        ("-", 7, 3, 4),
        ("*", 7, 3, 21),
        ("/", 7, 3, 2),
        ("%", -7, 3, -1),
        ("+", i64::MAX, 1, i64::MIN),
        ("-", i64::MIN, 1, i64::MAX),
        ("*", i64::MIN, -1, i64::MIN),
    ] {
        for gate in [false, true] {
            // Keep unary-literal parsing separate from the scalar value proof.
            let literal = |value: i64| {
                if value == i64::MIN {
                    format!("(-{} - 1)", i64::MAX)
                } else {
                    value.to_string()
                }
            };
            let (left, right) = (literal(left), literal(right));
            let source = format!(
                "mod cpu Main {{
                @noinline fn pick(choice: bool, a: i64, b: i64) -> i64 {{
                    print(7); let value: i64 = if choice {{ a {op} b }} else {{ 13 }}; return value;
                }}
                fn main() -> i64 {{ return pick({gate}, {left}, {right}); }}
            }}"
            );
            super::super::conditional_returns::tests::execute(
                &source,
                Some(if gate { expected } else { 13 }),
                &[7],
                0,
            );
        }
    }
    for gate in [false, true] {
        let source = format!(
            "mod cpu Main {{
            @noinline fn pick(choice: bool, divisor: i64) -> i64 {{
                print(7); let value: i64 = if choice {{ 1 / divisor }} else {{ 13 }}; return value;
            }}
            fn main() -> i64 {{ return pick({gate}, 0); }}
        }}"
        );
        super::super::conditional_returns::tests::execute(&source, (!gate).then_some(13), &[7], 0);
    }
}
