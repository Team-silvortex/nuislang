use std::collections::BTreeSet;

use nuis_semantics::model::{NirExpr, NirFunction, NirModule, NirStmt, NirTypeRef};

pub(crate) fn insert_owned_bytes_cleanup(module: &mut NirModule) -> bool {
    module
        .functions
        .iter_mut()
        .fold(false, |changed, function| {
            insert_function_cleanup(function) || changed
        })
}

fn insert_function_cleanup(function: &mut NirFunction) -> bool {
    if contains_while(&function.body)
        || function.return_type.is_none() && contains_value_return(&function.body)
    {
        return false;
    }

    let live = function
        .params
        .iter()
        .filter(|param| is_bytes_type(Some(&param.ty)))
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let declaration_order = function
        .params
        .iter()
        .filter(|param| is_bytes_type(Some(&param.ty)))
        .map(|param| param.name.clone())
        .collect::<Vec<_>>();
    let mut used_names = function
        .params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    collect_binding_names(&function.body, &mut used_names);

    let mut context = CleanupContext {
        return_type: function.return_type.as_ref(),
        used_names,
        return_index: 0,
        changed: false,
    };
    let state = CleanupState {
        live,
        declaration_order,
    };
    let Ok((mut rewritten, flow)) = rewrite_block(function.body.clone(), state, &mut context)
    else {
        return false;
    };
    if let CleanupFlow::Continues(mut state) = flow {
        if !state.live.is_empty() {
            append_drops(&mut rewritten, &state.declaration_order, &mut state.live);
            context.changed = true;
        }
    }
    function.body = rewritten;
    context.changed
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CleanupState {
    live: BTreeSet<String>,
    declaration_order: Vec<String>,
}

enum CleanupFlow {
    Continues(CleanupState),
    Terminates,
}

struct CleanupContext<'a> {
    return_type: Option<&'a NirTypeRef>,
    used_names: BTreeSet<String>,
    return_index: usize,
    changed: bool,
}

fn rewrite_block(
    stmts: Vec<NirStmt>,
    mut state: CleanupState,
    context: &mut CleanupContext<'_>,
) -> Result<(Vec<NirStmt>, CleanupFlow), ()> {
    let mut rewritten = Vec::with_capacity(stmts.len());
    let mut remaining = stmts.into_iter();

    while let Some(stmt) = remaining.next() {
        match stmt {
            NirStmt::Let { name, ty, value } => {
                let transfers_bytes =
                    direct_binding_name(&value).is_some_and(|source| state.live.contains(source));
                consume_for_binding(&value, &ty, &mut state.live);
                if transfers_bytes {
                    consume_direct(&value, &mut state.live);
                }
                let owns_bytes = is_bytes_type(ty.as_ref())
                    || matches!(value, NirExpr::CopyBufferOwned(_))
                    || transfers_bytes;
                if owns_bytes {
                    state.live.insert(name.clone());
                    state.declaration_order.push(name.clone());
                }
                rewritten.push(NirStmt::Let { name, ty, value });
            }
            NirStmt::Expr(NirExpr::DropBytes(inner)) => {
                if let Some(name) = direct_binding_name(&inner) {
                    state.live.remove(name);
                }
                rewritten.push(NirStmt::Expr(NirExpr::DropBytes(inner)));
            }
            NirStmt::Return(value) => {
                rewrite_return(value, &mut rewritten, &mut state, context);
                rewritten.extend(remaining);
                return Ok((rewritten, CleanupFlow::Terminates));
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                let scope_start = state.declaration_order.len();
                let (then_body, then_flow) =
                    rewrite_branch(then_body, state.clone(), scope_start, context)?;
                let (else_body, else_flow) =
                    rewrite_branch(else_body, state.clone(), scope_start, context)?;
                rewritten.push(NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                });
                match merge_branch_flows(then_flow, else_flow)? {
                    CleanupFlow::Continues(merged) => state = merged,
                    CleanupFlow::Terminates => {
                        rewritten.extend(remaining);
                        return Ok((rewritten, CleanupFlow::Terminates));
                    }
                }
            }
            other => rewritten.push(other),
        }
    }

    Ok((rewritten, CleanupFlow::Continues(state)))
}

