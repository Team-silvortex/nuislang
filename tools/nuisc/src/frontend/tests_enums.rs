use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{AstExpr, AstStmt, NirExpr, NirStmt};

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
            ] if name == "payload"
                && result == "payload"
                && matches!(value, NirExpr::FieldAccess { field, .. } if field == "value")
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
                ] if name == "payload"
                    && result == "payload"
                    && matches!(value, NirExpr::FieldAccess { field, .. } if field == "value")
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
fn lowers_generic_impl_method_calls_on_enum_variants() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Showable {
            fn show(value: Self) -> i64;
          }

          impl Showable for i64 {
            fn show(value: i64) -> i64 {
              return value;
            }
          }

          enum Option<T> {
            None,
            Some(T),
          }

          impl<T: Showable> Showable for Option<T> where T: Showable {
            fn show(value: Option<T>) -> i64 {
              match value {
                Option.Some(payload) => {
                  return Showable.show(payload);
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
            return Option.Some(7).show();
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
    let specialized_name = match main.body.first() {
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. }))) => callee.clone(),
        other => panic!("expected specialized generic impl call, found {other:?}"),
    };
    assert!(specialized_name.starts_with("impl.Showable.for.Option_T_.show__"));
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == specialized_name)
        .unwrap();
    assert!(matches!(specialized.body.first(), Some(NirStmt::If { .. })));
}

#[test]
fn lowers_generic_impl_binary_add_on_enum_variants() {
    let module = parse_nuis_module(
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

          enum Option<T> {
            None,
            Some(T),
          }

          impl<T: Addable> Addable for Option<T> where T: Addable {
            fn add(lhs: Option<T>, rhs: Option<T>) -> Option<T> {
              match lhs {
                Option.Some(left) => {
                  match rhs {
                    Option.Some(right) => {
                      return Option.Some(Addable.add(left, right));
                    }
                    _ => {
                      return Option.None;
                    }
                  }
                }
                _ => {
                  return Option.None;
                }
              }
            }
          }

          fn main() -> i64 {
            let sum: Option<i64> = Option.Some(1) + Option.Some(2);
            match sum {
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

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    let specialized_name = match main.body.first() {
        Some(NirStmt::Let {
            value: NirExpr::Call { callee, .. },
            ..
        }) => callee.clone(),
        other => panic!("expected specialized generic impl add call, found {other:?}"),
    };
    assert!(specialized_name.starts_with("impl.Addable.for.Option_T_.add__"));
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == specialized_name)
        .unwrap();
    assert!(matches!(specialized.body.first(), Some(NirStmt::If { .. })));
}

#[test]
fn lowers_generic_impl_unary_and_comparison_ops_on_enum_variants() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Notable {
            fn not(value: Self) -> bool;
          }

          trait Equatable {
            fn eq(lhs: Self, rhs: Self) -> bool;
          }

          trait Orderable {
            fn lt(lhs: Self, rhs: Self) -> bool;
            fn le(lhs: Self, rhs: Self) -> bool;
            fn gt(lhs: Self, rhs: Self) -> bool;
            fn ge(lhs: Self, rhs: Self) -> bool;
          }

          impl Notable for i64 {
            fn not(value: i64) -> bool {
              return value == 0;
            }
          }

          impl Equatable for i64 {
            fn eq(lhs: i64, rhs: i64) -> bool {
              return lhs == rhs;
            }
          }

          impl Orderable for i64 {
            fn lt(lhs: i64, rhs: i64) -> bool {
              return lhs < rhs;
            }
            fn le(lhs: i64, rhs: i64) -> bool {
              return lhs <= rhs;
            }
            fn gt(lhs: i64, rhs: i64) -> bool {
              return lhs > rhs;
            }
            fn ge(lhs: i64, rhs: i64) -> bool {
              return lhs >= rhs;
            }
          }

          enum Option<T> {
            None,
            Some(T),
          }

          impl<T: Notable + Equatable + Orderable> Notable for Option<T> where T: Notable + Equatable + Orderable {
            fn not(value: Option<T>) -> bool {
              match value {
                Option.Some(payload) => {
                  return !payload;
                }
                _ => {
                  return true;
                }
              }
            }
          }

          impl<T: Equatable + Orderable> Equatable for Option<T> where T: Equatable + Orderable {
            fn eq(lhs: Option<T>, rhs: Option<T>) -> bool {
              match lhs {
                Option.Some(left) => {
                  match rhs {
                    Option.Some(right) => {
                      return left == right;
                    }
                    _ => {
                      return false;
                    }
                  }
                }
                Option.None => {
                  match rhs {
                    Option.None => {
                      return true;
                    }
                    _ => {
                      return false;
                    }
                  }
                }
                _ => {
                  return false;
                }
              }
            }
          }

          impl<T: Equatable + Orderable> Orderable for Option<T> where T: Equatable + Orderable {
            fn lt(lhs: Option<T>, rhs: Option<T>) -> bool {
              match lhs {
                Option.Some(left) => {
                  match rhs {
                    Option.Some(right) => {
                      return left < right;
                    }
                    _ => {
                      return false;
                    }
                  }
                }
                _ => {
                  return false;
                }
              }
            }
            fn le(lhs: Option<T>, rhs: Option<T>) -> bool {
              return lhs < rhs || lhs == rhs;
            }
            fn gt(lhs: Option<T>, rhs: Option<T>) -> bool {
              return !(lhs <= rhs);
            }
            fn ge(lhs: Option<T>, rhs: Option<T>) -> bool {
              return !(lhs < rhs);
            }
          }

          fn main() -> i64 {
            let empty: bool = !Option.Some(0);
            let same: bool = Option.Some(2) == Option.Some(2);
            let less: bool = Option.Some(1) < Option.Some(3);
            if empty && same && less {
              return 1;
            }
            return 0;
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
        }) if name == "empty" && callee.starts_with("impl.Notable.for.Option_T_.not__")
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "same" && callee.starts_with("impl.Equatable.for.Option_T_.eq__")
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "less" && callee.starts_with("impl.Orderable.for.Option_T_.lt__")
    ));
}

