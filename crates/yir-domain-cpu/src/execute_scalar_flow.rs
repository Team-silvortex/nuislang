use yir_core::{
    ExecutionState, Node, RegisteredExecution, RegisteredExecutionStep, Resource, StructValue,
    Value,
};

use crate::loop_metadata::*;

enum Step {
    Add(i64),
    Sub(i64),
    Call(String),
}

struct ScalarFlow {
    node: Node,
    resource: Resource,
    compare: String,
    limit: i64,
    step: Step,
    current: i64,
    carries: Vec<i64>,
    updates: Vec<ParsedConditionalCarry>,
    control: Option<LoopFlowExpr>,
    post_flow: bool,
    pending_step: bool,
    iterations: usize,
}

pub(super) fn begin_execution(
    node: &Node,
    resource: &Resource,
    state: &ExecutionState,
) -> Result<Option<Box<dyn RegisteredExecution>>, String> {
    let Some(instruction) = node
        .op
        .instruction
        .strip_prefix("loop_while_scalar_")
        .or_else(|| node.op.instruction.strip_prefix("loop_while_i64_"))
    else {
        return Ok(None);
    };
    let (is_async, post_flow, conditional) = match instruction {
        "chain" => (false, false, false),
        "cond_chain" => (false, false, true),
        "flow_chain" => (false, false, false),
        "flow_cond_chain" => (false, false, true),
        "post_flow_chain" => (false, true, false),
        "post_flow_cond_chain" => (false, true, true),
        "async_flow_chain" => (true, false, false),
        "async_flow_cond_chain" => (true, false, true),
        "async_post_flow_chain" => (true, true, false),
        "async_post_flow_cond_chain" => (true, true, true),
        _ => return Ok(None),
    };
    let arg = |index: usize| {
        node.op.args.get(index).ok_or_else(|| {
            format!(
                "node `{}` is missing scalar flow argument {index}",
                node.name
            )
        })
    };
    let current = state.expect_int(arg(0)?)?;
    let limit = state.expect_int(arg(1)?)?;
    let compare = arg(3)?.clone();
    validate_loop_compare_kind(&compare, &node.name)?;
    let step = if is_async {
        Step::Call(arg(2)?.clone())
    } else {
        let value = state.expect_int(arg(2)?)?;
        validate_loop_step_kind(arg(4)?, &node.name)?;
        if arg(4)? == "add" {
            Step::Add(value)
        } else {
            Step::Sub(value)
        }
    };
    let validate = if post_flow {
        validate_post_flow_control_kind
    } else {
        validate_flow_control_kind
    };
    let (control, carry_start) = if matches!(instruction, "chain" | "cond_chain") {
        (None, 5)
    } else {
        let (control, start) = parse_loop_flow_expr(
            &node.op.args,
            if is_async { 4 } else { 5 },
            &node.name,
            &validate,
        )?;
        validate_flow(&control)?;
        (Some(control), start)
    };
    let updates = if conditional {
        parse_conditional_carries(&node.op.args, carry_start, &node.name, true)?
    } else {
        let args = &node.op.args[carry_start..];
        if !args.len().is_multiple_of(2) {
            return Err(format!(
                "node `{}` has incomplete scalar carries",
                node.name
            ));
        }
        args.chunks_exact(2)
            .map(|pair| {
                let source = ParsedCarryBranchSource {
                    kind: pair[1].clone(),
                    payload: vec![],
                };
                ParsedConditionalCarry {
                    initial: pair[0].clone(),
                    condition: LoopCondExpr::Leaf {
                        kind: "always".to_owned(),
                        rhs: None,
                    },
                    then_source: source.clone(),
                    else_source: source,
                }
            })
            .collect()
    };
    for update in &updates {
        for source in [&update.then_source, &update.else_source] {
            validate_source(source)?;
        }
    }
    let carries = updates
        .iter()
        .map(|update| state.expect_int(&update.initial))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(Box::new(ScalarFlow {
        node: node.clone(),
        resource: resource.clone(),
        compare,
        limit,
        step,
        current,
        carries,
        updates,
        control,
        post_flow,
        pending_step: false,
        iterations: 0,
    })))
}

