use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstBinaryOp, AstExpr, AstImplDef, AstStmt, AstStructDef, AstTypeRef, AstUnaryOp,
    NirResultFamily, NirTypeRef,
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

fn parent_enum_ast_type(ty: &AstTypeRef) -> Option<AstTypeRef> {
    let (parent, _variant) = ty.name.rsplit_once('.')?;
    Some(AstTypeRef {
        name: parent.to_owned(),
        generic_args: ty.generic_args.clone(),
        is_optional: ty.is_optional,
        is_ref: ty.is_ref,
    })
}

fn impl_lookup_types(ty: &AstTypeRef) -> Vec<String> {
    let mut rendered = vec![lower_type_ref(ty).render()];
    if let Some(parent) = parent_enum_ast_type(ty) {
        rendered.push(lower_type_ref(&parent).render());
    }
    rendered
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
        AstExpr::Text(_) => Some(ast_named_type("String")),
        AstExpr::Int(_) => Some(ast_named_type("i64")),
        AstExpr::Float(_) => Some(ast_named_type("f64")),
        AstExpr::Var(name) => env.get(name).cloned(),
        AstExpr::If {
            condition: _,
            then_body,
            else_body,
        } => {
            let then_ty = infer_ast_block_result_type(
                then_body,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            let else_ty = infer_ast_block_result_type(
                else_body,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if then_ty == else_ty {
                Some(then_ty)
            } else {
                None
            }
        }
        AstExpr::Match { value: _, arms } => {
            let mut arm_ty = None;
            for arm in arms {
                let current = infer_ast_block_result_type(
                    &arm.body,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                match &arm_ty {
                    Some(existing) if *existing != current => return None,
                    None => arm_ty = Some(current),
                    _ => {}
                }
            }
            arm_ty
        }
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
            "buffer_len" => Some(ast_named_type("i64")),
            "slice" => {
                let [buffer, _, _] = args.as_slice() else {
                    return None;
                };
                let payload = match generic_args.as_slice() {
                    [] => ast_named_type("i64"),
                    [payload]
                        if *payload == ast_named_type("i64")
                            || *payload == ast_named_type("i32")
                            || *payload == ast_named_type("f32")
                            || *payload == ast_named_type("f64") =>
                    {
                        payload.clone()
                    }
                    [payload] if *payload == ast_named_type("bool") => payload.clone(),
                    [payload] => {
                        return Some(ast_generic_named_type("Slice", vec![payload.clone()]))
                    }
                    _ => return None,
                };
                let buffer_ty = infer_ast_expr_type_inner(
                    buffer,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if buffer_ty.is_ref && buffer_ty.name == "Buffer" && !buffer_ty.is_optional {
                    Some(ast_generic_named_type("Slice", vec![payload]))
                } else {
                    None
                }
            }
            "bytes" => {
                let [buffer, _, _] = args.as_slice() else {
                    return None;
                };
                if !generic_args.is_empty() {
                    return None;
                }
                let buffer_ty = infer_ast_expr_type_inner(
                    buffer,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if buffer_ty.is_ref && buffer_ty.name == "Buffer" && !buffer_ty.is_optional {
                    Some(ast_generic_named_type("Slice", vec![ast_named_type("i64")]))
                } else {
                    None
                }
            }
            "slice_len" => {
                let [base] = args.as_slice() else {
                    return None;
                };
                let base_ty = infer_ast_expr_type_inner(
                    base,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if base_ty.name == "Slice"
                    && !base_ty.is_ref
                    && !base_ty.is_optional
                    && base_ty.generic_args.len() == 1
                {
                    Some(ast_named_type("i64"))
                } else {
                    None
                }
            }
            "slice_start" => {
                let [base] = args.as_slice() else {
                    return None;
                };
                let base_ty = infer_ast_expr_type_inner(
                    base,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if base_ty.name == "Slice"
                    && !base_ty.is_ref
                    && !base_ty.is_optional
                    && base_ty.generic_args.len() == 1
                {
                    Some(ast_named_type("i64"))
                } else {
                    None
                }
            }
            "slice_buffer" => {
                let [base] = args.as_slice() else {
                    return None;
                };
                let base_ty = infer_ast_expr_type_inner(
                    base,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if base_ty.name == "Slice"
                    && !base_ty.is_ref
                    && !base_ty.is_optional
                    && base_ty.generic_args.len() == 1
                {
                    Some(AstTypeRef {
                        name: "Buffer".to_owned(),
                        generic_args: vec![],
                        is_optional: false,
                        is_ref: true,
                    })
                } else {
                    None
                }
            }
            "subslice" => {
                let [base, _, _] = args.as_slice() else {
                    return None;
                };
                let base_ty = infer_ast_expr_type_inner(
                    base,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if base_ty.name == "Slice"
                    && !base_ty.is_ref
                    && !base_ty.is_optional
                    && base_ty.generic_args.len() == 1
                {
                    match generic_args.as_slice() {
                        [] => Some(base_ty),
                        [payload] if *payload == base_ty.generic_args[0] => Some(base_ty),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            "subbytes" => {
                let [base, _, _] = args.as_slice() else {
                    return None;
                };
                if !generic_args.is_empty() {
                    return None;
                }
                let base_ty = infer_ast_expr_type_inner(
                    base,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if base_ty.name == "Slice"
                    && !base_ty.is_ref
                    && !base_ty.is_optional
                    && base_ty.generic_args.len() == 1
                    && base_ty.generic_args[0] == ast_named_type("i64")
                {
                    Some(base_ty)
                } else {
                    None
                }
            }
            "fillbytes" | "copybytes" | "comparebytes" | "bytes_fill" | "bytes_copy_from"
            | "bytes_compare" => {
                let views = match callee.as_str() {
                    "fillbytes" | "bytes_fill" => {
                        let [base, _] = args.as_slice() else {
                            return None;
                        };
                        vec![base]
                    }
                    "copybytes" | "comparebytes" | "bytes_copy_from" | "bytes_compare" => {
                        let [lhs, rhs] = args.as_slice() else {
                            return None;
                        };
                        vec![lhs, rhs]
                    }
                    _ => return None,
                };
                if !generic_args.is_empty() {
                    return None;
                }
                for view in views {
                    let view_ty = infer_ast_expr_type_inner(
                        view,
                        env,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                        active_exprs,
                    )?;
                    if view_ty.name != "Slice"
                        || view_ty.is_ref
                        || view_ty.is_optional
                        || view_ty.generic_args.len() != 1
                        || view_ty.generic_args[0] != ast_named_type("i64")
                    {
                        return None;
                    }
                }
                Some(ast_named_type("i64"))
            }
            "bytes_eq" | "bytes_starts_with" | "bytes_ends_with" => {
                let [lhs, rhs] = args.as_slice() else {
                    return None;
                };
                if !generic_args.is_empty() {
                    return None;
                }
                for view in [lhs, rhs] {
                    let view_ty = infer_ast_expr_type_inner(
                        view,
                        env,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                        active_exprs,
                    )?;
                    if view_ty.name != "Slice"
                        || view_ty.is_ref
                        || view_ty.is_optional
                        || view_ty.generic_args.len() != 1
                        || view_ty.generic_args[0] != ast_named_type("i64")
                    {
                        return None;
                    }
                }
                Some(ast_named_type("bool"))
            }
            "bytes_find_byte" | "bytes_find_text" => {
                let view = match args.as_slice() {
                    [view, _] => view,
                    _ => return None,
                };
                if !generic_args.is_empty() {
                    return None;
                }
                let view_ty = infer_ast_expr_type_inner(
                    view,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if view_ty.name == "Slice"
                    && !view_ty.is_ref
                    && !view_ty.is_optional
                    && view_ty.generic_args.len() == 1
                    && view_ty.generic_args[0] == ast_named_type("i64")
                {
                    Some(ast_named_type("i64"))
                } else {
                    None
                }
            }
            "bytes_contains_byte" | "bytes_contains_text" => {
                let view = match args.as_slice() {
                    [view, _] => view,
                    _ => return None,
                };
                if !generic_args.is_empty() {
                    return None;
                }
                let view_ty = infer_ast_expr_type_inner(
                    view,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if view_ty.name == "Slice"
                    && !view_ty.is_ref
                    && !view_ty.is_optional
                    && view_ty.generic_args.len() == 1
                    && view_ty.generic_args[0] == ast_named_type("i64")
                {
                    Some(ast_named_type("bool"))
                } else {
                    None
                }
            }
            "bytes_find_line_end" | "bytes_trim_line_end" => {
                let [view] = args.as_slice() else {
                    return None;
                };
                if !generic_args.is_empty() {
                    return None;
                }
                let view_ty = infer_ast_expr_type_inner(
                    view,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if view_ty.name == "Slice"
                    && !view_ty.is_ref
                    && !view_ty.is_optional
                    && view_ty.generic_args.len() == 1
                    && view_ty.generic_args[0] == ast_named_type("i64")
                {
                    Some(ast_named_type("i64"))
                } else {
                    None
                }
            }
            "bytes_slice_before" | "bytes_slice_after" => {
                let [view, _] = args.as_slice() else {
                    return None;
                };
                if !generic_args.is_empty() {
                    return None;
                }
                let view_ty = infer_ast_expr_type_inner(
                    view,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if view_ty.name == "Slice"
                    && !view_ty.is_ref
                    && !view_ty.is_optional
                    && view_ty.generic_args.len() == 1
                    && view_ty.generic_args[0] == ast_named_type("i64")
                {
                    Some(view_ty)
                } else {
                    None
                }
            }
            "bytes_split_once_byte" | "bytes_split_once_text" => {
                let view = match args.as_slice() {
                    [view, _] => view,
                    _ => return None,
                };
                if !generic_args.is_empty() {
                    return None;
                }
                let view_ty = infer_ast_expr_type_inner(
                    view,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if view_ty.name == "Slice"
                    && !view_ty.is_ref
                    && !view_ty.is_optional
                    && view_ty.generic_args.len() == 1
                    && view_ty.generic_args[0] == ast_named_type("i64")
                {
                    Some(ast_named_type("ByteSplit"))
                } else {
                    None
                }
            }
            "load_at" => {
                let [target, _] = args.as_slice() else {
                    return None;
                };
                let target_ty = infer_ast_expr_type_inner(
                    target,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if target_ty.name == "Slice"
                    && !target_ty.is_ref
                    && !target_ty.is_optional
                    && target_ty.generic_args.len() == 1
                {
                    Some(target_ty.generic_args[0].clone())
                } else {
                    Some(ast_named_type("i64"))
                }
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
            _ => {
                if let Some((trait_name, method)) = callee.rsplit_once('.') {
                    let receiver = args.first()?;
                    let receiver_ty = infer_ast_expr_type_inner(
                        receiver,
                        env,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                        active_exprs,
                    )?;
                    for rendered_receiver_ty in impl_lookup_types(&receiver_ty) {
                        if let Some(definition) =
                            impl_lookup.get(&(trait_name.to_owned(), rendered_receiver_ty))
                        {
                            if let Some(method_def) =
                                definition.methods.iter().find(|item| item.name == *method)
                            {
                                return method_def
                                    .return_type
                                    .clone()
                                    .or_else(|| Some(receiver_ty.clone()));
                            }
                        }
                    }
                    None
                } else {
                    function_return_types.get(callee).cloned().flatten()
                }
            }
        },
        AstExpr::MethodCall {
            receiver,
            method,
            args,
        } => {
            if let Some(trait_name) = super::super::render_field_access_path(receiver) {
                if let Some(receiver_arg) = args.first() {
                    let receiver_ty = infer_ast_expr_type_inner(
                        receiver_arg,
                        env,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                        active_exprs,
                    )?;
                    for rendered_receiver_ty in impl_lookup_types(&receiver_ty) {
                        if let Some(definition) =
                            impl_lookup.get(&(trait_name.clone(), rendered_receiver_ty))
                        {
                            if let Some(method_def) =
                                definition.methods.iter().find(|item| item.name == *method)
                            {
                                return method_def
                                    .return_type
                                    .clone()
                                    .or_else(|| Some(receiver_ty.clone()));
                            }
                        }
                    }
                }
            }
            let receiver_ty = infer_ast_expr_type_inner(
                receiver,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            for candidate_ty in impl_lookup_types(&receiver_ty) {
                for ((_, for_type), definition) in impl_lookup {
                    if *for_type != candidate_ty {
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
            if let Some(base_path) = super::super::render_field_access_path(base) {
                let qualified_name = format!("{base_path}.{field}");
                if struct_table
                    .get(&qualified_name)
                    .is_some_and(|definition| definition.fields.is_empty())
                {
                    return Some(ast_named_type(&qualified_name));
                }
            }
            let base_ty = infer_ast_expr_type_inner(
                base,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if base_ty.is_ref && !base_ty.is_optional && base_ty.name == "Node" {
                return match field.as_str() {
                    "value" => Some(ast_named_type("i64")),
                    "next" => Some(AstTypeRef {
                        name: "Node".to_owned(),
                        generic_args: vec![],
                        is_optional: false,
                        is_ref: true,
                    }),
                    _ => None,
                };
            }
            if base_ty.is_ref && !base_ty.is_optional && base_ty.name == "Buffer" {
                return match field.as_str() {
                    "len" => Some(ast_named_type("i64")),
                    _ => None,
                };
            }
            if !base_ty.is_ref && !base_ty.is_optional && base_ty.name == "Slice" {
                return match field.as_str() {
                    "buffer" => Some(AstTypeRef {
                        name: "Buffer".to_owned(),
                        generic_args: vec![],
                        is_optional: false,
                        is_ref: true,
                    }),
                    "start" | "len" => Some(ast_named_type("i64")),
                    _ => None,
                };
            }
            if !base_ty.is_ref && !base_ty.is_optional && base_ty.name == "ByteSplit" {
                return match field.as_str() {
                    "before" | "after" => {
                        Some(ast_generic_named_type("Slice", vec![ast_named_type("i64")]))
                    }
                    "index" => Some(ast_named_type("i64")),
                    "found" => Some(ast_named_type("bool")),
                    _ => None,
                };
            }
            let definition = struct_table.get(&base_ty.name)?;
            definition
                .fields
                .iter()
                .find(|item| item.name == *field)
                .map(|field| instantiate_ast_struct_field_type(&base_ty, definition, &field.ty))
        }
        AstExpr::Unary { op, operand } => match op {
            AstUnaryOp::Not => Some(ast_named_type("bool")),
            AstUnaryOp::Neg => infer_ast_expr_type_inner(
                operand,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            ),
            AstUnaryOp::Deref => {
                let operand_ty = infer_ast_expr_type_inner(
                    operand,
                    env,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    active_exprs,
                )?;
                if operand_ty.is_ref && !operand_ty.is_optional && operand_ty.name == "Node" {
                    Some(ast_named_type("i64"))
                } else {
                    None
                }
            }
        },
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
            AstBinaryOp::Add
            | AstBinaryOp::Sub
            | AstBinaryOp::Mul
            | AstBinaryOp::Div
            | AstBinaryOp::Rem => {
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

fn infer_ast_block_result_type(
    body: &[AstStmt],
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    active_exprs: &mut BTreeSet<usize>,
) -> Option<AstTypeRef> {
    match body.last() {
        Some(AstStmt::Return(Some(expr))) => infer_ast_expr_type_inner(
            expr,
            env,
            impl_lookup,
            struct_table,
            function_return_types,
            active_exprs,
        ),
        _ => None,
    }
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
    let generic_names = definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    infer_payload_constructor_ast_type_seeded(
        callee,
        definition,
        &args[0],
        &generic_names,
        BTreeMap::new(),
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        active_exprs,
    )
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
    let generic_names = definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    infer_struct_literal_ast_type_seeded(
        type_name,
        definition,
        fields,
        &generic_names,
        BTreeMap::new(),
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        active_exprs,
    )
}

pub(crate) fn infer_ast_expr_type_for_pattern(
    expr: &AstExpr,
    expected_pattern: &AstTypeRef,
    placeholder_names: &BTreeSet<String>,
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Option<AstTypeRef> {
    infer_ast_expr_type_for_pattern_inner(
        expr,
        expected_pattern,
        placeholder_names,
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        &mut BTreeSet::new(),
    )
}

fn infer_ast_expr_type_for_pattern_inner(
    expr: &AstExpr,
    expected_pattern: &AstTypeRef,
    placeholder_names: &BTreeSet<String>,
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    active_exprs: &mut BTreeSet<usize>,
) -> Option<AstTypeRef> {
    match expr {
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } if type_args.is_empty() && expected_pattern.name == *type_name => {
            let definition = struct_table.get(type_name)?;
            let generic_names = definition
                .generic_params
                .iter()
                .map(|param| param.name.clone())
                .collect::<BTreeSet<_>>();
            let seed =
                seed_ast_generic_substitutions_from_expected(definition, expected_pattern, placeholder_names);
            infer_struct_literal_ast_type_seeded(
                type_name,
                definition,
                fields,
                &generic_names,
                seed,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )
        }
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } if generic_args.is_empty() && expected_pattern.name == *callee => {
            let definition = struct_table.get(callee)?;
            if definition.fields.len() != 1 || args.len() != 1 {
                return None;
            }
            let generic_names = definition
                .generic_params
                .iter()
                .map(|param| param.name.clone())
                .collect::<BTreeSet<_>>();
            let seed =
                seed_ast_generic_substitutions_from_expected(definition, expected_pattern, placeholder_names);
            infer_payload_constructor_ast_type_seeded(
                callee,
                definition,
                &args[0],
                &generic_names,
                seed,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )
        }
        _ => infer_ast_expr_type_inner(
            expr,
            env,
            impl_lookup,
            struct_table,
            function_return_types,
            active_exprs,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn infer_struct_literal_ast_type_seeded(
    type_name: &str,
    definition: &AstStructDef,
    fields: &[(String, AstExpr)],
    generic_names: &BTreeSet<String>,
    mut substitutions: BTreeMap<String, AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    active_exprs: &mut BTreeSet<usize>,
) -> Option<AstTypeRef> {
    let mut pending = fields
        .iter()
        .map(|(name, value)| (name.as_str(), value))
        .collect::<Vec<_>>();
    while !pending.is_empty() {
        let mut progress = false;
        let mut next_pending = Vec::new();
        for (name, value) in pending {
            let field = definition.fields.iter().find(|field| field.name == name)?;
            let field_pattern =
                specialize_ast_type_pattern_with_known_substitutions(&field.ty, &substitutions);
            let value_ty = infer_ast_expr_type_for_pattern_inner(
                value,
                &field_pattern,
                generic_names,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            );
            let Some(value_ty) = value_ty else {
                next_pending.push((name, value));
                continue;
            };
            unify_ast_generic_type_pattern(&field.ty, &value_ty, generic_names, &mut substitutions)
                .ok()?;
            progress = true;
        }
        if !progress {
            return None;
        }
        pending = next_pending;
    }
    let generic_args = definition
        .generic_params
        .iter()
        .map(|param| substitutions.get(&param.name).cloned())
        .collect::<Option<Vec<_>>>()?;
    Some(ast_generic_named_type(type_name, generic_args))
}

#[allow(clippy::too_many_arguments)]
fn infer_payload_constructor_ast_type_seeded(
    callee: &str,
    definition: &AstStructDef,
    arg: &AstExpr,
    generic_names: &BTreeSet<String>,
    mut substitutions: BTreeMap<String, AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    active_exprs: &mut BTreeSet<usize>,
) -> Option<AstTypeRef> {
    let field_pattern = specialize_ast_type_pattern_with_known_substitutions(
        &definition.fields[0].ty,
        &substitutions,
    );
    let arg_ty = infer_ast_expr_type_for_pattern_inner(
        arg,
        &field_pattern,
        generic_names,
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        active_exprs,
    )?;
    unify_ast_generic_type_pattern(
        &definition.fields[0].ty,
        &arg_ty,
        generic_names,
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

fn specialize_ast_type_pattern_with_known_substitutions(
    pattern: &AstTypeRef,
    substitutions: &BTreeMap<String, AstTypeRef>,
) -> AstTypeRef {
    if pattern.generic_args.is_empty()
        && !pattern.is_optional
        && !pattern.is_ref
        && substitutions.contains_key(&pattern.name)
    {
        return substitutions
            .get(&pattern.name)
            .cloned()
            .unwrap_or_else(|| pattern.clone());
    }
    AstTypeRef {
        name: pattern.name.clone(),
        generic_args: pattern
            .generic_args
            .iter()
            .map(|arg| specialize_ast_type_pattern_with_known_substitutions(arg, substitutions))
            .collect(),
        is_optional: pattern.is_optional,
        is_ref: pattern.is_ref,
    }
}

fn seed_ast_generic_substitutions_from_expected(
    definition: &AstStructDef,
    expected_pattern: &AstTypeRef,
    placeholder_names: &BTreeSet<String>,
) -> BTreeMap<String, AstTypeRef> {
    if expected_pattern.name != definition.name
        || expected_pattern.generic_args.len() != definition.generic_params.len()
    {
        return BTreeMap::new();
    }
    definition
        .generic_params
        .iter()
        .zip(&expected_pattern.generic_args)
        .filter_map(|(param, arg)| {
            (!contains_ast_placeholder_generic_name(arg, placeholder_names))
                .then_some((param.name.clone(), arg.clone()))
        })
        .collect()
}

fn contains_ast_placeholder_generic_name(
    ty: &AstTypeRef,
    placeholder_names: &BTreeSet<String>,
) -> bool {
    (ty.generic_args.is_empty()
        && !ty.is_optional
        && !ty.is_ref
        && placeholder_names.contains(&ty.name))
        || ty
            .generic_args
            .iter()
            .any(|arg| contains_ast_placeholder_generic_name(arg, placeholder_names))
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
