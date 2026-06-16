use super::{lower_project_ast_to_nir, parse_nuis_ast, parse_nuis_module};
use nuis_semantics::model::{NirExpr, NirStmt};

fn stmt_tree_contains_call<F>(body: &[NirStmt], predicate: &F) -> bool
where
    F: Fn(&str, &[NirExpr]) -> bool,
{
    body.iter().any(|stmt| stmt_contains_call(stmt, predicate))
}

fn stmt_contains_call<F>(stmt: &NirStmt, predicate: &F) -> bool
where
    F: Fn(&str, &[NirExpr]) -> bool,
{
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Expr(value)
        | NirStmt::Await(value)
        | NirStmt::Print(value) => expr_contains_call(value, predicate),
        NirStmt::Return(Some(value)) => expr_contains_call(value, predicate),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_contains_call(condition, predicate)
                || stmt_tree_contains_call(then_body, predicate)
                || stmt_tree_contains_call(else_body, predicate)
        }
        NirStmt::While { condition, body } => {
            expr_contains_call(condition, predicate) || stmt_tree_contains_call(body, predicate)
        }
        _ => false,
    }
}

fn expr_contains_call<F>(expr: &NirExpr, predicate: &F) -> bool
where
    F: Fn(&str, &[NirExpr]) -> bool,
{
    match expr {
        NirExpr::Call { callee, args } => {
            predicate(callee, args) || args.iter().any(|arg| expr_contains_call(arg, predicate))
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_contains_call(value, predicate)),
        NirExpr::FieldAccess { base, .. }
        | NirExpr::Await(base)
        | NirExpr::Borrow(base)
        | NirExpr::BorrowEnd(base)
        | NirExpr::CpuJoin(base)
        | NirExpr::CpuThreadJoin(base)
        | NirExpr::DataReady(base)
        | NirExpr::DataMoved(base)
        | NirExpr::DataWindowed(base)
        | NirExpr::DataValue(base)
        | NirExpr::CpuThreadJoinResult(base)
        | NirExpr::CpuTaskCompleted(base)
        | NirExpr::CpuTaskTimedOut(base)
        | NirExpr::CpuTaskCancelled(base)
        | NirExpr::CpuTaskValue(base)
        | NirExpr::CpuMutexNew(base)
        | NirExpr::CpuMutexLock(base)
        | NirExpr::CpuMutexUnlock(base)
        | NirExpr::CpuMutexValue(base) => expr_contains_call(base, predicate),
        NirExpr::Binary { lhs, rhs, .. } => {
            expr_contains_call(lhs, predicate) || expr_contains_call(rhs, predicate)
        }
        NirExpr::CpuExternCall { args, .. } => {
            args.iter().any(|arg| expr_contains_call(arg, predicate))
        }
        NirExpr::CpuSpawn { args, .. } | NirExpr::CpuThreadSpawn { args, .. } => {
            args.iter().any(|arg| expr_contains_call(arg, predicate))
        }
        _ => false,
    }
}

// Baseline callable specialization and named-function routing.
#[test]
fn combines_higher_order_specialization_with_trait_generic_monomorphization() {
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

          fn sum_two<T: Addable>(lhs: T, rhs: T) -> T {
            return lhs.add(rhs);
          }

          fn apply_and_sum(x: i64, y: i64, f: Fn1<i64, i64>) -> i64 {
            return sum_two(f(x), y);
          }

          fn main() -> i64 {
            return apply_and_sum(6, 1, |x: i64| -> i64 { return x; });
          }
        }
        "#,
    )
    .unwrap();

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized lambda function");
    let higher_order = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_and_sum_"))
        .expect("expected synthesized higher-order specialization");
    assert!(matches!(
        higher_order.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "sum_two__i64"
                && matches!(args.as_slice(), [NirExpr::Call { callee: inner, .. }, NirExpr::Var(y)] if inner == &lambda.name && y == "y")
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &higher_order.name
    ));
}

#[test]
fn lowers_generic_fn1_higher_order_lambda_family() {
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

          fn apply<T: Addable>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn main() -> i64 {
            return apply(6, |x: i64| -> i64 { return x + 1; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee == "impl.Addable.for.i64.add"
                || callee.starts_with("__lambda_main_")
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &higher_order_concrete.name
    ));
}

#[test]
fn lowers_result_map_and_and_then_higher_order_helpers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum CoreError {
            InvalidInput,
          }

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn result_map<T, R, E>(result: Result<T, E>, mapper: Fn1<T, R>) -> Result<R, E> {
            match result {
              Result.Ok(value) => {
                return Result.Ok(mapper(value));
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          fn result_and_then<T, R, E>(
            result: Result<T, E>,
            mapper: Fn1<T, Result<R, E>>
          ) -> Result<R, E> {
            match result {
              Result.Ok(value) => {
                return mapper(value);
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          fn main() -> i64 {
            let input: Result<i64, CoreError> = Result.Ok(7);
            let mapped: Result<i64, CoreError> = result_map(
              input,
              |value: i64| -> i64 { return value + 1; }
            );
            let chained: Result<i64, CoreError> = result_and_then(
              mapped,
              |value: i64| -> Result<i64, CoreError> { return Result.Ok(value * 2); }
            );
            match chained {
              Result.Ok(value) => {
                return value;
              }
              Result.Err(_) => {
                return -1;
              }
            }
          }
        }
        "#,
    )
    .unwrap();

    assert!(module
        .functions
        .iter()
        .any(|function| function.name.starts_with("__hof_result_map_")));
    assert!(module
        .functions
        .iter()
        .any(|function| function.name.starts_with("__hof_result_and_then_")));
}

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
#[test]
fn lowers_explicit_generic_fn1_higher_order_call_with_zero_arg_generic_argument() {
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

          fn typed_zero<T>() -> T {
            return 0;
          }

          fn apply<T: Addable>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn main() -> i64 {
            return apply<i64>(typed_zero(), |x: i64| -> i64 { return x + 1; });
          }
        }
        "#,
    )
    .unwrap();

    let specialized_zero = module
        .functions
        .iter()
        .find(|function| function.name == "typed_zero__i64")
        .expect("expected zero-arg generic call to specialize through explicit higher-order call");
    assert!(specialized_zero.generic_params.is_empty());

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected explicit-generic higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == &higher_order_concrete.name
                && matches!(
                    args.as_slice(),
                    [NirExpr::Call { callee: zero_callee, .. }]
                        if zero_callee == "typed_zero__i64"
                )
    ));
}

// Payload alias, async, and recursive higher-order specialization.
#[test]
fn lowers_generic_fn1_alias_higher_order_lambda_family() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Mapper<T> = Fn1<T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn apply<T: Addable>(x: T, f: Mapper<T>) -> T {
            return f(x);
          }

          fn main() -> i64 {
            return apply(6, |x: i64| -> i64 { return x + 1; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &higher_order_concrete.name
    ));
}

