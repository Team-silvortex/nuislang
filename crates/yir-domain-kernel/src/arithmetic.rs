use yir_core::{ExecutionState, Node, TensorValue, Value};

pub(crate) fn is_tensor_binary_instruction(instruction: &str) -> bool {
    matches!(instruction, "add" | "sub" | "mul" | "div")
}

pub(crate) fn is_tensor_scalar_instruction(instruction: &str) -> bool {
    matches!(
        instruction,
        "add_scalar" | "sub_scalar" | "mul_scalar" | "div_scalar"
    )
}

pub(crate) fn is_typed_scalar_instruction(instruction: &str) -> bool {
    matches!(
        instruction,
        "add_i32"
            | "sub_i32"
            | "mul_i32"
            | "div_i32"
            | "add_f32"
            | "sub_f32"
            | "mul_f32"
            | "div_f32"
            | "add_f64"
            | "sub_f64"
            | "mul_f64"
            | "div_f64"
    )
}

pub(crate) fn is_axis_scalar_instruction(instruction: &str) -> bool {
    matches!(
        instruction,
        "add_scalar_axis" | "sub_scalar_axis" | "mul_scalar_axis" | "div_scalar_axis"
    )
}

pub(crate) fn is_arithmetic_instruction(instruction: &str) -> bool {
    is_tensor_binary_instruction(instruction)
        || is_tensor_scalar_instruction(instruction)
        || is_typed_scalar_instruction(instruction)
        || is_axis_scalar_instruction(instruction)
}

pub(crate) fn execute_arithmetic_node(
    node: &Node,
    state: &ExecutionState,
) -> Result<Value, String> {
    match node.op.instruction.as_str() {
        "add" | "sub" | "mul" | "div" => execute_tensor_binary(node, state),
        "add_scalar" | "sub_scalar" | "mul_scalar" | "div_scalar" => {
            execute_tensor_scalar(node, state)
        }
        "add_i32" | "sub_i32" | "mul_i32" | "div_i32" | "add_f32" | "sub_f32" | "mul_f32"
        | "div_f32" | "add_f64" | "sub_f64" | "mul_f64" | "div_f64" => {
            execute_typed_scalar(node, state)
        }
        "add_scalar_axis" | "sub_scalar_axis" | "mul_scalar_axis" | "div_scalar_axis" => {
            execute_axis_scalar(node, state)
        }
        other => Err(format!("unknown kernel arithmetic instruction `{other}`")),
    }
}

pub(crate) fn expect_tensor_scalar_i64(value: &Value) -> Result<i64, String> {
    match value {
        Value::Bool(value) => Ok(if *value { 1 } else { 0 }),
        Value::I32(value) => Ok(*value as i64),
        Value::Int(value) => Ok(*value),
        Value::F32(value) => Ok(value.round() as i64),
        Value::F64(value) => Ok(value.round() as i64),
        other => Err(format!(
            "kernel tensor-scalar op expects scalar value, got {}",
            other
        )),
    }
}

fn execute_tensor_binary(node: &Node, state: &ExecutionState) -> Result<Value, String> {
    let lhs = state.expect_tensor(&node.op.args[0])?;
    let rhs = state.expect_tensor(&node.op.args[1])?;
    let tensor = match node.op.instruction.as_str() {
        "add" => elementwise_binary_broadcast(node, lhs, rhs, |lhs, rhs| Ok(lhs + rhs))?,
        "sub" => elementwise_binary_broadcast(node, lhs, rhs, |lhs, rhs| Ok(lhs - rhs))?,
        "mul" => elementwise_binary_broadcast(node, lhs, rhs, |lhs, rhs| Ok(lhs * rhs))?,
        "div" => elementwise_binary_broadcast(node, lhs, rhs, |lhs, rhs| {
            checked_i64_div("kernel.div", lhs, rhs)
        })?,
        other => return Err(format!("unknown kernel tensor arithmetic `{other}`")),
    };
    Ok(Value::Tensor(tensor))
}

fn execute_tensor_scalar(node: &Node, state: &ExecutionState) -> Result<Value, String> {
    let input = state.expect_tensor(&node.op.args[0])?;
    let scalar = expect_tensor_scalar_i64(state.expect_value(&node.op.args[1])?)?;
    map_tensor_scalar(input, scalar, node.op.instruction.as_str()).map(Value::Tensor)
}

fn execute_axis_scalar(node: &Node, state: &ExecutionState) -> Result<Value, String> {
    let input = state.expect_tensor(&node.op.args[0])?;
    let scalar = expect_tensor_scalar_i64(state.expect_value(&node.op.args[2])?)?;
    match node.op.args[1].as_str() {
        "rows" | "cols" => {
            map_tensor_scalar(input, scalar, node.op.instruction.as_str()).map(Value::Tensor)
        }
        other => Err(format!(
            "node `{}` has invalid map axis `{other}`; expected rows or cols",
            node.name
        )),
    }
}

