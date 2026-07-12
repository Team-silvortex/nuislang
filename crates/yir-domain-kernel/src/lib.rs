mod arithmetic;
mod describe;
mod tensor_ops;

use arithmetic::{execute_arithmetic_node, expect_tensor_scalar_i64, is_arithmetic_instruction};
use describe::describe_kernel_node;
use tensor_ops::{
    add_bias, argmax_cols, argmax_rows, argmin_cols, argmin_rows, broadcast, extract_col,
    extract_row, matmul, reduce_max_cols, reduce_max_rows, reduce_mean_cols, reduce_mean_rows,
    reduce_min_cols, reduce_min_rows, reduce_sum_cols, reduce_sum_rows, reshape, slice, sort_cols,
    sort_rows, sort_tensor_flat, topk_cols, topk_rows, topk_tensor_flat, transpose,
};
use yir_core::{
    ExecutionState, InstructionSemantics, KernelFlowState, KernelResultHandle, Node, RegisteredMod,
    Resource, TensorValue, Value,
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

fn execute_kernel_node(
    node: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
) -> Result<Value, String> {
    if is_arithmetic_instruction(node.op.instruction.as_str()) {
        return execute_arithmetic_node(node, state);
    }

    match node.op.instruction.as_str() {
        "const_bool" => Ok(Value::Bool(match node.op.args[0].as_str() {
            "true" => true,
            "false" => false,
            other => {
                return Err(format!(
                    "node `{}` has invalid bool literal `{other}`",
                    node.name
                ))
            }
        })),
        "const_i32" => Ok(Value::I32(node.op.args[0].parse::<i32>().map_err(
            |_| {
                format!(
                    "node `{}` has invalid i32 literal `{}`",
                    node.name, node.op.args[0]
                )
            },
        )?)),
        "const_i64" => Ok(Value::Int(node.op.args[0].parse::<i64>().map_err(
            |_| {
                format!(
                    "node `{}` has invalid i64 literal `{}`",
                    node.name, node.op.args[0]
                )
            },
        )?)),
        "const_f32" => Ok(Value::F32(node.op.args[0].parse::<f32>().map_err(
            |_| {
                format!(
                    "node `{}` has invalid f32 literal `{}`",
                    node.name, node.op.args[0]
                )
            },
        )?)),
        "const_f64" => Ok(Value::F64(node.op.args[0].parse::<f64>().map_err(
            |_| {
                format!(
                    "node `{}` has invalid f64 literal `{}`",
                    node.name, node.op.args[0]
                )
            },
        )?)),
        "target_config" => {
            let mut values = vec![
                Value::Symbol(node.op.args[0].clone()),
                Value::Symbol(node.op.args[1].clone()),
                Value::Int(node.op.args[2].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid lane width `{}`",
                        node.name, node.op.args[2]
                    )
                })?),
            ];
            if let Some(features) = node.op.args.get(3) {
                values.push(Value::Symbol(features.clone()));
            }
            Ok(Value::Tuple(values))
        }
        "observe" => {
            let value = state.expect_value(&node.op.args[0])?.clone();
            let flow = parse_kernel_flow_state(&node.op.args[1])?;
            Ok(Value::KernelResult(KernelResultHandle {
                state: flow,
                value: Box::new(value),
            }))
        }
        "is_config_ready" => {
            let result = state.expect_kernel_result(&node.op.args[0])?;
            Ok(Value::Bool(matches!(
                result.state,
                KernelFlowState::ConfigReady
            )))
        }
        "value" => {
            let result = state.expect_kernel_result(&node.op.args[0])?;
            Ok((*result.value).clone())
        }
        "tensor" => Ok(Value::Tensor(parse_tensor_literal(node)?)),
        "fill" => {
            let (rows, cols) = parse_shape(node)?;
            let value = resolve_scalar_arg_i64(state, &node.op.args[2]).map_err(|_| {
                format!(
                    "node `{}` has invalid fill scalar `{}`",
                    node.name, node.op.args[2]
                )
            })?;
            Ok(Value::Tensor(TensorValue {
                rows,
                cols,
                elements: vec![value; rows * cols],
            }))
        }
        "splat" => {
            let (rows, cols) = parse_shape(node)?;
            let value = expect_tensor_scalar_i64(state.expect_value(&node.op.args[2])?)?;
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
            match state.expect_value(&node.op.args[1])? {
                Value::Tensor(bias) => Ok(Value::Tensor(add_bias(input, bias)?)),
                scalar => {
                    let bias = expect_tensor_scalar_i64(scalar)?;
                    Ok(Value::Tensor(TensorValue {
                        rows: input.rows,
                        cols: input.cols,
                        elements: input.elements.iter().map(|value| *value + bias).collect(),
                    }))
                }
            }
        }
        "shape" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            Ok(Value::Tuple(vec![
                Value::Int(input.rows as i64),
                Value::Int(input.cols as i64),
            ]))
        }
        "rows" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            Ok(Value::Int(input.rows as i64))
        }
        "cols" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            Ok(Value::Int(input.cols as i64))
        }
        "row" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            if input.rows == 0 {
                return Err("kernel.row cannot operate on empty tensor".to_owned());
            }
            Ok(Value::Tensor(extract_row(input, 0)))
        }
        "col" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            if input.cols == 0 {
                return Err("kernel.col cannot operate on empty tensor".to_owned());
            }
            Ok(Value::Tensor(extract_col(input, 0)))
        }
        "element_at" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            let row = state.expect_int(&node.op.args[1])?;
            let col = state.expect_int(&node.op.args[2])?;
            if row < 0 || col < 0 || row as usize >= input.rows || col as usize >= input.cols {
                return Err(format!(
                    "kernel.element_at index out of bounds: {}x{} tensor accessed at ({row}, {col})",
                    input.rows, input.cols
                ));
            }
            Ok(Value::Int(
                input.elements[row as usize * input.cols + col as usize],
            ))
        }
        "reshape" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            let rows = node.op.args[1].parse::<usize>().map_err(|_| {
                format!(
                    "node `{}` has invalid rows `{}`",
                    node.name, node.op.args[1]
                )
            })?;
            let cols = node.op.args[2].parse::<usize>().map_err(|_| {
                format!(
                    "node `{}` has invalid cols `{}`",
                    node.name, node.op.args[2]
                )
            })?;
            reshape(input, rows, cols).map(Value::Tensor)
        }
        "slice" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            let row_offset = node.op.args[1].parse::<usize>().map_err(|_| {
                format!(
                    "node `{}` has invalid row_offset `{}`",
                    node.name, node.op.args[1]
                )
            })?;
            let col_offset = node.op.args[2].parse::<usize>().map_err(|_| {
                format!(
                    "node `{}` has invalid col_offset `{}`",
                    node.name, node.op.args[2]
                )
            })?;
            let rows = node.op.args[3].parse::<usize>().map_err(|_| {
                format!(
                    "node `{}` has invalid rows `{}`",
                    node.name, node.op.args[3]
                )
            })?;
            let cols = node.op.args[4].parse::<usize>().map_err(|_| {
                format!(
                    "node `{}` has invalid cols `{}`",
                    node.name, node.op.args[4]
                )
            })?;
            slice(input, row_offset, col_offset, rows, cols).map(Value::Tensor)
        }
        "broadcast" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            let rows = node.op.args[1].parse::<usize>().map_err(|_| {
                format!(
                    "node `{}` has invalid rows `{}`",
                    node.name, node.op.args[1]
                )
            })?;
            let cols = node.op.args[2].parse::<usize>().map_err(|_| {
                format!(
                    "node `{}` has invalid cols `{}`",
                    node.name, node.op.args[2]
                )
            })?;
            broadcast(input, rows, cols).map(Value::Tensor)
        }
        "transpose" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            Ok(Value::Tensor(transpose(input)))
        }
        "reduce_sum" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            Ok(Value::Int(input.elements.iter().copied().sum()))
        }
        "reduce_max" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            let value = input
                .elements
                .iter()
                .copied()
                .max()
                .ok_or_else(|| "kernel.reduce_max cannot operate on empty tensor".to_owned())?;
            Ok(Value::Int(value))
        }
        "reduce_mean" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            let sum: i64 = input.elements.iter().copied().sum();
            Ok(Value::Int(sum / input.elements.len() as i64))
        }
        "reduce_min" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            let value = input
                .elements
                .iter()
                .copied()
                .min()
                .ok_or_else(|| "kernel.reduce_min cannot operate on empty tensor".to_owned())?;
            Ok(Value::Int(value))
        }
        "argmax" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            let (index, _) = input
                .elements
                .iter()
                .copied()
                .enumerate()
                .max_by_key(|(_, value)| *value)
                .ok_or_else(|| "kernel.argmax cannot operate on empty tensor".to_owned())?;
            Ok(Value::Int(index as i64))
        }
        "argmin" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            let (index, _) = input
                .elements
                .iter()
                .copied()
                .enumerate()
                .min_by_key(|(_, value)| *value)
                .ok_or_else(|| "kernel.argmin cannot operate on empty tensor".to_owned())?;
            Ok(Value::Int(index as i64))
        }
        "sort" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            Ok(Value::Tensor(sort_tensor_flat(input)))
        }
        "topk" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            let k = node.op.args[1]
                .parse::<usize>()
                .map_err(|_| format!("node `{}` has invalid k `{}`", node.name, node.op.args[1]))?;
            topk_tensor_flat(input, k).map(Value::Tensor)
        }
        "topk_axis" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            let k = node.op.args[1]
                .parse::<usize>()
                .map_err(|_| format!("node `{}` has invalid k `{}`", node.name, node.op.args[1]))?;
            match node.op.args[2].as_str() {
                "rows" => topk_rows(input, k).map(Value::Tensor),
                "cols" => topk_cols(input, k).map(Value::Tensor),
                other => Err(format!(
                    "node `{}` has invalid topk axis `{other}`; expected rows or cols",
                    node.name
                )),
            }
        }
        "sort_axis" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            match node.op.args[1].as_str() {
                "rows" => Ok(Value::Tensor(sort_rows(input))),
                "cols" => Ok(Value::Tensor(sort_cols(input))),
                other => Err(format!(
                    "node `{}` has invalid sort axis `{other}`; expected rows or cols",
                    node.name
                )),
            }
        }
        "reduce_sum_axis" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            match node.op.args[1].as_str() {
                "rows" => Ok(Value::Tensor(reduce_sum_rows(input))),
                "cols" => Ok(Value::Tensor(reduce_sum_cols(input))),
                other => Err(format!(
                    "node `{}` has invalid reduce axis `{other}`; expected rows or cols",
                    node.name
                )),
            }
        }
        "reduce_max_axis" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            match node.op.args[1].as_str() {
                "rows" => Ok(Value::Tensor(reduce_max_rows(input)?)),
                "cols" => Ok(Value::Tensor(reduce_max_cols(input)?)),
                other => Err(format!(
                    "node `{}` has invalid reduce axis `{other}`; expected rows or cols",
                    node.name
                )),
            }
        }
        "reduce_mean_axis" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            match node.op.args[1].as_str() {
                "rows" => Ok(Value::Tensor(reduce_mean_rows(input))),
                "cols" => Ok(Value::Tensor(reduce_mean_cols(input))),
                other => Err(format!(
                    "node `{}` has invalid reduce axis `{other}`; expected rows or cols",
                    node.name
                )),
            }
        }
        "reduce_min_axis" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            match node.op.args[1].as_str() {
                "rows" => Ok(Value::Tensor(reduce_min_rows(input)?)),
                "cols" => Ok(Value::Tensor(reduce_min_cols(input)?)),
                other => Err(format!(
                    "node `{}` has invalid reduce axis `{other}`; expected rows or cols",
                    node.name
                )),
            }
        }
        "argmax_axis" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            match node.op.args[1].as_str() {
                "rows" => Ok(Value::Tensor(argmax_rows(input)?)),
                "cols" => Ok(Value::Tensor(argmax_cols(input)?)),
                other => Err(format!(
                    "node `{}` has invalid reduce axis `{other}`; expected rows or cols",
                    node.name
                )),
            }
        }
        "argmin_axis" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            match node.op.args[1].as_str() {
                "rows" => Ok(Value::Tensor(argmin_rows(input)?)),
                "cols" => Ok(Value::Tensor(argmin_cols(input)?)),
                other => Err(format!(
                    "node `{}` has invalid reduce axis `{other}`; expected rows or cols",
                    node.name
                )),
            }
        }
        "relu_axis" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            match node.op.args[1].as_str() {
                "rows" | "cols" => Ok(Value::Tensor(TensorValue {
                    rows: input.rows,
                    cols: input.cols,
                    elements: input.elements.iter().map(|value| (*value).max(0)).collect(),
                })),
                other => Err(format!(
                    "node `{}` has invalid map axis `{other}`; expected rows or cols",
                    node.name
                )),
            }
        }
        "relu" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            Ok(Value::Tensor(TensorValue {
                rows: input.rows,
                cols: input.cols,
                elements: input.elements.iter().map(|value| (*value).max(0)).collect(),
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

fn parse_kernel_flow_state(raw: &str) -> Result<KernelFlowState, String> {
    match raw {
        "config_ready" => Ok(KernelFlowState::ConfigReady),
        other => Err(format!("unknown kernel flow state `{other}`")),
    }
}

fn parse_shape(node: &Node) -> Result<(usize, usize), String> {
    let rows = node.op.args[0].parse::<usize>().map_err(|_| {
        format!(
            "node `{}` has invalid rows `{}`",
            node.name, node.op.args[0]
        )
    })?;
    let cols = node.op.args[1].parse::<usize>().map_err(|_| {
        format!(
            "node `{}` has invalid cols `{}`",
            node.name, node.op.args[1]
        )
    })?;
    if rows == 0 || cols == 0 {
        return Err(format!(
            "node `{}` tensor shape must be non-zero",
            node.name
        ));
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
                format!(
                    "node `{}` has invalid tensor literal element `{value}`",
                    node.name
                )
            })
        })
        .collect()
}

fn resolve_scalar_arg_i64(state: &ExecutionState, raw: &str) -> Result<i64, String> {
    match raw.parse::<i64>() {
        Ok(value) => Ok(value),
        Err(_) => expect_tensor_scalar_i64(state.expect_value(raw)?),
    }
}
