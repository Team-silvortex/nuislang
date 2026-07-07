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
    if let PreparedHostCallReturnSpec::Value(return_name) = &returned {
        push_dep_edges(state, return_name, &name);
    }
    for arg_name in calls.iter().flat_map(|(_, _, _, call_args)| call_args) {
        push_dep_edges(state, arg_name, &name);
    }
    let mut effects = vec![condition_name];
    if let PreparedHostCallReturnSpec::Value(return_name) = returned {
        effects.push(return_name);
    }
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
    for arg_name in calls.iter().flat_map(|(_, _, _, call_args)| call_args) {
        push_dep_edges(state, arg_name, name);
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
