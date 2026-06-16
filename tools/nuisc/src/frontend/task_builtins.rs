use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirResultFamily, NirStructDef, NirTypeRef};

use super::{
    ensure_mutex_guard_like, ensure_mutex_like, ensure_spawn_input_safe, ensure_task_like,
    ensure_thread_like, i64_type, infer_nir_expr_type, lower_nested_expr_with_async_and_consts,
    lower_result_observer_call_with_consts, FunctionSignature, ModuleConstValue,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_task_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    let expr = match callee {
        "spawn" => {
            if current_domain != "cpu" {
                return Err(
                    "spawn(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let [call] = args else {
                return Err("spawn(...) expects exactly one async function call".to_owned());
            };
            let AstExpr::Call {
                callee: spawned_callee,
                generic_args: spawned_generic_args,
                args: spawned_args,
            } = call
            else {
                return Err(
                    "spawn(...) expects an async function call like `spawn(task())`".to_owned(),
                );
            };
            if !spawned_generic_args.is_empty() {
                return Err(
                    "spawn(...) does not yet support explicit generic arguments on the spawned call"
                        .to_owned(),
                );
            }
            let signature = signatures.get(spawned_callee).ok_or_else(|| {
                format!("spawn(...) references unknown function `{spawned_callee}`")
            })?;
            if !signature.is_async {
                return Err(format!(
                    "spawn(...) expects async function call, found sync function `{spawned_callee}`"
                ));
            }
            if signature.params.len() != spawned_args.len() {
                return Err(format!(
                    "function `{spawned_callee}` expects {} args, found {}",
                    signature.params.len(),
                    spawned_args.len()
                ));
            }
            NirExpr::CpuSpawn {
                callee: spawned_callee.clone(),
                args: spawned_args
                    .iter()
                    .map(|arg| {
                        let lowered = lower_nested_expr_with_async_and_consts(
                            arg,
                            current_domain,
                            current_function_is_async,
                            bindings,
                            module_consts,
                            signatures,
                            struct_table,
                            None,
                        )?;
                        ensure_spawn_input_safe(
                            "spawn",
                            &lowered,
                            bindings,
                            signatures,
                            struct_table,
                        )?;
                        Ok::<NirExpr, String>(lowered)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            }
        }
        "thread_spawn" => {
            if current_domain != "cpu" {
                return Err(
                    "thread_spawn(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [call] = args else {
                return Err("thread_spawn(...) expects exactly one async function call".to_owned());
            };
            let AstExpr::Call {
                callee: spawned_callee,
                generic_args: spawned_generic_args,
                args: spawned_args,
            } = call
            else {
                return Err(
                    "thread_spawn(...) expects an async function call like `thread_spawn(task())`"
                        .to_owned(),
                );
            };
            if !spawned_generic_args.is_empty() {
                return Err(
                    "thread_spawn(...) does not yet support explicit generic arguments on the spawned call"
                        .to_owned(),
                );
            }
            let signature = signatures.get(spawned_callee).ok_or_else(|| {
                format!("thread_spawn(...) references unknown function `{spawned_callee}`")
            })?;
            if !signature.is_async {
                return Err(format!(
                    "thread_spawn(...) expects async function call, found sync function `{spawned_callee}`"
                ));
            }
            if signature.params.len() != spawned_args.len() {
                return Err(format!(
                    "function `{spawned_callee}` expects {} args, found {}",
                    signature.params.len(),
                    spawned_args.len()
                ));
            }
            NirExpr::CpuThreadSpawn {
                callee: spawned_callee.clone(),
                args: spawned_args
                    .iter()
                    .map(|arg| {
                        let lowered = lower_nested_expr_with_async_and_consts(
                            arg,
                            current_domain,
                            current_function_is_async,
                            bindings,
                            module_consts,
                            signatures,
                            struct_table,
                            None,
                        )?;
                        ensure_spawn_input_safe(
                            "thread_spawn",
                            &lowered,
                            bindings,
                            signatures,
                            struct_table,
                        )?;
                        Ok::<NirExpr, String>(lowered)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            }
        }
        "join" => {
            if current_domain != "cpu" {
                return Err(
                    "join(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let [task] = args else {
                return Err("join(...) expects exactly one task handle".to_owned());
            };
            let lowered = lower_nested_expr_with_async_and_consts(
                task,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                None,
            )?;
            ensure_task_like("join", &lowered, bindings, signatures, struct_table)?;
            NirExpr::CpuJoin(Box::new(lowered))
        }
        "cancel" => {
            if current_domain != "cpu" {
                return Err(
                    "cancel(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let [task] = args else {
                return Err("cancel(...) expects exactly one task handle".to_owned());
            };
            let lowered = lower_nested_expr_with_async_and_consts(
                task,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                None,
            )?;
            ensure_task_like("cancel", &lowered, bindings, signatures, struct_table)?;
            NirExpr::CpuCancel(Box::new(lowered))
        }
        "join_result" => {
            if current_domain != "cpu" {
                return Err(
                    "join_result(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let [task] = args else {
                return Err("join_result(...) expects exactly one task handle".to_owned());
            };
            let lowered = lower_nested_expr_with_async_and_consts(
                task,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                None,
            )?;
            ensure_task_like("join_result", &lowered, bindings, signatures, struct_table)?;
            NirExpr::CpuJoinResult(Box::new(lowered))
        }
        "thread_join" => {
            if current_domain != "cpu" {
                return Err(
                    "thread_join(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let [thread] = args else {
                return Err("thread_join(...) expects exactly one thread handle".to_owned());
            };
            let lowered = lower_nested_expr_with_async_and_consts(
                thread,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                None,
            )?;
            ensure_thread_like("thread_join", &lowered, bindings, signatures, struct_table)?;
            NirExpr::CpuThreadJoin(Box::new(lowered))
        }
        "thread_join_result" => {
            if current_domain != "cpu" {
                return Err(
                    "thread_join_result(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [thread] = args else {
                return Err("thread_join_result(...) expects exactly one thread handle".to_owned());
            };
            let lowered = lower_nested_expr_with_async_and_consts(
                thread,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                None,
            )?;
            ensure_thread_like(
                "thread_join_result",
                &lowered,
                bindings,
                signatures,
                struct_table,
            )?;
            NirExpr::CpuThreadJoinResult(Box::new(lowered))
        }
        "mutex_new" => {
            if current_domain != "cpu" {
                return Err(
                    "mutex_new(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let [value] = args else {
                return Err("mutex_new(...) expects exactly one value".to_owned());
            };
            let lowered = lower_nested_expr_with_async_and_consts(
                value,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                None,
            )?;
            let ty = infer_nir_expr_type(&lowered, bindings, signatures, struct_table)
                .ok_or_else(|| "mutex_new(...) requires an explicit typed value".to_owned())?;
            if ty.is_ref
                || ty.is_optional
                || ty.is_result_family()
                || ty.is_concurrency_bridge_family()
            {
                return Err(format!(
                    "mutex_new(...) expects a staged mutex payload value, found `{}`",
                    ty.render()
                ));
            }
            NirExpr::CpuMutexNew(Box::new(lowered))
        }
        "mutex_lock" => {
            if current_domain != "cpu" {
                return Err(
                    "mutex_lock(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let [mutex] = args else {
                return Err("mutex_lock(...) expects exactly one mutex handle".to_owned());
            };
            let lowered = lower_nested_expr_with_async_and_consts(
                mutex,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                None,
            )?;
            ensure_mutex_like("mutex_lock", &lowered, bindings, signatures, struct_table)?;
            NirExpr::CpuMutexLock(Box::new(lowered))
        }
        "mutex_unlock" => {
            if current_domain != "cpu" {
                return Err(
                    "mutex_unlock(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let [guard] = args else {
                return Err("mutex_unlock(...) expects exactly one mutex guard".to_owned());
            };
            let lowered = lower_nested_expr_with_async_and_consts(
                guard,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                None,
            )?;
            ensure_mutex_guard_like("mutex_unlock", &lowered, bindings, signatures, struct_table)?;
            NirExpr::CpuMutexUnlock(Box::new(lowered))
        }
        "mutex_value" => {
            if current_domain != "cpu" {
                return Err(
                    "mutex_value(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let [guard] = args else {
                return Err("mutex_value(...) expects exactly one mutex guard".to_owned());
            };
            let lowered = lower_nested_expr_with_async_and_consts(
                guard,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                None,
            )?;
            ensure_mutex_guard_like("mutex_value", &lowered, bindings, signatures, struct_table)?;
            NirExpr::CpuMutexValue(Box::new(lowered))
        }
        "task_completed" => lower_result_observer_call_with_consts(
            "task_completed",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            NirResultFamily::Task,
            |expr| NirExpr::CpuTaskCompleted(Box::new(expr)),
        )?,
        "task_timed_out" => lower_result_observer_call_with_consts(
            "task_timed_out",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            NirResultFamily::Task,
            |expr| NirExpr::CpuTaskTimedOut(Box::new(expr)),
        )?,
        "task_cancelled" => lower_result_observer_call_with_consts(
            "task_cancelled",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            NirResultFamily::Task,
            |expr| NirExpr::CpuTaskCancelled(Box::new(expr)),
        )?,
        "task_value" => lower_result_observer_call_with_consts(
            "task_value",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            NirResultFamily::Task,
            |expr| NirExpr::CpuTaskValue(Box::new(expr)),
        )?,
        "timeout" => {
            if current_domain != "cpu" {
                return Err(
                    "timeout(...) is currently only allowed inside `mod cpu <unit>`".to_owned(),
                );
            }
            let [task, limit] = args else {
                return Err("timeout(...) expects exactly two arguments: task and limit".to_owned());
            };
            let lowered_task = lower_nested_expr_with_async_and_consts(
                task,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                None,
            )?;
            ensure_task_like("timeout", &lowered_task, bindings, signatures, struct_table)?;
            let lowered_limit = lower_nested_expr_with_async_and_consts(
                limit,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let limit_ty = infer_nir_expr_type(&lowered_limit, bindings, signatures, struct_table)
                .ok_or_else(|| "timeout(...) limit requires an explicit integer type".to_owned())?;
            if !limit_ty.is_integer_scalar() {
                return Err(format!(
                    "timeout(...) expects integer limit, found `{}`",
                    limit_ty.render()
                ));
            }
            NirExpr::CpuTimeout {
                task: Box::new(lowered_task),
                limit: Box::new(lowered_limit),
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(expr))
}
