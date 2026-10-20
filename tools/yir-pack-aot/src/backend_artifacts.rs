use crate::backend_descriptor::{
    render_kernel_descriptor_lines, render_shader_descriptor_lines, render_summary_block,
};
use crate::shader_codegen::{
    render_shader_glsl_scaffold, render_shader_hlsl_scaffold, render_shader_metal_scaffold,
    render_shader_vulkan_glsl_scaffold, render_shader_wgsl_scaffold,
};
use std::fs;
use std::path::Path;

pub(super) fn emit_shader_backend_artifacts(
    output_dir: &Path,
    contract: &yir_lower_contract::ShaderLoweringContract,
) -> Result<(), String> {
    for stage in &contract.stages {
        cleanup_legacy_shader_backend_artifacts(output_dir, stage)?;
        let stage_summary = render_shader_stage_summary(stage);
        for variant in stage.backend_variants() {
            let artifact_path = output_dir.join(&variant.artifact);
            if let Some(parent) = artifact_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("failed to create `{}`: {error}", parent.display()))?;
            }
            let artifact_body = render_shader_artifact_stub(stage, &variant, &stage_summary);
            fs::write(&artifact_path, artifact_body).map_err(|error| {
                format!("failed to write `{}`: {error}", artifact_path.display())
            })?;
        }
    }
    Ok(())
}

pub(super) fn render_shader_backend_validation(
    output_dir: &Path,
    contract: &yir_lower_contract::ShaderLoweringContract,
) -> Result<String, String> {
    let mut lines = Vec::new();
    for stage in &contract.stages {
        let has_inline_wgsl = stage.wgsl_source.is_some();
        lines.push(format!(
            "stage={} inline_wgsl={}",
            stage.node, has_inline_wgsl
        ));
        for variant in stage.backend_variants() {
            let artifact_path = output_dir.join(&variant.artifact);
            let artifact = fs::read_to_string(&artifact_path).map_err(|error| {
                format!("failed to read `{}`: {error}", artifact_path.display())
            })?;
            let checks = shader_backend_checks(stage, variant.backend, &artifact);
            let status = if checks.iter().all(|(_, ok)| *ok) {
                "ok"
            } else {
                "fail"
            };
            let check_summary = checks
                .iter()
                .map(|(name, ok)| format!("{name}={ok}"))
                .collect::<Vec<_>>()
                .join(" ");
            lines.push(format!(
                "  backend={} family={} target_os={} target_device={} ir_format={} dispatch_abi={} kind={} priority={} declared_status={} verification={} validation_status={} {}",
                variant.backend,
                variant.backend_family,
                variant.target_os,
                variant.target_device,
                variant.ir_format,
                variant.dispatch_abi,
                variant.kind,
                variant.priority,
                variant.status,
                variant.verification,
                status,
                check_summary
            ));
        }
    }
    Ok(lines.join("\n") + "\n")
}

pub(super) fn emit_kernel_backend_artifacts(
    output_dir: &Path,
    contract: &yir_lower_contract::KernelLoweringContract,
) -> Result<(), String> {
    for graph in &contract.graphs {
        let graph_summary = render_kernel_graph_summary(graph);
        for variant in graph.backend_variants() {
            let artifact_path = output_dir.join(&variant.artifact);
            if let Some(parent) = artifact_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("failed to create `{}`: {error}", parent.display()))?;
            }
            let artifact_body = render_kernel_graph_artifact_stub(graph, &variant, &graph_summary);
            fs::write(&artifact_path, artifact_body).map_err(|error| {
                format!("failed to write `{}`: {error}", artifact_path.display())
            })?;
        }
    }

    for stage in &contract.stages {
        let stage_summary = render_kernel_stage_summary(stage);
        for variant in stage.backend_variants() {
            let artifact_path = output_dir.join(&variant.artifact);
            if let Some(parent) = artifact_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("failed to create `{}`: {error}", parent.display()))?;
            }
            let artifact_body = render_kernel_stage_artifact_stub(stage, &variant, &stage_summary);
            fs::write(&artifact_path, artifact_body).map_err(|error| {
                format!("failed to write `{}`: {error}", artifact_path.display())
            })?;
        }
    }
    Ok(())
}

pub(super) fn render_shader_stage_summary(
    stage: &yir_lower_contract::ShaderStageContract,
) -> String {
    let mut lines = vec![
        format!("stage={}", stage.node),
        format!("op={}", stage.op),
        format!("resource={}", stage.resource),
        format!("lowering={}", stage.lowering.as_str()),
        format!("reason={}", stage.reason),
    ];
    if let Some(pipeline) = &stage.pipeline {
        lines.push(format!("pipeline={pipeline}"));
    }
    if let Some(target_format) = &stage.target_format {
        lines.push(format!("target_format={target_format}"));
    }
    if let Some(topology) = &stage.topology {
        lines.push(format!("topology={topology}"));
    }
    if let Some(entry) = &stage.wgsl_entry {
        lines.push(format!("wgsl_entry={entry}"));
    }
    if let Some(source) = &stage.wgsl_source {
        lines.push(format!("wgsl_source_lines={}", source.lines().count()));
    }
    for binding in &stage.bindings {
        lines.push(format!(
            "binding.slot={} kind={} source={}",
            binding.slot, binding.kind, binding.source
        ));
    }
    lines.join("\n") + "\n"
}

