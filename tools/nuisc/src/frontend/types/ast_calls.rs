use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstExpr, AstImplDef, AstStructDef, AstTypeRef, NirResultFamily};

use super::ast_calls_views::{infer_view_call_type, AstCallInference};
use super::ast_infer::infer_ast_expr_type_inner;
use super::ast_patterns::{
    infer_payload_constructor_ast_type_seeded, type_args_are_pattern_placeholders,
};
use super::{ast_generic_named_type, ast_make_result_type, ast_named_type, impl_lookup_types};

#[allow(clippy::too_many_arguments)]
pub(super) fn infer_ast_call_type(
    callee: &str,
    generic_args: &[AstTypeRef],
    args: &[AstExpr],
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    active_exprs: &mut BTreeSet<usize>,
) -> Option<AstTypeRef> {
    if let AstCallInference::Handled(inferred) = infer_view_call_type(
        callee,
        generic_args,
        args,
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        active_exprs,
    ) {
        return inferred;
    }

    match callee {
        _ if struct_table
            .get(callee)
            .is_some_and(|definition| definition.fields.len() == 1 && args.len() == 1) =>
        {
            let definition = struct_table.get(callee)?;
            let placeholder_names = definition
                .generic_params
                .iter()
                .map(|param| param.name.clone())
                .collect::<BTreeSet<_>>();
            if type_args_are_pattern_placeholders(generic_args, &placeholder_names) {
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
                Some(ast_generic_named_type(callee, generic_args.to_vec()))
            }
        }
        "i32_from_i64" => {
            let [value] = args else {
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
            let [call] = args else {
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
        "thread_spawn" => {
            let [call] = args else {
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
            Some(ast_generic_named_type("Thread", vec![payload]))
        }
        "timeout" => {
            let [task, _] = args else {
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
            let [task] = args else {
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
            let [task] = args else {
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
        "thread_join" => {
            let [thread] = args else {
                return None;
            };
            let thread_ty = infer_ast_expr_type_inner(
                thread,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if thread_ty.name == "Thread" && thread_ty.generic_args.len() == 1 {
                Some(thread_ty.generic_args[0].clone())
            } else {
                None
            }
        }
        "thread_join_result" => {
            let [thread] = args else {
                return None;
            };
            let thread_ty = infer_ast_expr_type_inner(
                thread,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if thread_ty.name == "Thread" && thread_ty.generic_args.len() == 1 {
                Some(ast_make_result_type(
                    NirResultFamily::Task,
                    thread_ty.generic_args[0].clone(),
                ))
            } else {
                None
            }
        }
        "mutex_new" => {
            let [value] = args else {
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
            Some(ast_generic_named_type("Mutex", vec![payload]))
        }
        "mutex_lock" => {
            let [mutex] = args else {
                return None;
            };
            let mutex_ty = infer_ast_expr_type_inner(
                mutex,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if mutex_ty.name == "Mutex" && mutex_ty.generic_args.len() == 1 {
                Some(ast_generic_named_type(
                    "MutexGuard",
                    vec![mutex_ty.generic_args[0].clone()],
                ))
            } else {
                None
            }
        }
        "mutex_unlock" => {
            let [guard] = args else {
                return None;
            };
            let guard_ty = infer_ast_expr_type_inner(
                guard,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if guard_ty.name == "MutexGuard" && guard_ty.generic_args.len() == 1 {
                Some(ast_generic_named_type(
                    "Mutex",
                    vec![guard_ty.generic_args[0].clone()],
                ))
            } else {
                None
            }
        }
        "mutex_value" => {
            let [guard] = args else {
                return None;
            };
            let guard_ty = infer_ast_expr_type_inner(
                guard,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
                active_exprs,
            )?;
            if guard_ty.name == "MutexGuard" && guard_ty.generic_args.len() == 1 {
                Some(guard_ty.generic_args[0].clone())
            } else {
                None
            }
        }
        "task_completed" | "task_timed_out" | "task_cancelled" | "data_ready" | "data_moved"
        | "data_windowed" => Some(ast_named_type("bool")),
        "task_value" => {
            let [result] = args else {
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
            let [value] = args else {
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
            let [pipe] = args else {
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
            let [input, _, _] = args else {
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
            let [input] = args else {
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
            let [window, _] = args else {
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
            let [value] = args else {
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
            let [result] = args else {
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
