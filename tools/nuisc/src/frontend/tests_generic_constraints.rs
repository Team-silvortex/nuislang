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

    assert!(error.contains("where clause for generic parameter `T`"), "{error}");
    assert!(error.contains("unknown generic bound trait `Missing`"), "{error}");
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

    assert!(error.contains("enum `Option` where clause for generic parameter `T`"), "{error}");
    assert!(error.contains("unknown generic bound trait `Missing`"), "{error}");
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

    assert!(error.contains("type `bool` does not satisfy bound `Addable`"), "{error}");
    assert!(error.contains("via enum `Option` generic parameter `T`"), "{error}");
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

    assert!(error.contains("where clause for generic parameter `T`"), "{error}");
    assert!(error.contains("generic bounds currently require a bare trait name"), "{error}");
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

    println!("{error}");
    assert!(error.contains("type `Text` does not satisfy bound `Addable`"));
    assert!(error.contains("type alias `Alias` generic parameter `T`"));
}

#[test]
fn rejects_unannotated_generic_struct_literal_explicit_type_arg_that_violates_bound() {
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

          struct Boxed<T> where T: Addable {
            value: T,
          }

          fn main() -> i64 {
            let boxed = Boxed<Text> { value: "hi" };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("type `Text` does not satisfy bound `Addable`"), "{error}");
    assert!(error.contains("struct literal `Boxed`"), "{error}");
    assert!(error.contains("via struct `Boxed` generic parameter `T`"), "{error}");
}

#[test]
fn rejects_lambda_body_local_alias_bound_failure() {
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

          fn apply<T>(value: T, f: Fn1<T, T>) -> T {
            return f(value);
          }

          fn keep<T>(value: T) -> T {
            return apply(value, |x: T| -> T {
              let local: Alias<Text> = "hi";
              return x;
            });
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("type `Text` does not satisfy bound `Addable`"), "{error}");
    assert!(error.contains("function `keep` body lambda body local `local`"), "{error}");
    assert!(error.contains("type alias `Alias` generic parameter `T`"), "{error}");
}

#[test]
fn rejects_if_result_branch_struct_literal_type_arg_that_violates_bound_with_branch_context() {
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

          struct Boxed<T> where T: Addable {
            value: T,
          }

          fn main() -> i64 {
            let boxed = if true {
              Boxed<Text> { value: "hi" }
            } else {
              Boxed<i64> { value: 0 }
            };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("type `Text` does not satisfy bound `Addable`"), "{error}");
    assert!(error.contains("function `main` body if-then"), "{error}");
    assert!(error.contains("struct literal `Boxed`"), "{error}");
}

