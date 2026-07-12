use super::arithmetic::{
    is_axis_scalar_instruction, is_tensor_binary_instruction, is_tensor_scalar_instruction,
    is_typed_scalar_instruction,
};
use super::{parse_kernel_flow_state, parse_shape, validate_tensor_literal};
use yir_core::{InstructionSemantics, Node, Resource};

pub(crate) fn describe_kernel_node(
    node: &Node,
    resource: &Resource,
) -> Result<InstructionSemantics, String> {
    require_kernel_resource(node, resource)?;

    match node.op.instruction.as_str() {
        "const_bool" => describe_bool_const(node),
        "const_i32" => describe_typed_const::<i32>(node, "i32"),
        "const_i64" => describe_typed_const::<i64>(node, "i64"),
        "const_f32" => describe_typed_const::<f32>(node, "f32"),
        "const_f64" => describe_typed_const::<f64>(node, "f64"),
        "target_config" => describe_target_config(node),
        "observe" => describe_observe(node),
        "is_config_ready" | "value" => describe_kernel_result_access(node),
        "tensor" => describe_tensor_literal(node),
        "fill" => describe_fill(node),
        "splat" => describe_splat(node),
        instruction
            if instruction == "matmul"
                || instruction == "add_bias"
                || is_tensor_binary_instruction(instruction) =>
        {
            describe_binary_tensor_op(node)
        }
        instruction if is_tensor_scalar_instruction(instruction) => describe_tensor_scalar_op(node),
        instruction if is_typed_scalar_instruction(instruction) => describe_typed_binary_op(node),
        "shape" | "rows" | "cols" | "row" | "col" => describe_unary_tensor_op(node),
        "element_at" => describe_element_at(node),
        "reshape" => describe_reshape(node),
        "slice" => describe_slice(node),
        "broadcast" => describe_broadcast(node),
        "transpose" | "reduce_sum" | "reduce_max" | "reduce_mean" | "reduce_min" | "argmax"
        | "argmin" | "sort" => describe_unary_tensor_op(node),
        "topk" => describe_topk(node),
        "topk_axis" => describe_topk_axis(node),
        "sort_axis" => describe_axis_op(node, "sort"),
        "reduce_sum_axis" | "reduce_max_axis" | "reduce_mean_axis" | "reduce_min_axis"
        | "argmax_axis" | "argmin_axis" => describe_axis_op(node, "reduce"),
        "relu_axis" => describe_axis_op(node, "map"),
        instruction if is_axis_scalar_instruction(instruction) => describe_axis_scalar_op(node),
        "relu" => describe_unary_tensor_op(node),
        "print" => describe_print(node),
        other => Err(format!("unknown kernel instruction `{other}`")),
    }
}

pub(crate) fn require_kernel_resource(node: &Node, resource: &Resource) -> Result<(), String> {
    if resource.kind.is_family("kernel") || resource.kind.is_family("npu") {
        Ok(())
    } else {
        Err(format!(
            "node `{}` uses kernel mod on non-kernel resource `{}` ({})",
            node.name, resource.name, resource.kind.raw
        ))
    }
}

fn describe_bool_const(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(node, 1, "kernel.const_bool <name> <resource> <value>")?;
    match node.op.args[0].as_str() {
        "true" | "false" => Ok(InstructionSemantics::pure(Vec::new())),
        other => Err(format!(
            "node `{}` has invalid bool literal `{other}`",
            node.name
        )),
    }
}

fn describe_typed_const<T>(node: &Node, ty: &str) -> Result<InstructionSemantics, String>
where
    T: std::str::FromStr,
{
    expect_arg_count(
        node,
        1,
        &format!("kernel.const_{ty} <name> <resource> <value>"),
    )?;
    node.op.args[0].parse::<T>().map_err(|_| {
        format!(
            "node `{}` has invalid {ty} literal `{}`",
            node.name, node.op.args[0]
        )
    })?;
    Ok(InstructionSemantics::pure(Vec::new()))
}

fn describe_target_config(node: &Node) -> Result<InstructionSemantics, String> {
    if node.op.args.len() != 3 && node.op.args.len() != 4 {
        return Err(format!(
            "node `{}` expects `kernel.target_config <name> <resource> <arch> <runtime> <lane_width> [backend_features]`",
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

fn describe_observe(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(node, 2, "kernel.observe <name> <resource> <input> <state>")?;
    parse_kernel_flow_state(&node.op.args[1]).map_err(|error| {
        format!(
            "node `{}` has invalid kernel observe state: {error}",
            node.name
        )
    })?;
    Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
}

fn describe_kernel_result_access(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        1,
        &format!("kernel.{} <name> <resource> <result>", node.op.instruction),
    )?;
    Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
}

fn describe_tensor_literal(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        3,
        "kernel.tensor <name> <resource> <rows> <cols> <csv-elements>",
    )?;
    validate_tensor_literal(node)?;
    Ok(InstructionSemantics::pure(Vec::new()))
}

fn describe_fill(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        3,
        "kernel.fill <name> <resource> <rows> <cols> <value>",
    )?;
    parse_shape(node)?;
    let dependencies = if node.op.args[2].parse::<i64>().is_ok() {
        Vec::new()
    } else {
        vec![node.op.args[2].clone()]
    };
    Ok(InstructionSemantics::pure(dependencies))
}

fn describe_splat(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        3,
        "kernel.splat <name> <resource> <rows> <cols> <scalar>",
    )?;
    parse_shape(node)?;
    Ok(InstructionSemantics::pure(vec![node.op.args[2].clone()]))
}

