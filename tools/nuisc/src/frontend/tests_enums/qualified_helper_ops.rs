use super::*;

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