#[test]
fn rejects_match_result_branch_struct_literal_type_arg_that_violates_bound_with_arm_context() {
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

          struct Boxed<T> where T: Addable {
            value: T,
          }

          fn main() -> i64 {
            let boxed = match 1 {
              1 => {
                Boxed<Text> { value: "hi" }
              }
              _ => {
                Boxed<i64> { value: 0 }
              }
            };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("type `Text` does not satisfy bound `Addable`"), "{error}");
    assert!(error.contains("function `main` body match-arm"), "{error}");
    assert!(error.contains("struct literal `Boxed`"), "{error}");
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
fn reports_alias_chain_second_missing_bound_through_multi_bound_alias() {
    let error = parse_nuis_module(
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

          type Inner<T: Addable + Printable> = T;
          type Outer<T: Addable> = Inner<T>;

          fn main() -> i64 {
            let value: Outer<i64> = 7;
            return value;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `T` does not satisfy bound `Printable`"),
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
fn accepts_alias_generic_arg_when_outer_generic_param_satisfies_all_multi_bounds() {
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

          type Alias<T: Addable + Printable> = T;

          fn keep<U: Addable + Printable>(value: Alias<U>) -> U {
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
fn accepts_repeated_where_predicates_that_merge_into_multi_bounds() {
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

          fn keep<U>(value: U) -> U where U: Addable, U: Printable {
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
fn rejects_type_alias_where_clause_bound_at_use_site() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          type Alias<T> where T: Addable = T;

          fn main() -> i64 {
            let text: String = "hi";
            let value: Alias<String> = text;
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("type `String` does not satisfy bound `Addable`"));
    assert!(error.contains("type alias `Alias` generic parameter `T`"));
}

#[test]
fn rejects_struct_where_clause_bound_at_field_use_site() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          struct Boxed<T> where T: Addable {
            value: T,
          }

          fn main() -> i64 {
            let text: String = "hi";
            let boxed: Boxed<String> = Boxed { value: text };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("type `String` does not satisfy bound `Addable`"));
    assert!(error.contains("function `main` body local `boxed`"));
    assert!(error.contains("via struct `Boxed` generic parameter `T`"));
}

#[test]
fn reports_explicit_function_generic_arg_bound_failure_at_use_site() {
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

          fn keep<U: Addable>(value: U) -> U {
            return value;
          }

          fn main() -> i64 {
            let text: Text = "hi";
            keep<Text>(text);
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
fn reports_explicit_function_generic_arg_failure_for_second_required_bound() {
    let error = parse_nuis_module(
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

          fn keep<U: Addable + Printable>(value: U) -> U {
            return value;
          }

          fn main() -> i64 {
            keep<i64>(7);
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type `i64` does not satisfy bound `Printable` for generic parameter `U`"),
        "{error}"
    );
}

#[test]
fn accepts_explicit_function_generic_arg_when_all_required_bounds_are_satisfied() {
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

          fn keep<U: Addable + Printable>(value: U) -> U {
            return value;
          }

          fn main() -> i64 {
            return keep<i64>(7);
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
fn reports_explicit_function_generic_arg_bound_failure_from_where_clause() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          fn keep<U>(value: U) -> U where U: Addable {
            return value;
          }

          fn main() -> i64 {
            keep<Text>("hi");
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
fn reports_explicit_function_generic_arg_bound_failure_inside_if_result_branch() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          fn keep<U>(value: U) -> U where U: Addable {
            return value;
          }

          fn main() -> i64 {
            let value = if true {
              keep<Text>("hi")
            } else {
              keep<i64>(0)
            };
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
fn reports_explicit_function_generic_arg_bound_failure_inside_match_result_branch() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          fn keep<U>(value: U) -> U where U: Addable {
            return value;
          }

          fn main() -> i64 {
            let value = match 1 {
              1 => {
                keep<Text>("hi")
              }
              _ => {
                keep<i64>(0)
              }
            };
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
fn reports_explicit_function_generic_arg_bound_failure_inside_lambda_body() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          fn apply<T>(value: T, mapper: Fn1<T, T>) -> T {
            return mapper(value);
          }

          fn keep<U>(value: U) -> U where U: Addable {
            return value;
          }

          fn main() -> i64 {
            let value = apply(0, |x: i64| -> i64 {
              keep<Text>("hi");
              return x;
            });
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
fn accepts_explicit_function_generic_arg_when_where_clause_bounds_are_satisfied() {
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

          fn keep<U>(value: U) -> U where U: Addable {
            return value;
          }

          fn main() -> i64 {
            return keep<i64>(7);
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
fn accepts_function_generic_use_site_bound_through_visible_helper_trait_name_variant() {
    let main_ast = super::parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn keep<U: Helper.Addable>(value: U) -> U {
            return value;
          }

          fn main() -> i64 {
            return keep(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = super::parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = super::lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "keep__i64"));
}

#[test]
fn accepts_function_generic_use_site_with_helper_and_local_multi_bounds() {
    let main_ast = super::parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
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

          fn keep<U: Helper.Addable + Printable>(value: U) -> U {
            return value;
          }

          fn main() -> i64 {
            return keep(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = super::parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = super::lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "keep__i64"));
}

#[test]
fn reports_ambiguous_function_generic_use_site_bound_across_helper_trait_variants() {
    let main_ast = super::parse_nuis_ast(
        r#"
        use cpu HelperA;
        use cpu HelperB;

        mod cpu Main {
          impl HelperA.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl HelperB.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn keep<U: Addable>(value: U) -> U {
            return value;
          }

          fn main() -> i64 {
            return keep(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper_a_ast = super::parse_nuis_ast(
        r#"
        mod cpu HelperA {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();
    let helper_b_ast = super::parse_nuis_ast(
        r#"
        mod cpu HelperB {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let error =
        super::lower_project_ast_to_nir(&main_ast, &[helper_a_ast, helper_b_ast]).unwrap_err();
    assert!(
        error.contains("type `i64` ambiguously satisfies bound `Addable` for generic parameter `U`"),
        "{error}"
    );
    assert!(error.contains("HelperA.Addable"), "{error}");
    assert!(error.contains("HelperB.Addable"), "{error}");
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
