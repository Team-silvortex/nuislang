use super::*;

#[test]
fn lowers_explicit_generic_fn1_higher_order_call_with_zero_arg_generic_argument() {
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

          fn typed_zero<T>() -> T {
            return 0;
          }

          fn apply<T: Addable>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn main() -> i64 {
            return apply<i64>(typed_zero(), |x: i64| -> i64 { return x + 1; });
          }
        }
        "#,
    )
    .unwrap();

    let specialized_zero = module
        .functions
        .iter()
        .find(|function| function.name == "typed_zero__i64")
        .expect("expected zero-arg generic call to specialize through explicit higher-order call");
    assert!(specialized_zero.generic_params.is_empty());

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected explicit-generic higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == &higher_order_concrete.name
                && matches!(
                    args.as_slice(),
                    [NirExpr::Call { callee: zero_callee, .. }]
                        if zero_callee == "typed_zero__i64"
                )
    ));
}

// Payload alias, async, and recursive higher-order specialization.
#[test]
fn lowers_generic_fn1_alias_higher_order_lambda_family() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Mapper<T> = Fn1<T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn apply<T: Addable>(x: T, f: Mapper<T>) -> T {
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
fn lowers_capturing_lambda_through_generic_fn1_template_parameter() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn apply<T>(value: T, mapper: Fn1<T, T>) -> T {
            return mapper(value);
          }

          fn main() -> i64 {
            let seed: i64 = 6;
            return apply(1, |x: i64| -> i64 { return x + seed; });
          }
        }
        "#,
    )
    .unwrap();

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized captured generic lambda");
    assert!(lambda.generic_params.is_empty());
    assert_eq!(lambda.params.len(), 2);

    let helper = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized higher-order helper with capture threading");
    assert!(helper.generic_params.is_empty());
    assert_eq!(helper.params.len(), 2);
    assert!(matches!(
        helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &lambda.name
                && matches!(args.as_slice(), [NirExpr::Var(x), NirExpr::Var(seed)] if x == "value" && seed == "__capture_mapper_seed_0")
    ));
}

#[test]
fn lowers_capturing_lambda_through_generic_fn1_alias_parameter() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Mapper<T> = Fn1<T, T>;

          fn apply<T>(value: T, mapper: Mapper<T>) -> T {
            return mapper(value);
          }

          fn main() -> i64 {
            let seed: i64 = 6;
            return apply(1, |x: i64| -> i64 { return x + seed; });
          }
        }
        "#,
    )
    .unwrap();

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized captured alias generic lambda");
    assert!(lambda.generic_params.is_empty());
    let helper = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected alias higher-order helper with capture threading");
    assert!(helper.generic_params.is_empty());
    assert_eq!(helper.params.len(), 2);
    assert!(matches!(
        helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &lambda.name
                && matches!(args.as_slice(), [NirExpr::Var(x), NirExpr::Var(seed)] if x == "value" && seed == "__capture_mapper_seed_0")
    ));
}

#[test]
fn lowers_capturing_lambda_through_generic_fn2_and_fn3_parameters() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Reducer<T> = Fn3<T, T, T, T>;

          fn apply2<T>(lhs: T, rhs: T, mapper: Fn2<T, T, T>) -> T {
            return mapper(lhs, rhs);
          }

          fn apply3<T>(lhs: T, mid: T, rhs: T, mapper: Reducer<T>) -> T {
            return mapper(lhs, mid, rhs);
          }

          fn main() -> i64 {
            let seed: i64 = 6;
            let pair: i64 = apply2(1, 2, |x: i64, y: i64| -> i64 { return x + y + seed; });
            return apply3(pair, 3, 4, |lhs: i64, mid: i64, rhs: i64| -> i64 {
              return lhs + mid + rhs + seed;
            });
          }
        }
        "#,
    )
    .unwrap();

    let apply2_helper = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply2_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized Fn2 helper");
    assert!(apply2_helper.generic_params.is_empty());
    assert_eq!(apply2_helper.params.len(), 3);
    assert!(matches!(
        apply2_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_main_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs), NirExpr::Var(seed)]
                        if lhs == "lhs" && rhs == "rhs" && seed == "__capture_mapper_seed_0"
                )
    ));

    let apply3_helper = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply3_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized Fn3 alias helper");
    assert!(apply3_helper.generic_params.is_empty());
    assert_eq!(apply3_helper.params.len(), 4);
    assert!(matches!(
        apply3_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_main_")
                && matches!(
                    args.as_slice(),
                    [
                        NirExpr::Var(lhs),
                        NirExpr::Var(mid),
                        NirExpr::Var(rhs),
                        NirExpr::Var(seed)
                    ] if lhs == "lhs"
                        && mid == "mid"
                        && rhs == "rhs"
                        && seed == "__capture_mapper_seed_0"
                )
    ));
}

