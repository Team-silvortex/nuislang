use super::*;

#[test]
fn reports_missing_equatable_bound_for_generic_binary_ne() {
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

          fn different<T>(lhs: T, rhs: T) -> bool {
            return lhs != rhs;
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
            "calls operator `!=` on generic parameter `T` without required bound `Equatable`"
        ),
        "{error}"
    );
}

#[test]
fn reports_missing_orderable_bound_for_generic_binary_ordering() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Orderable {
            fn lt(lhs: Self, rhs: Self) -> bool;
            fn le(lhs: Self, rhs: Self) -> bool;
            fn gt(lhs: Self, rhs: Self) -> bool;
            fn ge(lhs: Self, rhs: Self) -> bool;
          }

          impl Orderable for i64 {
            fn lt(lhs: i64, rhs: i64) -> bool {
              return lhs < rhs;
            }

            fn le(lhs: i64, rhs: i64) -> bool {
              return lhs <= rhs;
            }

            fn gt(lhs: i64, rhs: i64) -> bool {
              return lhs > rhs;
            }

            fn ge(lhs: i64, rhs: i64) -> bool {
              return lhs >= rhs;
            }
          }

          fn less<T>(lhs: T, rhs: T) -> bool {
            return lhs < rhs;
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
            "calls operator `<` on generic parameter `T` without required bound `Orderable`"
        ),
        "{error}"
    );
}

#[test]
fn reports_missing_notable_bound_for_generic_unary_not() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Notable {
            fn not(value: Self) -> bool;
          }

          impl Notable for bool {
            fn not(value: bool) -> bool {
              return !value;
            }
          }

          fn invert<T>(value: T) -> bool {
            return !value;
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
            "calls operator `!` on generic parameter `T` without required bound `Notable`"
        ),
        "{error}"
    );
}

#[test]
fn reports_mismatched_bound_for_generic_unary_not() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Showable {
            fn show(value: Self) -> i64;
          }

          trait Notable {
            fn not(value: Self) -> bool;
          }

          impl Showable for bool {
            fn show(value: bool) -> i64 {
              if value {
                return 1;
              }
              return 0;
            }
          }

          impl Notable for bool {
            fn not(value: bool) -> bool {
              return !value;
            }
          }

          fn invert<T: Showable>(value: T) -> bool {
            return !value;
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
            "calls operator `!` on generic parameter `T` but bound `Showable` does not satisfy required trait `Notable`"
        ),
        "{error}"
    );
}

#[test]
fn reports_missing_negatable_bound_for_generic_unary_neg() {
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

          fn invert<T>(value: T) -> T {
            return -value;
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
            "calls operator `-` on generic parameter `T` without required bound `Negatable`"
        ),
        "{error}"
    );
}