pub(super) fn render_shader_artifact_stub(
    stage: &yir_lower_contract::ShaderStageContract,
    variant: &yir_lower_contract::ShaderBackendVariant,
    summary: &str,
) -> String {
    match variant.backend {
        "opengl" => render_shader_glsl_scaffold(stage, variant, summary),
        "metal" => render_shader_metal_scaffold(stage, variant, summary),
        "directx" => render_shader_hlsl_scaffold(stage, variant, summary),
        "vulkan" => render_shader_vulkan_glsl_scaffold(stage, variant, summary),
        "webgpu" => render_shader_wgsl_scaffold(stage, variant, summary),
        _ => render_shader_text_scaffold(variant, summary),
    }
}

fn cleanup_legacy_shader_backend_artifacts(
    output_dir: &Path,
    stage: &yir_lower_contract::ShaderStageContract,
) -> Result<(), String> {
    let legacy_paths = [
        output_dir.join(format!("metal/{}.metallib", stage.node)),
        output_dir.join(format!("directx/{}.dxil", stage.node)),
        output_dir.join(format!("vulkan/{}.spv", stage.node)),
    ];
    for path in legacy_paths {
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|error| format!("failed to remove `{}`: {error}", path.display()))?;
        }
    }
    Ok(())
}

pub(super) fn render_kernel_graph_summary(
    graph: &yir_lower_contract::KernelComputeGraphContract,
) -> String {
    let mut lines = vec![
        format!("graph={}", graph.id),
        format!("resource={}", graph.resource),
        format!("lowering={}", graph.lowering.as_str()),
        format!("reason={}", graph.reason),
        format!("stage_count={}", graph.stages.len()),
    ];
    if let Some(runtime) = &graph.target_runtime {
        lines.push(format!("target_runtime={runtime}"));
    }
    if let Some(arch) = &graph.target_arch {
        lines.push(format!("target_arch={arch}"));
    }
    if let Some(width) = graph.lane_width {
        lines.push(format!("lane_width={width}"));
    }
    for stage in &graph.stages {
        lines.push(format!("stage={stage}"));
    }
    lines.join("\n") + "\n"
}

pub(super) fn render_kernel_graph_artifact_stub(
    _graph: &yir_lower_contract::KernelComputeGraphContract,
    variant: &yir_lower_contract::KernelBackendVariant,
    summary: &str,
) -> String {
    match variant.kind {
        "graph" => render_kernel_json_scaffold("kernel_graph", variant, summary),
        "mlpackage" => render_kernel_manifest_scaffold("kernel_graph", variant, summary),
        _ => render_kernel_text_scaffold("kernel graph", variant, summary),
    }
}

pub(super) fn render_kernel_stage_summary(
    stage: &yir_lower_contract::KernelStageContract,
) -> String {
    let mut lines = vec![
        format!("stage={}", stage.node),
        format!("op={}", stage.op),
        format!("resource={}", stage.resource),
        format!("lowering={}", stage.lowering.as_str()),
        format!("reason={}", stage.reason),
    ];
    if let Some(runtime) = &stage.target_runtime {
        lines.push(format!("target_runtime={runtime}"));
    }
    if let Some(arch) = &stage.target_arch {
        lines.push(format!("target_arch={arch}"));
    }
    if let Some(width) = stage.lane_width {
        lines.push(format!("lane_width={width}"));
    }
    if let Some(rows) = stage.rows {
        lines.push(format!("rows={rows}"));
    }
    if let Some(cols) = stage.cols {
        lines.push(format!("cols={cols}"));
    }
    if let Some(axis) = &stage.axis {
        lines.push(format!("axis={axis}"));
    }
    if let Some(topk) = stage.topk {
        lines.push(format!("topk={topk}"));
    }
    for input in &stage.inputs {
        lines.push(format!("input={input}"));
    }
    lines.join("\n") + "\n"
}

pub(super) fn render_kernel_stage_artifact_stub(
    _stage: &yir_lower_contract::KernelStageContract,
    variant: &yir_lower_contract::KernelBackendVariant,
    summary: &str,
) -> String {
    match variant.kind {
        "graph" => render_kernel_json_scaffold("kernel_stage", variant, summary),
        "mlmodel" => render_kernel_manifest_scaffold("kernel_stage", variant, summary),
        _ => render_kernel_text_scaffold("kernel stage", variant, summary),
    }
}

fn render_shader_text_scaffold(
    variant: &yir_lower_contract::ShaderBackendVariant,
    summary: &str,
) -> String {
    format!(
        "# nuis shader backend scaffold\n{}{}",
        render_shader_descriptor_lines(variant, ""),
        render_summary_block(summary, "")
    )
}

