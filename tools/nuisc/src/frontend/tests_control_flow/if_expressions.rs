use super::*;

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
