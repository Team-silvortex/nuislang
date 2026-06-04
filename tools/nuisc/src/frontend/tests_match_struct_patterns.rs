use super::parse_nuis_module;
use nuis_semantics::model::{NirBinaryOp, NirExpr, NirStmt};

#[test]
fn lowers_struct_field_match_arms_inside_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Packet {
            kind: i64,
            ready: bool,
          }

          fn main() -> i64 {
            let armed: bool = true;
            let packet: Packet = Packet { kind: 2, ready: true };
            while 1 == 1 {
              match packet {
                Packet { kind: 1 | 2, ready: true } if armed => {
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
                            NirExpr::Binary {
                                op: NirBinaryOp::And,
                                ..
                            } => {}
                            other => panic!(
                                "expected field conjunction term in struct match condition, found {other:?}"
                            ),
                        }
                        assert!(matches!(
                            rhs.as_ref(),
                            NirExpr::Bool(true) | NirExpr::Var(_)
                        ));
                    }
                    other => panic!(
                        "expected `and` condition for guarded struct match arm, found {other:?}"
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
            other => panic!("expected lowered struct match if in while body, found {other:?}"),
        },
        other => panic!("expected while statement after bindings, found {other:?}"),
    }
}

#[test]
fn lowers_nested_struct_match_arms_inside_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Header {
            kind: i64,
            ready: bool,
          }

          struct Packet {
            header: Header,
            code: i64,
          }

          fn main() -> i64 {
            let armed: bool = true;
            let packet: Packet = Packet {
              header: Header { kind: 2, ready: true },
              code: 5,
            };
            while 1 == 1 {
              match packet {
                Packet { header: Header { kind: 1 | 2, ready: true }, code: 5 } if armed => {
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
                            NirExpr::Binary {
                                op: NirBinaryOp::And,
                                ..
                            } => {}
                            other => panic!(
                                "expected nested field conjunction term in struct match condition, found {other:?}"
                            ),
                        }
                        assert!(matches!(rhs.as_ref(), NirExpr::Bool(true) | NirExpr::Var(_)));
                    }
                    other => panic!(
                        "expected `and` condition for guarded nested struct match arm, found {other:?}"
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
            other => {
                panic!("expected lowered nested struct match if in while body, found {other:?}")
            }
        },
        other => panic!("expected while statement after bindings, found {other:?}"),
    }
}
