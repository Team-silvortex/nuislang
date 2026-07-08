use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{NirExpr, NirStmt};

fn is_payload_value_access_from_var(expr: &NirExpr, expected_base: &str) -> bool {
    match expr {
        NirExpr::FieldAccess { base, field } | NirExpr::VariantFieldAccess { base, field, .. } => {
            field == "value" && matches!(base.as_ref(), NirExpr::Var(name) if name == expected_base)
        }
        _ => false,
    }
}

fn stmt_tree_contains_expr<F>(body: &[NirStmt], predicate: &F) -> bool
where
    F: Fn(&NirExpr) -> bool,
{
    body.iter().any(|stmt| stmt_contains_expr(stmt, predicate))
}

fn stmt_contains_expr<F>(stmt: &NirStmt, predicate: &F) -> bool
where
    F: Fn(&NirExpr) -> bool,
{
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Expr(value)
        | NirStmt::Await(value)
        | NirStmt::Print(value)
        | NirStmt::Return(Some(value)) => expr_contains_expr(value, predicate),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_contains_expr(condition, predicate)
                || stmt_tree_contains_expr(then_body, predicate)
                || stmt_tree_contains_expr(else_body, predicate)
        }
        NirStmt::While { condition, body } => {
            expr_contains_expr(condition, predicate) || stmt_tree_contains_expr(body, predicate)
        }
        NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => false,
    }
}

fn expr_contains_expr<F>(expr: &NirExpr, predicate: &F) -> bool
where
    F: Fn(&NirExpr) -> bool,
{
    if predicate(expr) {
        return true;
    }
    match expr {
        NirExpr::Call { args, .. } => args.iter().any(|arg| expr_contains_expr(arg, predicate)),
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_contains_expr(value, predicate)),
        NirExpr::FieldAccess { base, .. }
        | NirExpr::VariantFieldAccess { base, .. }
        | NirExpr::VariantIs { base, .. }
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
        | NirExpr::CpuMutexValue(base) => expr_contains_expr(base, predicate),
        NirExpr::Binary { lhs, rhs, .. } => {
            expr_contains_expr(lhs, predicate) || expr_contains_expr(rhs, predicate)
        }
        _ => false,
    }
}

fn contains_showable_call_from_awaited_try_payload(body: &[NirStmt]) -> bool {
    stmt_tree_contains_expr(body, &|expr| {
        matches!(
            expr,
            NirExpr::Call { callee, args }
                if callee.starts_with("impl.Showable.for.Carrier")
                    && callee.ends_with(".show__i64__bool")
                    && matches!(
                        args.as_slice(),
                        [NirExpr::FieldAccess { base, field }]
                            if field == "inner"
                                && matches!(
                                    base.as_ref(),
                                    NirExpr::FieldAccess { base: outer_base, field: outer_field }
                                        if outer_field == "outer"
                                            && matches!(
                                                outer_base.as_ref(),
                                                NirExpr::Await(value)
                                                    if matches!(
                                                        value.as_ref(),
                                                        NirExpr::Var(name) if name.starts_with("__nuis_try_payload_")
                                                    )
                                            )
                                )
                    )
        )
    })
}

fn contains_result_variant(body: &[NirStmt], variant: &str) -> bool {
    stmt_tree_contains_expr(
        body,
        &|expr| matches!(expr, NirExpr::StructLiteral { type_name, .. } if type_name == variant),
    )
}

fn contains_fetch_call<F>(body: &[NirStmt], arg_predicate: F) -> bool
where
    F: Fn(&[NirExpr]) -> bool,
{
    stmt_tree_contains_expr(
        body,
        &|expr| matches!(expr, NirExpr::Call { callee, args } if callee == "fetch" && arg_predicate(args)),
    )
}

fn assert_result_task_show_chain_semantics(body: &[NirStmt]) {
    assert!(contains_result_variant(body, "Result.Ok"));
    assert!(contains_result_variant(body, "Result.Err"));
    assert!(contains_showable_call_from_awaited_try_payload(body));
}

#[path = "tests_generic_structs/async_result_chain.rs"]
mod async_result_chain;
#[path = "tests_generic_structs/basic_literals.rs"]
mod basic_literals;
#[path = "tests_generic_structs/destructuring_alias.rs"]
mod destructuring_alias;
#[path = "tests_generic_structs/if_result_chain.rs"]
mod if_result_chain;
#[path = "tests_generic_structs/match_result_chain.rs"]
mod match_result_chain;
#[path = "tests_generic_structs/outer_inference.rs"]
mod outer_inference;
#[path = "tests_generic_structs/payload_constructors.rs"]
mod payload_constructors;
#[path = "tests_generic_structs/receiver_anchoring.rs"]
mod receiver_anchoring;