#[test]
fn lowers_generic_impl_with_fully_qualified_helper_trait_for_method_calls() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
          }

          impl Helper.Showable for i64 {
            fn show(value: i64) -> i64 {
              return value;
            }
          }

          impl<T: Helper.Showable> Helper.Showable for Option<T> where T: Helper.Showable {
            fn show(value: Option<T>) -> i64 {
              match value {
                Option.Some(payload) => {
                  return Helper.Showable.show(payload);
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
            let explicit = Helper.Showable.show(Option.Some(8));
            return direct + explicit;
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Showable {
            fn show(value: Self) -> i64;
          }
        }
        "#,
    )
    .unwrap();

    let module = super::lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
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
        }) if name == "direct" && callee.starts_with("impl.Helper.Showable.for.Option_T_.show__")
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "explicit" && callee.starts_with("impl.Helper.Showable.for.Option_T_.show__")
    ));
}

#[test]
fn lowers_generic_impl_with_fully_qualified_helper_trait_for_operator_calls() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
          }

          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl<T: Helper.Addable> Helper.Addable for Option<T> where T: Helper.Addable {
            fn add(lhs: Option<T>, rhs: Option<T>) -> Option<T> {
              match lhs {
                Option.Some(left) => {
                  match rhs {
                    Option.Some(right) => {
                      return Option.Some(Helper.Addable.add(left, right));
                    }
                    _ => {
                      return Option.None;
                    }
                  }
                }
                _ => {
                  return Option.None;
                }
              }
            }
          }

          fn main() -> i64 {
            let sum: Option<i64> = Option.Some(1) + Option.Some(2);
            match sum {
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
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = super::lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(NirStmt::Let {
            value: NirExpr::Call { callee, .. },
            ..
        }) if callee.starts_with("impl.Helper.Addable.for.Option_T_.add__")
    ));
}

