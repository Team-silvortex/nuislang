use super::*;

pub(in crate::lowering) fn collect_aggregate_param_direct_call_functions(
    module: &NirModule,
) -> BTreeSet<String> {
    let definitions = module
        .structs
        .iter()
        .map(|definition| (definition.name.as_str(), definition))
        .collect::<BTreeMap<_, _>>();
    let reachable = reachable_function_names(module);
    module
        .functions
        .iter()
        .filter(|function| !function.is_async)
        .filter(|function| reachable.contains(&function.name))
        .filter(|function| {
            function.params.iter().all(|param| {
                direct_call_scalar_kind(&param.ty).is_some()
                    || flattenable_struct_type(&param.ty, &definitions, &mut BTreeSet::new())
            })
        })
        .filter(|function| {
            let aggregate_parameter = function
                .params
                .iter()
                .any(|param| definitions.contains_key(param.ty.name.as_str()));
            let owned_return = function
                .return_type
                .as_ref()
                .and_then(|ty| module_owned_struct_layout(module, ty))
                .is_some();
            let scalar_return = direct_call_return_kind(function)
                .is_some_and(|kind| kind != DirectCallScalarKind::OwnedExternalBuffer);
            owned_return || (aggregate_parameter && scalar_return)
        })
        .map(|function| function.name.clone())
        .collect()
}

fn reachable_function_names(module: &NirModule) -> BTreeSet<String> {
    let definitions = module
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect::<BTreeMap<_, _>>();
    let eligible = definitions.keys().copied().collect::<BTreeSet<_>>();
    let mut reachable = BTreeSet::new();
    let mut pending = vec!["main".to_owned()];
    while let Some(name) = pending.pop() {
        if !reachable.insert(name.clone()) {
            continue;
        }
        let Some(function) = definitions.get(name.as_str()) else {
            continue;
        };
        pending.extend(
            super::function_called_functions(function, &function.body, &eligible)
                .into_iter()
                .filter(|called| !reachable.contains(called)),
        );
    }
    reachable
}

pub(super) fn lower_direct_call_parameters(
    function: &NirFunction,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
    function_parameters: &mut Vec<YirFunctionParameter>,
) -> Result<(), String> {
    let mut physical_index = 0usize;
    for param in &function.params {
        if direct_call_scalar_kind(&param.ty).is_some() {
            let node = materialize_scalar_parameter(
                &function.name,
                &param.name,
                &param.ty,
                &mut physical_index,
                state,
                function_parameters,
            )?;
            bindings.insert(param.name.clone(), node);
            continue;
        }
        let node = materialize_struct_parameter(
            &function.name,
            &param.name,
            &param.ty,
            &mut physical_index,
            state,
            function_parameters,
        )?;
        bindings.insert(param.name.clone(), node);
    }
    Ok(())
}

pub(super) fn flatten_direct_call_arguments(
    function: &NirFunction,
    args: &[String],
    state: &mut LoweringState<'_>,
) -> Result<Vec<String>, String> {
    if function.params.len() != args.len() {
        return Err(format!(
            "function `{}` expects {} args, found {}",
            function.name,
            function.params.len(),
            args.len()
        ));
    }
    let mut flattened = Vec::new();
    for (param, arg) in function.params.iter().zip(args) {
        flattened.extend(flatten_direct_call_argument(&param.ty, arg, state)?);
    }
    Ok(flattened)
}

pub(in crate::lowering) fn flatten_direct_call_argument(
    ty: &NirTypeRef,
    arg: &str,
    state: &mut LoweringState<'_>,
) -> Result<Vec<String>, String> {
    if direct_call_scalar_kind(ty).is_some() {
        return Ok(vec![arg.to_owned()]);
    }
    let mut flattened = Vec::new();
    flatten_struct_argument(ty, arg, state, &mut flattened)?;
    Ok(flattened)
}

fn flattenable_struct_type(
    ty: &NirTypeRef,
    definitions: &BTreeMap<&str, &NirStructDef>,
    visiting: &mut BTreeSet<String>,
) -> bool {
    if ty.is_ref
        || ty.is_optional
        || !ty.generic_args.is_empty()
        || !visiting.insert(ty.name.clone())
    {
        return false;
    }
    let Some(definition) = definitions.get(ty.name.as_str()) else {
        visiting.remove(&ty.name);
        return false;
    };
    let valid = !definition.fields.is_empty()
        && definition.generic_params.is_empty()
        && definition.fields.iter().all(|field| {
            direct_call_scalar_kind(&field.ty).is_some_and(is_value_struct_leaf)
                || flattenable_struct_type(&field.ty, definitions, visiting)
        });
    visiting.remove(&ty.name);
    valid
}

