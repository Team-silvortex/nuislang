use super::*;

#[test]
fn lowers_generic_named_function_through_concrete_fn1_parameter() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn id<T>(value: T) -> T {
            return value;
          }

          fn apply_i64(value: i64, mapper: Fn1<i64, i64>) -> i64 {
            return mapper(value);
          }

          fn main() -> i64 {
            return apply_i64(6, id);
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| function.name == "__hof_apply_i64_id")
        .expect("expected higher-order helper specialized for generic named function");
    assert!(helper.generic_params.is_empty());
    assert!(matches!(
        helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "id__i64"
                && matches!(args.as_slice(), [NirExpr::Var(name)] if name == "value")
    ));

    let id_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "id__i64")
        .expect("expected generic callable specialization");
    assert!(id_specialized.generic_params.is_empty());
}

// Generic callable value coverage across Fn1/Fn2/Fn3 and callable aliases.
#[test]
fn lowers_generic_named_function_through_generic_fn1_template_parameter() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn id<T>(value: T) -> T {
            return value;
          }

          fn apply<T>(value: T, mapper: Fn1<T, T>) -> T {
            return mapper(value);
          }

          fn main() -> i64 {
            return apply(9, id);
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| function.name == "__hof_apply_id__i64")
        .expect("expected monomorphized higher-order helper");
    assert!(helper.generic_params.is_empty());
    assert!(matches!(
        helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "id__i64"
                && matches!(args.as_slice(), [NirExpr::Var(name)] if name == "value")
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &helper.name
    ));
}

#[test]
fn lowers_generic_named_function_through_generic_fn1_alias_parameter() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Mapper<T> = Fn1<T, T>;

          fn id<T>(value: T) -> T {
            return value;
          }

          fn apply<T>(value: T, mapper: Mapper<T>) -> T {
            return mapper(value);
          }

          fn main() -> i64 {
            return apply(9, id);
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| function.name == "__hof_apply_id__i64")
        .expect("expected monomorphized alias Fn1 higher-order helper");
    assert!(helper.generic_params.is_empty());
    assert!(matches!(
        helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "id__i64"
                && matches!(args.as_slice(), [NirExpr::Var(name)] if name == "value")
    ));
}

#[test]
fn lowers_generic_named_function_through_concrete_fn2_parameter() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn choose_left<T>(lhs: T, rhs: T) -> T {
            return lhs;
          }

          fn apply_i64(lhs: i64, rhs: i64, mapper: Fn2<i64, i64, i64>) -> i64 {
            return mapper(lhs, rhs);
          }

          fn main() -> i64 {
            return apply_i64(6, 2, choose_left);
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| function.name == "__hof_apply_i64_choose_left")
        .expect("expected Fn2 higher-order helper specialized for generic named function");
    assert!(helper.generic_params.is_empty());
    assert!(matches!(
        helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "choose_left__i64"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)] if lhs == "lhs" && rhs == "rhs"
                )
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "choose_left__i64")
        .expect("expected generic Fn2 callable specialization");
    assert!(specialized.generic_params.is_empty());
}

#[test]
fn lowers_generic_named_function_through_generic_fn2_template_parameter() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn choose_left<T>(lhs: T, rhs: T) -> T {
            return lhs;
          }

          fn apply<T>(lhs: T, rhs: T, mapper: Fn2<T, T, T>) -> T {
            return mapper(lhs, rhs);
          }

          fn main() -> i64 {
            return apply(9, 4, choose_left);
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| function.name == "__hof_apply_choose_left__i64")
        .expect("expected monomorphized Fn2 higher-order helper");
    assert!(helper.generic_params.is_empty());
    assert!(matches!(
        helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "choose_left__i64"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)] if lhs == "lhs" && rhs == "rhs"
                )
    ));
}

#[test]
fn lowers_generic_named_function_through_generic_fn2_alias_parameter() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Mapper2<T> = Fn2<T, T, T>;

          fn choose_left<T>(lhs: T, rhs: T) -> T {
            return lhs;
          }

          fn apply<T>(lhs: T, rhs: T, mapper: Mapper2<T>) -> T {
            return mapper(lhs, rhs);
          }

          fn main() -> i64 {
            return apply(9, 4, choose_left);
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| function.name == "__hof_apply_choose_left__i64")
        .expect("expected monomorphized alias Fn2 higher-order helper");
    assert!(helper.generic_params.is_empty());
    assert!(matches!(
        helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "choose_left__i64"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)] if lhs == "lhs" && rhs == "rhs"
                )
    ));
}

#[test]
fn lowers_generic_named_function_through_concrete_fn3_parameter() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn choose_first<T>(lhs: T, mid: T, rhs: T) -> T {
            return lhs;
          }

          fn apply_i64(lhs: i64, mid: i64, rhs: i64, mapper: Fn3<i64, i64, i64, i64>) -> i64 {
            return mapper(lhs, mid, rhs);
          }

          fn main() -> i64 {
            return apply_i64(6, 2, 1, choose_first);
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| function.name == "__hof_apply_i64_choose_first")
        .expect("expected Fn3 higher-order helper specialized for generic named function");
    assert!(helper.generic_params.is_empty());
    assert!(matches!(
        helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "choose_first__i64"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(mid), NirExpr::Var(rhs)]
                        if lhs == "lhs" && mid == "mid" && rhs == "rhs"
                )
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "choose_first__i64")
        .expect("expected generic Fn3 callable specialization");
    assert!(specialized.generic_params.is_empty());
}

#[test]
fn lowers_generic_named_function_through_generic_fn3_template_parameter() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn choose_first<T>(lhs: T, mid: T, rhs: T) -> T {
            return lhs;
          }

          fn apply<T>(lhs: T, mid: T, rhs: T, mapper: Fn3<T, T, T, T>) -> T {
            return mapper(lhs, mid, rhs);
          }

          fn main() -> i64 {
            return apply(9, 4, 1, choose_first);
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| function.name == "__hof_apply_choose_first__i64")
        .expect("expected monomorphized Fn3 higher-order helper");
    assert!(helper.generic_params.is_empty());
    assert!(matches!(
        helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "choose_first__i64"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(mid), NirExpr::Var(rhs)]
                        if lhs == "lhs" && mid == "mid" && rhs == "rhs"
                )
    ));
}

#[test]
fn lowers_generic_named_function_through_generic_fn3_alias_parameter() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Reducer<T> = Fn3<T, T, T, T>;

          fn choose_first<T>(lhs: T, mid: T, rhs: T) -> T {
            return lhs;
          }

          fn apply<T>(lhs: T, mid: T, rhs: T, mapper: Reducer<T>) -> T {
            return mapper(lhs, mid, rhs);
          }

          fn main() -> i64 {
            return apply(9, 4, 1, choose_first);
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| function.name == "__hof_apply_choose_first__i64")
        .expect("expected monomorphized alias Fn3 higher-order helper");
    assert!(helper.generic_params.is_empty());
    assert!(matches!(
        helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "choose_first__i64"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(mid), NirExpr::Var(rhs)]
                        if lhs == "lhs" && mid == "mid" && rhs == "rhs"
                )
    ));
}

// Capturing lambda threading through generic callable parameters.
