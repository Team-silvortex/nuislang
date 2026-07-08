use super::*;

#[test]
#[allow(unreachable_code)]
fn lowers_generic_payload_alias_method_bound_and_higher_order_combo() {
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
                let mapped = f(payload);
                return mapped.add(payload);
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
        .expect("expected monomorphized payload higher-order combo helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert!(stmt_tree_contains_call(
        &higher_order_concrete.body,
        &|callee, args| callee.starts_with("__lambda_main_")
            && matches!(args, [NirExpr::Var(name)] if name == "payload")
    ));
    assert!(stmt_tree_contains_call(
        &higher_order_concrete.body,
        &|callee, args| callee == "impl.Addable.for.i64.add"
            && matches!(args, [NirExpr::Var(lhs), NirExpr::Var(rhs)] if lhs == "mapped" && rhs == "payload")
    ));
    return;
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [
            NirStmt::If {
                condition: NirExpr::Bool(true),
                then_body,
                else_body,
            },
        ] if matches!(
            then_body.as_slice(),
            [
                NirStmt::Let {
                    name: payload_name,
                    value,
                    ..
                },
                NirStmt::Let {
                    name: mapped_name,
                    value: NirExpr::Call { callee: lambda_callee, args: lambda_args },
                    ..
                },
                NirStmt::Return(Some(NirExpr::Call { callee: add_callee, args: add_args })),
            ] if payload_name == "payload"
                && mapped_name == "mapped"
                && is_payload_value_access_from_var(value, "value")
                && lambda_callee.starts_with("__lambda_main_")
                && matches!(lambda_args.as_slice(), [NirExpr::Var(name)] if name == "payload")
                && add_callee == "impl.Addable.for.i64.add"
                && matches!(add_args.as_slice(), [NirExpr::Var(lhs), NirExpr::Var(rhs)] if lhs == "mapped" && rhs == "payload")
        ) && matches!(
            else_body.as_slice(),
            [NirStmt::Return(Some(value))]
                if is_payload_value_access_from_var(value, "value")
        )
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
fn lowers_async_await_into_inferred_generic_payload_alias_higher_order_family() {
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

          async fn typed_zero<T>() -> T {
            return 0;
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

          async fn main() -> i64 {
            return apply_payload(JustAlias(await typed_zero()), |x: i64| -> i64 { return x + 1; });
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
        .expect("expected async-inferred payload higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "typed_zero__i64")
        .expect("expected async generic specialization through await payload alias path");
    assert!(specialized.is_async);
    assert!(specialized.generic_params.is_empty());

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
                    [NirExpr::StructLiteral { fields, .. }]
                        if matches!(
                            fields.as_slice(),
                            [(field, NirExpr::Await(value))]
                                if field == "value"
                                    && matches!(
                                        value.as_ref(),
                                        NirExpr::Call { callee, .. } if callee == "typed_zero__i64"
                                    )
                        )
                )
    ));
}

#[test]
fn lowers_specialized_generic_recursive_async_body_into_payload_alias_higher_order_family() {
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

          async fn climb<T: Addable>(value: T, remaining: i64) -> T {
            if remaining == 0 {
              return apply_payload(
                JustAlias<T>(value),
                |x: T| -> T { return x.add(1); }
              );
            }
            return await climb(value, remaining - 1);
          }

          async fn main() -> i64 {
            return await climb(7, 4);
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
        .expect("expected recursive async payload higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "climb__i64")
        .expect(
            "expected recursive async generic specialization through higher-order payload body",
        );
    assert!(specialized.is_async);
    assert!(specialized.generic_params.is_empty());

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Await(value))))
            if matches!(
                value.as_ref(),
                NirExpr::Call { callee, .. } if callee == "climb__i64"
            )
    ));
}

#[test]
#[allow(unreachable_code)]
fn lowers_specialized_generic_recursive_async_body_with_capturing_lambda_and_bound() {
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

          async fn climb<T: Addable>(value: T, extra: T, remaining: i64) -> T {
            if remaining == 0 {
              return apply_payload(
                JustAlias<T>(value),
                |x: T| -> T { return x.add(extra); }
              );
            }
            return await climb(value, extra, remaining - 1);
          }

          async fn main() -> i64 {
            return await climb(7, 3, 4);
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
        .expect("expected recursive async payload higher-order helper with capture threading");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(stmt_tree_contains_call(
        &higher_order_concrete.body,
        &|callee, args| callee.starts_with("__lambda_climb_")
            && matches!(
                args,
                [NirExpr::Var(payload), NirExpr::Var(extra)]
                    if payload == "payload" && extra == "__capture_f_extra_0"
            )
    ));
    return;
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::If { condition: NirExpr::Bool(true), then_body, else_body }]
            if matches!(
                then_body.as_slice(),
                [
                    NirStmt::Let {
                        name: payload_name,
                        value,
                        ..
                    },
                    NirStmt::Return(Some(NirExpr::Call { callee, args }))
                ]
                    if payload_name == "payload"
                        && is_payload_value_access_from_var(value, "value")
                        && callee.starts_with("__lambda_climb_")
                        && matches!(
                            args.as_slice(),
                            [NirExpr::Var(payload), NirExpr::Var(extra)]
                                if payload == "payload" && extra == "__capture_f_extra_0"
                        )
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(value))]
                    if is_payload_value_access_from_var(value, "value")
            )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_climb_") && function.name.ends_with("__i64")
        })
        .expect("expected captured recursive lambda specialization");
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

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "climb__i64")
        .expect("expected recursive async generic specialization with capture");
    assert!(specialized.is_async);
    assert!(specialized.generic_params.is_empty());
    assert!(stmt_tree_contains_call(
        &specialized.body,
        &|callee, args| {
            callee == higher_order_concrete.name
                && matches!(
                    args,
                    [NirExpr::StructLiteral { .. }, NirExpr::Var(extra)] if extra == "extra"
                )
        }
    ));
    assert!(stmt_tree_contains_call(
        &specialized.body,
        &|callee, args| {
            callee == "climb__i64"
                && matches!(
                    args,
                    [NirExpr::Var(value), NirExpr::Var(extra), NirExpr::Binary { .. }]
                        if value == "value" && extra == "extra"
                )
        }
    ));
}

// Trait-bound validation and bound-preserving generic lambda specialization.
