use super::{constant_inputs, MAX_ITERATIONS};
use crate::{fresh_block, fresh_reg, LlvmValueRef};
use std::collections::BTreeMap;
use yir_core::Node;

/// Only the strict native profile calls this preflight. The loop emitters and
/// reference executor keep their own semantics; a trap is not reference fuel.
pub(crate) fn emit_guard(
    node: &Node,
    nodes: &BTreeMap<&str, &Node>,
    registers: &BTreeMap<String, LlvmValueRef>,
    body: &mut Vec<String>,
    next_reg: &mut usize,
    next_block: &mut usize,
) -> Result<(), String> {
    if !matches!(
        node.op.instruction.as_str(),
        "loop_while_i64" | "loop_while_i64_chain" | "loop_while_scalar_chain"
    ) {
        return Ok(());
    }
    super::validate(node, nodes)?;
    if constant_inputs(node, nodes).is_some() {
        return Ok(());
    }
    let value = |index: usize| match registers.get(&node.op.args[index]) {
        Some(LlvmValueRef::I64(value)) => Ok(value.as_str()),
        _ => Err(format!(
            "native scalar loop `{}` requires exact i64 induction inputs",
            node.name
        )),
    };
    let initial = value(0)?;
    let limit = value(1)?;
    let step = value(2)?;
    let compare = node.op.args[3].as_str();
    let pred = match compare {
        "eq" => "eq",
        "ne" => "ne",
        "lt" => "slt",
        "le" => "sle",
        "gt" => "sgt",
        "ge" => "sge",
        _ => unreachable!("validated comparison"),
    };
    let check = fresh_block(next_block, "native_loop_preflight");
    let ready = fresh_block(next_block, "native_loop_admitted");
    let trap = fresh_block(next_block, "native_loop_rejected");
    let mut ir = GuardEmitter { body, next_reg };
    let active = ir.emit(format!("icmp {pred} i64 {initial}, {limit}"));
    ir.body
        .push(format!("  br i1 {active}, label %{check}, label %{ready}"));
    ir.body.push(format!("{check}:"));

    let start = ir.emit(format!("sext i64 {initial} to i128"));
    let end = ir.emit(format!("sext i64 {limit} to i128"));
    let wide_step = ir.emit(format!("sext i64 {step} to i128"));
    let delta = if node.op.args[4] == "sub" {
        ir.emit(format!("sub i128 0, {wide_step}"))
    } else {
        wide_step
    };
    let positive = ir.emit(format!("icmp sgt i128 {delta}, 0"));
    let negative = ir.emit(format!("icmp slt i128 {delta}, 0"));
    let nonzero = ir.emit(format!("icmp ne i128 {delta}, 0"));
    let direction = match compare {
        "eq" => nonzero.clone(),
        "lt" | "le" => positive.clone(),
        "gt" | "ge" => negative.clone(),
        "ne" => {
            let ascending = ir.emit(format!("icmp sgt i128 {end}, {start}"));
            ir.emit(format!(
                "select i1 {ascending}, i1 {positive}, i1 {negative}"
            ))
        }
        _ => unreachable!("validated comparison"),
    };
    let (trips, finite) = if compare == "eq" {
        ("1".to_owned(), direction)
    } else {
        // Absolute distance fits u64 even across MIN..MAX, and |delta| <= 2^63.
        // Unsigned i64 division avoids an i128 compiler-runtime division helper.
        let difference = ir.emit(format!("sub i128 {end}, {start}"));
        let neg_distance = ir.emit(format!("sub i128 0, {difference}"));
        let ascending = ir.emit(format!("icmp sgt i128 {difference}, 0"));
        let distance = ir.emit(format!(
            "select i1 {ascending}, i128 {difference}, i128 {neg_distance}"
        ));
        let distance = ir.emit(format!("trunc i128 {distance} to i64"));
        let neg_delta = ir.emit(format!("sub i128 0, {delta}"));
        let magnitude = ir.emit(format!(
            "select i1 {positive}, i128 {delta}, i128 {neg_delta}"
        ));
        let magnitude = ir.emit(format!("trunc i128 {magnitude} to i64"));
        // Even a rejected zero step must never reach LLVM division-by-zero UB.
        let divisor = ir.emit(format!("select i1 {nonzero}, i64 {magnitude}, i64 1"));
        let quotient = ir.emit(format!("udiv i64 {distance}, {divisor}"));
        let remainder = ir.emit(format!("urem i64 {distance}, {divisor}"));
        let quotient = ir.emit(format!("zext i64 {quotient} to i128"));
        match compare {
            "ne" => {
                let exact = ir.emit(format!("icmp eq i64 {remainder}, 0"));
                (quotient, ir.emit(format!("and i1 {direction}, {exact}")))
            }
            "lt" | "gt" => {
                let round_up = ir.emit(format!("icmp ne i64 {remainder}, 0"));
                let round_up = ir.emit(format!("zext i1 {round_up} to i128"));
                (
                    ir.emit(format!("add i128 {quotient}, {round_up}")),
                    direction,
                )
            }
            "le" | "ge" => (ir.emit(format!("add i128 {quotient}, 1")), direction),
            _ => unreachable!("validated comparison"),
        }
    };
    let bounded = ir.emit(format!("icmp ule i128 {trips}, {MAX_ITERATIONS}"));
    let displacement = ir.emit(format!("mul i128 {trips}, {delta}"));
    let final_value = ir.emit(format!("add i128 {start}, {displacement}"));
    let lower = ir.emit(format!("icmp sge i128 {final_value}, {}", i64::MIN));
    let upper = ir.emit(format!("icmp sle i128 {final_value}, {}", i64::MAX));
    let range = ir.emit(format!("and i1 {lower}, {upper}"));
    let finite_bound = ir.emit(format!("and i1 {finite}, {bounded}"));
    let admitted = ir.emit(format!("and i1 {finite_bound}, {range}"));
    ir.body
        .push(format!("  br i1 {admitted}, label %{ready}, label %{trap}"));
    ir.body.push(format!("{trap}:"));
    ir.body.push("  call void @llvm.trap()".to_owned());
    ir.body.push("  unreachable".to_owned());
    ir.body.push(format!("{ready}:"));
    Ok(())
}

struct GuardEmitter<'a> {
    body: &'a mut Vec<String>,
    next_reg: &'a mut usize,
}

impl GuardEmitter<'_> {
    fn emit(&mut self, expression: String) -> String {
        let register = fresh_reg(self.next_reg);
        self.body.push(format!("  {register} = {expression}"));
        register
    }
}
