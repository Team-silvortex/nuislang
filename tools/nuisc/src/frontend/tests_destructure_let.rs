use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{
    AstDestructureBinding, AstDestructureField, AstExpr, AstStmt, NirExpr, NirStmt,
};

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
                &vec![bind_field("kind", "kind"), bind_field("ready", "ready"),]
            );
            assert!(matches!(value, AstExpr::Var(name) if name == "packet"));
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
                    bind_field("kind", "packet_kind"),
                    bind_field("ready", "is_ready"),
                ]
            );
            assert!(matches!(value, AstExpr::Var(name) if name == "packet"));
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
                &vec![bind_field("kind", "kind"), ignore_field("ready")]
            );
        }
        other => panic!("expected destructuring let statement, found {other:?}"),
    }
}

#[test]
fn parses_nested_struct_destructuring_let_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          struct Inner {
            kind: i64,
            ready: bool,
          }

          struct Outer {
            inner: Inner,
            code: i64,
          }

          fn main() -> i64 {
            let value: Outer = Outer {
              inner: Inner { kind: 7, ready: true },
              code: 1,
            };
            let Outer { inner: Inner { kind: packet_kind, ready: _ }, code } = value;
            return packet_kind + code;
          }
        }
        "#,
    )
    .unwrap();

    match &ast.functions[0].body[1] {
        AstStmt::DestructureLet {
            type_ref, fields, ..
        } => {
            assert_eq!(type_ref.name, "Outer");
            assert_eq!(
                fields,
                &vec![
                    nested_field(
                        "inner",
                        Some("Inner"),
                        vec![bind_field("kind", "packet_kind"), ignore_field("ready")],
                    ),
                    bind_field("code", "code"),
                ]
            );
        }
        other => panic!("expected destructuring let statement, found {other:?}"),
    }
}

#[test]
fn parses_nested_struct_destructuring_let_without_repeated_type_head() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          struct Inner {
            kind: i64,
            ready: bool,
          }

          struct Outer {
            inner: Inner,
            code: i64,
          }

          fn main() -> i64 {
            let value: Outer = Outer {
              inner: Inner { kind: 7, ready: true },
              code: 1,
            };
            let Outer { inner: { kind: packet_kind, ready: _ }, code: status } = value;
            return packet_kind + status;
          }
        }
        "#,
    )
    .unwrap();

    match &ast.functions[0].body[1] {
        AstStmt::DestructureLet {
            type_ref, fields, ..
        } => {
            assert_eq!(type_ref.name, "Outer");
            assert_eq!(
                fields,
                &vec![
                    nested_field(
                        "inner",
                        None,
                        vec![bind_field("kind", "packet_kind"), ignore_field("ready")],
                    ),
                    bind_field("code", "status"),
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

#[test]
fn lowers_nested_struct_destructuring_let_without_repeated_type_head() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Inner {
            kind: i64,
            ready: bool,
          }

          struct Outer {
            inner: Inner,
            code: i64,
          }

          fn main() -> i64 {
            let value: Outer = Outer {
              inner: Inner { kind: 7, ready: true },
              code: 2,
            };
            let Outer { inner: { kind: packet_kind, ready: _ }, code: status } = value;
            return packet_kind + status;
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        &module.functions[0].body[1],
        NirStmt::Let { name, value, .. }
            if name == "packet_kind"
                && matches!(
                    value,
                    NirExpr::FieldAccess {
                        field,
                        base,
                    } if field == "kind"
                        && matches!(
                            &**base,
                            NirExpr::FieldAccess { field, .. } if field == "inner"
                        )
                )
    ));
    assert!(matches!(
        &module.functions[0].body[2],
        NirStmt::Let { name, value, .. }
            if name == "status"
                && matches!(value, NirExpr::FieldAccess { field, .. } if field == "code")
    ));
}

#[test]
fn lowers_nested_struct_destructuring_let_into_nested_field_bindings() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Inner {
            kind: i64,
            ready: bool,
          }

          struct Outer {
            inner: Inner,
            code: i64,
          }

          fn main() -> i64 {
            let value: Outer = Outer {
              inner: Inner { kind: 7, ready: true },
              code: 2,
            };
            let Outer { inner: Inner { kind: packet_kind, ready: _ }, code } = value;
            return packet_kind + code;
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        &module.functions[0].body[1],
        NirStmt::Let { name, value, .. }
            if name == "packet_kind"
                && matches!(
                    value,
                    NirExpr::FieldAccess {
                        field,
                        base,
                    } if field == "kind"
                        && matches!(
                            &**base,
                            NirExpr::FieldAccess { field, .. } if field == "inner"
                        )
                )
    ));
    assert!(matches!(
        &module.functions[0].body[2],
        NirStmt::Let { name, value, .. }
            if name == "code"
                && matches!(value, NirExpr::FieldAccess { field, .. } if field == "code")
    ));
}

fn bind_field(field: &str, binding: &str) -> AstDestructureField {
    AstDestructureField {
        field: field.to_owned(),
        binding: AstDestructureBinding::Bind(binding.to_owned()),
    }
}

fn ignore_field(field: &str) -> AstDestructureField {
    AstDestructureField {
        field: field.to_owned(),
        binding: AstDestructureBinding::Ignore,
    }
}

fn nested_field(
    field: &str,
    type_name: Option<&str>,
    fields: Vec<AstDestructureField>,
) -> AstDestructureField {
    AstDestructureField {
        field: field.to_owned(),
        binding: AstDestructureBinding::Nested {
            type_ref: type_name.map(|type_name| nuis_semantics::model::AstTypeRef {
                name: type_name.to_owned(),
                generic_args: Vec::new(),
                is_optional: false,
                is_ref: false,
            }),
            fields,
        },
    }
}
