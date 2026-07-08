use super::*;

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