#[test]
fn lowers_generic_impl_with_fully_qualified_helper_trait_for_comparison_and_unary_ops() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
          }

          impl Helper.Notable for i64 {
            fn not(value: i64) -> bool {
              return value == 0;
            }
          }

          impl<T: Helper.Notable> Helper.Notable for Option<T> where T: Helper.Notable {
            fn not(value: Option<T>) -> bool {
              match value {
                Option.Some(payload) => {
                  return Helper.Notable.not(payload);
                }
                Option.None => {
                  return true;
                }
                _ => {
                  return false;
                }
              }
            }
          }

          impl Helper.Equatable for i64 {
            fn eq(lhs: i64, rhs: i64) -> bool {
              return lhs == rhs;
            }
          }

          impl Helper.Orderable for i64 {
            fn lt(lhs: i64, rhs: i64) -> bool {
              return lhs < rhs;
            }
            fn le(lhs: i64, rhs: i64) -> bool {
              return lhs < rhs || lhs == rhs;
            }
            fn gt(lhs: i64, rhs: i64) -> bool {
              return !(lhs <= rhs);
            }
            fn ge(lhs: i64, rhs: i64) -> bool {
              return !(lhs < rhs);
            }
          }

          impl<T: Helper.Equatable + Helper.Orderable> Helper.Equatable for Option<T>
          where T: Helper.Equatable + Helper.Orderable {
            fn eq(lhs: Option<T>, rhs: Option<T>) -> bool {
              match lhs {
                Option.Some(left) => {
                  match rhs {
                    Option.Some(right) => {
                      return Helper.Equatable.eq(left, right);
                    }
                    _ => {
                      return false;
                    }
                  }
                }
                Option.None => {
                  match rhs {
                    Option.None => {
                      return true;
                    }
                    _ => {
                      return false;
                    }
                  }
                }
                _ => {
                  return false;
                }
              }
            }
          }

          impl<T: Helper.Equatable + Helper.Orderable> Helper.Orderable for Option<T>
          where T: Helper.Equatable + Helper.Orderable {
            fn lt(lhs: Option<T>, rhs: Option<T>) -> bool {
              match lhs {
                Option.Some(left) => {
                  match rhs {
                    Option.Some(right) => {
                      return Helper.Orderable.lt(left, right);
                    }
                    _ => {
                      return false;
                    }
                  }
                }
                _ => {
                  return false;
                }
              }
            }
            fn le(lhs: Option<T>, rhs: Option<T>) -> bool {
              return lhs < rhs || lhs == rhs;
            }
            fn gt(lhs: Option<T>, rhs: Option<T>) -> bool {
              return !(lhs <= rhs);
            }
            fn ge(lhs: Option<T>, rhs: Option<T>) -> bool {
              return !(lhs < rhs);
            }
          }

          fn main() -> i64 {
            let empty: bool = !Option.Some(0);
            let same: bool = Option.Some(2) == Option.Some(2);
            let less: bool = Option.Some(1) < Option.Some(3);
            if empty && same && less {
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Notable {
            fn not(value: Self) -> bool;
          }

          pub trait Equatable {
            fn eq(lhs: Self, rhs: Self) -> bool;
          }

          pub trait Orderable {
            fn lt(lhs: Self, rhs: Self) -> bool;
            fn le(lhs: Self, rhs: Self) -> bool;
            fn gt(lhs: Self, rhs: Self) -> bool;
            fn ge(lhs: Self, rhs: Self) -> bool;
          }
        }
        "#,
    )
    .unwrap();

    let module = super::lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
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
        }) if name == "empty" && callee.starts_with("impl.Helper.Notable.for.Option_T_.not__")
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "same" && callee.starts_with("impl.Helper.Equatable.for.Option_T_.eq__")
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "less" && callee.starts_with("impl.Helper.Orderable.for.Option_T_.lt__")
    ));
}

