use super::*;

pub(in crate::lowering) fn collect_guarded_loop_direct_call_functions(
    module: &NirModule,
) -> BTreeSet<String> {
    let pure_helpers = collect_pure_helper_functions(module);
    let function_names = module
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect::<BTreeSet<_>>();
    let function_map = module
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect::<BTreeMap<_, _>>();
    let mut reachable = BTreeSet::from(["main".to_owned()]);
    let mut frontier = vec!["main".to_owned()];
    while let Some(name) = frontier.pop() {
        let Some(function) = function_map.get(name.as_str()) else {
            continue;
        };
        for called in function_called_functions(function, &function.body, &function_names) {
            if reachable.insert(called.clone()) {
                frontier.push(called);
            }
        }
    }

    module
        .functions
        .iter()
        .filter(|function| function.name != "main")
        .filter(|function| !function.is_async)
        .filter(|function| reachable.contains(&function.name))
        .filter(|function| supports_direct_call_signature(function))
        // A guarded borrowed-buffer helper must return to its caller, not from it.
        .filter(|function| {
            stmts_contain_guarded_loop_boundary(&function.body, &pure_helpers)
                || (function.params.iter().any(|param| param.ty.is_ref && param.ty.name == "Buffer")
                    && !pure_helpers.contains(&function.name)
                    && function.body.iter().any(|stmt| matches!(stmt,
                        NirStmt::If { then_body, else_body, .. }
                            if else_body.is_empty() && matches!(then_body.as_slice(), [NirStmt::Return(Some(_))])
                    )))
        })
        .map(|function| function.name.clone())
        .collect()
}

/// A guard is a control boundary even for pure operations that can trap.
pub(super) fn order_guarded_function_nodes(state: &mut LoweringState<'_>, start: usize) {
    let mut guard: Option<String> = None;
    let mut edges = Vec::new();
    for node in &state.yir.nodes[start..] {
        if let Some(previous) = &guard {
            edges.push((previous.clone(), node.name.clone()));
        }
        if node.op.module == "cpu" && node.op.instruction == "guard_return" {
            guard = Some(node.name.clone());
        }
    }
    for (from, to) in edges {
        crate::lowering::edge_helpers::push_effect_edge(state, &from, &to);
    }
}

fn stmts_contain_guarded_loop_boundary(stmts: &[NirStmt], pure_helpers: &BTreeSet<String>) -> bool {
    stmts.iter().any(|stmt| match stmt {
        NirStmt::While { body, .. } => {
            prepare_guarded_loop_body(body, pure_helpers).is_some()
                || stmts_contain_guarded_loop_boundary(body, pure_helpers)
        }
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            stmts_contain_guarded_loop_boundary(then_body, pure_helpers)
                || stmts_contain_guarded_loop_boundary(else_body, pure_helpers)
        }
        _ => false,
    })
}