#[test]
fn lowers_capturing_lambda_through_generic_fn1_template_parameter() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn apply<T>(value: T, mapper: Fn1<T, T>) -> T {
            return mapper(value);
          }

          fn main() -> i64 {
            let seed: i64 = 6;
            return apply(1, |x: i64| -> i64 { return x + seed; });
          }
        }
        "#,
    )
    .unwrap();

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized captured generic lambda");
    assert!(lambda.generic_params.is_empty());
    assert_eq!(lambda.params.len(), 2);

    let helper = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized higher-order helper with capture threading");
    assert!(helper.generic_params.is_empty());
    assert_eq!(helper.params.len(), 2);
    assert!(matches!(
        helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &lambda.name
                && matches!(args.as_slice(), [NirExpr::Var(x), NirExpr::Var(seed)] if x == "value" && seed == "__capture_mapper_seed_0")
    ));
}

#[test]
fn lowers_capturing_lambda_through_generic_fn1_alias_parameter() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Mapper<T> = Fn1<T, T>;

          fn apply<T>(value: T, mapper: Mapper<T>) -> T {
            return mapper(value);
          }

          fn main() -> i64 {
            let seed: i64 = 6;
            return apply(1, |x: i64| -> i64 { return x + seed; });
          }
        }
        "#,
    )
    .unwrap();

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized captured alias generic lambda");
    assert!(lambda.generic_params.is_empty());
    let helper = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected alias higher-order helper with capture threading");
    assert!(helper.generic_params.is_empty());
    assert_eq!(helper.params.len(), 2);
    assert!(matches!(
        helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == &lambda.name
                && matches!(args.as_slice(), [NirExpr::Var(x), NirExpr::Var(seed)] if x == "value" && seed == "__capture_mapper_seed_0")
    ));
}

#[test]
fn lowers_capturing_lambda_through_generic_fn2_and_fn3_parameters() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Reducer<T> = Fn3<T, T, T, T>;

          fn apply2<T>(lhs: T, rhs: T, mapper: Fn2<T, T, T>) -> T {
            return mapper(lhs, rhs);
          }

          fn apply3<T>(lhs: T, mid: T, rhs: T, mapper: Reducer<T>) -> T {
            return mapper(lhs, mid, rhs);
          }

          fn main() -> i64 {
            let seed: i64 = 6;
            let pair: i64 = apply2(1, 2, |x: i64, y: i64| -> i64 { return x + y + seed; });
            return apply3(pair, 3, 4, |lhs: i64, mid: i64, rhs: i64| -> i64 {
              return lhs + mid + rhs + seed;
            });
          }
        }
        "#,
    )
    .unwrap();

    let apply2_helper = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply2_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized Fn2 helper");
    assert!(apply2_helper.generic_params.is_empty());
    assert_eq!(apply2_helper.params.len(), 3);
    assert!(matches!(
        apply2_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_main_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs), NirExpr::Var(seed)]
                        if lhs == "lhs" && rhs == "rhs" && seed == "__capture_mapper_seed_0"
                )
    ));

    let apply3_helper = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply3_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized Fn3 alias helper");
    assert!(apply3_helper.generic_params.is_empty());
    assert_eq!(apply3_helper.params.len(), 4);
    assert!(matches!(
        apply3_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_main_")
                && matches!(
                    args.as_slice(),
                    [
                        NirExpr::Var(lhs),
                        NirExpr::Var(mid),
                        NirExpr::Var(rhs),
                        NirExpr::Var(seed)
                    ] if lhs == "lhs"
                        && mid == "mid"
                        && rhs == "rhs"
                        && seed == "__capture_mapper_seed_0"
                )
    ));
}

#[test]
fn lowers_explicit_generic_higher_order_template_call_inside_template_body() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn add_one(value: i64) -> i64 {
            return value + 1;
          }

          fn apply<T>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn chain(x: i64, f: Fn1<i64, i64>) -> i64 {
            return apply<i64>(f(x), add_one);
          }

          fn main() -> i64 {
            return chain(6, |x: i64| -> i64 { return x + 2; });
          }
        }
        "#,
    )
    .unwrap();

    let apply_helper_count = module
        .functions
        .iter()
        .filter(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .count();
    assert!(
        apply_helper_count >= 1,
        "expected nested explicit-generic higher-order expansion to emit an apply specialization, found {apply_helper_count}"
    );

    let chain_helper = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_chain_"))
        .expect("expected explicit-generic chain higher-order helper");
    assert!(chain_helper.generic_params.is_empty());
    assert!(matches!(
        chain_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee.starts_with("__hof_apply_") && callee.ends_with("__i64")
    ));
}

#[test]
fn lowers_forwarded_callable_parameter_into_nested_higher_order_template_call() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn add_one(value: i64) -> i64 {
            return value + 1;
          }

          fn apply<T>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn chain(x: i64, f: Fn1<i64, i64>) -> i64 {
            return apply(f(x), add_one);
          }

          fn relay(x: i64, f: Fn1<i64, i64>) -> i64 {
            return chain(x, f);
          }

          fn main() -> i64 {
            return relay(6, |x: i64| -> i64 { return x + 2; });
          }
        }
        "#,
    )
    .unwrap();

    let relay_helper = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_relay_"))
        .expect("expected forwarding relay higher-order helper");
    assert!(relay_helper.generic_params.is_empty());
    assert!(matches!(
        relay_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee.starts_with("__hof_chain_")
    ));

    let chain_helper = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_chain_"))
        .expect("expected forwarded chain higher-order helper");
    assert!(chain_helper.generic_params.is_empty());
    assert!(matches!(
        chain_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee.starts_with("__hof_apply_")
    ));
}

#[test]
fn lowers_nested_generic_fn1_alias_higher_order_lambda_family() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Mapper<T> = Fn1<T, T>;
          type NestedMapper<T> = Mapper<T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn apply<T: Addable>(x: T, f: NestedMapper<T>) -> T {
            return f(x);
          }

          fn main() -> i64 {
            return apply(6, |x: i64| -> i64 { return x + 1; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized nested higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &higher_order_concrete.name
    ));
}

#[test]
fn lowers_generic_payload_alias_into_generic_fn1_higher_order_lambda_family() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;
          type Mapper<T> = Fn1<T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          struct Just<T> {
            value: T,
          }

          fn apply_payload<T: Addable>(value: JustAlias<T>, f: Mapper<T>) -> T {
            match value {
              JustAlias<T>(payload) => {
                return f(payload);
              }
              _ => {
                return value.value;
              }
            }
          }

          fn main() -> i64 {
            return apply_payload(JustAlias<i64>(6), |x: i64| -> i64 { return x + 1; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_payload_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized payload higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &higher_order_concrete.name
    ));
}

#[test]
fn lowers_inferred_generic_payload_alias_into_generic_fn1_higher_order_lambda_family() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;
          type Mapper<T> = Fn1<T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          struct Just<T> {
            value: T,
          }

          fn apply_payload<T: Addable>(value: JustAlias<T>, f: Mapper<T>) -> T {
            match value {
              JustAlias<T>(payload) => {
                return f(payload);
              }
              _ => {
                return value.value;
              }
            }
          }

          fn main() -> i64 {
            return apply_payload(JustAlias(6), |x: i64| -> i64 { return x + 1; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_payload_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized inferred payload higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &higher_order_concrete.name
    ));
}

