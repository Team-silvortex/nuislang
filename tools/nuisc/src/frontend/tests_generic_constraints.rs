use super::parse_nuis_module;

#[test]
fn rejects_unknown_generic_bound_trait_in_function_declaration() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn keep<T: Missing>(value: T) -> T {
            return value;
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("unknown generic bound trait `Missing`"));
}

#[test]
fn rejects_type_alias_generic_arg_that_does_not_satisfy_bound() {
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

          type Alias<T: Addable> = T;

          fn main() -> i64 {
            let value: Alias<Text> = "hi";
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("type `Text` does not satisfy bound `Addable`"));
    assert!(error.contains("type alias `Alias` generic parameter `T`"));
}

#[test]
fn rejects_non_trait_shaped_generic_bound() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          type Alias<T: Pipe<i64>> = T;

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("generic bounds currently require a bare trait name"));
    assert!(error.contains("Pipe<i64>"));
}

#[test]
fn accepts_type_alias_generic_arg_when_outer_generic_param_satisfies_bound() {
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

          fn keep<U: Addable>(value: Alias<U>) -> U {
            return value;
          }

          fn main() -> i64 {
            return keep(7);
          }
        }
        "#,
    )
    .unwrap();

    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "keep__i64"));
}

#[test]
fn reports_nested_alias_bound_failure_with_alias_chain_context() {
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
          type Outer<T> = Inner<T>;

          fn main() -> i64 {
            let value: Outer<Text> = "hi";
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `T` does not satisfy bound `Addable`"),
        "{error}"
    );
    assert!(error.contains("type alias `Outer` target"), "{error}");
    assert!(
        error.contains("type alias `Inner` generic parameter `T`"),
        "{error}"
    );
}

#[test]
fn reports_alias_definition_missing_bound_through_alias_chain() {
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
          type Outer<T> = Inner<T>;

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `T` does not satisfy bound `Addable`"),
        "{error}"
    );
    assert!(error.contains("type alias `Outer` target"), "{error}");
    assert!(
        error.contains("type alias `Inner` generic parameter `T`"),
        "{error}"
    );
}

#[test]
fn reports_function_generic_call_arg_bound_failure_at_use_site() {
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

          fn keep<U: Addable>(value: Outer<U>) -> U {
            return value;
          }

          fn main() -> i64 {
            let text: Text = "hi";
            keep(text);
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `Text` does not satisfy bound `Addable` for generic parameter `U`"),
        "{error}"
    );
}

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
