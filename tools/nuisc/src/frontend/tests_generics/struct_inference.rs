use super::*;

#[test]
fn monomorphizes_generic_function_from_inferred_struct_literal_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          fn unwrap_box<T>(boxed: Boxed<T>) -> T {
            return boxed.value;
          }

          fn main() -> i64 {
            return unwrap_box(Boxed { value: 7 });
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "unwrap_box__i64"
    ));
}

#[test]
fn monomorphizes_generic_function_from_inferred_alias_struct_literal_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type BoxAlias<T> = Boxed<T>;

          struct Boxed<T> {
            value: T,
          }

          fn unwrap_box<T>(boxed: Boxed<T>) -> T {
            return boxed.value;
          }

          fn main() -> i64 {
            return unwrap_box(BoxAlias { value: 7 });
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "unwrap_box__i64"
    ));
}

#[test]
fn monomorphizes_generic_function_from_inferred_non_transparent_alias_struct_literal_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type WrappedStructAlias<T> = Wrapper<Boxed<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Wrapper<T> {
            inner: T,
            tag: i64,
          }

          fn unwrap_wrapped<T>(wrapped: Wrapper<Boxed<T>>) -> T {
            return wrapped.inner.value;
          }

          fn main() -> i64 {
            return unwrap_wrapped(WrappedStructAlias {
              inner: Boxed { value: 7 },
              tag: 1,
            });
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "unwrap_wrapped__i64"
    ));
}

#[test]
fn monomorphizes_generic_function_from_outer_struct_literal_with_deferred_inner_inference() {
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

          fn unwrap_outer<T, U>(outer: Outer<T, U>) -> T {
            return outer.inner.value;
          }

          fn main() -> i64 {
            return unwrap_outer(Outer {
              inner: Phantom { value: 7, tag: 1 },
              meta: "ok",
            });
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "unwrap_outer__i64__String"
    ));
}

#[test]
fn monomorphizes_generic_function_from_outer_struct_literal_with_deferred_inner_payload_inference()
{
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Just<T, U> {
            value: T,
          }

          struct Outer<T, U> {
            inner: Just<T, U>,
            meta: U,
          }

          fn unwrap_outer<T, U>(outer: Outer<T, U>) -> T {
            return outer.inner.value;
          }

          fn main() -> i64 {
            return unwrap_outer(Outer {
              inner: Just(7),
              meta: "ok",
            });
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "unwrap_outer__i64__String"
    ));
}

#[test]
fn monomorphizes_generic_function_from_inferred_payload_constructor_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Just<T> {
            value: T,
          }

          fn unwrap_just<T>(value: Just<T>) -> T {
            return value.value;
          }

          fn main() -> i64 {
            return unwrap_just(Just(7));
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "unwrap_just__i64"
    ));
}

#[test]
fn monomorphizes_generic_function_from_inferred_alias_payload_constructor_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;

          struct Just<T> {
            value: T,
          }

          fn unwrap_just<T>(value: Just<T>) -> T {
            return value.value;
          }

          fn main() -> i64 {
            return unwrap_just(JustAlias(7));
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "unwrap_just__i64"
    ));
}

#[test]
fn monomorphizes_generic_function_from_transparent_alias_outer_literal_with_deferred_inner_inference(
) {
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

          fn unwrap_outer<T, U>(outer: Outer<T, U>) -> T {
            return outer.inner.value;
          }

          fn main() -> i64 {
            return unwrap_outer(OuterAlias {
              inner: Phantom { value: 7, tag: 1 },
              meta: "ok",
            });
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "unwrap_outer__i64__String"
    ));
}

#[test]
fn monomorphizes_generic_function_from_non_transparent_alias_outer_literal_with_deferred_inner_inference(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type OuterAlias<T, U> = Wrapper<Outer<T, U>>;

          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          struct Wrapper<T> {
            inner: T,
            mark: i64,
          }

          fn unwrap_outer<T, U>(wrapped: Wrapper<Outer<T, U>>) -> T {
            return wrapped.inner.inner.value;
          }

          fn main() -> i64 {
            return unwrap_outer(OuterAlias {
              inner: Outer {
                inner: Phantom { value: 7, tag: 1 },
                meta: "ok",
              },
              mark: 1,
            });
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "unwrap_outer__i64__String"
    ));
}
