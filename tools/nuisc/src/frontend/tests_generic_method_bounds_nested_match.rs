use super::parse_nuis_module;

#[test]
fn accepts_outer_match_binding_used_inside_nested_match_when_bound_is_present() {
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

          type Alias<T> = T;
          type Outer<T> = Alias<T>;

          fn bump<T: Addable>(value: Outer<T>) -> T {
            match value {
              local => {
                match 0 {
                  0 => {
                    return local.add(local);
                  }
                  _ => {
                    return value;
                  }
                }
              }
              _ => {
                return value;
              }
            }
          }

          fn main() -> i64 {
            return bump(4);
          }
        }
        "#,
    )
    .unwrap();

    assert!(module
        .functions
        .iter()
        .any(|function| function.name.contains("bump")));
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "main"));
}

#[test]
fn accepts_outer_match_binding_used_inside_guarded_nested_match_when_bound_is_present() {
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

          type Alias<T> = T;
          type Outer<T> = Alias<T>;

          fn bump<T: Addable>(value: Outer<T>) -> T {
            match value {
              local => {
                match 0 {
                  0 if true => {
                    return local.add(local);
                  }
                  _ => {
                    return value;
                  }
                }
              }
              _ => {
                return value;
              }
            }
          }

          fn main() -> i64 {
            return bump(4);
          }
        }
        "#,
    )
    .unwrap();

    assert!(module
        .functions
        .iter()
        .any(|function| function.name.contains("bump")));
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "main"));
}

#[test]
fn reports_outer_match_binding_used_inside_nested_match_for_missing_generic_method_bound() {
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
            match value {
              local => {
                match 0 {
                  0 => {
                    return local.add(local);
                  }
                  _ => {
                    return value;
                  }
                }
              }
              _ => {
                return value;
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
fn reports_outer_match_binding_used_inside_nested_match_for_ambiguous_wrong_bound_method_call() {
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

          type Alias<T> = T;
          type Outer<T> = Alias<T>;

          fn bump<T: Showable>(value: Outer<T>) -> T {
            match value {
              local => {
                match 0 {
                  0 => {
                    return local.add(local);
                  }
                  _ => {
                    return value;
                  }
                }
              }
              _ => {
                return value;
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
            "function `bump` body via type alias `Alias` target via type alias `Outer` target"
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
