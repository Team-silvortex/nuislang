use super::parse_nuis_module;
use nuis_semantics::model::{NirBinaryOp, NirExpr, NirStmt};

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
