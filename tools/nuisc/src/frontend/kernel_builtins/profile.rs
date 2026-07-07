use nuis_semantics::model::{AstExpr, NirExpr, NirResultFamily, NirResultStage};

use super::super::{
    lower_result_observer_call_with_consts, lower_result_wrapper_call_with_consts,
    ResultObserverCallInput, ResultWrapperCallInput,
};
use super::KernelBuiltinInput;

pub(super) fn lower_kernel_profile_builtin_call(
    input: KernelBuiltinInput<'_>,
) -> Result<Option<NirExpr>, String> {
    let KernelBuiltinInput {
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    } = input;
    let expr = match callee {
        "kernel_profile_bind_core" => {
            let [unit] = args else {
                return Err("kernel_profile_bind_core(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "kernel_profile_bind_core(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "kernel_profile_bind_core(...) expects a string literal unit name".to_owned(),
                );
            };
            NirExpr::KernelProfileBindCoreRef { unit: unit.clone() }
        }
        "kernel_profile_queue_depth" => {
            let [unit] = args else {
                return Err("kernel_profile_queue_depth(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "kernel_profile_queue_depth(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "kernel_profile_queue_depth(...) expects a string literal unit name".to_owned(),
                );
            };
            NirExpr::KernelProfileQueueDepthRef { unit: unit.clone() }
        }
        "kernel_profile_batch_lanes" => {
            let [unit] = args else {
                return Err("kernel_profile_batch_lanes(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "kernel_profile_batch_lanes(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "kernel_profile_batch_lanes(...) expects a string literal unit name".to_owned(),
                );
            };
            NirExpr::KernelProfileBatchLanesRef { unit: unit.clone() }
        }
        "kernel_result" => lower_result_wrapper_call_with_consts(ResultWrapperCallInput {
            name: "kernel_result",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            family: NirResultFamily::Kernel,
            build: |value, stage| match stage {
                NirResultStage::Kernel(state) => Ok(NirExpr::KernelResult { value, state }),
                other => Err(format!(
                    "expected kernel result stage, found `{}`",
                    other.render()
                )),
            },
            expected_shape: "expects a direct kernel profile/config expression",
        })?,
        "kernel_config_ready" => lower_result_observer_call_with_consts(ResultObserverCallInput {
            name: "kernel_config_ready",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            family: NirResultFamily::Kernel,
            build: |expr| NirExpr::KernelConfigReady(Box::new(expr)),
        })?,
        "kernel_value" => lower_result_observer_call_with_consts(ResultObserverCallInput {
            name: "kernel_value",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            family: NirResultFamily::Kernel,
            build: |expr| NirExpr::KernelValue(Box::new(expr)),
        })?,
        _ => return Ok(None),
    };
    Ok(Some(expr))
}
