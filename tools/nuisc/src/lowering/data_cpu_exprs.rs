use super::*;

pub(super) fn lower_data_cpu_expr(
    expr: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Option<Result<String, String>> {
    match expr {
        NirExpr::DataProfileSendUplink { unit, input } => Some(lower_data_profile_send(
            state,
            bindings,
            unit,
            input,
            "data_immutable_window",
            "immutable_window",
            "uplink_len",
        )),
        NirExpr::DataProfileSendDownlink { unit, input } => Some(lower_data_profile_send(
            state,
            bindings,
            unit,
            input,
            "data_immutable_window",
            "immutable_window",
            "downlink_len",
        )),
        NirExpr::DataBindCore(core_index) => Some(Ok(lower_data_bind_core(*core_index, state))),
        NirExpr::DataMarker(tag) => Some(Ok(lower_data_marker(tag, state))),
        NirExpr::DataOutputPipe(value) => Some(lower_data_output_pipe(value, state, bindings)),
        NirExpr::DataInputPipe(pipe) => Some(lower_data_input_pipe(pipe, state, bindings)),
        NirExpr::DataResult { value, state: flow } => Some(lower_result_observe_node(
            state,
            bindings,
            ResultLoweringDomain::Data,
            value,
            "data_result",
            flow.render(),
        )),
        NirExpr::DataReady(result) => Some(lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Data,
            result,
            "data_ready",
            "is_ready",
        )),
        NirExpr::DataMoved(result) => Some(lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Data,
            result,
            "data_moved",
            "is_moved",
        )),
        NirExpr::DataWindowed(result) => Some(lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Data,
            result,
            "data_windowed",
            "is_windowed",
        )),
        NirExpr::DataValue(result) => Some(lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Data,
            result,
            "data_value",
            "value",
        )),
        NirExpr::DataCopyWindow { input, offset, len } => {
            Some(lower_data_copy_window(input, offset, len, state, bindings))
        }
        NirExpr::DataReadWindow { window, index } => {
            Some(lower_data_read_window(window, index, state, bindings))
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => Some(lower_data_write_window(
            window, index, value, state, bindings,
        )),
        NirExpr::DataFreezeWindow(input) => Some(lower_data_freeze_window(input, state, bindings)),
        NirExpr::DataImmutableWindow { input, offset, len } => Some(lower_data_immutable_window(
            input, offset, len, state, bindings,
        )),
        NirExpr::DataHandleTable(entries) => Some(Ok(lower_data_handle_table(entries, state))),
        NirExpr::DataProviderRequestIngress {
            request_handle,
            descriptor_table_handle,
            descriptor_count,
            provider_key,
            capability_hash,
            capsule_token,
            input_role_count,
            output_role_count,
        } => Some(lower_data_provider_request_ingress(
            request_handle,
            descriptor_table_handle,
            descriptor_count,
            provider_key,
            capability_hash,
            capsule_token.as_deref(),
            input_role_count.as_deref(),
            output_role_count.as_deref(),
            state,
            bindings,
        )),
        _ => None,
    }
}

fn lower_data_bind_core(core_index: i64, state: &mut LoweringState<'_>) -> String {
    ensure_fabric_resource(state.yir);
    let name = next_name(state, "data_bind_core");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "fabric0".to_owned(),
        op: Operation {
            module: "data".to_owned(),
            instruction: "bind_core".to_owned(),
            args: vec![core_index.to_string()],
        },
    });
    name
}

fn lower_data_marker(tag: &str, state: &mut LoweringState<'_>) -> String {
    ensure_fabric_resource(state.yir);
    let name = next_name(state, "data_marker");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "fabric0".to_owned(),
        op: Operation {
            module: "data".to_owned(),
            instruction: "marker".to_owned(),
            args: vec![tag.to_owned()],
        },
    });
    name
}

fn lower_data_output_pipe(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_fabric_resource(state.yir);
    let value_name = lower_expr(value, state, bindings)?;
    let name = next_name(state, "data_output_pipe");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "fabric0".to_owned(),
        op: Operation {
            module: "data".to_owned(),
            instruction: "output_pipe".to_owned(),
            args: vec![value_name.clone()],
        },
    });
    push_dep_edges(state, &value_name, &name);
    Ok(name)
}

fn lower_data_input_pipe(
    pipe: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_fabric_resource(state.yir);
    let pipe_name = lower_expr(pipe, state, bindings)?;
    let name = next_name(state, "data_input_pipe");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "fabric0".to_owned(),
        op: Operation {
            module: "data".to_owned(),
            instruction: "input_pipe".to_owned(),
            args: vec![pipe_name.clone()],
        },
    });
    push_dep_edges(state, &pipe_name, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: pipe_name,
        to: name.clone(),
    });
    Ok(name)
}

