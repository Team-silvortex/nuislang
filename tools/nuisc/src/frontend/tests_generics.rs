use super::lower_ast_to_nir;
use super::parse_nuis_ast;
use super::parse_nuis_module;
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

#[path = "tests_generics/async_shapes.rs"]
mod async_shapes;
#[path = "tests_generics/basic.rs"]
mod basic;
#[path = "tests_generics/branch_payload.rs"]
mod branch_payload;
#[path = "tests_generics/explicit_helpers.rs"]
mod explicit_helpers;
#[path = "tests_generics/higher_order_net.rs"]
mod higher_order_net;
#[path = "tests_generics/loop_branches.rs"]
mod loop_branches;
#[path = "tests_generics/nested_async.rs"]
mod nested_async;
#[path = "tests_generics/net_match_flow.rs"]
mod net_match_flow;
#[path = "tests_generics/net_nested_flow.rs"]
mod net_nested_flow;
#[path = "tests_generics/net_while_flow.rs"]
mod net_while_flow;
#[path = "tests_generics/network_session_demo.rs"]
mod network_session_demo;
#[path = "tests_generics/network_session_facade.rs"]
mod network_session_facade;
#[path = "tests_generics/network_session_summary.rs"]
mod network_session_summary;
#[path = "tests_generics/struct_inference.rs"]
mod struct_inference;
#[path = "tests_generics/thread_mutex.rs"]
mod thread_mutex;
#[path = "tests_generics/zero_arg_expectations.rs"]
mod zero_arg_expectations;
