use std::collections::BTreeMap;

use crate::registry::RegisteredAbiTarget;
use nuis_semantics::model::{AstModule, AstStmt};
use yir_core::{Operation, YirModule};

use super::{backend_features_for_registered_abi_target, sanitize_ident, ProjectAbiResolution};

#[path = "profile_apply_helpers.rs"]
mod profile_apply_helpers;
#[path = "profile_apply_targets.rs"]
mod profile_apply_targets;

pub(super) use profile_apply_helpers::{
    collect_profile_int_bindings, ensure_project_resource, extract_profile_call, push_profile_node,
};
use profile_apply_helpers::{
    expect_profile_int_arg, expect_profile_value_input_name, expect_text_arg,
    extract_profile_int_binding,
};
use profile_apply_targets::{
    kernel_target_tokens, materialize_default_kernel_target_config,
    materialize_default_network_target_config, materialize_default_shader_target_config,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TargetConfigTokens {
    pub arch: String,
    pub runtime: String,
    pub lane_width: String,
    pub backend_features: String,
}

impl TargetConfigTokens {
    pub(super) fn into_args(self) -> Vec<String> {
        vec![
            self.arch,
            self.runtime,
            self.lane_width,
            self.backend_features,
        ]
    }
}

pub(super) fn apply_support_module_profile(
    ast: &AstModule,
    module: &mut YirModule,
    abi_resolution: Option<&ProjectAbiResolution>,
) -> Result<(), String> {
    let Some(profile) = ast
        .functions
        .iter()
        .find(|function| function.name == "profile")
    else {
        return Ok(());
    };
    let int_bindings = collect_profile_int_bindings(&profile.body);

    match ast.domain.as_str() {
        "shader" => {
            ensure_project_resource(
                module,
                "shader0",
                &project_resource_kind("shader", abi_resolution)?,
            );
            materialize_default_shader_target_config(ast, module, abi_resolution)?;
            for stmt in &profile.body {
                apply_shader_profile_stmt(ast, stmt, module, &int_bindings)?;
            }
        }
        "kernel" => {
            ensure_project_resource(
                module,
                "kernel0",
                &project_resource_kind("kernel", abi_resolution)?,
            );
            for stmt in &profile.body {
                apply_kernel_profile_stmt(ast, stmt, module, &int_bindings)?;
            }
            materialize_default_kernel_target_config(ast, &profile.body, module, abi_resolution)?;
        }
        "network" => {
            ensure_project_resource(
                module,
                "network0",
                &project_resource_kind("network", abi_resolution)?,
            );
            materialize_default_network_target_config(ast, module, abi_resolution)?;
            for stmt in &profile.body {
                apply_network_profile_stmt(ast, stmt, module)?;
            }
        }
        "data" => {
            ensure_project_resource(module, "fabric0", "data.fabric");
            for stmt in &profile.body {
                apply_data_profile_stmt(ast, stmt, module, &int_bindings)?;
            }
        }
        _ => {}
    }

    Ok(())
}

fn project_resource_kind(
    domain: &str,
    abi_resolution: Option<&ProjectAbiResolution>,
) -> Result<String, String> {
    let Some(target) = resolve_registered_abi_target(domain, abi_resolution)? else {
        return Ok(match domain {
            "shader" => "shader.render".to_owned(),
            "kernel" => "kernel.compute".to_owned(),
            "network" => "network.io".to_owned(),
            "data" => "data.fabric".to_owned(),
            other => other.to_owned(),
        });
    };
    Ok(match domain {
        "shader" => {
            let backend = target
                .backend_family
                .as_deref()
                .unwrap_or("render")
                .replace('-', "_");
            format!("shader.{backend}")
        }
        "kernel" => {
            let backend = target.backend_family.as_deref().unwrap_or("compute");
            if backend == "coreml" {
                "kernel.apple".to_owned()
            } else {
                format!("kernel.{}", backend.replace('-', "_"))
            }
        }
        "network" => match target.machine_os.as_str() {
            "darwin" => "network.urlsession".to_owned(),
            "windows" => "network.winsock".to_owned(),
            _ => "network.socket".to_owned(),
        },
        "data" => "data.fabric".to_owned(),
        other => other.to_owned(),
    })
}

pub(super) fn resolve_registered_abi_target(
    domain: &str,
    abi_resolution: Option<&ProjectAbiResolution>,
) -> Result<Option<RegisteredAbiTarget>, String> {
    let Some(abi_resolution) = abi_resolution else {
        return Ok(None);
    };
    let Some(abi) = abi_resolution
        .requirements
        .iter()
        .find(|item| item.domain == domain)
        .map(|item| item.abi.as_str())
    else {
        return Ok(None);
    };
    let manifest =
        crate::registry::load_manifest_for_domain(std::path::Path::new("nustar-packages"), domain)?;
    crate::registry::registered_abi_target(&manifest, abi).map(Some)
}

pub(super) fn target_config_tokens_for_domain(
    domain: &str,
    target: &RegisteredAbiTarget,
) -> TargetConfigTokens {
    let (arch, runtime, lane_width) = match domain {
        "kernel" => {
            let (arch, runtime) = kernel_target_tokens(target);
            (arch, runtime, "1".to_owned())
        }
        "shader" => {
            let runtime = target
                .backend_family
                .clone()
                .unwrap_or_else(|| "render".to_owned());
            (target.machine_arch.clone(), runtime, "1".to_owned())
        }
        "network" => {
            let runtime = match target.machine_os.as_str() {
                "darwin" => "urlsession",
                "windows" => "winsock",
                _ => "socket",
            }
            .to_owned();
            (target.machine_arch.clone(), runtime, "1".to_owned())
        }
        _ => (
            target.machine_arch.clone(),
            target.calling_abi.clone(),
            "1".to_owned(),
        ),
    };
    TargetConfigTokens {
        arch,
        runtime,
        lane_width,
        backend_features: backend_features_for_registered_abi_target(domain, target).join(","),
    }
}

fn apply_shader_profile_stmt(
    ast: &AstModule,
    stmt: &AstStmt,
    module: &mut YirModule,
    int_bindings: &BTreeMap<String, i64>,
) -> Result<(), String> {
    if let Some((node_name, value)) = extract_profile_int_binding(stmt) {
        let name = format!(
            "project_profile_{}_{}_{}",
            sanitize_ident(&ast.domain),
            sanitize_ident(&ast.unit),
            sanitize_ident(node_name)
        );
        push_profile_node(
            module,
            name,
            "cpu0",
            Operation {
                module: "cpu".to_owned(),
                instruction: "const_i64".to_owned(),
                args: vec![value.to_string()],
            },
        );
        return Ok(());
    }

    let Some((node_name, callee, args)) = extract_profile_call(stmt) else {
        return Ok(());
    };
    let name = format!(
        "project_profile_{}_{}_{}",
        sanitize_ident(&ast.domain),
        sanitize_ident(&ast.unit),
        sanitize_ident(node_name)
    );
    let op = match callee {
        "shader_target" => Operation {
            module: "shader".to_owned(),
            instruction: "target".to_owned(),
            args: vec![
                expect_text_arg(args, 0, "shader_target")?,
                expect_profile_int_arg(args, 1, "shader_target", int_bindings)?.to_string(),
                expect_profile_int_arg(args, 2, "shader_target", int_bindings)?.to_string(),
            ],
        },
        "shader_viewport" => Operation {
            module: "shader".to_owned(),
            instruction: "viewport".to_owned(),
            args: vec![
                expect_profile_int_arg(args, 0, "shader_viewport", int_bindings)?.to_string(),
                expect_profile_int_arg(args, 1, "shader_viewport", int_bindings)?.to_string(),
            ],
        },
        "shader_pipeline" => Operation {
            module: "shader".to_owned(),
            instruction: "pipeline".to_owned(),
            args: vec![
                expect_text_arg(args, 0, "shader_pipeline")?,
                expect_text_arg(args, 1, "shader_pipeline")?,
            ],
        },
        "shader_inline_wgsl" => Operation {
            module: "shader".to_owned(),
            instruction: "inline_wgsl".to_owned(),
            args: vec![
                expect_text_arg(args, 0, "shader_inline_wgsl")?,
                crate::shader_source::normalize_inline_wgsl_source(&expect_text_arg(
                    args,
                    1,
                    "shader_inline_wgsl",
                )?)?,
            ],
        },
        _ => return Ok(()),
    };
    push_profile_node(module, name, "shader0", op);
    Ok(())
}

fn apply_data_profile_stmt(
    ast: &AstModule,
    stmt: &AstStmt,
    module: &mut YirModule,
    int_bindings: &BTreeMap<String, i64>,
) -> Result<(), String> {
    if let Some((node_name, value)) = extract_profile_int_binding(stmt) {
        let name = format!(
            "project_profile_{}_{}_{}",
            sanitize_ident(&ast.domain),
            sanitize_ident(&ast.unit),
            sanitize_ident(node_name)
        );
        push_profile_node(
            module,
            name,
            "cpu0",
            Operation {
                module: "cpu".to_owned(),
                instruction: "const_i64".to_owned(),
                args: vec![value.to_string()],
            },
        );
        return Ok(());
    }

    let Some((node_name, callee, args)) = extract_profile_call(stmt) else {
        return Ok(());
    };
    let name = format!(
        "project_profile_{}_{}_{}",
        sanitize_ident(&ast.domain),
        sanitize_ident(&ast.unit),
        sanitize_ident(node_name)
    );
    let op = match callee {
        "data_bind_core" => Operation {
            module: "data".to_owned(),
            instruction: "bind_core".to_owned(),
            args: vec![
                expect_profile_int_arg(args, 0, "data_bind_core", int_bindings)?.to_string(),
            ],
        },
        "data_handle_table" => Operation {
            module: "data".to_owned(),
            instruction: "handle_table".to_owned(),
            args: args
                .iter()
                .enumerate()
                .map(|(index, _)| expect_text_arg(args, index, "data_handle_table"))
                .collect::<Result<Vec<_>, _>>()?,
        },
        "data_marker" => Operation {
            module: "data".to_owned(),
            instruction: "marker".to_owned(),
            args: vec![expect_text_arg(args, 0, "data_marker")?],
        },
        "data_copy_window" => Operation {
            module: "data".to_owned(),
            instruction: "copy_window".to_owned(),
            args: vec![
                expect_profile_value_input_name(ast, args, 0, "data_copy_window")?,
                expect_profile_int_arg(args, 1, "data_copy_window", int_bindings)?.to_string(),
                expect_profile_int_arg(args, 2, "data_copy_window", int_bindings)?.to_string(),
            ],
        },
        "data_immutable_window" => Operation {
            module: "data".to_owned(),
            instruction: "immutable_window".to_owned(),
            args: vec![
                expect_profile_value_input_name(ast, args, 0, "data_immutable_window")?,
                expect_profile_int_arg(args, 1, "data_immutable_window", int_bindings)?.to_string(),
                expect_profile_int_arg(args, 2, "data_immutable_window", int_bindings)?.to_string(),
            ],
        },
        _ => return Ok(()),
    };
    push_profile_node(module, name, "fabric0", op);
    Ok(())
}

fn apply_kernel_profile_stmt(
    ast: &AstModule,
    stmt: &AstStmt,
    module: &mut YirModule,
    int_bindings: &BTreeMap<String, i64>,
) -> Result<(), String> {
    if let Some((node_name, value)) = extract_profile_int_binding(stmt) {
        let name = format!(
            "project_profile_{}_{}_{}",
            sanitize_ident(&ast.domain),
            sanitize_ident(&ast.unit),
            sanitize_ident(node_name)
        );
        push_profile_node(
            module,
            name,
            "cpu0",
            Operation {
                module: "cpu".to_owned(),
                instruction: "const_i64".to_owned(),
                args: vec![value.to_string()],
            },
        );
        return Ok(());
    }

    let Some((node_name, callee, args)) = extract_profile_call(stmt) else {
        return Ok(());
    };
    let name = format!(
        "project_profile_{}_{}_{}",
        sanitize_ident(&ast.domain),
        sanitize_ident(&ast.unit),
        sanitize_ident(node_name)
    );
    let op = match callee {
        "kernel_target_config" => Operation {
            module: "kernel".to_owned(),
            instruction: "target_config".to_owned(),
            args: vec![
                expect_text_arg(args, 0, "kernel_target_config")?,
                expect_text_arg(args, 1, "kernel_target_config")?,
                expect_profile_int_arg(args, 2, "kernel_target_config", int_bindings)?.to_string(),
            ],
        },
        _ => return Ok(()),
    };
    push_profile_node(module, name, "kernel0", op);
    Ok(())
}

fn apply_network_profile_stmt(
    ast: &AstModule,
    stmt: &AstStmt,
    module: &mut YirModule,
) -> Result<(), String> {
    if let Some((node_name, value)) = extract_profile_int_binding(stmt) {
        let name = format!(
            "project_profile_{}_{}_{}",
            sanitize_ident(&ast.domain),
            sanitize_ident(&ast.unit),
            sanitize_ident(node_name)
        );
        push_profile_node(
            module,
            name,
            "network0",
            Operation {
                module: "network".to_owned(),
                instruction: "const_i64".to_owned(),
                args: vec![value.to_string()],
            },
        );
    }
    Ok(())
}
