use crate::registry::RegisteredAbiTarget;
use nuis_semantics::model::{AstModule, AstStmt};
use yir_core::{Operation, YirModule};

use super::profile_apply_helpers::{extract_profile_call, push_profile_node};
use super::{resolve_registered_abi_target, target_config_tokens_for_domain};
use super::{sanitize_ident, ProjectAbiResolution};

pub(in crate::project) fn materialize_default_kernel_target_config(
    ast: &AstModule,
    body: &[AstStmt],
    module: &mut YirModule,
    abi_resolution: Option<&ProjectAbiResolution>,
) -> Result<(), String> {
    let has_explicit = body.iter().any(|stmt| {
        extract_profile_call(stmt)
            .map(|(_, callee, _)| callee == "kernel_target_config")
            .unwrap_or(false)
    });
    if has_explicit {
        return Ok(());
    }
    let Some(target) = resolve_registered_abi_target("kernel", abi_resolution)? else {
        return Ok(());
    };
    let tokens = target_config_tokens_for_domain("kernel", &target);
    let name = format!(
        "project_profile_{}_{}_kernel_target_config_auto",
        sanitize_ident(&ast.domain),
        sanitize_ident(&ast.unit)
    );
    push_profile_node(
        module,
        name,
        "kernel0",
        Operation {
            module: "kernel".to_owned(),
            instruction: "target_config".to_owned(),
            args: tokens.into_args(),
        },
    );
    Ok(())
}

pub(in crate::project) fn materialize_default_shader_target_config(
    ast: &AstModule,
    module: &mut YirModule,
    abi_resolution: Option<&ProjectAbiResolution>,
) -> Result<(), String> {
    let Some(target) = resolve_registered_abi_target("shader", abi_resolution)? else {
        return Ok(());
    };
    let tokens = target_config_tokens_for_domain("shader", &target);
    let name = format!(
        "project_profile_{}_{}_shader_target_config_auto",
        sanitize_ident(&ast.domain),
        sanitize_ident(&ast.unit)
    );
    push_profile_node(
        module,
        name,
        "shader0",
        Operation {
            module: "shader".to_owned(),
            instruction: "target_config".to_owned(),
            args: tokens.into_args(),
        },
    );
    Ok(())
}

pub(in crate::project) fn materialize_default_network_target_config(
    ast: &AstModule,
    module: &mut YirModule,
    abi_resolution: Option<&ProjectAbiResolution>,
) -> Result<(), String> {
    let Some(target) = resolve_registered_abi_target("network", abi_resolution)? else {
        return Ok(());
    };
    let tokens = target_config_tokens_for_domain("network", &target);
    let name = format!(
        "project_profile_{}_{}_network_target_config_auto",
        sanitize_ident(&ast.domain),
        sanitize_ident(&ast.unit)
    );
    push_profile_node(
        module,
        name,
        "network0",
        Operation {
            module: "network".to_owned(),
            instruction: "target_config".to_owned(),
            args: tokens.into_args(),
        },
    );
    Ok(())
}

pub(in crate::project) fn kernel_target_tokens(target: &RegisteredAbiTarget) -> (String, String) {
    match target.backend_family.as_deref() {
        Some("coreml") => ("apple_ane".to_owned(), "coreml".to_owned()),
        Some("cpu-fallback") => (target.machine_arch.clone(), "cpu-fallback".to_owned()),
        Some(other) => (target.machine_arch.clone(), other.to_owned()),
        None => (target.machine_arch.clone(), target.calling_abi.clone()),
    }
}