fn rewrite_return(
    value: Option<NirExpr>,
    out: &mut Vec<NirStmt>,
    state: &mut CleanupState,
    context: &mut CleanupContext<'_>,
) {
    if let Some(value) = value.as_ref() {
        consume_return_value(value, &mut state.live);
    }
    if state.live.is_empty() {
        out.push(NirStmt::Return(value));
        return;
    }

    let return_value = match (value, context.return_type) {
        (Some(value), Some(return_type)) => {
            let temp = fresh_return_name(&mut context.used_names, &mut context.return_index);
            out.push(NirStmt::Let {
                name: temp.clone(),
                ty: Some(return_type.clone()),
                value,
            });
            Some(NirExpr::Var(temp))
        }
        (value, _) => value,
    };
    append_drops(out, &state.declaration_order, &mut state.live);
    out.push(NirStmt::Return(return_value));
    context.changed = true;
}

fn rewrite_branch(
    stmts: Vec<NirStmt>,
    state: CleanupState,
    scope_start: usize,
    context: &mut CleanupContext<'_>,
) -> Result<(Vec<NirStmt>, CleanupFlow), ()> {
    let (mut rewritten, flow) = rewrite_block(stmts, state, context)?;
    let CleanupFlow::Continues(mut state) = flow else {
        return Ok((rewritten, CleanupFlow::Terminates));
    };
    let branch_locals = state.declaration_order[scope_start..].to_vec();
    if branch_locals.iter().any(|name| state.live.contains(name)) {
        append_drops(&mut rewritten, &branch_locals, &mut state.live);
        context.changed = true;
    }
    state.declaration_order.truncate(scope_start);
    Ok((rewritten, CleanupFlow::Continues(state)))
}

fn merge_branch_flows(then_flow: CleanupFlow, else_flow: CleanupFlow) -> Result<CleanupFlow, ()> {
    match (then_flow, else_flow) {
        (CleanupFlow::Terminates, CleanupFlow::Terminates) => Ok(CleanupFlow::Terminates),
        (CleanupFlow::Continues(state), CleanupFlow::Terminates)
        | (CleanupFlow::Terminates, CleanupFlow::Continues(state)) => {
            Ok(CleanupFlow::Continues(state))
        }
        (CleanupFlow::Continues(then_state), CleanupFlow::Continues(else_state)) => (then_state
            == else_state)
            .then_some(CleanupFlow::Continues(then_state))
            .ok_or(()),
    }
}

fn contains_while(stmts: &[NirStmt]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        NirStmt::While { .. } => true,
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => contains_while(then_body) || contains_while(else_body),
        _ => false,
    })
}

fn contains_value_return(stmts: &[NirStmt]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        NirStmt::Return(Some(_)) => true,
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => contains_value_return(then_body) || contains_value_return(else_body),
        NirStmt::While { body, .. } => contains_value_return(body),
        _ => false,
    })
}

fn collect_binding_names(stmts: &[NirStmt], names: &mut BTreeSet<String>) {
    for stmt in stmts {
        match stmt {
            NirStmt::Let { name, .. } | NirStmt::Const { name, .. } => {
                names.insert(name.clone());
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_binding_names(then_body, names);
                collect_binding_names(else_body, names);
            }
            NirStmt::While { body, .. } => collect_binding_names(body, names),
            _ => {}
        }
    }
}

fn is_bytes_type(ty: Option<&NirTypeRef>) -> bool {
    ty.is_some_and(|ty| ty.name == "Bytes" && !ty.is_ref && ty.generic_args.is_empty())
}

fn consume_for_binding(
    value: &NirExpr,
    target_type: &Option<NirTypeRef>,
    live: &mut BTreeSet<String>,
) {
    if is_bytes_type(target_type.as_ref()) {
        consume_direct(value, live);
    } else {
        consume_owned_aggregate(value, live);
    }
}

fn consume_return_value(value: &NirExpr, live: &mut BTreeSet<String>) {
    consume_direct(value, live);
    consume_owned_aggregate(value, live);
}