#[test]
fn lowers_generic_payload_alias_method_bound_and_higher_order_combo() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;
          type Mapper<T> = Fn1<T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          struct Just<T> {
            value: T,
          }

          fn apply_payload<T: Addable>(value: JustAlias<T>, f: Mapper<T>) -> T {
            match value {
              JustAlias<T>(payload) => {
                let mapped = f(payload);
                return mapped.add(payload);
              }
              _ => {
                return value.value;
              }
            }
          }

          fn main() -> i64 {
            return apply_payload(JustAlias<i64>(6), |x: i64| -> i64 { return x + 1; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_payload_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized payload higher-order combo helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [
            NirStmt::If {
                condition: NirExpr::Bool(true),
                then_body,
                else_body,
            },
        ] if matches!(
            then_body.as_slice(),
            [
                NirStmt::Let {
                    name: payload_name,
                    value: NirExpr::FieldAccess { base, field },
                    ..
                },
                NirStmt::Let {
                    name: mapped_name,
                    value: NirExpr::Call { callee: lambda_callee, args: lambda_args },
                    ..
                },
                NirStmt::Return(Some(NirExpr::Call { callee: add_callee, args: add_args })),
            ] if payload_name == "payload"
                && mapped_name == "mapped"
                && matches!(&**base, NirExpr::Var(name) if name == "value")
                && field == "value"
                && lambda_callee.starts_with("__lambda_main_")
                && matches!(lambda_args.as_slice(), [NirExpr::Var(name)] if name == "payload")
                && add_callee == "impl.Addable.for.i64.add"
                && matches!(add_args.as_slice(), [NirExpr::Var(lhs), NirExpr::Var(rhs)] if lhs == "mapped" && rhs == "payload")
        ) && matches!(
            else_body.as_slice(),
            [NirStmt::Return(Some(NirExpr::FieldAccess { base, field }))]
                if matches!(&**base, NirExpr::Var(name) if name == "value")
                    && field == "value"
        )
    ));

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &higher_order_concrete.name
    ));
}

#[test]
fn lowers_async_await_into_inferred_generic_payload_alias_higher_order_family() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;
          type Mapper<T> = Fn1<T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          struct Just<T> {
            value: T,
          }

          async fn typed_zero<T>() -> T {
            return 0;
          }

          fn apply_payload<T: Addable>(value: JustAlias<T>, f: Mapper<T>) -> T {
            match value {
              JustAlias<T>(payload) => {
                return f(payload);
              }
              _ => {
                return value.value;
              }
            }
          }

          async fn main() -> i64 {
            return apply_payload(JustAlias(await typed_zero()), |x: i64| -> i64 { return x + 1; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_payload_") && function.name.ends_with("__i64")
        })
        .expect("expected async-inferred payload higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "typed_zero__i64")
        .expect("expected async generic specialization through await payload alias path");
    assert!(specialized.is_async);
    assert!(specialized.generic_params.is_empty());

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == &higher_order_concrete.name
                && matches!(
                    args.as_slice(),
                    [NirExpr::StructLiteral { fields, .. }]
                        if matches!(
                            fields.as_slice(),
                            [(field, NirExpr::Await(value))]
                                if field == "value"
                                    && matches!(
                                        value.as_ref(),
                                        NirExpr::Call { callee, .. } if callee == "typed_zero__i64"
                                    )
                        )
                )
    ));
}

#[test]
fn lowers_specialized_generic_recursive_async_body_into_payload_alias_higher_order_family() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;
          type Mapper<T> = Fn1<T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          struct Just<T> {
            value: T,
          }

          fn apply_payload<T: Addable>(value: JustAlias<T>, f: Mapper<T>) -> T {
            match value {
              JustAlias<T>(payload) => {
                return f(payload);
              }
              _ => {
                return value.value;
              }
            }
          }

          async fn climb<T: Addable>(value: T, remaining: i64) -> T {
            if remaining == 0 {
              return apply_payload(
                JustAlias<T>(value),
                |x: T| -> T { return x.add(1); }
              );
            }
            return await climb(value, remaining - 1);
          }

          async fn main() -> i64 {
            return await climb(7, 4);
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_payload_") && function.name.ends_with("__i64")
        })
        .expect("expected recursive async payload higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "climb__i64")
        .expect(
            "expected recursive async generic specialization through higher-order payload body",
        );
    assert!(specialized.is_async);
    assert!(specialized.generic_params.is_empty());

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Await(value))))
            if matches!(
                value.as_ref(),
                NirExpr::Call { callee, .. } if callee == "climb__i64"
            )
    ));
}

#[test]
fn lowers_specialized_generic_recursive_async_body_with_capturing_lambda_and_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;
          type Mapper<T> = Fn1<T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          struct Just<T> {
            value: T,
          }

          fn apply_payload<T: Addable>(value: JustAlias<T>, f: Mapper<T>) -> T {
            match value {
              JustAlias<T>(payload) => {
                return f(payload);
              }
              _ => {
                return value.value;
              }
            }
          }

          async fn climb<T: Addable>(value: T, extra: T, remaining: i64) -> T {
            if remaining == 0 {
              return apply_payload(
                JustAlias<T>(value),
                |x: T| -> T { return x.add(extra); }
              );
            }
            return await climb(value, extra, remaining - 1);
          }

          async fn main() -> i64 {
            return await climb(7, 3, 4);
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_payload_") && function.name.ends_with("__i64")
        })
        .expect("expected recursive async payload higher-order helper with capture threading");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::If { condition: NirExpr::Bool(true), then_body, else_body }]
            if matches!(
                then_body.as_slice(),
                [
                    NirStmt::Let {
                        name: payload_name,
                        value: NirExpr::FieldAccess { base, field },
                        ..
                    },
                    NirStmt::Return(Some(NirExpr::Call { callee, args }))
                ]
                    if payload_name == "payload"
                        && matches!(base.as_ref(), NirExpr::Var(name) if name == "value")
                        && field == "value"
                        && callee.starts_with("__lambda_climb_")
                        && matches!(
                            args.as_slice(),
                            [NirExpr::Var(payload), NirExpr::Var(extra)]
                                if payload == "payload" && extra == "__capture_f_extra_0"
                        )
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::FieldAccess { base, field }))]
                    if field == "value"
                        && matches!(base.as_ref(), NirExpr::Var(name) if name == "value")
            )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_climb_") && function.name.ends_with("__i64")
        })
        .expect("expected captured recursive lambda specialization");
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Addable.for.i64.add"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "climb__i64")
        .expect("expected recursive async generic specialization with capture");
    assert!(specialized.is_async);
    assert!(specialized.generic_params.is_empty());
    assert!(stmt_tree_contains_call(
        &specialized.body,
        &|callee, args| {
            callee == higher_order_concrete.name
                && matches!(
                    args,
                    [NirExpr::StructLiteral { .. }, NirExpr::Var(extra)] if extra == "extra"
                )
        }
    ));
    assert!(stmt_tree_contains_call(
        &specialized.body,
        &|callee, args| {
            callee == "climb__i64"
                && matches!(
                    args,
                    [NirExpr::Var(value), NirExpr::Var(extra), NirExpr::Binary { .. }]
                        if value == "value" && extra == "extra"
                )
        }
    ));
}

