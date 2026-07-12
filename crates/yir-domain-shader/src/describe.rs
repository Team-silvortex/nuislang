use super::{
    parse_shader_flow_state,
    texture_sampling::{
        parse_bool_flag, parse_csv_indices, parse_csv_ints, validate_texture_literal,
    },
};
use yir_core::{InstructionSemantics, Node, Resource};

pub(crate) fn describe_shader_node(
    node: &Node,
    resource: &Resource,
) -> Result<InstructionSemantics, String> {
    require_shader_resource(node, resource)?;

    match node.op.instruction.as_str() {
        "target_config" => describe_target_config(node),
        "const" => describe_int_const(node),
        "const_bool" => describe_bool_const(node),
        "const_i32" => describe_typed_const::<i32>(node, "i32"),
        "const_i64" => describe_typed_const::<i64>(node, "i64"),
        "const_f32" => describe_typed_const::<f32>(node, "f32"),
        "const_f64" => describe_typed_const::<f64>(node, "f64"),
        "add" | "sub" | "mul" => describe_binary_op(node),
        "add_i32" | "mul_i32" | "add_f32" | "mul_f32" | "add_f64" | "mul_f64" => {
            describe_binary_op(node)
        }
        "target" => describe_target(node),
        "viewport" => describe_viewport(node),
        "pipeline" => describe_fixed_pure(
            node,
            2,
            "shader.pipeline <name> <resource> <shading_model> <topology>",
        ),
        "inline_wgsl" => describe_inline_wgsl(node),
        "vertex_layout" => describe_vertex_layout(node),
        "vertex_buffer" => describe_vertex_buffer(node),
        "index_buffer" => describe_index_buffer(node),
        "blend_state" => describe_blend_state(node),
        "depth_state" => describe_depth_state(node),
        "raster_state" => describe_fixed_pure(
            node,
            2,
            "shader.raster_state <name> <resource> <cull_mode> <front_face>",
        ),
        "render_state" => describe_fixed_pure_deps(
            node,
            4,
            "shader.render_state <name> <resource> <pipeline> <blend> <depth> <raster>",
        ),
        "uv" => describe_uv(node),
        "texture2d" => describe_texture2d(node),
        "sampler" => describe_fixed_pure(
            node,
            2,
            "shader.sampler <name> <resource> <filter> <address_mode>",
        ),
        "pack_ball_state" => describe_fixed_pure_deps(
            node,
            2,
            "shader.pack_ball_state <name> <resource> <color> <speed>",
        ),
        "begin_pass" => describe_fixed_pure_deps(
            node,
            3,
            "shader.begin_pass <name> <resource> <target> <pipeline> <viewport>",
        ),
        "observe" => describe_observe(node),
        "is_pass_ready" | "is_frame_ready" | "value" => describe_result_access(node),
        "uniform"
        | "storage"
        | "attachment"
        | "texture_binding"
        | "sampler_binding"
        | "vertex_layout_binding"
        | "vertex_binding"
        | "index_binding" => describe_binding(node),
        "bind_set" => describe_bind_set(node),
        "clear" => describe_clear(node),
        "overlay" => {
            describe_fixed_effect(node, 2, "shader.overlay <name> <resource> <base> <top>")
        }
        "sample" | "sample_nearest" => describe_fixed_pure_deps(
            node,
            4,
            &format!(
                "shader.{} <name> <resource> <texture> <sampler> <x> <y>",
                node.op.instruction
            ),
        ),
        "sample_uv" | "sample_uv_nearest" | "sample_uv_linear" => describe_fixed_pure_deps(
            node,
            3,
            &format!(
                "shader.{} <name> <resource> <texture> <sampler> <uv>",
                node.op.instruction
            ),
        ),
        "dispatch" => describe_fixed_effect(node, 1, "shader.dispatch <name> <resource> <input>"),
        "draw_instanced" => describe_draw_instanced(node),
        "draw_ball" => {
            describe_fixed_effect(node, 1, "shader.draw_ball <name> <resource> <packet>")
        }
        "draw_sphere" => {
            describe_fixed_effect(node, 1, "shader.draw_sphere <name> <resource> <packet>")
        }
        "print" => describe_fixed_effect(node, 1, "shader.print <name> <resource> <input>"),
        other => Err(format!("unknown shader instruction `{other}`")),
    }
}

