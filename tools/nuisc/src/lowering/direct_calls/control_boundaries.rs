use super::*;

pub(super) fn lower_guarded_body(
    function: &NirFunction,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let Some((
        NirStmt::If {
            condition,
            then_body,
            else_body,
        },
        tail,
    )) = function.body.split_first()
    else {
        return Err(format!(
            "outlined helper `{}` is missing its leading guard",
            function.name
        ));
    };
    let [NirStmt::Return(Some(default))] = then_body.as_slice() else {
        return Err(format!(
            "outlined helper `{}` has an invalid leading guard",
            function.name
        ));
    };
    let neutral = function
        .return_type
        .as_ref()
        .is_some_and(|ty| is_neutral_guard_seed(ty, default, &state.struct_defs));
    if !else_body.is_empty() || (!neutral && !is_pass_through_guard_seed(function, default, state))
    {
        return Err(format!(
            "outlined helper `{}` has an invalid leading guard",
            function.name
        ));
    }
    // A speculative select is not equivalent: even unused branch arithmetic can trap.
    let condition = lower_expr(condition, state, bindings)?;
    let returned = lower_expr(default, state, bindings)?;
    lower_guard_return(condition, returned, state);
    crate::lowering::body_lowering::lower_inline_stmts(tail, state, bindings, &mut BTreeMap::new())
}

// Only typed literal zeros may be prepared before the guard. This is a
// separate check from source admission; no call, projection or resource is total
// merely because an outliner emitted it as a default.
fn is_neutral_guard_seed(
    ty: &NirTypeRef,
    value: &NirExpr,
    structs: &BTreeMap<&str, &NirStructDef>,
) -> bool {
    let mut pending = vec![(ty, value)];
    while let Some((ty, value)) = pending.pop() {
        if ty.is_ref || ty.is_optional || !ty.generic_args.is_empty() {
            return false;
        }
        let scalar_zero = match (ty.name.as_str(), value) {
            ("bool", NirExpr::Bool(false)) | ("i64", NirExpr::Int(0)) => true,
            ("i32", NirExpr::CastI64ToI32(inner)) => **inner == NirExpr::Int(0),
            ("f32", NirExpr::F32(text)) | ("f64", NirExpr::F64(text)) => text == "0.0",
            _ => false,
        };
        if scalar_zero {
            continue;
        }
        let NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } = value
        else {
            return false;
        };
        let Some(definition) = structs.get(type_name.as_str()) else {
            return false;
        };
        let mut seen = BTreeSet::new();
        if type_name != &ty.name
            || !type_args.is_empty()
            || !definition.generic_params.is_empty()
            || !definition.where_bounds.is_empty()
            || fields.is_empty()
            || fields.len() != definition.fields.len()
        {
            return false;
        }
        for ((name, value), field) in fields.iter().zip(&definition.fields) {
            if name != &field.name || !seen.insert(name) {
                return false;
            }
            pending.push((&field.ty, value));
        }
    }
    true
}

#[cfg(test)]
mod guard_seed_tests {
    use super::*;

    #[test]
    fn nested_guard_zeros_require_exact_types_and_never_evaluate_work() {
        let module = crate::frontend::parse_nuis_module(
            "mod cpu Main {
            struct Leaf { flag: bool, tag: i32, gain: f32, scale: f64 }
            struct Packet { payload: Leaf }
            fn main() -> i64 { return 0; }
        }",
        )
        .unwrap();
        let structs = module
            .structs
            .iter()
            .map(|d| (d.name.as_str(), d))
            .collect();
        let scalar = |name: &str| NirTypeRef {
            name: name.into(),
            generic_args: vec![],
            is_ref: false,
            is_optional: false,
        };
        let leaf = NirExpr::StructLiteral {
            type_name: "Leaf".into(),
            type_args: vec![],
            fields: vec![
                ("flag".into(), NirExpr::Bool(false)),
                (
                    "tag".into(),
                    NirExpr::CastI64ToI32(Box::new(NirExpr::Int(0))),
                ),
                ("gain".into(), NirExpr::F32("0.0".into())),
                ("scale".into(), NirExpr::F64("0.0".into())),
            ],
        };
        for mutation in [
            "valid",
            "kind",
            "nonzero",
            "call",
            "field",
            "duplicate",
            "nominal",
        ] {
            let mut value = leaf.clone();
            let NirExpr::StructLiteral {
                type_name, fields, ..
            } = &mut value
            else {
                unreachable!()
            };
            match mutation {
                "valid" => {}
                "kind" => fields[1].1 = NirExpr::Int(0),
                "nonzero" => fields[0].1 = NirExpr::Bool(true),
                "call" => {
                    fields[0].1 = NirExpr::Call {
                        callee: "effect".into(),
                        args: vec![],
                    }
                }
                "field" => {
                    fields[0].1 = NirExpr::FieldAccess {
                        base: Box::new(NirExpr::Var("input".into())),
                        field: "flag".into(),
                    }
                }
                "duplicate" => fields[1].0 = fields[0].0.clone(),
                "nominal" => *type_name = "Other".into(),
                _ => unreachable!(),
            }
            let packet = NirExpr::StructLiteral {
                type_name: "Packet".into(),
                type_args: vec![],
                fields: vec![("payload".into(), value)],
            };
            assert_eq!(
                is_neutral_guard_seed(&scalar("Packet"), &packet, &structs),
                mutation == "valid",
                "{mutation}"
            );
        }
    }
}