// Trait-bound validation and bound-preserving generic lambda specialization.
#[test]
fn rejects_generic_lambda_method_call_without_required_bound() {
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

          fn apply<T>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn bump<T>(value: T) -> T {
            return apply(value, |x: T| -> T { return x.add(x); });
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
            "function `bump` body lambda body calls method `add` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn rejects_generic_higher_order_specialization_method_call_without_required_bound() {
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

          fn apply<T>(x: T, f: Fn1<T, T>) -> T {
            let local = f(x);
            return local.add(x);
          }

          fn bump<T>(value: T) -> T {
            return apply(value, |x: T| -> T { return x; });
          }

          fn main() -> i64 {
            return bump(0);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "function `apply` body higher-order specialization body calls method `add` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn rejects_generic_higher_order_specialization_without_required_bound_inside_nested_while_match() {
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

          fn apply<T>(x: T, f: Fn1<T, T>) -> T {
            let local = f(x);
            return local.add(x);
          }

          fn choose<T>(value: T, mode: i64) -> T {
            while mode > 0 {
              match mode {
                1 => {
                  return apply(value, |x: T| -> T { return x; });
                }
                _ => {
                  return value;
                }
              }
            }
            return value;
          }

          fn main() -> i64 {
            return choose(0, 1);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "function `apply` body higher-order specialization body calls method `add` on generic parameter `T` without required bound `Addable`"
        ),
        "{error}"
    );
}

#[test]
fn rejects_result_map_higher_order_specialization_method_call_without_required_bound() {
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

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn result_map<T, R, E>(result: Result<T, E>, mapper: Fn1<T, R>) -> Result<R, E> {
            match result {
              Result.Ok(value) => {
                let mapped = mapper(value);
                return Result.Ok(mapped.add(mapped));
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          fn bump<T, E>(input: Result<T, E>) -> Result<T, E> {
            return result_map(input, |x: T| -> T { return x; });
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("function `result_map` body higher-order specialization body match-arm"),
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
fn rejects_result_and_then_higher_order_specialization_method_call_without_required_bound() {
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

          enum Result<T, E> {
            Ok(T),
            Err(E),
          }

          fn result_and_then<T, R, E>(
            result: Result<T, E>,
            mapper: Fn1<T, Result<R, E>>
          ) -> Result<R, E> {
            match result {
              Result.Ok(value) => {
                let mapped = mapper(value);
                match mapped {
                  Result.Ok(inner) => {
                    return Result.Ok(inner.add(inner));
                  }
                  Result.Err(error) => {
                    return Result.Err(error);
                  }
                }
              }
              Result.Err(error) => {
                return Result.Err(error);
              }
            }
          }

          fn bump<T, E>(input: Result<T, E>) -> Result<T, E> {
            return result_and_then(input, |x: T| -> Result<T, E> { return Result.Ok(x); });
          }

          fn main() -> i64 {
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("function `result_and_then` body higher-order specialization body"),
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
fn lowers_generic_lambda_method_call_with_present_bound() {
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

          fn apply<T: Addable>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn bump<T: Addable>(value: T) -> T {
            return apply(value, |x: T| -> T { return x.add(x); });
          }

          fn main() -> i64 {
            return bump(2);
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee.starts_with("__lambda_bump_")
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_bump_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Addable.for.i64.add"
                && matches!(args.as_slice(), [NirExpr::Var(lhs), NirExpr::Var(rhs)] if lhs == "x" && rhs == "x")
    ));
}

#[test]
fn lowers_capturing_generic_lambda_method_call_with_present_bound() {
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

          fn apply<T: Addable>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn bump<T: Addable>(value: T, extra: T) -> T {
            return apply(value, |x: T| -> T { return x.add(extra); });
          }

          fn main() -> i64 {
            return bump(2, 3);
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_bump_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(x), NirExpr::Var(extra)] if x == "x" && extra == "__capture_f_extra_0"
                )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_bump_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    assert_eq!(lambda.params.len(), 2);
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Addable.for.i64.add"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));
}

#[test]
fn lowers_higher_order_generic_lambda_with_qualified_helper_trait_bound() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn apply<T: Helper.Addable>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn bump<T: Helper.Addable>(value: T, extra: T) -> T {
            return apply(value, |x: T| -> T { return x.add(extra); });
          }

          fn main() -> i64 {
            return bump(2, 3);
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

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected helper-trait monomorphized higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_bump_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(x), NirExpr::Var(extra)]
                        if x == "x" && extra == "__capture_f_extra_0"
                )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_bump_") && function.name.ends_with("__i64")
        })
        .expect("expected helper-trait monomorphized captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Helper.Addable.for.i64.add"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));
}

#[test]
fn lowers_higher_order_generic_lambda_with_qualified_helper_trait_bound_through_alias_chain() {
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

          fn apply<T: Helper.Addable>(x: Outer<T>, f: Fn1<Outer<T>, T>) -> T {
            return f(x);
          }

          fn bump<T: Helper.Addable>(value: Outer<T>, extra: Outer<T>) -> T {
            return apply(value, |x: Outer<T>| -> T { return x.add(extra); });
          }

          fn main() -> i64 {
            return bump(2, 3);
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

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected helper-trait alias-chain monomorphized higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_bump_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(x), NirExpr::Var(extra)]
                        if x == "x" && extra == "__capture_f_extra_0"
                )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_bump_") && function.name.ends_with("__i64")
        })
        .expect(
            "expected helper-trait alias-chain monomorphized captured generic lambda specialization",
        );
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Helper.Addable.for.i64.add"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));
}

#[test]
fn lowers_higher_order_lambda_returning_outer_literal_with_deferred_inner_inference() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          fn apply<T, R>(value: T, mapper: Fn1<T, R>) -> R {
            return mapper(value);
          }

          fn main() -> i64 {
            let outer: Outer<i64, String> =
              apply(7, |x: i64| -> Outer<i64, String> {
                return Outer {
                  inner: Phantom { value: x, tag: 1 },
                  meta: "ok",
                };
              });
            return outer.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    let hof = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_"))
        .expect("expected monomorphized higher-order helper");
    assert!(hof.generic_params.is_empty());
    assert!(matches!(
        hof.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Outer<i64, String>"
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized lambda");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Outer<i64, String>"
    ));
}

