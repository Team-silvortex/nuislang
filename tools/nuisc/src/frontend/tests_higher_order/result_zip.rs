use super::*;

#[test]
fn lowers_result_zip_with_direct_ok_constructors_and_generic_named_fn2_callable() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum CoreError {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn choose_left<T>(lhs: T, rhs: T) -> T {
            return lhs;
          }

          fn result_zip_with<T, U, R, E>(
            lhs: Result<T, E>,
            rhs: Result<U, E>,
            mapper: Fn2<T, U, R>
          ) -> Result<R, E> {
            match lhs {
              Result.Ok(lhs_value) => {
                match rhs {
                  Result.Ok(rhs_value) => {
                    return Result.Ok(mapper(lhs_value, rhs_value));
                  }
                  Result.Err(error) => {
                    return Result.Err(error);
                  }
                }
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          fn main() -> i64 {
            let mapped: Result<i64, CoreError> =
              result_zip_with(Result.Ok(7), Result.Ok(3), choose_left);
            match mapped {
              Result.Ok(value) => {
                return value;
              }
              Result.Err(_) => {
                return -1;
              }
            }
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| {
            function.name == "__hof_result_zip_with_choose_left__i64__i64__i64__CoreError"
        })
        .expect("expected monomorphized result_zip_with helper for generic Fn2 callable");
    assert!(helper.generic_params.is_empty());

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "choose_left__i64__i64__i64__CoreError")
        .expect("expected generic Fn2 callable specialization");
    assert!(specialized.generic_params.is_empty());
}

#[test]
fn lowers_result_zip3_with_direct_ok_constructors_and_generic_named_fn3_callable() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum CoreError {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn choose_first<T>(lhs: T, mid: T, rhs: T) -> T {
            return lhs;
          }

          fn result_zip3_with<T, U, V, R, E>(
            first: Result<T, E>,
            second: Result<U, E>,
            third: Result<V, E>,
            mapper: Fn3<T, U, V, R>
          ) -> Result<R, E> {
            match first {
              Result.Ok(first_value) => {
                match second {
                  Result.Ok(second_value) => {
                    match third {
                      Result.Ok(third_value) => {
                        return Result.Ok(mapper(first_value, second_value, third_value));
                      }
                      Result.Err(error) => {
                        return Result.Err(error);
                      }
                    }
                  }
                  Result.Err(error) => {
                    return Result.Err(error);
                  }
                }
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          fn main() -> i64 {
            let mapped: Result<i64, CoreError> = result_zip3_with(
              Result.Ok(7),
              Result.Ok(3),
              Result.Ok(1),
              choose_first
            );
            match mapped {
              Result.Ok(value) => {
                return value;
              }
              Result.Err(_) => {
                return -1;
              }
            }
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| {
            function.name == "__hof_result_zip3_with_choose_first__i64__i64__i64__i64__CoreError"
        })
        .expect("expected monomorphized result_zip3_with helper for generic Fn3 callable");
    assert!(helper.generic_params.is_empty());

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "choose_first__i64__i64__i64__i64__CoreError")
        .expect("expected generic Fn3 callable specialization");
    assert!(specialized.generic_params.is_empty());
}

#[test]
fn lowers_result_zip_with_direct_ok_constructors_and_generic_named_fn2_alias_callable() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum CoreError {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          type Mapper2<T, U, R> = Fn2<T, U, R>;

          fn choose_left<T>(lhs: T, rhs: T) -> T {
            return lhs;
          }

          fn result_zip_with<T, U, R, E>(
            lhs: Result<T, E>,
            rhs: Result<U, E>,
            mapper: Mapper2<T, U, R>
          ) -> Result<R, E> {
            match lhs {
              Result.Ok(lhs_value) => {
                match rhs {
                  Result.Ok(rhs_value) => {
                    return Result.Ok(mapper(lhs_value, rhs_value));
                  }
                  Result.Err(error) => {
                    return Result.Err(error);
                  }
                }
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          fn main() -> i64 {
            let mapped: Result<i64, CoreError> =
              result_zip_with(Result.Ok(7), Result.Ok(3), choose_left);
            match mapped {
              Result.Ok(value) => {
                return value;
              }
              Result.Err(_) => {
                return -1;
              }
            }
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| {
            function.name == "__hof_result_zip_with_choose_left__i64__i64__i64__CoreError"
        })
        .expect("expected monomorphized alias result_zip_with helper for generic Fn2 callable");
    assert!(helper.generic_params.is_empty());

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "choose_left__i64__i64__i64__CoreError")
        .expect("expected generic Fn2 alias callable specialization");
    assert!(specialized.generic_params.is_empty());
}

#[test]
fn lowers_result_zip3_with_direct_ok_constructors_and_generic_named_fn3_alias_callable() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum CoreError {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          type Reducer<T, U, V, R> = Fn3<T, U, V, R>;

          fn choose_first<T>(lhs: T, mid: T, rhs: T) -> T {
            return lhs;
          }

          fn result_zip3_with<T, U, V, R, E>(
            first: Result<T, E>,
            second: Result<U, E>,
            third: Result<V, E>,
            mapper: Reducer<T, U, V, R>
          ) -> Result<R, E> {
            match first {
              Result.Ok(first_value) => {
                match second {
                  Result.Ok(second_value) => {
                    match third {
                      Result.Ok(third_value) => {
                        return Result.Ok(mapper(first_value, second_value, third_value));
                      }
                      Result.Err(error) => {
                        return Result.Err(error);
                      }
                    }
                  }
                  Result.Err(error) => {
                    return Result.Err(error);
                  }
                }
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          fn main() -> i64 {
            let mapped: Result<i64, CoreError> = result_zip3_with(
              Result.Ok(7),
              Result.Ok(3),
              Result.Ok(1),
              choose_first
            );
            match mapped {
              Result.Ok(value) => {
                return value;
              }
              Result.Err(_) => {
                return -1;
              }
            }
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| {
            function.name == "__hof_result_zip3_with_choose_first__i64__i64__i64__i64__CoreError"
        })
        .expect("expected monomorphized alias result_zip3_with helper for generic Fn3 callable");
    assert!(helper.generic_params.is_empty());

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "choose_first__i64__i64__i64__i64__CoreError")
        .expect("expected generic Fn3 alias callable specialization");
    assert!(specialized.generic_params.is_empty());
}
