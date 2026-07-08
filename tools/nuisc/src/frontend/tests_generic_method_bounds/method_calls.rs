use super::*;

#[test]
fn reports_missing_generic_bound_for_method_call_on_type_param() {
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
            "function `bump` body calls method `add` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn accepts_method_call_on_type_param_with_where_clause_bound() {
    let module = parse_nuis_module(
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

          fn bump<T>(value: T) -> T where T: Addable {
            return value.add(value);
          }

          fn main() -> i64 {
            return bump(7);
          }
        }
        "#,
    )
    .unwrap();

    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "bump__i64"));
}

#[test]
fn accepts_method_call_on_type_param_with_repeated_where_predicates() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          trait Printable {
            fn print(value: Self) -> Text;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl Printable for i64 {
            fn print(value: i64) -> Text {
              return "ok";
            }
          }

          fn bump<T>(value: T) -> T where T: Printable, T: Addable {
            return value.add(value);
          }

          fn main() -> i64 {
            return bump(7);
          }
        }
        "#,
    )
    .unwrap();

    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "bump__i64"));
}

#[test]
fn reports_missing_generic_bound_for_method_call_on_call_inferred_local() {
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

          fn id<T>(value: T) -> T {
            return value;
          }

          fn bump<T>(value: T) -> T {
            let local = id(value);
            return local.add(value);
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
fn reports_missing_generic_bound_for_method_call_on_call_receiver() {
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

          fn id<T>(value: T) -> T {
            return value;
          }

          fn bump<T>(value: T) -> T {
            return id(value).add(value);
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
fn reports_wrong_generic_bound_for_method_call_on_type_param() {
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
            "function `bump` body calls method `add` on generic parameter `T` but bound `Showable` does not define that method; consider bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn accepts_alias_bound_for_method_call_on_type_param() {
    let module = parse_nuis_module(
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

          fn bump<T: Addable>(value: Alias<T>) -> T {
            return value.add(value);
          }

          fn main() -> i64 {
            return bump(7);
          }
        }
        "#,
    )
    .unwrap();

    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "bump__i64"));
}

#[test]
fn accepts_method_call_on_receiver_from_outer_literal_with_deferred_inner_inference() {
    let module = parse_nuis_module(
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

          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          fn value_of<T, U>(outer: Outer<T, U>) -> T {
            return outer.inner.value;
          }

          fn bump<T: Addable, U>(outer: Outer<T, U>) -> T {
            return value_of(outer).add(value_of(outer));
          }

          fn main() -> i64 {
            return bump(Outer {
              inner: Phantom { value: 7, tag: 1 },
              meta: "ok",
            });
          }
        }
        "#,
    )
    .unwrap();

    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "bump__i64__String"));
}