fn require_shader_resource(node: &Node, resource: &Resource) -> Result<(), String> {
    if resource.kind.is_family("shader") {
        Ok(())
    } else {
        Err(format!(
            "node `{}` uses shader mod on non-shader resource `{}` ({})",
            node.name, resource.name, resource.kind.raw
        ))
    }
}

fn describe_target_config(node: &Node) -> Result<InstructionSemantics, String> {
    if node.op.args.len() != 3 && node.op.args.len() != 4 {
        return Err(format!(
            "node `{}` expects `shader.target_config <name> <resource> <arch> <runtime> <lane_width> [backend_features]`",
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

fn describe_int_const(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(node, 1, "shader.const <name> <resource> <value>")?;
    node.op.args[0].parse::<i64>().map_err(|_| {
        format!(
            "node `{}` has invalid integer literal `{}`",
            node.name, node.op.args[0]
        )
    })?;
    Ok(InstructionSemantics::pure(Vec::new()))
}

fn describe_bool_const(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(node, 1, "shader.const_bool <name> <resource> <value>")?;
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
        &format!("shader.const_{ty} <name> <resource> <value>"),
    )?;
    node.op.args[0].parse::<T>().map_err(|_| {
        format!(
            "node `{}` has invalid {ty} literal `{}`",
            node.name, node.op.args[0]
        )
    })?;
    Ok(InstructionSemantics::pure(Vec::new()))
}

fn describe_binary_op(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        2,
        &format!(
            "shader.{} <name> <resource> <lhs> <rhs>",
            node.op.instruction
        ),
    )?;
    Ok(InstructionSemantics::pure(node.op.args.clone()))
}

fn describe_target(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        3,
        "shader.target <name> <resource> <format> <width> <height>",
    )?;
    parse_i64_arg(node, 1, "width")?;
    parse_i64_arg(node, 2, "height")?;
    Ok(InstructionSemantics::pure(Vec::new()))
}

fn describe_viewport(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        2,
        "shader.viewport <name> <resource> <width> <height>",
    )?;
    parse_i64_arg(node, 0, "width")?;
    parse_i64_arg(node, 1, "height")?;
    Ok(InstructionSemantics::pure(Vec::new()))
}

fn describe_inline_wgsl(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        2,
        "shader.inline_wgsl <name> <resource> <entry> <source>",
    )?;
    let entry = node.op.args[0].trim();
    let source = node.op.args[1].trim();
    if entry.is_empty() {
        return Err(format!("node `{}` has empty inline_wgsl entry", node.name));
    }
    if source.is_empty() {
        return Err(format!("node `{}` has empty inline_wgsl source", node.name));
    }
    if !source.contains("@vertex") || !source.contains("@fragment") {
        return Err(format!(
            "node `{}` inline_wgsl source must contain both @vertex and @fragment",
            node.name
        ));
    }
    Ok(InstructionSemantics::pure(Vec::new()))
}

fn describe_vertex_layout(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        2,
        "shader.vertex_layout <name> <resource> <stride> <csv-attributes>",
    )?;
    node.op.args[0].parse::<usize>().map_err(|_| {
        format!(
            "node `{}` has invalid vertex stride `{}`",
            node.name, node.op.args[0]
        )
    })?;
    Ok(InstructionSemantics::pure(Vec::new()))
}

fn describe_vertex_buffer(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        2,
        "shader.vertex_buffer <name> <resource> <vertex_count> <csv-elements>",
    )?;
    node.op.args[0].parse::<usize>().map_err(|_| {
        format!(
            "node `{}` has invalid vertex count `{}`",
            node.name, node.op.args[0]
        )
    })?;
    parse_csv_ints(node, &node.op.args[1], "vertex element")?;
    Ok(InstructionSemantics::pure(Vec::new()))
}

fn describe_index_buffer(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        1,
        "shader.index_buffer <name> <resource> <csv-indices>",
    )?;
    parse_csv_indices(node, &node.op.args[0])?;
    Ok(InstructionSemantics::pure(Vec::new()))
}

fn describe_blend_state(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        2,
        "shader.blend_state <name> <resource> <enabled> <mode>",
    )?;
    parse_bool_flag(node, 0, "blend enabled")?;
    Ok(InstructionSemantics::pure(Vec::new()))
}

fn describe_depth_state(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        3,
        "shader.depth_state <name> <resource> <test_enabled> <write_enabled> <compare>",
    )?;
    parse_bool_flag(node, 0, "depth test")?;
    parse_bool_flag(node, 1, "depth write")?;
    Ok(InstructionSemantics::pure(Vec::new()))
}

