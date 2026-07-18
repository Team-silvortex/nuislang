use super::*;

pub(super) fn lower_owned_bytes_while(
    condition: &NirExpr,
    body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<bool, String> {
    let Some((owned_name, source, tail)) = split_owned_copy(body) else {
        return Ok(false);
    };
    let Some(tail) = strip_owned_drops(tail, owned_name) else {
        return Ok(false);
    };

    if tail.as_slice() == [NirStmt::Break] {
        return lower_copy_drop_break(condition, source, state, bindings).map(|()| true);
    }

    let counted_body = match tail.as_slice() {
        [step] | [step, NirStmt::Continue] => Some(std::slice::from_ref(step)),
        _ => None,
    };
    if let Some(prepared) = counted_body.and_then(|counted_body| {
        prepare_counted_while(
            condition,
            counted_body,
            &state.pure_helpers,
            &state.inlineable_pure_helpers,
            &state.pure_helper_blocks,
        )
    }) {
        lower_counted_copy_drop(prepared, source, state, bindings)?;
        return Ok(true);
    }

    let Some(prepared) = prepare_flow_while(
        condition,
        &tail,
        &state.pure_helpers,
        &state.inlineable_pure_helpers,
        &state.pure_helper_blocks,
    ) else {
        return Ok(false);
    };
    lower_flow_copy_drop(prepared, source, state, bindings)?;
    Ok(true)
}

fn split_owned_copy(body: &[NirStmt]) -> Option<(&str, &NirExpr, &[NirStmt])> {
    let (first, tail) = body.split_first()?;
    let NirStmt::Let {
        name,
        value: NirExpr::CopyBufferOwned(source),
        ..
    } = first
    else {
        return None;
    };
    Some((name, source, tail))
}

fn strip_owned_drops(tail: &[NirStmt], owned_name: &str) -> Option<Vec<NirStmt>> {
    fn rewrite(stmts: &[NirStmt], owned_name: &str, removed: &mut bool) -> Vec<NirStmt> {
        stmts
            .iter()
            .filter_map(|stmt| match stmt {
                NirStmt::Expr(NirExpr::DropBytes(dropped))
                    if matches!(dropped.as_ref(), NirExpr::Var(name) if name == owned_name) =>
                {
                    *removed = true;
                    None
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => Some(NirStmt::If {
                    condition: condition.clone(),
                    then_body: rewrite(then_body, owned_name, removed),
                    else_body: rewrite(else_body, owned_name, removed),
                }),
                other => Some(other.clone()),
            })
            .collect()
    }

    let mut removed = false;
    let rewritten = rewrite(tail, owned_name, &mut removed);
    removed.then_some(rewritten)
}

fn lower_copy_drop_break(
    condition: &NirExpr,
    source: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<(), String> {
    let condition_name = lower_expr(condition, state, bindings)?;
    let source_name = lower_expr(source, state, bindings)?;
    push_owned_loop_node(
        state,
        "loop_owned_bytes_copy_drop_break",
        vec![condition_name, source_name],
    );
    Ok(())
}

fn lower_counted_copy_drop(
    prepared: PreparedCountedWhile,
    source: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let Some(initial_name) = bindings.get(&prepared.binding_name).cloned() else {
        return Err(format!(
            "resource-aware counted `while` expected an existing binding for `{}`",
            prepared.binding_name
        ));
    };
    let limit_name = lower_expr(&prepared.limit, state, bindings)?;
    let step_name = lower_expr(&prepared.step, state, bindings)?;
    let source_name = lower_expr(source, state, bindings)?;
    let compare = render_loop_compare(prepared.compare).to_owned();
    let step_kind = match prepared.step_kind {
        PreparedLoopStepKind::Add => "add",
        PreparedLoopStepKind::Sub => "sub",
    };
    let name = push_owned_loop_node(
        state,
        "loop_while_i64_effect",
        vec![
            initial_name,
            limit_name,
            step_name,
            compare,
            step_kind.to_owned(),
            "cpu".to_owned(),
            "owned_bytes_copy_drop".to_owned(),
            "1".to_owned(),
            source_name,
        ],
    );
    bindings.insert(prepared.binding_name, name);
    Ok(())
}

fn lower_flow_copy_drop(
    prepared: PreparedFlowWhile,
    source: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let (control_args, _, _, _) =
        encode_loop_flow_control_args(&prepared.control, state, bindings)?;
    let Some(initial_name) = bindings.get(&prepared.binding_name).cloned() else {
        return Err(format!(
            "resource-aware flow `while` expected an existing binding for `{}`",
            prepared.binding_name
        ));
    };
    let limit_name = lower_expr(&prepared.limit, state, bindings)?;
    let step_name = lower_expr(&prepared.step, state, bindings)?;
    let source_name = lower_expr(source, state, bindings)?;
    let mut carry_names = Vec::with_capacity(prepared.carries.len());
    let mut carry_args = Vec::new();
    for (carry_index, carry) in prepared.carries.iter().enumerate() {
        let Some(initial) = bindings.get(&carry.binding_name).cloned() else {
            return Err(format!(
                "resource-aware flow `while` expected carry binding `{}`",
                carry.binding_name
            ));
        };
        let PreparedCarryUpdateKind::Linear { op, source } = &carry.kind else {
            return Err("resource-aware flow loops require linear scalar carries".to_owned());
        };
        let (encoded, _, _) = encode_loop_carry_source_args(*op, source, state, bindings)?;
        let Some(kind) = encoded.first() else {
            return Err(format!(
                "resource-aware flow loop carry `{}` is missing its update kind",
                carry.binding_name
            ));
        };
        let supported = (kind == "add_current" && encoded.len() == 1)
            || (matches!(kind.as_str(), "add_invariant" | "mul_invariant") && encoded.len() == 2)
            || (matches!(
                kind.as_str(),
                "add_current_plus_invariant" | "mul_current_plus_invariant"
            ) && encoded.len() == 2)
            || ((kind.starts_with("mul_scaled_") || kind.starts_with("add_scaled_"))
                && effect_flow_kind_has_only_available_new_carries(kind, carry_index))
            || effect_flow_state_list_kind_is_supported(kind, encoded.len() - 1, carry_index)
            || (encoded.len() == 1
                && kind
                    .strip_prefix("add_carry")
                    .and_then(|index| index.parse::<usize>().ok())
                    .is_some_and(|index| index < carry_index));
        if !supported {
            return Err(format!(
                "resource-aware flow loop carry `{}` has forward or unsupported source `{kind}`",
                carry.binding_name
            ));
        }
        carry_args.push(initial);
        carry_args.extend(encoded);
        carry_names.push(carry.binding_name.clone());
    }
    let step_kind = match prepared.step_kind {
        PreparedLoopStepKind::Add => "add",
        PreparedLoopStepKind::Sub => "sub",
    };
    let mut args = vec![
        initial_name,
        limit_name,
        step_name,
        render_loop_compare(prepared.compare).to_owned(),
        step_kind.to_owned(),
        control_args.len().to_string(),
    ];
    args.extend(control_args);
    args.push(carry_names.len().to_string());
    args.extend(carry_args);
    args.extend([
        "cpu".to_owned(),
        "owned_bytes_copy_drop".to_owned(),
        "1".to_owned(),
        source_name,
    ]);
    let name = push_owned_loop_node(state, "loop_while_i64_effect_flow", args);
    let current = push_loop_field(state, &name, "current");
    bindings.insert(prepared.binding_name, current);
    for (index, carry_name) in carry_names.into_iter().enumerate() {
        let value = push_loop_field(state, &name, &format!("carry{index}"));
        bindings.insert(carry_name, value);
    }
    Ok(())
}

fn effect_flow_kind_has_only_available_new_carries(kind: &str, carry_index: usize) -> bool {
    kind.match_indices("carry").all(|(offset, _)| {
        if kind[..offset].ends_with("prev_") {
            return true;
        }
        let digits = kind[offset + 5..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>();
        !digits.is_empty()
            && digits
                .parse::<usize>()
                .is_ok_and(|source| source < carry_index)
    })
}

fn effect_flow_state_list_kind_is_supported(
    kind: &str,
    payload_len: usize,
    carry_index: usize,
) -> bool {
    let Some(terms) = kind
        .strip_prefix("add_")
        .or_else(|| kind.strip_prefix("mul_"))
    else {
        return false;
    };
    let (terms, expected_payload_len) = terms
        .strip_suffix("_plus_invariant")
        .map_or((terms, 0), |terms| (terms, 1));
    let terms = terms.split("_plus_").collect::<Vec<_>>();
    terms.len() >= 2
        && payload_len == expected_payload_len
        && terms.iter().all(|term| match *term {
            "current" | "prev_current" => true,
            other if other.starts_with("prev_carry") => other[10..].parse::<usize>().is_ok(),
            other if other.starts_with("carry") => other[5..]
                .parse::<usize>()
                .is_ok_and(|source| source < carry_index),
            _ => false,
        })
}

fn push_loop_field(state: &mut LoweringState<'_>, loop_name: &str, field: &str) -> String {
    let name = next_name(state, "loop_effect_flow_field");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "field".to_owned(),
            args: vec![loop_name.to_owned(), field.to_owned()],
        },
    });
    push_dep_edges(state, loop_name, &name);
    name
}

fn push_owned_loop_node(
    state: &mut LoweringState<'_>,
    instruction: &str,
    args: Vec<String>,
) -> String {
    let name = next_name(state, instruction);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: instruction.to_owned(),
            args: args.clone(),
        },
    });
    for input in args {
        if state.yir.nodes.iter().any(|node| node.name == input) {
            push_dep_edges(state, &input, &name);
            state.yir.edges.push(Edge {
                kind: EdgeKind::Effect,
                from: input,
                to: name.clone(),
            });
        }
    }
    super::body_lowering::chain_statement_effect(state, &name);
    name
}
