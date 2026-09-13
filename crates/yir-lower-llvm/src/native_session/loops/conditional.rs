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
    // Bound the shared recursive metadata parser before it sees untrusted input.
    // This slice has at most five words per carry, including an optional rhs.
    if node.op.args.len() < 9 || node.op.args.len() > 5 + 5 * MAX_SCALAR_SLOTS {
        return Err(fail("exceeds its flat conditional-carry shape/bound"));
    }
    let carries = yir_domain_cpu::parse_conditional_carries(&node.op.args, 5, &node.name, true)?;
    if carries.is_empty() || carries.len() > MAX_SCALAR_SLOTS {
        return Err(fail("exceeds its flat conditional-carry shape/bound"));
    }
    let mut cursor = 5;
    for (index, carry) in carries.iter().enumerate() {
        let LoopCondExpr::Leaf { kind, rhs } = &carry.condition else {
            return Err(fail("does not admit compound carry conditions"));
        };
        if !condition_source(kind, index, carries.len()) || (kind == "always") != rhs.is_none() {
            return Err(fail("does not admit this conditional state source"));
        }
        for branch in [&carry.then_source, &carry.else_source] {
            if !branch.payload.is_empty()
                || !(matches!(branch.kind.as_str(), "keep" | "keep_prev_carry")
                    || linear_source(&branch.kind, index, carries.len()))
            {
                return Err(fail("does not admit this conditional carry source"));
            }
        }
        let take = |cursor: &mut usize, expected: &str| -> Result<(), String> {
            if node.op.args.get(*cursor).map(String::as_str) != Some(expected) {
                return Err(fail("has noncanonical conditional metadata"));
            }
            *cursor += 1;
            Ok(())
        };
        take(&mut cursor, &carry.initial)?;
        take(&mut cursor, kind)?;
        if let Some(rhs) = rhs {
            take(&mut cursor, rhs)?;
        } else if node.op.args.get(cursor) != Some(&carry.then_source.kind)
            && node.op.args.get(cursor) == node.op.args.first()
        {
            // Existing source lowering encodes the induction seed after always.
            // Do not inherit the generic parser's arbitrary ignored-token fallback.
            cursor += 1;
        }
        take(&mut cursor, &carry.then_source.kind)?;
        take(&mut cursor, &carry.else_source.kind)?;
    }
    if cursor != node.op.args.len() {
        return Err(fail("has trailing conditional metadata"));
    }
    Ok(carries)
}

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
