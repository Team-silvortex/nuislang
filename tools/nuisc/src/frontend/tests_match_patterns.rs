use super::parse_nuis_module;
use nuis_semantics::model::{NirBinaryOp, NirExpr, NirStmt};

#[test]
fn lowers_multi_pattern_match_arms_inside_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            while 1 == 1 {
              match 2 {
                1 | 2 => {
                  return 7;
                }
                _ => {
                  return 9;
                }
              }
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::While { body, .. } => match &body[0] {
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                match condition {
                    NirExpr::Binary {
                        op: NirBinaryOp::Or,
                        lhs,
                        rhs,
                    } => {
                        for side in [lhs.as_ref(), rhs.as_ref()] {
                            match side {
                                NirExpr::Binary { op, rhs, .. } => {
                                    assert_eq!(*op, NirBinaryOp::Eq);
                                    assert!(matches!(rhs.as_ref(), NirExpr::Int(1) | NirExpr::Int(2)));
                                }
                                other => panic!(
                                    "expected equality term in multi-pattern condition, found {other:?}"
                                ),
                            }
                        }
                    }
                    other => panic!(
                        "expected `or` condition for multi-pattern match arm, found {other:?}"
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
fn lowers_inclusive_range_match_arms_inside_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            while 1 == 1 {
              match 2 {
                1..=3 => {
                  return 7;
                }
                _ => {
                  return 9;
                }
              }
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::While { body, .. } => match &body[0] {
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                match condition {
                    NirExpr::Binary {
                        op: NirBinaryOp::And,
                        lhs,
                        rhs,
                    } => {
                        match lhs.as_ref() {
                            NirExpr::Binary { op, rhs, .. } => {
                                assert_eq!(*op, NirBinaryOp::Ge);
                                assert!(matches!(rhs.as_ref(), NirExpr::Int(1)));
                            }
                            other => panic!(
                                "expected lower-bound comparison in range match, found {other:?}"
                            ),
                        }
                        match rhs.as_ref() {
                            NirExpr::Binary { op, rhs, .. } => {
                                assert_eq!(*op, NirBinaryOp::Le);
                                assert!(matches!(rhs.as_ref(), NirExpr::Int(3)));
                            }
                            other => panic!(
                                "expected upper-bound comparison in range match, found {other:?}"
                            ),
                        }
                    }
                    other => {
                        panic!("expected `and` condition for range match arm, found {other:?}")
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
fn lowers_match_guard_arms_inside_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let ready: bool = true;
            while 1 == 1 {
              match 2 {
                2 if ready => {
                  return 7;
                }
                _ => {
                  return 9;
                }
              }
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::While { body, .. } => match &body[0] {
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                match condition {
                    NirExpr::Binary {
                        op: NirBinaryOp::And,
                        lhs,
                        rhs,
                    } => {
                        match lhs.as_ref() {
                            NirExpr::Binary { op, rhs, .. } => {
                                assert_eq!(*op, NirBinaryOp::Eq);
                                assert!(matches!(rhs.as_ref(), NirExpr::Int(2)));
                            }
                            other => panic!(
                                "expected equality term in guarded match condition, found {other:?}"
                            ),
                        }
                        assert!(matches!(
                            rhs.as_ref(),
                            NirExpr::Bool(true) | NirExpr::Var(_)
                        ));
                    }
                    other => {
                        panic!("expected `and` condition for guarded match arm, found {other:?}")
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
        other => panic!("expected while statement after let binding, found {other:?}"),
    }
}

#[test]
fn lowers_multiple_guarded_match_arms_inside_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let ready: bool = true;
            let armed: bool = false;
            while 1 == 1 {
              match 2 {
                1 if armed => {
                  return 5;
                }
                2 if ready => {
                  return 7;
                }
                _ => {
                  return 9;
                }
              }
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[2] {
        NirStmt::While { body, .. } => match &body[0] {
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                match condition {
                    NirExpr::Binary {
                        op: NirBinaryOp::And,
                        lhs,
                        rhs,
                    } => {
                        match lhs.as_ref() {
                            NirExpr::Binary { op, rhs, .. } => {
                                assert_eq!(*op, NirBinaryOp::Eq);
                                assert!(matches!(rhs.as_ref(), NirExpr::Int(1)));
                            }
                            other => panic!(
                                "expected equality term in first guarded match condition, found {other:?}"
                            ),
                        }
                        assert!(matches!(
                            rhs.as_ref(),
                            NirExpr::Bool(false) | NirExpr::Var(_)
                        ));
                    }
                    other => panic!(
                        "expected `and` condition for first guarded match arm, found {other:?}"
                    ),
                }
                assert!(matches!(
                    then_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(5)))]
                ));
                match else_body.as_slice() {
                    [NirStmt::If {
                        condition,
                        then_body,
                        else_body,
                    }] => {
                        match condition {
                            NirExpr::Binary {
                                op: NirBinaryOp::And,
                                lhs,
                                rhs,
                            } => {
                                match lhs.as_ref() {
                                    NirExpr::Binary { op, rhs, .. } => {
                                        assert_eq!(*op, NirBinaryOp::Eq);
                                        assert!(matches!(rhs.as_ref(), NirExpr::Int(2)));
                                    }
                                    other => panic!(
                                        "expected equality term in second guarded match condition, found {other:?}"
                                    ),
                                }
                                assert!(matches!(rhs.as_ref(), NirExpr::Bool(true) | NirExpr::Var(_)));
                            }
                            other => panic!(
                                "expected `and` condition for second guarded match arm, found {other:?}"
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
                    other => panic!(
                        "expected nested if chain for multiple guarded match arms, found {other:?}"
                    ),
                }
            }
            other => panic!("expected lowered match if in while body, found {other:?}"),
        },
        other => panic!("expected while statement after let bindings, found {other:?}"),
    }
}

#[test]
fn lowers_or_pattern_guard_match_arms_inside_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let ready: bool = true;
            while 1 == 1 {
              match 2 {
                1 | 2 if ready => {
                  return 7;
                }
                _ => {
                  return 9;
                }
              }
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::While { body, .. } => match &body[0] {
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                match condition {
                    NirExpr::Binary {
                        op: NirBinaryOp::And,
                        lhs,
                        rhs,
                    } => {
                        match lhs.as_ref() {
                            NirExpr::Binary {
                                op: NirBinaryOp::Or,
                                ..
                            } => {}
                            other => panic!(
                                "expected `or` term in guarded multi-pattern match condition, found {other:?}"
                            ),
                        }
                        assert!(matches!(rhs.as_ref(), NirExpr::Bool(true) | NirExpr::Var(_)));
                    }
                    other => panic!(
                        "expected `and` condition for guarded multi-pattern match arm, found {other:?}"
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
            other => panic!("expected lowered guarded match if in while body, found {other:?}"),
        },
        other => panic!("expected while statement after let binding, found {other:?}"),
    }
}

#[test]
fn lowers_range_guard_match_arms_inside_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let ready: bool = true;
            while 1 == 1 {
              match 2 {
                1..=3 if ready => {
                  return 7;
                }
                _ => {
                  return 9;
                }
              }
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::While { body, .. } => match &body[0] {
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                match condition {
                    NirExpr::Binary {
                        op: NirBinaryOp::And,
                        lhs,
                        rhs,
                    } => {
                        match lhs.as_ref() {
                            NirExpr::Binary {
                                op: NirBinaryOp::And,
                                ..
                            } => {}
                            other => panic!(
                                "expected range conjunction term in guarded range match condition, found {other:?}"
                            ),
                        }
                        assert!(matches!(
                            rhs.as_ref(),
                            NirExpr::Bool(true) | NirExpr::Var(_)
                        ));
                    }
                    other => panic!(
                        "expected `and` condition for guarded range match arm, found {other:?}"
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
            other => panic!("expected lowered guarded match if in while body, found {other:?}"),
        },
        other => panic!("expected while statement after let binding, found {other:?}"),
    }
}
