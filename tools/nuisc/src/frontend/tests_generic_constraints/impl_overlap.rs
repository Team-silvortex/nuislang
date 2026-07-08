use super::*;

#[test]
fn rejects_overlapping_generic_impl_and_concrete_impl() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Showable {
            fn show(value: Self) -> i64;
          }

          enum Option<T> {
            None,
            Some(T),
          }

          impl<T: Showable> Showable for Option<T> where T: Showable {
            fn show(value: Option<T>) -> i64 {
              return 0;
            }
          }

          impl Showable for Option<i64> {
            fn show(value: Option<i64>) -> i64 {
              return 1;
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
        error.contains("overlapping impls for trait `Showable`"),
        "{error}"
    );
    assert!(error.contains("Option<T>"), "{error}");
    assert!(error.contains("Option<i64>"), "{error}");
}

#[test]
fn rejects_alpha_equivalent_generic_impl_duplicates() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Showable {
            fn show(value: Self) -> i64;
          }

          enum Option<T> {
            None,
            Some(T),
          }

          impl<T: Showable> Showable for Option<T> where T: Showable {
            fn show(value: Option<T>) -> i64 {
              return 0;
            }
          }

          impl<U: Showable> Showable for Option<U> where U: Showable {
            fn show(value: Option<U>) -> i64 {
              return 1;
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
        error.contains("overlapping impls for trait `Showable`"),
        "{error}"
    );
}
