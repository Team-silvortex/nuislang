use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstBinaryOp, AstExpr, AstImplDef, AstStructDef, AstTypeRef, NirResultFamily, NirTypeRef,
};

use super::super::lower_type_ref;
use super::super::validation_binding_env::instantiate_ast_struct_field_type;

pub(crate) fn ast_type_from_nir(ty: &NirTypeRef) -> AstTypeRef {
    AstTypeRef {
        name: ty.name.clone(),
        generic_args: ty.generic_args.iter().map(ast_type_from_nir).collect(),
        is_optional: ty.is_optional,
        is_ref: ty.is_ref,
    }
}

pub(crate) fn ast_named_type(name: &str) -> AstTypeRef {
    AstTypeRef {
        name: name.to_owned(),
        generic_args: vec![],
        is_optional: false,
        is_ref: false,
    }
}

pub(crate) fn ast_generic_named_type(name: &str, generic_args: Vec<AstTypeRef>) -> AstTypeRef {
    AstTypeRef {
        name: name.to_owned(),
        generic_args,
        is_optional: false,
        is_ref: false,
    }
}

pub(crate) fn ast_make_result_type(family: NirResultFamily, payload: AstTypeRef) -> AstTypeRef {
    let name = match family {
        NirResultFamily::Task => "TaskResult",
        NirResultFamily::Data => "DataResult",
        NirResultFamily::Shader => "ShaderResult",
        NirResultFamily::Kernel => "KernelResult",
        NirResultFamily::Network => "NetworkResult",
    };
    ast_generic_named_type(name, vec![payload])
}

