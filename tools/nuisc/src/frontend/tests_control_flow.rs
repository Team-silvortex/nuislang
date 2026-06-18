use super::parse_nuis_module;
use nuis_semantics::model::{NirBinaryOp, NirExpr, NirStmt};

#[test]
fn lowers_mutable_local_reassignment_inside_if_branches() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main(flag: bool) -> i64 {
            let mut value: i64 = 1;
            if flag {
              value = value + 2;
            } else {
              value = value + 3;
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
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            assert!(matches!(
                then_body.as_slice(),
                [NirStmt::Let {
                    name,
                    value: NirExpr::Binary { op, lhs, rhs },
                    ..
                }] if name == "value"
                    && *op == NirBinaryOp::Add
                    && matches!(lhs.as_ref(), NirExpr::Var(lhs_name) if lhs_name == "value")
                    && matches!(rhs.as_ref(), NirExpr::Int(2))
            ));
            assert!(matches!(
                else_body.as_slice(),
                [NirStmt::Let {
                    name,
                    value: NirExpr::Binary { op, lhs, rhs },
                    ..
                }] if name == "value"
                    && *op == NirBinaryOp::Add
                    && matches!(lhs.as_ref(), NirExpr::Var(lhs_name) if lhs_name == "value")
                    && matches!(rhs.as_ref(), NirExpr::Int(3))
            ));
        }
        other => panic!("expected if statement after mutable binding, found {other:?}"),
    }
}

#[test]
fn lowers_mutable_local_reassignment_inside_while_body() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let mut value: i64 = 0;
            while value < 3 {
              value += 1;
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
        NirStmt::While { body, .. } => {
            assert!(matches!(
                body.as_slice(),
                [NirStmt::Let {
                    name,
                    value: NirExpr::Binary { op, lhs, rhs },
                    ..
                }] if name == "value"
                    && *op == NirBinaryOp::Add
                    && matches!(lhs.as_ref(), NirExpr::Var(lhs_name) if lhs_name == "value")
                    && matches!(rhs.as_ref(), NirExpr::Int(1))
            ));
        }
        other => panic!("expected while statement after mutable binding, found {other:?}"),
    }
}

#[test]
fn lowers_mutable_local_reassignment_inside_match_arms() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main(flag: bool) -> i64 {
            let mut value: i64 = 1;
            match flag {
              true => {
                value = value + 2;
              }
              _ => {
                value = value + 3;
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
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            assert!(matches!(
                then_body.as_slice(),
                [NirStmt::Let {
                    name,
                    value: NirExpr::Binary { op, lhs, rhs },
                    ..
                }] if name == "value"
                    && *op == NirBinaryOp::Add
                    && matches!(lhs.as_ref(), NirExpr::Var(lhs_name) if lhs_name == "value")
                    && matches!(rhs.as_ref(), NirExpr::Int(2))
            ));
            assert!(matches!(
                else_body.as_slice(),
                [NirStmt::Let {
                    name,
                    value: NirExpr::Binary { op, lhs, rhs },
                    ..
                }] if name == "value"
                    && *op == NirBinaryOp::Add
                    && matches!(lhs.as_ref(), NirExpr::Var(lhs_name) if lhs_name == "value")
                    && matches!(rhs.as_ref(), NirExpr::Int(3))
            ));
        }
        other => panic!("expected match-lowered if after mutable binding, found {other:?}"),
    }
}

#[test]
fn lowers_integer_comparison_into_bool_condition() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            if 2 < 5 {
              return 1;
            } else {
              return 0;
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
        NirStmt::If { condition, .. } => match condition {
            NirExpr::Binary { op, .. } => {
                assert_eq!(*op, nuis_semantics::model::NirBinaryOp::Lt);
            }
            other => panic!("expected comparison condition, found {other:?}"),
        },
        other => panic!("expected if statement, found {other:?}"),
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

#[test]
fn lowers_if_expression_with_branch_prelude_statements() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = if true {
              let local: i64 = 7;
              local
            } else {
              let local: i64 = 9;
              local
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
                [NirStmt::Let { name, .. }, NirStmt::Let { name: value_name, .. }]
                if name == "local" && value_name == "value"
            ));
            assert!(matches!(
                else_body.as_slice(),
                [NirStmt::Let { name, .. }, NirStmt::Let { name: value_name, .. }]
                if name == "local" && value_name == "value"
            ));
        }
        other => panic!("expected lowered if-expression with branch prelude, found {other:?}"),
    }
}

