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

fn describe_kernel_node(node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
    require_kernel_resource(node, resource)?;

    match node.op.instruction.as_str() {
        "const_bool" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `kernel.const_bool <name> <resource> <value>`",
                    node.name
                ));
            }
            match node.op.args[0].as_str() {
                "true" | "false" => Ok(InstructionSemantics::pure(Vec::new())),
                other => Err(format!(
                    "node `{}` has invalid bool literal `{other}`",
                    node.name
                )),
            }
        }
        "const_i32" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `kernel.const_i32 <name> <resource> <value>`",
                    node.name
                ));
            }
            node.op.args[0].parse::<i32>().map_err(|_| {
                format!(
                    "node `{}` has invalid i32 literal `{}`",
                    node.name, node.op.args[0]
                )
            })?;
            Ok(InstructionSemantics::pure(Vec::new()))
        }
        "const_i64" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `kernel.const_i64 <name> <resource> <value>`",
                    node.name
                ));
            }
            node.op.args[0].parse::<i64>().map_err(|_| {
                format!(
                    "node `{}` has invalid i64 literal `{}`",
                    node.name, node.op.args[0]
                )
            })?;
            Ok(InstructionSemantics::pure(Vec::new()))
        }
        "const_f32" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `kernel.const_f32 <name> <resource> <value>`",
                    node.name
                ));
            }
            node.op.args[0].parse::<f32>().map_err(|_| {
                format!(
                    "node `{}` has invalid f32 literal `{}`",
                    node.name, node.op.args[0]
                )
            })?;
            Ok(InstructionSemantics::pure(Vec::new()))
        }
        "const_f64" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `kernel.const_f64 <name> <resource> <value>`",
                    node.name
                ));
            }
            node.op.args[0].parse::<f64>().map_err(|_| {
                format!(
                    "node `{}` has invalid f64 literal `{}`",
                    node.name, node.op.args[0]
                )
            })?;
            Ok(InstructionSemantics::pure(Vec::new()))
        }
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
            let dependencies = if node.op.args[2].parse::<i64>().is_ok() {
                Vec::new()
            } else {
                vec![node.op.args[2].clone()]
            };
            Ok(InstructionSemantics::pure(dependencies))
        }
        "splat" => {
            if node.op.args.len() != 3 {
                return Err(format!(
                    "node `{}` expects `kernel.splat <name> <resource> <rows> <cols> <scalar>`",
                    node.name
                ));
            }
            parse_shape(node)?;
            Ok(InstructionSemantics::pure(vec![node.op.args[2].clone()]))
        }
        "matmul" | "add_bias" | "add" | "mul" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `kernel.{} <name> <resource> <lhs> <rhs>`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::pure(node.op.args.clone()))
        }
        "add_scalar" | "mul_scalar" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `kernel.{} <name> <resource> <tensor> <scalar>`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::pure(node.op.args.clone()))
        }
        "add_i32" | "mul_i32" | "add_f32" | "mul_f32" | "add_f64" | "mul_f64" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `kernel.{} <name> <resource> <lhs> <rhs>`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::pure(node.op.args.clone()))
        }
        "shape" | "rows" | "cols" | "row" | "col" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `kernel.{} <name> <resource> <input>`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::pure(node.op.args.clone()))
        }
        "element_at" => {
            if node.op.args.len() != 3 {
                return Err(format!(
                    "node `{}` expects `kernel.element_at <name> <resource> <input> <row> <col>`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::pure(node.op.args.clone()))
        }
        "reshape" => {
            if node.op.args.len() != 3 {
                return Err(format!(
                    "node `{}` expects `kernel.reshape <name> <resource> <input> <rows> <cols>`",
                    node.name
                ));
            }
            node.op.args[1].parse::<usize>().map_err(|_| {
                format!(
                    "node `{}` has invalid rows `{}`",
                    node.name, node.op.args[1]
                )
            })?;
            node.op.args[2].parse::<usize>().map_err(|_| {
                format!(
                    "node `{}` has invalid cols `{}`",
                    node.name, node.op.args[2]
                )
            })?;
            Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
        }
        "slice" => {
            if node.op.args.len() != 5 {
                return Err(format!(
                    "node `{}` expects `kernel.slice <name> <resource> <input> <row_offset> <col_offset> <rows> <cols>`",
                    node.name
                ));
            }
            for (index, label) in [
                (1, "row_offset"),
                (2, "col_offset"),
                (3, "rows"),
                (4, "cols"),
            ] {
                node.op.args[index].parse::<usize>().map_err(|_| {
                    format!(
                        "node `{}` has invalid {} `{}`",
                        node.name, label, node.op.args[index]
                    )
                })?;
            }
            Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
        }
        "broadcast" => {
            if node.op.args.len() != 3 {
                return Err(format!(
                    "node `{}` expects `kernel.broadcast <name> <resource> <input> <rows> <cols>`",
                    node.name
                ));
            }
            node.op.args[1].parse::<usize>().map_err(|_| {
                format!(
                    "node `{}` has invalid rows `{}`",
                    node.name, node.op.args[1]
                )
            })?;
            node.op.args[2].parse::<usize>().map_err(|_| {
                format!(
                    "node `{}` has invalid cols `{}`",
                    node.name, node.op.args[2]
                )
            })?;
            Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
        }
        "transpose" | "reduce_sum" | "reduce_max" | "reduce_mean" | "reduce_min" | "argmax"
        | "argmin" | "sort" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `kernel.{} <name> <resource> <input>`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::pure(node.op.args.clone()))
        }
        "topk" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `kernel.topk <name> <resource> <input> <k>`",
                    node.name
                ));
            }
            node.op.args[1]
                .parse::<usize>()
                .map_err(|_| format!("node `{}` has invalid k `{}`", node.name, node.op.args[1]))?;
            Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
        }
        "topk_axis" => {
            if node.op.args.len() != 3 {
                return Err(format!(
                    "node `{}` expects `kernel.topk_axis <name> <resource> <input> <k> <axis>`",
                    node.name
                ));
            }
            node.op.args[1]
                .parse::<usize>()
                .map_err(|_| format!("node `{}` has invalid k `{}`", node.name, node.op.args[1]))?;
            match node.op.args[2].as_str() {
                "rows" | "cols" => Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()])),
                other => Err(format!(
                    "node `{}` has invalid topk axis `{other}`; expected rows or cols",
                    node.name
                )),
            }
        }
        "reduce_sum_axis" | "reduce_max_axis" | "reduce_mean_axis" | "reduce_min_axis"
        | "argmax_axis" | "argmin_axis" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `kernel.{} <name> <resource> <input> <axis>`",
                    node.name, node.op.instruction
                ));
            }
            match node.op.args[1].as_str() {
                "rows" | "cols" => Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()])),
                other => Err(format!(
                    "node `{}` has invalid reduce axis `{other}`; expected rows or cols",
                    node.name
                )),
            }
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
        "add" => {
            let lhs = state.expect_tensor(&node.op.args[0])?;
            let rhs = state.expect_tensor(&node.op.args[1])?;
            Ok(Value::Tensor(elementwise_binary_broadcast(
                node,
                lhs,
                rhs,
                |lhs, rhs| lhs + rhs,
            )?))
        }
        "mul" => {
            let lhs = state.expect_tensor(&node.op.args[0])?;
            let rhs = state.expect_tensor(&node.op.args[1])?;
            Ok(Value::Tensor(elementwise_binary_broadcast(
                node,
                lhs,
                rhs,
                |lhs, rhs| lhs * rhs,
            )?))
        }
        "add_scalar" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            let scalar = expect_tensor_scalar_i64(state.expect_value(&node.op.args[1])?)?;
            Ok(Value::Tensor(TensorValue {
                rows: input.rows,
                cols: input.cols,
                elements: input.elements.iter().map(|value| *value + scalar).collect(),
            }))
        }
        "mul_scalar" => {
            let input = state.expect_tensor(&node.op.args[0])?;
            let scalar = expect_tensor_scalar_i64(state.expect_value(&node.op.args[1])?)?;
            Ok(Value::Tensor(TensorValue {
                rows: input.rows,
                cols: input.cols,
                elements: input.elements.iter().map(|value| *value * scalar).collect(),
            }))
        }
        "add_i32" => Ok(Value::I32(
            state.expect_i32(&node.op.args[0])? + state.expect_i32(&node.op.args[1])?,
        )),
        "mul_i32" => Ok(Value::I32(
            state.expect_i32(&node.op.args[0])? * state.expect_i32(&node.op.args[1])?,
        )),
        "add_f32" => Ok(Value::F32(
            state.expect_f32(&node.op.args[0])? + state.expect_f32(&node.op.args[1])?,
        )),
        "mul_f32" => Ok(Value::F32(
            state.expect_f32(&node.op.args[0])? * state.expect_f32(&node.op.args[1])?,
        )),
        "add_f64" => Ok(Value::F64(
            state.expect_f64(&node.op.args[0])? + state.expect_f64(&node.op.args[1])?,
        )),
        "mul_f64" => Ok(Value::F64(
            state.expect_f64(&node.op.args[0])? * state.expect_f64(&node.op.args[1])?,
        )),
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