#[test]
fn lowers_higher_order_lambda_without_explicit_return_type_returning_outer_literal_with_deferred_inner_inference(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          fn apply<T, R>(value: T, mapper: Fn1<T, R>) -> R {
            return mapper(value);
          }

          fn main() -> i64 {
            let outer: Outer<i64, String> =
              apply(7, |x: i64| {
                return Outer {
                  inner: Phantom { value: x, tag: 1 },
                  meta: "ok",
                };
              });
            return outer.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    let hof = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_"))
        .expect("expected monomorphized higher-order helper");
    assert!(hof.generic_params.is_empty());
    assert!(matches!(
        hof.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Outer<i64, String>"
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized lambda");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Outer<i64, String>"
    ));
}

#[test]
fn lowers_higher_order_lambda_without_explicit_return_type_inside_if_result_context() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          fn apply<T, R>(value: T, mapper: Fn1<T, R>) -> R {
            return mapper(value);
          }

          fn main() -> i64 {
            let outer: Outer<i64, String> = if true {
              apply(7, |x: i64| {
                return Outer {
                  inner: Phantom { value: x, tag: 1 },
                  meta: "ok",
                };
              })
            } else {
              apply(8, |x: i64| {
                return Outer {
                  inner: Phantom { value: x, tag: 2 },
                  meta: "fallback",
                };
              })
            };
            return outer.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    let lambda_count = module
        .functions
        .iter()
        .filter(|function| function.name.starts_with("__lambda_main_"))
        .count();
    assert_eq!(
        lambda_count, 2,
        "expected one synthesized lambda per if branch"
    );

    let outer_helpers = module
        .functions
        .iter()
        .filter(|function| {
            function.name.starts_with("__hof_apply_")
                && matches!(
                    function.return_type.as_ref().map(|ty| ty.render()),
                    Some(rendered) if rendered == "Outer<i64, String>"
                )
        })
        .count();
    assert_eq!(
        outer_helpers, 2,
        "expected both if branches to specialize apply to Outer<i64, String>"
    );
}

#[test]
fn lowers_higher_order_lambda_without_explicit_return_type_inside_match_result_context() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          fn apply<T, R>(value: T, mapper: Fn1<T, R>) -> R {
            return mapper(value);
          }

          fn main() -> i64 {
            let outer: Outer<i64, String> = match 1 {
              1 => {
                apply(7, |x: i64| {
                  return Outer {
                    inner: Phantom { value: x, tag: 1 },
                    meta: "ok",
                  };
                })
              },
              _ => {
                apply(8, |x: i64| {
                  return Outer {
                    inner: Phantom { value: x, tag: 2 },
                    meta: "fallback",
                  };
                })
              }
            };
            return outer.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    let lambda_count = module
        .functions
        .iter()
        .filter(|function| function.name.starts_with("__lambda_main_"))
        .count();
    assert_eq!(
        lambda_count, 2,
        "expected one synthesized lambda per match arm"
    );

    let outer_helpers = module
        .functions
        .iter()
        .filter(|function| {
            function.name.starts_with("__hof_apply_")
                && matches!(
                    function.return_type.as_ref().map(|ty| ty.render()),
                    Some(rendered) if rendered == "Outer<i64, String>"
                )
        })
        .count();
    assert_eq!(
        outer_helpers, 2,
        "expected both match arms to specialize apply to Outer<i64, String>"
    );
}

#[test]
fn lowers_higher_order_lambda_returning_alias_outer_literal_with_deferred_inner_inference() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type OuterAlias<T, U> = Outer<T, U>;

          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          fn apply<T, R>(value: T, mapper: Fn1<T, R>) -> R {
            return mapper(value);
          }

          fn main() -> i64 {
            let outer: OuterAlias<i64, String> =
              apply(7, |x: i64| -> OuterAlias<i64, String> {
                return OuterAlias {
                  inner: Phantom { value: x, tag: 1 },
                  meta: "ok",
                };
              });
            return outer.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    let hof = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_"))
        .expect("expected monomorphized higher-order helper");
    assert!(hof.generic_params.is_empty());
    assert!(matches!(
        hof.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Outer<i64, String>"
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized lambda");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Outer<i64, String>"
    ));
}

#[test]
fn lowers_method_call_lambda_without_explicit_return_type_returning_outer_literal() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Runner {
            fn apply(host: Self, value: i64, mapper: Fn1<i64, Outer<i64, String>>) -> Outer<i64, String>;
          }

          struct Host {}

          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          impl Runner for Host {
            fn apply(host: Host, value: i64, mapper: Fn1<i64, Outer<i64, String>>) -> Outer<i64, String> {
              return mapper(value);
            }
          }

          fn main() -> i64 {
            let host: Host = Host {};
            let outer: Outer<i64, String> = host.apply(7, |x: i64| {
              return Outer {
                inner: Phantom { value: x, tag: 1 },
                meta: "ok",
              };
            });
            return outer.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized method-call lambda");
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Outer<i64, String>"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function
                .name
                .starts_with("__hof_impl_Runner_for_Host_apply")
        })
        .expect("expected specialized higher-order impl method helper");
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Outer<i64, String>"
    ));
}

#[test]
fn lowers_generic_impl_method_call_lambda_for_concrete_receiver() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Runner {
            fn apply(host: Self, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64>;
          }

          struct Box<T> {
            value: T,
          }

          struct Pair<T> {
            left: T,
            right: T,
          }

          impl<T> Runner for Box<T> {
            fn apply(host: Box<T>, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64> {
              return mapper(value);
            }
          }

          fn main() -> i64 {
            let host: Box<i64> = Box { value: 3 };
            let pair: Pair<i64> = host.apply(7, |x: i64| {
              return Pair { left: x, right: x + 1 };
            });
            return pair.right;
          }
        }
        "#,
    )
    .unwrap();

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .expect("expected synthesized generic method-call lambda");
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Pair<i64>"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_impl_Runner_for_Box")
                && function.name.contains("apply")
        })
        .expect("expected specialized higher-order generic impl method helper");
    assert!(specialized.generic_params.is_empty());
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Pair<i64>"
    ));
}

#[test]
fn lowers_method_call_lambda_when_receiver_comes_from_if_expr() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Runner {
            fn apply(host: Self, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64>;
          }

          struct Host {}

          struct Pair<T> {
            left: T,
            right: T,
          }

          impl Runner for Host {
            fn apply(host: Host, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64> {
              return mapper(value);
            }
          }

          fn main() -> i64 {
            let pick_left = true;
            let host = if pick_left {
              Host {}
            } else {
              Host {}
            };
            let pair = host.apply(7, |x: i64| {
              return Pair { left: x, right: x + 1 };
            });
            return pair.right;
          }
        }
        "#,
    )
    .unwrap();

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function
                .name
                .starts_with("__hof_impl_Runner_for_Host_apply")
        })
        .expect("expected specialized higher-order impl helper for if receiver");
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Pair<i64>"
    ));
}

#[test]
fn lowers_method_call_lambda_when_receiver_comes_from_match_expr() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Runner {
            fn apply(host: Self, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64>;
          }

          struct Host {}

          struct Pair<T> {
            left: T,
            right: T,
          }

          impl Runner for Host {
            fn apply(host: Host, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64> {
              return mapper(value);
            }
          }

          fn main() -> i64 {
            let choice: i64 = 1;
            let host = match choice {
              1 => {
                Host {}
              }
              _ => {
                Host {}
              }
            };
            let pair = host.apply(7, |x: i64| {
              return Pair { left: x, right: x + 1 };
            });
            return pair.right;
          }
        }
        "#,
    )
    .unwrap();

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function
                .name
                .starts_with("__hof_impl_Runner_for_Host_apply")
        })
        .expect("expected specialized higher-order impl helper for match receiver");
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Pair<i64>"
    ));
}

#[test]
fn lowers_method_call_lambda_when_receiver_comes_from_method_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Runner {
            fn id(host: Self) -> Self;
            fn apply(host: Self, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64>;
          }

          struct Host {}

          struct Pair<T> {
            left: T,
            right: T,
          }

          impl Runner for Host {
            fn id(host: Host) -> Host {
              return host;
            }

            fn apply(host: Host, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64> {
              return mapper(value);
            }
          }

          fn main() -> i64 {
            let host: Host = Host {};
            let pair = host.id().apply(7, |x: i64| {
              return Pair { left: x, right: x + 1 };
            });
            return pair.right;
          }
        }
        "#,
    )
    .unwrap();

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function
                .name
                .starts_with("__hof_impl_Runner_for_Host_apply")
        })
        .expect("expected specialized higher-order impl helper for chained receiver");
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Pair<i64>"
    ));
}

