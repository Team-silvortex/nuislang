use yir_core::Node;
use yir_domain_cpu::{LoopCondExpr, ParsedConditionalCarry};

use super::{linear_source, MAX_SCALAR_SLOTS};

pub(crate) fn is_conditional(node: &Node) -> bool {
    matches!(
        node.op.instruction.as_str(),
        "loop_while_i64_cond_chain" | "loop_while_scalar_cond_chain"
    )
}

pub(crate) fn parse(node: &Node) -> Result<Vec<ParsedConditionalCarry>, String> {
    let fail = |message: &str| format!("native conditional loop `{}` {message}", node.name);
    shape::validate(node)?;
    let carries = yir_domain_cpu::parse_conditional_carries(&node.op.args, 5, &node.name, true)?;
    if carries.is_empty() || carries.len() > MAX_SCALAR_SLOTS {
        return Err(fail("exceeds its flat conditional-carry shape/bound"));
    }
    for (index, carry) in carries.iter().enumerate() {
        let mut pending = vec![&carry.condition];
        while let Some(condition) = pending.pop() {
            match condition {
                LoopCondExpr::Leaf { kind, rhs } => {
                    if !condition_source(kind, index, carries.len())
                        || (kind == "always") != rhs.is_none()
                    {
                        return Err(fail("does not admit this conditional state source"));
                    }
                }
                LoopCondExpr::Binary { lhs, rhs, .. } => {
                    pending.push(rhs);
                    pending.push(lhs);
                }
            }
        }
        for branch in [&carry.then_source, &carry.else_source] {
            if !branch.payload.is_empty() || !branch_source(&branch.kind, index, carries.len()) {
                return Err(fail("does not admit this conditional carry source"));
            }
        }
    }
    Ok(carries)
}

fn branch_source(kind: &str, available: usize, count: usize) -> bool {
    matches!(kind, "keep" | "keep_prev_carry") || linear_source(kind, available, count)
}

mod shape;

fn condition_source(kind: &str, available: usize, count: usize) -> bool {
    if kind == "always" {
        return true;
    }
    let Some((source, compare)) = kind.rsplit_once('_') else {
        return false;
    };
    if !matches!(compare, "eq" | "ne" | "lt" | "le" | "gt" | "ge") {
        return false;
    }
    if matches!(source, "current" | "prev_current") {
        return true;
    }
    [("prev_carry", count), ("carry", available)]
        .into_iter()
        .any(|(prefix, bound)| {
            source
                .strip_prefix(prefix)
                .and_then(|index| index.parse::<usize>().ok())
                .is_some_and(|index| index < bound)
        })
}

#[cfg(test)]
mod tests;
