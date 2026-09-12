use std::collections::BTreeMap;

use super::MAX_SCALAR_SLOTS;
use yir_core::Node;

const MAX_ITERATIONS: i128 = 65536;

mod dynamic;
pub(crate) use dynamic::emit_guard;
pub(crate) mod scoped;

fn constant_inputs(node: &Node, nodes: &BTreeMap<&str, &Node>) -> Option<[i64; 3]> {
    let constant = |name: &str| {
        nodes
            .get(name)
            .filter(|value| {
                value.op.module == "cpu"
                    && matches!(value.op.instruction.as_str(), "const" | "const_i64")
                    && value.op.args.len() == 1
            })
            .and_then(|value| value.op.args[0].parse::<i64>().ok())
    };
    Some([
        constant(node.op.args.first()?)?,
        constant(node.op.args.get(1)?)?,
        constant(node.op.args.get(2)?)?,
    ])
}

pub(super) fn validate(node: &Node, nodes: &BTreeMap<&str, &Node>) -> Result<(), String> {
    let chain = match node.op.instruction.as_str() {
        "loop_while_i64" => false,
        "loop_while_i64_chain" | "loop_while_scalar_chain" => true,
        "loop_while_i64_effect" => {
            scoped::parse(node)?;
            false
        }
        _ => return Ok(()),
    };
    let fail = |message: &str| format!("native scalar loop `{}` {message}", node.name);
    let args = &node.op.args;
    if args.len() < 5 || (node.op.instruction == "loop_while_i64" && args.len() != 5) {
        return Err(fail("has an invalid counted-loop shape"));
    }
    if !matches!(args[3].as_str(), "eq" | "ne" | "lt" | "le" | "gt" | "ge") {
        return Err(fail("has an unsupported induction comparison"));
    }
    if !matches!(args[4].as_str(), "add" | "sub") {
        return Err(fail("has an unsupported induction step"));
    }
    if let Some([initial, limit, step]) = constant_inputs(node, nodes) {
        bounded_iterations(initial, limit, step, &args[3], &args[4])
            .map_err(|error| fail(error))?;
    }
    if !chain {
        return Ok(());
    }
    let carries = &args[5..];
    let count = carries.len() / 2;
    if carries.len() % 2 != 0 || count == 0 || count > MAX_SCALAR_SLOTS {
        return Err(fail("exceeds its flat scalar-carry shape/bound"));
    }
    for (index, pair) in carries.chunks_exact(2).enumerate() {
        let kind = pair[1].as_str();
        if matches!(
            kind,
            "add_current" | "add_prev_current" | "mul_current" | "mul_prev_current"
        ) {
            continue;
        }
        let valid = [
            ("add_prev_carry", count),
            ("mul_prev_carry", count),
            ("add_carry", index),
            ("mul_carry", index),
        ]
        .into_iter()
        .any(|(prefix, available)| {
            kind.strip_prefix(prefix)
                .and_then(|index| index.parse::<usize>().ok())
                .is_some_and(|index| index < available)
        });
        if !valid {
            return Err(fail("does not admit this scalar carry source"));
        }
    }
    Ok(())
}