#[test]
fn lowers_method_call_lambda_when_receiver_comes_from_struct_field() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Runner {
            fn apply(host: Self, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64>;
          }

          struct Host {}

          struct State {
            host: Host,
          }

          struct Pair<T> {
            left: T,
            right: T,
          }

          impl Runner for Host {
            fn apply(host: Host, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64> {
              return mapper(value);
            }
          }

          fn main() -> i64 {
            let state = State { host: Host {} };
            let pair = state.host.apply(7, |x: i64| {
              return Pair { left: x, right: x + 1 };
            });
            return pair.right;
          }
        }
        "#,
    )
    .unwrap();

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function
                .name
                .starts_with("__hof_impl_Runner_for_Host_apply")
        })
        .expect("expected specialized higher-order impl helper for field receiver");
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Pair<i64>"
    ));
}

#[test]
fn lowers_method_call_lambda_when_typed_receiver_comes_from_struct_field() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Runner {
            fn apply(host: Self, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64>;
          }

          struct Host {}

          struct State {
            host: Host,
          }

          struct Pair<T> {
            left: T,
            right: T,
          }

          impl Runner for Host {
            fn apply(host: Host, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64> {
              return mapper(value);
            }
          }

          fn main() -> i64 {
            let state: State = State { host: Host {} };
            let pair = state.host.apply(7, |x: i64| {
              return Pair { left: x, right: x + 1 };
            });
            return pair.right;
          }
        }
        "#,
    )
    .unwrap();

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function
                .name
                .starts_with("__hof_impl_Runner_for_Host_apply")
        })
        .expect("expected specialized higher-order impl helper for typed field receiver");
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Pair<i64>"
    ));
}

#[test]
fn lowers_method_call_lambda_when_receiver_comes_from_nested_struct_field() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Runner {
            fn apply(host: Self, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64>;
          }

          struct Host {}

          struct Inner {
            host: Host,
          }

          struct State {
            inner: Inner,
          }

          struct Pair<T> {
            left: T,
            right: T,
          }

          impl Runner for Host {
            fn apply(host: Host, value: i64, mapper: Fn1<i64, Pair<i64>>) -> Pair<i64> {
              return mapper(value);
            }
          }

          fn main() -> i64 {
            let state: State = State { inner: Inner { host: Host {} } };
            let pair = state.inner.host.apply(7, |x: i64| {
              return Pair { left: x, right: x + 1 };
            });
            return pair.right;
          }
        }
        "#,
    )
    .unwrap();

    let specialized = module
        .functions
        .iter()
        .find(|function| {
            function
                .name
                .starts_with("__hof_impl_Runner_for_Host_apply")
        })
        .expect("expected specialized higher-order impl helper for nested field receiver");
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Pair<i64>"
    ));
}

#[test]
fn lowers_capturing_generic_lambda_explicit_trait_call_with_present_bound() {
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

          fn apply<T: Addable>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn bump<T: Addable>(value: T, extra: T) -> T {
            return apply(value, |x: T| -> T { return Addable.add(x, extra); });
          }

          fn main() -> i64 {
            return bump(2, 3);
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_bump_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(x), NirExpr::Var(extra)] if x == "x" && extra == "__capture_f_extra_0"
                )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_bump_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized explicit-trait captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Addable.for.i64.add"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));
}

#[test]
fn lowers_capturing_generic_lambda_operator_call_with_present_bound() {
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

          fn apply<T: Addable>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn bump<T: Addable>(value: T, extra: T) -> T {
            return apply(value, |x: T| -> T { return x + extra; });
          }

          fn main() -> i64 {
            return bump(2, 3);
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_bump_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(x), NirExpr::Var(extra)] if x == "x" && extra == "__capture_f_extra_0"
                )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_bump_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized operator captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Binary { lhs, rhs, .. }))]
            if matches!(lhs.as_ref(), NirExpr::Var(name) if name == "x")
                && matches!(rhs.as_ref(), NirExpr::Var(name) if name == &capture_param_name)
    ));
}

#[test]
fn lowers_capturing_generic_lambda_equality_operator_with_present_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Equatable {
            fn eq(lhs: Self, rhs: Self) -> bool;
          }

          impl Equatable for Pair {
            fn eq(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value == rhs.value;
            }
          }

          fn apply<T: Equatable>(x: T, f: Fn1<T, bool>) -> bool {
            return f(x);
          }

          fn same<T: Equatable>(value: T, other: T) -> bool {
            return apply(value, |x: T| -> bool { return x == other; });
          }

          fn main() -> i64 {
            let lhs: Pair = Pair { value: 2 };
            let rhs: Pair = Pair { value: 2 };
            if same(lhs, rhs) {
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized bool-returning higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_same_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(x), NirExpr::Var(other)]
                        if x == "x" && other == "__capture_f_other_0"
                )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_same_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized equality captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    assert_eq!(lambda.params.len(), 2);
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Equatable.for.Pair.eq"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));
}

#[test]
fn lowers_generic_lambda_unary_neg_with_present_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Negatable {
            fn neg(value: Self) -> Self;
          }

          impl Negatable for Pair {
            fn neg(value: Pair) -> Pair {
              return Pair { value: 0 - value.value };
            }
          }

          fn apply<T: Negatable>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn flip<T: Negatable>(value: T) -> T {
            return apply(value, |x: T| -> T { return -x; });
          }

          fn main() -> i64 {
            return flip(Pair { value: 7 }).value;
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized unary-neg higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_flip_")
                && matches!(args.as_slice(), [NirExpr::Var(x)] if x == "x")
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_flip_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized unary-neg generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Negatable.for.Pair.neg"
                && matches!(args.as_slice(), [NirExpr::Var(value)] if value == "x")
    ));
}

#[test]
fn lowers_capturing_generic_lambda_inequality_operator_with_present_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Equatable {
            fn eq(lhs: Self, rhs: Self) -> bool;
          }

          impl Equatable for Pair {
            fn eq(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value == rhs.value;
            }
          }

          fn apply<T: Equatable>(x: T, f: Fn1<T, bool>) -> bool {
            return f(x);
          }

          fn different<T: Equatable>(value: T, other: T) -> bool {
            return apply(value, |x: T| -> bool { return x != other; });
          }

          fn main() -> i64 {
            let lhs: Pair = Pair { value: 2 };
            let rhs: Pair = Pair { value: 3 };
            if different(lhs, rhs) {
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized bool-returning inequality helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_different_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(x), NirExpr::Var(other)]
                        if x == "x" && other == "__capture_f_other_0"
                )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_different_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized inequality captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Binary { lhs, rhs, .. }))]
            if matches!(
                lhs.as_ref(),
                NirExpr::Call { callee, args }
                    if callee == "impl.Equatable.for.Pair.eq"
                        && matches!(
                            args.as_slice(),
                            [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                                if lhs == "x" && rhs == &capture_param_name
                        )
            ) && matches!(rhs.as_ref(), NirExpr::Bool(false))
    ));
}

