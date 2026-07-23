use super::*;

pub(super) fn lower_guard_return(
    condition_name: String,
    return_name: String,
    state: &mut LoweringState<'_>,
) {
    let name = next_name(state, "guard_return");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "guard_return".to_owned(),
            args: vec![condition_name.clone(), return_name.clone()],
        },
    });
    push_dep_edges(state, &condition_name, &name);
    push_dep_edges(state, &return_name, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: condition_name,
        to: name.clone(),
    });
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: return_name,
        to: name,
    });
}

pub(super) fn lower_guard_drop_owned_bytes_return(
    condition_name: String,
    bytes_name: String,
    return_name: String,
    state: &mut LoweringState<'_>,
) {
    let name = next_name(state, "guard_drop_owned_bytes_return");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "guard_drop_owned_bytes_return".to_owned(),
            args: vec![
                condition_name.clone(),
                bytes_name.clone(),
                return_name.clone(),
            ],
        },
    });
    for input in [condition_name, bytes_name, return_name] {
        push_dep_edges(state, &input, &name);
        state.yir.edges.push(Edge {
            kind: EdgeKind::Effect,
            from: input,
            to: name.clone(),
        });
    }
}

pub(super) fn lower_branch_drop_owned_bytes_return(
    condition_name: String,
    then_bytes_name: String,
    then_return_name: String,
    else_bytes_name: String,
    else_return_name: String,
    state: &mut LoweringState<'_>,
) {
    let name = next_name(state, "branch_drop_owned_bytes_return");
    let inputs = vec![
        condition_name,
        then_bytes_name,
        then_return_name,
        else_bytes_name,
        else_return_name,
    ];
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "branch_drop_owned_bytes_return".to_owned(),
            args: inputs.clone(),
        },
    });
    for input in inputs {
        push_dep_edges(state, &input, &name);
        state.yir.edges.push(Edge {
            kind: EdgeKind::Effect,
            from: input,
            to: name.clone(),
        });
    }
}

pub(super) fn lower_guard_print(
    condition_name: String,
    print_name: String,
    state: &mut LoweringState<'_>,
) {
    let name = next_name(state, "guard_print");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "guard_print".to_owned(),
            args: vec![condition_name.clone(), print_name.clone()],
        },
    });
    push_dep_edges(state, &condition_name, &name);
    push_dep_edges(state, &print_name, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: condition_name,
        to: name.clone(),
    });
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: print_name,
        to: name,
    });
}

pub(super) fn lower_guard_print_return(
    condition_name: String,
    print_name: String,
    return_name: String,
    state: &mut LoweringState<'_>,
) {
    let name = next_name(state, "guard_print_return");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "guard_print_return".to_owned(),
            args: vec![
                condition_name.clone(),
                print_name.clone(),
                return_name.clone(),
            ],
        },
    });
    push_dep_edges(state, &condition_name, &name);
    push_dep_edges(state, &print_name, &name);
    push_dep_edges(state, &return_name, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: condition_name,
        to: name.clone(),
    });
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: print_name,
        to: name.clone(),
    });
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: return_name,
        to: name,
    });
}

pub(super) fn lower_guard_host_call_return(
    condition_name: String,
    calls: Vec<(Option<String>, String, String, Vec<String>)>,
    returned: PreparedHostCallReturnSpec,
    state: &mut LoweringState<'_>,
) {
    let name = next_name(state, "guard_host_call_return");
    let mut args = vec![condition_name.clone()];
    encode_host_call_return_args(&mut args, &returned, &calls);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "guard_host_call_return".to_owned(),
            args,
        },
    });
    push_dep_edges(state, &condition_name, &name);
    push_host_call_return_deps(state, &name, &returned, &calls);
    let mut effects = vec![condition_name];
    effects.extend(
        calls
            .iter()
            .flat_map(|(_, _, _, call_args)| call_args)
            .cloned(),
    );
    for effect in effects {
        state.yir.edges.push(Edge {
            kind: EdgeKind::Effect,
            from: effect,
            to: name.clone(),
        });
    }
}

