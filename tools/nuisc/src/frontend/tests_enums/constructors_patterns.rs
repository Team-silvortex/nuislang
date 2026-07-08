use super::*;

#[test]
fn parses_qualified_enum_variant_constructors_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
            Named {
              value: T,
            },
          }

          fn main() {
            let none = Option.None;
            let some = Option.Some(7);
            let named = Option.Named { value: 9 };
          }
        }
        "#,
    )
    .unwrap();

    let body = &ast.functions[0].body;
    assert!(matches!(
        &body[0],
        AstStmt::Let {
            value: AstExpr::FieldAccess { field, .. },
            ..
        } if field == "None"
    ));
    assert!(matches!(
        &body[1],
        AstStmt::Let {
            value: AstExpr::Call { callee, args, .. },
            ..
        } if callee == "Option.Some" && args.len() == 1
    ));
    assert!(matches!(
        &body[2],
        AstStmt::Let {
            value: AstExpr::StructLiteral { type_name, fields, .. },
            ..
        } if type_name == "Option.Named" && fields.len() == 1
    ));
}

#[test]
fn lowers_qualified_enum_variant_constructors_via_synthesized_variant_structs() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
            Named {
              value: T,
            },
          }

          fn main() {
            let none = Option.None;
            let some = Option.Some(7);
            let named = Option.Named { value: 9 };
          }
        }
        "#,
    )
    .unwrap();

    let body = &module.functions[0].body;
    assert!(matches!(
        &body[0],
        NirStmt::Let {
            value: NirExpr::StructLiteral { type_name, fields, .. },
            ..
        } if type_name == "Option.None" && fields.is_empty()
    ));
    assert!(matches!(
        &body[1],
        NirStmt::Let {
            value: NirExpr::StructLiteral { type_name, fields, .. },
            ..
        } if type_name == "Option.Some" && fields.len() == 1
    ));
    assert!(matches!(
        &body[2],
        NirStmt::Let {
            value: NirExpr::StructLiteral { type_name, fields, .. },
            ..
        } if type_name == "Option.Named" && fields.len() == 1
    ));
}

#[test]
fn lowers_unit_and_payload_variant_match_patterns() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
          }

          fn none_case() -> i64 {
            let value = Option.None;
            match value {
              Option.None => {
                return 1;
              }
              _ => {
                return 0;
              }
            }
          }

          fn some_case() -> i64 {
            let value = Option.Some(7);
            match value {
              Option.Some(payload) => {
                return payload;
              }
              _ => {
                return 0;
              }
            }
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        &module.functions[0].body[1],
        NirStmt::If {
            condition: NirExpr::Bool(true),
            ..
        }
    ));
    assert!(matches!(
        &module.functions[1].body[1],
        NirStmt::If {
            then_body,
            ..
        } if matches!(
            then_body.as_slice(),
            [
                NirStmt::Let { name, value, .. },
                NirStmt::Return(Some(NirExpr::Var(result)))
                ] if name == "payload" && result == "payload" && is_payload_value_access(value)
        )
    ));
}

#[test]
fn parent_enum_type_accepts_variant_constructors() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
            Named {
              value: T,
            },
          }

          fn main() {
            let none: Option<i64> = Option.None;
            let some: Option<i64> = Option.Some(7);
            let named: Option<i64> = Option.Named { value: 9 };
          }
        }
        "#,
    )
    .unwrap();

    let body = &module.functions[0].body;
    assert!(matches!(
        &body[0],
        NirStmt::Let { ty: Some(ty), value: NirExpr::StructLiteral { type_name, type_args, .. }, .. }
            if ty.render() == "Option<i64>"
                && type_name == "Option.None"
                && type_args.len() == 1
                && type_args[0].render() == "i64"
    ));
    assert!(matches!(
        &body[1],
        NirStmt::Let { ty: Some(ty), value: NirExpr::StructLiteral { type_name, type_args, .. }, .. }
            if ty.render() == "Option<i64>"
                && type_name == "Option.Some"
                && type_args.len() == 1
                && type_args[0].render() == "i64"
    ));
    assert!(matches!(
        &body[2],
        NirStmt::Let { ty: Some(ty), value: NirExpr::StructLiteral { type_name, type_args, .. }, .. }
            if ty.render() == "Option<i64>"
                && type_name == "Option.Named"
                && type_args.len() == 1
                && type_args[0].render() == "i64"
    ));
}

