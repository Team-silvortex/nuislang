use std::collections::BTreeMap;

use nuis_semantics::model::NirExpr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TaskResultStateFact {
    Completed,
    TimedOut,
    Cancelled,
}

fn task_result_condition_fact(expr: &NirExpr) -> Option<(String, TaskResultStateFact)> {
    let (inner, fact) = match expr {
        NirExpr::CpuTaskCompleted(inner) => (inner.as_ref(), TaskResultStateFact::Completed),
        NirExpr::CpuTaskTimedOut(inner) => (inner.as_ref(), TaskResultStateFact::TimedOut),
        NirExpr::CpuTaskCancelled(inner) => (inner.as_ref(), TaskResultStateFact::Cancelled),
        _ => return None,
    };
    super::expr_resource_key(inner).map(|name| (name, fact))
}

pub(super) fn apply_task_result_condition_facts(
    condition: &NirExpr,
    then_facts: &mut BTreeMap<String, TaskResultStateFact>,
    else_facts: &mut BTreeMap<String, TaskResultStateFact>,
) {
    if let Some((name, fact)) = task_result_condition_fact(condition) {
        then_facts.insert(name.clone(), fact);
        match fact {
            TaskResultStateFact::Completed => {
                else_facts.remove(&name);
            }
            TaskResultStateFact::TimedOut | TaskResultStateFact::Cancelled => {}
        }
    }
}

pub(super) fn expr_is_borrowed_pointer(
    expr: &NirExpr,
    borrow_bindings: &BTreeMap<String, String>,
) -> bool {
    match expr {
        NirExpr::Borrow(_) => true,
        NirExpr::Var(name) => borrow_bindings.contains_key(name),
        NirExpr::Await(inner) => expr_is_borrowed_pointer(inner, borrow_bindings),
        _ => false,
    }
}
