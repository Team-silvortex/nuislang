use super::*;

#[test]
fn reports_expected_type_driven_generic_bound_failure_at_local_use_site() {
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

          fn typed_zero<T: Addable>() -> T {
            return 0;
          }

          fn main() -> i64 {
            let value: Text = typed_zero();
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `Text` does not satisfy bound `Addable`"),
        "{error}"
    );
    assert!(
        error
            .contains("function `main` body local `value` call `typed_zero` generic parameter `T`"),
        "{error}"
    );
}

#[test]
fn reports_expected_type_driven_generic_bound_failure_inside_if_result_branch() {
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

          fn typed_zero<T: Addable>() -> T {
            return 0;
          }

          fn main() -> i64 {
            let value: Text = if true {
              typed_zero()
            } else {
              "ok"
            };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `Text` does not satisfy bound `Addable`"),
        "{error}"
    );
    assert!(
        error.contains(
            "function `main` body local `value` if-then call `typed_zero` generic parameter `T`"
        ),
        "{error}"
    );
}

#[test]
fn reports_expected_type_driven_generic_bound_failure_inside_lambda_body() {
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

          fn apply<T>(value: T, mapper: Fn1<T, T>) -> T {
            return mapper(value);
          }

          fn typed_zero<U: Addable>() -> U {
            return 0;
          }

          fn main() -> i64 {
            let value = apply("ok", |_x: Text| -> Text {
              return typed_zero();
            });
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `Text` does not satisfy bound `Addable`"),
        "{error}"
    );
    assert!(
        error.contains("function `main` body lambda body call `typed_zero` generic parameter `U`"),
        "{error}"
    );
}

#[test]
fn reports_alias_expected_type_driven_generic_bound_failure_at_local_use_site() {
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

          type Alias<T: Addable> = T;

          fn typed_zero<U: Addable>() -> U {
            return 0;
          }

          fn main() -> i64 {
            let value: Alias<Text> = typed_zero();
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `Text` does not satisfy bound `Addable`"),
        "{error}"
    );
    assert!(
        error.contains(
            "function `main` body local `value` via type alias `Alias` generic parameter `T`"
        ),
        "{error}"
    );
}

#[test]
fn reports_struct_field_expected_type_driven_generic_bound_failure() {
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

          struct Wrapper {
            value: Text,
          }

          fn typed_zero<T: Addable>() -> T {
            return 0;
          }

          fn main() -> i64 {
            let wrapped: Wrapper = Wrapper { value: typed_zero() };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `Text` does not satisfy bound `Addable`"),
        "{error}"
    );
    assert!(
        error.contains(
            "function `main` body local `wrapped` call `typed_zero` generic parameter `T`"
        ),
        "{error}"
    );
}

#[test]
fn reports_enum_payload_expected_type_driven_generic_bound_failure() {
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

          enum Option<T> {
            None,
            Some(T),
          }

          fn typed_zero<U: Addable>() -> U {
            return 0;
          }

          fn main() -> i64 {
            let value: Option<Text> = Option.Some(typed_zero());
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `Text` does not satisfy bound `Addable`"),
        "{error}"
    );
    assert!(
        error
            .contains("function `main` body local `value` call `typed_zero` generic parameter `U`"),
        "{error}"
    );
}

#[test]
fn reports_nested_alias_struct_enum_expected_type_driven_generic_bound_failure() {
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

          type Alias<T: Addable> = T;

          struct Boxed<T> {
            value: T,
          }

          enum Option<T> {
            None,
            Some(T),
          }

          fn typed_zero<U: Addable>() -> U {
            return 0;
          }

          fn main() -> i64 {
            let value: Option<Boxed<Alias<Text>>> = Option.Some(Boxed { value: typed_zero() });
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `Text` does not satisfy bound `Addable`"),
        "{error}"
    );
    assert!(
        error.contains(
            "function `main` body local `value` via type alias `Alias` generic parameter `T`"
        ),
        "{error}"
    );
}
