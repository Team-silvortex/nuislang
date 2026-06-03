use std::collections::BTreeMap;

use nuis_semantics::model::{NirExpr, NirModule, NirStmt};
use yir_core::{OperationDomainFamily, YirModule};

use super::profile_usage::{expr_walk_any, stmt_uses_expr_predicate};
use super::support_contracts::{
    kernel_support_surface_contract, require_declared_support_surface, support_surface_for_domain,
};
use super::{
    collect_profile_int_bindings, extract_profile_call, kernel_profile_slot_targets,
    require_declared_profile_slot, split_domain_unit, support_profile_slots_for_domain,
    LoadedProject,
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
    for required_surface in kernel_support_surface_contract() {
        require_declared_support_surface(&declared_support, "kernel", &unit, required_surface)?;
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

    let target_config_uses_batch_lanes = profile_fn.body.iter().any(|stmt| {
        matches!(
            extract_profile_call(stmt),
            Some((_name, "kernel_target_config", args))
                if matches!(args.get(2), Some(super::AstExpr::Var(name)) if name == "batch_lanes")
        )
    });
    if !target_config_uses_batch_lanes {
        return Err(format!(
            "project kernel unit `kernel.{}` requires kernel_target_config(..., batch_lanes) to consume the `batch_lanes` profile slot",
            unit
        ));
    }

    Ok(())
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
