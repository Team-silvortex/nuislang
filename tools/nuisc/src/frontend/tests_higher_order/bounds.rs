use super::*;

#[test]
fn rejects_generic_lambda_method_call_without_required_bound() {
    let error = parse_nuis_module(
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

          fn apply<T>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn bump<T>(value: T) -> T {
            return apply(value, |x: T| -> T { return x.add(x); });
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "function `bump` body lambda body calls method `add` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn rejects_generic_higher_order_specialization_method_call_without_required_bound() {
    let error = parse_nuis_module(
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

          fn apply<T>(x: T, f: Fn1<T, T>) -> T {
            let local = f(x);
            return local.add(x);
          }

          fn bump<T>(value: T) -> T {
            return apply(value, |x: T| -> T { return x; });
          }

          fn main() -> i64 {
            return bump(0);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "function `apply` body higher-order specialization body calls method `add` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn rejects_generic_higher_order_specialization_without_required_bound_inside_nested_while_match() {
    let error = parse_nuis_module(
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

          fn apply<T>(x: T, f: Fn1<T, T>) -> T {
            let local = f(x);
            return local.add(x);
          }

          fn choose<T>(value: T, mode: i64) -> T {
            while mode > 0 {
              match mode {
                1 => {
                  return apply(value, |x: T| -> T { return x; });
                }
                _ => {
                  return value;
                }
              }
            }
            return value;
          }

          fn main() -> i64 {
            return choose(0, 1);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "function `apply` body higher-order specialization body calls method `add` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn rejects_result_map_higher_order_specialization_method_call_without_required_bound() {
    let error = parse_nuis_module(
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

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn result_map<T, R, E>(result: Result<T, E>, mapper: Fn1<T, R>) -> Result<R, E> {
            match result {
              Result.Ok(value) => {
                let mapped = mapper(value);
                return Result.Ok(mapped.add(mapped));
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          fn bump<T, E>(input: Result<T, E>) -> Result<T, E> {
            return result_map(input, |x: T| -> T { return x; });
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("function `result_map` body higher-order specialization body match-arm"),
        "{error}"
    );
    assert!(
        error.contains(
            "calls method `add` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn rejects_result_and_then_higher_order_specialization_method_call_without_required_bound() {
    let error = parse_nuis_module(
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
                let mapped = mapper(value);
                match mapped {
                  Result.Ok(inner) => {
                    return Result.Ok(inner.add(inner));
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

          fn bump<T, E>(input: Result<T, E>) -> Result<T, E> {
            return result_and_then(input, |x: T| -> Result<T, E> { return Result.Ok(x); });
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("function `result_and_then` body higher-order specialization body"),
        "{error}"
    );
    assert!(
        error.contains(
            "calls method `add` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn lowers_generic_lambda_method_call_with_present_bound() {
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

          fn bump<T: Addable>(value: T) -> T {
            return apply(value, |x: T| -> T { return x.add(x); });
          }

          fn main() -> i64 {
            return bump(2);
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
            if callee.starts_with("__lambda_bump_")
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_bump_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Addable.for.i64.add"
                && matches!(args.as_slice(), [NirExpr::Var(lhs), NirExpr::Var(rhs)] if lhs == "x" && rhs == "x")
    ));
}

#[test]
fn lowers_capturing_generic_lambda_method_call_with_present_bound() {
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

          fn bump<T: Addable>(value: T, extra: T) -> T {
            return apply(value, |x: T| -> T { return x.add(extra); });
          }

          fn main() -> i64 {
            return bump(2, 3);
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
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_bump_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(x), NirExpr::Var(extra)] if x == "x" && extra == "__capture_f_extra_0"
                )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_bump_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    assert_eq!(lambda.params.len(), 2);
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Addable.for.i64.add"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));
}

#[test]
fn lowers_higher_order_generic_lambda_with_qualified_helper_trait_bound() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn apply<T: Helper.Addable>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn bump<T: Helper.Addable>(value: T, extra: T) -> T {
            return apply(value, |x: T| -> T { return x.add(extra); });
          }

          fn main() -> i64 {
            return bump(2, 3);
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

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected helper-trait monomorphized higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_bump_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(x), NirExpr::Var(extra)]
                        if x == "x" && extra == "__capture_f_extra_0"
                )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_bump_") && function.name.ends_with("__i64")
        })
        .expect("expected helper-trait monomorphized captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Helper.Addable.for.i64.add"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));
}