fn expect_tensor_scalar_i64(value: &Value) -> Result<i64, String> {
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

fn reshape(input: &TensorValue, rows: usize, cols: usize) -> Result<TensorValue, String> {
    if rows == 0 || cols == 0 {
        return Err("kernel.reshape requires non-zero target shape".to_owned());
    }
    if rows * cols != input.elements.len() {
        return Err(format!(
            "kernel.reshape element mismatch: input has {} elements, target shape is {}x{}",
            input.elements.len(),
            rows,
            cols
        ));
    }
    Ok(TensorValue {
        rows,
        cols,
        elements: input.elements.clone(),
    })
}

fn slice(
    input: &TensorValue,
    row_offset: usize,
    col_offset: usize,
    rows: usize,
    cols: usize,
) -> Result<TensorValue, String> {
    if rows == 0 || cols == 0 {
        return Err("kernel.slice requires non-zero slice shape".to_owned());
    }
    if row_offset + rows > input.rows || col_offset + cols > input.cols {
        return Err(format!(
            "kernel.slice out of bounds: {}x{} tensor cannot provide slice ({}, {}) + {}x{}",
            input.rows, input.cols, row_offset, col_offset, rows, cols
        ));
    }

    let mut elements = Vec::with_capacity(rows * cols);
    for row in row_offset..(row_offset + rows) {
        let start = row * input.cols + col_offset;
        let end = start + cols;
        elements.extend_from_slice(&input.elements[start..end]);
    }

    Ok(TensorValue {
        rows,
        cols,
        elements,
    })
}

fn extract_row(input: &TensorValue, row: usize) -> TensorValue {
    let start = row * input.cols;
    let end = start + input.cols;
    TensorValue {
        rows: 1,
        cols: input.cols,
        elements: input.elements[start..end].to_vec(),
    }
}

fn extract_col(input: &TensorValue, col: usize) -> TensorValue {
    TensorValue {
        rows: input.rows,
        cols: 1,
        elements: (0..input.rows)
            .map(|row| input.elements[row * input.cols + col])
            .collect(),
    }
}

fn broadcast(input: &TensorValue, rows: usize, cols: usize) -> Result<TensorValue, String> {
    if rows == 0 || cols == 0 {
        return Err("kernel.broadcast requires non-zero target shape".to_owned());
    }
    if input.rows == rows && input.cols == cols {
        return Ok(input.clone());
    }

    let row_compatible = input.rows == 1 || input.rows == rows;
    let col_compatible = input.cols == 1 || input.cols == cols;
    if !row_compatible || !col_compatible {
        return Err(format!(
            "kernel.broadcast shape mismatch: cannot broadcast {}x{} to {}x{}",
            input.rows, input.cols, rows, cols
        ));
    }

    let mut elements = Vec::with_capacity(rows * cols);
    for row in 0..rows {
        let src_row = if input.rows == 1 { 0 } else { row };
        for col in 0..cols {
            let src_col = if input.cols == 1 { 0 } else { col };
            elements.push(input.elements[src_row * input.cols + src_col]);
        }
    }

    Ok(TensorValue {
        rows,
        cols,
        elements,
    })
}

fn reduce_sum_rows(input: &TensorValue) -> TensorValue {
    TensorValue {
        rows: input.rows,
        cols: 1,
        elements: (0..input.rows)
            .map(|row| {
                let start = row * input.cols;
                let end = start + input.cols;
                input.elements[start..end].iter().copied().sum()
            })
            .collect(),
    }
}

fn reduce_max_rows(input: &TensorValue) -> Result<TensorValue, String> {
    let elements = (0..input.rows)
        .map(|row| {
            let start = row * input.cols;
            let end = start + input.cols;
            input.elements[start..end]
                .iter()
                .copied()
                .max()
                .ok_or_else(|| "kernel.reduce_max_axis cannot operate on empty tensor".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TensorValue {
        rows: input.rows,
        cols: 1,
        elements,
    })
}

fn reduce_min_rows(input: &TensorValue) -> Result<TensorValue, String> {
    let elements = (0..input.rows)
        .map(|row| {
            let start = row * input.cols;
            let end = start + input.cols;
            input.elements[start..end]
                .iter()
                .copied()
                .min()
                .ok_or_else(|| "kernel.reduce_min_axis cannot operate on empty tensor".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TensorValue {
        rows: input.rows,
        cols: 1,
        elements,
    })
}

fn reduce_sum_cols(input: &TensorValue) -> TensorValue {
    TensorValue {
        rows: 1,
        cols: input.cols,
        elements: (0..input.cols)
            .map(|col| {
                (0..input.rows)
                    .map(|row| input.elements[row * input.cols + col])
                    .sum()
            })
            .collect(),
    }
}

fn reduce_max_cols(input: &TensorValue) -> Result<TensorValue, String> {
    let elements = (0..input.cols)
        .map(|col| {
            (0..input.rows)
                .map(|row| input.elements[row * input.cols + col])
                .max()
                .ok_or_else(|| "kernel.reduce_max_axis cannot operate on empty tensor".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TensorValue {
        rows: 1,
        cols: input.cols,
        elements,
    })
}

fn reduce_min_cols(input: &TensorValue) -> Result<TensorValue, String> {
    let elements = (0..input.cols)
        .map(|col| {
            (0..input.rows)
                .map(|row| input.elements[row * input.cols + col])
                .min()
                .ok_or_else(|| "kernel.reduce_min_axis cannot operate on empty tensor".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TensorValue {
        rows: 1,
        cols: input.cols,
        elements,
    })
}

fn reduce_mean_rows(input: &TensorValue) -> TensorValue {
    TensorValue {
        rows: input.rows,
        cols: 1,
        elements: (0..input.rows)
            .map(|row| {
                let start = row * input.cols;
                let end = start + input.cols;
                let sum: i64 = input.elements[start..end].iter().copied().sum();
                sum / input.cols as i64
            })
            .collect(),
    }
}

fn reduce_mean_cols(input: &TensorValue) -> TensorValue {
    TensorValue {
        rows: 1,
        cols: input.cols,
        elements: (0..input.cols)
            .map(|col| {
                let sum: i64 = (0..input.rows)
                    .map(|row| input.elements[row * input.cols + col])
                    .sum();
                sum / input.rows as i64
            })
            .collect(),
    }
}

fn argmax_rows(input: &TensorValue) -> Result<TensorValue, String> {
    let elements = (0..input.rows)
        .map(|row| {
            let start = row * input.cols;
            let end = start + input.cols;
            input.elements[start..end]
                .iter()
                .copied()
                .enumerate()
                .max_by_key(|(_, value)| *value)
                .map(|(index, _)| index as i64)
                .ok_or_else(|| "kernel.argmax_axis cannot operate on empty tensor".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TensorValue {
        rows: input.rows,
        cols: 1,
        elements,
    })
}

fn argmax_cols(input: &TensorValue) -> Result<TensorValue, String> {
    let elements = (0..input.cols)
        .map(|col| {
            (0..input.rows)
                .map(|row| (row, input.elements[row * input.cols + col]))
                .max_by_key(|(_, value)| *value)
                .map(|(row, _)| row as i64)
                .ok_or_else(|| "kernel.argmax_axis cannot operate on empty tensor".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TensorValue {
        rows: 1,
        cols: input.cols,
        elements,
    })
}

fn argmin_rows(input: &TensorValue) -> Result<TensorValue, String> {
    let elements = (0..input.rows)
        .map(|row| {
            let start = row * input.cols;
            let end = start + input.cols;
            input.elements[start..end]
                .iter()
                .copied()
                .enumerate()
                .min_by_key(|(_, value)| *value)
                .map(|(index, _)| index as i64)
                .ok_or_else(|| "kernel.argmin_axis cannot operate on empty tensor".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TensorValue {
        rows: input.rows,
        cols: 1,
        elements,
    })
}

fn argmin_cols(input: &TensorValue) -> Result<TensorValue, String> {
    let elements = (0..input.cols)
        .map(|col| {
            (0..input.rows)
                .map(|row| (row, input.elements[row * input.cols + col]))
                .min_by_key(|(_, value)| *value)
                .map(|(row, _)| row as i64)
                .ok_or_else(|| "kernel.argmin_axis cannot operate on empty tensor".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TensorValue {
        rows: 1,
        cols: input.cols,
        elements,
    })
}

fn sort_tensor_flat(input: &TensorValue) -> TensorValue {
    let mut elements = input.elements.clone();
    elements.sort_unstable();
    TensorValue {
        rows: 1,
        cols: elements.len(),
        elements,
    }
}

fn topk_tensor_flat(input: &TensorValue, k: usize) -> Result<TensorValue, String> {
    if k == 0 {
        return Err("kernel.topk requires k > 0".to_owned());
    }
    if k > input.elements.len() {
        return Err(format!(
            "kernel.topk requested {} values from tensor with only {} elements",
            k,
            input.elements.len()
        ));
    }
    let mut elements = input.elements.clone();
    elements.sort_unstable_by(|lhs, rhs| rhs.cmp(lhs));
    elements.truncate(k);
    Ok(TensorValue {
        rows: 1,
        cols: k,
        elements,
    })
}

fn topk_rows(input: &TensorValue, k: usize) -> Result<TensorValue, String> {
    if k == 0 {
        return Err("kernel.topk_axis requires k > 0".to_owned());
    }
    if k > input.cols {
        return Err(format!(
            "kernel.topk_axis requested top-{} across rows of width {}",
            k, input.cols
        ));
    }
    let mut elements = Vec::with_capacity(input.rows * k);
    for row in 0..input.rows {
        let start = row * input.cols;
        let end = start + input.cols;
        let mut row_values = input.elements[start..end].to_vec();
        row_values.sort_unstable_by(|lhs, rhs| rhs.cmp(lhs));
        row_values.truncate(k);
        elements.extend(row_values);
    }
    Ok(TensorValue {
        rows: input.rows,
        cols: k,
        elements,
    })
}

fn topk_cols(input: &TensorValue, k: usize) -> Result<TensorValue, String> {
    if k == 0 {
        return Err("kernel.topk_axis requires k > 0".to_owned());
    }
    if k > input.rows {
        return Err(format!(
            "kernel.topk_axis requested top-{} across cols of height {}",
            k, input.rows
        ));
    }
    let mut columns = Vec::with_capacity(input.cols * k);
    for col in 0..input.cols {
        let mut col_values = (0..input.rows)
            .map(|row| input.elements[row * input.cols + col])
            .collect::<Vec<_>>();
        col_values.sort_unstable_by(|lhs, rhs| rhs.cmp(lhs));
        col_values.truncate(k);
        columns.push(col_values);
    }

    let mut elements = Vec::with_capacity(k * input.cols);
    for rank in 0..k {
        for column in &columns {
            elements.push(column[rank]);
        }
    }
    Ok(TensorValue {
        rows: k,
        cols: input.cols,
        elements,
    })
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
    let broadcasted = broadcast(bias, input.rows, input.cols).map_err(|_| {
        format!(
            "kernel.add_bias shape mismatch: input is {}x{}, bias is {}x{}",
            input.rows, input.cols, bias.rows, bias.cols
        )
    })?;
    Ok(TensorValue {
        rows: input.rows,
        cols: input.cols,
        elements: input
            .elements
            .iter()
            .copied()
            .zip(broadcasted.elements)
            .map(|(lhs, rhs)| lhs + rhs)
            .collect(),
    })
}

fn elementwise_binary_broadcast(
    node: &Node,
    lhs: &TensorValue,
    rhs: &TensorValue,
    op: impl Fn(i64, i64) -> i64,
) -> Result<TensorValue, String> {
    let rows = lhs.rows.max(rhs.rows);
    let cols = lhs.cols.max(rhs.cols);
    let lhs = broadcast(lhs, rows, cols).map_err(|_| {
        format!(
            "node `{}` expects broadcast-compatible tensor shapes, got {}x{} and {}x{}",
            node.name, lhs.rows, lhs.cols, rhs.rows, rhs.cols
        )
    })?;
    let rhs = broadcast(rhs, rows, cols).map_err(|_| {
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
            .collect(),
    })
}

fn transpose(input: &TensorValue) -> TensorValue {
    let mut elements = vec![0; input.rows * input.cols];
    for row in 0..input.rows {
        for col in 0..input.cols {
            elements[col * input.rows + row] = input.elements[row * input.cols + col];
        }
    }

    TensorValue {
        rows: input.cols,
        cols: input.rows,
        elements,
    }
}