pub(super) fn lower_branch_host_call_return(
    condition_name: String,
    then_calls: Vec<(Option<String>, String, String, Vec<String>)>,
    then_returned: PreparedHostCallReturnSpec,
    else_calls: Vec<(Option<String>, String, String, Vec<String>)>,
    else_returned: PreparedHostCallReturnSpec,
    state: &mut LoweringState<'_>,
) {
    let name = next_name(state, "branch_host_call_return");
    let mut args = vec![condition_name.clone()];
    encode_host_call_return_args(&mut args, &then_returned, &then_calls);
    encode_host_call_return_args(&mut args, &else_returned, &else_calls);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "branch_host_call_return".to_owned(),
            args,
        },
    });
    push_dep_edges(state, &condition_name, &name);
    push_host_call_return_deps(state, &name, &then_returned, &then_calls);
    push_host_call_return_deps(state, &name, &else_returned, &else_calls);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: condition_name,
        to: name.clone(),
    });
}

pub(super) enum PreparedHostCallReturnSpec {
    Value(String),
    CompareCallResult {
        result_name: String,
        op: NirBinaryOp,
        expected: String,
        matched: String,
        unmatched: String,
    },
    WriteFlushExitCode {
        write_name: String,
        flush_name: String,
        offset: i64,
    },
}

fn encode_host_call_return_args(
    args: &mut Vec<String>,
    returned: &PreparedHostCallReturnSpec,
    calls: &[(Option<String>, String, String, Vec<String>)],
) {
    match returned {
        PreparedHostCallReturnSpec::Value(return_name) => {
            args.extend(["value".to_owned(), "1".to_owned(), return_name.clone()]);
        }
        PreparedHostCallReturnSpec::CompareCallResult {
            result_name,
            op,
            expected,
            matched,
            unmatched,
        } => args.extend([
            "compare_call_result".to_owned(),
            "5".to_owned(),
            result_name.clone(),
            render_branch_comparison(*op).to_owned(),
            expected.clone(),
            matched.clone(),
            unmatched.clone(),
        ]),
        PreparedHostCallReturnSpec::WriteFlushExitCode {
            write_name,
            flush_name,
            offset,
        } => args.extend([
            "write_flush_exit_code".to_owned(),
            "3".to_owned(),
            write_name.clone(),
            flush_name.clone(),
            offset.to_string(),
        ]),
    }
    args.push(calls.len().to_string());
    for (alias, abi, callee, call_args) in calls {
        args.push(alias.clone().unwrap_or_else(|| "_".to_owned()));
        args.push(abi.clone());
        args.push(callee.clone());
        args.push(call_args.len().to_string());
        args.extend(call_args.iter().cloned());
    }
}

fn push_host_call_return_deps(
    state: &mut LoweringState<'_>,
    name: &str,
    returned: &PreparedHostCallReturnSpec,
    calls: &[(Option<String>, String, String, Vec<String>)],
) {
    if let PreparedHostCallReturnSpec::Value(return_name) = returned {
        push_dep_edges(state, return_name, name);
    }
    if let PreparedHostCallReturnSpec::CompareCallResult {
        expected,
        matched,
        unmatched,
        ..
    } = returned
    {
        for value in [expected, matched, unmatched] {
            push_dep_edges(state, value, name);
        }
    }
    for arg_name in calls.iter().flat_map(|(_, _, _, call_args)| call_args) {
        push_dep_edges(state, arg_name, name);
    }
}

fn render_branch_comparison(op: NirBinaryOp) -> &'static str {
    match op {
        NirBinaryOp::Eq => "eq",
        NirBinaryOp::Ne => "ne",
        NirBinaryOp::Lt => "lt",
        NirBinaryOp::Le => "le",
        NirBinaryOp::Gt => "gt",
        NirBinaryOp::Ge => "ge",
        _ => unreachable!("prepared host-call comparison must use a comparison operator"),
    }
}

pub(super) fn lower_branch_print_return(
    condition_name: String,
    then_print_name: String,
    then_return_name: String,
    else_print_name: String,
    else_return_name: String,
    state: &mut LoweringState<'_>,
) {
    let name = next_name(state, "branch_print_return");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "branch_print_return".to_owned(),
            args: vec![
                condition_name.clone(),
                then_print_name.clone(),
                then_return_name.clone(),
                else_print_name.clone(),
                else_return_name.clone(),
            ],
        },
    });
    for dep in [
        &condition_name,
        &then_print_name,
        &then_return_name,
        &else_print_name,
        &else_return_name,
    ] {
        push_dep_edges(state, dep, &name);
    }
    for effect in [
        &condition_name,
        &then_print_name,
        &then_return_name,
        &else_print_name,
        &else_return_name,
    ] {
        state.yir.edges.push(Edge {
            kind: EdgeKind::Effect,
            from: effect.clone(),
            to: name.clone(),
        });
    }
}

