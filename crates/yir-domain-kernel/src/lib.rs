use yir_core::{
    ExecutionState, InstructionSemantics, Node, RegisteredMod, Resource, TensorValue, Value,
};

pub struct KernelMod;

impl RegisteredMod for KernelMod {
    fn module_name(&self) -> &'static str {
        "kernel"
    }

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        describe_kernel_node(node, resource)
    }

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        execute_kernel_node(node, resource, state)
    }
}

pub struct LegacyNpuMod;

impl RegisteredMod for LegacyNpuMod {
    fn module_name(&self) -> &'static str {
        "npu"
    }

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        describe_kernel_node(node, resource)
    }

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        execute_kernel_node(node, resource, state)
    }
}

fn describe_kernel_node(
    node: &Node,
    resource: &Resource,
) -> Result<InstructionSemantics, String> {
    require_kernel_resource(node, resource)?;

    match node.op.instruction.as_str() {
        "target_config" => {
            if node.op.args.len() != 3 {
                return Err(format!(
                    "node `{}` expects `kernel.target_config <name> <resource> <arch> <runtime> <lane_width>`",
                    node.name
                ));
            }
            node.op.args[2].parse::<i64>().map_err(|_| {
                format!(
                    "node `{}` has invalid lane width `{}`",
                    node.name, node.op.args[2]
                )
            })?;
            Ok(InstructionSemantics::pure(Vec::new()))
        }
        "tensor" => {
            if node.op.args.len() != 3 {
                return Err(format!(
                    "node `{}` expects `kernel.tensor <name> <resource> <rows> <cols> <csv-elements>`",
                    node.name
                ));
            }
            validate_tensor_literal(node)?;
            Ok(InstructionSemantics::pure(Vec::new()))
        }
        "fill" => {
            if node.op.args.len() != 3 {
                return Err(format!(
                    "node `{}` expects `kernel.fill <name> <resource> <rows> <cols> <value>`",
                    node.name
                ));
            }
            parse_shape(node)?;
            node.op.args[2].parse::<i64>().map_err(|_| {
                format!(
                    "node `{}` has invalid fill value `{}`",
                    node.name, node.op.args[2]
                )
            })?;
            Ok(InstructionSemantics::pure(Vec::new()))
        }
        "matmul" | "add_bias" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `kernel.{} <name> <resource> <lhs> <rhs>`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::pure(node.op.args.clone()))
        }
        "relu" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `kernel.relu <name> <resource> <input>`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::pure(node.op.args.clone()))
        }
        "print" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `kernel.print <name> <resource> <input>`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        other => Err(format!("unknown kernel instruction `{other}`")),
    }
}

fn execute_kernel_node(
    node: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
) -> Result<Value, String> {
    match node.op.instruction.as_str() {
        "target_config" => Ok(Value::Tuple(vec![
            Value::Symbol(node.op.args[0].clone()),
            Value::Symbol(node.op.args[1].clone()),
            Value::Int(node.op.args[2].parse::<i64>().map_err(|_| {
                format!(
                    "node `{}` has invalid lane width `{}`",
                    node.name, node.op.args[2]
                )
            })?),
        ])),
        "tensor" => Ok(Value::Tensor(parse_tensor_literal(node)?)),
        "fill" => {
            let (rows, cols) = parse_shape(node)?;
            let value = node.op.args[2].parse::<i64>().map_err(|_| {
                format!(
                    "node `{}` has invalid fill value `{}`",
                    node.name, node.op.args[2]
                )
            })?;
            Ok(Value::Tensor(TensorValue {
                rows,
                cols,
                elements: vec![value; rows * cols],
            }))
        }
        "matmul" => {
            let lhs = state.expect_tensor(&node.op.args[0])?;
            let rhs = state.expect_tensor(&node.op.args[1])?;
            Ok(Value::Tensor(matmul(lhs, rhs)?))
        }
        "add_bias" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            let bias = state.expect_tensor(&node.op.args[1])?;
            Ok(Value::Tensor(add_bias(input, bias)?))
        }
        "relu" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            Ok(Value::Tensor(TensorValue {
                rows: input.rows,
                cols: input.cols,
                elements: input
                    .elements
                    .iter()
                    .map(|value| (*value).max(0))
                    .collect(),
            }))
        }
        "print" => {
            let value = state.expect_value(&node.op.args[0])?.clone();
            state.push_resource_event(
                resource,
                format!(
                    "effect kernel.print @{} [{}]: {}",
                    node.resource, resource.kind.raw, value
                ),
            );
            Ok(Value::Unit)
        }
        other => Err(format!("unknown kernel instruction `{other}`")),
    }
}

