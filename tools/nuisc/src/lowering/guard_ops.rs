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
