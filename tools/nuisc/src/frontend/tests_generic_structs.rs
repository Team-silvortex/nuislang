use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{NirExpr, NirStmt};

#[test]
fn parses_generic_struct_definition_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }
        }
        "#,
    )
    .unwrap();

    let definition = &ast.structs[0];
    assert_eq!(definition.name, "Boxed");
    assert_eq!(definition.generic_params.len(), 1);
    assert_eq!(definition.generic_params[0].name, "T");
    assert!(definition.generic_params[0].bound.is_none());
    assert_eq!(definition.fields[0].ty.name, "T");
}

#[test]
fn lowers_generic_struct_literal_with_expected_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed: Boxed<i64> = Boxed { value: 7 };
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
        other => panic!("expected generic struct let, found {other:?}"),
    }
}

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