fn lower_data_copy_window(
    input: &NirExpr,
    offset: &NirExpr,
    len: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_fabric_resource(state.yir);
    let input_name = lower_expr(input, state, bindings)?;
    let offset_name = lower_expr(offset, state, bindings)?;
    let len_name = lower_expr(len, state, bindings)?;
    let name = next_name(state, "data_copy_window");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "fabric0".to_owned(),
        op: Operation {
            module: "data".to_owned(),
            instruction: "copy_window".to_owned(),
            args: vec![input_name.clone(), offset_name.clone(), len_name.clone()],
        },
    });
    push_dep_edges(state, &input_name, &name);
    push_dep_edges(state, &offset_name, &name);
    push_dep_edges(state, &len_name, &name);
    Ok(name)
}

fn lower_data_read_window(
    window: &NirExpr,
    index: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_fabric_resource(state.yir);
    let window_name = lower_expr(window, state, bindings)?;
    let index_name = lower_expr(index, state, bindings)?;
    let name = next_name(state, "data_read_window");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "fabric0".to_owned(),
        op: Operation {
            module: "data".to_owned(),
            instruction: "read_window".to_owned(),
            args: vec![window_name.clone(), index_name.clone()],
        },
    });
    push_dep_edges(state, &window_name, &name);
    push_dep_edges(state, &index_name, &name);
    Ok(name)
}

fn lower_data_write_window(
    window: &NirExpr,
    index: &NirExpr,
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_fabric_resource(state.yir);
    let window_name = lower_expr(window, state, bindings)?;
    let index_name = lower_expr(index, state, bindings)?;
    let value_name = lower_expr(value, state, bindings)?;
    let name = next_name(state, "data_write_window");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "fabric0".to_owned(),
        op: Operation {
            module: "data".to_owned(),
            instruction: "write_window".to_owned(),
            args: vec![window_name.clone(), index_name.clone(), value_name.clone()],
        },
    });
    push_dep_edges(state, &window_name, &name);
    push_dep_edges(state, &index_name, &name);
    push_dep_edges(state, &value_name, &name);
    Ok(name)
}

fn lower_data_freeze_window(
    input: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_fabric_resource(state.yir);
    let input_name = lower_expr(input, state, bindings)?;
    let name = next_name(state, "data_freeze_window");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "fabric0".to_owned(),
        op: Operation {
            module: "data".to_owned(),
            instruction: "freeze_window".to_owned(),
            args: vec![input_name.clone()],
        },
    });
    push_dep_edges(state, &input_name, &name);
    Ok(name)
}

fn lower_data_immutable_window(
    input: &NirExpr,
    offset: &NirExpr,
    len: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_fabric_resource(state.yir);
    let input_name = lower_expr(input, state, bindings)?;
    let offset_name = lower_expr(offset, state, bindings)?;
    let len_name = lower_expr(len, state, bindings)?;
    let name = next_name(state, "data_immutable_window");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "fabric0".to_owned(),
        op: Operation {
            module: "data".to_owned(),
            instruction: "immutable_window".to_owned(),
            args: vec![input_name.clone(), offset_name.clone(), len_name.clone()],
        },
    });
    push_dep_edges(state, &input_name, &name);
    push_dep_edges(state, &offset_name, &name);
    push_dep_edges(state, &len_name, &name);
    Ok(name)
}

fn lower_data_handle_table(entries: &[(String, String)], state: &mut LoweringState<'_>) -> String {
    ensure_fabric_resource(state.yir);
    let name = next_name(state, "data_handle_table");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "fabric0".to_owned(),
        op: Operation {
            module: "data".to_owned(),
            instruction: "handle_table".to_owned(),
            args: entries
                .iter()
                .map(|(slot, resource)| format!("{slot}={resource}"))
                .collect(),
        },
    });
    name
}

fn lower_data_provider_request_ingress(
    request_handle: &NirExpr,
    descriptor_table_handle: &NirExpr,
    descriptor_count: &NirExpr,
    provider_key: &NirExpr,
    capability_hash: &NirExpr,
    capsule_token: Option<&NirExpr>,
    input_role_count: Option<&NirExpr>,
    output_role_count: Option<&NirExpr>,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    ensure_fabric_resource(state.yir);
    let mut args = vec![
        lower_expr(request_handle, state, bindings)?,
        lower_expr(descriptor_table_handle, state, bindings)?,
        lower_expr(descriptor_count, state, bindings)?,
        lower_expr(provider_key, state, bindings)?,
        lower_expr(capability_hash, state, bindings)?,
    ];
    match (capsule_token, input_role_count, output_role_count) {
        (Some(token), Some(inputs), Some(outputs)) => {
            args.push(lower_expr(token, state, bindings)?);
            args.push(lower_expr(inputs, state, bindings)?);
            args.push(lower_expr(outputs, state, bindings)?);
        }
        (None, None, None) => {}
        _ => return Err("provider request capsule ingress metadata is incomplete".to_owned()),
    }
    let name = next_name(state, "provider_request_ingress");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "fabric0".to_owned(),
        op: Operation {
            module: "data".to_owned(),
            instruction: "provider_request_ingress".to_owned(),
            args: args.clone(),
        },
    });
    for arg in &args {
        push_dep_edges(state, arg, &name);
    }
    Ok(name)
}
