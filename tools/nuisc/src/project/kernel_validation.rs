use std::collections::BTreeMap;

use nuis_semantics::model::{NirExpr, NirModule, NirStmt};
use yir_core::{OperationDomainFamily, YirModule};

use super::profile_apply::{resolve_registered_abi_target, target_config_tokens_for_domain};
use super::profile_usage::{expr_walk_any, stmt_uses_expr_predicate};
use super::support_contracts::support_surface_for_domain;
use super::{
    collect_profile_int_bindings, extract_profile_call, kernel_profile_slot_targets,
    require_declared_profile_slot, resolve_project_abi, split_domain_unit,
    support_profile_slots_for_domain, AstExpr, LoadedProject, ProjectAbiResolution,
};

pub(super) fn validate_kernel_profile_for_link(
    project: &LoadedProject,
    module: &YirModule,
    endpoint: &str,
) -> Result<(), String> {
    let (domain, unit) = split_domain_unit(endpoint)?;
    if domain != "kernel" {
        return Ok(());
    }
    let declared_support = support_surface_for_domain(&mut BTreeMap::new(), "kernel")?;
    let declared_slots = support_profile_slots_for_domain("kernel")?;
    if declared_support.is_empty() {
        return Err(format!(
            "project kernel unit `kernel.{}` requires nustar to declare at least one support surface",
            unit
        ));
    }

    for (slot, node_name) in kernel_profile_slot_targets(&unit) {
        require_declared_profile_slot(&declared_slots, "kernel", &unit, slot)?;
        let exists = module.nodes.iter().any(|node| node.name == node_name);
        if !exists {
            return Err(format!(
                "project kernel unit `kernel.{}` requires support profile slot `{}` in YIR",
                unit, slot
            ));
        }
    }

    let has_kernel_work = module
        .nodes
        .iter()
        .any(|node| node.op.is_domain_family(OperationDomainFamily::Kernel));
    if !has_kernel_work {
        return Err(format!(
            "project kernel unit `kernel.{}` requires at least one kernel.* node in YIR",
            unit
        ));
    }

    validate_kernel_profile_slot_contract(project, &unit)?;
    validate_kernel_target_config_contract(project, &unit)?;

    Ok(())
}

pub(super) fn nir_uses_kernel_profile_bind_core(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_kernel_profile_bind_core(stmt, unit))
    })
}

pub(super) fn nir_uses_kernel_profile_queue_depth(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_kernel_profile_queue_depth(stmt, unit))
    })
}

pub(super) fn nir_uses_kernel_profile_batch_lanes(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_kernel_profile_batch_lanes(stmt, unit))
    })
}

pub(super) fn validate_kernel_profile_slot_contract(
    project: &LoadedProject,
    unit: &str,
) -> Result<(), String> {
    let profile_module = project
        .modules
        .iter()
        .find(|module| module.ast.domain == "kernel" && module.ast.unit == unit)
        .ok_or_else(|| format!("project is missing support module `kernel.{unit}`"))?;
    let profile_fn = profile_module
        .ast
        .functions
        .iter()
        .find(|function| function.name == "profile")
        .ok_or_else(|| {
            format!(
                "project kernel unit `kernel.{}` requires a `profile()` function",
                unit
            )
        })?;
    let int_bindings = collect_profile_int_bindings(&profile_fn.body);

    let bind_core = int_bindings.get("bind_core").copied().ok_or_else(|| {
        format!(
            "project kernel unit `kernel.{}` requires `bind_core` profile const",
            unit
        )
    })?;
    let queue_depth = int_bindings.get("queue_depth").copied().ok_or_else(|| {
        format!(
            "project kernel unit `kernel.{}` requires `queue_depth` profile const",
            unit
        )
    })?;
    let batch_lanes = int_bindings.get("batch_lanes").copied().ok_or_else(|| {
        format!(
            "project kernel unit `kernel.{}` requires `batch_lanes` profile const",
            unit
        )
    })?;

    if bind_core < 0 {
        return Err(format!(
            "project kernel unit `kernel.{}` requires `bind_core >= 0`",
            unit
        ));
    }
    if queue_depth <= 0 {
        return Err(format!(
            "project kernel unit `kernel.{}` requires `queue_depth > 0`",
            unit
        ));
    }
    if batch_lanes <= 0 {
        return Err(format!(
            "project kernel unit `kernel.{}` requires `batch_lanes > 0`",
            unit
        ));
    }

    let explicit_target_config = profile_fn.body.iter().find_map(|stmt| {
        let call = extract_profile_call(stmt)?;
        (call.1 == "kernel_target_config").then_some(call)
    });
    if matches!(
        explicit_target_config,
        Some((_name, "kernel_target_config", args))
            if !matches!(args.get(2), Some(super::AstExpr::Var(name)) if name == "batch_lanes")
    ) {
        return Err(format!(
            "project kernel unit `kernel.{}` requires kernel_target_config(..., batch_lanes) to consume the `batch_lanes` profile slot",
            unit
        ));
    }

    Ok(())
}

