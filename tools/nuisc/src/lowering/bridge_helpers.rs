use super::*;

pub(super) fn lower_project_profile_ref(
    state: &mut LoweringState<'_>,
    domain: &str,
    unit: &str,
    slot: &str,
) -> Result<String, String> {
    let name = next_name(state, "project_profile_ref");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "project_profile_ref".to_owned(),
            args: vec![domain.to_owned(), unit.to_owned(), slot.to_owned()],
        },
    });
    Ok(name)
}

pub(super) fn lower_data_profile_send(
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
    unit: &str,
    input: &NirExpr,
    window_prefix: &str,
    window_instruction: &str,
    len_slot: &str,
) -> Result<String, String> {
    ensure_fabric_resource(state.yir);

    let input_name = lower_expr(input, state, bindings)?;
    let offset_name = lower_project_profile_ref(state, "data", unit, "window_offset")?;
    let len_name = lower_project_profile_ref(state, "data", unit, len_slot)?;
    let handle_table_name = lower_project_profile_ref(state, "data", unit, "handle_table")?;

    let window_name = next_name(state, window_prefix);
    state.yir.nodes.push(Node {
        name: window_name.clone(),
        resource: "fabric0".to_owned(),
        op: Operation {
            module: "data".to_owned(),
            instruction: window_instruction.to_owned(),
            args: vec![input_name.clone(), offset_name.clone(), len_name.clone()],
        },
    });
    push_dep_edges(state, &input_name, &window_name);
    push_dep_edges(state, &offset_name, &window_name);
    push_dep_edges(state, &len_name, &window_name);
    push_dep_edges(state, &handle_table_name, &window_name);

    let output_name = next_name(state, "data_output_pipe");
    state.yir.nodes.push(Node {
        name: output_name.clone(),
        resource: "fabric0".to_owned(),
        op: Operation {
            module: "data".to_owned(),
            instruction: "output_pipe".to_owned(),
            args: vec![window_name.clone()],
        },
    });
    push_dep_edges(state, &window_name, &output_name);
    push_dep_edges(state, &handle_table_name, &output_name);

    let input_pipe_name = next_name(state, "data_input_pipe");
    state.yir.nodes.push(Node {
        name: input_pipe_name.clone(),
        resource: "fabric0".to_owned(),
        op: Operation {
            module: "data".to_owned(),
            instruction: "input_pipe".to_owned(),
            args: vec![output_name.clone()],
        },
    });
    push_dep_edges(state, &output_name, &input_pipe_name);
    push_dep_edges(state, &handle_table_name, &input_pipe_name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: output_name,
        to: input_pipe_name.clone(),
    });

    Ok(input_pipe_name)
}
