use super::*;
use nuis_semantics::model::{NirShaderSampleMode, NirShaderSampleUvMode};

pub(in crate::lowering) fn lower_shader_begin_pass(
    target: &NirExpr,
    pipeline: &NirExpr,
    viewport: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_shader_resource(state.yir);
    let target_name = lower_expr(target, state, bindings)?;
    let pipeline_name = lower_expr(pipeline, state, bindings)?;
    let viewport_name = lower_expr(viewport, state, bindings)?;
    let name = next_name(state, "shader_begin_pass");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "begin_pass".to_owned(),
            args: vec![
                target_name.clone(),
                pipeline_name.clone(),
                viewport_name.clone(),
            ],
        },
    });
    push_dep_edges(state, &target_name, &name);
    push_dep_edges(state, &pipeline_name, &name);
    push_dep_edges(state, &viewport_name, &name);
    Ok(name)
}

pub(in crate::lowering) fn lower_shader_sample(
    texture: &NirExpr,
    sampler: &NirExpr,
    x: &NirExpr,
    y: &NirExpr,
    mode: NirShaderSampleMode,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_shader_resource(state.yir);
    let texture_name = lower_expr(texture, state, bindings)?;
    let sampler_name = lower_expr(sampler, state, bindings)?;
    let x_name = lower_expr(x, state, bindings)?;
    let y_name = lower_expr(y, state, bindings)?;
    let name = next_name(state, "shader_sample");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: mode.render().to_owned(),
            args: vec![
                texture_name.clone(),
                sampler_name.clone(),
                x_name.clone(),
                y_name.clone(),
            ],
        },
    });
    push_dep_edges(state, &texture_name, &name);
    push_dep_edges(state, &sampler_name, &name);
    push_dep_edges(state, &x_name, &name);
    push_dep_edges(state, &y_name, &name);
    Ok(name)
}

pub(in crate::lowering) fn lower_shader_sample_uv(
    texture: &NirExpr,
    sampler: &NirExpr,
    uv: &NirExpr,
    mode: NirShaderSampleUvMode,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_shader_resource(state.yir);
    let texture_name = lower_expr(texture, state, bindings)?;
    let sampler_name = lower_expr(sampler, state, bindings)?;
    let uv_name = lower_expr(uv, state, bindings)?;
    let name = next_name(state, "shader_sample_uv");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: mode.render().to_owned(),
            args: vec![texture_name.clone(), sampler_name.clone(), uv_name.clone()],
        },
    });
    push_dep_edges(state, &texture_name, &name);
    push_dep_edges(state, &sampler_name, &name);
    push_dep_edges(state, &uv_name, &name);
    Ok(name)
}

pub(in crate::lowering) fn lower_shader_binding(
    kind: &str,
    slot: i64,
    layout: Option<&str>,
    profile_contract: Option<&str>,
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_shader_resource(state.yir);
    let value_name = lower_expr(value, state, bindings)?;
    let name = next_name(state, "shader_binding");
    let mut args = vec![slot.to_string()];
    if let Some(layout) = layout {
        args.push(layout.to_owned());
    }
    if let Some(profile_contract) = profile_contract {
        args.push(profile_contract.to_owned());
    }
    args.push(value_name.clone());
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: kind.to_owned(),
            args,
        },
    });
    push_dep_edges(state, &value_name, &name);
    Ok(name)
}

pub(in crate::lowering) fn lower_shader_bind_set(
    pipeline: &NirExpr,
    set_bindings: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_shader_resource(state.yir);
    let pipeline_name = lower_expr(pipeline, state, bindings)?;
    let mut binding_names = Vec::with_capacity(set_bindings.len());
    for binding in set_bindings {
        let binding_name = lower_expr(binding, state, bindings)?;
        binding_names.push(binding_name);
    }
    let name = next_name(state, "shader_bind_set");
    let mut args = vec![pipeline_name.clone()];
    args.extend(binding_names.iter().cloned());
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "bind_set".to_owned(),
            args,
        },
    });
    push_dep_edges(state, &pipeline_name, &name);
    for binding_name in &binding_names {
        push_dep_edges(state, binding_name, &name);
    }
    Ok(name)
}

pub(in crate::lowering) fn lower_shader_draw_instanced(
    pass: &NirExpr,
    packet: &NirExpr,
    vertex_count: &NirExpr,
    instance_count: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_shader_resource(state.yir);
    let pass_name = lower_expr(pass, state, bindings)?;
    let packet_name = lower_expr(packet, state, bindings)?;
    let vertex_count_name = lower_expr(vertex_count, state, bindings)?;
    let instance_count_name = lower_expr(instance_count, state, bindings)?;
    let name = next_name(state, "shader_draw_instanced");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "draw_instanced".to_owned(),
            args: vec![
                pass_name.clone(),
                packet_name.clone(),
                vertex_count_name.clone(),
                instance_count_name.clone(),
            ],
        },
    });
    push_dep_edges(state, &pass_name, &name);
    push_xfer_edge(state, &packet_name, &name);
    push_xfer_edge(state, &vertex_count_name, &name);
    push_xfer_edge(state, &instance_count_name, &name);
    Ok(name)
}