fn describe_binary_tensor_op(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        2,
        &format!(
            "kernel.{} <name> <resource> <lhs> <rhs>",
            node.op.instruction
        ),
    )?;
    Ok(InstructionSemantics::pure(node.op.args.clone()))
}

fn describe_tensor_scalar_op(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        2,
        &format!(
            "kernel.{} <name> <resource> <tensor> <scalar>",
            node.op.instruction
        ),
    )?;
    Ok(InstructionSemantics::pure(node.op.args.clone()))
}

fn describe_typed_binary_op(node: &Node) -> Result<InstructionSemantics, String> {
    describe_binary_tensor_op(node)
}

fn describe_unary_tensor_op(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        1,
        &format!("kernel.{} <name> <resource> <input>", node.op.instruction),
    )?;
    Ok(InstructionSemantics::pure(node.op.args.clone()))
}

fn describe_element_at(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        3,
        "kernel.element_at <name> <resource> <input> <row> <col>",
    )?;
    Ok(InstructionSemantics::pure(node.op.args.clone()))
}

fn describe_reshape(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        3,
        "kernel.reshape <name> <resource> <input> <rows> <cols>",
    )?;
    parse_usize_arg(node, 1, "rows")?;
    parse_usize_arg(node, 2, "cols")?;
    Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
}

fn describe_slice(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        5,
        "kernel.slice <name> <resource> <input> <row_offset> <col_offset> <rows> <cols>",
    )?;
    for (index, label) in [
        (1, "row_offset"),
        (2, "col_offset"),
        (3, "rows"),
        (4, "cols"),
    ] {
        parse_usize_arg(node, index, label)?;
    }
    Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
}

fn describe_broadcast(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        3,
        "kernel.broadcast <name> <resource> <input> <rows> <cols>",
    )?;
    parse_usize_arg(node, 1, "rows")?;
    parse_usize_arg(node, 2, "cols")?;
    Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
}

fn describe_topk(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(node, 2, "kernel.topk <name> <resource> <input> <k>")?;
    parse_usize_arg(node, 1, "k")?;
    Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
}

fn describe_topk_axis(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        3,
        "kernel.topk_axis <name> <resource> <input> <k> <axis>",
    )?;
    parse_usize_arg(node, 1, "k")?;
    match node.op.args[2].as_str() {
        "rows" | "cols" => Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()])),
        other => Err(format!(
            "node `{}` has invalid topk axis `{other}`; expected rows or cols",
            node.name
        )),
    }
}

fn describe_axis_op(node: &Node, axis_kind: &str) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        2,
        &format!(
            "kernel.{} <name> <resource> <input> <axis>",
            node.op.instruction
        ),
    )?;
    match node.op.args[1].as_str() {
        "rows" | "cols" => Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()])),
        other => Err(format!(
            "node `{}` has invalid {axis_kind} axis `{other}`; expected rows or cols",
            node.name
        )),
    }
}

fn describe_axis_scalar_op(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        3,
        &format!(
            "kernel.{} <name> <resource> <input> <axis> <scalar>",
            node.op.instruction
        ),
    )?;
    match node.op.args[1].as_str() {
        "rows" | "cols" => Ok(InstructionSemantics::pure(vec![
            node.op.args[0].clone(),
            node.op.args[2].clone(),
        ])),
        other => Err(format!(
            "node `{}` has invalid map axis `{other}`; expected rows or cols",
            node.name
        )),
    }
}

fn describe_print(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(node, 1, "kernel.print <name> <resource> <input>")?;
    Ok(InstructionSemantics::effect(node.op.args.clone()))
}

fn expect_arg_count(node: &Node, expected: usize, usage: &str) -> Result<(), String> {
    if node.op.args.len() == expected {
        Ok(())
    } else {
        Err(format!("node `{}` expects `{usage}`", node.name))
    }
}

fn parse_usize_arg(node: &Node, index: usize, label: &str) -> Result<usize, String> {
    node.op.args[index].parse::<usize>().map_err(|_| {
        format!(
            "node `{}` has invalid {} `{}`",
            node.name, label, node.op.args[index]
        )
    })
}