fn execute_typed_scalar(node: &Node, state: &ExecutionState) -> Result<Value, String> {
    match node.op.instruction.as_str() {
        "add_i32" => Ok(Value::I32(
            state.expect_i32(&node.op.args[0])? + state.expect_i32(&node.op.args[1])?,
        )),
        "sub_i32" => Ok(Value::I32(
            state.expect_i32(&node.op.args[0])? - state.expect_i32(&node.op.args[1])?,
        )),
        "mul_i32" => Ok(Value::I32(
            state.expect_i32(&node.op.args[0])? * state.expect_i32(&node.op.args[1])?,
        )),
        "div_i32" => {
            let lhs = state.expect_i32(&node.op.args[0])?;
            let rhs = state.expect_i32(&node.op.args[1])?;
            if rhs == 0 {
                return Err("kernel.div_i32 cannot divide by zero".to_owned());
            }
            Ok(Value::I32(lhs / rhs))
        }
        "add_f32" => Ok(Value::F32(
            state.expect_f32(&node.op.args[0])? + state.expect_f32(&node.op.args[1])?,
        )),
        "sub_f32" => Ok(Value::F32(
            state.expect_f32(&node.op.args[0])? - state.expect_f32(&node.op.args[1])?,
        )),
        "mul_f32" => Ok(Value::F32(
            state.expect_f32(&node.op.args[0])? * state.expect_f32(&node.op.args[1])?,
        )),
        "div_f32" => {
            let lhs = state.expect_f32(&node.op.args[0])?;
            let rhs = state.expect_f32(&node.op.args[1])?;
            if rhs == 0.0 {
                return Err("kernel.div_f32 cannot divide by zero".to_owned());
            }
            Ok(Value::F32(lhs / rhs))
        }
        "add_f64" => Ok(Value::F64(
            state.expect_f64(&node.op.args[0])? + state.expect_f64(&node.op.args[1])?,
        )),
        "sub_f64" => Ok(Value::F64(
            state.expect_f64(&node.op.args[0])? - state.expect_f64(&node.op.args[1])?,
        )),
        "mul_f64" => Ok(Value::F64(
            state.expect_f64(&node.op.args[0])? * state.expect_f64(&node.op.args[1])?,
        )),
        "div_f64" => {
            let lhs = state.expect_f64(&node.op.args[0])?;
            let rhs = state.expect_f64(&node.op.args[1])?;
            if rhs == 0.0 {
                return Err("kernel.div_f64 cannot divide by zero".to_owned());
            }
            Ok(Value::F64(lhs / rhs))
        }
        other => Err(format!("unknown kernel typed arithmetic `{other}`")),
    }
}

fn map_tensor_scalar(
    input: &TensorValue,
    scalar: i64,
    instruction: &str,
) -> Result<TensorValue, String> {
    let elements = input
        .elements
        .iter()
        .map(|value| match instruction {
            "add_scalar" | "add_scalar_axis" => Ok(*value + scalar),
            "sub_scalar" | "sub_scalar_axis" => Ok(*value - scalar),
            "mul_scalar" | "mul_scalar_axis" => Ok(*value * scalar),
            "div_scalar" | "div_scalar_axis" => {
                checked_i64_div("kernel.div_scalar", *value, scalar)
            }
            other => Err(format!("unknown kernel scalar arithmetic `{other}`")),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TensorValue {
        rows: input.rows,
        cols: input.cols,
        elements,
    })
}

fn checked_i64_div(op: &str, lhs: i64, rhs: i64) -> Result<i64, String> {
    if rhs == 0 {
        return Err(format!("{op} cannot divide by zero"));
    }
    Ok(lhs / rhs)
}

fn elementwise_binary_broadcast(
    node: &Node,
    lhs: &TensorValue,
    rhs: &TensorValue,
    op: impl Fn(i64, i64) -> Result<i64, String>,
) -> Result<TensorValue, String> {
    let rows = lhs.rows.max(rhs.rows);
    let cols = lhs.cols.max(rhs.cols);
    let lhs = super::broadcast(lhs, rows, cols).map_err(|_| {
        format!(
            "node `{}` expects broadcast-compatible tensor shapes, got {}x{} and {}x{}",
            node.name, lhs.rows, lhs.cols, rhs.rows, rhs.cols
        )
    })?;
    let rhs = super::broadcast(rhs, rows, cols).map_err(|_| {
        format!(
            "node `{}` expects broadcast-compatible tensor shapes, got {}x{} and {}x{}",
            node.name, lhs.rows, lhs.cols, rhs.rows, rhs.cols
        )
    })?;

    Ok(TensorValue {
        rows,
        cols,
        elements: lhs
            .elements
            .iter()
            .copied()
            .zip(rhs.elements.iter().copied())
            .map(|(lhs, rhs)| op(lhs, rhs))
            .collect::<Result<Vec<_>, _>>()?,
    })
}
