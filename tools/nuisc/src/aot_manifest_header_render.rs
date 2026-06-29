use std::path::Path;

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
    out.push_str(&format!(
        "generated_at_unix = {}\n",
        input.generated_at_unix
    ));
    out.push_str(&format!(
        "input = \"{}\"\n",
        escape_toml_string(&context.input_path)
    ));
    out.push_str(&format!(
        "output_dir = \"{}\"\n",
        escape_toml_string(&context.output_dir)
    ));
    out.push_str(&format!(
        "packaging_mode = \"{}\"\n",
        escape_toml_string(input.packaging_mode)
    ));
    out.push_str(&format!(
        "cpu_target_abi = \"{}\"\n",
        escape_toml_string(&context.cpu_target.abi)
    ));
    out.push_str(&format!(
        "cpu_target_machine_arch = \"{}\"\n",
        escape_toml_string(&context.cpu_target.machine_arch)
    ));
    out.push_str(&format!(
        "cpu_target_machine_os = \"{}\"\n",
        escape_toml_string(&context.cpu_target.machine_os)
    ));
    out.push_str(&format!(
        "cpu_target_object_format = \"{}\"\n",
        escape_toml_string(&context.cpu_target.object_format)
    ));
    out.push_str(&format!(
        "cpu_target_calling_abi = \"{}\"\n",
        escape_toml_string(&context.cpu_target.calling_abi)
    ));
    out.push_str(&format!(
        "cpu_target_clang = \"{}\"\n",
        escape_toml_string(&context.cpu_target.clang_target)
    ));
    out.push_str(&format!(
        "cpu_target_cross = {}\n",
        if context.cpu_target.cross_compile {
            "true"
        } else {
            "false"
        }
    ));
    out.push_str(&format!(
        "tool_nuisc = \"{}\"\n",
        escape_toml_string(env!("CARGO_PKG_VERSION"))
    ));
    out.push_str(&format!(
        "engine_version = \"{}\"\n",
        escape_toml_string(input.engine_version)
    ));
    out.push_str(&format!(
        "engine_profile = \"{}\"\n",
        escape_toml_string(input.engine_profile)
    ));
    match input.vcs {
        VcsInfo::Git { root, head, dirty } => {
            out.push_str("vcs = \"git\"\n");
            out.push_str(&format!(
                "vcs_dirty = {}\n",
                if *dirty { "true" } else { "false" }
            ));
            out.push_str(&format!("vcs_head = \"{}\"\n", escape_toml_string(head)));
            out.push_str(&format!("vcs_root = \"{}\"\n", escape_toml_string(root)));
        }
        VcsInfo::None => {
            out.push_str("vcs = \"none\"\n");
        }
    }
    out.push_str(&format!(
        "loaded_nustar = {}\n",
        render_string_array(input.loaded_nustar)
    ));
}

fn append_envelope_section(out: &mut String, path: &Path, envelope: &NuisExecutableEnvelope) {
    out.push('\n');
    out.push_str("[nuis_envelope]\n");
    out.push_str(&format!(
        "path = \"{}\"\n",
        escape_toml_string(&path.display().to_string())
    ));
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(&envelope.schema)
    ));
    out.push_str(&format!(
        "executable_kind = \"{}\"\n",
        escape_toml_string(&envelope.executable_kind)
    ));
    out.push_str(&format!("package_count = {}\n", envelope.package_count));
    out.push_str(&format!(
        "domain_families = {}\n",
        render_string_array(&envelope.domain_families)
    ));
    out.push_str(&format!(
        "contract_families = {}\n",
        render_string_array(&envelope.contract_families)
    ));
    out.push_str(&format!(
        "function_kind = \"{}\"\n",
        escape_toml_string(&envelope.function_kind)
    ));
    out.push_str(&format!(
        "graph_kind = \"{}\"\n",
        escape_toml_string(&envelope.graph_kind)
    ));
    out.push_str(&format!(
        "default_time_mode = \"{}\"\n",
        escape_toml_string(&envelope.default_time_mode)
    ));
}

fn append_artifact_section(
    out: &mut String,
    artifact_path: &Path,
    artifact_binary_name: &str,
    artifact_binary_bytes: usize,
) {
    out.push('\n');
    out.push_str("[nuis_artifact]\n");
    out.push_str(&format!(
        "artifact_path = \"{}\"\n",
        escape_toml_string(&artifact_path.display().to_string())
    ));
    out.push_str(&format!(
        "artifact_schema = \"{}\"\n",
        escape_toml_string("nuis-compiled-artifact-v1")
    ));
    out.push_str(&format!(
        "artifact_binary_name = \"{}\"\n",
        escape_toml_string(artifact_binary_name)
    ));
    out.push_str(&format!(
        "artifact_binary_bytes = {artifact_binary_bytes}\n"
    ));
}

fn append_lifecycle_section(
    out: &mut String,
    context: &BuildManifestContext,
    lifecycle: &NuisLifecycleContract,
) {
    out.push('\n');
    out.push_str("[nuis_lifecycle]\n");
    out.push_str(&format!(
        "lifecycle_schema = \"{}\"\n",
        escape_toml_string(&lifecycle.schema)
    ));
    out.push_str(&format!(
        "lifecycle_bootstrap_entry = \"{}\"\n",
        escape_toml_string(&lifecycle.bootstrap_entry)
    ));
    out.push_str(&format!(
        "lifecycle_tick_policy = \"{}\"\n",
        escape_toml_string(&lifecycle.tick_policy)
    ));
    out.push_str(&format!(
        "lifecycle_shutdown_policy = \"{}\"\n",
        escape_toml_string(&lifecycle.shutdown_policy)
    ));
    out.push_str(&format!(
        "lifecycle_yalivia_rpc = \"{}\"\n",
        escape_toml_string(&lifecycle.yalivia_rpc)
    ));
    out.push_str(&format!(
        "lifecycle_hook_surface = {}\n",
        render_string_array(&lifecycle.hook_surface)
    ));
    out.push_str(&format!(
        "lifecycle_export_surface = {}\n",
        render_string_array(&lifecycle.export_surface)
    ));
    out.push_str(&format!(
        "lifecycle_runtime_capability_flags = {}\n",
        render_string_array(&lifecycle.runtime_capability_flags)
    ));
    append_cache_fields(out, context);
    append_doc_index_fields(out, context.doc_index.as_ref());
}

fn append_cache_fields(out: &mut String, context: &BuildManifestContext) {
    if let Some(cache) = &context.compile_cache {
        out.push_str(&format!(
            "compile_cache_status = \"{}\"\n",
            escape_toml_string(&cache.status)
        ));
        out.push_str(&format!(
            "compile_cache_key = \"{}\"\n",
            escape_toml_string(&cache.key)
        ));
        out.push_str(&format!(
            "compile_cache_root = \"{}\"\n",
            escape_toml_string(&cache.root)
        ));
    }
}

fn append_doc_index_fields(out: &mut String, doc_index: Option<&BuildManifestDocIndexInfo>) {
    if let Some(doc_index) = doc_index {
        out.push_str(&format!(
            "doc_index_path = \"{}\"\n",
            escape_toml_string(&doc_index.path)
        ));
        out.push_str(&format!(
            "doc_index_module_count = {}\n",
            doc_index.module_count
        ));
        out.push_str(&format!(
            "doc_index_documented_item_count = {}\n",
            doc_index.documented_item_count
        ));
    }
}
