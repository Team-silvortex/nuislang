use super::{lower_project_ast_to_nir, parse_nuis_ast, parse_nuis_module};
use nuis_semantics::model::{NirExpr, NirStmt};

fn is_payload_value_access_from_var(expr: &NirExpr, expected_base: &str) -> bool {
    match expr {
        NirExpr::FieldAccess { base, field } | NirExpr::VariantFieldAccess { base, field, .. } => {
            field == "value" && matches!(base.as_ref(), NirExpr::Var(name) if name == expected_base)
        }
        _ => false,
    }
}

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
        | NirExpr::VariantFieldAccess { base, .. }
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
        | NirExpr::CpuTaskFailed(base)
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

#[path = "tests_higher_order/baseline.rs"]
mod baseline;
#[path = "tests_higher_order/bounds.rs"]
mod bounds;
#[path = "tests_higher_order/fn23_tail.rs"]
mod fn23_tail;
#[path = "tests_higher_order/lambda_templates.rs"]
mod lambda_templates;
#[path = "tests_higher_order/method_receivers.rs"]
mod method_receivers;
#[path = "tests_higher_order/named_callables.rs"]
mod named_callables;
#[path = "tests_higher_order/nested_aliases.rs"]
mod nested_aliases;
#[path = "tests_higher_order/operator_bounds_a.rs"]
mod operator_bounds_a;
#[path = "tests_higher_order/operator_bounds_b.rs"]
mod operator_bounds_b;
#[path = "tests_higher_order/payload_async.rs"]
mod payload_async;
#[path = "tests_higher_order/result_zip.rs"]
mod result_zip;
#[path = "tests_higher_order/return_inference.rs"]
mod return_inference;
#[path = "tests_higher_order/try_await_inference.rs"]
mod try_await_inference;
