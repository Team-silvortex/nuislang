use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use nuis_semantics::model::{AstExternFunction, AstTypeRef};

use super::{
    profile_apply::resolve_registered_abi_target, resolve_project_abi,
    selected_lowering_target_for_registered_abi_target, LoadedProject, ProjectAbiRequirement,
    ProjectAbiResolution, ProjectAbiSelectionView, ProjectExchangeOrganization,
    ProjectExchangeRoute, ProjectImportsSummary, ProjectLoweringIssue, ProjectLoweringIssueKind,
    ProjectLoweringSelectionView, ProjectModuleOrigin, ProjectOrganization,
    ProjectOrganizationLink, ProjectOrganizationModule,
};

fn json_escape(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

#[path = "rendering_abi.rs"]
mod rendering_abi;
#[path = "rendering_abi_index.rs"]
mod rendering_abi_index;
#[path = "rendering_host_ffi.rs"]
mod rendering_host_ffi;
#[path = "rendering_imports.rs"]
mod rendering_imports;
#[path = "rendering_lowering.rs"]
mod rendering_lowering;
#[path = "rendering_organization.rs"]
mod rendering_organization;

pub(in crate::project::rendering) use rendering_abi::selected_lowering_target_for_domain;

pub use rendering_abi::{
    project_abi_selection_view_json, project_abi_selection_views,
    render_project_abi_selection_lines, render_project_abi_selection_view_lines,
    write_project_abi_selection_lines, write_project_abi_selection_view_lines,
};
pub use rendering_abi_index::{describe_project_abi_graph, render_project_abi_graph_line};
pub(super) use rendering_abi_index::{render_project_abi_index, write_project_abi_index};
pub(super) use rendering_host_ffi::{render_project_host_ffi_index, write_project_host_ffi_index};
pub use rendering_imports::{
    project_imports_summary, render_project_import_index, write_project_import_index,
};
pub use rendering_lowering::{
    ensure_project_lowering_selections_valid, project_lowering_selection_json,
    render_project_lowering_selection_lines, validate_project_lowering_selections,
    write_project_lowering_selection_lines,
};
pub use rendering_organization::{organize_project, organize_project_exchanges};
pub(super) use rendering_organization::{
    render_project_exchange_index, render_project_organization_index, write_project_exchange_index,
    write_project_organization_index,
};

fn write_joined<W, T, F>(out: &mut W, items: &[T], sep: &str, mut write_item: F) -> fmt::Result
where
    W: fmt::Write,
    F: FnMut(&mut W, &T) -> fmt::Result,
{
    let mut first = true;
    for item in items {
        if !first {
            out.write_str(sep)?;
        }
        first = false;
        write_item(out, item)?;
    }
    Ok(())
}

fn write_project_abi_graph_line<W: fmt::Write>(
    out: &mut W,
    resolution: &ProjectAbiResolution,
) -> fmt::Result {
    let mut has_cpu = false;
    let mut has_data = false;
    let mut has_kernel = false;
    let mut has_shader = false;
    let mut has_network = false;

    write!(
        out,
        "graph\tmode={}\tdomains=",
        if resolution.explicit {
            "explicit"
        } else {
            "auto"
        }
    )?;
    write_joined(out, &resolution.requirements, ",", |out, item| {
        match item.domain.as_str() {
            "cpu" => has_cpu = true,
            "data" => has_data = true,
            "kernel" => has_kernel = true,
            "shader" => has_shader = true,
            "network" => has_network = true,
            _ => {}
        }
        write!(out, "{}", item.domain)
    })?;
    write!(
        out,
        "\tcpu_summary={}\tdata_summary={}\tkernel_target={}\tshader_target={}\tnetwork_target={}",
        if has_cpu { "present" } else { "absent" },
        if has_data { "present" } else { "absent" },
        if has_kernel { "present" } else { "absent" },
        if has_shader { "present" } else { "absent" },
        if has_network { "present" } else { "absent" },
    )
}

pub(super) fn render_ast_type_ref(ty: &AstTypeRef) -> String {
    let mut out = String::new();
    write_ast_type_ref(&mut out, ty).expect("writing ast type ref to String should not fail");
    out
}

fn write_host_ffi_signature<W: fmt::Write>(
    out: &mut W,
    function: &AstExternFunction,
) -> fmt::Result {
    write!(out, "fn {}(", function.name)?;
    write_joined(out, &function.params, ", ", |out, param| {
        write!(out, "{}: ", param.name)?;
        write_ast_type_ref(out, &param.ty)
    })?;
    write!(out, ") -> ")?;
    write_ast_type_ref(out, &function.return_type)
}

fn write_ast_type_ref<W: fmt::Write>(out: &mut W, ty: &AstTypeRef) -> fmt::Result {
    if ty.is_ref {
        write!(out, "ref ")?;
    }
    write!(out, "{}", ty.name)?;
    if !ty.generic_args.is_empty() {
        write!(out, "<")?;
        write_joined(out, &ty.generic_args, ", ", |out, arg| {
            write_ast_type_ref(out, arg)
        })?;
        write!(out, ">")?;
    }
    if ty.is_optional {
        write!(out, "?")?;
    }
    Ok(())
}
