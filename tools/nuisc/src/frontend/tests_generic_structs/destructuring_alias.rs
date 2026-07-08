use super::*;

#[test]
fn lowers_generic_struct_destructuring_let() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed: Boxed<i64> = Boxed { value: 7 };
            let Boxed<i64> { value } = boxed;
            return value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "value");
            assert_eq!(ty.as_ref().unwrap().render(), "i64");
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "value"
            ));
        }
        other => panic!("expected lowered destructured field let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_struct_destructuring_let_with_alias_type_head() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type BoxI64 = Boxed<i64>;

          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed: BoxI64 = Boxed<i64> { value: 7 };
            let BoxI64 { value } = boxed;
            return value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "value");
            assert_eq!(ty.as_ref().unwrap().render(), "i64");
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "value"
            ));
        }
        other => panic!("expected lowered aliased generic destructured field let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_struct_destructuring_let_with_generic_alias_type_head() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type BoxAlias<T> = Boxed<T>;

          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed: BoxAlias<i64> = Boxed<i64> { value: 7 };
            let BoxAlias<i64> { value } = boxed;
            return value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "value");
            assert_eq!(ty.as_ref().unwrap().render(), "i64");
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "value"
            ));
        }
        other => panic!("expected lowered generic-aliased destructured field let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_struct_literal_with_inferred_generic_alias_type_head() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type BoxAlias<T> = Boxed<T>;

          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed = BoxAlias { value: 7 };
            return boxed.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "boxed");
            assert_eq!(ty.as_ref().unwrap().render(), "Boxed<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Boxed"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected inferred generic-alias struct let, found {other:?}"),
    }
}