pub(super) fn lower_select(
    condition_name: String,
    then_name: String,
    else_name: String,
    state: &mut LoweringState<'_>,
) -> Result<String, String> {
    let name = next_name(state, "select");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "select".to_owned(),
            args: vec![condition_name.clone(), then_name.clone(), else_name.clone()],
        },
    });
    push_dep_edges(state, &condition_name, &name);
    push_dep_edges(state, &then_name, &name);
    push_dep_edges(state, &else_name, &name);
    Ok(name)
}

pub(super) fn lower_select_owned_bytes(
    condition_name: String,
    then_name: String,
    else_name: String,
    state: &mut LoweringState<'_>,
) -> String {
    let name = next_name(state, "select_owned_bytes");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "select_owned_bytes".to_owned(),
            args: vec![condition_name.clone(), then_name.clone(), else_name.clone()],
        },
    });
    push_dep_edges(state, &condition_name, &name);
    push_dep_edges(state, &then_name, &name);
    push_dep_edges(state, &else_name, &name);
    push_lifetime_edge(state, &then_name, &name);
    if then_name != else_name {
        push_lifetime_edge(state, &else_name, &name);
    }
    name
}

pub(super) fn lower_select_owned_bytes_drop_unselected(
    condition_name: String,
    then_name: String,
    else_name: String,
    state: &mut LoweringState<'_>,
) -> String {
    let name = next_name(state, "select_owned_bytes_drop_unselected");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "select_owned_bytes_drop_unselected".to_owned(),
            args: vec![condition_name.clone(), then_name.clone(), else_name.clone()],
        },
    });
    push_dep_edges(state, &condition_name, &name);
    push_dep_edges(state, &then_name, &name);
    push_dep_edges(state, &else_name, &name);
    push_lifetime_edge(state, &then_name, &name);
    push_lifetime_edge(state, &else_name, &name);
    name
}

pub(super) fn lower_select_owned_bytes_tree(
    owners: Vec<String>,
    tree_tokens: Vec<String>,
    conditions: &[String],
    state: &mut LoweringState<'_>,
) -> String {
    let name = next_name(state, "select_owned_bytes_tree");
    let mut args = vec![owners.len().to_string()];
    args.extend(owners.iter().cloned());
    args.extend(tree_tokens);
    let mut scalar_dependencies = Vec::new();
    let parsed = yir_core::parse_owned_select_tree_args(&args)
        .expect("owned select tree builder must encode a valid protocol");
    yir_core::owned_select_tree_scalar_args(&parsed.tree, &mut scalar_dependencies);
    let mut pointer_transfers = Vec::new();
    yir_core::owned_select_tree_transfers(&parsed.tree, &mut pointer_transfers);
    let pointer_transfers = pointer_transfers
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let scalar_dependencies = scalar_dependencies
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "select_owned_bytes_tree".to_owned(),
            args,
        },
    });
    for condition in conditions {
        push_dep_edges(state, condition, &name);
    }
    for scalar in &scalar_dependencies {
        push_dep_edges(state, scalar, &name);
    }
    for transfer in &pointer_transfers {
        push_lifetime_edge(state, transfer, &name);
    }
    for owner in &owners {
        push_dep_edges(state, owner, &name);
        push_lifetime_edge(state, owner, &name);
    }
    name
}

pub(super) fn lower_branch_call_owned_bytes(
    condition_name: String,
    then_callee: String,
    else_callee: String,
    owner_name: String,
    then_scalar_args: Vec<String>,
    else_scalar_args: Vec<String>,
    state: &mut LoweringState<'_>,
) -> String {
    let name = next_name(state, "branch_call_owned_bytes");
    let mut args = vec![
        condition_name.clone(),
        then_callee,
        else_callee,
        owner_name.clone(),
        then_scalar_args.len().to_string(),
    ];
    args.extend(then_scalar_args.iter().cloned());
    args.push(else_scalar_args.len().to_string());
    args.extend(else_scalar_args.iter().cloned());
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "branch_call_owned_bytes".to_owned(),
            args,
        },
    });
    push_dep_edges(state, &condition_name, &name);
    push_dep_edges(state, &owner_name, &name);
    for arg in then_scalar_args.iter().chain(&else_scalar_args) {
        push_dep_edges(state, arg, &name);
    }
    push_lifetime_edge(state, &owner_name, &name);
    name
}
