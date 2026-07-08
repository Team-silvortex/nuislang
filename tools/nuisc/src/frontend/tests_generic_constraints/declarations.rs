use super::*;

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
fn rejects_where_clause_for_unknown_generic_parameter() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          fn keep<T>(value: T) -> T where U: Addable {
            return value;
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("where clause references unknown generic parameter `U`"));
}

#[test]
fn rejects_unknown_generic_bound_trait_in_where_clause() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn keep<T>(value: T) -> T where T: Missing {
            return value;
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("where clause for generic parameter `T`"),
        "{error}"
    );
    assert!(
        error.contains("unknown generic bound trait `Missing`"),
        "{error}"
    );
}

#[test]
fn rejects_struct_where_clause_for_unknown_generic_parameter() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          struct Boxed<T> where U: Addable {
            value: T,
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("struct `Boxed` where clause references unknown generic parameter `U`"));
}

#[test]
fn rejects_enum_where_clause_for_unknown_generic_parameter() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          enum Option<T> where U: Addable {
            None,
            Some(T),
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("enum `Option` where clause references unknown generic parameter `U`"));
}

#[test]
fn rejects_unknown_generic_bound_trait_in_enum_declaration() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option<T> where T: Missing {
            None,
            Some(T),
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("enum `Option` where clause for generic parameter `T`"),
        "{error}"
    );
    assert!(
        error.contains("unknown generic bound trait `Missing`"),
        "{error}"
    );
}

#[test]
fn rejects_enum_use_site_argument_that_violates_declared_bound() {
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

          enum Option<T> where T: Addable {
            None,
            Some(T),
          }

          fn main() {
            let value: Option<bool> = Option.Some(true);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `bool` does not satisfy bound `Addable`"),
        "{error}"
    );
    assert!(
        error.contains("via enum `Option` generic parameter `T`"),
        "{error}"
    );
}

#[test]
fn rejects_non_trait_shaped_generic_bound_in_where_clause() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn keep<T>(value: T) -> T where T: Pipe<i64> {
            return value;
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("where clause for generic parameter `T`"),
        "{error}"
    );
    assert!(
        error.contains("generic bounds currently require a bare trait name"),
        "{error}"
    );
    assert!(error.contains("Pipe<i64>"), "{error}");
}

#[test]
fn suggests_visible_qualified_trait_name_for_unknown_bare_generic_bound() {
    let main_ast = super::parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          fn keep<T: Worker.Missing>(value: T) -> T {
            return value;
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = super::parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Missing {
            fn keep(value: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let error = super::lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap_err();
    assert!(
        error.contains("unknown generic bound trait `Worker.Missing`"),
        "{error}"
    );
    assert!(error.contains("did you mean `Helper.Missing`?"), "{error}");
}

#[test]
fn rejects_impl_for_unknown_trait() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          impl Missing for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
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
        error.contains("impl references unknown trait `Missing`"),
        "{error}"
    );
}

#[test]
fn rejects_duplicate_impl_for_same_trait_and_type() {
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

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
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
        error.contains("duplicate impl for trait `Addable` and type `i64`"),
        "{error}"
    );
}

#[test]
fn rejects_impl_missing_required_trait_method() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
            fn zero() -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
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
        error.contains("missing required trait method `zero`"),
        "{error}"
    );
    assert!(error.contains("impl `Addable` for `i64`"), "{error}");
}

#[test]
fn accepts_impl_omitting_trait_method_with_default_body() {
    parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;

            fn zero() -> Self {
              return Addable.add(0, 0);
            }
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();
}

#[test]
fn rejects_impl_extra_method_not_declared_by_trait() {
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

            fn zero() -> i64 {
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

    assert!(error.contains("extra impl method `zero`"), "{error}");
    assert!(error.contains("trait `Addable`"), "{error}");
}

#[test]
fn rejects_impl_method_signature_mismatch_against_trait() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> Text {
              return "oops";
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
        error.contains("method `add` in impl `Addable` for `i64` does not match trait signature"),
        "{error}"
    );
}