#[test]
fn lowers_match_expression_in_let_initializer() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = match 1 {
              1 => { 7 },
              _ => { 9 }
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
        other => panic!("expected lowered match-expression let binding, found {other:?}"),
    }
}

#[test]
fn lowers_match_expression_inside_call_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn pick(value: i64) -> i64 {
            return value;
          }

          fn main() -> i64 {
            return pick(match 1 {
              1 => { 7 },
              _ => { 9 }
            });
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
        other => panic!("expected lowered match-expression around call argument, found {other:?}"),
    }
}

#[test]
fn lowers_match_expression_with_arm_prelude_statements() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = match 1 {
              1 => {
                let local: i64 = 7;
                local
              },
              _ => {
                let local: i64 = 9;
                local
              }
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
                [NirStmt::Let { name, .. }, NirStmt::Let { name: value_name, .. }]
                if name == "local" && value_name == "value"
            ));
            assert!(matches!(
                else_body.as_slice(),
                [NirStmt::Let { name, .. }, NirStmt::Let { name: value_name, .. }]
                if name == "local" && value_name == "value"
            ));
        }
        other => panic!("expected lowered match-expression with arm prelude, found {other:?}"),
    }
}

#[test]
fn lowers_integer_match_into_nested_ifs() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 1;
            match value {
              0 => { return 0; },
              1 => { return 7; },
              _ => { return 9; }
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
    match &function.body[1] {
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            match condition {
                NirExpr::Binary { op, rhs, .. } => {
                    assert_eq!(*op, NirBinaryOp::Eq);
                    assert!(matches!(rhs.as_ref(), NirExpr::Int(0)));
                }
                other => panic!("expected equality condition, found {other:?}"),
            }
            assert!(matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Int(0)))]
            ));
            match else_body.as_slice() {
                [NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                }] => {
                    match condition {
                        NirExpr::Binary { op, rhs, .. } => {
                            assert_eq!(*op, NirBinaryOp::Eq);
                            assert!(matches!(rhs.as_ref(), NirExpr::Int(1)));
                        }
                        other => panic!("expected nested equality condition, found {other:?}"),
                    }
                    assert!(matches!(
                        then_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(7)))]
                    ));
                    assert!(matches!(
                        else_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(9)))]
                    ));
                }
                other => panic!("expected nested if in match fallback, found {other:?}"),
            }
        }
        other => panic!("expected lowered match as if statement, found {other:?}"),
    }
}

#[test]
fn rejects_match_without_final_wildcard_arm() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 1;
            match value {
              0 => { return 0; },
              1 => { return 1; }
            }
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("requires a final unguarded `_` arm"));
}

#[test]
fn rejects_match_on_non_scalar_scrutinee() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            match "hello" {
              0 => { return 1; },
              _ => { return 0; }
            }
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("integer patterns require an `i64` scrutinee"));
}

#[test]
fn lowers_match_inside_while_body_into_nested_if() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let state: i64 = 1;
            while state > 0 {
              match state {
                1 => { return 7; },
                _ => { return 9; }
              }
            }
            return 0;
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
            [NirStmt::If {
                condition,
                then_body,
                else_body,
            }] => {
                match condition {
                    NirExpr::Binary { op, rhs, .. } => {
                        assert_eq!(*op, NirBinaryOp::Eq);
                        assert!(matches!(rhs.as_ref(), NirExpr::Int(1)));
                    }
                    other => {
                        panic!("expected equality condition in lowered match, found {other:?}")
                    }
                }
                assert!(matches!(
                    then_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(7)))]
                ));
                assert!(matches!(
                    else_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(9)))]
                ));
            }
            other => panic!("expected lowered match if in while body, found {other:?}"),
        },
        other => panic!("expected while statement, found {other:?}"),
    }
}

