use super::{lower_project_ast_to_nir, parse_nuis_ast, parse_nuis_module};

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

#[test]
fn reports_ambiguous_candidate_bounds_for_unbounded_generic_method_call() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          trait Mergeable {
            fn add(lhs: Self, rhs: Self) -> Self;
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
            "function `bump` body calls method `add` on generic parameter `T` without a trait bound; candidate bounds: Addable, Mergeable"
        ),
        "{error}"
    );
}

#[test]
fn reports_ambiguous_candidate_bounds_for_wrong_generic_method_bound() {
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
            "function `bump` body calls method `add` on generic parameter `T` but bound `Showable` does not define that method; candidate bounds: Addable, Mergeable"
        ),
        "{error}"
    );
}

#[test]
fn reports_alias_chain_context_for_missing_generic_method_bound() {
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
fn reports_alias_chain_context_for_wrong_generic_method_bound() {
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

          type Alias<T> = T;
          type Outer<T> = Alias<T>;

          fn bump<T: Showable>(value: Outer<T>) -> T {
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
            "function `bump` body via type alias `Alias` target via type alias `Outer` target"
        ),
        "{error}"
    );
    assert!(
        error.contains(
            "calls method `add` on generic parameter `T` but bound `Showable` does not define that method; consider bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn reports_nested_alias_chain_context_for_ambiguous_unbounded_method_call() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          trait Mergeable {
            fn add(lhs: Self, rhs: Self) -> Self;
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

          type Inner<T> = T;
          type Alias<T> = Inner<T>;
          type Outer<T> = Alias<T>;

          fn bump<T>(value: Outer<T>) -> T {
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
            "function `bump` body via type alias `Inner` target via type alias `Alias` target via type alias `Outer` target"
        ),
        "{error}"
    );
    assert!(
        error.contains(
            "calls method `add` on generic parameter `T` without a trait bound; candidate bounds: Addable, Mergeable"
        ),
        "{error}"
    );
}

#[test]
fn reports_nested_alias_chain_context_for_ambiguous_wrong_bound_method_call() {
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

          type Inner<T> = T;
          type Alias<T> = Inner<T>;
          type Outer<T> = Alias<T>;

          fn bump<T: Showable>(value: Outer<T>) -> T {
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
            "function `bump` body via type alias `Inner` target via type alias `Alias` target via type alias `Outer` target"
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

#[test]
fn reports_missing_bound_for_explicit_trait_qualified_call_on_type_param() {
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
            return Addable.add(value, value);
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
            "function `bump` body calls trait method `Addable.add` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn accepts_explicit_trait_qualified_call_on_bound_type_param() {
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

          fn bump<T: Addable>(value: T) -> T {
            return Addable.add(value, value);
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
fn accepts_explicit_trait_qualified_call_when_trait_is_one_of_multiple_bounds() {
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

          fn bump<T: Addable + Printable>(value: T) -> T {
            return Addable.add(value, value);
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
fn reports_explicit_trait_qualified_call_when_multiple_bounds_miss_required_trait() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          trait Showable {
            fn show(value: Self) -> i64;
          }

          trait Printable {
            fn print(value: Self) -> Text;
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

          impl Printable for i64 {
            fn print(value: i64) -> Text {
              return "ok";
            }
          }

          fn bump<T: Showable + Printable>(value: T) -> T {
            return Addable.add(value, value);
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
            "calls trait method `Addable.add` on generic parameter `T` but declared bounds `Showable + Printable` do not satisfy required trait `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn accepts_explicit_trait_qualified_call_with_public_helper_trait_bound() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn bump<T: Addable>(value: T) -> T {
            return Addable.add(value, value);
          }

          fn main() -> i64 {
            return bump(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "bump__i64"));
}

#[test]
fn accepts_fully_qualified_helper_trait_bound_and_explicit_call() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn bump<T: Helper.Addable>(value: T) -> T {
            return Helper.Addable.add(value, value);
          }

          fn main() -> i64 {
            return bump(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "bump__i64"));
}

#[test]
fn reports_mixed_bare_and_qualified_trait_names_consistently() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn bump<T: Addable>(value: T) -> T {
            return Helper.Addable.add(value, value);
          }

          fn main() -> i64 {
            return bump(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let error = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap_err();
    assert!(
        error.contains("bound `Addable` uses a different visible name for the same trait"),
        "{error}"
    );
    assert!(
        error.contains("use `Helper.Addable` consistently"),
        "{error}"
    );
}

#[test]
fn reports_mixed_bare_and_qualified_trait_names_consistently_with_multiple_bounds() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          trait Printable {
            fn print(value: Self) -> Text;
          }

          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl Printable for i64 {
            fn print(value: i64) -> Text {
              return "ok";
            }
          }

          fn bump<T: Addable + Printable>(value: T) -> T {
            return Helper.Addable.add(value, value);
          }

          fn main() -> i64 {
            return bump(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let error = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap_err();
    assert!(
        error.contains("bound `Addable` uses a different visible name for the same trait"),
        "{error}"
    );
    assert!(
        error.contains("use `Helper.Addable` consistently"),
        "{error}"
    );
}

#[test]
fn accepts_qualified_helper_trait_bound_for_operator_call() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn bump<T: Helper.Addable>(lhs: T, rhs: T) -> T {
            return lhs + rhs;
          }

          fn main() -> i64 {
            return bump(7, 8);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "bump__i64"));
}

#[test]
fn accepts_qualified_helper_trait_bound_for_operator_call_with_multiple_bounds() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          trait Printable {
            fn print(value: Self) -> Text;
          }

          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl Printable for i64 {
            fn print(value: i64) -> Text {
              return "ok";
            }
          }

          fn bump<T: Helper.Addable + Printable>(lhs: T, rhs: T) -> T {
            return lhs + rhs;
          }

          fn main() -> i64 {
            return bump(7, 8);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "bump__i64"));
}

#[test]
fn accepts_qualified_helper_trait_bound_for_operator_call_with_where_clause() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          trait Printable {
            fn print(value: Self) -> Text;
          }

          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl Printable for i64 {
            fn print(value: i64) -> Text {
              return "ok";
            }
          }

          fn bump<T>(lhs: T, rhs: T) -> T where T: Printable, T: Helper.Addable {
            return lhs + rhs;
          }

          fn main() -> i64 {
            return bump(7, 8);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "bump__i64"));
}

#[test]
fn accepts_bare_bound_name_for_operator_call_with_multiple_bounds_when_helper_variant_is_visible() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          trait Printable {
            fn print(value: Self) -> Text;
          }

          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          impl Printable for i64 {
            fn print(value: i64) -> Text {
              return "ok";
            }
          }

          fn bump<T: Addable + Printable>(lhs: T, rhs: T) -> T {
            return lhs + rhs;
          }

          fn main() -> i64 {
            return bump(7, 8);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "bump__i64"));
}