fn consume_direct(expr: &NirExpr, live: &mut BTreeSet<String>) {
    if let Some(name) = direct_binding_name(expr) {
        live.remove(name);
    }
}

fn consume_owned_aggregate(expr: &NirExpr, live: &mut BTreeSet<String>) {
    match expr {
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                consume_direct(value, live);
                consume_owned_aggregate(value, live);
            }
        }
        NirExpr::CpuSpawn { args, .. }
        | NirExpr::CpuThreadSpawn { args, .. }
        | NirExpr::Call { args, .. } => {
            for arg in args {
                consume_direct(arg, live);
                consume_owned_aggregate(arg, live);
            }
        }
        NirExpr::Move(inner) | NirExpr::DropBytes(inner) => consume_direct(inner, live),
        _ => {}
    }
}

fn direct_binding_name(expr: &NirExpr) -> Option<&str> {
    match expr {
        NirExpr::Var(name) => Some(name),
        NirExpr::Move(inner) => direct_binding_name(inner),
        _ => None,
    }
}

fn append_drops(out: &mut Vec<NirStmt>, declaration_order: &[String], live: &mut BTreeSet<String>) {
    for name in declaration_order.iter().rev() {
        if live.remove(name) {
            out.push(NirStmt::Expr(NirExpr::DropBytes(Box::new(NirExpr::Var(
                name.clone(),
            )))));
        }
    }
}

