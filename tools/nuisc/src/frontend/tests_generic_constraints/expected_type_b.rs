use super::*;

#[test]
fn reports_nested_alias_struct_enum_expected_type_driven_generic_bound_failure_inside_if_branch() {
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
            let value: Option<Boxed<Alias<Text>>> = if true {
              Option.Some(Boxed { value: typed_zero() })
            } else {
              Option.None
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
            "function `main` body local `value` via type alias `Alias` generic parameter `T`"
        ),
        "{error}"
    );
}

#[test]
fn reports_nested_struct_enum_expected_type_driven_generic_bound_failure_without_alias_at_call_site(
) {
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
            let value: Option<Boxed<Text>> = Option.Some(Boxed { value: typed_zero() });
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
fn reports_nested_struct_enum_expected_type_driven_generic_bound_failure_without_alias_inside_if_branch_at_call_site(
) {
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
            let value: Option<Boxed<Text>> = if true {
              Option.Some(Boxed { value: typed_zero() })
            } else {
              Option.None
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
            "function `main` body local `value` if-then call `typed_zero` generic parameter `U`"
        ),
        "{error}"
    );
}

#[test]
fn reports_return_expected_type_driven_generic_bound_failure_without_alias_at_call_site() {
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

          fn build() -> Option<Boxed<Text>> {
            return Option.Some(Boxed { value: typed_zero() });
          }

          fn main() -> i64 {
            let _value = build();
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
        error.contains("function `build` body call `typed_zero` generic parameter `U`"),
        "{error}"
    );
}

#[test]
fn reports_return_expected_type_driven_generic_bound_failure_with_outer_alias_precedence() {
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

          fn build() -> Option<Boxed<Alias<Text>>> {
            return Option.Some(Boxed { value: typed_zero() });
          }

          fn main() -> i64 {
            let _value = build();
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
        error.contains("function `build` return type via type alias `Alias` generic parameter `T`"),
        "{error}"
    );
}

#[test]
fn reports_error_style_result_payload_expected_type_driven_generic_bound_failure_at_call_site() {
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

          struct Boxed<T> {
            value: T,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn typed_zero<U: Addable>() -> U {
            return 0;
          }

          fn build() -> Result<Boxed<Text>, i64> {
            return Result.Ok(Boxed { value: typed_zero() });
          }

          fn main() -> i64 {
            let _value = build();
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
        error.contains("function `build` body call `typed_zero` generic parameter `U`"),
        "{error}"
    );
}

#[test]
fn reports_error_style_result_payload_expected_type_driven_generic_bound_failure_with_alias_precedence(
) {
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

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn typed_zero<U: Addable>() -> U {
            return 0;
          }

          fn build() -> Result<Boxed<Alias<Text>>, i64> {
            return Result.Ok(Boxed { value: typed_zero() });
          }

          fn main() -> i64 {
            let _value = build();
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
        error.contains("function `build` return type via type alias `Alias` generic parameter `T`"),
        "{error}"
    );
}