#[test]
fn accepts_qualified_helper_trait_bound_for_operator_call_through_alias_chain_and_if() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          type Alias<T> = T;
          type Outer<T> = Alias<T>;

          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn bump<T: Helper.Addable>(lhs: Outer<T>, rhs: Outer<T>) -> T {
            if true {
              return lhs + rhs;
            }
            return lhs;
          }

          fn main() -> i64 {
            return bump(7, 8);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "bump__i64"));
}

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

#[test]
fn accepts_qualified_helper_trait_bound_for_equality_operator_with_multiple_bounds() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          trait Printable {
            fn print(value: Self) -> Text;
          }

          impl Helper.Equatable for i64 {
            fn eq(lhs: i64, rhs: i64) -> bool {
              return lhs == rhs;
            }
          }

          impl Printable for i64 {
            fn print(value: i64) -> Text {
              return "ok";
            }
          }

          fn same<T: Helper.Equatable + Printable>(lhs: T, rhs: T) -> bool {
            return lhs == rhs;
          }

          fn main() -> i64 {
            if same(7, 7) {
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Equatable {
            fn eq(lhs: Self, rhs: Self) -> bool;
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert_eq!(module.unit, "Main");
}

#[test]
fn accepts_bare_bound_name_for_equality_operator_with_multiple_bounds_when_helper_variant_is_visible() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          trait Printable {
            fn print(value: Self) -> Text;
          }

          impl Helper.Equatable for i64 {
            fn eq(lhs: i64, rhs: i64) -> bool {
              return lhs == rhs;
            }
          }

          impl Printable for i64 {
            fn print(value: i64) -> Text {
              return "ok";
            }
          }

          fn same<T: Equatable + Printable>(lhs: T, rhs: T) -> bool {
            return lhs == rhs;
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Equatable {
            fn eq(lhs: Self, rhs: Self) -> bool;
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert_eq!(module.unit, "Main");
}

#[test]
fn accepts_qualified_helper_trait_bounds_for_mul_div_rem_operators() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Helper.Multipliable for i64 {
            fn mul(lhs: i64, rhs: i64) -> i64 {
              return lhs * rhs;
            }
          }

          impl Helper.Dividable for i64 {
            fn div(lhs: i64, rhs: i64) -> i64 {
              return lhs / rhs;
            }
          }

          impl Helper.Remainderable for i64 {
            fn rem(lhs: i64, rhs: i64) -> i64 {
              return lhs % rhs;
            }
          }

          fn mul_it<T: Helper.Multipliable>(lhs: T, rhs: T) -> T {
            return lhs * rhs;
          }

          fn div_it<T: Helper.Dividable>(lhs: T, rhs: T) -> T {
            return lhs / rhs;
          }

          fn rem_it<T: Helper.Remainderable>(lhs: T, rhs: T) -> T {
            return lhs % rhs;
          }

          fn main() -> i64 {
            return mul_it(6, 7) + div_it(8, 2) + rem_it(9, 4);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Multipliable {
            fn mul(lhs: Self, rhs: Self) -> Self;
          }

          pub trait Dividable {
            fn div(lhs: Self, rhs: Self) -> Self;
          }

          pub trait Remainderable {
            fn rem(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "mul_it__i64"));
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "div_it__i64"));
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "rem_it__i64"));
}

