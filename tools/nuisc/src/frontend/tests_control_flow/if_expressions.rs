use super::*;

#[test]
fn control_expression_initializers_preserve_explicit_function_exits_and_tail_values() {
    for declaration in ["let", "const"] {
        for expression in [
            "if true { return 7; } else { true }",
            "if true { true } else { return 7; }",
            "match 1 { 1 => { return 7; }, _ => { true } }",
            "if true { return 7; } else if false { true } else { false }",
        ] {
            let text = format!("mod cpu Main {{ fn main() -> i64 {{ {declaration} chosen: bool = {expression}; return if chosen {{ 11 }} else {{ 19 }}; }} }}");
            let module = parse_nuis_module(&text).unwrap();
            let main = module.functions.iter().find(|f| f.name == "main").unwrap();
            let body = format!("{:?}", main.body[0]);
            assert!(body.contains("Return(Some(Int(7)))"), "{body}");
            assert!(!body.contains("value: Int(7)"), "{body}");
            assert!(body.contains("name: \"chosen\""), "{body}");
            let alternate = parse_nuis_module(&text.replace("return 7;", "return false;")).unwrap();
            let main = alternate
                .functions
                .iter()
                .find(|f| f.name == "main")
                .unwrap();
            assert!(format!("{:?}", main.body[0]).contains("Return(Some(Bool(false)))"));
            assert!(parse_nuis_module(&text.replace("{ true }", "{ chosen }")).is_err());
        }
    }
}

#[test]
fn control_expression_let_results_are_not_published_before_branch_validation() {
    for expression in [
        "if true { chosen } else { 9 }",
        "match 1 { 1 => { chosen }, _ => { 9 } }",
        "if chosen > 0 { 7 } else { 9 }",
        "match chosen { 1 => { 7 }, _ => { 9 } }",
    ] {
        let text = format!("mod cpu Main {{ fn main() -> i64 {{ let chosen: i64 = {expression}; return chosen; }} }}");
        assert!(parse_nuis_module(&text).is_err(), "{expression}");
    }
    let text = "mod cpu Main { fn main() -> i64 { let chosen: i64 = if true { let local = 7; local } else { 9 }; return local; } }";
    assert!(parse_nuis_module(text).is_err());
}

#[test]
fn control_expression_generic_tail_hints_do_not_leak_to_non_tail_calls() {
    for expression in [
        "if true { zero() } else { return 19; }",
        "if true { let local = zero(); local } else { return 19; }",
        "match 1 { 1 => { zero() }, _ => { return 19; } }",
        "match 1 { 1 => { let local = zero(); local }, _ => { return 19; } }",
    ] {
        let text = format!("mod cpu Main {{ fn zero<T>() -> T {{ return 0; }} fn main() -> i64 {{ let chosen: i64 = {expression}; return chosen; }} }}");
        let module = parse_nuis_module(&text).unwrap();
        assert!(
            module
                .functions
                .iter()
                .any(|f| f.name != "main"
                    && f.return_type.as_ref().is_some_and(|ty| ty.name == "i64"))
        );
        let not_tail = text.replacen("zero()", "zero(); zero()", 1);
        assert!(parse_nuis_module(&not_tail).is_err());
    }
}

#[test]
fn const_control_expression_results_are_visible_only_after_validated_branches() {
    for expression in [
        "if true { let local = 7; local } else { let local = 9; local }",
        "match 1 { 1 => { let local = 7; local }, _ => { let local = 9; local } }",
    ] {
        let text = format!("mod cpu Main {{ fn main() -> i64 {{ const chosen: i64 = {expression}; return chosen; }} }}");
        let module = parse_nuis_module(&text).unwrap();
        let main = module.functions.iter().find(|f| f.name == "main").unwrap();
        assert!(
            matches!(main.body.last(), Some(NirStmt::Return(Some(NirExpr::Var(name)))) if name == "chosen")
        );
        assert!(parse_nuis_module(&text.replace("return chosen;", "return local;")).is_err());
        assert!(parse_nuis_module(&text.replace("let local = 7; local", "chosen")).is_err());
        assert!(parse_nuis_module(&text.replace("let local = 9; local", "true")).is_err());
        assert!(parse_nuis_module(&text.replace("const chosen: i64", "const chosen")).is_err());
    }
}

#[test]
fn lowers_if_expression_in_let_initializer() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = if true {
              7
            } else {
              9
            };
            return value;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    match &function.body[0] {
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            assert!(matches!(condition, NirExpr::Bool(true)));
            assert!(matches!(
                then_body.as_slice(),
                [NirStmt::Let {
                    name,
                    value: NirExpr::Int(7),
                    ..
                }] if name == "value"
            ));
            assert!(matches!(
                else_body.as_slice(),
                [NirStmt::Let {
                    name,
                    value: NirExpr::Int(9),
                    ..
                }] if name == "value"
            ));
        }
        other => panic!("expected lowered if-expression let binding, found {other:?}"),
    }
}

#[test]
fn lowers_if_expression_in_return_position() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            return if false {
              1
            } else {
              2
            };
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    match &function.body[0] {
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            assert!(matches!(condition, NirExpr::Bool(false)));
            assert!(matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Int(1)))]
            ));
            assert!(matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Int(2)))]
            ));
        }
        other => panic!("expected lowered if-expression return, found {other:?}"),
    }
}

#[test]
fn lowers_tail_if_expression_without_explicit_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            if false {
              1
            } else {
              2
            }
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    match &function.body[0] {
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            assert!(matches!(condition, NirExpr::Bool(false)));
            assert!(matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Int(1)))]
            ));
            assert!(matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Int(2)))]
            ));
        }
        other => panic!("expected lowered tail if-expression return, found {other:?}"),
    }
}