fn is_value_struct_leaf(kind: DirectCallScalarKind) -> bool {
    matches!(
        kind,
        DirectCallScalarKind::Bool
            | DirectCallScalarKind::I32
            | DirectCallScalarKind::I64
            | DirectCallScalarKind::F32
            | DirectCallScalarKind::F64
    )
}

fn materialize_scalar_parameter(
    function_name: &str,
    parameter_name: &str,
    ty: &NirTypeRef,
    physical_index: &mut usize,
    state: &mut LoweringState<'_>,
    function_parameters: &mut Vec<YirFunctionParameter>,
) -> Result<String, String> {
    let kind = direct_call_scalar_kind(ty).ok_or_else(|| {
        format!(
            "ordinary direct-call lowering does not support parameter `{parameter_name}` type `{}` in `{function_name}`",
            ty.render()
        )
    })?;
    let instruction = parameter_instruction(kind)?;
    let node_name = format!("__fn_{function_name}_param_{physical_index}");
    state.yir.nodes.push(Node {
        name: node_name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: instruction.to_owned(),
            args: vec![physical_index.to_string()],
        },
    });
    function_parameters.push(YirFunctionParameter {
        name: parameter_name.to_owned(),
        ty: ty.render(),
        ownership: yir_value_ownership(ty),
        node: node_name.clone(),
    });
    *physical_index += 1;
    Ok(node_name)
}

fn materialize_struct_parameter(
    function_name: &str,
    parameter_path: &str,
    ty: &NirTypeRef,
    physical_index: &mut usize,
    state: &mut LoweringState<'_>,
    function_parameters: &mut Vec<YirFunctionParameter>,
) -> Result<String, String> {
    let fields = struct_fields(ty, state)?;
    let mut struct_args = vec![ty.name.clone()];
    let mut field_nodes = Vec::new();
    for (field_name, field_ty) in fields {
        let field_path = format!("{parameter_path}.{field_name}");
        let field_node = if direct_call_scalar_kind(&field_ty).is_some() {
            materialize_scalar_parameter(
                function_name,
                &field_path,
                &field_ty,
                physical_index,
                state,
                function_parameters,
            )?
        } else {
            materialize_struct_parameter(
                function_name,
                &field_path,
                &field_ty,
                physical_index,
                state,
                function_parameters,
            )?
        };
        struct_args.push(format!("{field_name}={field_node}"));
        field_nodes.push(field_node);
    }
    let struct_node = next_name(state, "param_struct");
    state.yir.nodes.push(Node {
        name: struct_node.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "struct".to_owned(),
            args: struct_args,
        },
    });
    for field_node in field_nodes {
        push_dep_edges(state, &field_node, &struct_node);
    }
    Ok(struct_node)
}

fn flatten_struct_argument(
    ty: &NirTypeRef,
    base_node: &str,
    state: &mut LoweringState<'_>,
    flattened: &mut Vec<String>,
) -> Result<(), String> {
    for (field_name, field_ty) in struct_fields(ty, state)? {
        let field_node = next_name(state, "call_arg_field");
        state.yir.nodes.push(Node {
            name: field_node.clone(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "field".to_owned(),
                args: vec![base_node.to_owned(), field_name],
            },
        });
        push_dep_edges(state, base_node, &field_node);
        if direct_call_scalar_kind(&field_ty).is_some() {
            flattened.push(field_node);
        } else {
            flatten_struct_argument(&field_ty, &field_node, state, flattened)?;
        }
    }
    Ok(())
}

fn struct_fields(
    ty: &NirTypeRef,
    state: &LoweringState<'_>,
) -> Result<Vec<(String, NirTypeRef)>, String> {
    state
        .struct_defs
        .get(ty.name.as_str())
        .map(|definition| {
            definition
                .fields
                .iter()
                .map(|field| (field.name.clone(), field.ty.clone()))
                .collect()
        })
        .ok_or_else(|| {
            format!(
                "direct-call aggregate type `{}` is unavailable",
                ty.render()
            )
        })
}

fn parameter_instruction(kind: DirectCallScalarKind) -> Result<&'static str, String> {
    match kind {
        DirectCallScalarKind::Bool => Ok("param_bool"),
        DirectCallScalarKind::I32 => Ok("param_i32"),
        DirectCallScalarKind::I64 => Ok("param_i64"),
        DirectCallScalarKind::F32 => Ok("param_f32"),
        DirectCallScalarKind::F64 => Ok("param_f64"),
        DirectCallScalarKind::BorrowedBuffer => Ok("param_buffer_ref"),
        DirectCallScalarKind::TraversalPointer => Ok("param_node_ref"),
        DirectCallScalarKind::OwnedBytes => Ok("param_owned_bytes"),
        DirectCallScalarKind::OwnedExternalBuffer => {
            Err("owned external buffers cannot be helper parameters".to_owned())
        }
    }
}
