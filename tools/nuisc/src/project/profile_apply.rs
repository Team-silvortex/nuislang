use std::collections::BTreeMap;

use crate::registry::RegisteredAbiTarget;
use nuis_semantics::model::{AstExpr, AstModule, AstStmt};
use yir_core::{Node, Operation, Resource, ResourceKind, YirModule};

use super::{sanitize_ident, ProjectAbiResolution};

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
) -> (String, String, String) {
    match domain {
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
    }
}

fn materialize_default_kernel_target_config(
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
    let (arch, runtime, lane_width) = target_config_tokens_for_domain("kernel", &target);
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
            args: vec![arch, runtime, lane_width],
        },
    );
    Ok(())
}

fn materialize_default_shader_target_config(
    ast: &AstModule,
    module: &mut YirModule,
    abi_resolution: Option<&ProjectAbiResolution>,
) -> Result<(), String> {
    let Some(target) = resolve_registered_abi_target("shader", abi_resolution)? else {
        return Ok(());
    };
    let (arch, runtime, lane_width) = target_config_tokens_for_domain("shader", &target);
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
            args: vec![arch, runtime, lane_width],
        },
    );
    Ok(())
}

fn materialize_default_network_target_config(
    ast: &AstModule,
    module: &mut YirModule,
    abi_resolution: Option<&ProjectAbiResolution>,
) -> Result<(), String> {
    let Some(target) = resolve_registered_abi_target("network", abi_resolution)? else {
        return Ok(());
    };
    let (arch, runtime, lane_width) = target_config_tokens_for_domain("network", &target);
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
            args: vec![arch, runtime, lane_width],
        },
    );
    Ok(())
}

fn kernel_target_tokens(target: &RegisteredAbiTarget) -> (String, String) {
    match target.backend_family.as_deref() {
        Some("coreml") => ("apple_ane".to_owned(), "coreml".to_owned()),
        Some("cpu-fallback") => (target.machine_arch.clone(), "cpu-fallback".to_owned()),
        Some(other) => (target.machine_arch.clone(), other.to_owned()),
        None => (target.machine_arch.clone(), target.calling_abi.clone()),
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

pub(super) fn collect_profile_int_bindings(body: &[AstStmt]) -> BTreeMap<String, i64> {
    let mut bindings = BTreeMap::new();
    for stmt in body {
        if let Some((name, value)) = extract_profile_int_binding(stmt) {
            bindings.insert(name.to_owned(), value);
        }
    }
    bindings
}

pub(super) fn extract_profile_call(stmt: &AstStmt) -> Option<(&str, &str, &[AstExpr])> {
    match stmt {
        AstStmt::Let { name, value, .. } | AstStmt::Const { name, value, .. } => {
            if let AstExpr::Call {
                callee,
                generic_args: _,
                args,
            } = value
            {
                Some((name.as_str(), callee.as_str(), args.as_slice()))
            } else {
                None
            }
        }
        AstStmt::Expr(AstExpr::Call {
            callee,
            generic_args: _,
            args,
        }) => Some((callee.as_str(), callee.as_str(), args.as_slice())),
        _ => None,
    }
}

fn extract_profile_int_binding(stmt: &AstStmt) -> Option<(&str, i64)> {
    match stmt {
        AstStmt::Let { name, value, .. } | AstStmt::Const { name, value, .. } => {
            if let AstExpr::Int(value) = value {
                Some((name.as_str(), *value))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn expect_text_arg(args: &[AstExpr], index: usize, callee: &str) -> Result<String, String> {
    match args.get(index) {
        Some(AstExpr::Text(value)) => Ok(value.clone()),
        _ => Err(format!(
            "{callee}(...) expects string literal arg {}",
            index + 1
        )),
    }
}

fn expect_profile_int_arg(
    args: &[AstExpr],
    index: usize,
    callee: &str,
    int_bindings: &BTreeMap<String, i64>,
) -> Result<i64, String> {
    match args.get(index) {
        Some(AstExpr::Int(value)) => Ok(*value),
        Some(AstExpr::Var(name)) => int_bindings.get(name).copied().ok_or_else(|| {
            format!(
                "{callee}(...) expects integer literal or profile const arg {}, unknown `{}`",
                index + 1,
                name
            )
        }),
        _ => Err(format!(
            "{callee}(...) expects integer literal or profile const arg {}",
            index + 1
        )),
    }
}

fn expect_profile_value_input_name(
    ast: &AstModule,
    args: &[AstExpr],
    index: usize,
    callee: &str,
) -> Result<String, String> {
    match args.get(index) {
        Some(AstExpr::Var(name)) => Ok(format!(
            "project_profile_{}_{}_{}",
            sanitize_ident(&ast.domain),
            sanitize_ident(&ast.unit),
            sanitize_ident(name)
        )),
        _ => Err(format!(
            "{callee}(...) expects profile value reference arg {}",
            index + 1
        )),
    }
}

pub(super) fn ensure_project_resource(module: &mut YirModule, name: &str, kind: &str) {
    if let Some(resource) = module
        .resources
        .iter_mut()
        .find(|resource| resource.name == name)
    {
        resource.kind = ResourceKind::parse(kind);
        return;
    }
    module.resources.push(Resource {
        name: name.to_owned(),
        kind: ResourceKind::parse(kind),
    });
}

pub(super) fn push_profile_node(
    module: &mut YirModule,
    name: String,
    resource: &str,
    op: Operation,
) {
    if module.nodes.iter().any(|node| node.name == name) {
        return;
    }
    module.nodes.push(Node {
        name,
        resource: resource.to_owned(),
        op,
    });
}
