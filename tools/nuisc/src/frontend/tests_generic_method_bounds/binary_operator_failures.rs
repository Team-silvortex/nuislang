use super::*;

#[test]
fn reports_missing_addable_bound_for_generic_binary_add() {
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

          fn bump<T>(lhs: T, rhs: T) -> T {
            return lhs + rhs;
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
            "calls operator `+` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn reports_alias_chain_context_for_missing_generic_binary_add_bound() {
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

          fn bump<T>(lhs: Outer<T>, rhs: Outer<T>) -> T {
            return lhs + rhs;
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
            "calls operator `+` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn reports_missing_subtractable_bound_for_generic_binary_sub() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Subtractable {
            fn sub(lhs: Self, rhs: Self) -> Self;
          }

          impl Subtractable for i64 {
            fn sub(lhs: i64, rhs: i64) -> i64 {
              return lhs - rhs;
            }
          }

          fn bump<T>(lhs: T, rhs: T) -> T {
            return lhs - rhs;
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
            "calls operator `-` on generic parameter `T` without required bound `Subtractable`"
        ),
        "{error}"
    );
}

#[test]
fn reports_missing_multipliable_bound_for_generic_binary_mul() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Multipliable {
            fn mul(lhs: Self, rhs: Self) -> Self;
          }

          impl Multipliable for i64 {
            fn mul(lhs: i64, rhs: i64) -> i64 {
              return lhs * rhs;
            }
          }

          fn bump<T>(lhs: T, rhs: T) -> T {
            return lhs * rhs;
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
            "calls operator `*` on generic parameter `T` without required bound `Multipliable`"
        ),
        "{error}"
    );
}

#[test]
fn reports_missing_dividable_bound_for_generic_binary_div() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Dividable {
            fn div(lhs: Self, rhs: Self) -> Self;
          }

          impl Dividable for i64 {
            fn div(lhs: i64, rhs: i64) -> i64 {
              return lhs / rhs;
            }
          }

          fn bump<T>(lhs: T, rhs: T) -> T {
            return lhs / rhs;
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
            "calls operator `/` on generic parameter `T` without required bound `Dividable`"
        ),
        "{error}"
    );
}

#[test]
fn reports_missing_remainderable_bound_for_generic_binary_rem() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Remainderable {
            fn rem(lhs: Self, rhs: Self) -> Self;
          }

          impl Remainderable for i64 {
            fn rem(lhs: i64, rhs: i64) -> i64 {
              return lhs % rhs;
            }
          }

          fn bump<T>(lhs: T, rhs: T) -> T {
            return lhs % rhs;
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
            "calls operator `%` on generic parameter `T` without required bound `Remainderable`"
        ),
        "{error}"
    );
}

#[test]
fn reports_missing_equatable_bound_for_generic_binary_eq() {
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

          fn same<T>(lhs: T, rhs: T) -> bool {
            return lhs == rhs;
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
            "calls operator `==` on generic parameter `T` without required bound `Equatable`"
        ),
        "{error}"
    );
}

#[test]
fn reports_mismatched_bound_for_generic_binary_eq() {
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

          fn same<T: Showable>(lhs: T, rhs: T) -> bool {
            return lhs == rhs;
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
            "calls operator `==` on generic parameter `T` but bound `Showable` does not satisfy required trait `Equatable`"
        ),
        "{error}"
    );
}

#[test]
fn reports_alias_chain_context_for_mismatched_generic_binary_eq_bound() {
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

          fn same<T: Showable>(lhs: Outer<T>, rhs: Outer<T>) -> bool {
            return lhs == rhs;
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
            "function `same` body via type alias `Alias` target via type alias `Outer` target"
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