// Prove the existing non-wrapping induction in wide arithmetic, without executing
// the loop or changing its YIR. This bounds each loop, not whole-callback fuel.
fn bounded_iterations(
    initial: i64,
    limit: i64,
    step: i64,
    compare: &str,
    operation: &str,
) -> Result<i128, &'static str> {
    let initial = i128::from(initial);
    let limit = i128::from(limit);
    let delta = match operation {
        "add" => i128::from(step),
        "sub" => -i128::from(step),
        _ => return Err("has an unsupported induction step"),
    };
    let active = match compare {
        "eq" => initial == limit,
        "ne" => initial != limit,
        "lt" => initial < limit,
        "le" => initial <= limit,
        "gt" => initial > limit,
        "ge" => initial >= limit,
        _ => return Err("has an unsupported induction comparison"),
    };
    if !active {
        return Ok(0);
    }
    let nonterminating = "cannot prove finite non-wrapping induction";
    let trips = match compare {
        "eq" if delta != 0 => 1,
        "ne" if delta != 0 && (limit - initial) % delta == 0 => {
            let trips = (limit - initial) / delta;
            if trips <= 0 {
                return Err(nonterminating);
            }
            trips
        }
        "lt" if delta > 0 => (limit - initial + delta - 1) / delta,
        "le" if delta > 0 => (limit - initial) / delta + 1,
        "gt" if delta < 0 => (initial - limit - delta - 1) / -delta,
        "ge" if delta < 0 => (initial - limit) / -delta + 1,
        _ => return Err(nonterminating),
    };
    if trips > MAX_ITERATIONS {
        return Err("exceeds its 65536-iteration bound");
    }
    i64::try_from(initial + trips * delta)
        .map_err(|_| "induction overflows i64 before its exit")?;
    Ok(trips)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counted_loop_proof_handles_directions_zero_trips_and_exact_limits() {
        for (start, end, step, cmp, op, expected) in [
            (0, 5, 2, "lt", "add", 3),
            (0, 6, 2, "le", "add", 4),
            (5, 0, 2, "gt", "sub", 3),
            (6, 0, 2, "ge", "sub", 4),
            (0, 5, -2, "lt", "sub", 3),
            (5, 0, -2, "gt", "add", 3),
            (2, 2, 1, "eq", "add", 1),
            (2, 3, 0, "eq", "add", 0),
            (2, 2, 0, "ne", "add", 0),
            (2, 8, 2, "ne", "add", 3),
            (8, 2, 2, "ne", "sub", 3),
            (4, 0, 0, "lt", "add", 0),
            (0, 65536, 1, "lt", "add", MAX_ITERATIONS),
            (i64::MIN, -1, i64::MAX, "lt", "add", 1),
            (-1, i64::MAX, i64::MIN, "lt", "sub", 1),
        ] {
            assert_eq!(bounded_iterations(start, end, step, cmp, op), Ok(expected));
        }
    }

    #[test]
    fn counted_loop_proof_rejects_nontermination_wrap_and_excess_work() {
        for (start, end, step, cmp, op) in [
            (0, 1, 0, "lt", "add"),
            (0, 0, 0, "eq", "add"),
            (0, 5, 1, "lt", "sub"),
            (5, 0, 1, "gt", "add"),
            (0, 3, 2, "ne", "add"),
            (0, 3, 1, "ne", "sub"),
            (0, 65537, 1, "lt", "add"),
            (i64::MIN, i64::MAX, 1, "lt", "add"),
            (i64::MAX, i64::MAX, 1, "eq", "add"),
            (i64::MAX, i64::MAX, 1, "le", "add"),
            (i64::MIN, i64::MIN, 1, "ge", "sub"),
            (0, i64::MAX, i64::MAX - 1, "lt", "add"),
        ] {
            assert!(bounded_iterations(start, end, step, cmp, op).is_err());
        }
    }

    #[test]
    fn counted_loop_proof_matches_small_independent_step_simulation() {
        for initial in -6..=6 {
            for limit in -6..=6 {
                for step in -4..=4 {
                    for operation in ["add", "sub"] {
                        for compare in ["eq", "ne", "lt", "le", "gt", "ge"] {
                            let mut current = initial;
                            let mut expected = None;
                            for trips in 0..32 {
                                let active = match compare {
                                    "eq" => current == limit,
                                    "ne" => current != limit,
                                    "lt" => current < limit,
                                    "le" => current <= limit,
                                    "gt" => current > limit,
                                    "ge" => current >= limit,
                                    _ => unreachable!(),
                                };
                                if !active {
                                    expected = Some(trips);
                                    break;
                                }
                                current += if operation == "add" { step } else { -step };
                            }
                            assert_eq!(
                                bounded_iterations(initial, limit, step, compare, operation).ok(),
                                expected,
                                "{initial} {compare} {limit}; {operation} {step}"
                            );
                        }
                    }
                }
            }
        }
    }
}
