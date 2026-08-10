use crate::frontend::is_host_execution_domain;

use std::collections::BTreeMap;

use nuis_semantics::model::{
    AstExpr, NirExpr, NirMutexCapabilityOp, NirResultFamily, NirStructDef, NirTypeRef,
};

use super::{
    ensure_mutex_guard_like, ensure_mutex_lease_like, ensure_mutex_like, ensure_mutex_permit_like,
    ensure_shared_mutex_like, ensure_spawn_input_safe, ensure_task_like, ensure_thread_like,
    i64_type, infer_nir_expr_type, lower_nested_expr_with_async_and_consts,
    lower_result_observer_call_with_consts, FunctionSignature, ModuleConstValue,
    NestedExprWithConstsInput, ResultObserverCallInput,
};

pub(super) struct TaskBuiltinInput<'a> {
    pub(super) callee: &'a str,
    pub(super) args: &'a [AstExpr],
    pub(super) current_domain: &'a str,
    pub(super) current_function_is_async: bool,
    pub(super) bindings: &'a BTreeMap<String, NirTypeRef>,
    pub(super) module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
}

pub(super) fn lower_task_builtin_call(
    input: TaskBuiltinInput<'_>,
) -> Result<Option<NirExpr>, String> {
    let TaskBuiltinInput {
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    } = input;
    macro_rules! lower_task_expr {
        ($expr:expr, $expected:expr) => {
            lower_nested_expr_with_async_and_consts(NestedExprWithConstsInput {
                expr: $expr,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                expected: $expected,
            })
        };
    }
    let expr = match callee {
        "spawn" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "spawn(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
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
                        let lowered = lower_task_expr!(arg, None)?;
                        ensure_spawn_input_safe(
                            "spawn",
                            &lowered,
                            bindings,
                            signatures,
                            struct_table,
                        )?;
                        if infer_nir_expr_type(&lowered, bindings, signatures, struct_table)
                            .is_some_and(|ty| ty.is_mutex_permit_family())
                            && !matches!(
                                &lowered,
                                NirExpr::CpuMutexCapability {
                                    op: NirMutexCapabilityOp::Permit,
                                    ..
                                }
                            )
                        {
                            return Err(
                                "spawn(...) requires a freshly issued inline MutexPermit; stored permits cannot be copied across task boundaries"
                                    .to_owned(),
                            );
                        }
                        Ok::<NirExpr, String>(lowered)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            }
        }
        "thread_spawn" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "thread_spawn(...) requires a host execution module (`mod cpu` or `mod cffi`)"
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
                        let lowered = lower_task_expr!(arg, None)?;
                        ensure_spawn_input_safe(
                            "thread_spawn",
                            &lowered,
                            bindings,
                            signatures,
                            struct_table,
                        )?;
                        if infer_nir_expr_type(&lowered, bindings, signatures, struct_table)
                            .is_some_and(|ty| ty.is_mutex_permit_family())
                            && !matches!(
                                &lowered,
                                NirExpr::CpuMutexCapability {
                                    op: NirMutexCapabilityOp::Permit,
                                    ..
                                }
                            )
                        {
                            return Err(
                                "thread_spawn(...) requires a freshly issued inline MutexPermit; stored permits cannot be copied across worker boundaries"
                                    .to_owned(),
                            );
                        }
                        Ok::<NirExpr, String>(lowered)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            }
        }
        "join" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "join(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let [task] = args else {
                return Err("join(...) expects exactly one task handle".to_owned());
            };
            let lowered = lower_task_expr!(task, None)?;
            ensure_task_like("join", &lowered, bindings, signatures, struct_table)?;
            NirExpr::CpuJoin(Box::new(lowered))
        }
        "cancel" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "cancel(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let [task] = args else {
                return Err("cancel(...) expects exactly one task handle".to_owned());
            };
            let lowered = lower_task_expr!(task, None)?;
            ensure_task_like("cancel", &lowered, bindings, signatures, struct_table)?;
            NirExpr::CpuCancel(Box::new(lowered))
        }
        "join_result" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "join_result(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let [task] = args else {
                return Err("join_result(...) expects exactly one task handle".to_owned());
            };
            let lowered = lower_task_expr!(task, None)?;
            ensure_task_like("join_result", &lowered, bindings, signatures, struct_table)?;
            NirExpr::CpuJoinResult(Box::new(lowered))
        }
        "thread_join" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "thread_join(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let [thread] = args else {
                return Err("thread_join(...) expects exactly one thread handle".to_owned());
            };
            let lowered = lower_task_expr!(thread, None)?;
            ensure_thread_like("thread_join", &lowered, bindings, signatures, struct_table)?;
            NirExpr::CpuThreadJoin(Box::new(lowered))
        }
        "thread_join_result" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "thread_join_result(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let [thread] = args else {
                return Err("thread_join_result(...) expects exactly one thread handle".to_owned());
            };
            let lowered = lower_task_expr!(thread, None)?;
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
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "mutex_new(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let [value] = args else {
                return Err("mutex_new(...) expects exactly one value".to_owned());
            };
            let lowered = lower_task_expr!(value, None)?;
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
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "mutex_lock(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let [mutex] = args else {
                return Err("mutex_lock(...) expects exactly one mutex handle".to_owned());
            };
            let lowered = lower_task_expr!(mutex, None)?;
            ensure_mutex_like("mutex_lock", &lowered, bindings, signatures, struct_table)?;
            NirExpr::CpuMutexLock(Box::new(lowered))
        }
        "mutex_unlock" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "mutex_unlock(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let [guard] = args else {
                return Err("mutex_unlock(...) expects exactly one mutex guard".to_owned());
            };
            let lowered = lower_task_expr!(guard, None)?;
            ensure_mutex_guard_like("mutex_unlock", &lowered, bindings, signatures, struct_table)?;
            NirExpr::CpuMutexUnlock(Box::new(lowered))
        }
        "mutex_value" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "mutex_value(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let [guard] = args else {
                return Err("mutex_value(...) expects exactly one mutex guard".to_owned());
            };
            let lowered = lower_task_expr!(guard, None)?;
            ensure_mutex_guard_like("mutex_value", &lowered, bindings, signatures, struct_table)?;
            NirExpr::CpuMutexValue(Box::new(lowered))
        }
        "mutex_share" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "mutex_share(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let (mutex, permit_cardinality) = match args {
                [mutex] => (mutex, 2),
                [mutex, AstExpr::Int(permit_cardinality)]
                    if (1..=64).contains(permit_cardinality) =>
                {
                    (mutex, *permit_cardinality)
                }
                [_, AstExpr::Int(permit_cardinality)] => {
                    return Err(format!(
                        "mutex_share(...) permit cardinality must be in `1..=64`, found `{permit_cardinality}`"
                    ));
                }
                [_, _] => {
                    return Err(
                        "mutex_share(...) requires a static permit cardinality literal".to_owned(),
                    );
                }
                _ => {
                    return Err(
                        "mutex_share(...) expects one mutex and an optional permit cardinality literal"
                            .to_owned(),
                    );
                }
            };
            let lowered = lower_task_expr!(mutex, None)?;
            ensure_mutex_like("mutex_share", &lowered, bindings, signatures, struct_table)?;
            NirExpr::CpuMutexCapability {
                op: NirMutexCapabilityOp::Share,
                args: vec![lowered, NirExpr::Int(permit_cardinality)],
            }
        }
        "mutex_shared_close" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "mutex_shared_close(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let [shared] = args else {
                return Err("mutex_shared_close(...) expects exactly one shared mutex".to_owned());
            };
            let lowered = lower_task_expr!(shared, None)?;
            ensure_shared_mutex_like(
                "mutex_shared_close",
                &lowered,
                bindings,
                signatures,
                struct_table,
            )?;
            NirExpr::CpuMutexCapability {
                op: NirMutexCapabilityOp::SharedClose,
                args: vec![lowered],
            }
        }
        "mutex_permit" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "mutex_permit(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let [shared, lane] = args else {
                return Err(
                    "mutex_permit(...) expects one shared mutex and one lane literal".to_owned(),
                );
            };
            let AstExpr::Int(lane) = lane else {
                return Err(
                    "mutex_permit(...) requires a unique static lane literal in `0..=63`"
                        .to_owned(),
                );
            };
            if !(0..=63).contains(lane) {
                return Err(format!(
                    "mutex_permit(...) lane literal must be in `0..=63`, found `{lane}`"
                ));
            }
            let lowered_shared = lower_task_expr!(shared, None)?;
            ensure_shared_mutex_like(
                "mutex_permit",
                &lowered_shared,
                bindings,
                signatures,
                struct_table,
            )?;
            let lowered_lane = NirExpr::Int(*lane);
            NirExpr::CpuMutexCapability {
                op: NirMutexCapabilityOp::Permit,
                args: vec![lowered_shared, lowered_lane],
            }
        }
        "mutex_permit_lock" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "mutex_permit_lock(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let [permit] = args else {
                return Err("mutex_permit_lock(...) expects exactly one permit".to_owned());
            };
            let lowered = lower_task_expr!(permit, None)?;
            ensure_mutex_permit_like(
                "mutex_permit_lock",
                &lowered,
                bindings,
                signatures,
                struct_table,
            )?;
            NirExpr::CpuMutexCapability {
                op: NirMutexCapabilityOp::PermitLock,
                args: vec![lowered],
            }
        }
        "mutex_lease_replace" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "mutex_lease_replace(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let [lease, replacement] = args else {
                return Err(
                    "mutex_lease_replace(...) expects one mutex lease and one replacement value"
                        .to_owned(),
                );
            };
            let lowered_lease = lower_task_expr!(lease, None)?;
            ensure_mutex_lease_like(
                "mutex_lease_replace",
                &lowered_lease,
                bindings,
                signatures,
                struct_table,
            )?;
            let payload_ty =
                infer_nir_expr_type(&lowered_lease, bindings, signatures, struct_table)
                    .and_then(|ty| ty.mutex_lease_payload().cloned())
                    .ok_or_else(|| {
                        "mutex_lease_replace(...) cannot resolve the lease payload type".to_owned()
                    })?;
            if !matches!(payload_ty.render().as_str(), "i32" | "i64") {
                return Err(format!(
                    "mutex_lease_replace(...) currently supports native scalar `MutexLease<i32>` or `MutexLease<i64>`, found `MutexLease<{}>`",
                    payload_ty.render()
                ));
            }
            let lowered_replacement = lower_task_expr!(replacement, Some(&payload_ty))?;
            NirExpr::CpuMutexCapability {
                op: NirMutexCapabilityOp::LeaseReplace,
                args: vec![lowered_lease, lowered_replacement],
            }
        }
        "mutex_lease_value" | "mutex_lease_unlock" => {
            if !is_host_execution_domain(current_domain) {
                return Err(format!(
                    "{callee}(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                ));
            }
            let [lease] = args else {
                return Err(format!("{callee}(...) expects exactly one mutex lease"));
            };
            let lowered = lower_task_expr!(lease, None)?;
            ensure_mutex_lease_like(callee, &lowered, bindings, signatures, struct_table)?;
            NirExpr::CpuMutexCapability {
                op: if callee == "mutex_lease_value" {
                    NirMutexCapabilityOp::LeaseValue
                } else {
                    NirMutexCapabilityOp::LeaseUnlock
                },
                args: vec![lowered],
            }
        }
        "task_completed" => lower_result_observer_call_with_consts(ResultObserverCallInput {
            name: "task_completed",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            family: NirResultFamily::Task,
            build: |expr| NirExpr::CpuTaskCompleted(Box::new(expr)),
        })?,
        "task_timed_out" => lower_result_observer_call_with_consts(ResultObserverCallInput {
            name: "task_timed_out",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            family: NirResultFamily::Task,
            build: |expr| NirExpr::CpuTaskTimedOut(Box::new(expr)),
        })?,
        "task_cancelled" => lower_result_observer_call_with_consts(ResultObserverCallInput {
            name: "task_cancelled",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            family: NirResultFamily::Task,
            build: |expr| NirExpr::CpuTaskCancelled(Box::new(expr)),
        })?,
        "task_failed" => lower_result_observer_call_with_consts(ResultObserverCallInput {
            name: "task_failed",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            family: NirResultFamily::Task,
            build: |expr| NirExpr::CpuTaskFailed(Box::new(expr)),
        })?,
        "task_value" => lower_result_observer_call_with_consts(ResultObserverCallInput {
            name: "task_value",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            family: NirResultFamily::Task,
            build: |expr| NirExpr::CpuTaskValue(Box::new(expr)),
        })?,
        "timeout" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "timeout(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let [task, limit] = args else {
                return Err("timeout(...) expects exactly two arguments: task and limit".to_owned());
            };
            let lowered_task = lower_task_expr!(task, None)?;
            ensure_task_like("timeout", &lowered_task, bindings, signatures, struct_table)?;
            let lowered_limit = lower_task_expr!(limit, Some(&i64_type()))?;
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
        "ready_after" => {
            if !is_host_execution_domain(current_domain) {
                return Err(
                    "ready_after(...) requires a host execution module (`mod cpu` or `mod cffi`)"
                        .to_owned(),
                );
            }
            let [task, delay] = args else {
                return Err(
                    "ready_after(...) expects exactly two arguments: task and delay".to_owned(),
                );
            };
            let lowered_task = lower_task_expr!(task, None)?;
            ensure_task_like(
                "ready_after",
                &lowered_task,
                bindings,
                signatures,
                struct_table,
            )?;
            let lowered_delay = lower_task_expr!(delay, Some(&i64_type()))?;
            let delay_ty = infer_nir_expr_type(&lowered_delay, bindings, signatures, struct_table)
                .ok_or_else(|| {
                    "ready_after(...) delay requires an explicit integer type".to_owned()
                })?;
            if !delay_ty.is_integer_scalar() {
                return Err(format!(
                    "ready_after(...) expects integer delay, found `{}`",
                    delay_ty.render()
                ));
            }
            NirExpr::CpuReadyAfter {
                task: Box::new(lowered_task),
                delay: Box::new(lowered_delay),
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(expr))
}