#[test]
fn lowers_workflow_style_if_expression_chain_without_empty_branches() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Report {
            overall_success: bool,
            executed: bool
          }

          fn main() -> i64 {
            let report: Report = Report { overall_success: true, executed: false };
            let overall_bonus: i64 = if report.overall_success {
              1
            } else {
              0
            };
            let executed_bonus: i64 = if report.executed {
              1
            } else {
              0
            };
            return overall_bonus + executed_bonus;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    let if_count = function
        .body
        .iter()
        .filter(|stmt| matches!(stmt, NirStmt::If { .. }))
        .count();
    assert_eq!(if_count, 2);
    for stmt in &function.body {
        if let NirStmt::If {
            then_body,
            else_body,
            ..
        } = stmt
        {
            assert!(!then_body.is_empty(), "then branch should not be empty");
            assert!(!else_body.is_empty(), "else branch should not be empty");
        }
    }
}

#[test]
fn lowers_tail_match_expression_without_explicit_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            match 1 {
              1 => { 7 },
              _ => { 9 }
            }
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    match &function.body[0] {
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            assert!(matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Int(7)))]
            ));
            assert!(matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Int(9)))]
            ));
        }
        other => panic!("expected lowered tail match-expression return, found {other:?}"),
    }
}

#[test]
fn lowers_if_expression_inside_call_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn pick(value: i64) -> i64 {
            return value;
          }

          fn main() -> i64 {
            let value: i64 = pick(if true {
              7
            } else {
              9
            });
            return value;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    match &function.body[0] {
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            assert!(matches!(
                then_body.as_slice(),
                [NirStmt::Let {
                    name,
                    value: NirExpr::Call { .. },
                    ..
                }] if name == "value"
            ));
            assert!(matches!(
                else_body.as_slice(),
                [NirStmt::Let {
                    name,
                    value: NirExpr::Call { .. },
                    ..
                }] if name == "value"
            ));
        }
        other => panic!("expected lowered if-expression around call argument, found {other:?}"),
    }
}

#[test]
fn lowers_if_expression_inside_binary_operand() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 1 + if false {
              2
            } else {
              3
            };
            return value;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    match &function.body[0] {
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            assert!(matches!(
                then_body.as_slice(),
                [NirStmt::Let {
                    value: NirExpr::Binary { .. },
                    ..
                }]
            ));
            assert!(matches!(
                else_body.as_slice(),
                [NirStmt::Let {
                    value: NirExpr::Binary { .. },
                    ..
                }]
            ));
        }
        other => panic!("expected lowered if-expression around binary operand, found {other:?}"),
    }
}

#[test]
fn lowers_if_expression_inside_struct_field_value() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Packet {
            value: i64
          }

          fn main() -> i64 {
            let packet: Packet = Packet {
              value: if true {
                7
              } else {
                9
              }
            };
            return packet.value;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    match &function.body[0] {
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            assert!(matches!(
                then_body.as_slice(),
                [NirStmt::Let {
                    name,
                    value: NirExpr::StructLiteral { .. },
                    ..
                }] if name == "packet"
            ));
            assert!(matches!(
                else_body.as_slice(),
                [NirStmt::Let {
                    name,
                    value: NirExpr::StructLiteral { .. },
                    ..
                }] if name == "packet"
            ));
        }
        other => {
            panic!("expected lowered if-expression around struct field value, found {other:?}")
        }
    }
}

#[test]
fn lowers_if_expression_inside_method_receiver() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn main() -> i64 {
            return (if true {
              7
            } else {
              9
            }).add(3);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    match &function.body[0] {
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            assert!(matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { .. }))]
            ));
            assert!(matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { .. }))]
            ));
        }
        other => panic!("expected lowered if-expression around method receiver, found {other:?}"),
    }
}

#[test]
fn lowers_if_expression_inside_await_operand() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn one() -> i64 {
            return 1;
          }

          async fn two() -> i64 {
            return 2;
          }

          async fn main() -> i64 {
            let value: i64 = await if true {
              one()
            } else {
              two()
            };
            return value;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    match &function.body[0] {
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            assert!(matches!(
                then_body.as_slice(),
                [NirStmt::Let {
                    name,
                    value: NirExpr::Await(_),
                    ..
                }] if name == "value"
            ));
            assert!(matches!(
                else_body.as_slice(),
                [NirStmt::Let {
                    name,
                    value: NirExpr::Await(_),
                    ..
                }] if name == "value"
            ));
        }
        other => panic!("expected lowered if-expression around await operand, found {other:?}"),
    }
}

#[test]
fn lowers_match_expression_inside_await_operand() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn one() -> i64 {
            return 1;
          }

          async fn two() -> i64 {
            return 2;
          }

          async fn main() -> i64 {
            let value: i64 = await match 1 {
              1 => { one() },
              _ => { two() }
            };
            return value;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    match &function.body[0] {
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            assert!(matches!(
                then_body.as_slice(),
                [NirStmt::Let {
                    name,
                    value: NirExpr::Await(_),
                    ..
                }] if name == "value"
            ));
            assert!(matches!(
                else_body.as_slice(),
                [NirStmt::Let {
                    name,
                    value: NirExpr::Await(_),
                    ..
                }] if name == "value"
            ));
        }
        other => panic!("expected lowered match-expression around await operand, found {other:?}"),
    }
}

#[test]
fn lowers_tail_await_expression_without_explicit_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn one() -> i64 {
            return 1;
          }

          async fn main() -> i64 {
            await one()
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Await(_)))]
    ));
}
