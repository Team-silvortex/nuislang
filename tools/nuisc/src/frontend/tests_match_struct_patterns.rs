use super::{parse_nuis_ast, parse_nuis_module};
use nuis_semantics::model::{AstMatchPattern, NirBinaryOp, NirExpr, NirStmt};

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

#[test]
fn lowers_type_alias_struct_match_arms_inside_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type PacketAlias = Packet;

          struct Packet {
            kind: i64,
            ready: bool,
          }

          fn main() -> i64 {
            let armed: bool = true;
            let packet: Packet = Packet { kind: 2, ready: true };
            while 1 == 1 {
              match packet {
                PacketAlias { kind: 1 | 2, ready: true } if armed => {
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
                                "expected field conjunction term in aliased struct match condition, found {other:?}"
                            ),
                        }
                        assert!(matches!(
                            rhs.as_ref(),
                            NirExpr::Bool(true) | NirExpr::Var(_)
                        ));
                    }
                    other => panic!(
                        "expected `and` condition for guarded aliased struct match arm, found {other:?}"
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
                panic!("expected lowered aliased struct match if in while body, found {other:?}")
            }
        },
        other => panic!("expected while statement after bindings, found {other:?}"),
    }
}

#[test]
fn lowers_zero_field_struct_match_arm_inside_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Done {}

          fn main() -> i64 {
            let armed: bool = true;
            let value: Done = Done {};
            while 1 == 1 {
              match value {
                Done {} if armed => {
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
                assert!(matches!(
                    condition,
                    NirExpr::Binary {
                        op: NirBinaryOp::And,
                        lhs,
                        rhs
                    } if matches!(lhs.as_ref(), NirExpr::Bool(true))
                        && matches!(rhs.as_ref(), NirExpr::Bool(true) | NirExpr::Var(_))
                ));
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
                panic!("expected lowered zero-field struct match if in while body, found {other:?}")
            }
        },
        other => panic!("expected while statement after bindings, found {other:?}"),
    }
}

#[test]
fn parses_shorthand_generic_struct_match_pattern_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let value = Boxed<i64> { value: 7 };
            match value {
              { value: payload } => {
                return payload;
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
            AstMatchPattern::StructFields { type_ref, fields } => {
                assert!(type_ref.is_none());
                assert!(matches!(
                    fields.as_slice(),
                    [(field, AstMatchPattern::Bind(name))]
                        if field == "value" && name == "payload"
                ));
            }
            other => panic!("expected shorthand struct match pattern, found {other:?}"),
        },
        other => panic!("expected match statement, found {other:?}"),
    }
}

#[test]
fn parses_aliased_generic_struct_match_pattern_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          type BoxI64 = Boxed<i64>;

          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let value: BoxI64 = Boxed<i64> { value: 7 };
            match value {
              BoxI64 { value: payload } => {
                return payload;
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
            AstMatchPattern::StructFields { type_ref, fields } => {
                assert_eq!(type_ref.as_ref().unwrap().name, "BoxI64");
                assert!(matches!(
                    fields.as_slice(),
                    [(field, AstMatchPattern::Bind(name))]
                        if field == "value" && name == "payload"
                ));
            }
            other => panic!("expected aliased generic struct match pattern, found {other:?}"),
        },
        other => panic!("expected match statement, found {other:?}"),
    }
}

#[test]
fn parses_generic_aliased_struct_match_pattern_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          type BoxAlias<T> = Boxed<T>;

          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let value: BoxAlias<i64> = Boxed<i64> { value: 7 };
            match value {
              BoxAlias<i64> { value: payload } => {
                return payload;
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
            AstMatchPattern::StructFields { type_ref, fields } => {
                assert_eq!(type_ref.as_ref().unwrap().name, "BoxAlias");
                assert_eq!(type_ref.as_ref().unwrap().generic_args[0].name, "i64");
                assert!(matches!(
                    fields.as_slice(),
                    [(field, AstMatchPattern::Bind(name))]
                        if field == "value" && name == "payload"
                ));
            }
            other => panic!("expected generic-aliased struct match pattern, found {other:?}"),
        },
        other => panic!("expected match statement, found {other:?}"),
    }
}

#[test]
fn rejects_empty_match_pattern_for_non_empty_struct() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Packet {
            kind: i64,
          }

          fn main() -> i64 {
            let value: Packet = Packet { kind: 1 };
            match value {
              Packet {} => {
                return 7;
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

    assert!(error
        .contains("empty struct match pattern `Packet` is only supported for zero-field structs"));
}

#[test]
fn lowers_struct_field_binding_match_arms_inside_while() {
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
                Packet { kind: packet_kind, ready: true } => {
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
                then_body,
                else_body,
                ..
            } => {
                assert!(matches!(
                    then_body.as_slice(),
                    [
                        NirStmt::Let { name, value, .. },
                        NirStmt::Return(Some(NirExpr::Var(result)))
                    ] if name == "packet_kind"
                        && result == "packet_kind"
                        && matches!(value, NirExpr::FieldAccess { field, .. } if field == "kind")
                ));
                assert!(matches!(
                    else_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(9)))]
                ));
            }
            other => panic!("expected lowered struct match if in while body, found {other:?}"),
        },
        other => panic!("expected while statement after binding, found {other:?}"),
    }
}

#[test]
fn lowers_nested_struct_field_binding_match_arms_inside_while() {
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
                Packet { header: Header { kind: packet_kind, ready: true }, code: 5 } => {
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
            other => {
                panic!("expected lowered nested struct match if in while body, found {other:?}")
            }
        },
        other => panic!("expected while statement after binding, found {other:?}"),
    }
}
