use super::*;

#[test]
fn lowers_capturing_generic_lambda_ordering_operator_with_present_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Orderable {
            fn lt(lhs: Self, rhs: Self) -> bool;
            fn le(lhs: Self, rhs: Self) -> bool;
            fn gt(lhs: Self, rhs: Self) -> bool;
            fn ge(lhs: Self, rhs: Self) -> bool;
          }

          impl Orderable for Pair {
            fn lt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value < rhs.value;
            }

            fn le(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value <= rhs.value;
            }

            fn gt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value > rhs.value;
            }

            fn ge(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value >= rhs.value;
            }
          }

          fn apply<T: Orderable>(x: T, f: Fn1<T, bool>) -> bool {
            return f(x);
          }

          fn less<T: Orderable>(value: T, other: T) -> bool {
            return apply(value, |x: T| -> bool { return x < other; });
          }

          fn main() -> i64 {
            let lhs: Pair = Pair { value: 2 };
            let rhs: Pair = Pair { value: 3 };
            if less(lhs, rhs) {
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
        .expect("expected monomorphized bool-returning ordering helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_less_")
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
            function.name.starts_with("__lambda_less_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized ordering captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Orderable.for.Pair.lt"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));
}

#[test]
fn lowers_capturing_generic_lambda_ordering_le_operator_with_present_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Orderable {
            fn lt(lhs: Self, rhs: Self) -> bool;
            fn le(lhs: Self, rhs: Self) -> bool;
            fn gt(lhs: Self, rhs: Self) -> bool;
            fn ge(lhs: Self, rhs: Self) -> bool;
          }

          impl Orderable for Pair {
            fn lt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value < rhs.value;
            }

            fn le(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value <= rhs.value;
            }

            fn gt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value > rhs.value;
            }

            fn ge(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value >= rhs.value;
            }
          }

          fn apply<T: Orderable>(x: T, f: Fn1<T, bool>) -> bool {
            return f(x);
          }

          fn less_eq<T: Orderable>(value: T, other: T) -> bool {
            return apply(value, |x: T| -> bool { return x <= other; });
          }

          fn main() -> i64 {
            let lhs: Pair = Pair { value: 2 };
            let rhs: Pair = Pair { value: 3 };
            if less_eq(lhs, rhs) {
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
        .expect("expected monomorphized bool-returning ordering <= helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_less_eq_")
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
            function.name.starts_with("__lambda_less_eq_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized ordering <= captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Orderable.for.Pair.le"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));
}

#[test]
fn lowers_capturing_generic_lambda_ordering_gt_operator_with_present_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Orderable {
            fn lt(lhs: Self, rhs: Self) -> bool;
            fn le(lhs: Self, rhs: Self) -> bool;
            fn gt(lhs: Self, rhs: Self) -> bool;
            fn ge(lhs: Self, rhs: Self) -> bool;
          }

          impl Orderable for Pair {
            fn lt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value < rhs.value;
            }

            fn le(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value <= rhs.value;
            }

            fn gt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value > rhs.value;
            }

            fn ge(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value >= rhs.value;
            }
          }

          fn apply<T: Orderable>(x: T, f: Fn1<T, bool>) -> bool {
            return f(x);
          }

          fn greater<T: Orderable>(value: T, other: T) -> bool {
            return apply(value, |x: T| -> bool { return x > other; });
          }

          fn main() -> i64 {
            let lhs: Pair = Pair { value: 3 };
            let rhs: Pair = Pair { value: 2 };
            if greater(lhs, rhs) {
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
        .expect("expected monomorphized bool-returning ordering > helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_greater_")
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
            function.name.starts_with("__lambda_greater_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized ordering > captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Orderable.for.Pair.gt"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));
}

#[test]
fn lowers_capturing_generic_lambda_ordering_ge_operator_with_present_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Orderable {
            fn lt(lhs: Self, rhs: Self) -> bool;
            fn le(lhs: Self, rhs: Self) -> bool;
            fn gt(lhs: Self, rhs: Self) -> bool;
            fn ge(lhs: Self, rhs: Self) -> bool;
          }

          impl Orderable for Pair {
            fn lt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value < rhs.value;
            }

            fn le(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value <= rhs.value;
            }

            fn gt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value > rhs.value;
            }

            fn ge(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value >= rhs.value;
            }
          }

          fn apply<T: Orderable>(x: T, f: Fn1<T, bool>) -> bool {
            return f(x);
          }

          fn greater_eq<T: Orderable>(value: T, other: T) -> bool {
            return apply(value, |x: T| -> bool { return x >= other; });
          }

          fn main() -> i64 {
            let lhs: Pair = Pair { value: 3 };
            let rhs: Pair = Pair { value: 2 };
            if greater_eq(lhs, rhs) {
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
        .expect("expected monomorphized bool-returning ordering >= helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_greater_eq_")
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
            function.name.starts_with("__lambda_greater_eq_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized ordering >= captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Orderable.for.Pair.ge"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));
}

#[test]
fn lowers_generic_lambda_unary_not_with_present_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Notable {
            fn not(value: Self) -> bool;
          }

          impl Notable for Pair {
            fn not(value: Pair) -> bool {
              return value.value == 0;
            }
          }

          fn apply<T: Notable>(x: T, f: Fn1<T, bool>) -> bool {
            return f(x);
          }

          fn empty<T: Notable>(value: T) -> bool {
            return apply(value, |x: T| -> bool { return !x; });
          }

          fn main() -> i64 {
            let value: Pair = Pair { value: 0 };
            if empty(value) {
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
        .expect("expected monomorphized unary-not higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_empty_")
                && matches!(args.as_slice(), [NirExpr::Var(x)] if x == "x")
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_empty_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized unary-not generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Notable.for.Pair.not"
                && matches!(args.as_slice(), [NirExpr::Var(value)] if value == "x")
    ));
}
