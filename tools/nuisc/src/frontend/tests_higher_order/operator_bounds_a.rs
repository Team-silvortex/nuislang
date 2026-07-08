use super::*;

#[test]
fn lowers_capturing_generic_lambda_explicit_trait_call_with_present_bound() {
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
            return apply(value, |x: T| -> T { return Addable.add(x, extra); });
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
        .expect("expected monomorphized explicit-trait captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
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
fn lowers_capturing_generic_lambda_operator_call_with_present_bound() {
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
            return apply(value, |x: T| -> T { return x + extra; });
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
        .expect("expected monomorphized operator captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Binary { lhs, rhs, .. }))]
            if matches!(lhs.as_ref(), NirExpr::Var(name) if name == "x")
                && matches!(rhs.as_ref(), NirExpr::Var(name) if name == &capture_param_name)
    ));
}

#[test]
fn lowers_capturing_generic_lambda_equality_operator_with_present_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Equatable {
            fn eq(lhs: Self, rhs: Self) -> bool;
          }

          impl Equatable for Pair {
            fn eq(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value == rhs.value;
            }
          }

          fn apply<T: Equatable>(x: T, f: Fn1<T, bool>) -> bool {
            return f(x);
          }

          fn same<T: Equatable>(value: T, other: T) -> bool {
            return apply(value, |x: T| -> bool { return x == other; });
          }

          fn main() -> i64 {
            let lhs: Pair = Pair { value: 2 };
            let rhs: Pair = Pair { value: 2 };
            if same(lhs, rhs) {
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized bool-returning higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_same_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(x), NirExpr::Var(other)]
                        if x == "x" && other == "__capture_f_other_0"
                )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_same_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized equality captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    assert_eq!(lambda.params.len(), 2);
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Equatable.for.Pair.eq"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));
}

#[test]
fn lowers_generic_lambda_unary_neg_with_present_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Negatable {
            fn neg(value: Self) -> Self;
          }

          impl Negatable for Pair {
            fn neg(value: Pair) -> Pair {
              return Pair { value: 0 - value.value };
            }
          }

          fn apply<T: Negatable>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn flip<T: Negatable>(value: T) -> T {
            return apply(value, |x: T| -> T { return -x; });
          }

          fn main() -> i64 {
            return flip(Pair { value: 7 }).value;
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized unary-neg higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_flip_")
                && matches!(args.as_slice(), [NirExpr::Var(x)] if x == "x")
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_flip_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized unary-neg generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Negatable.for.Pair.neg"
                && matches!(args.as_slice(), [NirExpr::Var(value)] if value == "x")
    ));
}

#[test]
fn lowers_capturing_generic_lambda_inequality_operator_with_present_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Equatable {
            fn eq(lhs: Self, rhs: Self) -> bool;
          }

          impl Equatable for Pair {
            fn eq(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value == rhs.value;
            }
          }

          fn apply<T: Equatable>(x: T, f: Fn1<T, bool>) -> bool {
            return f(x);
          }

          fn different<T: Equatable>(value: T, other: T) -> bool {
            return apply(value, |x: T| -> bool { return x != other; });
          }

          fn main() -> i64 {
            let lhs: Pair = Pair { value: 2 };
            let rhs: Pair = Pair { value: 3 };
            if different(lhs, rhs) {
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized bool-returning inequality helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_different_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(x), NirExpr::Var(other)]
                        if x == "x" && other == "__capture_f_other_0"
                )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_different_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized inequality captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Binary { lhs, rhs, .. }))]
            if matches!(
                lhs.as_ref(),
                NirExpr::Call { callee, args }
                    if callee == "impl.Equatable.for.Pair.eq"
                        && matches!(
                            args.as_slice(),
                            [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                                if lhs == "x" && rhs == &capture_param_name
                        )
            ) && matches!(rhs.as_ref(), NirExpr::Bool(false))
    ));
}
