use super::*;

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
fn lowers_generic_impl_default_method_on_enum_variants() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;

            fn twice(value: Self) -> Self {
              return Addable.add(value, value);
            }
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
            let sum: Option<i64> = Addable.twice(Option.Some(2));
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
        other => panic!("expected specialized generic impl default-method call, found {other:?}"),
    };
    assert!(specialized_name.starts_with("impl.Addable.for.Option_T_.twice__"));
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == specialized_name)
        .unwrap();
    assert!(matches!(
        specialized.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee.starts_with("impl.Addable.for.Option_T_.add__")
    ));
}

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_on_enum_variants() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;

            fn twice(value: Self) -> Self {
              return Addable.add(value, value);
            }
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
            let sum: Option<i64> = Option.Some(2).twice<i64>();
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
        other => {
            panic!("expected specialized receiver explicit-generic method call, found {other:?}")
        }
    };
    assert!(specialized_name.starts_with("impl.Addable.for.Option_T_.twice__"));
}

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_anchoring_none_variant() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;

            fn twice(value: Self) -> Self {
              return Addable.add(value, value);
            }
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
            let sum: Option<i64> = Option.None.twice<i64>();
            match sum {
              Option.None => {
                return 0;
              }
              _ => {
                return 1;
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
        other => panic!(
            "expected specialized explicit-generic none-variant method call, found {other:?}"
        ),
    };
    assert!(specialized_name.starts_with("impl.Addable.for.Option_T_.twice__"));
}