fn require_kernel_resource(node: &Node, resource: &Resource) -> Result<(), String> {
    if resource.kind.is_family("kernel") || resource.kind.is_family("npu") {
        Ok(())
    } else {
        Err(format!(
            "node `{}` uses kernel mod on non-kernel resource `{}` ({})",
            node.name, resource.name, resource.kind.raw
        ))
    }
}

fn parse_shape(node: &Node) -> Result<(usize, usize), String> {
    let rows = node.op.args[0].parse::<usize>().map_err(|_| {
        format!("node `{}` has invalid rows `{}`", node.name, node.op.args[0])
    })?;
    let cols = node.op.args[1].parse::<usize>().map_err(|_| {
        format!("node `{}` has invalid cols `{}`", node.name, node.op.args[1])
    })?;
    if rows == 0 || cols == 0 {
        return Err(format!("node `{}` tensor shape must be non-zero", node.name));
    }
    Ok((rows, cols))
}

fn validate_tensor_literal(node: &Node) -> Result<(), String> {
    let (rows, cols) = parse_shape(node)?;
    let elements = parse_csv_elements(node, &node.op.args[2])?;
    if elements.len() != rows * cols {
        return Err(format!(
            "node `{}` expected {} tensor elements, got {}",
            node.name,
            rows * cols,
            elements.len()
        ));
    }
    Ok(())
}

fn parse_tensor_literal(node: &Node) -> Result<TensorValue, String> {
    let (rows, cols) = parse_shape(node)?;
    let elements = parse_csv_elements(node, &node.op.args[2])?;
    if elements.len() != rows * cols {
        return Err(format!(
            "node `{}` expected {} tensor elements, got {}",
            node.name,
            rows * cols,
            elements.len()
        ));
    }
    Ok(TensorValue {
        rows,
        cols,
        elements,
    })
}

fn parse_csv_elements(node: &Node, raw: &str) -> Result<Vec<i64>, String> {
    raw.split(',')
        .map(|part| {
            let value = part.trim();
            value.parse::<i64>().map_err(|_| {
                format!("node `{}` has invalid tensor literal element `{value}`", node.name)
            })
        })
        .collect()
}

fn matmul(lhs: &TensorValue, rhs: &TensorValue) -> Result<TensorValue, String> {
    if lhs.cols != rhs.rows {
        return Err(format!(
            "kernel.matmul shape mismatch: lhs is {}x{}, rhs is {}x{}",
            lhs.rows, lhs.cols, rhs.rows, rhs.cols
        ));
    }

    let mut elements = vec![0i64; lhs.rows * rhs.cols];
    for row in 0..lhs.rows {
        for col in 0..rhs.cols {
            let mut acc = 0i64;
            for k in 0..lhs.cols {
                acc += lhs.elements[row * lhs.cols + k] * rhs.elements[k * rhs.cols + col];
            }
            elements[row * rhs.cols + col] = acc;
        }
    }

    Ok(TensorValue {
        rows: lhs.rows,
        cols: rhs.cols,
        elements,
    })
}

fn add_bias(input: &TensorValue, bias: &TensorValue) -> Result<TensorValue, String> {
    if bias.rows == 1 && bias.cols == input.cols {
        let mut elements = input.elements.clone();
        for row in 0..input.rows {
            for col in 0..input.cols {
                elements[row * input.cols + col] += bias.elements[col];
            }
        }
        return Ok(TensorValue {
            rows: input.rows,
            cols: input.cols,
            elements,
        });
    }

    if bias.rows == input.rows && bias.cols == input.cols {
        let elements = input
            .elements
            .iter()
            .zip(&bias.elements)
            .map(|(lhs, rhs)| lhs + rhs)
            .collect();
        return Ok(TensorValue {
            rows: input.rows,
            cols: input.cols,
            elements,
        });
    }

    Err(format!(
        "kernel.add_bias shape mismatch: input is {}x{}, bias is {}x{}",
        input.rows, input.cols, bias.rows, bias.cols
    ))
}
