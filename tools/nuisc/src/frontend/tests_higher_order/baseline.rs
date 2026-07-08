use super::*;

// Baseline callable specialization and named-function routing.
#[test]
fn combines_higher_order_specialization_with_trait_generic_monomorphization() {
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

          fn sum_two<T: Addable>(lhs: T, rhs: T) -> T {
            return lhs.add(rhs);
          }

          fn apply_and_sum(x: i64, y: i64, f: Fn1<i64, i64>) -> i64 {
            return sum_two(f(x), y);
          }

          fn main() -> i64 {
            return apply_and_sum(6, 1, |x: i64| -> i64 { return x; });
          }
        }
        "#,
    )
    .unwrap();

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized lambda function");
    let higher_order = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_and_sum_"))
        .expect("expected synthesized higher-order specialization");
    assert!(matches!(
        higher_order.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "sum_two__i64"
                && matches!(args.as_slice(), [NirExpr::Call { callee: inner, .. }, NirExpr::Var(y)] if inner == &lambda.name && y == "y")
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &higher_order.name
    ));
}

#[test]
fn lowers_generic_fn1_higher_order_lambda_family() {
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

          fn apply<T: Addable>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn main() -> i64 {
            return apply(6, |x: i64| -> i64 { return x + 1; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee == "impl.Addable.for.i64.add"
                || callee.starts_with("__lambda_main_")
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &higher_order_concrete.name
    ));
}

#[test]
fn lowers_result_map_and_and_then_higher_order_helpers() {
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

          fn result_map<T, R, E>(result: Result<T, E>, mapper: Fn1<T, R>) -> Result<R, E> {
            match result {
              Result.Ok(value) => {
                return Result.Ok(mapper(value));
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          fn result_and_then<T, R, E>(
            result: Result<T, E>,
            mapper: Fn1<T, Result<R, E>>
          ) -> Result<R, E> {
            match result {
              Result.Ok(value) => {
                return mapper(value);
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          fn main() -> i64 {
            let input: Result<i64, CoreError> = Result.Ok(7);
            let mapped: Result<i64, CoreError> = result_map(
              input,
              |value: i64| -> i64 { return value + 1; }
            );
            let chained: Result<i64, CoreError> = result_and_then(
              mapped,
              |value: i64| -> Result<i64, CoreError> { return Result.Ok(value * 2); }
            );
            match chained {
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

    assert!(module
        .functions
        .iter()
        .any(|function| function.name.starts_with("__hof_result_map_")));
    assert!(module
        .functions
        .iter()
        .any(|function| function.name.starts_with("__hof_result_and_then_")));
}

#[test]
fn lowers_option_map_higher_order_helper_with_direct_payload_constructor() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
          }

          fn option_map<T, R>(value: Option<T>, mapper: Fn1<T, R>) -> Option<R> {
            match value {
              Option.Some(payload) => {
                return Option.Some(mapper(payload));
              }
              Option.None => {
                return Option.None;
              }
            }
          }

          fn add5(value: i64) -> i64 {
            return value + 5;
          }

          fn main() -> i64 {
            let mapped: Option<i64> = option_map(Option.Some(7), add5);
            match mapped {
              Option.Some(value) => {
                return value;
              }
              Option.None => {
                return -1;
              }
            }
          }
        }
        "#,
    )
    .unwrap();

    assert!(module
        .functions
        .iter()
        .any(|function| function.name.starts_with("__hof_option_map_")));
}

#[test]
fn lowers_result_map_higher_order_helper_with_direct_ok_constructor() {
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

          fn result_map<T, R, E>(result: Result<T, E>, mapper: Fn1<T, R>) -> Result<R, E> {
            match result {
              Result.Ok(value) => {
                return Result.Ok(mapper(value));
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          fn add5(value: i64) -> i64 {
            return value + 5;
          }

          fn main() -> i64 {
            let mapped: Result<i64, CoreError> = result_map(Result.Ok(7), add5);
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

    assert!(module
        .functions
        .iter()
        .any(|function| function.name.starts_with("__hof_result_map_")));
}

#[test]
fn lowers_result_and_then_higher_order_helper_with_direct_err_constructor() {
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

          fn result_and_then<T, R, E>(
            result: Result<T, E>,
            mapper: Fn1<T, Result<R, E>>
          ) -> Result<R, E> {
            match result {
              Result.Ok(value) => {
                return mapper(value);
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          fn map_value(value: i64) -> Result<i64, CoreError> {
            return Result.Ok(value + 1);
          }

          fn main() -> i64 {
            let mapped: Result<i64, CoreError> =
              result_and_then(Result.Err(CoreError.InvalidInput), map_value);
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

    assert!(module
        .functions
        .iter()
        .any(|function| function.name.starts_with("__hof_result_and_then_")));
}

#[test]
fn lowers_result_map_with_direct_ok_constructor_and_generic_named_callable() {
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

          fn id<T>(value: T) -> T {
            return value;
          }

          fn result_map<T, R, E>(result: Result<T, E>, mapper: Fn1<T, R>) -> Result<R, E> {
            match result {
              Result.Ok(value) => {
                return Result.Ok(mapper(value));
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          fn main() -> i64 {
            let mapped: Result<i64, CoreError> = result_map(Result.Ok(7), id);
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
        .find(|function| function.name == "__hof_result_map_id__i64__i64__CoreError")
        .expect("expected monomorphized result_map helper for generic named callable");
    assert!(helper.generic_params.is_empty());

    let id_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "id__i64__i64__CoreError")
        .expect("expected generic callable specialization");
    assert!(id_specialized.generic_params.is_empty());
}

#[test]
fn lowers_result_and_then_with_direct_ok_constructor_and_generic_named_callable() {
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

          fn lift_ok<T, E>(value: T) -> Result<T, E> {
            return Result.Ok(value);
          }

          fn result_and_then<T, R, E>(
            result: Result<T, E>,
            mapper: Fn1<T, Result<R, E>>
          ) -> Result<R, E> {
            match result {
              Result.Ok(value) => {
                return mapper(value);
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          fn main() -> i64 {
            let mapped: Result<i64, CoreError> = result_and_then(Result.Ok(7), lift_ok);
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
        .find(|function| function.name == "__hof_result_and_then_lift_ok__i64__i64__CoreError")
        .expect("expected monomorphized result_and_then helper for generic named callable");
    assert!(helper.generic_params.is_empty());

    let lift_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "lift_ok__i64__i64__CoreError")
        .expect("expected generic result callable specialization");
    assert!(lift_specialized.generic_params.is_empty());
}
