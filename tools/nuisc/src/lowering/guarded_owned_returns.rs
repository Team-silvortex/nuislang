use super::*;
use nuis_semantics::model::NirParam;
use std::collections::VecDeque;
use yir_core::native_resource_session::{MAX_RETURN_HELPER_DEPTH, MAX_RETURN_HELPER_FUNCTIONS};

#[path = "guarded_owned_returns/branches.rs"]
mod branches;
#[path = "guarded_owned_returns/continuations.rs"]
mod continuations;
#[path = "guarded_owned_returns/owners.rs"]
mod owners;
#[path = "guarded_owned_returns/regions.rs"]
mod regions;

#[derive(Default)]
pub(super) struct Outlines {
    pub functions: BTreeSet<String>,
    pub owners: BTreeMap<String, OwnerEdges>,
}

#[derive(Clone)]
pub(super) struct OwnerEdges {
    pub cleanup: Vec<String>,
    pub transfers: Vec<String>,
}

// Outline the complete arm, not just its returned expression: allocations,
// checked arithmetic and borrowed-buffer writes must all stay behind the guard.
pub(super) fn outline(
    module: &mut NirModule,
    host_entries: &BTreeSet<String>,
) -> Result<Outlines, String> {
    let eligible = module
        .functions
        .iter()
        .filter(|f| {
            host_entries.contains(&f.name)
                && !f.is_async
                && f.generic_params.is_empty()
                && f.params
                    .first()
                    .is_some_and(|p| p.ty.is_ref && p.ty.name == "Buffer")
                && f.params[1..].iter().all(|p| scalar(&p.ty))
                && f.return_type
                    .as_ref()
                    .and_then(|t| module_owned_struct_layout(module, t))
                    .is_some_and(|layout| layout.contains(":Bytes"))
        })
        .map(|f| f.name.clone())
        .collect::<BTreeSet<_>>();
    let reserved = module
        .functions
        .iter()
        .map(|f| f.name.clone())
        .collect::<BTreeSet<_>>();
    let mut context = OutlineContext {
        reserved,
        pending: VecDeque::new(),
        roots: host_entries.len(),
        outlines: Outlines::default(),
    };
    for function in &mut module.functions {
        if !eligible.contains(&function.name) {
            continue;
        }
        outline_body(function, 1, &mut context)?;
    }
    while let Some((mut helper, depth)) = context.pending.pop_front() {
        outline_body(&mut helper, depth, &mut context)?;
        context.outlines.functions.insert(helper.name.clone());
        module.functions.push(helper);
    }
    Ok(context.outlines)
}

struct OutlineContext {
    reserved: BTreeSet<String>,
    pending: VecDeque<(NirFunction, usize)>,
    roots: usize,
    outlines: Outlines,
}

fn outline_body(
    function: &mut NirFunction,
    depth: usize,
    context: &mut OutlineContext,
) -> Result<(), String> {
    let template = function.clone();
    let mut scope = function
        .params
        .iter()
        .filter(|p| !owned_bytes(&p.ty))
        .cloned()
        .collect::<Vec<_>>();
    let mut live_bytes = function
        .params
        .iter()
        .filter(|p| owned_bytes(&p.ty))
        .map(|p| p.name.clone())
        .collect::<BTreeSet<_>>();
    let mut remaining = VecDeque::from(std::mem::take(&mut function.body));
    while let Some(mut statement) = remaining.pop_front() {
        if let NirStmt::If {
            then_body,
            else_body,
            ..
        } = &mut statement
        {
            if !else_body.is_empty() {
                let fallback = branches::pair(
                    &template,
                    then_body,
                    else_body,
                    &mut remaining,
                    &scope,
                    &live_bytes,
                    depth,
                    context,
                )?;
                function.body.push(statement);
                remaining.clear();
                remaining.push_back(fallback);
                continue;
            }
            if else_body.is_empty()
                && regions::block(then_body, depth + 1)?
                && !regions::ends_in_return(then_body)
            {
                continuations::check_captures(then_body, &scope, &live_bytes)?;
                let tail = continuations::share(
                    &template,
                    std::mem::take(&mut remaining).into_iter().collect(),
                    &scope,
                    &live_bytes,
                    depth + 1,
                    context,
                )?;
                then_body.push(tail.clone());
                remaining.push_back(tail);
            }
            if else_body.is_empty()
                && regions::terminal_return_arm(then_body, depth + 1)?
                && (then_body.iter().any(allocates)
                    || !live_bytes.is_empty()
                    || matches!(then_body.last(), Some(NirStmt::Return(Some(NirExpr::Call { callee, .. }))) if context.outlines.owners.contains_key(callee))
                    || then_body.iter().any(|s| matches!(s, NirStmt::If { .. })))
            {
                let call = branches::arm(
                    &template,
                    std::mem::take(then_body),
                    &scope,
                    &live_bytes,
                    depth + 1,
                    context,
                )?;
                *then_body = vec![call];
            }
        }
        owners::advance(&statement, &mut live_bytes);
        let declaration = match &statement {
            NirStmt::Let {
                name, ty: Some(ty), ..
            }
            | NirStmt::Const { name, ty, .. } => Some((name.clone(), ty.clone())),
            _ => None,
        };
        if let Some((name, ty)) = declaration {
            if scalar(&ty) {
                if let Some(old) = scope.iter_mut().find(|p| p.name == name) {
                    old.ty = ty;
                } else {
                    scope.push(NirParam { name, ty });
                }
            }
        }
        function.body.push(statement);
    }
    Ok(())
}

