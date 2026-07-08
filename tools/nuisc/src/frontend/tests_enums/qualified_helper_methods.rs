use super::*;

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
