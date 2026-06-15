use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{
    AstDestructureBinding, AstDestructureField, AstExpr, AstStmt, NirExpr, NirStmt,
};

#[test]
fn reports_call_inferred_generic_destructure_payload_missing_method_bound() {
    let error = parse_nuis_module(
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

          struct Boxed<T> {
            value: T,
          }

          fn wrap<T>(value: T) -> Boxed<T> {
            return Boxed<T> { value: value };
          }

          fn bump<T>(value: T) -> T {
            let { value: payload } = wrap(value);
            return payload.add(value);
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "function `bump` body calls method `add` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn parses_shorthand_generic_struct_destructuring_let_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed = Boxed<i64> { value: 7 };
            let { value } = boxed;
            return value;
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
            assert!(type_ref.is_none());
            assert_eq!(fields, &vec![bind_field("value", "value")]);
            assert!(matches!(value, AstExpr::Var(name) if name == "boxed"));
        }
        other => panic!("expected shorthand destructuring let statement, found {other:?}"),
    }
}

#[test]
fn parses_nested_generic_struct_destructuring_let_without_repeated_type_head() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          struct Outer<T> {
            inner: Boxed<T>,
            code: T,
          }

          fn main() -> i64 {
            let value: Outer<i64> = Outer<i64> {
              inner: Boxed<i64> { value: 7 },
              code: 1,
            };
            let Outer<i64> { inner: { value: payload }, code: status } = value;
            return payload + status;
          }
        }
        "#,
    )
    .unwrap();

    match &ast.functions[0].body[1] {
        AstStmt::DestructureLet {
            type_ref, fields, ..
        } => {
            assert_eq!(type_ref.as_ref().unwrap().name, "Outer");
            assert_eq!(
                fields,
                &vec![
                    nested_field("inner", None, vec![bind_field("value", "payload")]),
                    bind_field("code", "status"),
                ]
            );
        }
        other => panic!("expected generic nested destructuring let statement, found {other:?}"),
    }
}

#[test]
fn lowers_nested_generic_struct_destructuring_let_without_repeated_type_head() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          struct Outer<T> {
            inner: Boxed<T>,
            code: T,
          }

          fn main() -> i64 {
            let value: Outer<i64> = Outer<i64> {
              inner: Boxed<i64> { value: 7 },
              code: 1,
            };
            let Outer<i64> { inner: { value: payload }, code: status } = value;
            return payload + status;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "payload");
            assert!(matches!(ty, Some(ty) if ty.render() == "i64"));
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, base }
                    if field == "value"
                        && matches!(
                            base.as_ref(),
                            NirExpr::FieldAccess { field, .. } if field == "inner"
                        )
            ));
        }
        other => panic!("expected first generic nested destructured binding, found {other:?}"),
    }
    match &module.functions[0].body[2] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "status");
            assert!(matches!(ty, Some(ty) if ty.render() == "i64"));
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "code"
            ));
        }
        other => panic!("expected second generic nested destructured binding, found {other:?}"),
    }
}

#[test]
fn lowers_inferred_alias_generic_struct_shorthand_destructuring_let() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type BoxAlias<T> = Boxed<T>;

          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let value = BoxAlias { value: 7 };
            let { value: payload } = value;
            return payload;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "payload");
            assert!(matches!(ty, Some(ty) if ty.render() == "i64"));
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "value"
            ));
        }
        other => panic!("expected inferred alias shorthand destructured binding, found {other:?}"),
    }
}

#[test]
fn lowers_nested_generic_alias_struct_destructuring_let() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type BoxAlias<T> = Boxed<T>;
          type OuterAlias<T> = Outer<T>;

          struct Boxed<T> {
            value: T,
          }

          struct Outer<T> {
            inner: BoxAlias<T>,
            code: T,
          }

          fn main() -> i64 {
            let value: OuterAlias<i64> = Outer<i64> {
              inner: Boxed<i64> { value: 7 },
              code: 1,
            };
            let OuterAlias<i64> { inner: { value: payload }, code: status } = value;
            return payload + status;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "payload");
            assert!(matches!(ty, Some(ty) if ty.render() == "i64"));
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, base }
                    if field == "value"
                        && matches!(
                            base.as_ref(),
                            NirExpr::FieldAccess { field, .. } if field == "inner"
                        )
            ));
        }
        other => {
            panic!("expected first nested generic alias destructured binding, found {other:?}")
        }
    }
    match &module.functions[0].body[2] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "status");
            assert!(matches!(ty, Some(ty) if ty.render() == "i64"));
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "code"
            ));
        }
        other => {
            panic!("expected second nested generic alias destructured binding, found {other:?}")
        }
    }
}

#[test]
fn lowers_shorthand_generic_struct_destructuring_let_into_field_bindings() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed = Boxed<i64> { value: 7 };
            let { value } = boxed;
            return value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "value");
            assert!(matches!(ty, Some(ty) if ty.render() == "i64"));
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "value"
            ));
        }
        other => panic!("expected shorthand destructured field binding, found {other:?}"),
    }
}

#[test]
fn lowers_destructure_after_outer_literal_with_deferred_inner_inference() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          fn main() -> i64 {
            let value = Outer {
              inner: Phantom { value: 7, tag: 1 },
              meta: "ok",
            };
            let { inner: { value: payload }, meta } = value;
            return payload;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "payload");
            assert!(matches!(ty, Some(ty) if ty.render() == "i64"));
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, base }
                    if field == "value"
                        && matches!(
                            base.as_ref(),
                            NirExpr::FieldAccess { field, .. } if field == "inner"
                        )
            ));
        }
        other => panic!("expected deferred-inference payload destructured binding, found {other:?}"),
    }
    match &module.functions[0].body[2] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "meta");
            assert!(matches!(ty, Some(ty) if ty.render() == "String"));
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "meta"
            ));
        }
        other => panic!("expected deferred-inference meta destructured binding, found {other:?}"),
    }
}

fn bind_field(field: &str, binding: &str) -> AstDestructureField {
    AstDestructureField {
        field: field.to_owned(),
        binding: AstDestructureBinding::Bind(binding.to_owned()),
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
