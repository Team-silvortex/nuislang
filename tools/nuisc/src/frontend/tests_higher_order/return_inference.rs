use super::*;

#[test]
fn lowers_higher_order_generic_lambda_with_qualified_helper_trait_bound_through_alias_chain() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          type Alias<T> = T;
          type Outer<T> = Alias<T>;

          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn apply<T: Helper.Addable>(x: Outer<T>, f: Fn1<Outer<T>, T>) -> T {
            return f(x);
          }

          fn bump<T: Helper.Addable>(value: Outer<T>, extra: Outer<T>) -> T {
            return apply(value, |x: Outer<T>| -> T { return x.add(extra); });
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
        .expect("expected helper-trait alias-chain monomorphized higher-order helper");
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
        .expect(
            "expected helper-trait alias-chain monomorphized captured generic lambda specialization",
        );
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

#[test]
fn lowers_higher_order_lambda_returning_outer_literal_with_deferred_inner_inference() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          fn apply<T, R>(value: T, mapper: Fn1<T, R>) -> R {
            return mapper(value);
          }

          fn main() -> i64 {
            let outer: Outer<i64, String> =
              apply(7, |x: i64| -> Outer<i64, String> {
                return Outer {
                  inner: Phantom { value: x, tag: 1 },
                  meta: "ok",
                };
              });
            return outer.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    let hof = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_"))
        .expect("expected monomorphized higher-order helper");
    assert!(hof.generic_params.is_empty());
    assert!(matches!(
        hof.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Outer<i64, String>"
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized lambda");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Outer<i64, String>"
    ));
}

#[test]
fn lowers_higher_order_lambda_without_explicit_return_type_returning_outer_literal_with_deferred_inner_inference(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          fn apply<T, R>(value: T, mapper: Fn1<T, R>) -> R {
            return mapper(value);
          }

          fn main() -> i64 {
            let outer: Outer<i64, String> =
              apply(7, |x: i64| {
                return Outer {
                  inner: Phantom { value: x, tag: 1 },
                  meta: "ok",
                };
              });
            return outer.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    let hof = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_"))
        .expect("expected monomorphized higher-order helper");
    assert!(hof.generic_params.is_empty());
    assert!(matches!(
        hof.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Outer<i64, String>"
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized lambda");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Outer<i64, String>"
    ));
}

#[test]
fn lowers_higher_order_lambda_without_explicit_return_type_inside_if_result_context() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          fn apply<T, R>(value: T, mapper: Fn1<T, R>) -> R {
            return mapper(value);
          }

          fn main() -> i64 {
            let outer: Outer<i64, String> = if true {
              apply(7, |x: i64| {
                return Outer {
                  inner: Phantom { value: x, tag: 1 },
                  meta: "ok",
                };
              })
            } else {
              apply(8, |x: i64| {
                return Outer {
                  inner: Phantom { value: x, tag: 2 },
                  meta: "fallback",
                };
              })
            };
            return outer.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    let lambda_count = module
        .functions
        .iter()
        .filter(|function| function.name.starts_with("__lambda_main_"))
        .count();
    assert_eq!(
        lambda_count, 2,
        "expected one synthesized lambda per if branch"
    );

    let outer_helpers = module
        .functions
        .iter()
        .filter(|function| {
            function.name.starts_with("__hof_apply_")
                && matches!(
                    function.return_type.as_ref().map(|ty| ty.render()),
                    Some(rendered) if rendered == "Outer<i64, String>"
                )
        })
        .count();
    assert_eq!(
        outer_helpers, 2,
        "expected both if branches to specialize apply to Outer<i64, String>"
    );
}

#[test]
fn lowers_higher_order_lambda_without_explicit_return_type_inside_match_result_context() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          fn apply<T, R>(value: T, mapper: Fn1<T, R>) -> R {
            return mapper(value);
          }

          fn main() -> i64 {
            let outer: Outer<i64, String> = match 1 {
              1 => {
                apply(7, |x: i64| {
                  return Outer {
                    inner: Phantom { value: x, tag: 1 },
                    meta: "ok",
                  };
                })
              },
              _ => {
                apply(8, |x: i64| {
                  return Outer {
                    inner: Phantom { value: x, tag: 2 },
                    meta: "fallback",
                  };
                })
              }
            };
            return outer.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    let lambda_count = module
        .functions
        .iter()
        .filter(|function| function.name.starts_with("__lambda_main_"))
        .count();
    assert_eq!(
        lambda_count, 2,
        "expected one synthesized lambda per match arm"
    );

    let outer_helpers = module
        .functions
        .iter()
        .filter(|function| {
            function.name.starts_with("__hof_apply_")
                && matches!(
                    function.return_type.as_ref().map(|ty| ty.render()),
                    Some(rendered) if rendered == "Outer<i64, String>"
                )
        })
        .count();
    assert_eq!(
        outer_helpers, 2,
        "expected both match arms to specialize apply to Outer<i64, String>"
    );
}

#[test]
fn lowers_higher_order_lambda_without_explicit_return_type_with_tail_match_expr() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          fn apply<T, R>(value: T, mapper: Fn1<T, R>) -> R {
            return mapper(value);
          }

          fn main() -> i64 {
            let outer: Outer<i64, String> = apply(7, |x: i64| {
              match x {
                7 => {
                  Outer {
                    inner: Phantom { value: x, tag: 1 },
                    meta: "ok",
                  }
                },
                _ => {
                  Outer {
                    inner: Phantom { value: x, tag: 2 },
                    meta: "fallback",
                  }
                }
              }
            });
            return outer.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    let hof = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_"))
        .expect("expected monomorphized higher-order helper");
    assert!(hof.generic_params.is_empty());
    assert!(matches!(
        hof.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Outer<i64, String>"
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized lambda");
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Outer<i64, String>"
    ));
}
