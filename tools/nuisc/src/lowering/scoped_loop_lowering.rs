use super::*;

pub(super) fn collect_scoped_loop_helper_functions(module: &NirModule) -> BTreeSet<String> {
    let eligible = module
        .functions
        .iter()
        .filter(|function| direct_calls::supports_direct_call_signature(function))
        .map(|function| function.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut helpers = BTreeSet::new();
    for function in &module.functions {
        collect_from_stmts(&function.body, &eligible, &mut helpers);
    }
    helpers
}

fn collect_from_stmts(
    stmts: &[NirStmt],
    eligible: &BTreeSet<&str>,
    helpers: &mut BTreeSet<String>,
) {
    for stmt in stmts {
        match stmt {
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_from_stmts(then_body, eligible, helpers);
                collect_from_stmts(else_body, eligible, helpers);
            }
            NirStmt::While { body, .. } => {
                if let Some(NirStmt::Expr(NirExpr::Call { callee, .. })) = body.first() {
                    if eligible.contains(callee.as_str()) {
                        helpers.insert(callee.clone());
                    }
                }
                collect_from_stmts(body, eligible, helpers);
            }
            _ => {}
        }
    }
}

pub(super) fn lower_scoped_call_while(
    condition: &NirExpr,
    body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<bool, String> {
    let Some((NirStmt::Expr(NirExpr::Call { callee, args }), counted_body)) = body.split_first()
    else {
        return Ok(false);
    };
    if !state.direct_call_functions.contains(callee) {
        return Ok(false);
    }
    let Some(prepared) = prepare_counted_while(
        condition,
        counted_body,
        &state.pure_helpers,
        &state.inlineable_pure_helpers,
        &state.pure_helper_blocks,
    ) else {
        return Ok(false);
    };
    let function = state
        .function_map
        .get(callee.as_str())
        .copied()
        .ok_or_else(|| format!("unknown scoped loop helper `{callee}`"))?;
    if function.params.len() != args.len() {
        return Err(format!(
            "scoped loop helper `{callee}` expects {} args, found {}",
            function.params.len(),
            args.len()
        ));
    }

    let Some(initial) = bindings.get(&prepared.binding_name).cloned() else {
        return Err(format!(
            "scoped-call loop expected an existing binding for `{}`",
            prepared.binding_name
        ));
    };
    let limit = lower_expr(&prepared.limit, state, bindings)?;
    let step = lower_expr(&prepared.step, state, bindings)?;
    let mut action_args = vec![callee.clone()];
    for arg in args {
        if matches!(arg, NirExpr::Var(name) if name == &prepared.binding_name) {
            action_args.push("$current".to_owned());
        } else {
            action_args.push(lower_expr(arg, state, bindings)?);
        }
    }
    let step_kind = match prepared.step_kind {
        PreparedLoopStepKind::Add => "add",
        PreparedLoopStepKind::Sub => "sub",
    };
    let mut node_args = vec![
        initial,
        limit,
        step,
        render_loop_compare(prepared.compare).to_owned(),
        step_kind.to_owned(),
        "cpu".to_owned(),
        "scoped_call".to_owned(),
        action_args.len().to_string(),
    ];
    node_args.extend(action_args);
    let name = next_name(state, "loop_while_i64_scoped_call");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "loop_while_i64_effect".to_owned(),
            args: node_args.clone(),
        },
    });
    for input in node_args {
        if state.yir.nodes.iter().any(|node| node.name == input) {
            push_dep_edges(state, &input, &name);
        }
    }
    for (param, arg) in function.params.iter().zip(args) {
        if param.ty.is_ref && param.ty.name == "Buffer" {
            let input = bindings.get(match arg {
                NirExpr::Var(name) => name,
                _ => continue,
            });
            if let Some(input) = input {
                push_lifetime_edge(state, input, &name);
            }
        }
    }
    body_lowering::chain_statement_effect(state, &name);
    bindings.insert(prepared.binding_name, name);
    Ok(true)
}
