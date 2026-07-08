use super::*;

#[test]
fn suggests_trait_method_name_for_explicit_trait_call_on_generic_param() {
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

          fn bump<T: Addable>(value: T) -> T {
            return Addable.ad(value, value);
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("does not define method `ad`"), "{error}");
    assert!(error.contains("did you mean `Addable.add`?"), "{error}");
}

#[test]
fn suggests_trait_method_name_for_receiver_method_call_on_generic_param() {
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

          fn bump<T: Addable>(value: T) -> T {
            return value.ad(value);
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("does not define that method"), "{error}");
    assert!(error.contains("did you mean `add`?"), "{error}");
}

#[test]
fn prefers_closest_trait_method_name_when_multiple_candidates_exist() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Mathy {
            fn sub(lhs: Self, rhs: Self) -> Self;
            fn sum(lhs: Self, rhs: Self) -> Self;
          }

          impl Mathy for i64 {
            fn sub(lhs: i64, rhs: i64) -> i64 {
              return lhs - rhs;
            }

            fn sum(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn bump<T: Mathy>(value: T) -> T {
            return value.sud(value);
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("does not define that method"), "{error}");
    assert!(error.contains("did you mean `sub`?"), "{error}");
}
