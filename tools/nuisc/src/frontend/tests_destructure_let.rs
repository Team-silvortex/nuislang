use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{AstDestructureField, AstStmt, NirExpr, NirStmt};

#[test]
fn parses_struct_destructuring_let_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          struct Packet {
            kind: i64,
            ready: bool,
          }

          fn main() -> i64 {
            let packet: Packet = Packet { kind: 7, ready: true };
            let Packet { kind, ready } = packet;
            if ready {
              return kind;
            }
            return 9;
          }
        }
        "#,
    )
    .unwrap();

    match &ast.functions[0].body[1] {
        AstStmt::DestructureLet {
            type_ref,
            fields,
            value,
        } => {
            assert_eq!(type_ref.name, "Packet");
            assert_eq!(
                fields,
                &vec![
                    AstDestructureField {
                        field: "kind".to_owned(),
                        binding: "kind".to_owned()
                    },
                    AstDestructureField {
                        field: "ready".to_owned(),
                        binding: "ready".to_owned()
                    }
                ]
            );
            assert!(matches!(value, nuis_semantics::model::AstExpr::Var(name) if name == "packet"));
        }
        other => panic!("expected destructuring let statement, found {other:?}"),
    }
}

#[test]
fn parses_struct_destructuring_let_with_renamed_bindings() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          struct Packet {
            kind: i64,
            ready: bool,
          }

          fn main() -> i64 {
            let packet: Packet = Packet { kind: 7, ready: true };
            let Packet { kind: packet_kind, ready: is_ready } = packet;
            if is_ready {
              return packet_kind;
            }
            return 9;
          }
        }
        "#,
    )
    .unwrap();

    match &ast.functions[0].body[1] {
        AstStmt::DestructureLet {
            type_ref,
            fields,
            value,
        } => {
            assert_eq!(type_ref.name, "Packet");
            assert_eq!(
                fields,
                &vec![
                    AstDestructureField {
                        field: "kind".to_owned(),
                        binding: "packet_kind".to_owned()
                    },
                    AstDestructureField {
                        field: "ready".to_owned(),
                        binding: "is_ready".to_owned()
                    }
                ]
            );
            assert!(matches!(value, nuis_semantics::model::AstExpr::Var(name) if name == "packet"));
        }
        other => panic!("expected destructuring let statement, found {other:?}"),
    }
}

#[test]
fn parses_struct_destructuring_let_with_ignored_field() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          struct Packet {
            kind: i64,
            ready: bool,
          }

          fn main() -> i64 {
            let packet: Packet = Packet { kind: 7, ready: true };
            let Packet { kind, ready: _ } = packet;
            return kind;
          }
        }
        "#,
    )
    .unwrap();

    match &ast.functions[0].body[1] {
        AstStmt::DestructureLet { fields, .. } => {
            assert_eq!(
                fields,
                &vec![
                    AstDestructureField {
                        field: "kind".to_owned(),
                        binding: "kind".to_owned()
                    },
                    AstDestructureField {
                        field: "ready".to_owned(),
                        binding: "_".to_owned()
                    }
                ]
            );
        }
        other => panic!("expected destructuring let statement, found {other:?}"),
    }
}

#[test]
fn lowers_struct_destructuring_let_into_field_bindings() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Packet {
            kind: i64,
            ready: bool,
          }

          fn main() -> i64 {
            let packet: Packet = Packet { kind: 7, ready: true };
            let Packet { kind, ready } = packet;
            if ready {
              return kind;
            }
            return 9;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::Let { name, value, .. } => {
            assert_eq!(name, "kind");
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "kind"
            ));
        }
        other => panic!("expected first destructured field binding, found {other:?}"),
    }
    match &module.functions[0].body[2] {
        NirStmt::Let { name, value, .. } => {
            assert_eq!(name, "ready");
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "ready"
            ));
        }
        other => panic!("expected second destructured field binding, found {other:?}"),
    }
}

#[test]
fn lowers_struct_destructuring_let_with_renamed_bindings_into_field_bindings() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Packet {
            kind: i64,
            ready: bool,
          }

          fn main() -> i64 {
            let packet: Packet = Packet { kind: 7, ready: true };
            let Packet { kind: packet_kind, ready: is_ready } = packet;
            if is_ready {
              return packet_kind;
            }
            return 9;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::Let { name, value, .. } => {
            assert_eq!(name, "packet_kind");
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "kind"
            ));
        }
        other => panic!("expected first destructured field binding, found {other:?}"),
    }
    match &module.functions[0].body[2] {
        NirStmt::Let { name, value, .. } => {
            assert_eq!(name, "is_ready");
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "ready"
            ));
        }
        other => panic!("expected second destructured field binding, found {other:?}"),
    }
}

#[test]
fn lowers_struct_destructuring_let_with_ignored_field_without_binding() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Packet {
            kind: i64,
            ready: bool,
          }

          fn main() -> i64 {
            let packet: Packet = Packet { kind: 7, ready: true };
            let Packet { kind, ready: _ } = packet;
            return kind;
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        &module.functions[0].body[1],
        NirStmt::Let { name, value, .. }
            if name == "kind"
                && matches!(value, NirExpr::FieldAccess { field, .. } if field == "kind")
    ));
    assert!(matches!(
        &module.functions[0].body[2],
        NirStmt::Return(Some(NirExpr::Var(name))) if name == "kind"
    ));
}
