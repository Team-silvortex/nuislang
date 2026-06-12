use super::parse_nuis_module;

#[test]
fn rejects_generic_lambda_destructure_payload_method_call_without_required_bound() {
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

          fn wrap<T>(value: T) -> Boxed<T> {
            return Boxed<T> { value: value };
          }

          fn apply<T>(value: T, f: Fn1<T, T>) -> T {
            return f(value);
          }

          fn bump<T>(value: T) -> T {
            return apply(value, |x: T| -> T {
              let { value: payload } = wrap(x);
              return payload.add(x);
            });
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "function `bump` body lambda body calls method `add` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn rejects_generic_match_guard_method_call_on_payload_without_required_bound() {
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

          struct Just<T> {
            value: T,
          }

          fn bump<T>(value: Just<T>) -> T {
            match value {
              Just(payload) if payload.add(payload) == value.value => {
                return payload;
              }
              _ => {
                return value.value;
              }
            }
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "function `bump` body calls method `add` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}