#[test]
fn match_on_parent_enum_typed_value_accepts_variant_patterns() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
          }

          fn some_case() -> i64 {
            let value: Option<i64> = Option.Some(7);
            match value {
              Option.Some(payload) => {
                return payload;
              }
              Option.None => {
                return 0;
              }
              _ => {
                return -1;
              }
            }
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        &module.functions[0].body[1],
        NirStmt::If { then_body, else_body, .. }
            if matches!(
                then_body.as_slice(),
                [
                    NirStmt::Let { name, value, .. },
                    NirStmt::Return(Some(NirExpr::Var(result)))
                ] if name == "payload" && result == "payload" && is_payload_value_access(value)
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::If { condition: NirExpr::Bool(true), .. }]
            )
    ));
}

#[test]
fn lowers_trait_method_calls_on_variant_values_via_parent_enum_impls() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
          }

          trait Showable {
            fn show(value: Self) -> i64;
          }

          impl Showable for Option<i64> {
            fn show(value: Option<i64>) -> i64 {
              match value {
                Option.Some(payload) => {
                  return payload;
                }
                Option.None => {
                  return 0;
                }
                _ => {
                  return -1;
                }
              }
            }
          }

          fn main() -> i64 {
            let direct = Option.Some(7).show();
            let explicit = Showable.show(Option.Some(8));
            return direct + explicit;
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "direct" && callee == "impl.Showable.for.Option_i64_.show"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "explicit" && callee == "impl.Showable.for.Option_i64_.show"
    ));
}

#[test]
fn lowers_result_error_patterns_into_nested_variant_checks() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum CoreError {
            InvalidInput,
            MissingValue,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn handle(result: Result<i64, CoreError>) -> i64 {
            match result {
              Result.Ok(value) => {
                return value;
              }
              Result.Err(CoreError.MissingValue) => {
                return 0;
              }
              Result.Err(CoreError.InvalidInput) => {
                return -1;
              }
              _ => {
                return -2;
              }
            }
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        &module.functions[0].body[0],
        NirStmt::If { then_body, .. }
            if matches!(
                then_body.as_slice(),
                [
                    NirStmt::Let { name, value, .. },
                    NirStmt::Return(Some(NirExpr::Var(result)))
                ] if name == "value" && result == "value" && is_payload_value_access(value)
            )
    ));
}

#[test]
fn result_err_constructor_accepts_enum_error_payload_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum CoreError {
            InvalidInput,
            MissingValue,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn main() {
            let failure: Result<i64, CoreError> = Result.Err(CoreError.MissingValue);
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        &module.functions[0].body[0],
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, type_args, fields },
            ..
        } if ty.render() == "Result<i64, CoreError>"
            && type_name == "Result.Err"
            && type_args.len() == 2
            && type_args[0].render() == "i64"
            && type_args[1].render() == "CoreError"
            && fields.len() == 1
    ));
}

#[test]
fn lowers_result_match_expression_propagation_pattern() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum CoreError {
            InvalidInput,
            MissingValue,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn choose(seed: i64) -> Result<i64, CoreError> {
            if seed > 0 {
              return Result.Ok(seed);
            }
            return Result.Err(CoreError.InvalidInput);
          }

          fn pipeline(seed: i64) -> Result<i64, CoreError> {
            let chosen: Result<i64, CoreError> = choose(seed);
            let mut value: i64 = 0;
            match chosen {
              Result.Ok(payload) => {
                value = payload;
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
            return Result.Ok(value + 1);
          }
        }
        "#,
    )
    .unwrap();

    let pipeline = module
        .functions
        .iter()
        .find(|function| function.name == "pipeline")
        .unwrap();
    assert!(matches!(
        pipeline.body.as_slice(),
        [
            NirStmt::Let { .. },
            NirStmt::Let { .. },
            NirStmt::If { .. },
            NirStmt::Return(Some(NirExpr::StructLiteral { type_name, .. }))
        ] if type_name == "Result.Ok"
    ));
}
