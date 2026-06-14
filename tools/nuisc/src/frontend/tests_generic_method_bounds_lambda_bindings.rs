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

#[test]
fn rejects_capturing_generic_lambda_method_call_without_required_bound() {
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

          fn apply<T>(value: T, f: Fn1<T, T>) -> T {
            return f(value);
          }

          fn bump<T>(value: T, extra: T) -> T {
            return apply(value, |x: T| -> T { return x.add(extra); });
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
fn rejects_capturing_generic_lambda_explicit_trait_call_without_required_bound() {
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

          fn apply<T>(value: T, f: Fn1<T, T>) -> T {
            return f(value);
          }

          fn bump<T>(value: T, extra: T) -> T {
            return apply(value, |x: T| -> T { return Addable.add(x, extra); });
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
            "function `bump` body lambda body calls trait method `Addable.add` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn rejects_capturing_generic_lambda_operator_call_without_required_bound() {
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

          fn apply<T>(value: T, f: Fn1<T, T>) -> T {
            return f(value);
          }

          fn bump<T>(value: T, extra: T) -> T {
            return apply(value, |x: T| -> T { return x + extra; });
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
            "function `bump` body lambda body calls operator `+` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn rejects_capturing_generic_lambda_equality_operator_without_required_bound() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Equatable {
            fn eq(lhs: Self, rhs: Self) -> bool;
          }

          impl Equatable for i64 {
            fn eq(lhs: i64, rhs: i64) -> bool {
              return lhs == rhs;
            }
          }

          fn apply<T>(value: T, f: Fn1<T, bool>) -> bool {
            return f(value);
          }

          fn same<T>(value: T, other: T) -> bool {
            return apply(value, |x: T| -> bool { return x == other; });
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
            "function `same` body lambda body calls operator `==` on generic parameter `T` without required bound `Equatable`"
        ),
        "{error}"
    );
}

#[test]
fn rejects_capturing_generic_lambda_equality_operator_with_mismatched_bound() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Showable {
            fn show(value: Self) -> i64;
          }

          trait Equatable {
            fn eq(lhs: Self, rhs: Self) -> bool;
          }

          impl Showable for i64 {
            fn show(value: i64) -> i64 {
              return value;
            }
          }

          impl Equatable for i64 {
            fn eq(lhs: i64, rhs: i64) -> bool {
              return lhs == rhs;
            }
          }

          fn apply<T: Showable>(value: T, f: Fn1<T, bool>) -> bool {
            return f(value);
          }

          fn same<T: Showable>(value: T, other: T) -> bool {
            return apply(value, |x: T| -> bool { return x == other; });
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
            "function `same` body lambda body calls operator `==` on generic parameter `T` but bound `Showable` does not satisfy required trait `Equatable`"
        ),
        "{error}"
    );
}

#[test]
fn rejects_capturing_generic_lambda_unary_neg_without_required_bound() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Negatable {
            fn neg(value: Self) -> Self;
          }

          impl Negatable for i64 {
            fn neg(value: i64) -> i64 {
              return 0 - value;
            }
          }

          fn apply<T>(value: T, f: Fn1<T, T>) -> T {
            return f(value);
          }

          fn flip<T>(value: T) -> T {
            return apply(value, |x: T| -> T { return -x; });
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
            "function `flip` body lambda body calls operator `-` on generic parameter `T` without required bound `Negatable`"
        ),
        "{error}"
    );
}

#[test]
fn rejects_capturing_generic_lambda_alias_payload_equality_operator_with_mismatched_bound() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Showable {
            fn show(value: Self) -> i64;
          }

          trait Equatable {
            fn eq(lhs: Self, rhs: Self) -> bool;
          }

          impl Showable for i64 {
            fn show(value: i64) -> i64 {
              return value;
            }
          }

          impl Equatable for i64 {
            fn eq(lhs: i64, rhs: i64) -> bool {
              return lhs == rhs;
            }
          }

          type Alias<T> = T;
          type Outer<T> = Alias<T>;

          fn apply<T: Showable>(value: Outer<T>, f: Fn1<Outer<T>, bool>) -> bool {
            return f(value);
          }

          fn same<T: Showable>(value: Outer<T>, other: Outer<T>) -> bool {
            return apply(value, |x: Outer<T>| -> bool { return x == other; });
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
            "function `same` body lambda body via type alias `Alias` target via type alias `Outer` target"
        ),
        "{error}"
    );
    assert!(
        error.contains(
            "calls operator `==` on generic parameter `T` but bound `Showable` does not satisfy required trait `Equatable`"
        ),
        "{error}"
    );
}