#[test]
fn lowers_capturing_generic_lambda_ordering_operator_with_present_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Orderable {
            fn lt(lhs: Self, rhs: Self) -> bool;
            fn le(lhs: Self, rhs: Self) -> bool;
            fn gt(lhs: Self, rhs: Self) -> bool;
            fn ge(lhs: Self, rhs: Self) -> bool;
          }

          impl Orderable for Pair {
            fn lt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value < rhs.value;
            }

            fn le(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value <= rhs.value;
            }

            fn gt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value > rhs.value;
            }

            fn ge(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value >= rhs.value;
            }
          }

          fn apply<T: Orderable>(x: T, f: Fn1<T, bool>) -> bool {
            return f(x);
          }

          fn less<T: Orderable>(value: T, other: T) -> bool {
            return apply(value, |x: T| -> bool { return x < other; });
          }

          fn main() -> i64 {
            let lhs: Pair = Pair { value: 2 };
            let rhs: Pair = Pair { value: 3 };
            if less(lhs, rhs) {
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized bool-returning ordering helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_less_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(x), NirExpr::Var(other)]
                        if x == "x" && other == "__capture_f_other_0"
                )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_less_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized ordering captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Orderable.for.Pair.lt"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));
}

#[test]
fn lowers_capturing_generic_lambda_ordering_le_operator_with_present_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Orderable {
            fn lt(lhs: Self, rhs: Self) -> bool;
            fn le(lhs: Self, rhs: Self) -> bool;
            fn gt(lhs: Self, rhs: Self) -> bool;
            fn ge(lhs: Self, rhs: Self) -> bool;
          }

          impl Orderable for Pair {
            fn lt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value < rhs.value;
            }

            fn le(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value <= rhs.value;
            }

            fn gt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value > rhs.value;
            }

            fn ge(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value >= rhs.value;
            }
          }

          fn apply<T: Orderable>(x: T, f: Fn1<T, bool>) -> bool {
            return f(x);
          }

          fn less_eq<T: Orderable>(value: T, other: T) -> bool {
            return apply(value, |x: T| -> bool { return x <= other; });
          }

          fn main() -> i64 {
            let lhs: Pair = Pair { value: 2 };
            let rhs: Pair = Pair { value: 3 };
            if less_eq(lhs, rhs) {
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized bool-returning ordering <= helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_less_eq_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(x), NirExpr::Var(other)]
                        if x == "x" && other == "__capture_f_other_0"
                )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_less_eq_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized ordering <= captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Orderable.for.Pair.le"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));
}

#[test]
fn lowers_capturing_generic_lambda_ordering_gt_operator_with_present_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Orderable {
            fn lt(lhs: Self, rhs: Self) -> bool;
            fn le(lhs: Self, rhs: Self) -> bool;
            fn gt(lhs: Self, rhs: Self) -> bool;
            fn ge(lhs: Self, rhs: Self) -> bool;
          }

          impl Orderable for Pair {
            fn lt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value < rhs.value;
            }

            fn le(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value <= rhs.value;
            }

            fn gt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value > rhs.value;
            }

            fn ge(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value >= rhs.value;
            }
          }

          fn apply<T: Orderable>(x: T, f: Fn1<T, bool>) -> bool {
            return f(x);
          }

          fn greater<T: Orderable>(value: T, other: T) -> bool {
            return apply(value, |x: T| -> bool { return x > other; });
          }

          fn main() -> i64 {
            let lhs: Pair = Pair { value: 3 };
            let rhs: Pair = Pair { value: 2 };
            if greater(lhs, rhs) {
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized bool-returning ordering > helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_greater_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(x), NirExpr::Var(other)]
                        if x == "x" && other == "__capture_f_other_0"
                )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_greater_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized ordering > captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Orderable.for.Pair.gt"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));
}

#[test]
fn lowers_capturing_generic_lambda_ordering_ge_operator_with_present_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Orderable {
            fn lt(lhs: Self, rhs: Self) -> bool;
            fn le(lhs: Self, rhs: Self) -> bool;
            fn gt(lhs: Self, rhs: Self) -> bool;
            fn ge(lhs: Self, rhs: Self) -> bool;
          }

          impl Orderable for Pair {
            fn lt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value < rhs.value;
            }

            fn le(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value <= rhs.value;
            }

            fn gt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value > rhs.value;
            }

            fn ge(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value >= rhs.value;
            }
          }

          fn apply<T: Orderable>(x: T, f: Fn1<T, bool>) -> bool {
            return f(x);
          }

          fn greater_eq<T: Orderable>(value: T, other: T) -> bool {
            return apply(value, |x: T| -> bool { return x >= other; });
          }

          fn main() -> i64 {
            let lhs: Pair = Pair { value: 3 };
            let rhs: Pair = Pair { value: 2 };
            if greater_eq(lhs, rhs) {
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized bool-returning ordering >= helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert_eq!(higher_order_concrete.params.len(), 2);
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_greater_eq_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(x), NirExpr::Var(other)]
                        if x == "x" && other == "__capture_f_other_0"
                )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_greater_eq_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized ordering >= captured generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Orderable.for.Pair.ge"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));
}

#[test]
fn lowers_generic_lambda_unary_not_with_present_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Notable {
            fn not(value: Self) -> bool;
          }

          impl Notable for Pair {
            fn not(value: Pair) -> bool {
              return value.value == 0;
            }
          }

          fn apply<T: Notable>(x: T, f: Fn1<T, bool>) -> bool {
            return f(x);
          }

          fn empty<T: Notable>(value: T) -> bool {
            return apply(value, |x: T| -> bool { return !x; });
          }

          fn main() -> i64 {
            let value: Pair = Pair { value: 0 };
            if empty(value) {
              return 1;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized unary-not higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_empty_")
                && matches!(args.as_slice(), [NirExpr::Var(x)] if x == "x")
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_empty_") && function.name.ends_with("__Pair")
        })
        .expect("expected monomorphized unary-not generic lambda specialization");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Notable.for.Pair.not"
                && matches!(args.as_slice(), [NirExpr::Var(value)] if value == "x")
    ));
}

#[test]
fn lowers_capturing_generic_lambda_with_bound_inside_nested_while_match() {
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

          fn apply<T: Addable>(x: T, f: Fn1<T, T>) -> T {
            return f(x);
          }

          fn choose<T: Addable>(value: T, extra: T, mode: i64) -> T {
            while mode > 0 {
              match mode {
                1 => {
                  return apply(value, |x: T| -> T { return x.add(extra); });
                }
                _ => {
                  return value;
                }
              }
            }
            return value;
          }

          fn main() -> i64 {
            return choose(2, 3, 1);
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized higher-order helper inside nested control flow");
    assert!(helper.generic_params.is_empty());
    assert_eq!(helper.params.len(), 2);
    assert!(matches!(
        helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee.starts_with("__lambda_choose_")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(x), NirExpr::Var(extra)] if x == "x" && extra == "__capture_f_extra_0"
                )
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_choose_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized captured lambda inside nested control flow");
    assert!(lambda.generic_params.is_empty());
    let capture_param_name = lambda.params[1].name.clone();
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Addable.for.i64.add"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Var(lhs), NirExpr::Var(rhs)]
                        if lhs == "x" && rhs == &capture_param_name
                )
    ));

    let choose = module
        .functions
        .iter()
        .find(|function| function.name == "choose__i64")
        .expect("expected monomorphized control-flow generic function");
    assert!(matches!(
        choose.body.first(),
        Some(NirStmt::While { body, .. })
            if matches!(
                body.as_slice(),
                [NirStmt::If { then_body, else_body, .. }]
                    if matches!(
                        then_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                            if callee == &helper.name
                                && matches!(
                                    args.as_slice(),
                                    [NirExpr::Var(value), NirExpr::Var(extra)]
                                        if value == "value" && extra == "extra"
                                )
                    ) && matches!(
                        else_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Var(name)))] if name == "value"
                    )
            )
    ));
}

