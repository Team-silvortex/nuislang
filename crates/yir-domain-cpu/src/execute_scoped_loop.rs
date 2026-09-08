use yir_core::{
    ExecutionState, Node, RegisteredExecution, RegisteredExecutionStep, Resource, Value,
};

use crate::loop_metadata::{validate_loop_compare_kind, validate_loop_step_kind};

enum Argument {
    Current,
    Capture(Value),
    CopyBuffer(Option<usize>),
    MoveOwned(String),
}

struct ScopedLoop {
    node: Node,
    resource: Resource,
    function: String,
    arguments: Vec<Argument>,
    current: i64,
    limit: i64,
    step: i64,
    pending_call: bool,
    iterations: usize,
}

pub(super) fn begin_execution(
    node: &Node,
    resource: &Resource,
    state: &ExecutionState,
) -> Result<Option<Box<dyn RegisteredExecution>>, String> {
    if node.op.instruction != "loop_while_i64_effect"
        || node.op.args.get(5).map(String::as_str) != Some("cpu")
        || node.op.args.get(6).map(String::as_str) != Some("scoped_call")
    {
        return Ok(None);
    }
    let invalid = || {
        format!(
            "node `{}` has invalid scoped-call loop arguments",
            node.name
        )
    };
    let arity = node
        .op
        .args
        .get(7)
        .ok_or_else(invalid)?
        .parse::<usize>()
        .map_err(|_| invalid())?;
    if arity == 0 || arity.checked_add(8) != Some(node.op.args.len()) {
        return Err(invalid());
    }
    validate_loop_compare_kind(&node.op.args[3], &node.name)?;
    validate_loop_step_kind(&node.op.args[4], &node.name)?;
    let mut moved = std::collections::BTreeSet::new();
    let arguments = node.op.args[9..]
        .iter()
        .map(|input| {
            if input == "$current" {
                Ok(Argument::Current)
            } else if let Some(input) = input.strip_prefix("copy_owned:") {
                Ok(Argument::CopyBuffer(state.expect_pointer(input)?))
            } else if let Some(input) = input.strip_prefix("move_owned:") {
                if !matches!(state.expect_value(input)?, Value::OwnedBytes(_))
                    || !moved.insert(input)
                {
                    return Err(format!(
                        "node `{}` requires a distinct owned Bytes move capture",
                        node.name
                    ));
                }
                Ok(Argument::MoveOwned(input.to_owned()))
            } else {
                let value = state.expect_value(input)?;
                if !scalar(value) && !matches!(value, Value::Pointer(_)) {
                    return Err(format!(
                        "node `{}` has unsupported scoped capture `{input}`",
                        node.name
                    ));
                }
                Ok(Argument::Capture(value.clone()))
            }
        })
        .collect::<Result<_, String>>()?;
    Ok(Some(Box::new(ScopedLoop {
        node: node.clone(),
        resource: resource.clone(),
        function: node.op.args[8].clone(),
        arguments,
        current: state.expect_int(&node.op.args[0])?,
        limit: state.expect_int(&node.op.args[1])?,
        step: state.expect_int(&node.op.args[2])?,
        pending_call: false,
        iterations: 0,
    })))
}

impl RegisteredExecution for ScopedLoop {
    fn resume(
        &mut self,
        state: &mut ExecutionState,
        call_result: Option<Value>,
    ) -> Result<RegisteredExecutionStep, String> {
        if self.pending_call {
            if !call_result.as_ref().is_some_and(scalar) {
                return Err(format!(
                    "node `{}` requires a scalar scoped-call result",
                    self.node.name
                ));
            }
            self.pending_call = false;
            self.current = if self.node.op.args[4] == "add" {
                self.current.wrapping_add(self.step)
            } else {
                self.current.wrapping_sub(self.step)
            };
            self.iterations += 1;
        } else if call_result.is_some() {
            return Err(format!(
                "node `{}` received an unsolicited scoped-call result",
                self.node.name
            ));
        }
        let active = match self.node.op.args[3].as_str() {
            "lt" => self.current < self.limit,
            "le" => self.current <= self.limit,
            "gt" => self.current > self.limit,
            "ge" => self.current >= self.limit,
            "eq" => self.current == self.limit,
            "ne" => self.current != self.limit,
            _ => unreachable!("validated comparison"),
        };
        if !active {
            state.push_resource_event(&self.resource, format!(
                "effect cpu.loop_while_i64_effect @{} [{}]: iterations={} final={} action cpu.scoped_call {}",
                self.node.resource, self.resource.kind.raw, self.iterations, self.current, self.function
            ));
            return Ok(RegisteredExecutionStep::Complete(Value::Int(self.current)));
        }
        let arguments = self
            .arguments
            .iter()
            .map(|arg| match arg {
                Argument::Current => Ok(Value::Int(self.current)),
                Argument::Capture(value) => Ok(value.clone()),
                // Snapshot at the iteration, not at loop entry: preceding calls may write it.
                Argument::CopyBuffer(pointer) => Ok(Value::OwnedBytes(
                    state.read_heap_buffer(*pointer)?.elements.clone(),
                )),
                Argument::MoveOwned(input) => {
                    if self.iterations != 0 {
                        return Err(format!(
                            "node `{}` cannot repeat an owned Bytes move",
                            self.node.name
                        ));
                    }
                    state
                        .values
                        .remove(input)
                        .ok_or_else(|| format!("missing move capture `{input}`"))
                }
            })
            .collect::<Result<_, String>>()?;
        self.pending_call = true;
        Ok(RegisteredExecutionStep::Call {
            function: self.function.clone(),
            arguments,
        })
    }
}

fn scalar(value: &Value) -> bool {
    matches!(
        value,
        Value::Bool(_)
            | Value::I32(_)
            | Value::Int(_)
            | Value::F32(_)
            | Value::F64(_)
            | Value::Symbol(_)
            | Value::Unit
    )
}
