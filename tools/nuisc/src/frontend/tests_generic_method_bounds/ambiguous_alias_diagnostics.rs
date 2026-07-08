use super::*;

#[test]
fn reports_ambiguous_candidate_bounds_for_unbounded_generic_method_call() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          trait Mergeable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl Mergeable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn bump<T>(value: T) -> T {
            return value.add(value);
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
            "function `bump` body calls method `add` on generic parameter `T` without a trait bound; candidate bounds: Addable, Mergeable"
        ),
        "{error}"
    );
}

#[test]
fn reports_ambiguous_candidate_bounds_for_wrong_generic_method_bound() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          trait Mergeable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          trait Showable {
            fn show(value: Self) -> i64;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl Mergeable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl Showable for i64 {
            fn show(value: i64) -> i64 {
              return value;
            }
          }

          fn bump<T: Showable>(value: T) -> T {
            return value.add(value);
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
            "function `bump` body calls method `add` on generic parameter `T` but bound `Showable` does not define that method; candidate bounds: Addable, Mergeable"
        ),
        "{error}"
    );
}

#[test]
fn reports_alias_chain_context_for_missing_generic_method_bound() {
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

          type Alias<T> = T;
          type Outer<T> = Alias<T>;

          fn bump<T>(value: Outer<T>) -> T {
            return value.add(value);
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
            "function `bump` body via type alias `Alias` target via type alias `Outer` target"
        ),
        "{error}"
    );
    assert!(
        error.contains(
            "calls method `add` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn reports_alias_chain_context_for_wrong_generic_method_bound() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          trait Showable {
            fn show(value: Self) -> i64;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl Showable for i64 {
            fn show(value: i64) -> i64 {
              return value;
            }
          }

          type Alias<T> = T;
          type Outer<T> = Alias<T>;

          fn bump<T: Showable>(value: Outer<T>) -> T {
            return value.add(value);
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
            "function `bump` body via type alias `Alias` target via type alias `Outer` target"
        ),
        "{error}"
    );
    assert!(
        error.contains(
            "calls method `add` on generic parameter `T` but bound `Showable` does not define that method; consider bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn reports_nested_alias_chain_context_for_ambiguous_unbounded_method_call() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          trait Mergeable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl Mergeable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          type Inner<T> = T;
          type Alias<T> = Inner<T>;
          type Outer<T> = Alias<T>;

          fn bump<T>(value: Outer<T>) -> T {
            return value.add(value);
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
            "function `bump` body via type alias `Inner` target via type alias `Alias` target via type alias `Outer` target"
        ),
        "{error}"
    );
    assert!(
        error.contains(
            "calls method `add` on generic parameter `T` without a trait bound; candidate bounds: Addable, Mergeable"
        ),
        "{error}"
    );
}

#[test]
fn reports_nested_alias_chain_context_for_ambiguous_wrong_bound_method_call() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          trait Mergeable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          trait Showable {
            fn show(value: Self) -> i64;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl Mergeable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl Showable for i64 {
            fn show(value: i64) -> i64 {
              return value;
            }
          }

          type Inner<T> = T;
          type Alias<T> = Inner<T>;
          type Outer<T> = Alias<T>;

          fn bump<T: Showable>(value: Outer<T>) -> T {
            return value.add(value);
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
            "function `bump` body via type alias `Inner` target via type alias `Alias` target via type alias `Outer` target"
        ),
        "{error}"
    );
    assert!(
        error.contains(
            "calls method `add` on generic parameter `T` but bound `Showable` does not define that method; candidate bounds: Addable, Mergeable"
        ),
        "{error}"
    );
}
