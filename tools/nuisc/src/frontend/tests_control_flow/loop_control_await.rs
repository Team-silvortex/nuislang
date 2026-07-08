use super::*;

#[test]
fn lowers_nested_if_break_continue_control_inside_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            while value < 3 {
              let value: i64 = value + 1;
              if value > 1 {
                break;
              } else {
                if value < 1 {
                  continue;
                } else {
                  break;
                }
              }
            }
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
    match &function.body[1] {
        NirStmt::While { body, .. } => match body.as_slice() {
            [NirStmt::Let {
                name,
                value:
                    NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        ..
                    },
                ..
            }, NirStmt::If {
                condition,
                then_body,
                else_body,
            }] if name == "value" => {
                assert!(matches!(
                    condition,
                    NirExpr::Binary {
                        op: NirBinaryOp::Gt,
                        ..
                    }
                ));
                assert!(matches!(then_body.as_slice(), [NirStmt::Break]));
                assert!(matches!(
                    else_body.as_slice(),
                    [NirStmt::If {
                        condition: NirExpr::Binary {
                            op: NirBinaryOp::Lt,
                            ..
                        },
                        then_body,
                        else_body,
                    }] if matches!(then_body.as_slice(), [NirStmt::Continue])
                        && matches!(else_body.as_slice(), [NirStmt::Break])
                ));
            }
            other => {
                panic!("expected nested break/continue if chain in while body, found {other:?}")
            }
        },
        other => panic!("expected while statement, found {other:?}"),
    }
}

#[test]
fn lowers_nested_match_break_continue_control_inside_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            while value < 4 {
              let value: i64 = value + 1;
              match value {
                1 => {
                  continue;
                }
                2 => {
                  break;
                }
                _ => {
                  break;
                }
              }
            }
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
    match &function.body[1] {
        NirStmt::While { body, .. } => match body.as_slice() {
            [NirStmt::Let {
                name,
                value:
                    NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        ..
                    },
                ..
            }, NirStmt::If {
                condition,
                then_body,
                else_body,
            }] if name == "value" => {
                assert!(matches!(
                    condition,
                    NirExpr::Binary {
                        op: NirBinaryOp::Eq,
                        rhs,
                        ..
                    } if matches!(rhs.as_ref(), NirExpr::Int(1))
                ));
                assert!(matches!(then_body.as_slice(), [NirStmt::Continue]));
                assert!(matches!(
                    else_body.as_slice(),
                    [NirStmt::If {
                        condition: NirExpr::Binary {
                            op: NirBinaryOp::Eq,
                            rhs,
                            ..
                        },
                        then_body,
                        else_body,
                    }] if matches!(rhs.as_ref(), NirExpr::Int(2))
                        && matches!(then_body.as_slice(), [NirStmt::Break])
                        && matches!(else_body.as_slice(), [NirStmt::Break])
                ));
            }
            other => panic!(
                "expected nested match-lowered break/continue chain in while body, found {other:?}"
            ),
        },
        other => panic!("expected while statement, found {other:?}"),
    }
}

#[test]
fn reports_control_expr_branch_tail_requirement_with_terminal_loop_control_hint() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main(flag: bool) -> i64 {
            let value: i64 = if flag {
              let local: i64 = 1;
            } else {
              2
            };
            return value;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("tail expression result or terminal loop control"),
        "{error}"
    );
    assert!(error.contains("`break`/`continue`"), "{error}");
}

#[test]
fn lowers_tail_match_stmt_inside_if_statement_branch_without_implicit_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main(flag: bool) -> i64 {
            if flag {
              let state: i64 = 1;
              match state {
                1 => {
                  let picked: i64 = 7;
                }
                _ => {
                  let picked: i64 = 9;
                }
              }
            } else {
              let fallback: i64 = 0;
            }
            return 3;
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
                [
                    NirStmt::Let { name, .. },
                    NirStmt::If {
                        then_body: nested_then,
                        else_body: nested_else,
                        ..
                    }
                ] if name == "state"
                    && matches!(nested_then.as_slice(), [NirStmt::Let { name, .. }] if name == "picked")
                    && matches!(nested_else.as_slice(), [NirStmt::Let { name, .. }] if name == "picked")
            ));
            assert!(matches!(
                else_body.as_slice(),
                [NirStmt::Let { name, .. }] if name == "fallback"
            ));
        }
        other => panic!("expected if statement, found {other:?}"),
    }
    assert!(matches!(
        function.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Int(3))))
    ));
}

#[test]
fn lowers_tail_await_stmt_inside_if_statement_branch_without_implicit_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn one() -> i64 {
            return 1;
          }

          async fn compute(flag: bool) -> i64 {
            if flag {
              await one();
            } else {
              await one();
            }
            return 3;
          }

          async fn main() -> i64 {
            return await compute(true);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "compute")
        .unwrap();
    match &function.body[0] {
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            assert!(matches!(
                then_body.as_slice(),
                [NirStmt::Await(NirExpr::Call { .. })]
            ));
            assert!(matches!(
                else_body.as_slice(),
                [NirStmt::Await(NirExpr::Call { .. })]
            ));
        }
        other => panic!("expected if statement, found {other:?}"),
    }
    assert!(matches!(
        function.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Int(3))))
    ));
}

#[test]
fn lowers_await_expression_inside_while_body() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 3 {
              let value: i64 = await step(value);
              let acc: i64 = acc + value;
            }
            return acc;
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
    match &function.body[2] {
        NirStmt::While { condition, body } => {
            assert!(matches!(
                condition,
                NirExpr::Binary {
                    op: NirBinaryOp::Lt,
                    ..
                }
            ));
            assert!(matches!(
                body.as_slice(),
                [
                    NirStmt::Let {
                        name: value_name,
                        value: NirExpr::Await(_),
                        ..
                    },
                    NirStmt::Let {
                        name: acc_name,
                        value: NirExpr::Binary {
                            op: NirBinaryOp::Add,
                            ..
                        },
                        ..
                    }
                ] if value_name == "value" && acc_name == "acc"
            ));
        }
        other => panic!("expected while statement with await body, found {other:?}"),
    }
}