fn scalar(ty: &NirTypeRef) -> bool {
    ty.name == "i64" && !ty.is_ref && !ty.is_optional && ty.generic_args.is_empty()
}

fn owned_bytes(ty: &NirTypeRef) -> bool {
    ty.name == "Bytes" && !ty.is_ref && !ty.is_optional && ty.generic_args.is_empty()
}

fn allocates(stmt: &NirStmt) -> bool {
    let expr = match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Expr(value)
        | NirStmt::Return(Some(value)) => value,
        _ => return false,
    };
    fn visit(expr: &NirExpr) -> bool {
        match expr {
            NirExpr::CopyBufferOwned(_) => true,
            NirExpr::StructLiteral { fields, .. } => fields.iter().any(|(_, v)| visit(v)),
            NirExpr::Call { args, .. } => args.iter().any(visit),
            NirExpr::Move(value) => visit(value),
            _ => false,
        }
    }
    visit(expr)
}

pub(super) fn lower(
    condition: &str,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<bool, String> {
    let [NirStmt::Return(Some(NirExpr::Call { callee, args }))] = then_body else {
        return Ok(false);
    };
    if !else_body.is_empty() || !state.guarded_owned_helpers.contains_key(callee) {
        return Ok(false);
    }
    Ok(emit(Some(condition), callee, args, state, bindings)?.is_some())
}

pub(super) fn lower_tail(
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let NirExpr::Call { callee, args } = value else {
        return Ok(None);
    };
    if !state.guarded_owned_helpers.contains_key(callee) {
        return Ok(None);
    }
    emit(None, callee, args, state, bindings)
}

fn emit(
    condition: Option<&str>,
    callee: &str,
    args: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let function = state.function_map[callee];
    let ownership = &state.guarded_owned_helpers[callee];
    let Some(input_count) = function.params.len().checked_sub(ownership.transfers.len()) else {
        return Err("guarded owned helper parameter count drift".into());
    };
    let Some(layout) = function_owned_struct_layout(function, state) else {
        return Ok(None);
    };
    let current = state
        .call_stack
        .last()
        .and_then(|n| state.function_map.get(n.as_str()));
    if current
        .and_then(|f| function_owned_struct_layout(f, state))
        .as_ref()
        != Some(&layout)
        || function.params.len() != args.len()
        || !function
            .params
            .first()
            .is_some_and(|p| p.ty.is_ref && p.ty.name == "Buffer")
        || input_count == 0
        || !function.params[1..input_count]
            .iter()
            .all(|p| scalar(&p.ty))
        || !function.params[input_count..].iter().all(|p| {
            p.ty.name == "Bytes"
                && !p.ty.is_ref
                && !p.ty.is_optional
                && p.ty.generic_args.is_empty()
        })
        || !args.iter().all(|arg| matches!(arg, NirExpr::Var(_)))
    {
        return Ok(None);
    }
    let inputs = args
        .iter()
        .map(|arg| lower_expr(arg, state, bindings))
        .collect::<Result<Vec<_>, _>>()?;
    let cleanup = state.guarded_owned_helpers[callee]
        .cleanup
        .iter()
        .map(|binding| {
            bindings
                .get(binding)
                .cloned()
                .ok_or_else(|| format!("unavailable guarded Bytes owner `{binding}`"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let name = next_name(state, "guard_owned_call");
    let mut arguments = condition.into_iter().map(str::to_owned).collect::<Vec<_>>();
    arguments.extend([callee.to_owned(), layout]);
    arguments.extend(inputs[..input_count].iter().cloned());
    if inputs.len() > input_count {
        arguments.push(format!(
            "{}{}",
            yir_core::guarded_owned_call::TRANSFER_PREFIX,
            inputs.len() - input_count
        ));
        arguments.extend(inputs[input_count..].iter().cloned());
    }
    if !cleanup.is_empty() {
        arguments.push(format!(
            "{}{}",
            yir_core::guarded_owned_call::CLEANUP_PREFIX,
            cleanup.len()
        ));
        arguments.extend(cleanup.iter().cloned());
    }
    let instruction = if condition.is_some() {
        yir_core::guarded_owned_call::parse(&arguments)?;
        "guard_call_owned_struct_return"
    } else {
        yir_core::guarded_owned_call::parse_tail(&arguments)?;
        "tail_call_owned_struct_return"
    };
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".into(),
            instruction: instruction.into(),
            args: arguments,
        },
    });
    for input in condition
        .into_iter()
        .chain(inputs.iter().map(String::as_str))
    {
        push_dep_edges(state, input, &name);
        super::edge_helpers::push_effect_edge(state, input, &name);
    }
    push_lifetime_edge(state, &inputs[0], &name);
    for owner in &inputs[input_count..] {
        push_lifetime_edge(state, owner, &name);
    }
    for owner in cleanup {
        push_dep_edges(state, &owner, &name);
        push_lifetime_edge(state, &owner, &name);
        super::edge_helpers::push_effect_edge(state, &owner, &name);
    }
    chain_statement_effect(state, &name);
    Ok(Some(name))
}