#[test]
fn lowers_expression_scrutinee_match_inside_while_into_nested_if() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let state: i64 = 2;
            while state > 0 {
              match state + 1 {
                3 => { return 7; },
                _ => { return 9; }
              }
            }
            return 0;
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
            [NirStmt::If {
                condition,
                then_body,
                else_body,
            }] => {
                match condition {
                    NirExpr::Binary { op, lhs, rhs } => {
                        assert_eq!(*op, NirBinaryOp::Eq);
                        assert!(matches!(rhs.as_ref(), NirExpr::Int(3)));
                        assert!(matches!(
                            lhs.as_ref(),
                            NirExpr::Binary {
                                op: NirBinaryOp::Add,
                                ..
                            }
                        ));
                    }
                    other => panic!(
                        "expected equality against expression scrutinee in lowered match, found {other:?}"
                    ),
                }
                assert!(matches!(
                    then_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(7)))]
                ));
                assert!(matches!(
                    else_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(9)))]
                ));
            }
            other => panic!("expected lowered match if in while body, found {other:?}"),
        },
        other => panic!("expected while statement, found {other:?}"),
    }
}

#[test]
fn lowers_bool_match_inside_while_into_nested_if() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let state: i64 = 2;
            while state > 0 {
              match state > 1 {
                true => { return 7; },
                _ => { return 9; }
              }
            }
            return 0;
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
            [NirStmt::If {
                condition,
                then_body,
                else_body,
            }] => {
                match condition {
                    NirExpr::Binary { op: NirBinaryOp::Gt, .. } => {}
                    other => panic!(
                        "expected direct bool expression scrutinee in lowered match, found {other:?}"
                    ),
                }
                assert!(matches!(
                    then_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(7)))]
                ));
                assert!(matches!(
                    else_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(9)))]
                ));
            }
            other => panic!("expected lowered bool match if in while body, found {other:?}"),
        },
        other => panic!("expected while statement, found {other:?}"),
    }
}

#[test]
fn lowers_multi_arm_match_inside_while_into_nested_if_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let state: i64 = 2;
            while state > 0 {
              match state {
                1 => { return 7; },
                2 => { return 8; },
                _ => { return 9; }
              }
            }
            return 0;
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
            [NirStmt::If {
                condition,
                then_body,
                else_body,
            }] => {
                match condition {
                    NirExpr::Binary { op, rhs, .. } => {
                        assert_eq!(*op, NirBinaryOp::Eq);
                        assert!(matches!(rhs.as_ref(), NirExpr::Int(1)));
                    }
                    other => panic!(
                        "expected first equality arm in lowered multi-arm match, found {other:?}"
                    ),
                }
                assert!(matches!(
                    then_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(7)))]
                ));
                match else_body.as_slice() {
                    [NirStmt::If {
                        condition,
                        then_body,
                        else_body,
                    }] => {
                        match condition {
                            NirExpr::Binary { op, rhs, .. } => {
                                assert_eq!(*op, NirBinaryOp::Eq);
                                assert!(matches!(rhs.as_ref(), NirExpr::Int(2)));
                            }
                            other => panic!(
                                "expected second equality arm in lowered multi-arm match, found {other:?}"
                            ),
                        }
                        assert!(matches!(
                            then_body.as_slice(),
                            [NirStmt::Return(Some(NirExpr::Int(8)))]
                        ));
                        assert!(matches!(
                            else_body.as_slice(),
                            [NirStmt::Return(Some(NirExpr::Int(9)))]
                        ));
                    }
                    other => panic!(
                        "expected nested if for second arm in lowered multi-arm match, found {other:?}"
                    ),
                }
            }
            other => {
                panic!("expected lowered multi-arm match if in while body, found {other:?}")
            }
        },
        other => panic!("expected while statement, found {other:?}"),
    }
}

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
