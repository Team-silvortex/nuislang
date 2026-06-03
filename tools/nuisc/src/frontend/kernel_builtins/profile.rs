use std::collections::BTreeMap;

use nuis_semantics::model::{
    AstExpr, NirExpr, NirResultFamily, NirResultStage, NirStructDef, NirTypeRef,
};

use super::super::{
    lower_result_observer_call_with_consts, lower_result_wrapper_call_with_consts,
    FunctionSignature, ModuleConstValue,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_kernel_profile_builtin_call(
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
        "kernel_result" => lower_result_wrapper_call_with_consts(
            "kernel_result",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            NirResultFamily::Kernel,
            |value, stage| match stage {
                NirResultStage::Kernel(state) => Ok(NirExpr::KernelResult { value, state }),
                other => Err(format!(
                    "expected kernel result stage, found `{}`",
                    other.render()
                )),
            },
            "expects a direct kernel profile/config expression",
        )?,
        "kernel_config_ready" => lower_result_observer_call_with_consts(
            "kernel_config_ready",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            NirResultFamily::Kernel,
            |expr| NirExpr::KernelConfigReady(Box::new(expr)),
        )?,
        "kernel_value" => lower_result_observer_call_with_consts(
            "kernel_value",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            NirResultFamily::Kernel,
            |expr| NirExpr::KernelValue(Box::new(expr)),
        )?,
        _ => return Ok(None),
    };
    Ok(Some(expr))
}
