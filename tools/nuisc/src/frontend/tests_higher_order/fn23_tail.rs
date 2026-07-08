use super::*;

#[test]
fn lowers_capturing_generic_lambda_with_bound_inside_nested_while_match() {
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

          fn choose<T: Addable>(value: T, extra: T, mode: i64) -> T {
            while mode > 0 {
              match mode {
                1 => {
                  return apply(value, |x: T| -> T { return x.add(extra); });
                }
                _ => {
                  return value;
                }
              }
            }
            return value;
          }

          fn main() -> i64 {
            return choose(2, 3, 1);
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized higher-order helper inside nested control flow");
    assert!(helper.generic_params.is_empty());
    assert_eq!(helper.params.len(), 2);
    assert!(matches!(
        helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_choose_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(x), NirExpr::Var(extra)] if x == "x" && extra == "__capture_f_extra_0"
                )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_choose_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized captured lambda inside nested control flow");
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

    let choose = module
        .functions
        .iter()
        .find(|function| function.name == "choose__i64")
        .expect("expected monomorphized control-flow generic function");
    assert!(matches!(
        choose.body.first(),
        Some(NirStmt::While { body, .. })
            if matches!(
                body.as_slice(),
                [NirStmt::If { then_body, else_body, .. }]
                    if matches!(
                        then_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                            if callee == &helper.name
                                && matches!(
                                    args.as_slice(),
                                    [NirExpr::Var(value), NirExpr::Var(extra)]
                                        if value == "value" && extra == "extra"
                                )
                    ) && matches!(
                        else_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Var(name)))] if name == "value"
                    )
            )
    ));
}

#[test]
fn lowers_generic_fn2_lambda_method_call_with_present_bound() {
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

          fn apply2<T: Addable>(x: T, y: T, f: Fn2<T, T, T>) -> T {
            return f(x, y);
          }

          fn bump<T: Addable>(lhs: T, rhs: T) -> T {
            return apply2(lhs, rhs, |x: T, y: T| -> T { return x.add(y); });
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
            function.name.starts_with("__hof_apply2_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized Fn2 higher-order helper");
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
        .expect("expected monomorphized generic Fn2 lambda specialization");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Addable.for.i64.add"
                && matches!(args.as_slice(), [NirExpr::Var(lhs), NirExpr::Var(rhs)] if lhs == "x" && rhs == "y")
    ));
}

#[test]
fn lowers_forwarded_fn2_callable_parameter_into_nested_higher_order_template_call() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn add_pair(lhs: i64, rhs: i64) -> i64 {
            return lhs + rhs;
          }

          fn apply2<T>(x: T, y: T, f: Fn2<T, T, T>) -> T {
            return f(x, y);
          }

          fn chain2(x: i64, y: i64, f: Fn2<i64, i64, i64>) -> i64 {
            return apply2(f(x, y), y, add_pair);
          }

          fn relay2(x: i64, y: i64, f: Fn2<i64, i64, i64>) -> i64 {
            return chain2(x, y, f);
          }

          fn main() -> i64 {
            return relay2(6, 2, |x: i64, y: i64| -> i64 { return x - y; });
          }
        }
        "#,
    )
    .unwrap();

    let relay_helper = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_relay2_"))
        .expect("expected forwarding Fn2 relay higher-order helper");
    assert!(relay_helper.generic_params.is_empty());
    assert!(matches!(
        relay_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee.starts_with("__hof_chain2_")
    ));

    let chain_helper = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_chain2_"))
        .expect("expected forwarded Fn2 chain higher-order helper");
    assert!(chain_helper.generic_params.is_empty());
    assert!(matches!(
        chain_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee.starts_with("__hof_apply2_")
    ));
}

