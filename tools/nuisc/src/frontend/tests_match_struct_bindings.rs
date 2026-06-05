use super::{parse_nuis_ast, parse_nuis_module};
use nuis_semantics::model::{AstMatchPattern, NirBinaryOp, NirExpr, NirStmt};

#[test]
fn parses_struct_match_field_binding_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          struct Packet {
            kind: i64,
            ready: bool,
          }

          fn main() -> i64 {
            let packet: Packet = Packet { kind: 2, ready: true };
            match packet {
              Packet { kind: packet_kind, ready: true } => {
                return packet_kind;
              }
              _ => {
                return 9;
              }
            }
          }
        }
        "#,
    )
    .unwrap();

    match &ast.functions[0].body[1] {
        nuis_semantics::model::AstStmt::Match { arms, .. } => match &arms[0].pattern {
            AstMatchPattern::StructFields { fields, .. } => {
                assert!(matches!(
                    &fields[0],
                    (field, AstMatchPattern::Bind(name))
                        if field == "kind" && name == "packet_kind"
                ));
                assert!(matches!(
                    &fields[1],
                    (field, AstMatchPattern::Bool(true))
                        if field == "ready"
                ));
            }
            other => panic!("expected struct match pattern, found {other:?}"),
        },
        other => panic!("expected match statement, found {other:?}"),
    }
}

#[test]
fn parses_nested_struct_match_field_binding_shorthand_into_ast() {
    let ast = parse_nuis_ast(
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
            let packet: Packet = Packet {
              header: Header { kind: 2, ready: true },
              code: 5,
            };
            match packet {
              Packet { header: { kind: packet_kind, ready: true }, code: 5 } => {
                return packet_kind;
              }
              _ => {
                return 9;
              }
            }
          }
        }
        "#,
    )
    .unwrap();

    match &ast.functions[0].body[1] {
        nuis_semantics::model::AstStmt::Match { arms, .. } => match &arms[0].pattern {
            AstMatchPattern::StructFields { fields, .. } => {
                assert!(matches!(
                    &fields[0],
                    (
                        field,
                        AstMatchPattern::StructFields {
                            type_ref: None,
                            fields: nested_fields
                        }
                    ) if field == "header"
                        && matches!(
                            &nested_fields[0],
                            (nested_field, AstMatchPattern::Bind(name))
                                if nested_field == "kind" && name == "packet_kind"
                        )
                ));
            }
            other => panic!("expected struct match pattern, found {other:?}"),
        },
        other => panic!("expected match statement, found {other:?}"),
    }
}

#[test]
fn lowers_nested_struct_field_binding_shorthand_match_arms_inside_while() {
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
            let packet: Packet = Packet {
              header: Header { kind: 2, ready: true },
              code: 5,
            };
            while 1 == 1 {
              match packet {
                Packet { header: { kind: packet_kind, ready: true }, code: 5 } => {
                  return packet_kind;
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
            NirStmt::If { then_body, .. } => {
                assert!(matches!(
                    then_body.as_slice(),
                    [
                        NirStmt::Let { name, value, .. },
                        NirStmt::Return(Some(NirExpr::Var(result)))
                    ] if name == "packet_kind"
                        && result == "packet_kind"
                        && matches!(
                            value,
                            NirExpr::FieldAccess { field, base }
                                if field == "kind"
                                    && matches!(
                                        &**base,
                                        NirExpr::FieldAccess { field, .. } if field == "header"
                                    )
                        )
                ));
            }
            other => panic!(
                "expected lowered shorthand nested struct match if in while body, found {other:?}"
            ),
        },
        other => panic!("expected while statement after binding, found {other:?}"),
    }
}

#[test]
fn lowers_struct_field_binding_visible_in_guard_inside_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Packet {
            kind: i64,
            ready: bool,
          }

          fn main() -> i64 {
            let packet: Packet = Packet { kind: 2, ready: true };
            while 1 == 1 {
              match packet {
                Packet { kind: packet_kind, ready: true } if packet_kind == 2 => {
                  return packet_kind;
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
                        lhs: _,
                        rhs
                    } if matches!(
                        rhs.as_ref(),
                        NirExpr::Binary {
                            op: NirBinaryOp::Eq,
                            lhs,
                            rhs
                        } if matches!(
                            lhs.as_ref(),
                            NirExpr::FieldAccess { field, .. } if field == "kind"
                        )
                            && matches!(rhs.as_ref(), NirExpr::Int(2))
                    )
                ));
                assert!(matches!(
                    then_body.as_slice(),
                    [
                        NirStmt::Let { name, .. },
                        NirStmt::Return(Some(NirExpr::Var(result)))
                    ] if name == "packet_kind" && result == "packet_kind"
                ));
                assert!(matches!(
                    else_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(9)))]
                ));
            }
            other => {
                panic!("expected lowered guarded struct match if in while body, found {other:?}")
            }
        },
        other => panic!("expected while statement after binding, found {other:?}"),
    }
}