impl RegisteredExecution for ScalarFlow {
    fn resume(
        &mut self,
        state: &mut ExecutionState,
        call_result: Option<Value>,
    ) -> Result<RegisteredExecutionStep, String> {
        let previous = self.current;
        if self.pending_step {
            let Some(Value::Int(value)) = call_result else {
                return Err(format!(
                    "node `{}` requires an i64 step result",
                    self.node.name
                ));
            };
            self.pending_step = false;
            self.current = value;
        } else {
            if call_result.is_some() {
                return Err(format!(
                    "node `{}` received an unsolicited step result",
                    self.node.name
                ));
            }
            if !compare(&self.compare, self.current, self.limit)? {
                return Ok(self.complete(state));
            }
            self.current = match &self.step {
                Step::Add(value) => self.current.wrapping_add(*value),
                Step::Sub(value) => self.current.wrapping_sub(*value),
                Step::Call(function) => {
                    self.pending_step = true;
                    return Ok(RegisteredExecutionStep::Call {
                        function: function.clone(),
                        arguments: vec![Value::Int(self.current)],
                    });
                }
            };
        }
        self.iterations += 1;
        let old_carries = self.carries.clone();
        if self.post_flow {
            self.update_carries(previous, &old_carries, state)?;
        }
        let values = LoopValues {
            previous,
            current: self.current,
            old_carries: &old_carries,
            carries: &self.carries,
        };
        let action = self
            .control
            .as_ref()
            .map(|control| evaluate_flow(control, &values, state))
            .transpose()?
            .flatten();
        let should_break = action == Some("break");
        if !self.post_flow && action.is_none() {
            self.update_carries(previous, &old_carries, state)?;
        }
        if should_break {
            return Ok(self.complete(state));
        }
        Ok(RegisteredExecutionStep::Continue)
    }
}

impl ScalarFlow {
    fn update_carries(
        &mut self,
        previous: i64,
        old_carries: &[i64],
        state: &ExecutionState,
    ) -> Result<(), String> {
        let mut next = Vec::with_capacity(self.carries.len());
        for (index, update) in self.updates.iter().enumerate() {
            let values = LoopValues {
                previous,
                current: self.current,
                old_carries,
                carries: &next,
            };
            let source = if evaluate_condition(&update.condition, &values, state)? {
                &update.then_source
            } else {
                &update.else_source
            };
            let updated = if matches!(source.kind.as_str(), "keep" | "keep_prev_carry") {
                old_carries[index]
            } else {
                let (operation, name) = source
                    .kind
                    .split_once('_')
                    .ok_or_else(|| "invalid scalar carry source".to_owned())?;
                let operand = values.get(name)?;
                match operation {
                    "add" => old_carries[index].wrapping_add(operand),
                    "mul" => old_carries[index].wrapping_mul(operand),
                    _ => return Err("unsupported scalar carry operation".to_owned()),
                }
            };
            next.push(updated);
        }
        self.carries = next;
        Ok(())
    }

    fn complete(&self, state: &mut ExecutionState) -> RegisteredExecutionStep {
        let mut fields = vec![("current".to_owned(), Value::Int(self.current))];
        fields.extend(
            self.carries
                .iter()
                .enumerate()
                .map(|(index, value)| (format!("carry{index}"), Value::Int(*value))),
        );
        state.push_resource_event(
            &self.resource,
            format!(
                "effect {} @{} [{}]: iterations={} final={}",
                self.node.op.full_name(),
                self.node.resource,
                self.resource.kind.raw,
                self.iterations,
                self.current,
            ),
        );
        RegisteredExecutionStep::Complete(Value::Struct(StructValue {
            type_name: "LoopState".to_owned(),
            fields,
        }))
    }
}

struct LoopValues<'a> {
    previous: i64,
    current: i64,
    old_carries: &'a [i64],
    carries: &'a [i64],
}