fn describe_uv(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(node, 2, "shader.uv <name> <resource> <u_1024> <v_1024>")?;
    parse_i64_arg(node, 0, "u coord")?;
    parse_i64_arg(node, 1, "v coord")?;
    Ok(InstructionSemantics::pure(Vec::new()))
}

fn describe_texture2d(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        4,
        "shader.texture2d <name> <resource> <format> <width> <height> <csv-texels>",
    )?;
    validate_texture_literal(node)?;
    Ok(InstructionSemantics::pure(Vec::new()))
}

fn describe_observe(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(node, 2, "shader.observe <name> <resource> <input> <state>")?;
    parse_shader_flow_state(&node.op.args[1]).map_err(|error| {
        format!(
            "node `{}` has invalid shader observe state: {error}",
            node.name
        )
    })?;
    Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
}

fn describe_result_access(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        1,
        &format!("shader.{} <name> <resource> <result>", node.op.instruction),
    )?;
    Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
}

fn describe_binding(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(
        node,
        2,
        &format!(
            "shader.{} <name> <resource> <slot> <value>",
            node.op.instruction
        ),
    )?;
    node.op.args[0].parse::<usize>().map_err(|_| {
        format!(
            "node `{}` has invalid binding slot `{}`",
            node.name, node.op.args[0]
        )
    })?;
    Ok(InstructionSemantics::pure(vec![node.op.args[1].clone()]))
}

fn describe_bind_set(node: &Node) -> Result<InstructionSemantics, String> {
    if node.op.args.len() < 2 {
        return Err(format!(
            "node `{}` expects `shader.bind_set <name> <resource> <pipeline> <binding> [binding...]`",
            node.name
        ));
    }
    Ok(InstructionSemantics::pure(node.op.args.clone()))
}

fn describe_clear(node: &Node) -> Result<InstructionSemantics, String> {
    expect_arg_count(node, 2, "shader.clear <name> <resource> <target> <fill>")?;
    parse_i64_arg(node, 1, "clear fill")?;
    Ok(InstructionSemantics::effect(vec![node.op.args[0].clone()]))
}

fn describe_draw_instanced(node: &Node) -> Result<InstructionSemantics, String> {
    if !(node.op.args.len() == 4 || node.op.args.len() == 5) {
        return Err(format!(
            "node `{}` expects `shader.draw_instanced <name> <resource> <pass> <packet> <vertex_count> <instance_count> [bind_set]`",
            node.name
        ));
    }
    let mut deps = vec![node.op.args[0].clone(), node.op.args[1].clone()];
    if node.op.args[2].parse::<i64>().is_err() {
        deps.push(node.op.args[2].clone());
    }
    if node.op.args[3].parse::<i64>().is_err() {
        deps.push(node.op.args[3].clone());
    }
    if let Some(bind_set) = node.op.args.get(4) {
        deps.push(bind_set.clone());
    }
    Ok(InstructionSemantics::effect(deps))
}

fn describe_fixed_pure(
    node: &Node,
    expected: usize,
    usage: &str,
) -> Result<InstructionSemantics, String> {
    expect_arg_count(node, expected, usage)?;
    Ok(InstructionSemantics::pure(Vec::new()))
}

fn describe_fixed_pure_deps(
    node: &Node,
    expected: usize,
    usage: &str,
) -> Result<InstructionSemantics, String> {
    expect_arg_count(node, expected, usage)?;
    Ok(InstructionSemantics::pure(node.op.args.clone()))
}

fn describe_fixed_effect(
    node: &Node,
    expected: usize,
    usage: &str,
) -> Result<InstructionSemantics, String> {
    expect_arg_count(node, expected, usage)?;
    Ok(InstructionSemantics::effect(node.op.args.clone()))
}

fn expect_arg_count(node: &Node, expected: usize, usage: &str) -> Result<(), String> {
    if node.op.args.len() == expected {
        Ok(())
    } else {
        Err(format!("node `{}` expects `{usage}`", node.name))
    }
}

fn parse_i64_arg(node: &Node, index: usize, label: &str) -> Result<i64, String> {
    node.op.args[index].parse::<i64>().map_err(|_| {
        format!(
            "node `{}` has invalid {label} `{}`",
            node.name, node.op.args[index]
        )
    })
}
