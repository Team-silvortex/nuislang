use std::collections::BTreeMap;

use nuis_semantics::model::NirExpr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TaskResultStateFact {
    Completed,
    TimedOut,
    Cancelled,
    NotCompleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BorrowedAddressBinding {
    pub(super) source: String,
    pub(super) via_traversal: bool,
}

impl BorrowedAddressBinding {
    pub(super) fn direct(source: String) -> Self {
        Self {
            source,
            via_traversal: false,
        }
    }

    pub(super) fn traversed(source: String) -> Self {
        Self {
            source,
            via_traversal: true,
        }
    }
}

pub(super) type BorrowBindings = BTreeMap<String, BorrowedAddressBinding>;

fn task_result_condition_fact(expr: &NirExpr) -> Option<(String, TaskResultStateFact)> {
    let (inner, fact) = match expr {
        NirExpr::CpuTaskCompleted(inner) => (inner.as_ref(), TaskResultStateFact::Completed),
        NirExpr::CpuTaskTimedOut(inner) => (inner.as_ref(), TaskResultStateFact::TimedOut),
        NirExpr::CpuTaskCancelled(inner) => (inner.as_ref(), TaskResultStateFact::Cancelled),
        _ => return None,
    };
    super::expr_resource_key(inner).map(|name| (name, fact))
}

fn apply_direct_truth_fact(
    condition: &NirExpr,
    facts: &mut BTreeMap<String, TaskResultStateFact>,
) -> bool {
    if let Some((name, fact)) = task_result_condition_fact(condition) {
        facts.insert(name, fact);
        true
    } else {
        false
    }
}

fn apply_direct_false_fact(
    condition: &NirExpr,
    facts: &mut BTreeMap<String, TaskResultStateFact>,
) -> bool {
    if let Some((name, fact)) = task_result_condition_fact(condition) {
        match fact {
            TaskResultStateFact::Completed => {
                facts.insert(name, TaskResultStateFact::NotCompleted);
            }
            TaskResultStateFact::TimedOut
            | TaskResultStateFact::Cancelled
            | TaskResultStateFact::NotCompleted => {}
        }
        true
    } else {
        false
    }
}

pub(super) fn apply_truthy_task_result_condition_facts(
    condition: &NirExpr,
    facts: &mut BTreeMap<String, TaskResultStateFact>,
) {
    if apply_direct_truth_fact(condition, facts) {
        return;
    }
    if let NirExpr::Binary { op, lhs, rhs } = condition {
        match op {
            nuis_semantics::model::NirBinaryOp::And => {
                apply_truthy_task_result_condition_facts(lhs, facts);
                apply_truthy_task_result_condition_facts(rhs, facts);
            }
            nuis_semantics::model::NirBinaryOp::Or => {}
            _ => {}
        }
    }
}

pub(super) fn apply_falsy_task_result_condition_facts(
    condition: &NirExpr,
    facts: &mut BTreeMap<String, TaskResultStateFact>,
) {
    if apply_direct_false_fact(condition, facts) {
        return;
    }
    if let NirExpr::Binary { op, lhs, rhs } = condition {
        match op {
            nuis_semantics::model::NirBinaryOp::Or => {
                apply_falsy_task_result_condition_facts(lhs, facts);
                apply_falsy_task_result_condition_facts(rhs, facts);
            }
            nuis_semantics::model::NirBinaryOp::And => {}
            _ => {}
        }
    }
}

pub(super) fn apply_task_result_condition_facts(
    condition: &NirExpr,
    then_facts: &mut BTreeMap<String, TaskResultStateFact>,
    else_facts: &mut BTreeMap<String, TaskResultStateFact>,
) {
    apply_truthy_task_result_condition_facts(condition, then_facts);
    apply_falsy_task_result_condition_facts(condition, else_facts);
}

pub(super) fn task_result_facts_for_short_circuit_rhs(
    condition: &NirExpr,
    rhs_is_for_and: bool,
    facts: &BTreeMap<String, TaskResultStateFact>,
) -> BTreeMap<String, TaskResultStateFact> {
    let mut derived = facts.clone();
    if rhs_is_for_and {
        apply_truthy_task_result_condition_facts(condition, &mut derived);
    } else {
        apply_falsy_task_result_condition_facts(condition, &mut derived);
    }
    derived
}

pub(super) fn merge_control_flow_task_result_facts(
    task_result_facts: &mut BTreeMap<String, TaskResultStateFact>,
    then_facts: &BTreeMap<String, TaskResultStateFact>,
    else_facts: &BTreeMap<String, TaskResultStateFact>,
) {
    let mut merged = BTreeMap::new();
    for (name, then_fact) in then_facts {
        if let Some(else_fact) = else_facts.get(name) {
            if then_fact == else_fact {
                merged.insert(name.clone(), *then_fact);
            }
        }
    }
    *task_result_facts = merged;
}

pub(super) fn borrowed_address_binding(
    expr: &NirExpr,
    borrow_bindings: &BorrowBindings,
) -> Option<BorrowedAddressBinding> {
    match expr {
        NirExpr::Borrow(inner) => {
            super::expr_resource_key(inner).map(BorrowedAddressBinding::direct)
        }
        NirExpr::Var(name) => borrow_bindings.get(name).cloned(),
        NirExpr::LoadNext(inner) => borrowed_address_binding(inner, borrow_bindings)
            .map(|binding| BorrowedAddressBinding::traversed(binding.source)),
        NirExpr::Await(inner) => borrowed_address_binding(inner, borrow_bindings),
        _ => None,
    }
}

pub(super) fn borrowed_address_alias_source(
    expr: &NirExpr,
    borrow_bindings: &BorrowBindings,
) -> Option<String> {
    borrowed_address_binding(expr, borrow_bindings).map(|binding| binding.source)
}

pub(super) fn expr_is_borrowed_pointer(expr: &NirExpr, borrow_bindings: &BorrowBindings) -> bool {
    borrowed_address_alias_source(expr, borrow_bindings).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_next_from_borrow_alias_is_classified_as_traversal_binding() {
        let mut borrow_bindings = BorrowBindings::new();
        borrow_bindings.insert(
            "head_ref".to_owned(),
            BorrowedAddressBinding::direct("head".to_owned()),
        );
        let binding = borrowed_address_binding(
            &NirExpr::LoadNext(Box::new(NirExpr::Var("head_ref".to_owned()))),
            &borrow_bindings,
        )
        .expect("expected borrowed traversal binding");
        assert_eq!(binding.source, "head");
        assert!(binding.via_traversal);
    }

    #[test]
    fn load_next_from_owned_source_is_not_classified_as_borrowed_binding() {
        let borrow_bindings = BorrowBindings::new();
        assert!(borrowed_address_binding(
            &NirExpr::LoadNext(Box::new(NirExpr::Var("head".to_owned()))),
            &borrow_bindings,
        )
        .is_none());
    }
}