#[test]
fn lowers_nested_struct_field_binding_shorthand_visible_in_guard_inside_while() {
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
            let packet: Packet = Packet {
              header: Header { kind: 2, ready: true },
              code: 5,
            };
            while 1 == 1 {
              match packet {
                Packet { header: { kind: packet_kind, ready: true }, code: 5 }
                    if packet_kind == 2 =>
                {
                  return packet_kind;
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
                        lhs: _,
                        rhs
                    } if matches!(
                        rhs.as_ref(),
                        NirExpr::Binary {
                            op: NirBinaryOp::Eq,
                            lhs,
                            rhs
                        } if matches!(
                            lhs.as_ref(),
                            NirExpr::FieldAccess { field, base }
                                if field == "kind"
                                    && matches!(
                                        base.as_ref(),
                                        NirExpr::FieldAccess { field, .. } if field == "header"
                                    )
                        )
                            && matches!(rhs.as_ref(), NirExpr::Int(2))
                    )
                ));
                assert!(matches!(
                    then_body.as_slice(),
                    [
                        NirStmt::Let { name, .. },
                        NirStmt::Return(Some(NirExpr::Var(result)))
                    ] if name == "packet_kind" && result == "packet_kind"
                ));
                assert!(matches!(
                    else_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(9)))]
                ));
            }
            other => panic!(
                "expected lowered guarded nested shorthand struct match if in while body, found {other:?}"
            ),
        },
        other => panic!("expected while statement after binding, found {other:?}"),
    }
}

#[test]
fn rejects_struct_field_binding_inside_multi_pattern_arm() {
    let err = parse_nuis_ast(
        r#"
        mod cpu Main {
          struct Packet {
            kind: i64,
            ready: bool,
          }

          fn main() -> i64 {
            let packet: Packet = Packet { kind: 2, ready: true };
            match packet {
              Packet { kind: packet_kind | 2, ready: true } => {
                return 1;
              }
              _ => {
                return 9;
              }
            }
          }
        }
        "#,
    )
    .unwrap_err();

    assert_eq!(
        err,
        "minimal struct field match patterns do not allow `_` or bindings inside `|` multi-pattern arms; use a standalone binding arm or move the extra condition into a guard"
    );
}

#[test]
fn rejects_struct_field_binding_mixed_with_range_inside_multi_pattern_arm() {
    let err = parse_nuis_ast(
        r#"
        mod cpu Main {
          struct Packet {
            kind: i64,
            ready: bool,
          }

          fn main() -> i64 {
            let packet: Packet = Packet { kind: 2, ready: true };
            match packet {
              Packet { kind: 1..=3 | packet_kind, ready: true } => {
                return 1;
              }
              _ => {
                return 9;
              }
            }
          }
        }
        "#,
    )
    .unwrap_err();

    assert_eq!(
        err,
        "minimal struct field match patterns do not allow `_` or bindings inside `|` multi-pattern arms; use a standalone binding arm or move the extra condition into a guard"
    );
}

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
