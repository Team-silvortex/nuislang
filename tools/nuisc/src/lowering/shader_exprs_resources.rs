use super::*;

pub(in crate::lowering) fn lower_shader_target(
    format: &str,
    width: i64,
    height: i64,
    state: &mut LoweringState<'_>,
) -> String {
    ensure_shader_resource(state.yir);
    let name = next_name(state, "shader_target");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "target".to_owned(),
            args: vec![format.to_owned(), width.to_string(), height.to_string()],
        },
    });
    name
}

pub(in crate::lowering) fn lower_shader_viewport(
    width: i64,
    height: i64,
    state: &mut LoweringState<'_>,
) -> String {
    ensure_shader_resource(state.yir);
    let name = next_name(state, "shader_viewport");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "viewport".to_owned(),
            args: vec![width.to_string(), height.to_string()],
        },
    });
    name
}

pub(in crate::lowering) fn lower_shader_pipeline(
    pipe_name: &str,
    topology: &str,
    state: &mut LoweringState<'_>,
) -> String {
    ensure_shader_resource(state.yir);
    let name = next_name(state, "shader_pipeline");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "pipeline".to_owned(),
            args: vec![pipe_name.to_owned(), topology.to_owned()],
        },
    });
    name
}

pub(in crate::lowering) fn lower_shader_texture2d(
    format: &str,
    width: i64,
    height: i64,
    texels: &str,
    state: &mut LoweringState<'_>,
) -> String {
    ensure_shader_resource(state.yir);
    let name = next_name(state, "shader_texture2d");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "texture2d".to_owned(),
            args: vec![
                format.to_owned(),
                width.to_string(),
                height.to_string(),
                texels.to_owned(),
            ],
        },
    });
    name
}

pub(in crate::lowering) fn lower_shader_sampler(
    filter: &str,
    address_mode: &str,
    state: &mut LoweringState<'_>,
) -> String {
    ensure_shader_resource(state.yir);
    let name = next_name(state, "shader_sampler");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "sampler".to_owned(),
            args: vec![filter.to_owned(), address_mode.to_owned()],
        },
    });
    name
}

pub(in crate::lowering) fn lower_shader_uv(
    u: i64,
    v: i64,
    state: &mut LoweringState<'_>,
) -> String {
    ensure_shader_resource(state.yir);
    let name = next_name(state, "shader_uv");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "uv".to_owned(),
            args: vec![u.to_string(), v.to_string()],
        },
    });
    name
}

pub(in crate::lowering) fn lower_shader_inline_wgsl(
    entry: &str,
    source: &str,
    state: &mut LoweringState<'_>,
) -> Result<String, String> {
    ensure_shader_resource(state.yir);
    let name = next_name(state, "shader_inline_wgsl");
    let normalized = crate::shader_source::normalize_inline_wgsl_source(source)?;
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "shader0".to_owned(),
        op: Operation {
            module: "shader".to_owned(),
            instruction: "inline_wgsl".to_owned(),
            args: vec![entry.to_owned(), normalized],
        },
    });
    Ok(name)
}