#[test]
fn lowers_generic_impl_with_fully_qualified_helper_trait_for_subtraction() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
          }

          impl Helper.Subtractable for i64 {
            fn sub(lhs: i64, rhs: i64) -> i64 {
              return lhs - rhs;
            }
          }

          impl<T: Helper.Subtractable> Helper.Subtractable for Option<T> where T: Helper.Subtractable {
            fn sub(lhs: Option<T>, rhs: Option<T>) -> Option<T> {
              match lhs {
                Option.Some(left) => {
                  match rhs {
                    Option.Some(right) => {
                      return Option.Some(Helper.Subtractable.sub(left, right));
                    }
                    _ => {
                      return Option.None;
                    }
                  }
                }
                _ => {
                  return Option.None;
                }
              }
            }
          }

          fn main() -> i64 {
            let diff: Option<i64> = Option.Some(5) - Option.Some(3);
            match diff {
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
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Subtractable {
            fn sub(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = super::lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(NirStmt::Let {
            value: NirExpr::Call { callee, .. },
            ..
        }) if callee.starts_with("impl.Helper.Subtractable.for.Option_T_.sub__")
    ));
}

#[test]
fn lowers_generic_impl_with_fully_qualified_helper_trait_for_mul_div_rem() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
          }

          impl Helper.Multipliable for i64 {
            fn mul(lhs: i64, rhs: i64) -> i64 {
              return lhs * rhs;
            }
          }

          impl Helper.Dividable for i64 {
            fn div(lhs: i64, rhs: i64) -> i64 {
              return lhs / rhs;
            }
          }

          impl Helper.Remainderable for i64 {
            fn rem(lhs: i64, rhs: i64) -> i64 {
              return lhs % rhs;
            }
          }

          impl<T: Helper.Multipliable> Helper.Multipliable for Option<T> where T: Helper.Multipliable {
            fn mul(lhs: Option<T>, rhs: Option<T>) -> Option<T> {
              match lhs {
                Option.Some(left) => {
                  match rhs {
                    Option.Some(right) => {
                      return Option.Some(Helper.Multipliable.mul(left, right));
                    }
                    _ => {
                      return Option.None;
                    }
                  }
                }
                _ => {
                  return Option.None;
                }
              }
            }
          }

          impl<T: Helper.Dividable> Helper.Dividable for Option<T> where T: Helper.Dividable {
            fn div(lhs: Option<T>, rhs: Option<T>) -> Option<T> {
              match lhs {
                Option.Some(left) => {
                  match rhs {
                    Option.Some(right) => {
                      return Option.Some(Helper.Dividable.div(left, right));
                    }
                    _ => {
                      return Option.None;
                    }
                  }
                }
                _ => {
                  return Option.None;
                }
              }
            }
          }

          impl<T: Helper.Remainderable> Helper.Remainderable for Option<T> where T: Helper.Remainderable {
            fn rem(lhs: Option<T>, rhs: Option<T>) -> Option<T> {
              match lhs {
                Option.Some(left) => {
                  match rhs {
                    Option.Some(right) => {
                      return Option.Some(Helper.Remainderable.rem(left, right));
                    }
                    _ => {
                      return Option.None;
                    }
                  }
                }
                _ => {
                  return Option.None;
                }
              }
            }
          }

          fn main() -> i64 {
            let prod: Option<i64> = Option.Some(6) * Option.Some(7);
            let quot: Option<i64> = Option.Some(8) / Option.Some(2);
            let rest: Option<i64> = Option.Some(9) % Option.Some(4);
            match prod {
              Option.Some(prod_value) => {
                match quot {
                  Option.Some(quot_value) => {
                    match rest {
                      Option.Some(rest_value) => {
                        return prod_value + quot_value + rest_value;
                      }
                      _ => {
                        return 0;
                      }
                    }
                  }
                  _ => {
                    return 0;
                  }
                }
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
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Multipliable {
            fn mul(lhs: Self, rhs: Self) -> Self;
          }

          pub trait Dividable {
            fn div(lhs: Self, rhs: Self) -> Self;
          }

          pub trait Remainderable {
            fn rem(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = super::lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
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
        }) if name == "prod" && callee.starts_with("impl.Helper.Multipliable.for.Option_T_.mul__")
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "quot" && callee.starts_with("impl.Helper.Dividable.for.Option_T_.div__")
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "rest" && callee.starts_with("impl.Helper.Remainderable.for.Option_T_.rem__")
    ));
}