pub(crate) fn infer_ast_expr_type(
    expr: &AstExpr,
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Option<AstTypeRef> {
    infer_ast_expr_type_inner(
        expr,
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        &mut BTreeSet::new(),
    )
}

pub(crate) fn infer_ast_expr_type_inner(
    expr: &AstExpr,
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    active_exprs: &mut BTreeSet<usize>,
) -> Option<AstTypeRef> {
    let expr_key = expr as *const AstExpr as usize;
    if !active_exprs.insert(expr_key) {
        return None;
    }
    let inferred = match expr {
        AstExpr::Bool(_) => Some(ast_named_type("bool")),
        AstExpr::Text(_) => Some(ast_named_type("Text")),
        AstExpr::Int(_) => Some(ast_named_type("i64")),
        AstExpr::Var(name) => env.get(name).cloned(),
        AstExpr::Lambda { .. } => None,
        AstExpr::Invoke { .. } => None,
        AstExpr::Await(value) => infer_ast_expr_type_inner(
            value,
            env,
            impl_lookup,
            struct_table,
            function_return_types,
            active_exprs,
        )
        .and_then(|ty| {
            if ty.name == "Task" && ty.generic_args.len() == 1 {
                ty.generic_args.first().cloned()
            } else {
                Some(ty)
            }
        }),
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => match callee.as_str() {
            _ if struct_table
                .get(callee)
                .is_some_and(|definition| definition.fields.len() == 1 && args.len() == 1) =>
            {
                if generic_args.is_empty() {
                    infer_payload_constructor_ast_type(
                        callee,
                        args,
                        env,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                        active_exprs,
                    )
                } else {
                    Some(ast_generic_named_type(callee, generic_args.clone()))
                }
            }
            "i32_from_i64" => {
                let [value] = args.as_slice() else {
                    return None;
                };
                let inner = infer_ast_expr_type_inner(
                    value,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if inner.name == "i64" && !inner.is_ref && !inner.is_optional {
                    Some(ast_named_type("i32"))
                } else {
                    None
                }
            }
            "spawn" => {
                let [call] = args.as_slice() else {
                    return None;
                };
                let AstExpr::Call {
                    callee: spawned_callee,
                    ..
                } = call
                else {
                    return None;
                };
                let payload = function_return_types
                    .get(spawned_callee)
                    .cloned()
                    .flatten()?;
                Some(ast_generic_named_type("Task", vec![payload]))
            }
            "timeout" => {
                let [task, _] = args.as_slice() else {
                    return None;
                };
                infer_ast_expr_type_inner(
                    task,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )
            }
            "join" => {
                let [task] = args.as_slice() else {
                    return None;
                };
                let task_ty = infer_ast_expr_type_inner(
                    task,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if task_ty.name == "Task" && task_ty.generic_args.len() == 1 {
                    Some(task_ty.generic_args[0].clone())
                } else {
                    None
                }
            }
            "join_result" => {
                let [task] = args.as_slice() else {
                    return None;
                };
                let task_ty = infer_ast_expr_type_inner(
                    task,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if task_ty.name == "Task" && task_ty.generic_args.len() == 1 {
                    Some(ast_make_result_type(
                        NirResultFamily::Task,
                        task_ty.generic_args[0].clone(),
                    ))
                } else {
                    None
                }
            }
            "task_completed" | "task_timed_out" | "task_cancelled" | "data_ready"
            | "data_moved" | "data_windowed" => Some(ast_named_type("bool")),
            "task_value" => {
                let [result] = args.as_slice() else {
                    return None;
                };
                let result_ty = infer_ast_expr_type_inner(
                    result,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if result_ty.name == "TaskResult" && result_ty.generic_args.len() == 1 {
                    Some(result_ty.generic_args[0].clone())
                } else {
                    None
                }
            }
            "data_output_pipe" => {
                let [value] = args.as_slice() else {
                    return None;
                };
                let inner = infer_ast_expr_type_inner(
                    value,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                Some(ast_generic_named_type("Pipe", vec![inner]))
            }
            "data_input_pipe" => {
                let [pipe] = args.as_slice() else {
                    return None;
                };
                let pipe_ty = infer_ast_expr_type_inner(
                    pipe,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                pipe_ty.generic_args.first().cloned()
            }
            "data_copy_window" => {
                let [input, _, _] = args.as_slice() else {
                    return None;
                };
                let inner = infer_ast_expr_type_inner(
                    input,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                let payload = if inner.is_ref && inner.name == "Buffer" {
                    ast_named_type("i64")
                } else {
                    inner
                };
                Some(ast_generic_named_type("WindowMut", vec![payload]))
            }
            "data_freeze_window" => {
                let [input] = args.as_slice() else {
                    return None;
                };
                let input_ty = infer_ast_expr_type_inner(
                    input,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if (input_ty.name == "Window" || input_ty.name == "WindowMut")
                    && input_ty.generic_args.len() == 1
                {
                    Some(ast_generic_named_type(
                        "Window",
                        vec![input_ty.generic_args[0].clone()],
                    ))
                } else {
                    None
                }
            }
            "data_read_window" => {
                let [window, _] = args.as_slice() else {
                    return None;
                };
                let window_ty = infer_ast_expr_type_inner(
                    window,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if (window_ty.name == "Window" || window_ty.name == "WindowMut")
                    && window_ty.generic_args.len() == 1
                {
                    Some(window_ty.generic_args[0].clone())
                } else {
                    None
                }
            }
            "data_result" => {
                let [value] = args.as_slice() else {
                    return None;
                };
                let payload = infer_ast_expr_type_inner(
                    value,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                Some(ast_make_result_type(NirResultFamily::Data, payload))
            }
            "data_value" => {
                let [result] = args.as_slice() else {
                    return None;
                };
                let result_ty = infer_ast_expr_type_inner(
                    result,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if result_ty.name == "DataResult" && result_ty.generic_args.len() == 1 {
                    Some(result_ty.generic_args[0].clone())
                } else {
                    None
                }
            }
            _ => function_return_types.get(callee).cloned().flatten(),
        },
        AstExpr::MethodCall {
            receiver,
            method,
            args: _,
        } => {
            let receiver_ty = infer_ast_expr_type_inner(
                receiver,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            for ((_, for_type), definition) in impl_lookup {
                if *for_type != lower_type_ref(&receiver_ty).render() {
                    continue;
                }
                if let Some(method_def) =
                    definition.methods.iter().find(|item| item.name == *method)
                {
                    return method_def
                        .return_type
                        .clone()
                        .or_else(|| Some(receiver_ty.clone()));
                }
            }
            None
        }
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => {
            let definition = struct_table.get(type_name)?;
            if definition.generic_params.is_empty() {
                Some(ast_named_type(type_name))
            } else if type_args.len() == definition.generic_params.len() {
                Some(ast_generic_named_type(type_name, type_args.clone()))
            } else {
                infer_struct_literal_ast_type(
                    type_name,
                    fields,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )
            }
        }
        AstExpr::FieldAccess { base, field } => {
            let base_ty = infer_ast_expr_type_inner(
                base,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            let definition = struct_table.get(&base_ty.name)?;
            definition
                .fields
                .iter()
                .find(|item| item.name == *field)
                .map(|field| instantiate_ast_struct_field_type(&base_ty, definition, &field.ty))
        }
        AstExpr::Binary { op, lhs, rhs } => match op {
            AstBinaryOp::Eq
            | AstBinaryOp::Ne
            | AstBinaryOp::Lt
            | AstBinaryOp::Le
            | AstBinaryOp::Gt
            | AstBinaryOp::Ge => Some(ast_named_type("bool")),
            AstBinaryOp::And | AstBinaryOp::Or => {
                let lhs_ty = infer_ast_expr_type_inner(
                    lhs,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                let rhs_ty = infer_ast_expr_type_inner(
                    rhs,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if lower_type_ref(&lhs_ty).render() == lower_type_ref(&rhs_ty).render()
                    && lhs_ty.name == "bool"
                    && rhs_ty.name == "bool"
                {
                    Some(ast_named_type("bool"))
                } else {
                    None
                }
            }
            AstBinaryOp::Add | AstBinaryOp::Sub | AstBinaryOp::Mul | AstBinaryOp::Div => {
                let lhs_ty = infer_ast_expr_type_inner(
                    lhs,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                let rhs_ty = infer_ast_expr_type_inner(
                    rhs,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if lower_type_ref(&lhs_ty).render() == lower_type_ref(&rhs_ty).render() {
                    Some(lhs_ty)
                } else {
                    None
                }
            }
        },
        AstExpr::Instantiate { .. } => None,
    };
    active_exprs.remove(&expr_key);
    inferred
}

fn infer_payload_constructor_ast_type(
    callee: &str,
    args: &[AstExpr],
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    active_exprs: &mut BTreeSet<usize>,
) -> Option<AstTypeRef> {
    let definition = struct_table.get(callee)?;
    if definition.generic_params.is_empty() || definition.fields.len() != 1 || args.len() != 1 {
        return Some(ast_named_type(callee));
    }
    let arg_ty = infer_ast_expr_type_inner(
        &args[0],
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        active_exprs,
    )?;
    let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
    let generic_names = definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    unify_ast_generic_type_pattern(
        &definition.fields[0].ty,
        &arg_ty,
        &generic_names,
        &mut substitutions,
    )
    .ok()?;
    let generic_args = definition
        .generic_params
        .iter()
        .map(|param| substitutions.get(&param.name).cloned())
        .collect::<Option<Vec<_>>>()?;
    Some(ast_generic_named_type(callee, generic_args))
}

fn infer_struct_literal_ast_type(
    type_name: &str,
    fields: &[(String, AstExpr)],
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    active_exprs: &mut BTreeSet<usize>,
) -> Option<AstTypeRef> {
    let definition = struct_table.get(type_name)?;
    let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
    let generic_names = definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    for (name, value) in fields {
        let field = definition.fields.iter().find(|field| field.name == *name)?;
        let value_ty = infer_ast_expr_type_inner(
            value,
            env,
            impl_lookup,
            struct_table,
            function_return_types,
            active_exprs,
        )?;
        unify_ast_generic_type_pattern(&field.ty, &value_ty, &generic_names, &mut substitutions)
            .ok()?;
    }
    let generic_args = definition
        .generic_params
        .iter()
        .map(|param| substitutions.get(&param.name).cloned())
        .collect::<Option<Vec<_>>>()?;
    Some(ast_generic_named_type(type_name, generic_args))
}

fn unify_ast_generic_type_pattern(
    pattern: &AstTypeRef,
    concrete: &AstTypeRef,
    generic_names: &BTreeSet<String>,
    substitutions: &mut BTreeMap<String, AstTypeRef>,
) -> Result<(), ()> {
    if generic_names.contains(&pattern.name) && pattern.generic_args.is_empty() {
        if let Some(existing) = substitutions.get(&pattern.name) {
            if lower_type_ref(existing).render() != lower_type_ref(concrete).render() {
                return Err(());
            }
        } else {
            substitutions.insert(pattern.name.clone(), concrete.clone());
        }
        return Ok(());
    }
    if pattern.name != concrete.name
        || pattern.generic_args.len() != concrete.generic_args.len()
        || pattern.is_optional != concrete.is_optional
        || pattern.is_ref != concrete.is_ref
    {
        return Err(());
    }
    for (pattern_arg, concrete_arg) in pattern.generic_args.iter().zip(&concrete.generic_args) {
        unify_ast_generic_type_pattern(pattern_arg, concrete_arg, generic_names, substitutions)?;
    }
    Ok(())
}