fn render_kernel_text_scaffold(
    subject: &str,
    variant: &yir_lower_contract::KernelBackendVariant,
    summary: &str,
) -> String {
    format!(
        "# nuis {subject} backend scaffold\n{}{}",
        render_kernel_descriptor_lines(variant, ""),
        render_summary_block(summary, "")
    )
}

pub(super) fn shader_backend_checks(
    stage: &yir_lower_contract::ShaderStageContract,
    backend: &str,
    artifact: &str,
) -> Vec<(&'static str, bool)> {
    let expects_wgsl = stage.wgsl_source.is_some();
    match backend {
        "opengl" => vec![
            (
                "vertex_section",
                artifact.contains("#ifdef NUIS_STAGE_VERTEX"),
            ),
            (
                "fragment_section",
                artifact.contains("#ifdef NUIS_STAGE_FRAGMENT"),
            ),
            (
                "wgsl_origin",
                !expects_wgsl || artifact.contains("original_wgsl_begin"),
            ),
        ],
        "vulkan" => vec![
            ("version_450", artifact.contains("#version 450")),
            ("vertex_index", artifact.contains("gl_VertexIndex")),
            (
                "wgsl_origin",
                !expects_wgsl || artifact.contains("original_wgsl_begin"),
            ),
        ],
        "metal" => vec![
            (
                "metal_include",
                artifact.contains("#include <metal_stdlib>"),
            ),
            ("vertex_fn", artifact.contains("vertex VsOut")),
            (
                "wgsl_origin",
                !expects_wgsl || artifact.contains("original_wgsl_begin"),
            ),
        ],
        "directx" => vec![
            ("sv_position", artifact.contains("SV_Position")),
            ("pixel_fn", artifact.contains(": SV_Target0")),
            (
                "wgsl_origin",
                !expects_wgsl || artifact.contains("original_wgsl_begin"),
            ),
        ],
        "webgpu" => vec![
            ("wgsl_vertex", artifact.contains("@vertex")),
            ("wgsl_fragment", artifact.contains("@fragment")),
            (
                "wgsl_origin",
                !expects_wgsl || artifact.contains("original_wgsl_begin"),
            ),
        ],
        _ => vec![("non_empty", !artifact.trim().is_empty())],
    }
}

fn render_kernel_json_scaffold(
    subject: &str,
    variant: &yir_lower_contract::KernelBackendVariant,
    summary: &str,
) -> String {
    let summary_json = summary
        .lines()
        .map(|line| {
            format!(
                "    \"{}\"",
                line.replace('\\', "\\\\").replace('"', "\\\"")
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        "{{\n  \"schema\": \"nuis-{subject}-backend-scaffold-v1\",\n  \"backend\": \"{backend}\",\n  \"backend_family\": \"{backend_family}\",\n  \"target_os\": \"{target_os}\",\n  \"target_device\": \"{target_device}\",\n  \"ir_format\": \"{ir_format}\",\n  \"dispatch_abi\": \"{dispatch_abi}\",\n  \"kind\": \"{kind}\",\n  \"priority\": {priority},\n  \"status\": \"{status}\",\n  \"verification\": \"{verification}\",\n  \"entry\": \"{entry}\",\n  \"artifact\": \"{artifact}\",\n  \"summary\": [\n{summary_json}\n  ]\n}}\n",
        subject = subject,
        backend = variant.backend,
        backend_family = variant.backend_family,
        target_os = variant.target_os,
        target_device = variant.target_device,
        ir_format = variant.ir_format,
        dispatch_abi = variant.dispatch_abi,
        kind = variant.kind,
        priority = variant.priority,
        status = variant.status,
        verification = variant.verification,
        entry = variant.entry,
        artifact = variant.artifact,
        summary_json = summary_json
    )
}

fn render_kernel_manifest_scaffold(
    subject: &str,
    variant: &yir_lower_contract::KernelBackendVariant,
    summary: &str,
) -> String {
    format!(
        "schema = \"nuis-{subject}-backend-scaffold-v1\"\nbackend = \"{backend}\"\nbackend_family = \"{backend_family}\"\ntarget_os = \"{target_os}\"\ntarget_device = \"{target_device}\"\nir_format = \"{ir_format}\"\ndispatch_abi = \"{dispatch_abi}\"\nkind = \"{kind}\"\npriority = {priority}\nstatus = \"{status}\"\nverification = \"{verification}\"\nentry = \"{entry}\"\nartifact = \"{artifact}\"\n\n[summary]\ntext = \"{summary_text}\"\n",
        subject = subject,
        backend = variant.backend,
        backend_family = variant.backend_family,
        target_os = variant.target_os,
        target_device = variant.target_device,
        ir_format = variant.ir_format,
        dispatch_abi = variant.dispatch_abi,
        kind = variant.kind,
        priority = variant.priority,
        status = variant.status,
        verification = variant.verification,
        entry = variant.entry,
        artifact = variant.artifact,
        summary_text = summary.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', " | ")
    )
}