#[test]
fn lowers_explicit_generic_higher_order_template_call_inside_template_body() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn add_one(value: i64) -> i64 {
            return value + 1;
          }

          fn apply<T>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn chain(x: i64, f: Fn1<i64, i64>) -> i64 {
            return apply<i64>(f(x), add_one);
          }

          fn main() -> i64 {
            return chain(6, |x: i64| -> i64 { return x + 2; });
          }
        }
        "#,
    )
    .unwrap();

    let apply_helper_count = module
        .functions
        .iter()
        .filter(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .count();
    assert!(
        apply_helper_count >= 1,
        "expected nested explicit-generic higher-order expansion to emit an apply specialization, found {apply_helper_count}"
    );

    let chain_helper = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_chain_"))
        .expect("expected explicit-generic chain higher-order helper");
    assert!(chain_helper.generic_params.is_empty());
    assert!(matches!(
        chain_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee.starts_with("__hof_apply_") && callee.ends_with("__i64")
    ));
}

#[test]
fn lowers_forwarded_callable_parameter_into_nested_higher_order_template_call() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn add_one(value: i64) -> i64 {
            return value + 1;
          }

          fn apply<T>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn chain(x: i64, f: Fn1<i64, i64>) -> i64 {
            return apply(f(x), add_one);
          }

          fn relay(x: i64, f: Fn1<i64, i64>) -> i64 {
            return chain(x, f);
          }

          fn main() -> i64 {
            return relay(6, |x: i64| -> i64 { return x + 2; });
          }
        }
        "#,
    )
    .unwrap();

    let relay_helper = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_relay_"))
        .expect("expected forwarding relay higher-order helper");
    assert!(relay_helper.generic_params.is_empty());
    assert!(matches!(
        relay_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee.starts_with("__hof_chain_")
    ));

    let chain_helper = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_chain_"))
        .expect("expected forwarded chain higher-order helper");
    assert!(chain_helper.generic_params.is_empty());
    assert!(matches!(
        chain_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee.starts_with("__hof_apply_")
    ));
}

#[test]
fn lowers_nested_generic_fn1_alias_higher_order_lambda_family() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Mapper<T> = Fn1<T, T>;
          type NestedMapper<T> = Mapper<T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn apply<T: Addable>(x: T, f: NestedMapper<T>) -> T {
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
        .expect("expected monomorphized nested higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

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
fn lowers_generic_payload_alias_into_generic_fn1_higher_order_lambda_family() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;
          type Mapper<T> = Fn1<T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          struct Just<T> {
            value: T,
          }

          fn apply_payload<T: Addable>(value: JustAlias<T>, f: Mapper<T>) -> T {
            match value {
              JustAlias<T>(payload) => {
                return f(payload);
              }
              _ => {
                return value.value;
              }
            }
          }

          fn main() -> i64 {
            return apply_payload(JustAlias<i64>(6), |x: i64| -> i64 { return x + 1; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_payload_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized payload higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

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
fn lowers_inferred_generic_payload_alias_into_generic_fn1_higher_order_lambda_family() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;
          type Mapper<T> = Fn1<T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          struct Just<T> {
            value: T,
          }

          fn apply_payload<T: Addable>(value: JustAlias<T>, f: Mapper<T>) -> T {
            match value {
              JustAlias<T>(payload) => {
                return f(payload);
              }
              _ => {
                return value.value;
              }
            }
          }

          fn main() -> i64 {
            return apply_payload(JustAlias(6), |x: i64| -> i64 { return x + 1; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_payload_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized inferred payload higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

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