#[test]
fn lowers_generic_fn3_lambda_method_call_with_present_bound() {
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

          fn apply3<T: Addable>(x: T, y: T, z: T, f: Fn3<T, T, T, T>) -> T {
            return f(x, y, z);
          }

          fn bump<T: Addable>(x: T, y: T, z: T) -> T {
            return apply3(x, y, z, |lhs: T, mid: T, rhs: T| -> T {
              return lhs.add(mid).add(rhs);
            });
          }

          fn main() -> i64 {
            return bump(2, 3, 4);
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply3_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized Fn3 higher-order helper");
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
        .expect("expected monomorphized generic Fn3 lambda specialization");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call {
            callee: outer_callee,
            args: outer_args
        }))] if outer_callee == "impl.Addable.for.i64.add"
            && matches!(
                outer_args.as_slice(),
                [
                    NirExpr::Call { callee: inner_callee, args: inner_args },
                    NirExpr::Var(rhs)
                ] if inner_callee == "impl.Addable.for.i64.add"
                    && matches!(inner_args.as_slice(), [NirExpr::Var(lhs), NirExpr::Var(mid)] if lhs == "lhs" && mid == "mid")
                    && rhs == "rhs"
            )
    ));
}

#[test]
fn lowers_forwarded_fn3_callable_parameter_into_nested_higher_order_template_call() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn add_three(lhs: i64, mid: i64, rhs: i64) -> i64 {
            return lhs + mid + rhs;
          }

          fn apply3<T>(x: T, y: T, z: T, f: Fn3<T, T, T, T>) -> T {
            return f(x, y, z);
          }

          fn chain3(x: i64, y: i64, z: i64, f: Fn3<i64, i64, i64, i64>) -> i64 {
            return apply3(f(x, y, z), y, z, add_three);
          }

          fn relay3(x: i64, y: i64, z: i64, f: Fn3<i64, i64, i64, i64>) -> i64 {
            return chain3(x, y, z, f);
          }

          fn main() -> i64 {
            return relay3(6, 2, 1, |x: i64, y: i64, z: i64| -> i64 { return x - y - z; });
          }
        }
        "#,
    )
    .unwrap();

    let relay_helper = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_relay3_"))
        .expect("expected forwarding Fn3 relay higher-order helper");
    assert!(relay_helper.generic_params.is_empty());
    assert!(matches!(
        relay_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee.starts_with("__hof_chain3_")
    ));

    let chain_helper = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_chain3_"))
        .expect("expected forwarded Fn3 chain higher-order helper");
    assert!(chain_helper.generic_params.is_empty());
    assert!(matches!(
        chain_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee.starts_with("__hof_apply3_")
    ));
}

#[test]
fn lowers_higher_order_call_scrutinee_match_inside_while_via_hoisted_let() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn apply(x: i64, f: Fn1<i64, i64>) -> i64 {
            return f(x);
          }

          fn main() -> i64 {
            let state: i64 = 2;
            while state > 0 {
              match apply(state, |x: i64| -> i64 { return x + 1; }) {
                3 => { return 7; },
                _ => { return 9; }
              }
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    match &function.body[1] {
        NirStmt::While { body, .. } => {
            assert!(matches!(
                body.as_slice(),
                [
                    NirStmt::Let { name, value: NirExpr::Call { .. }, .. },
                    NirStmt::If { .. }
                ] if name.starts_with("__match_scrutinee_")
            ));
        }
        other => panic!("expected while statement, found {other:?}"),
    }
}

#[test]
fn lowers_generic_fn3_higher_order_lambda_family() {
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

          fn apply3<T: Addable>(x: T, y: T, z: T, f: Fn3<T, T, T, T>) -> T {
            return f(x, y, z);
          }

          fn main() -> i64 {
            return apply3(5, 1, 1, |x: i64, y: i64, z: i64| -> i64 { return x + y + z; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply3_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized Fn3 higher-order helper");
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
fn lowers_generic_fn3_alias_higher_order_lambda_family() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Reducer<T> = Fn3<T, T, T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn apply3<T: Addable>(x: T, y: T, z: T, f: Reducer<T>) -> T {
            return f(x, y, z);
          }

          fn main() -> i64 {
            return apply3(5, 1, 1, |x: i64, y: i64, z: i64| -> i64 { return x + y + z; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply3_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized Fn3 alias higher-order helper");
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