#[test]
fn accepts_qualified_helper_trait_bounds_for_order_operators() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          trait Printable {
            fn print(value: Self) -> Text;
          }

          impl Helper.Orderable for i64 {
            fn lt(lhs: i64, rhs: i64) -> bool {
              return lhs < rhs;
            }
            fn le(lhs: i64, rhs: i64) -> bool {
              return lhs < rhs || lhs == rhs;
            }
            fn gt(lhs: i64, rhs: i64) -> bool {
              return !(lhs <= rhs);
            }
            fn ge(lhs: i64, rhs: i64) -> bool {
              return !(lhs < rhs);
            }
          }

          impl Printable for i64 {
            fn print(value: i64) -> Text {
              return "ok";
            }
          }

          fn ordered<T: Helper.Orderable + Printable>(lhs: T, rhs: T) -> bool {
            return lhs < rhs;
          }

          fn main() -> i64 {
            if ordered(1, 2) {
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Orderable {
            fn lt(lhs: Self, rhs: Self) -> bool;
            fn le(lhs: Self, rhs: Self) -> bool;
            fn gt(lhs: Self, rhs: Self) -> bool;
            fn ge(lhs: Self, rhs: Self) -> bool;
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "ordered__i64"));
}

#[test]
fn accepts_qualified_helper_trait_bounds_for_unary_not_and_neg_on_custom_type() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          struct Pair {
            value: i64,
          }

          impl Helper.Notable for Pair {
            fn not(value: Pair) -> bool {
              return value.value == 0;
            }
          }

          impl Helper.Negatable for Pair {
            fn neg(value: Pair) -> Pair {
              return Pair { value: 0 - value.value };
            }
          }

          fn empty<T: Helper.Notable>(value: T) -> bool {
            return !value;
          }

          fn flip<T: Helper.Negatable>(value: T) -> T {
            return -value;
          }

          fn main() -> i64 {
            let zero: Pair = Pair { value: 0 };
            let seven: Pair = Pair { value: 7 };
            let is_empty = empty(zero);
            let flipped = flip(seven);
            if is_empty {
              return flipped.value;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Notable {
            fn not(value: Self) -> bool;
          }

          pub trait Negatable {
            fn neg(value: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "empty__Pair"));
    assert!(module
        .functions
        .iter()
        .any(|function| function.name == "flip__Pair"));
}

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
