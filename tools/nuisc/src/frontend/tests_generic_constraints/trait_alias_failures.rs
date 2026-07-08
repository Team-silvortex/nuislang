use super::*;

#[test]
fn reports_trait_method_parameter_alias_bound_failure_with_chain_context() {
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

          type Inner<T: Addable> = T;
          type Outer<T: Addable> = Inner<T>;

          trait UsesAlias {
            fn keep(value: Outer<Text>) -> i64;
          }

          fn main() -> i64 {
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
        error.contains("trait `UsesAlias` method `keep` parameter `value`"),
        "{error}"
    );
    assert!(
        error.contains("type alias `Outer` generic parameter `T`"),
        "{error}"
    );
}

#[test]
fn reports_trait_method_return_alias_bound_failure_with_chain_context() {
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

          type Inner<T: Addable> = T;
          type Outer<T: Addable> = Inner<T>;

          trait UsesAlias {
            fn make() -> Outer<Text>;
          }

          fn main() -> i64 {
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
        error.contains("trait `UsesAlias` method `make` return type"),
        "{error}"
    );
    assert!(
        error.contains("type alias `Outer` generic parameter `T`"),
        "{error}"
    );
}

#[test]
fn reports_extern_interface_method_parameter_alias_bound_failure_with_chain_context() {
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

          type Inner<T: Addable> = T;
          type Outer<T: Addable> = Inner<T>;

          extern interface UsesAlias {
            fn keep(value: Outer<Text>) -> i64;
          }

          fn main() -> i64 {
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
        error.contains("extern interface `UsesAlias` method `keep` parameter `value`"),
        "{error}"
    );
    assert!(
        error.contains("type alias `Outer` generic parameter `T`"),
        "{error}"
    );
}

#[test]
fn reports_extern_interface_method_return_alias_bound_failure_with_chain_context() {
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

          type Inner<T: Addable> = T;
          type Outer<T: Addable> = Inner<T>;

          extern interface UsesAlias {
            fn make() -> Outer<Text>;
          }

          fn main() -> i64 {
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
        error.contains("extern interface `UsesAlias` method `make` return type"),
        "{error}"
    );
    assert!(
        error.contains("type alias `Outer` generic parameter `T`"),
        "{error}"
    );
}

#[test]
fn reports_impl_method_parameter_alias_bound_failure_with_chain_context() {
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

          trait UsesAlias {
            fn keep(value: i64) -> i64;
          }

          type Inner<T: Addable> = T;
          type Outer<T: Addable> = Inner<T>;

          impl UsesAlias for i64 {
            fn keep(value: Outer<Text>) -> i64 {
              return 0;
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
        error.contains("type `Text` does not satisfy bound `Addable`"),
        "{error}"
    );
    assert!(
        error.contains("impl `UsesAlias` method `keep` parameter `value`"),
        "{error}"
    );
    assert!(
        error.contains("type alias `Outer` generic parameter `T`"),
        "{error}"
    );
}

#[test]
fn reports_impl_method_return_alias_bound_failure_with_chain_context() {
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

          trait UsesAlias {
            fn make() -> i64;
          }

          type Inner<T: Addable> = T;
          type Outer<T: Addable> = Inner<T>;

          impl UsesAlias for i64 {
            fn make() -> Outer<Text> {
              return "hi";
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
        error.contains("type `Text` does not satisfy bound `Addable`"),
        "{error}"
    );
    assert!(
        error.contains("impl `UsesAlias` method `make` return type"),
        "{error}"
    );
    assert!(
        error.contains("type alias `Outer` generic parameter `T`"),
        "{error}"
    );
}