pub(super) fn validate_kernel_target_config_contract(
    project: &LoadedProject,
    unit: &str,
) -> Result<(), String> {
    let resolution = resolve_project_abi(project)?;
    let Some(target) = resolve_registered_abi_target("kernel", Some(&resolution))? else {
        return Ok(());
    };
    let profile_module = project
        .modules
        .iter()
        .find(|module| module.ast.domain == "kernel" && module.ast.unit == unit)
        .ok_or_else(|| format!("project is missing support module `kernel.{unit}`"))?;
    let profile_fn = profile_module
        .ast
        .functions
        .iter()
        .find(|function| function.name == "profile")
        .ok_or_else(|| {
            format!(
                "project kernel unit `kernel.{}` requires a `profile()` function",
                unit
            )
        })?;

    let expected = expected_kernel_target_config_literals(&resolution, &target)?;
    for stmt in &profile_fn.body {
        let Some((_name, "kernel_target_config", args)) = extract_profile_call(stmt) else {
            continue;
        };
        let actual_arch = expect_kernel_target_config_text(args, 0, "kernel_target_config")?;
        let actual_runtime = expect_kernel_target_config_text(args, 1, "kernel_target_config")?;
        if actual_arch != expected.0 || actual_runtime != expected.1 {
            return Err(format!(
                "project kernel unit `kernel.{}` requires kernel_target_config(\"{}\", \"{}\", ...) to match selected ABI `{}`",
                unit, expected.0, expected.1, target.abi
            ));
        }
    }

    Ok(())
}

fn expected_kernel_target_config_literals(
    resolution: &ProjectAbiResolution,
    target: &crate::registry::RegisteredAbiTarget,
) -> Result<(String, String), String> {
    resolution
        .requirements
        .iter()
        .find(|item| item.domain == "kernel")
        .ok_or_else(|| {
            "missing kernel ABI requirement while validating kernel profile".to_owned()
        })?;
    let tokens = target_config_tokens_for_domain("kernel", target);
    Ok((tokens.arch, tokens.runtime))
}

fn expect_kernel_target_config_text(
    args: &[AstExpr],
    index: usize,
    callee: &str,
) -> Result<String, String> {
    match args.get(index) {
        Some(AstExpr::Text(value)) => Ok(value.clone()),
        _ => Err(format!(
            "{callee}(...) expects string literal arg {}",
            index + 1
        )),
    }
}

fn stmt_uses_kernel_profile_bind_core(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_kernel_profile_bind_core(value, unit)
    })
}

fn stmt_uses_kernel_profile_queue_depth(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_kernel_profile_queue_depth(value, unit)
    })
}

fn stmt_uses_kernel_profile_batch_lanes(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_kernel_profile_batch_lanes(value, unit)
    })
}

fn expr_uses_kernel_profile_bind_core(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::KernelProfileBindCoreRef { unit: kernel_unit } => kernel_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_kernel_profile_bind_core(inner, unit)
        }),
    }
}

fn expr_uses_kernel_profile_queue_depth(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::KernelProfileQueueDepthRef { unit: kernel_unit } => kernel_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_kernel_profile_queue_depth(inner, unit)
        }),
    }
}

fn expr_uses_kernel_profile_batch_lanes(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::KernelProfileBatchLanesRef { unit: kernel_unit } => kernel_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_kernel_profile_batch_lanes(inner, unit)
        }),
    }
}