impl LoopValues<'_> {
    fn get(&self, name: &str) -> Result<i64, String> {
        match name {
            "current" => return Ok(self.current),
            "prev_current" => return Ok(self.previous),
            _ => {}
        }
        let (index, carries) = if let Some(index) = name.strip_prefix("prev_carry") {
            (index, self.old_carries)
        } else if let Some(index) = name.strip_prefix("carry") {
            (index, self.carries)
        } else {
            return Err(format!("unsupported scalar loop source `{name}`"));
        };
        index
            .parse::<usize>()
            .ok()
            .and_then(|index| carries.get(index))
            .copied()
            .ok_or_else(|| format!("unavailable scalar loop source `{name}`"))
    }
}

fn compare(kind: &str, lhs: i64, rhs: i64) -> Result<bool, String> {
    Ok(match kind {
        "eq" => lhs == rhs,
        "ne" => lhs != rhs,
        "lt" => lhs < rhs,
        "le" => lhs <= rhs,
        "gt" => lhs > rhs,
        "ge" => lhs >= rhs,
        _ => return Err(format!("unsupported scalar loop comparison `{kind}`")),
    })
}

fn evaluate_condition(
    expr: &LoopCondExpr,
    values: &LoopValues<'_>,
    state: &ExecutionState,
) -> Result<bool, String> {
    match expr {
        LoopCondExpr::Leaf { kind, rhs } => {
            if kind == "always" {
                return Ok(true);
            }
            let (source, comparison) = kind
                .rsplit_once('_')
                .ok_or_else(|| format!("invalid scalar loop condition `{kind}`"))?;
            let rhs = rhs
                .as_ref()
                .ok_or_else(|| "missing scalar loop rhs".to_owned())?;
            compare(comparison, values.get(source)?, state.expect_int(rhs)?)
        }
        LoopCondExpr::Binary { op, lhs, rhs } => {
            match op.as_str() {
                "and" => Ok(evaluate_condition(lhs, values, state)?
                    && evaluate_condition(rhs, values, state)?),
                "or" => Ok(evaluate_condition(lhs, values, state)?
                    || evaluate_condition(rhs, values, state)?),
                _ => Err(format!("unsupported scalar loop condition operator `{op}`")),
            }
        }
    }
}

fn evaluate_flow<'a>(
    expr: &'a LoopFlowExpr,
    values: &LoopValues<'_>,
    state: &ExecutionState,
) -> Result<Option<&'a str>, String> {
    match expr {
        LoopFlowExpr::Legacy { condition, action }
        | LoopFlowExpr::Terminal { condition, action } => {
            Ok(evaluate_condition(condition, values, state)?.then_some(action.as_str()))
        }
        LoopFlowExpr::Binary { op, lhs, rhs } if op == "flow_or" => {
            if let Some(action) = evaluate_flow(lhs, values, state)? {
                return Ok(Some(action));
            }
            evaluate_flow(rhs, values, state)
        }
        LoopFlowExpr::Binary { op, .. } => {
            Err(format!("unsupported scalar loop flow operator `{op}`"))
        }
    }
}

fn validate_flow(expr: &LoopFlowExpr) -> Result<(), String> {
    match expr {
        LoopFlowExpr::Binary { op, lhs, rhs } if op == "flow_or" => {
            validate_flow(lhs)?;
            validate_flow(rhs)
        }
        LoopFlowExpr::Binary { op, .. } => {
            Err(format!("unsupported scalar loop flow operator `{op}`"))
        }
        _ => Ok(()),
    }
}

fn validate_source(source: &ParsedCarryBranchSource) -> Result<(), String> {
    let kind = source.kind.as_str();
    let operand = kind
        .strip_prefix("add_")
        .or_else(|| kind.strip_prefix("mul_"));
    if source.payload.is_empty()
        && (matches!(kind, "keep" | "keep_prev_carry")
            || operand.is_some_and(|source| {
                matches!(source, "current" | "prev_current")
                    || source
                        .strip_prefix("carry")
                        .or_else(|| source.strip_prefix("prev_carry"))
                        .is_some_and(|index| index.parse::<usize>().is_ok())
            }))
    {
        Ok(())
    } else {
        Err(format!("unsupported reference scalar flow carry `{kind}`"))
    }
}
