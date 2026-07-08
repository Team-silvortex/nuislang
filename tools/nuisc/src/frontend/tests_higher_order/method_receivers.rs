use super::*;

#[test]
fn lowers_higher_order_lambda_returning_alias_outer_literal_with_deferred_inner_inference() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type OuterAlias<T, U> = Outer<T, U>;

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
            let outer: OuterAlias<i64, String> =
              apply(7, |x: i64| -> OuterAlias<i64, String> {
                return OuterAlias {
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
fn lowers_method_call_lambda_without_explicit_return_type_returning_outer_literal() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Runner {
            fn apply(host: Self, value: i64, mapper: Fn1<i64, Outer<i64, String>>) -> Outer<i64, String>;
          }

          struct Host {}

          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          impl Runner for Host {
            fn apply(host: Host, value: i64, mapper: Fn1<i64, Outer<i64, String>>) -> Outer<i64, String> {
              return mapper(value);
            }
          }

          fn main() -> i64 {
            let host: Host = Host {};
            let outer: Outer<i64, String> = host.apply(7, |x: i64| {
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

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized method-call lambda");
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Outer<i64, String>"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function
                .name
                .starts_with("__hof_impl_Runner_for_Host_apply")
        })
        .expect("expected specialized higher-order impl method helper");
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Outer<i64, String>"
    ));
}

#[test]
fn lowers_generic_impl_method_call_lambda_for_concrete_receiver() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Runner {
            fn apply(host: Self, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64>;
          }

          struct Box<T> {
            value: T,
          }

          struct Pair<T> {
            left: T,
            right: T,
          }

          impl<T> Runner for Box<T> {
            fn apply(host: Box<T>, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64> {
              return mapper(value);
            }
          }

          fn main() -> i64 {
            let host: Box<i64> = Box { value: 3 };
            let pair: Pair<i64> = host.apply(7, |x: i64| {
              return Pair { left: x, right: x + 1 };
            });
            return pair.right;
          }
        }
        "#,
    )
    .unwrap();

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized generic method-call lambda");
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Pair<i64>"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_impl_Runner_for_Box")
                && function.name.contains("apply")
        })
        .expect("expected specialized higher-order generic impl method helper");
    assert!(specialized.generic_params.is_empty());
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Pair<i64>"
    ));
}

#[test]
fn lowers_method_call_lambda_when_receiver_comes_from_if_expr() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Runner {
            fn apply(host: Self, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64>;
          }

          struct Host {}

          struct Pair<T> {
            left: T,
            right: T,
          }

          impl Runner for Host {
            fn apply(host: Host, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64> {
              return mapper(value);
            }
          }

          fn main() -> i64 {
            let pick_left = true;
            let host = if pick_left {
              Host {}
            } else {
              Host {}
            };
            let pair = host.apply(7, |x: i64| {
              return Pair { left: x, right: x + 1 };
            });
            return pair.right;
          }
        }
        "#,
    )
    .unwrap();

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function
                .name
                .starts_with("__hof_impl_Runner_for_Host_apply")
        })
        .expect("expected specialized higher-order impl helper for if receiver");
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Pair<i64>"
    ));
}

#[test]
fn lowers_method_call_lambda_when_receiver_comes_from_match_expr() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Runner {
            fn apply(host: Self, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64>;
          }

          struct Host {}

          struct Pair<T> {
            left: T,
            right: T,
          }

          impl Runner for Host {
            fn apply(host: Host, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64> {
              return mapper(value);
            }
          }

          fn main() -> i64 {
            let choice: i64 = 1;
            let host = match choice {
              1 => {
                Host {}
              }
              _ => {
                Host {}
              }
            };
            let pair = host.apply(7, |x: i64| {
              return Pair { left: x, right: x + 1 };
            });
            return pair.right;
          }
        }
        "#,
    )
    .unwrap();

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function
                .name
                .starts_with("__hof_impl_Runner_for_Host_apply")
        })
        .expect("expected specialized higher-order impl helper for match receiver");
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Pair<i64>"
    ));
}

#[test]
fn lowers_method_call_lambda_when_receiver_comes_from_method_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Runner {
            fn id(host: Self) -> Self;
            fn apply(host: Self, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64>;
          }

          struct Host {}

          struct Pair<T> {
            left: T,
            right: T,
          }

          impl Runner for Host {
            fn id(host: Host) -> Host {
              return host;
            }

            fn apply(host: Host, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64> {
              return mapper(value);
            }
          }

          fn main() -> i64 {
            let host: Host = Host {};
            let pair = host.id().apply(7, |x: i64| {
              return Pair { left: x, right: x + 1 };
            });
            return pair.right;
          }
        }
        "#,
    )
    .unwrap();

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function
                .name
                .starts_with("__hof_impl_Runner_for_Host_apply")
        })
        .expect("expected specialized higher-order impl helper for chained receiver");
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Pair<i64>"
    ));
}

#[test]
fn lowers_method_call_lambda_when_receiver_comes_from_struct_field() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Runner {
            fn apply(host: Self, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64>;
          }

          struct Host {}

          struct State {
            host: Host,
          }

          struct Pair<T> {
            left: T,
            right: T,
          }

          impl Runner for Host {
            fn apply(host: Host, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64> {
              return mapper(value);
            }
          }

          fn main() -> i64 {
            let state = State { host: Host {} };
            let pair = state.host.apply(7, |x: i64| {
              return Pair { left: x, right: x + 1 };
            });
            return pair.right;
          }
        }
        "#,
    )
    .unwrap();

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function
                .name
                .starts_with("__hof_impl_Runner_for_Host_apply")
        })
        .expect("expected specialized higher-order impl helper for field receiver");
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Pair<i64>"
    ));
}

#[test]
fn lowers_method_call_lambda_when_typed_receiver_comes_from_struct_field() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Runner {
            fn apply(host: Self, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64>;
          }

          struct Host {}

          struct State {
            host: Host,
          }

          struct Pair<T> {
            left: T,
            right: T,
          }

          impl Runner for Host {
            fn apply(host: Host, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64> {
              return mapper(value);
            }
          }

          fn main() -> i64 {
            let state: State = State { host: Host {} };
            let pair = state.host.apply(7, |x: i64| {
              return Pair { left: x, right: x + 1 };
            });
            return pair.right;
          }
        }
        "#,
    )
    .unwrap();

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function
                .name
                .starts_with("__hof_impl_Runner_for_Host_apply")
        })
        .expect("expected specialized higher-order impl helper for typed field receiver");
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Pair<i64>"
    ));
}

#[test]
fn lowers_method_call_lambda_when_receiver_comes_from_nested_struct_field() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Runner {
            fn apply(host: Self, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64>;
          }

          struct Host {}

          struct Inner {
            host: Host,
          }

          struct State {
            inner: Inner,
          }

          struct Pair<T> {
            left: T,
            right: T,
          }

          impl Runner for Host {
            fn apply(host: Host, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64> {
              return mapper(value);
            }
          }

          fn main() -> i64 {
            let state: State = State { inner: Inner { host: Host {} } };
            let pair = state.inner.host.apply(7, |x: i64| {
              return Pair { left: x, right: x + 1 };
            });
            return pair.right;
          }
        }
        "#,
    )
    .unwrap();

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function
                .name
                .starts_with("__hof_impl_Runner_for_Host_apply")
        })
        .expect("expected specialized higher-order impl helper for nested field receiver");
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Pair<i64>"
    ));
}
