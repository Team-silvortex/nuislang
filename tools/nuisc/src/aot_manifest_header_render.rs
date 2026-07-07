use std::{fmt::Write as _, path::Path};

use nuis_artifact::{NuisExecutableEnvelope, NuisLifecycleContract};

use crate::aot_manifest_types::{BuildManifestContext, BuildManifestDocIndexInfo};
use crate::aot_toml::{escape_toml_string, render_string_array};
use crate::aot_vcs_info::VcsInfo;

pub(crate) struct BuildManifestHeaderRenderInput<'a> {
    pub generated_at_unix: u64,
    pub packaging_mode: &'a str,
    pub engine_version: &'a str,
    pub engine_profile: &'a str,
    pub vcs: &'a VcsInfo,
    pub loaded_nustar: &'a [String],
    pub envelope_path: &'a Path,
    pub envelope: &'a NuisExecutableEnvelope,
    pub artifact_path: &'a Path,
    pub artifact_binary_name: &'a str,
    pub artifact_binary_bytes: usize,
    pub lifecycle: &'a NuisLifecycleContract,
}

pub(crate) fn append_manifest_header_sections(
    out: &mut String,
    context: &BuildManifestContext,
    input: BuildManifestHeaderRenderInput<'_>,
) {
    append_manifest_root(out, context, &input);
    append_envelope_section(out, input.envelope_path, input.envelope);
    append_artifact_section(
        out,
        input.artifact_path,
        input.artifact_binary_name,
        input.artifact_binary_bytes,
    );
    append_lifecycle_section(out, context, input.lifecycle);
}

fn append_manifest_root(
    out: &mut String,
    context: &BuildManifestContext,
    input: &BuildManifestHeaderRenderInput<'_>,
) {
    out.push_str("manifest_schema = \"nuis-build-manifest-v1\"\n");
    writeln!(out, "generated_at_unix = {}", input.generated_at_unix).unwrap();
    push_toml_string(out, "input", &context.input_path);
    push_toml_string(out, "output_dir", &context.output_dir);
    push_toml_string(out, "packaging_mode", input.packaging_mode);
    push_toml_string(out, "cpu_target_abi", &context.cpu_target.abi);
    push_toml_string(
        out,
        "cpu_target_machine_arch",
        &context.cpu_target.machine_arch,
    );
    push_toml_string(out, "cpu_target_machine_os", &context.cpu_target.machine_os);
    push_toml_string(
        out,
        "cpu_target_object_format",
        &context.cpu_target.object_format,
    );
    push_toml_string(
        out,
        "cpu_target_calling_abi",
        &context.cpu_target.calling_abi,
    );
    push_toml_string(out, "cpu_target_clang", &context.cpu_target.clang_target);
    writeln!(
        out,
        "cpu_target_cross = {}",
        context.cpu_target.cross_compile
    )
    .unwrap();
    push_toml_string(out, "tool_nuisc", env!("CARGO_PKG_VERSION"));
    push_toml_string(out, "engine_version", input.engine_version);
    push_toml_string(out, "engine_profile", input.engine_profile);
    match input.vcs {
        VcsInfo::Git { root, head, dirty } => {
            out.push_str("vcs = \"git\"\n");
            writeln!(out, "vcs_dirty = {dirty}").unwrap();
            push_toml_string(out, "vcs_head", head);
            push_toml_string(out, "vcs_root", root);
        }
        VcsInfo::None => {
            out.push_str("vcs = \"none\"\n");
        }
    }
    writeln!(
        out,
        "loaded_nustar = {}",
        render_string_array(input.loaded_nustar)
    )
    .unwrap();
}

fn append_envelope_section(out: &mut String, path: &Path, envelope: &NuisExecutableEnvelope) {
    out.push('\n');
    out.push_str("[nuis_envelope]\n");
    push_toml_string(out, "path", &path.display().to_string());
    push_toml_string(out, "schema", &envelope.schema);
    push_toml_string(out, "executable_kind", &envelope.executable_kind);
    writeln!(out, "package_count = {}", envelope.package_count).unwrap();
    writeln!(
        out,
        "domain_families = {}",
        render_string_array(&envelope.domain_families)
    )
    .unwrap();
    writeln!(
        out,
        "contract_families = {}",
        render_string_array(&envelope.contract_families)
    )
    .unwrap();
    push_toml_string(out, "function_kind", &envelope.function_kind);
    push_toml_string(out, "graph_kind", &envelope.graph_kind);
    push_toml_string(out, "default_time_mode", &envelope.default_time_mode);
}

fn append_artifact_section(
    out: &mut String,
    artifact_path: &Path,
    artifact_binary_name: &str,
    artifact_binary_bytes: usize,
) {
    out.push('\n');
    out.push_str("[nuis_artifact]\n");
    push_toml_string(out, "artifact_path", &artifact_path.display().to_string());
    push_toml_string(out, "artifact_schema", "nuis-compiled-artifact-v1");
    push_toml_string(out, "artifact_binary_name", artifact_binary_name);
    writeln!(out, "artifact_binary_bytes = {artifact_binary_bytes}").unwrap();
}

fn append_lifecycle_section(
    out: &mut String,
    context: &BuildManifestContext,
    lifecycle: &NuisLifecycleContract,
) {
    out.push('\n');
    out.push_str("[nuis_lifecycle]\n");
    push_toml_string(out, "lifecycle_schema", &lifecycle.schema);
    push_toml_string(out, "lifecycle_bootstrap_entry", &lifecycle.bootstrap_entry);
    push_toml_string(out, "lifecycle_tick_policy", &lifecycle.tick_policy);
    push_toml_string(out, "lifecycle_shutdown_policy", &lifecycle.shutdown_policy);
    push_toml_string(out, "lifecycle_yalivia_rpc", &lifecycle.yalivia_rpc);
    writeln!(
        out,
        "lifecycle_hook_surface = {}",
        render_string_array(&lifecycle.hook_surface)
    )
    .unwrap();
    writeln!(
        out,
        "lifecycle_export_surface = {}",
        render_string_array(&lifecycle.export_surface)
    )
    .unwrap();
    writeln!(
        out,
        "lifecycle_runtime_capability_flags = {}",
        render_string_array(&lifecycle.runtime_capability_flags)
    )
    .unwrap();
    append_cache_fields(out, context);
    append_doc_index_fields(out, context.doc_index.as_ref());
}

fn append_cache_fields(out: &mut String, context: &BuildManifestContext) {
    if let Some(cache) = &context.compile_cache {
        push_toml_string(out, "compile_cache_status", &cache.status);
        push_toml_string(out, "compile_cache_key", &cache.key);
        push_toml_string(out, "compile_cache_root", &cache.root);
    }
}

fn append_doc_index_fields(out: &mut String, doc_index: Option<&BuildManifestDocIndexInfo>) {
    if let Some(doc_index) = doc_index {
        push_toml_string(out, "doc_index_path", &doc_index.path);
        writeln!(out, "doc_index_module_count = {}", doc_index.module_count).unwrap();
        writeln!(
            out,
            "doc_index_documented_item_count = {}",
            doc_index.documented_item_count
        )
        .unwrap();
    }
}

fn push_toml_string(out: &mut String, key: &str, value: &str) {
    writeln!(out, "{key} = \"{}\"", escape_toml_string(value)).unwrap();
}