#[test]
fn lowers_generic_fn2_lambda_method_call_with_present_bound() {
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

          fn apply2<T: Addable>(x: T, y: T, f: Fn2<T, T, T>) -> T {
            return f(x, y);
          }

          fn bump<T: Addable>(lhs: T, rhs: T) -> T {
            return apply2(lhs, rhs, |x: T, y: T| -> T { return x.add(y); });
          }

          fn main() -> i64 {
            return bump(2, 3);
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply2_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized Fn2 higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee.starts_with("__lambda_bump_")
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_bump_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized generic Fn2 lambda specialization");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
            if callee == "impl.Addable.for.i64.add"
                && matches!(args.as_slice(), [NirExpr::Var(lhs), NirExpr::Var(rhs)] if lhs == "x" && rhs == "y")
    ));
}

#[test]
fn lowers_forwarded_fn2_callable_parameter_into_nested_higher_order_template_call() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn add_pair(lhs: i64, rhs: i64) -> i64 {
            return lhs + rhs;
          }

          fn apply2<T>(x: T, y: T, f: Fn2<T, T, T>) -> T {
            return f(x, y);
          }

          fn chain2(x: i64, y: i64, f: Fn2<i64, i64, i64>) -> i64 {
            return apply2(f(x, y), y, add_pair);
          }

          fn relay2(x: i64, y: i64, f: Fn2<i64, i64, i64>) -> i64 {
            return chain2(x, y, f);
          }

          fn main() -> i64 {
            return relay2(6, 2, |x: i64, y: i64| -> i64 { return x - y; });
          }
        }
        "#,
    )
    .unwrap();

    let relay_helper = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_relay2_"))
        .expect("expected forwarding Fn2 relay higher-order helper");
    assert!(relay_helper.generic_params.is_empty());
    assert!(matches!(
        relay_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee.starts_with("__hof_chain2_")
    ));

    let chain_helper = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_chain2_"))
        .expect("expected forwarded Fn2 chain higher-order helper");
    assert!(chain_helper.generic_params.is_empty());
    assert!(matches!(
        chain_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee.starts_with("__hof_apply2_")
    ));
}

#[test]
fn lowers_generic_fn3_lambda_method_call_with_present_bound() {
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

          fn apply3<T: Addable>(x: T, y: T, z: T, f: Fn3<T, T, T, T>) -> T {
            return f(x, y, z);
          }

          fn bump<T: Addable>(x: T, y: T, z: T) -> T {
            return apply3(x, y, z, |lhs: T, mid: T, rhs: T| -> T {
              return lhs.add(mid).add(rhs);
            });
          }

          fn main() -> i64 {
            return bump(2, 3, 4);
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply3_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized Fn3 higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());
    assert!(matches!(
        higher_order_concrete.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee.starts_with("__lambda_bump_")
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__lambda_bump_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized generic Fn3 lambda specialization");
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call {
            callee: outer_callee,
            args: outer_args
        }))] if outer_callee == "impl.Addable.for.i64.add"
            && matches!(
                outer_args.as_slice(),
                [
                    NirExpr::Call { callee: inner_callee, args: inner_args },
                    NirExpr::Var(rhs)
                ] if inner_callee == "impl.Addable.for.i64.add"
                    && matches!(inner_args.as_slice(), [NirExpr::Var(lhs), NirExpr::Var(mid)] if lhs == "lhs" && mid == "mid")
                    && rhs == "rhs"
            )
    ));
}

#[test]
fn lowers_forwarded_fn3_callable_parameter_into_nested_higher_order_template_call() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn add_three(lhs: i64, mid: i64, rhs: i64) -> i64 {
            return lhs + mid + rhs;
          }

          fn apply3<T>(x: T, y: T, z: T, f: Fn3<T, T, T, T>) -> T {
            return f(x, y, z);
          }

          fn chain3(x: i64, y: i64, z: i64, f: Fn3<i64, i64, i64, i64>) -> i64 {
            return apply3(f(x, y, z), y, z, add_three);
          }

          fn relay3(x: i64, y: i64, z: i64, f: Fn3<i64, i64, i64, i64>) -> i64 {
            return chain3(x, y, z, f);
          }

          fn main() -> i64 {
            return relay3(6, 2, 1, |x: i64, y: i64, z: i64| -> i64 { return x - y - z; });
          }
        }
        "#,
    )
    .unwrap();

    let relay_helper = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_relay3_"))
        .expect("expected forwarding Fn3 relay higher-order helper");
    assert!(relay_helper.generic_params.is_empty());
    assert!(matches!(
        relay_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee.starts_with("__hof_chain3_")
    ));

    let chain_helper = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_chain3_"))
        .expect("expected forwarded Fn3 chain higher-order helper");
    assert!(chain_helper.generic_params.is_empty());
    assert!(matches!(
        chain_helper.body.as_slice(),
        [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
            if callee.starts_with("__hof_apply3_")
    ));
}

#[test]
fn lowers_higher_order_call_scrutinee_match_inside_while_via_hoisted_let() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn apply(x: i64, f: Fn1<i64, i64>) -> i64 {
            return f(x);
          }

          fn main() -> i64 {
            let state: i64 = 2;
            while state > 0 {
              match apply(state, |x: i64| -> i64 { return x + 1; }) {
                3 => { return 7; },
                _ => { return 9; }
              }
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    match &function.body[1] {
        NirStmt::While { body, .. } => {
            assert!(matches!(
                body.as_slice(),
                [
                    NirStmt::Let { name, value: NirExpr::Call { .. }, .. },
                    NirStmt::If { .. }
                ] if name.starts_with("__match_scrutinee_")
            ));
        }
        other => panic!("expected while statement, found {other:?}"),
    }
}

#[test]
fn lowers_generic_fn3_higher_order_lambda_family() {
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

          fn apply3<T: Addable>(x: T, y: T, z: T, f: Fn3<T, T, T, T>) -> T {
            return f(x, y, z);
          }

          fn main() -> i64 {
            return apply3(5, 1, 1, |x: i64, y: i64, z: i64| -> i64 { return x + y + z; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply3_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized Fn3 higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &higher_order_concrete.name
    ));
}

#[test]
fn lowers_generic_fn3_alias_higher_order_lambda_family() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Reducer<T> = Fn3<T, T, T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn apply3<T: Addable>(x: T, y: T, z: T, f: Reducer<T>) -> T {
            return f(x, y, z);
          }

          fn main() -> i64 {
            return apply3(5, 1, 1, |x: i64, y: i64, z: i64| -> i64 { return x + y + z; });
          }
        }
        "#,
    )
    .unwrap();

    let higher_order_concrete = module
        .functions
        .iter()
        .find(|function| {
            function.name.starts_with("__hof_apply3_") && function.name.ends_with("__i64")
        })
        .expect("expected monomorphized Fn3 alias higher-order helper");
    assert!(higher_order_concrete.generic_params.is_empty());

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == &higher_order_concrete.name
    ));
}
