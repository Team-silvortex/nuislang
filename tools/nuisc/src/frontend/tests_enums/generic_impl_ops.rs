use super::*;

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