fn is_pass_through_guard_seed(
    function: &NirFunction,
    value: &NirExpr,
    state: &LoweringState<'_>,
) -> bool {
    let parameter = |value: &NirExpr, kind| {
        matches!(value, NirExpr::Var(name)
        if function.params.iter().any(|param| &param.name == name
            && direct_call_scalar_kind(&param.ty) == Some(kind)))
    };
    let word_seed = |value: &NirExpr| {
        parameter(value, DirectCallScalarKind::I64)
            || matches!(value, NirExpr::CastBoolToI64(inner)
                if parameter(inner, DirectCallScalarKind::Bool))
            || is_flat_parameter_field(function, value, state)
    };
    let Some(ty) = &function.return_type else {
        return false;
    };
    if direct_call_scalar_kind(ty) == Some(DirectCallScalarKind::I64) {
        return word_seed(value);
    }
    let NirExpr::StructLiteral {
        type_name,
        type_args,
        fields,
    } = value
    else {
        return false;
    };
    let Some(definition) = state.struct_defs.get(type_name.as_str()) else {
        return false;
    };
    !ty.is_ref
        && !ty.is_optional
        && ty.generic_args.is_empty()
        && type_name == &ty.name
        && type_args.is_empty()
        && definition.generic_params.is_empty()
        && !fields.is_empty()
        && fields.len() == definition.fields.len()
        && fields
            .iter()
            .zip(&definition.fields)
            .all(|((name, value), field)| {
                name == &field.name
                    && direct_call_scalar_kind(&field.ty) == Some(DirectCallScalarKind::I64)
                    && (word_seed(value) || value == &NirExpr::Int(0))
            })
}

fn is_flat_parameter_field(
    function: &NirFunction,
    value: &NirExpr,
    state: &LoweringState<'_>,
) -> bool {
    let NirExpr::FieldAccess { base, field } = value else {
        return false;
    };
    let NirExpr::Var(name) = base.as_ref() else {
        return false;
    };
    let Some(param) = function.params.iter().find(|param| &param.name == name) else {
        return false;
    };
    let Some(definition) = state.struct_defs.get(param.ty.name.as_str()) else {
        return false;
    };
    // A captured flat-value projection is total. Calls, nested records and
    // resource-bearing fields must never become speculative guard defaults.
    !param.ty.is_ref
        && !param.ty.is_optional
        && param.ty.generic_args.is_empty()
        && definition.generic_params.is_empty()
        && definition.where_bounds.is_empty()
        && definition.fields.iter().any(|entry| &entry.name == field)
        && definition
            .fields
            .iter()
            .all(|entry| direct_call_scalar_kind(&entry.ty) == Some(DirectCallScalarKind::I64))
}

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
pub(super) fn order_guarded_function_nodes(
    state: &mut LoweringState<'_>,
    start: usize,
    preserve_source_order: bool,
) {
    let mut guard: Option<String> = None;
    let mut edges = Vec::new();
    if preserve_source_order {
        // Iterations, guarded arms and admitted scalar callees retain source evaluation order.
        edges.extend(
            state.yir.nodes[start..]
                .windows(2)
                .map(|pair| (pair[0].name.clone(), pair[1].name.clone())),
        );
    }
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
