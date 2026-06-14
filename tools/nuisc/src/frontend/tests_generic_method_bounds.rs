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
    assert!(error.contains("bound `Addable` uses a different visible name for the same trait"), "{error}");
    assert!(error.contains("use `Helper.Addable` consistently"), "{error}");
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
