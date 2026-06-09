use super::parse_nuis_module;
use nuis_semantics::model::{NirBinaryOp, NirExpr, NirStmt};

#[test]
fn lowers_payload_style_struct_match_binding_visible_in_guard_inside_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Just {
            value: i64,
          }

          fn main() -> i64 {
            let value: Just = Just(2);
            while 1 == 1 {
              match value {
                Just(payload) if payload == 2 => {
                  return payload;
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
                assert!(matches!(
                    condition,
                    NirExpr::Binary {
                        op: NirBinaryOp::And,
                        lhs,
                        rhs
                    } if matches!(lhs.as_ref(), NirExpr::Bool(true))
                        && matches!(
                            rhs.as_ref(),
                            NirExpr::Binary {
                                op: NirBinaryOp::Eq,
                                lhs,
                                rhs
                            } if matches!(
                                lhs.as_ref(),
                                NirExpr::FieldAccess { field, .. } if field == "value"
                            ) && matches!(rhs.as_ref(), NirExpr::Int(2))
                        )
                ));
                assert!(matches!(
                    then_body.as_slice(),
                    [
                        NirStmt::Let { name, .. },
                        NirStmt::Return(Some(NirExpr::Var(result)))
                    ] if name == "payload" && result == "payload"
                ));
                assert!(matches!(
                    else_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(9)))]
                ));
            }
            other => panic!(
                "expected lowered payload-style guarded struct match if in while body, found {other:?}"
            ),
        },
        other => panic!("expected while statement after payload value binding, found {other:?}"),
    }
}

#[test]
fn lowers_generic_payload_style_struct_match_binding_visible_in_guard_inside_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let value: Just<i64> = Just(2);
            while 1 == 1 {
              match value {
                Just(payload) if payload == 2 => {
                  return payload;
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
                assert!(matches!(
                    condition,
                    NirExpr::Binary {
                        op: NirBinaryOp::And,
                        lhs,
                        rhs
                    } if matches!(lhs.as_ref(), NirExpr::Bool(true))
                        && matches!(
                            rhs.as_ref(),
                            NirExpr::Binary {
                                op: NirBinaryOp::Eq,
                                lhs,
                                rhs
                            } if matches!(
                                lhs.as_ref(),
                                NirExpr::FieldAccess { field, .. } if field == "value"
                            ) && matches!(rhs.as_ref(), NirExpr::Int(2))
                        )
                ));
                assert!(matches!(
                    then_body.as_slice(),
                    [
                        NirStmt::Let { name, ty, .. },
                        NirStmt::Return(Some(NirExpr::Var(result)))
                    ] if name == "payload"
                        && result == "payload"
                        && matches!(ty, Some(ty) if ty.render() == "i64")
                ));
                assert!(matches!(
                    else_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(9)))]
                ));
            }
            other => panic!(
                "expected lowered generic payload-style guarded struct match if in while body, found {other:?}"
            ),
        },
        other => {
            panic!("expected while statement after generic payload value binding, found {other:?}")
        }
    }
}

#[test]
fn lowers_explicit_generic_payload_style_struct_match_binding_visible_in_guard_inside_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let value = Just<i64>(2);
            while 1 == 1 {
              match value {
                Just<i64>(payload) if payload == 2 => {
                  return payload;
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
                assert!(matches!(
                    condition,
                    NirExpr::Binary {
                        op: NirBinaryOp::And,
                        lhs,
                        rhs
                    } if matches!(lhs.as_ref(), NirExpr::Bool(true))
                        && matches!(
                            rhs.as_ref(),
                            NirExpr::Binary {
                                op: NirBinaryOp::Eq,
                                lhs,
                                rhs
                            } if matches!(
                                lhs.as_ref(),
                                NirExpr::FieldAccess { field, .. } if field == "value"
                            ) && matches!(rhs.as_ref(), NirExpr::Int(2))
                        )
                ));
                assert!(matches!(
                    then_body.as_slice(),
                    [
                        NirStmt::Let { name, ty, .. },
                        NirStmt::Return(Some(NirExpr::Var(result)))
                    ] if name == "payload"
                        && result == "payload"
                        && matches!(ty, Some(ty) if ty.render() == "i64")
                ));
                assert!(matches!(
                    else_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(9)))]
                ));
            }
            other => panic!(
                "expected lowered explicit generic payload guarded struct match if in while body, found {other:?}"
            ),
        },
        other => panic!("expected while statement after payload value binding, found {other:?}"),
    }
}

#[test]
fn lowers_generic_alias_payload_style_struct_match_binding_visible_in_guard_inside_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;

          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let value = JustAlias<i64>(2);
            while 1 == 1 {
              match value {
                JustAlias<i64>(payload) if payload == 2 => {
                  return payload;
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
                assert!(matches!(
                    condition,
                    NirExpr::Binary {
                        op: NirBinaryOp::And,
                        lhs,
                        rhs
                    } if matches!(lhs.as_ref(), NirExpr::Bool(true))
                        && matches!(
                            rhs.as_ref(),
                            NirExpr::Binary {
                                op: NirBinaryOp::Eq,
                                lhs,
                                rhs
                            } if matches!(
                                lhs.as_ref(),
                                NirExpr::FieldAccess { field, .. } if field == "value"
                            ) && matches!(rhs.as_ref(), NirExpr::Int(2))
                        )
                ));
                assert!(matches!(
                    then_body.as_slice(),
                    [
                        NirStmt::Let { name, ty, .. },
                        NirStmt::Return(Some(NirExpr::Var(result)))
                    ] if name == "payload"
                        && result == "payload"
                        && matches!(ty, Some(ty) if ty.render() == "i64")
                ));
                assert!(matches!(
                    else_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(9)))]
                ));
            }
            other => panic!(
                "expected lowered generic-alias payload guarded struct match if in while body, found {other:?}"
            ),
        },
        other => panic!("expected while statement after payload alias value binding, found {other:?}"),
    }
}