fn fresh_return_name(used_names: &mut BTreeSet<String>, return_index: &mut usize) -> String {
    loop {
        let candidate = format!("__nuis_owned_cleanup_return_{}", *return_index);
        *return_index += 1;
        if used_names.insert(candidate.clone()) {
            return candidate;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuis_semantics::model::{NirParam, NirVisibility};

    fn ty(name: &str) -> NirTypeRef {
        NirTypeRef {
            name: name.into(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }
    }

    fn function(body: Vec<NirStmt>, return_type: Option<NirTypeRef>) -> NirFunction {
        NirFunction {
            visibility: NirVisibility::Private,
            name: "sample".into(),
            annotations: Vec::new(),
            test_name: None,
            test_ignored: false,
            test_should_fail: false,
            test_reason: None,
            test_timeout_ms: None,
            test_clock_domain: None,
            test_clock_policy: None,
            benchmark_name: None,
            benchmark_warmup_iters: None,
            benchmark_measure_iters: None,
            benchmark_timeout_ms: None,
            benchmark_clock_domain: None,
            benchmark_clock_policy: None,
            is_async: false,
            generic_params: Vec::new(),
            where_bounds: Vec::new(),
            params: Vec::<NirParam>::new(),
            return_type,
            body,
        }
    }

    fn bytes_let(name: &str) -> NirStmt {
        NirStmt::Let {
            name: name.into(),
            ty: Some(ty("Bytes")),
            value: NirExpr::CopyBufferOwned(Box::new(NirExpr::Var("buffer".into()))),
        }
    }

    fn drop_name(stmt: &NirStmt) -> Option<&str> {
        match stmt {
            NirStmt::Expr(NirExpr::DropBytes(inner)) => direct_binding_name(inner),
            _ => None,
        }
    }

    #[test]
    fn inserts_drop_after_return_value_evaluation() {
        let mut function = function(
            vec![bytes_let("bytes"), NirStmt::Return(Some(NirExpr::Int(7)))],
            Some(ty("i64")),
        );
        assert!(insert_function_cleanup(&mut function));
        assert!(matches!(
            function.body.as_slice(),
            [
                NirStmt::Let { name, .. },
                NirStmt::Let { name: temp, .. },
                NirStmt::Expr(NirExpr::DropBytes(_)),
                NirStmt::Return(Some(NirExpr::Var(result)))
            ] if name == "bytes" && temp == result
        ));
    }

    #[test]
    fn does_not_duplicate_explicit_drop() {
        let mut function = function(
            vec![
                bytes_let("bytes"),
                NirStmt::Expr(NirExpr::DropBytes(Box::new(NirExpr::Var("bytes".into())))),
                NirStmt::Return(None),
            ],
            None,
        );
        assert!(!insert_function_cleanup(&mut function));
        assert_eq!(function.body.iter().filter_map(drop_name).count(), 1);
    }

    #[test]
    fn struct_return_transfers_owned_bytes() {
        let mut function = function(
            vec![
                bytes_let("bytes"),
                NirStmt::Return(Some(NirExpr::StructLiteral {
                    type_name: "Packet".into(),
                    type_args: Vec::new(),
                    fields: vec![("bytes".into(), NirExpr::Var("bytes".into()))],
                })),
            ],
            Some(ty("Packet")),
        );
        assert!(!insert_function_cleanup(&mut function));
        assert_eq!(function.body.iter().filter_map(drop_name).count(), 0);
    }

    #[test]
    fn transfers_inferred_alias_and_drops_only_new_owner() {
        let mut function = function(
            vec![
                bytes_let("bytes"),
                NirStmt::Let {
                    name: "alias".into(),
                    ty: None,
                    value: NirExpr::Var("bytes".into()),
                },
                NirStmt::Return(None),
            ],
            None,
        );
        assert!(insert_function_cleanup(&mut function));
        assert_eq!(function.body.get(2).and_then(drop_name), Some("alias"));
    }

    #[test]
    fn cleans_up_owned_bytes_parameter() {
        let mut function = function(vec![NirStmt::Return(None)], None);
        function.params.push(NirParam {
            name: "payload".into(),
            ty: ty("Bytes"),
        });
        assert!(insert_function_cleanup(&mut function));
        assert_eq!(function.body.first().and_then(drop_name), Some("payload"));
    }

    #[test]
    fn leaves_loops_for_cfg_cleanup() {
        let mut function = function(
            vec![
                bytes_let("bytes"),
                NirStmt::While {
                    condition: NirExpr::Bool(true),
                    body: vec![NirStmt::Break],
                },
            ],
            None,
        );
        assert!(!insert_function_cleanup(&mut function));
        assert_eq!(function.body.len(), 2);
    }

    #[test]
    fn cleans_branch_locals_before_merging_outer_owner() {
        let mut function = function(
            vec![
                bytes_let("outer"),
                NirStmt::If {
                    condition: NirExpr::Bool(true),
                    then_body: vec![bytes_let("left")],
                    else_body: vec![bytes_let("right")],
                },
                NirStmt::Return(None),
            ],
            None,
        );
        assert!(insert_function_cleanup(&mut function));
        let NirStmt::If {
            then_body,
            else_body,
            ..
        } = &function.body[1]
        else {
            panic!("expected if");
        };
        assert_eq!(then_body.last().and_then(drop_name), Some("left"));
        assert_eq!(else_body.last().and_then(drop_name), Some("right"));
        assert_eq!(function.body.get(2).and_then(drop_name), Some("outer"));
    }

    #[test]
    fn early_return_drops_outer_owner_only_on_returning_path() {
        let mut function = function(
            vec![
                bytes_let("outer"),
                NirStmt::If {
                    condition: NirExpr::Bool(true),
                    then_body: vec![NirStmt::Return(Some(NirExpr::Int(1)))],
                    else_body: Vec::new(),
                },
                NirStmt::Return(Some(NirExpr::Int(2))),
            ],
            Some(ty("i64")),
        );
        assert!(insert_function_cleanup(&mut function));
        let NirStmt::If { then_body, .. } = &function.body[1] else {
            panic!("expected if");
        };
        assert!(then_body
            .iter()
            .any(|stmt| drop_name(stmt) == Some("outer")));
        assert!(function
            .body
            .iter()
            .any(|stmt| drop_name(stmt) == Some("outer")));
    }

    #[test]
    fn rejects_inconsistent_live_state_at_branch_merge() {
        let original = vec![
            bytes_let("outer"),
            NirStmt::If {
                condition: NirExpr::Bool(true),
                then_body: vec![NirStmt::Expr(NirExpr::DropBytes(Box::new(NirExpr::Var(
                    "outer".into(),
                ))))],
                else_body: Vec::new(),
            },
            NirStmt::Return(None),
        ];
        let mut function = function(original.clone(), None);
        assert!(!insert_function_cleanup(&mut function));
        assert_eq!(function.body, original);
    }
}
