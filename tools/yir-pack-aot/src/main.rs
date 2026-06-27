use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
    process::{self, Command},
};

use yir_core::{
    ffi::{ffi_symbol_signature_hash, is_ffi_symbol_hash_token, FFI_SYMBOL_HASH_PREFIX},
    Value, YirModule,
};
use yir_exec::execute_module;
use yir_host_render::rasterize_frame;
use yir_lower_contract::{analyze_kernel_lowering, analyze_shader_lowering};
use yir_lower_llvm::emit_module;
use yir_verify::verify_module;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let input = args.next().ok_or_else(|| {
        "usage: cargo run -p yir-pack-aot -- <module.yir> <output-dir> [frame-scale]".to_owned()
    })?;
    let output_dir = args.next().ok_or_else(|| {
        "usage: cargo run -p yir-pack-aot -- <module.yir> <output-dir> [frame-scale]".to_owned()
    })?;
    let frame_scale = args
        .next()
        .map(|raw| {
            raw.parse::<usize>()
                .map_err(|_| format!("invalid frame scale `{raw}`"))
        })
        .transpose()?
        .unwrap_or(8);

    let source =
        fs::read_to_string(&input).map_err(|error| format!("failed to read `{input}`: {error}"))?;
    let module = yir_syntax::parse_module(&source)?;
    verify_module(&module)?;
    validate_host_ffi_symbols(&module)?;
    let host_ffi_symbols = collect_host_ffi_symbols(&module)?;
    let host_ffi_stub_source = render_host_ffi_stubs(&host_ffi_symbols);

    let output_dir = PathBuf::from(output_dir);
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", output_dir.display()))?;

    let stem = Path::new(&input)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("yir_module");
    let ll_path = output_dir.join(format!("{stem}.ll"));
    let shim_path = output_dir.join(format!("{stem}_shim.c"));
    let host_path = output_dir.join(format!("{stem}_host.m"));
    let exe_path = output_dir.join(stem);
    let manifest_path = output_dir.join("bundle.txt");
    let shader_contract_path = output_dir.join("shader_contract.txt");
    let shader_package_path = output_dir.join("shader_package.toml");
    let shader_validation_path = output_dir.join("shader_backend_validation.txt");
    let kernel_contract_path = output_dir.join("kernel_contract.txt");
    let kernel_package_path = output_dir.join("kernel_package.toml");

    let llvm_ir = emit_module(&module)?;
    fs::write(&ll_path, llvm_ir)
        .map_err(|error| format!("failed to write `{}`: {error}", ll_path.display()))?;
    fs::write(&shim_path, c_shim_source(&host_ffi_symbols))
        .map_err(|error| format!("failed to write `{}`: {error}", shim_path.display()))?;

    let mut manifest = vec![
        format!("module={input}"),
        format!("llvm_ir={}", ll_path.display()),
    ];
    append_host_ffi_manifest_entries(&mut manifest, &host_ffi_symbols)?;

    let shader_contract = analyze_shader_lowering(&module);
    let kernel_contract = analyze_kernel_lowering(&module);
    let primary_fabric_binding = extract_primary_fabric_binding(&shader_contract);
    manifest.push(format!(
        "fabric_handle_tables={}",
        shader_contract.fabric_handle_tables.len()
    ));
    manifest.push(format!(
        "fabric_core_bindings={}",
        shader_contract.fabric_core_bindings.len()
    ));
    if let Some(binding) = &primary_fabric_binding {
        manifest.push(format!("fabric_handle_table_id={}", binding.table_id));
        manifest.push(format!("fabric_host_resource={}", binding.host_resource));
        manifest.push(format!(
            "fabric_render_resource={}",
            binding.render_resource
        ));
    }
    if let Some(core_binding) = shader_contract.fabric_core_bindings.first() {
        manifest.push(format!("fabric_worker_resource={}", core_binding.resource));
        manifest.push(format!("fabric_worker_core={}", core_binding.core_index));
        manifest.push("fabric_worker_core_mode=macos_affinity_worker_thread".to_owned());
    }
    if shader_contract.has_shader_work() {
        fs::write(&shader_contract_path, shader_contract.render_text()).map_err(|error| {
            format!(
                "failed to write `{}`: {error}",
                shader_contract_path.display()
            )
        })?;
        fs::write(
            &shader_package_path,
            shader_contract.render_package_manifest(),
        )
        .map_err(|error| {
            format!(
                "failed to write `{}`: {error}",
                shader_package_path.display()
            )
        })?;
        emit_shader_backend_artifacts(&output_dir, &shader_contract)?;
        let shader_validation = render_shader_backend_validation(&output_dir, &shader_contract)?;
        fs::write(&shader_validation_path, shader_validation).map_err(|error| {
            format!(
                "failed to write `{}`: {error}",
                shader_validation_path.display()
            )
        })?;
        manifest.push(format!(
            "shader_contract={}",
            shader_contract_path.display()
        ));
        manifest.push(format!("shader_package={}", shader_package_path.display()));
        manifest.push(format!(
            "shader_backend_validation={}",
            shader_validation_path.display()
        ));
        manifest.push(format!(
            "shader_backend_eligible={}",
            shader_contract.has_backend_eligible_work()
        ));
        manifest.push(format!(
            "shader_requires_prerender_fallback={}",
            shader_contract.requires_prerender_fallback()
        ));
    } else {
        manifest.push("shader_backend_eligible=false".to_owned());
        manifest.push("shader_requires_prerender_fallback=false".to_owned());
    }
    if kernel_contract.has_kernel_work() {
        fs::write(&kernel_contract_path, kernel_contract.render_text()).map_err(|error| {
            format!(
                "failed to write `{}`: {error}",
                kernel_contract_path.display()
            )
        })?;
        fs::write(
            &kernel_package_path,
            kernel_contract.render_package_manifest(),
        )
        .map_err(|error| {
            format!(
                "failed to write `{}`: {error}",
                kernel_package_path.display()
            )
        })?;
        emit_kernel_backend_artifacts(&output_dir, &kernel_contract)?;
        manifest.push(format!(
            "kernel_contract={}",
            kernel_contract_path.display()
        ));
        manifest.push(format!("kernel_package={}", kernel_package_path.display()));
        manifest.push(format!(
            "kernel_backend_eligible={}",
            kernel_contract.has_backend_eligible_work()
        ));
        manifest.push(format!(
            "kernel_requires_cpu_fallback={}",
            kernel_contract.requires_cpu_fallback()
        ));
    } else {
        manifest.push("kernel_backend_eligible=false".to_owned());
        manifest.push("kernel_requires_cpu_fallback=false".to_owned());
    }

    let runtime_frame_support =
        maybe_prepare_embedded_runtime_support(&module, &source, frame_scale)?;
    let shader_requires_prerender_fallback = shader_contract.requires_prerender_fallback();
    let frame_bundle = if runtime_frame_support.is_some() && !shader_requires_prerender_fallback {
        None
    } else {
        maybe_emit_prerendered_frame(&module, &output_dir, stem, frame_scale)?
    };
    if runtime_frame_support.is_some() && frame_bundle.is_none() {
        cleanup_stale_fallback_frame(&output_dir, stem)?;
    }
    let window_spec = extract_cpu_window_spec(
        &module,
        primary_fabric_binding
            .as_ref()
            .map(|binding| binding.host_resource.as_str()),
    );

    if runtime_frame_support.is_some() {
        manifest.push("render_mode=runtime_tick".to_owned());
        if let Some(frame_bundle) = &frame_bundle {
            manifest.push(format!(
                "fallback_frame_asset={}",
                frame_bundle.asset_path.display()
            ));
            manifest.push("fallback_frame_mode=prerendered".to_owned());
        }
    } else if let Some(frame_bundle) = &frame_bundle {
        manifest.push("render_mode=prerendered".to_owned());
        manifest.push(format!("frame_asset={}", frame_bundle.asset_path.display()));
    } else {
        manifest.push("render_mode=none".to_owned());
    }

    let use_window_host =
        runtime_frame_support.is_some() || frame_bundle.is_some() || window_spec.is_some();

    if use_window_host {
        let fabric_boot_plan = extract_fabric_boot_plan(
            &module,
            primary_fabric_binding.as_ref(),
            shader_contract
                .fabric_core_bindings
                .first()
                .map(|binding| binding.resource.as_str()),
        );
        fs::write(
            &host_path,
            objc_host_source(
                window_spec
                    .as_ref()
                    .map(|spec| spec.title.as_str())
                    .unwrap_or(stem),
                window_spec.as_ref().map(|spec| spec.width).unwrap_or(640),
                window_spec.as_ref().map(|spec| spec.height).unwrap_or(480),
                shader_contract
                    .fabric_core_bindings
                    .first()
                    .map(|binding| binding.core_index),
                primary_fabric_binding
                    .as_ref()
                    .map(|binding| binding.table_id.as_str()),
                primary_fabric_binding
                    .as_ref()
                    .map(|binding| binding.host_resource.as_str()),
                primary_fabric_binding
                    .as_ref()
                    .map(|binding| binding.render_resource.as_str()),
                &render_fabric_boot_plan(&fabric_boot_plan),
                fabric_boot_plan.len(),
                frame_bundle
                    .as_ref()
                    .map(|bundle| bundle.embedded_ppm_bytes.as_str()),
                runtime_frame_support.as_ref(),
                &host_ffi_stub_source,
            ),
        )
        .map_err(|error| format!("failed to write `{}`: {error}", host_path.display()))?;
        compile_native_appkit_binary(
            &ll_path,
            &host_path,
            runtime_frame_support
                .as_ref()
                .map(|support| support.staticlib_path.as_path()),
            &exe_path,
        )?;
        manifest.push(format!("binary={}", exe_path.display()));
        manifest.push("binary_mode=llvm_objc_appkit".to_owned());
        manifest.push(format!("host_stub={}", host_path.display()));
        if let Some(runtime_support) = &runtime_frame_support {
            manifest.push("runtime_bootstrap_mode=embedded_yir_tick".to_owned());
            manifest.push(format!(
                "runtime_host_staticlib={}",
                runtime_support.staticlib_path.display()
            ));
            manifest.push("single_binary=true".to_owned());
        } else {
            manifest.push("runtime_bootstrap_mode=embedded_prerendered_fallback".to_owned());
            manifest.push("single_binary=true".to_owned());
        }
        manifest.push(format!(
            "fabric_boot_plan_events={}",
            fabric_boot_plan.len()
        ));
        manifest.push("fabric_boot_plan_mode=static_typed_action_table".to_owned());
        if let Some(spec) = &window_spec {
            manifest.push(format!("window_title={}", spec.title));
            manifest.push(format!("window_width={}", spec.width));
            manifest.push(format!("window_height={}", spec.height));
        }
    } else {
        compile_native_binary(&ll_path, &shim_path, &exe_path)?;
        manifest.push(format!("binary={}", exe_path.display()));
        manifest.push("binary_mode=llvm_clang".to_owned());
        manifest.push(format!("host_stub={}", shim_path.display()));
    }

    fs::write(&manifest_path, manifest.join("\n") + "\n")
        .map_err(|error| format!("failed to write `{}`: {error}", manifest_path.display()))?;

    println!("packed AOT bundle into {}", output_dir.display());
    println!("binary: {}", exe_path.display());
    println!("manifest: {}", manifest_path.display());
    Ok(())
}

#[derive(Debug, Clone)]
struct CpuWindowSpec {
    title: String,
    width: usize,
    height: usize,
}

#[derive(Debug, Clone)]
struct PrimaryFabricBinding {
    table_id: String,
    host_resource: String,
    render_resource: String,
}

#[derive(Debug, Clone)]
struct FabricBootEvent {
    action_kind: String,
    action_class: String,
    action_slot: String,
    event_name: String,
    table_id: String,
    source: String,
    target: String,
}

fn extract_primary_fabric_binding(
    contract: &yir_lower_contract::ShaderLoweringContract,
) -> Option<PrimaryFabricBinding> {
    let stage = contract.stages.first()?;
    let table_id = stage.fabric_handle_table.as_ref()?;
    let table = contract
        .fabric_handle_tables
        .iter()
        .find(|table| &table.node == table_id)?;
    let host_resource = table
        .entries
        .iter()
        .find(|entry| entry.slot == "host")
        .map(|entry| entry.resource.clone())?;
    let render_resource = table
        .entries
        .iter()
        .find(|entry| entry.slot == "render")
        .map(|entry| entry.resource.clone())?;
    Some(PrimaryFabricBinding {
        table_id: table.node.clone(),
        host_resource,
        render_resource,
    })
}

fn extract_cpu_window_spec(
    module: &YirModule,
    host_resource: Option<&str>,
) -> Option<CpuWindowSpec> {
    module.nodes.iter().find_map(|node| {
        if node.op.module == "cpu"
            && node.op.instruction == "window"
            && node.op.args.len() == 3
            && host_resource.is_none_or(|resource| resource == node.resource)
        {
            let width = node.op.args[0].parse::<usize>().ok()?;
            let height = node.op.args[1].parse::<usize>().ok()?;
            Some(CpuWindowSpec {
                title: node.op.args[2].clone(),
                width,
                height,
            })
        } else {
            None
        }
    })
}

fn extract_fabric_boot_plan(
    module: &YirModule,
    primary_binding: Option<&PrimaryFabricBinding>,
    worker_resource: Option<&str>,
) -> Vec<FabricBootEvent> {
    let table_id = primary_binding
        .map(|binding| binding.table_id.as_str())
        .unwrap_or("none");
    let host_resource = primary_binding
        .map(|binding| binding.host_resource.as_str())
        .unwrap_or("none");
    let render_resource = primary_binding
        .map(|binding| binding.render_resource.as_str())
        .unwrap_or("none");
    let worker_resource = worker_resource.unwrap_or("none");

    module
        .nodes
        .iter()
        .filter(|node| node.op.module == "data")
        .map(|node| {
            let (action_kind, action_class, action_slot) = match node.op.instruction.as_str() {
                "bind_core" => ("NUIS_FABRIC_ACTION_BIND_CORE", "worker", "bind_core"),
                "handle_table" => ("NUIS_FABRIC_ACTION_HANDLE_TABLE", "binding", "handle_table"),
                "output_pipe" => ("NUIS_FABRIC_ACTION_OUTPUT_PIPE", "pipe", "output"),
                "input_pipe" => ("NUIS_FABRIC_ACTION_INPUT_PIPE", "pipe", "input"),
                "marker" => ("NUIS_FABRIC_ACTION_MARKER", "sync", "marker"),
                "copy_window" => ("NUIS_FABRIC_ACTION_COPY_WINDOW", "window", "copy"),
                "immutable_window" => {
                    ("NUIS_FABRIC_ACTION_IMMUTABLE_WINDOW", "window", "immutable")
                }
                "move" => ("NUIS_FABRIC_ACTION_MOVE_VALUE", "move", "value"),
                _ => ("NUIS_FABRIC_ACTION_UNKNOWN", "unknown", "unknown"),
            };
            let (source, target) = match node.op.instruction.as_str() {
                "bind_core" => (worker_resource.to_owned(), worker_resource.to_owned()),
                "handle_table" => (host_resource.to_owned(), render_resource.to_owned()),
                "output_pipe" => (host_resource.to_owned(), worker_resource.to_owned()),
                "input_pipe" => (worker_resource.to_owned(), render_resource.to_owned()),
                "marker" => (worker_resource.to_owned(), worker_resource.to_owned()),
                "copy_window" | "immutable_window" => {
                    (worker_resource.to_owned(), render_resource.to_owned())
                }
                "move" => (host_resource.to_owned(), render_resource.to_owned()),
                _ => (node.resource.clone(), node.resource.clone()),
            };

            FabricBootEvent {
                action_kind: action_kind.to_owned(),
                action_class: action_class.to_owned(),
                action_slot: action_slot.to_owned(),
                event_name: format!("data.{}:{}", node.op.instruction, node.name),
                table_id: table_id.to_owned(),
                source,
                target,
            }
        })
        .collect()
}

fn render_fabric_boot_plan(events: &[FabricBootEvent]) -> String {
    if events.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    for event in events {
        out.push_str("    {\n");
        out.push_str(&format!("        {},\n", event.action_kind));
        out.push_str(&format!(
            "        \"{}\",\n",
            c_string_literal(&event.action_class)
        ));
        out.push_str(&format!(
            "        \"{}\",\n",
            c_string_literal(&event.action_slot)
        ));
        out.push_str(&format!(
            "        \"{}\",\n",
            c_string_literal(&event.event_name)
        ));
        out.push_str(&format!(
            "        \"{}\",\n",
            c_string_literal(&event.table_id)
        ));
        out.push_str(&format!(
            "        \"{}\",\n",
            c_string_literal(&event.source)
        ));
        out.push_str(&format!(
            "        \"{}\",\n",
            c_string_literal(&event.target)
        ));
        out.push_str("    },\n");
    }
    out
}

fn emit_shader_backend_artifacts(
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

fn render_shader_backend_validation(
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
                "  backend={} kind={} status={} {}",
                variant.backend, variant.kind, status, check_summary
            ));
        }
    }
    Ok(lines.join("\n") + "\n")
}

fn emit_kernel_backend_artifacts(
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

fn render_shader_stage_summary(stage: &yir_lower_contract::ShaderStageContract) -> String {
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

fn render_shader_artifact_stub(
    stage: &yir_lower_contract::ShaderStageContract,
    variant: &yir_lower_contract::ShaderBackendVariant,
    summary: &str,
) -> String {
    match variant.backend {
        "opengl" => render_shader_glsl_scaffold(stage, variant, summary),
        "metal" => render_shader_metal_scaffold(stage, variant, summary),
        "directx" => render_shader_hlsl_scaffold(stage, variant, summary),
        "vulkan" => render_shader_vulkan_glsl_scaffold(stage, variant, summary),
        other => format!(
            "# nuis shader backend scaffold\nbackend={other}\nkind={}\nstatus={}\nentry={}\nartifact={}\n\n{}",
            variant.kind, variant.status, variant.entry, variant.artifact, summary
        ),
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

fn render_kernel_graph_summary(graph: &yir_lower_contract::KernelComputeGraphContract) -> String {
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

fn render_kernel_graph_artifact_stub(
    _graph: &yir_lower_contract::KernelComputeGraphContract,
    variant: &yir_lower_contract::KernelBackendVariant,
    summary: &str,
) -> String {
    match variant.kind {
        "graph" => render_kernel_json_scaffold("kernel_graph", variant, summary),
        "mlpackage" => render_kernel_manifest_scaffold("kernel_graph", variant, summary),
        _ => format!(
            "# nuis kernel graph backend scaffold\nbackend={}\nkind={}\nstatus={}\nentry={}\nartifact={}\n\n{}",
            variant.backend, variant.kind, variant.status, variant.entry, variant.artifact, summary
        ),
    }
}

fn render_kernel_stage_summary(stage: &yir_lower_contract::KernelStageContract) -> String {
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

fn render_kernel_stage_artifact_stub(
    _stage: &yir_lower_contract::KernelStageContract,
    variant: &yir_lower_contract::KernelBackendVariant,
    summary: &str,
) -> String {
    match variant.kind {
        "graph" => render_kernel_json_scaffold("kernel_stage", variant, summary),
        "mlmodel" => render_kernel_manifest_scaffold("kernel_stage", variant, summary),
        _ => format!(
            "# nuis kernel stage backend scaffold\nbackend={}\nkind={}\nstatus={}\nentry={}\nartifact={}\n\n{}",
            variant.backend, variant.kind, variant.status, variant.entry, variant.artifact, summary
        ),
    }
}

fn render_shader_glsl_scaffold(
    stage: &yir_lower_contract::ShaderStageContract,
    variant: &yir_lower_contract::ShaderBackendVariant,
    summary: &str,
) -> String {
    let uses_texture = shader_stage_uses_texture(stage);
    let vertex_body = shader_vertex_body(stage, ShaderTargetFlavor::OpenGl)
        .unwrap_or_else(default_glsl_vertex_body);
    let fragment_body = shader_fragment_body(stage, ShaderTargetFlavor::OpenGl, uses_texture);
    let wgsl_comment = render_wgsl_comment_block(
        stage.wgsl_entry.as_deref(),
        stage.wgsl_source.as_deref(),
        "// ",
    );
    format!(
        r#"#version 460 core
// nuis shader backend scaffold
// backend={backend}
// entry={entry}
// kind={kind}
// status={status}
//
{wgsl_comment}
//
// contract:
// {summary_comment}

#ifdef NUIS_STAGE_VERTEX
layout(location = 0) out vec2 v_uv;

void main() {{
{vertex_body}
}}
#endif

#ifdef NUIS_STAGE_FRAGMENT
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
{texture_uniforms}

void main() {{
{fragment_body}
}}
#endif
"#,
        backend = variant.backend,
        entry = variant.entry,
        kind = variant.kind,
        status = variant.status,
        wgsl_comment = wgsl_comment,
        texture_uniforms = if uses_texture {
            "layout(binding = 2) uniform sampler2D u_texture0;"
        } else {
            ""
        },
        vertex_body = vertex_body,
        fragment_body = fragment_body,
        summary_comment = summary
            .lines()
            .map(|line| format!("// {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn render_shader_metal_scaffold(
    stage: &yir_lower_contract::ShaderStageContract,
    variant: &yir_lower_contract::ShaderBackendVariant,
    summary: &str,
) -> String {
    let uses_texture = shader_stage_uses_texture(stage);
    let vertex_body = shader_vertex_body(stage, ShaderTargetFlavor::Metal)
        .unwrap_or_else(default_metal_vertex_body);
    let fragment_signature = if uses_texture {
        "fragment float4 frame_fs(VsOut in [[stage_in]], texture2d<float> tex [[texture(0)]], sampler samp [[sampler(0)]])"
    } else {
        "fragment float4 frame_fs(VsOut in [[stage_in]])"
    };
    let fragment_body = shader_fragment_body(stage, ShaderTargetFlavor::Metal, uses_texture);
    let wgsl_comment = render_wgsl_comment_block(
        stage.wgsl_entry.as_deref(),
        stage.wgsl_source.as_deref(),
        "// ",
    );
    format!(
        r#"// nuis shader backend scaffold
// backend={backend}
// entry={entry}
// kind={kind}
// status={status}
//
{wgsl_comment}
//
// contract:
// {summary_comment}

#include <metal_stdlib>
using namespace metal;

struct VsOut {{
    float4 position [[position]];
    float2 uv;
}};

vertex VsOut {entry}_vs(uint vid [[vertex_id]]) {{
    VsOut out;
{vertex_body}
    return out;
}}

{fragment_signature} {{
{fragment_body}
}}
"#,
        backend = variant.backend,
        entry = variant.entry,
        kind = variant.kind,
        status = variant.status,
        wgsl_comment = wgsl_comment,
        fragment_signature =
            fragment_signature.replace("frame_fs", &format!("{}_fs", variant.entry)),
        vertex_body = vertex_body,
        fragment_body = fragment_body,
        summary_comment = summary
            .lines()
            .map(|line| format!("// {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn render_shader_hlsl_scaffold(
    stage: &yir_lower_contract::ShaderStageContract,
    variant: &yir_lower_contract::ShaderBackendVariant,
    summary: &str,
) -> String {
    let uses_texture = shader_stage_uses_texture(stage);
    let vertex_body = shader_vertex_body(stage, ShaderTargetFlavor::Hlsl)
        .unwrap_or_else(default_hlsl_vertex_body);
    let fragment_prelude = if uses_texture {
        "Texture2D shaderTexture : register(t0);\nSamplerState shaderSampler : register(s0);\n"
    } else {
        ""
    };
    let fragment_body = shader_fragment_body(stage, ShaderTargetFlavor::Hlsl, uses_texture);
    let wgsl_comment = render_wgsl_comment_block(
        stage.wgsl_entry.as_deref(),
        stage.wgsl_source.as_deref(),
        "// ",
    );
    format!(
        r#"// nuis shader backend scaffold
// backend={backend}
// entry={entry}
// kind={kind}
// status={status}
//
{wgsl_comment}
//
// contract:
// {summary_comment}

struct VsOut {{
    float4 position : SV_Position;
    float2 uv : TEXCOORD0;
}};

VsOut {entry}_vs(uint vid : SV_VertexID) {{
    VsOut outp;
{vertex_body}
    return outp;
}}

{fragment_prelude}
float4 {entry}_ps(VsOut input) : SV_Target0 {{
{fragment_body}
}}
"#,
        backend = variant.backend,
        entry = variant.entry,
        kind = variant.kind,
        status = variant.status,
        wgsl_comment = wgsl_comment,
        fragment_prelude = fragment_prelude,
        vertex_body = vertex_body,
        fragment_body = fragment_body,
        summary_comment = summary
            .lines()
            .map(|line| format!("// {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn render_shader_vulkan_glsl_scaffold(
    stage: &yir_lower_contract::ShaderStageContract,
    variant: &yir_lower_contract::ShaderBackendVariant,
    summary: &str,
) -> String {
    let uses_texture = shader_stage_uses_texture(stage);
    let vertex_body = shader_vertex_body(stage, ShaderTargetFlavor::VulkanGlsl)
        .unwrap_or_else(default_vulkan_vertex_body);
    let fragment_body = shader_fragment_body(stage, ShaderTargetFlavor::VulkanGlsl, uses_texture);
    let wgsl_comment = render_wgsl_comment_block(
        stage.wgsl_entry.as_deref(),
        stage.wgsl_source.as_deref(),
        "// ",
    );
    format!(
        r#"#version 450
// nuis shader backend scaffold
// backend={backend}
// entry={entry}
// kind={kind}
// status={status}
//
{wgsl_comment}
//
// contract:
// {summary_comment}

#ifdef NUIS_STAGE_VERTEX
layout(location = 0) out vec2 v_uv;

void main() {{
{vertex_body}
}}
#endif

#ifdef NUIS_STAGE_FRAGMENT
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
{texture_uniforms}

void main() {{
{fragment_body}
}}
#endif
"#,
        backend = variant.backend,
        entry = variant.entry,
        kind = variant.kind,
        status = variant.status,
        wgsl_comment = wgsl_comment,
        texture_uniforms = if uses_texture {
            "layout(set = 0, binding = 2) uniform sampler2D u_texture0;"
        } else {
            ""
        },
        vertex_body = vertex_body,
        fragment_body = fragment_body,
        summary_comment = summary
            .lines()
            .map(|line| format!("// {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn shader_stage_uses_texture(stage: &yir_lower_contract::ShaderStageContract) -> bool {
    let has_texture = stage
        .bindings
        .iter()
        .any(|binding| binding.kind == "texture_binding");
    let has_sampler = stage
        .bindings
        .iter()
        .any(|binding| binding.kind == "sampler_binding");
    has_texture && has_sampler
}

#[derive(Clone, Copy)]
enum ShaderTargetFlavor {
    OpenGl,
    VulkanGlsl,
    Metal,
    Hlsl,
}

fn shader_vertex_body(
    stage: &yir_lower_contract::ShaderStageContract,
    flavor: ShaderTargetFlavor,
) -> Option<String> {
    let shader_ir = stage
        .shader_ir_stages
        .iter()
        .find(|shader_ir| shader_ir.stage == "vertex")?;
    let mut lines = Vec::new();
    for inst in &shader_ir.instructions {
        let expr = compile_backend_vertex_expr(&inst.expr, flavor)?;
        match inst.result.as_str() {
            "out.pos" => lines.push(vertex_output_assign("position", &expr, flavor)),
            "out.uv" => lines.push(vertex_output_assign("uv", &expr, flavor)),
            other => {
                let ty = inst
                    .ty
                    .as_deref()
                    .map(|ty| map_wgsl_type_to_backend(ty, flavor))
                    .unwrap_or_else(|| infer_backend_type(&inst.expr, flavor));
                lines.push(format!("    {ty} {other} = {expr};"));
            }
        }
    }
    Some(lines.join("\n"))
}

fn compile_backend_vertex_expr(expr: &str, flavor: ShaderTargetFlavor) -> Option<String> {
    let mapped = apply_backend_builtin_mapping(expr, flavor)
        .replace("f32(", "float(")
        .replace("1u", "1")
        .replace("2u", "2");
    let mapped = replace_identifier_token(
        &mapped,
        "vid",
        match flavor {
            ShaderTargetFlavor::OpenGl => "gl_VertexID",
            ShaderTargetFlavor::VulkanGlsl => "gl_VertexIndex",
            ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "vid",
        },
    );
    Some(normalize_backend_expr(&mapped))
}

fn vertex_output_assign(field: &str, expr: &str, flavor: ShaderTargetFlavor) -> String {
    match (field, flavor) {
        ("position", ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl) => {
            format!("    gl_Position = {expr};")
        }
        ("uv", ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl) => {
            format!("    v_uv = {expr};")
        }
        ("position", ShaderTargetFlavor::Metal) => format!("    out.position = {expr};"),
        ("uv", ShaderTargetFlavor::Metal) => format!("    out.uv = {expr};"),
        ("position", ShaderTargetFlavor::Hlsl) => format!("    outp.position = {expr};"),
        ("uv", ShaderTargetFlavor::Hlsl) => format!("    outp.uv = {expr};"),
        _ => format!("    {field} = {expr};"),
    }
}

fn default_glsl_vertex_body() -> String {
    "    vec2 corners[4] = vec2[](\n        vec2(-1.0, -1.0),\n        vec2( 1.0, -1.0),\n        vec2(-1.0,  1.0),\n        vec2( 1.0,  1.0)\n    );\n    vec2 uvs[4] = vec2[](\n        vec2(0.0, 0.0),\n        vec2(1.0, 0.0),\n        vec2(0.0, 1.0),\n        vec2(1.0, 1.0)\n    );\n    int idx = min(gl_VertexID, 3);\n    gl_Position = vec4(corners[idx], 0.0, 1.0);\n    v_uv = uvs[idx];".to_owned()
}

fn default_vulkan_vertex_body() -> String {
    "    vec2 corners[4] = vec2[](\n        vec2(-1.0, -1.0),\n        vec2( 1.0, -1.0),\n        vec2(-1.0,  1.0),\n        vec2( 1.0,  1.0)\n    );\n    vec2 uvs[4] = vec2[](\n        vec2(0.0, 0.0),\n        vec2(1.0, 0.0),\n        vec2(0.0, 1.0),\n        vec2(1.0, 1.0)\n    );\n    int idx = min(gl_VertexIndex, 3);\n    gl_Position = vec4(corners[idx], 0.0, 1.0);\n    v_uv = uvs[idx];".to_owned()
}

fn default_metal_vertex_body() -> String {
    "    float2 corners[4] = {\n        float2(-1.0, -1.0),\n        float2( 1.0, -1.0),\n        float2(-1.0,  1.0),\n        float2( 1.0,  1.0),\n    };\n    float2 xy = corners[min(vid, 3u)];\n    out.position = float4(xy, 0.0, 1.0);\n    float2 uvs[4] = {\n        float2(0.0, 0.0),\n        float2(1.0, 0.0),\n        float2(0.0, 1.0),\n        float2(1.0, 1.0),\n    };\n    out.uv = uvs[min(vid, 3u)];".to_owned()
}

fn default_hlsl_vertex_body() -> String {
    "    float2 corners[4] = {\n        float2(-1.0, -1.0),\n        float2( 1.0, -1.0),\n        float2(-1.0,  1.0),\n        float2( 1.0,  1.0),\n    };\n    float2 xy = corners[min(vid, 3u)];\n    outp.position = float4(xy, 0.0, 1.0);\n    float2 uvs[4] = {\n        float2(0.0, 0.0),\n        float2(1.0, 0.0),\n        float2(0.0, 1.0),\n        float2(1.0, 1.0),\n    };\n    outp.uv = uvs[min(vid, 3u)];".to_owned()
}

fn shader_fragment_body(
    stage: &yir_lower_contract::ShaderStageContract,
    flavor: ShaderTargetFlavor,
    uses_texture: bool,
) -> String {
    if let Some(shader_ir) = stage
        .shader_ir_stages
        .iter()
        .find(|shader_ir| shader_ir.stage == "fragment")
    {
        if let Some(mapped) = render_shader_ir_fragment(shader_ir, flavor, uses_texture) {
            return mapped;
        }
    }

    if let Some(wgsl) = stage.wgsl_source.as_deref() {
        if let Some(model) = extract_wgsl_fragment_model(wgsl) {
            if let Some(mapped) = render_wgsl_fragment_model(&model, flavor, uses_texture) {
                return mapped;
            }
        }
    }

    if let Some(mapped) = try_map_simple_wgsl_vec4_return(stage, flavor) {
        return mapped;
    }

    if let Some(wgsl) = stage.wgsl_source.as_deref() {
        if let Some(return_expr) = extract_wgsl_fragment_lowerable_expr(wgsl) {
            if let Some(mapped) = map_wgsl_expr_to_backend(&return_expr, flavor, uses_texture) {
                return mapped;
            }
        }
    }

    match (flavor, uses_texture) {
        (ShaderTargetFlavor::OpenGl, true) | (ShaderTargetFlavor::VulkanGlsl, true) => {
            "    vec4 sampled = texture(u_texture0, v_uv);\n    outColor = sampled;".to_owned()
        }
        (ShaderTargetFlavor::OpenGl, false) | (ShaderTargetFlavor::VulkanGlsl, false) => {
            "    outColor = vec4(0.82, 0.88, 0.97, 1.0);".to_owned()
        }
        (ShaderTargetFlavor::Metal, true) => "    return tex.sample(samp, in.uv);".to_owned(),
        (ShaderTargetFlavor::Metal, false) => {
            "    return float4(0.82, 0.88, 0.97, 1.0);".to_owned()
        }
        (ShaderTargetFlavor::Hlsl, true) => {
            "    return shaderTexture.Sample(shaderSampler, input.uv);".to_owned()
        }
        (ShaderTargetFlavor::Hlsl, false) => "    return float4(0.82, 0.88, 0.97, 1.0);".to_owned(),
    }
}

struct WgslFragmentBinding {
    name: String,
    ty: Option<String>,
    expr: String,
}

struct WgslFragmentModel {
    bindings: Vec<WgslFragmentBinding>,
    return_expr: String,
}

fn try_map_simple_wgsl_vec4_return(
    stage: &yir_lower_contract::ShaderStageContract,
    flavor: ShaderTargetFlavor,
) -> Option<String> {
    let wgsl = stage.wgsl_source.as_deref()?;
    let expr = extract_wgsl_fragment_lowerable_expr(wgsl)?;
    let inner = expr
        .strip_prefix("vec4<f32>(")
        .or_else(|| expr.strip_prefix("vec4("))?
        .strip_suffix(')')?;
    let comps = split_top_level_args(inner);
    if comps.len() != 4 {
        return None;
    }
    if comps
        .iter()
        .any(|component| !is_simple_wgsl_component_expr(component.trim()))
    {
        return None;
    }

    let mapped = comps
        .iter()
        .map(|component| map_wgsl_component_ref(component.trim(), flavor))
        .collect::<Vec<_>>()
        .join(", ");

    Some(match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => {
            format!("    outColor = vec4({mapped});")
        }
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => {
            let ctor = match flavor {
                ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float4",
                _ => "vec4",
            };
            format!("    return {ctor}({mapped});")
        }
    })
}

fn map_wgsl_component_ref(component: &str, flavor: ShaderTargetFlavor) -> String {
    map_wgsl_expr_identifiers(component, flavor)
}

fn is_simple_wgsl_component_expr(component: &str) -> bool {
    !component.contains('(') && !component.contains(')')
}

fn extract_wgsl_fragment_return_expr(wgsl: &str) -> Option<String> {
    let fragment_pos = wgsl.find("@fragment")?;
    let fragment_src = &wgsl[fragment_pos..];
    let return_pos = fragment_src.find("return")?;
    let after_return = &fragment_src[return_pos + "return".len()..];
    let semicolon_pos = after_return.find(';')?;
    Some(after_return[..semicolon_pos].trim().to_owned())
}

fn extract_wgsl_fragment_model(wgsl: &str) -> Option<WgslFragmentModel> {
    let fragment_pos = wgsl.find("@fragment")?;
    let fragment_src = &wgsl[fragment_pos..];
    let return_expr = extract_wgsl_fragment_return_expr(wgsl)?;
    let mut bindings = Vec::new();
    for raw_line in fragment_src.lines() {
        let line = raw_line.trim();
        if !line.starts_with("let ") {
            continue;
        }
        let Some(eq_pos) = line.find('=') else {
            continue;
        };
        let lhs = line["let ".len()..eq_pos].trim();
        let rhs = line[eq_pos + 1..].trim().trim_end_matches(';').trim();
        if rhs.is_empty() {
            continue;
        }
        let (name, ty) = if let Some(colon_pos) = lhs.find(':') {
            (
                lhs[..colon_pos].trim().to_owned(),
                Some(lhs[colon_pos + 1..].trim().to_owned()),
            )
        } else {
            (lhs.to_owned(), None)
        };
        if name.is_empty() {
            continue;
        }
        bindings.push(WgslFragmentBinding {
            name,
            ty,
            expr: rhs.to_owned(),
        });
    }
    Some(WgslFragmentModel {
        bindings,
        return_expr,
    })
}

fn extract_wgsl_fragment_lowerable_expr(wgsl: &str) -> Option<String> {
    let return_expr = extract_wgsl_fragment_return_expr(wgsl)?;
    let fragment_pos = wgsl.find("@fragment")?;
    let fragment_src = &wgsl[fragment_pos..];
    let let_bindings = extract_wgsl_fragment_let_bindings(fragment_src);
    Some(expand_wgsl_expr_bindings(&return_expr, &let_bindings))
}

fn extract_wgsl_fragment_let_bindings(fragment_src: &str) -> Vec<(String, String)> {
    let mut bindings = Vec::new();
    for raw_line in fragment_src.lines() {
        let line = raw_line.trim();
        if !line.starts_with("let ") {
            continue;
        }
        let Some(eq_pos) = line.find('=') else {
            continue;
        };
        let lhs = line["let ".len()..eq_pos].trim();
        let name = lhs.split(':').next().map(str::trim).unwrap_or(lhs);
        if name.is_empty() {
            continue;
        }
        let rhs = line[eq_pos + 1..].trim().trim_end_matches(';').trim();
        if rhs.is_empty() {
            continue;
        }
        bindings.push((name.to_owned(), rhs.to_owned()));
    }
    bindings
}

fn expand_wgsl_expr_bindings(expr: &str, bindings: &[(String, String)]) -> String {
    let mut expanded = expr.to_owned();
    for _ in 0..bindings.len() {
        let mut changed = false;
        for (name, value) in bindings {
            let next = replace_identifier_token(&expanded, name, &format!("({value})"));
            if next != expanded {
                expanded = next;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    expanded
}

fn replace_identifier_token(haystack: &str, name: &str, replacement: &str) -> String {
    let chars = haystack.chars().collect::<Vec<_>>();
    let name_chars = name.chars().collect::<Vec<_>>();
    let mut out = String::with_capacity(haystack.len() + replacement.len());
    let mut i = 0;
    while i < chars.len() {
        let matches_name =
            i + name_chars.len() <= chars.len() && chars[i..i + name_chars.len()] == name_chars[..];
        if matches_name {
            let prev_ok = i == 0 || (!is_identifier_char(chars[i - 1]) && chars[i - 1] != '.');
            let next_ok = i + name_chars.len() == chars.len()
                || !is_identifier_char(chars[i + name_chars.len()]);
            if prev_ok && next_ok {
                out.push_str(replacement);
                i += name_chars.len();
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn is_identifier_char(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn render_wgsl_fragment_model(
    model: &WgslFragmentModel,
    flavor: ShaderTargetFlavor,
    uses_texture: bool,
) -> Option<String> {
    let mut lines = Vec::new();
    for binding in &model.bindings {
        let ty = binding
            .ty
            .as_deref()
            .map(|ty| map_wgsl_type_to_backend(ty, flavor))
            .unwrap_or_else(|| infer_backend_type(&binding.expr, flavor));
        let expr = compile_backend_expr(&binding.expr, flavor, uses_texture)?;
        lines.push(format!("    {ty} {} = {expr};", binding.name));
    }
    let return_expr = compile_backend_expr(&model.return_expr, flavor, uses_texture)?;
    match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => {
            lines.push(format!("    outColor = {return_expr};"));
        }
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => {
            lines.push(format!("    return {return_expr};"));
        }
    }
    Some(lines.join("\n"))
}

fn render_shader_ir_fragment(
    shader_ir: &yir_lower_contract::NustarContractStage,
    flavor: ShaderTargetFlavor,
    uses_texture: bool,
) -> Option<String> {
    if shader_ir.stage != "fragment" {
        return None;
    }

    let mut lines = Vec::new();
    for inst in &shader_ir.instructions {
        let ty = inst
            .ty
            .as_deref()
            .map(|ty| map_wgsl_type_to_backend(ty, flavor))
            .unwrap_or_else(|| infer_backend_type(&inst.expr, flavor));
        let expr = compile_backend_expr(&inst.expr, flavor, uses_texture)?;
        lines.push(format!("    {ty} {} = {expr};", inst.result));
    }

    let return_expr = compile_backend_expr(&shader_ir.terminator.expr, flavor, uses_texture)?;
    match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => {
            lines.push(format!("    outColor = {return_expr};"));
        }
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => {
            lines.push(format!("    return {return_expr};"));
        }
    }

    Some(lines.join("\n"))
}

fn compile_backend_expr(
    expr: &str,
    flavor: ShaderTargetFlavor,
    uses_texture: bool,
) -> Option<String> {
    let normalized_expr =
        map_wgsl_expr_identifiers(&apply_backend_builtin_mapping(expr, flavor), flavor);
    let mapped_expr = map_wgsl_texture_sample_expr(&normalized_expr, flavor, uses_texture)?;
    Some(normalize_backend_expr(&mapped_expr))
}

fn map_wgsl_type_to_backend(ty: &str, flavor: ShaderTargetFlavor) -> String {
    ty.replace(
        "vec4<f32>",
        match flavor {
            ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec4",
            ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float4",
        },
    )
    .replace(
        "vec3<f32>",
        match flavor {
            ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec3",
            ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float3",
        },
    )
    .replace(
        "vec2<f32>",
        match flavor {
            ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec2",
            ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float2",
        },
    )
    .replace("f32", "float")
    .replace("u32", "uint")
}

fn infer_backend_type(expr: &str, flavor: ShaderTargetFlavor) -> String {
    if expr.contains("vec4") || expr.contains("textureSample(") {
        match flavor {
            ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec4".to_owned(),
            ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float4".to_owned(),
        }
    } else if expr.contains("vec3") || expr.contains(".xyz") {
        match flavor {
            ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec3".to_owned(),
            ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float3".to_owned(),
        }
    } else if expr.contains("vec2") || expr.contains(".xy") {
        match flavor {
            ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec2".to_owned(),
            ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float2".to_owned(),
        }
    } else {
        "float".to_owned()
    }
}

fn map_wgsl_expr_to_backend(
    expr: &str,
    flavor: ShaderTargetFlavor,
    uses_texture: bool,
) -> Option<String> {
    let normalized_expr =
        map_wgsl_expr_identifiers(&apply_backend_builtin_mapping(expr, flavor), flavor);
    let mapped_expr = map_wgsl_texture_sample_expr(&normalized_expr, flavor, uses_texture)?;
    let (prelude_lines, final_expr) = hoist_repeated_texture_samples(&mapped_expr, flavor);
    let (prelude_lines, final_expr) =
        hoist_repeated_math_subexpressions(prelude_lines, final_expr, flavor);
    let prelude_lines = prelude_lines
        .into_iter()
        .map(|line| normalize_backend_expr(&line))
        .collect::<Vec<_>>();
    let final_expr = normalize_backend_expr(&final_expr);
    let (prelude_lines, final_expr) =
        hoist_normalized_math_subexpressions(prelude_lines, final_expr, flavor);

    Some(match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => {
            if prelude_lines.is_empty() {
                format!("    outColor = {};", final_expr)
            } else {
                format!(
                    "{}\n    outColor = {};",
                    prelude_lines.join("\n"),
                    final_expr
                )
            }
        }
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => {
            if prelude_lines.is_empty() {
                format!("    return {};", final_expr)
            } else {
                format!("{}\n    return {};", prelude_lines.join("\n"), final_expr)
            }
        }
    })
}

fn apply_backend_builtin_mapping(expr: &str, flavor: ShaderTargetFlavor) -> String {
    let mapped = expr
        .replace(
            "vec4<f32>",
            match flavor {
                ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec4",
                ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float4",
            },
        )
        .replace(
            "vec3<f32>",
            match flavor {
                ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec3",
                ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float3",
            },
        )
        .replace(
            "vec2<f32>",
            match flavor {
                ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec2",
                ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float2",
            },
        )
        .replace("@location(0)", "");

    let mix_name = match flavor {
        ShaderTargetFlavor::Hlsl => "lerp",
        _ => "mix",
    };
    let fract_name = match flavor {
        ShaderTargetFlavor::Hlsl => "frac",
        _ => "fract",
    };

    let mapped = replace_identifier_token(&mapped, "mix", mix_name);
    replace_identifier_token(&mapped, "fract", fract_name)
}

fn map_wgsl_expr_identifiers(expr: &str, flavor: ShaderTargetFlavor) -> String {
    replace_identifier_token(
        expr,
        "uv",
        match flavor {
            ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "v_uv",
            ShaderTargetFlavor::Metal => "in.uv",
            ShaderTargetFlavor::Hlsl => "input.uv",
        },
    )
}

fn map_wgsl_texture_sample_expr(
    expr: &str,
    flavor: ShaderTargetFlavor,
    uses_texture: bool,
) -> Option<String> {
    if expr.contains("textureSample(") && !uses_texture {
        return None;
    }
    let mut mapped = expr.to_owned();
    while let Some(call) = find_texture_sample_call(&mapped) {
        let replacement = render_backend_texture_sample_call(&call.args, flavor)?;
        mapped.replace_range(call.start..call.end, &replacement);
    }
    Some(mapped)
}

struct TextureSampleCall {
    start: usize,
    end: usize,
    args: Vec<String>,
}

fn find_texture_sample_call(expr: &str) -> Option<TextureSampleCall> {
    let start = expr.find("textureSample(")?;
    let open = start + "textureSample".len();
    let mut depth = 0usize;
    let mut end = None;
    for (idx, ch) in expr[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end = Some(open + idx + ch.len_utf8());
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end?;
    let inner = &expr[open + 1..end - 1];
    Some(TextureSampleCall {
        start,
        end,
        args: split_top_level_args(inner),
    })
}

fn render_backend_texture_sample_call(
    args: &[String],
    flavor: ShaderTargetFlavor,
) -> Option<String> {
    if args.len() != 3 {
        return None;
    }
    let coord = args[2].trim();
    Some(match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => {
            format!("texture(u_texture0, {coord})")
        }
        ShaderTargetFlavor::Metal => format!("tex.sample(samp, {coord})"),
        ShaderTargetFlavor::Hlsl => format!("shaderTexture.Sample(shaderSampler, {coord})"),
    })
}

fn split_top_level_args(raw: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (idx, ch) in raw.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                args.push(raw[start..idx].trim().to_owned());
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }
    let tail = raw[start..].trim();
    if !tail.is_empty() {
        args.push(tail.to_owned());
    }
    args
}

fn hoist_repeated_texture_samples(expr: &str, flavor: ShaderTargetFlavor) -> (Vec<String>, String) {
    let sample_prefix = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "texture(u_texture0, ",
        ShaderTargetFlavor::Metal => "tex.sample(samp, ",
        ShaderTargetFlavor::Hlsl => "shaderTexture.Sample(shaderSampler, ",
    };
    let sample_type = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec4",
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float4",
    };

    let mut rewritten = expr.to_owned();
    let mut prelude = Vec::new();
    let mut slot = 0usize;
    loop {
        let calls = collect_function_calls(&rewritten, sample_prefix);
        let Some(first) = calls.first() else {
            break;
        };
        let target = first.text.clone();
        let count = calls.iter().filter(|call| call.text == target).count();
        if count <= 1 {
            break;
        }
        let temp_name = format!("nuis_sample_{slot}");
        slot += 1;
        prelude.push(format!("    {sample_type} {temp_name} = {target};"));
        rewritten = rewritten.replace(&target, &temp_name);
    }
    (prelude, rewritten)
}

fn hoist_repeated_math_subexpressions(
    mut prelude: Vec<String>,
    expr: String,
    flavor: ShaderTargetFlavor,
) -> (Vec<String>, String) {
    let vec2_ty = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec2",
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float2",
    };

    let mut rewritten = expr;
    let joined = prelude.join("\n") + "\n" + &rewritten;
    if let Some(call) =
        canonical_uv_clamp_expr(flavor).filter(|call| joined.matches(call).count() >= 2)
    {
        let temp = "nuis_uv_0";
        prelude = prelude
            .into_iter()
            .map(|line| {
                line.replace(&format!("({call})"), temp)
                    .replace(&call, temp)
            })
            .collect::<Vec<_>>();
        prelude.insert(0, format!("    {vec2_ty} {temp} = {};", call));
        prelude[0] = format!("    {vec2_ty} {temp} = {};", call);
        rewritten = rewritten.replace(&call, temp);
    }

    if let Some(call) =
        canonical_wave_expr(flavor).filter(|call| rewritten.matches(call).count() >= 2)
    {
        let temp = "nuis_wave_0";
        prelude.push(format!("    {vec2_ty} {temp} = {};", call));
        rewritten = rewritten.replace(&call, temp);
    }

    (prelude, rewritten)
}

fn canonical_uv_clamp_expr(flavor: ShaderTargetFlavor) -> Option<String> {
    Some(match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => {
            "clamp(v_uv, vec2(0.0, 0.0), vec2(1.0, 1.0))".to_owned()
        }
        ShaderTargetFlavor::Metal => "clamp(in.uv, float2(0.0, 0.0), float2(1.0, 1.0))".to_owned(),
        ShaderTargetFlavor::Hlsl => {
            "clamp(input.uv, float2(0.0, 0.0), float2(1.0, 1.0))".to_owned()
        }
    })
}

fn canonical_wave_expr(flavor: ShaderTargetFlavor) -> Option<String> {
    let uv_temp = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "nuis_uv_0",
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "nuis_uv_0",
    };
    Some(match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => {
            format!("fract({uv_temp} * 3.0)")
        }
        ShaderTargetFlavor::Metal => format!("fract({uv_temp} * 3.0)"),
        ShaderTargetFlavor::Hlsl => format!("frac({uv_temp} * 3.0)"),
    })
}

fn hoist_normalized_math_subexpressions(
    prelude: Vec<String>,
    expr: String,
    flavor: ShaderTargetFlavor,
) -> (Vec<String>, String) {
    let vec2_ty = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec2",
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float2",
    };
    let vec3_ty = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec3",
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float3",
    };
    let scalar_ty = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "float",
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float",
    };

    let mut prelude = prelude;
    let mut rewritten = expr;
    let wave_expr = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => {
            "fract(nuis_uv_0 * 3.0)".to_owned()
        }
        ShaderTargetFlavor::Metal => "fract(nuis_uv_0 * 3.0)".to_owned(),
        ShaderTargetFlavor::Hlsl => "frac(nuis_uv_0 * 3.0)".to_owned(),
    };
    let wave_xy_expr = format!("({wave_expr}).xy");
    let joined = prelude.join("\n") + "\n" + &rewritten;
    if joined.matches(&wave_xy_expr).count() >= 2 {
        let temp = "nuis_wave_0";
        prelude = prelude
            .into_iter()
            .map(|line| line.replace(&wave_xy_expr, &format!("{temp}.xy")))
            .collect::<Vec<_>>();
        rewritten = rewritten.replace(&wave_xy_expr, &format!("{temp}.xy"));
        prelude.push(format!("    {vec2_ty} {temp} = {wave_expr};"));
    }

    let light_expr = canonical_light_expr(flavor);
    let joined = prelude.join("\n") + "\n" + &rewritten;
    if joined.matches(&light_expr).count() >= 1 {
        let temp = "nuis_light_0";
        prelude = prelude
            .into_iter()
            .map(|line| line.replace(&light_expr, temp))
            .collect::<Vec<_>>();
        rewritten = rewritten.replace(&light_expr, temp);
        prelude.push(format!("    {scalar_ty} {temp} = {light_expr};"));
    }

    let tint_expr = canonical_tint_expr(flavor);
    let tint_xyz_expr = format!("({tint_expr}).xyz");
    let joined = prelude.join("\n") + "\n" + &rewritten;
    if joined.matches(&tint_xyz_expr).count() >= 1 {
        let temp = "nuis_tint_0";
        prelude = prelude
            .into_iter()
            .map(|line| line.replace(&tint_xyz_expr, &format!("{temp}.xyz")))
            .collect::<Vec<_>>();
        rewritten = rewritten.replace(&tint_xyz_expr, &format!("{temp}.xyz"));
        prelude.push(format!("    {vec3_ty} {temp} = {tint_expr};"));
    }

    (prelude, rewritten)
}

fn canonical_light_expr(flavor: ShaderTargetFlavor) -> String {
    let vec3_ty = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec3",
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float3",
    };
    format!(
        "smoothstep(0.15, 0.95, dot((normalize({vec3_ty}(nuis_wave_0.xy, 0.5))), (normalize({vec3_ty}(0.4, 0.6, 0.7)))))"
    )
}

fn canonical_tint_expr(flavor: ShaderTargetFlavor) -> String {
    let vec3_ty = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec3",
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float3",
    };
    format!(
        "clamp(nuis_sample_0.xyz * nuis_light_0 + {vec3_ty}(nuis_wave_0.xy, 1.0 - nuis_uv_0.x) * 0.25, {vec3_ty}(0.0, 0.0, 0.0), {vec3_ty}(1.0, 1.0, 1.0))"
    )
}

fn normalize_backend_expr(expr: &str) -> String {
    let mut normalized = expr.to_owned();
    for _ in 0..8 {
        let next = normalize_backend_expr_once(&normalized);
        if next == normalized {
            break;
        }
        normalized = next;
    }
    normalized
}

fn normalize_backend_expr_once(expr: &str) -> String {
    let mut out = expr.to_owned();
    out = strip_parens_around_member_bases(&out);
    out = strip_parens_around_simple_atoms(&out);
    out = remove_redundant_vec_ctor_swizzles(&out);
    out = remove_wrapped_ctor_swizzles(&out);
    out = remove_redundant_fn_result_swizzles(&out, "mix");
    out = remove_redundant_fn_result_swizzles(&out, "lerp");
    out = remove_wrapped_fn_result_swizzles(&out, "mix");
    out = remove_wrapped_fn_result_swizzles(&out, "lerp");
    out
}

fn strip_parens_around_member_bases(expr: &str) -> String {
    let chars = expr.chars().collect::<Vec<_>>();
    let mut out = String::with_capacity(expr.len());
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '(' {
            let mut j = i + 1;
            let mut seen = false;
            while j < chars.len() && is_simple_member_char(chars[j]) {
                seen = true;
                j += 1;
            }
            if seen
                && j < chars.len()
                && chars[j] == ')'
                && j + 1 < chars.len()
                && chars[j + 1] == '.'
            {
                for ch in &chars[i + 1..j] {
                    out.push(*ch);
                }
                i = j + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn is_simple_member_char(ch: char) -> bool {
    is_identifier_char(ch) || ch == '.'
}

fn strip_parens_around_simple_atoms(expr: &str) -> String {
    let chars = expr.chars().collect::<Vec<_>>();
    let mut out = String::with_capacity(expr.len());
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '(' {
            let mut j = i + 1;
            let mut seen = false;
            while j < chars.len() && is_simple_member_char(chars[j]) {
                seen = true;
                j += 1;
            }
            if seen && j < chars.len() && chars[j] == ')' {
                for ch in &chars[i + 1..j] {
                    out.push(*ch);
                }
                i = j + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn remove_redundant_vec_ctor_swizzles(expr: &str) -> String {
    let mut out = expr.to_owned();
    for ctor in ["vec2", "vec3", "vec4", "float2", "float3", "float4"] {
        let full = format!("{ctor}(");
        while let Some(start) = out.find(&full) {
            let open = start + ctor.len();
            let Some(end) = find_matching_paren(&out, open) else {
                break;
            };
            let tail = &out[end + 1..];
            let redundant = match ctor {
                "vec2" | "float2" => tail.starts_with(".xy"),
                "vec3" | "float3" => tail.starts_with(".xyz"),
                "vec4" | "float4" => tail.starts_with(".xyzw"),
                _ => false,
            };
            if redundant {
                out.replace_range(end + 1..end + 1 + redundant_swizzle_len(ctor), "");
                continue;
            }
            break;
        }
    }
    out
}

fn remove_wrapped_ctor_swizzles(expr: &str) -> String {
    let mut out = expr.to_owned();
    for ctor in ["vec2", "vec3", "vec4", "float2", "float3", "float4"] {
        out = remove_wrapped_call_swizzle(&out, ctor);
    }
    out
}

fn remove_redundant_fn_result_swizzles(expr: &str, fn_name: &str) -> String {
    let mut out = expr.to_owned();
    let needle = format!("{fn_name}(");
    while let Some(start) = out.find(&needle) {
        let open = start + fn_name.len();
        let Some(end) = find_matching_paren(&out, open) else {
            break;
        };
        if out[end + 1..].starts_with(".xyz") {
            out.replace_range(end + 1..end + 5, "");
            continue;
        }
        break;
    }
    out
}

fn remove_wrapped_fn_result_swizzles(expr: &str, fn_name: &str) -> String {
    remove_wrapped_call_swizzle(expr, fn_name)
}

fn remove_wrapped_call_swizzle(expr: &str, name: &str) -> String {
    let mut out = expr.to_owned();
    let needle = format!("({name}(");
    while let Some(start) = out.find(&needle) {
        let open = start + 1 + name.len();
        let Some(end) = find_matching_paren(&out, open) else {
            break;
        };
        let swizzle = if out[end + 1..].starts_with(").xyzw") {
            Some((".xyzw", 6usize))
        } else if out[end + 1..].starts_with(").xyz") {
            Some((".xyz", 5usize))
        } else if out[end + 1..].starts_with(").xy") {
            Some((".xy", 4usize))
        } else {
            None
        };
        let Some((suffix, remove_len)) = swizzle else {
            break;
        };
        let inner = out[start + 1..end + 1].to_owned();
        let replacement = if suffix == ".xyzw" { inner } else { inner };
        out.replace_range(start..end + 1 + remove_len, &replacement);
    }
    out
}

fn redundant_swizzle_len(ctor: &str) -> usize {
    match ctor {
        "vec2" | "float2" => 3,
        "vec3" | "float3" => 4,
        "vec4" | "float4" => 5,
        _ => 0,
    }
}

fn find_matching_paren(expr: &str, open_idx: usize) -> Option<usize> {
    let chars = expr.char_indices().collect::<Vec<_>>();
    let start_pos = chars.iter().position(|(idx, _)| *idx == open_idx)?;
    let mut depth = 0usize;
    for (idx, ch) in chars.into_iter().skip(start_pos) {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

#[derive(Clone)]
struct FunctionCallMatch {
    text: String,
}

fn collect_function_calls(expr: &str, prefix: &str) -> Vec<FunctionCallMatch> {
    let mut calls = Vec::new();
    let mut search_start = 0usize;
    while let Some(rel_start) = expr[search_start..].find(prefix) {
        let start = search_start + rel_start;
        let Some(prefix_open_rel) = prefix.find('(') else {
            break;
        };
        let open = start + prefix_open_rel;
        let mut depth = 1usize;
        let mut end = None;
        for (idx, ch) in expr[open + 1..].char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        end = Some(open + 1 + idx + ch.len_utf8());
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(end) = end else {
            break;
        };
        calls.push(FunctionCallMatch {
            text: expr[start..end].to_owned(),
        });
        search_start = end;
    }
    calls
}

fn render_wgsl_comment_block(
    wgsl_entry: Option<&str>,
    wgsl_source: Option<&str>,
    prefix: &str,
) -> String {
    let Some(source) = wgsl_source else {
        return format!("{prefix}original_wgsl=absent");
    };
    let mut lines = Vec::new();
    if let Some(entry) = wgsl_entry {
        lines.push(format!("{prefix}original_wgsl_entry={entry}"));
    } else {
        lines.push(format!("{prefix}original_wgsl_entry=unknown"));
    }
    lines.push(format!("{prefix}original_wgsl_begin"));
    for line in source.lines() {
        lines.push(format!("{prefix}{line}"));
    }
    lines.push(format!("{prefix}original_wgsl_end"));
    lines.join("\n")
}

fn shader_backend_checks(
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
        "{{\n  \"schema\": \"nuis-{subject}-backend-scaffold-v1\",\n  \"backend\": \"{backend}\",\n  \"kind\": \"{kind}\",\n  \"status\": \"{status}\",\n  \"entry\": \"{entry}\",\n  \"artifact\": \"{artifact}\",\n  \"summary\": [\n{summary_json}\n  ]\n}}\n",
        subject = subject,
        backend = variant.backend,
        kind = variant.kind,
        status = variant.status,
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
        "schema = \"nuis-{subject}-backend-scaffold-v1\"\nbackend = \"{backend}\"\nkind = \"{kind}\"\nstatus = \"{status}\"\nentry = \"{entry}\"\nartifact = \"{artifact}\"\n\n[summary]\ntext = \"{summary_text}\"\n",
        subject = subject,
        backend = variant.backend,
        kind = variant.kind,
        status = variant.status,
        entry = variant.entry,
        artifact = variant.artifact,
        summary_text = summary.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', " | ")
    )
}

fn c_string_literal(raw: &str) -> String {
    raw.replace('\\', "\\\\").replace('"', "\\\"")
}

fn compile_native_binary(ll_path: &Path, shim_path: &Path, exe_path: &Path) -> Result<(), String> {
    let output = Command::new("/usr/bin/clang")
        .arg(ll_path)
        .arg(shim_path)
        .arg("-O2")
        .arg("-o")
        .arg(exe_path)
        .output()
        .map_err(|error| format!("failed to invoke clang: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "clang failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn compile_native_appkit_binary(
    ll_path: &Path,
    host_path: &Path,
    runtime_staticlib_path: Option<&Path>,
    exe_path: &Path,
) -> Result<(), String> {
    let mut command = Command::new("/usr/bin/clang");
    command
        .arg(ll_path)
        .arg(host_path)
        .arg("-O2")
        .arg("-framework")
        .arg("AppKit")
        .arg("-framework")
        .arg("Foundation");
    if let Some(staticlib_path) = runtime_staticlib_path {
        command.arg(staticlib_path);
    }
    let output = command
        .arg("-o")
        .arg(exe_path)
        .output()
        .map_err(|error| format!("failed to invoke clang for AppKit host: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "clang AppKit host build failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn maybe_emit_prerendered_frame(
    module: &YirModule,
    output_dir: &Path,
    stem: &str,
    frame_scale: usize,
) -> Result<Option<FrameBundle>, String> {
    let has_non_cpu = module
        .resources
        .iter()
        .any(|resource| !resource.kind.is_family("cpu"));
    if !has_non_cpu {
        return Ok(None);
    }

    let trace = execute_module(module)?;
    let frame = trace
        .values
        .values()
        .filter_map(|value| match value {
            Value::Frame(frame) => Some(frame),
            _ => None,
        })
        .last();

    let Some(frame) = frame else {
        return Ok(None);
    };

    let image = rasterize_frame(frame, frame_scale);
    let assets_dir = output_dir.join("assets");
    fs::create_dir_all(&assets_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", assets_dir.display()))?;
    let ppm_path = assets_dir.join(format!("{stem}.ppm"));
    let ppm_bytes = image.to_ppm();
    fs::write(&ppm_path, &ppm_bytes)
        .map_err(|error| format!("failed to write `{}`: {error}", ppm_path.display()))?;
    Ok(Some(FrameBundle {
        asset_path: ppm_path,
        embedded_ppm_bytes: bytes_to_c_array(&ppm_bytes),
    }))
}

fn cleanup_stale_fallback_frame(output_dir: &Path, stem: &str) -> Result<(), String> {
    let stale_path = output_dir.join("assets").join(format!("{stem}.ppm"));
    if stale_path.exists() {
        fs::remove_file(&stale_path)
            .map_err(|error| format!("failed to remove `{}`: {error}", stale_path.display()))?;
    }
    Ok(())
}

struct FrameBundle {
    asset_path: PathBuf,
    embedded_ppm_bytes: String,
}

struct RuntimeFrameSupport {
    staticlib_path: PathBuf,
    embedded_module_bytes: String,
    frame_scale: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HostFfiReturnType {
    I64,
    I32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HostFfiArgType {
    I64,
    I32,
}

trait HostFfiScalarType {
    fn token(self) -> &'static str;
    fn c_type(self) -> &'static str;
}

impl HostFfiScalarType for HostFfiReturnType {
    fn token(self) -> &'static str {
        match self {
            Self::I64 => "i64",
            Self::I32 => "i32",
        }
    }

    fn c_type(self) -> &'static str {
        match self {
            Self::I64 => "int64_t",
            Self::I32 => "int32_t",
        }
    }
}

impl HostFfiScalarType for HostFfiArgType {
    fn token(self) -> &'static str {
        match self {
            Self::I64 => "i64",
            Self::I32 => "i32",
        }
    }

    fn c_type(self) -> &'static str {
        match self {
            Self::I64 => "int64_t",
            Self::I32 => "int32_t",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HostFfiSignature {
    abi: String,
    return_type: HostFfiReturnType,
    arg_types: Vec<HostFfiArgType>,
}

const PACKER_BUILTIN_HOST_FFI_REGISTRY_LINES: &[&str] = &[
    "nurs:ffi_symbol:HostRenderCurves__color_bias=i64(i64)|ffi_symbol:HostRenderCurves__speed_curve=i64(i64)|ffi_symbol:HostRenderCurves__radius_curve=i64(i64)|ffi_symbol:HostRenderCurves__mix_tick=i64(i64,i64)|ffi_symbol:HostMath__speed_curve=i64(i64)",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct HostFfiRegistryLines {
    source: String,
    lines: Vec<String>,
}

fn default_host_ffi_registry() -> HostFfiRegistryLines {
    load_default_host_ffi_registry().unwrap_or_else(|error| HostFfiRegistryLines {
        source: format!("packer-builtin-fallback:{error}"),
        lines: PACKER_BUILTIN_HOST_FFI_REGISTRY_LINES
            .iter()
            .map(|line| line.to_string())
            .collect(),
    })
}

fn load_default_host_ffi_registry() -> Result<HostFfiRegistryLines, String> {
    let manifest_path = host_ffi_registry_manifest_candidates()
        .into_iter()
        .find(|path| path.exists())
        .ok_or_else(|| {
            "could not find `nustar-packages/cpu.toml` for host FFI registry loading".to_owned()
        })?;
    let source = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("failed to read `{}`: {error}", manifest_path.display()))?;
    let mut lines = PACKER_BUILTIN_HOST_FFI_REGISTRY_LINES
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    lines.extend(
        parse_manifest_string_array(&source, "abi_capabilities").ok_or_else(|| {
            format!(
                "manifest `{}` does not declare `abi_capabilities`",
                manifest_path.display()
            )
        })?,
    );
    parse_host_ffi_registry_lines(&lines)?;
    Ok(HostFfiRegistryLines {
        source: format!("cpu-manifest:{}", manifest_path.display()),
        lines,
    })
}

fn host_ffi_registry_manifest_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(path) = env::var("NUIS_HOST_FFI_REGISTRY_MANIFEST") {
        out.push(PathBuf::from(path));
    }
    out.push(PathBuf::from("nustar-packages/cpu.toml"));
    out.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../nustar-packages/cpu.toml"));
    out
}

fn parse_manifest_string_array(source: &str, key: &str) -> Option<Vec<String>> {
    let key_start = source.find(&format!("{key} = ["))?;
    let after_open = &source[key_start..].split_once('[')?.1;
    let mut in_string = false;
    let mut escaped = false;
    let mut end_index = None;
    for (index, ch) in after_open.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            ']' if !in_string => {
                end_index = Some(index);
                break;
            }
            _ => {}
        }
    }
    let body = &after_open[..end_index?];
    Some(split_quoted_array_items(body))
}

fn split_quoted_array_items(value: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for ch in value.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            ',' if !in_string => {
                push_manifest_array_item(&mut values, &current);
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    push_manifest_array_item(&mut values, &current);
    values
}

fn push_manifest_array_item(values: &mut Vec<String>, raw: &str) {
    let item = raw.trim().trim_matches('"').trim();
    if !item.is_empty() {
        values.push(item.to_owned());
    }
}

impl HostFfiSignature {
    fn arg_count(&self) -> usize {
        self.arg_types.len()
    }

    fn render(&self) -> String {
        format!(
            "{}({})",
            self.return_type.token(),
            self.arg_types
                .iter()
                .map(|arg| arg.token())
                .collect::<Vec<_>>()
                .join(",")
        )
    }

    fn hash(&self, symbol: &str) -> String {
        ffi_symbol_signature_hash(&self.abi, symbol, &self.render())
    }
}

fn bytes_to_c_array(bytes: &[u8]) -> String {
    let mut out = String::new();
    for (index, byte) in bytes.iter().enumerate() {
        if index % 12 == 0 {
            out.push_str("\n    ");
        }
        out.push_str(&format!("0x{byte:02X}, "));
    }
    out.push('\n');
    out
}

fn c_shim_source(host_ffi_symbols: &BTreeMap<String, HostFfiSignature>) -> String {
    let host_ffi_stubs = render_host_ffi_stubs(host_ffi_symbols);
    let mut out = String::new();
    out.push_str(
        r#"#include <stdint.h>
#include <stdio.h>

extern int64_t nuis_yir_entry(void);

void nuis_debug_print_i64(int64_t value) {
    printf("%lld\n", (long long)value);
}

void nuis_debug_print_bool(int32_t value) {
    printf("%s\n", value ? "true" : "false");
}

void nuis_debug_print_i32(int32_t value) {
    printf("%d\n", value);
}

void nuis_debug_print_f32(float value) {
    printf("%g\n", value);
}

void nuis_debug_print_f64(double value) {
    printf("%g\n", value);
}

int64_t host_color_bias(int64_t value) {
    int64_t biased = value + 12;
    if (biased < 0) return 0;
    if (biased > 255) return 255;
    return biased;
}

int64_t host_speed_curve(int64_t value) {
    return value * 2 + 3;
}

int64_t host_radius_curve(int64_t value) {
    return (value * 3) / 2 + 8;
}

int64_t host_mix_tick(int64_t base, int64_t tick) {
    return base + tick;
}
"#,
    );
    out.push_str(&host_ffi_stubs);
    out.push_str(
        r#"

int main(void) {
    return (int)nuis_yir_entry();
}
"#,
    );
    out
}

fn validate_host_ffi_symbols(module: &YirModule) -> Result<(), String> {
    for node in &module.nodes {
        if node.op.module != "cpu" || !is_cpu_extern_call_instruction(&node.op.instruction) {
            continue;
        }
        if node.op.args.len() < 2 {
            return Err(format!(
                "cpu.{} node `{}` must include abi and symbol arguments",
                node.op.instruction, node.name
            ));
        }
        let abi = node.op.args[0].as_str();
        if abi != "nurs" && abi != "c" {
            return Err(format!(
                "cpu.{} node `{}` uses unsupported abi `{abi}`; expected `nurs` or `c`",
                node.op.instruction, node.name
            ));
        }
        let symbol = node.op.args[1].as_str();
        if symbol.trim().is_empty() {
            return Err(format!(
                "cpu.{} node `{}` has an empty symbol name",
                node.op.instruction, node.name
            ));
        }
    }

    for (symbol, signature) in collect_host_ffi_symbols(module)? {
        let expected = if symbol.ends_with("color_bias")
            || symbol.ends_with("speed_curve")
            || symbol.ends_with("radius_curve")
        {
            Some(1usize)
        } else if symbol.ends_with("mix_tick") {
            Some(2usize)
        } else {
            None
        };
        if let Some(expected_arg_count) = expected {
            if signature.arg_count() != expected_arg_count {
                return Err(format!(
                    "host ffi symbol `{symbol}` expects {expected_arg_count} argument(s) but YIR uses {}",
                    signature.arg_count()
                ));
            }
        }
    }

    Ok(())
}

fn collect_host_ffi_symbols(
    module: &YirModule,
) -> Result<BTreeMap<String, HostFfiSignature>, String> {
    let mut out = BTreeMap::new();
    let nodes_by_name = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    for node in &module.nodes {
        if node.op.module != "cpu" || !is_cpu_extern_call_instruction(&node.op.instruction) {
            continue;
        }
        if node.op.args.len() < 2 {
            continue;
        }
        let symbol = node.op.args[1].clone();
        let signature = HostFfiSignature {
            abi: node.op.args[0].clone(),
            return_type: host_ffi_return_type(&node.op.instruction).ok_or_else(|| {
                format!(
                    "cpu.{} node `{}` has no known host FFI return type",
                    node.op.instruction, node.name
                )
            })?,
            arg_types: node
                .op
                .args
                .iter()
                .skip(2)
                .map(|arg| host_ffi_arg_type_for_input(arg, &nodes_by_name))
                .collect(),
        };
        if let Some(existing) = out.insert(symbol.clone(), signature.clone()) {
            if existing != signature {
                return Err(format!(
                    "host ffi symbol `{symbol}` is used with conflicting signatures: {} and {}",
                    existing.render(),
                    signature.render()
                ));
            }
        }
    }
    Ok(out)
}

fn is_cpu_extern_call_instruction(instruction: &str) -> bool {
    matches!(instruction, "extern_call_i64" | "extern_call_i32")
}

fn host_ffi_return_type(instruction: &str) -> Option<HostFfiReturnType> {
    match instruction {
        "extern_call_i64" => Some(HostFfiReturnType::I64),
        "extern_call_i32" => Some(HostFfiReturnType::I32),
        _ => None,
    }
}

fn host_ffi_arg_type_for_input(
    input: &str,
    nodes_by_name: &BTreeMap<&str, &yir_core::Node>,
) -> HostFfiArgType {
    nodes_by_name
        .get(input)
        .map(|node| host_ffi_arg_type_for_node(node))
        .unwrap_or(HostFfiArgType::I64)
}

fn host_ffi_arg_type_for_node(node: &yir_core::Node) -> HostFfiArgType {
    if node.op.module != "cpu" {
        return HostFfiArgType::I64;
    }
    match node.op.instruction.as_str() {
        "const_i32" | "cast_i64_to_i32" | "extern_call_i32" | "call_i32" | "param_i32" => {
            HostFfiArgType::I32
        }
        _ => HostFfiArgType::I64,
    }
}

fn render_host_ffi_symbol_manifest(
    host_ffi_symbols: &BTreeMap<String, HostFfiSignature>,
) -> String {
    host_ffi_symbols
        .iter()
        .map(|(symbol, signature)| format!("{}@{}:{}", symbol, signature.abi, signature.render()))
        .collect::<Vec<_>>()
        .join(";")
}

fn render_host_ffi_symbol_hash_manifest(
    host_ffi_symbols: &BTreeMap<String, HostFfiSignature>,
) -> String {
    host_ffi_symbols
        .iter()
        .map(|(symbol, signature)| format!("{}:{}", symbol, signature.hash(symbol)))
        .collect::<Vec<_>>()
        .join(";")
}

fn host_ffi_footprint_hash(symbol_list: &str, hash_list: &str) -> String {
    fnv1a64_hash(&format!("nuis-ffi-footprint-v1|{symbol_list}|{hash_list}"))
}

fn host_ffi_registry_hash(registry_lines: &[String]) -> String {
    let mut canonical_lines = registry_lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    canonical_lines.sort();
    fnv1a64_hash(&format!(
        "nuis-ffi-registry-v1|{}",
        canonical_lines.join("|")
    ))
}

fn host_ffi_registry_abis(
    registry: &BTreeMap<(String, String), Vec<HostFfiRegistration>>,
) -> String {
    let abis = registry
        .keys()
        .map(|(abi, _)| abi.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if abis.is_empty() {
        "none".to_owned()
    } else {
        abis.join(",")
    }
}

fn fnv1a64_hash(value: &str) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{FFI_SYMBOL_HASH_PREFIX}{hash:016x}")
}

fn append_host_ffi_manifest_entries(
    manifest: &mut Vec<String>,
    host_ffi_symbols: &BTreeMap<String, HostFfiSignature>,
) -> Result<(), String> {
    if host_ffi_symbols.is_empty() {
        manifest.push("host_ffi_symbols=none".to_owned());
        manifest.push("host_ffi_symbol_hashes=none".to_owned());
        manifest.push("host_ffi_footprint_hash=none".to_owned());
        manifest.push("host_ffi_used_symbols=0".to_owned());
        manifest.push("host_ffi_used_abis=none".to_owned());
        manifest.push("host_ffi_registry_source=none".to_owned());
        manifest.push("host_ffi_registry_lines=0".to_owned());
        manifest.push("host_ffi_registry_symbols=0".to_owned());
        manifest.push("host_ffi_registry_abis=none".to_owned());
        manifest.push("host_ffi_registry_hash=none".to_owned());
        return Ok(());
    }

    let symbol_list = render_host_ffi_symbol_manifest(host_ffi_symbols);
    let hash_list = render_host_ffi_symbol_hash_manifest(host_ffi_symbols);
    verify_host_ffi_manifest_lines(&symbol_list, &hash_list)?;
    let registry = default_host_ffi_registry();
    let registry_view = parse_host_ffi_registry_lines(&registry.lines)?;
    let registry_symbols = registry_view.len();
    let registry_abis = host_ffi_registry_abis(&registry_view);
    verify_host_ffi_manifest_against_registry_lines(&symbol_list, &hash_list, &registry.lines)?;
    let used_abis = host_ffi_symbols
        .values()
        .map(|signature| signature.abi.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(",");
    manifest.push(format!("host_ffi_symbols={symbol_list}"));
    manifest.push(format!("host_ffi_symbol_hashes={hash_list}"));
    manifest.push(format!(
        "host_ffi_footprint_hash={}",
        host_ffi_footprint_hash(&symbol_list, &hash_list)
    ));
    manifest.push(format!("host_ffi_used_symbols={}", host_ffi_symbols.len()));
    manifest.push(format!("host_ffi_used_abis={used_abis}"));
    manifest.push(format!("host_ffi_registry_source={}", registry.source));
    manifest.push(format!("host_ffi_registry_lines={}", registry.lines.len()));
    manifest.push(format!("host_ffi_registry_symbols={registry_symbols}"));
    manifest.push(format!("host_ffi_registry_abis={registry_abis}"));
    manifest.push(format!(
        "host_ffi_registry_hash={}",
        host_ffi_registry_hash(&registry.lines)
    ));
    Ok(())
}

fn verify_host_ffi_manifest_lines(symbol_list: &str, hash_list: &str) -> Result<(), String> {
    let symbols = parse_host_ffi_symbol_manifest_entries(symbol_list)?;
    let hashes = parse_host_ffi_hash_manifest_entries(hash_list)?;
    for (symbol, (abi, signature)) in &symbols {
        let Some(actual_hash) = hashes.get(symbol) else {
            return Err(format!(
                "host ffi manifest is missing hash for symbol `{symbol}`"
            ));
        };
        let expected_hash = ffi_symbol_signature_hash(abi, symbol, signature);
        if actual_hash != &expected_hash {
            return Err(format!(
                "host ffi manifest hash mismatch for `{symbol}`: expected {expected_hash}, found {actual_hash}"
            ));
        }
    }
    for symbol in hashes.keys() {
        if !symbols.contains_key(symbol) {
            return Err(format!(
                "host ffi manifest contains hash for undeclared symbol `{symbol}`"
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum HostFfiRegistration {
    Signature(String),
    Hash(String),
}

impl HostFfiRegistration {
    fn allows(&self, signature: &str, hash: &str) -> bool {
        match self {
            Self::Signature(expected) => expected == signature,
            Self::Hash(expected) => expected == hash,
        }
    }
}

fn verify_host_ffi_manifest_against_registry_lines(
    symbol_list: &str,
    hash_list: &str,
    registry_lines: &[String],
) -> Result<(), String> {
    let symbols = parse_host_ffi_symbol_manifest_entries(symbol_list)?;
    let hashes = parse_host_ffi_hash_manifest_entries(hash_list)?;
    let registry = parse_host_ffi_registry_lines(registry_lines)?;
    for (symbol, (abi, signature)) in &symbols {
        let Some(actual_hash) = hashes.get(symbol) else {
            return Err(format!(
                "host ffi manifest is missing hash for symbol `{symbol}`"
            ));
        };
        let allowed = registry
            .get(&(abi.clone(), symbol.clone()))
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        if allowed.is_empty() {
            return Err(format!(
                "host ffi symbol `{symbol}` ABI `{abi}` is not registered by the packer host FFI registry"
            ));
        }
        if !allowed
            .iter()
            .any(|entry| entry.allows(signature, actual_hash))
        {
            return Err(format!(
                "host ffi symbol `{symbol}` ABI `{abi}` signature `{signature}` hash `{actual_hash}` is not allowed by the packer host FFI registry"
            ));
        }
    }
    Ok(())
}

fn parse_host_ffi_registry_lines(
    registry_lines: &[String],
) -> Result<BTreeMap<(String, String), Vec<HostFfiRegistration>>, String> {
    let mut out = BTreeMap::new();
    for raw in registry_lines {
        let Some((abi, caps)) = raw.split_once(':') else {
            return Err(format!(
                "invalid host ffi registry entry `{raw}`; expected `abi:capability[...]`"
            ));
        };
        if abi.trim().is_empty() {
            return Err(format!(
                "invalid host ffi registry entry `{raw}`; ABI is required"
            ));
        }
        for cap in caps.split('|').map(str::trim).filter(|cap| !cap.is_empty()) {
            if let Some(entry) = cap.strip_prefix("ffi_symbol:") {
                let Some((symbol, signature)) = entry.split_once('=') else {
                    return Err(format!(
                        "invalid host ffi registry capability `{cap}`; expected `ffi_symbol:symbol=signature`"
                    ));
                };
                out.entry((abi.to_owned(), symbol.trim().to_owned()))
                    .or_insert_with(Vec::new)
                    .push(HostFfiRegistration::Signature(signature.trim().to_owned()));
            } else if let Some(entry) = cap.strip_prefix("ffi_symbol_hash:") {
                let Some((symbol, hash)) = entry.split_once('=') else {
                    return Err(format!(
                        "invalid host ffi registry capability `{cap}`; expected `ffi_symbol_hash:symbol=fnv1a64:<hex>`"
                    ));
                };
                if !is_ffi_symbol_hash_token(hash.trim()) {
                    return Err(format!(
                        "invalid host ffi registry capability `{cap}`; hash must use `fnv1a64:<hex>`"
                    ));
                }
                out.entry((abi.to_owned(), symbol.trim().to_owned()))
                    .or_insert_with(Vec::new)
                    .push(HostFfiRegistration::Hash(hash.trim().to_owned()));
            }
        }
    }
    Ok(out)
}

fn parse_host_ffi_symbol_manifest_entries(
    value: &str,
) -> Result<BTreeMap<String, (String, String)>, String> {
    let mut out = BTreeMap::new();
    if value == "none" || value.trim().is_empty() {
        return Ok(out);
    }
    for entry in value.split(';') {
        let Some((symbol_abi, signature)) = entry.split_once(':') else {
            return Err(format!(
                "invalid host_ffi_symbols entry `{entry}`; expected `symbol@abi:signature`"
            ));
        };
        let Some((symbol, abi)) = symbol_abi.split_once('@') else {
            return Err(format!(
                "invalid host_ffi_symbols entry `{entry}`; expected `symbol@abi:signature`"
            ));
        };
        if symbol.trim().is_empty() || abi.trim().is_empty() || signature.trim().is_empty() {
            return Err(format!(
                "invalid host_ffi_symbols entry `{entry}`; symbol, abi, and signature are required"
            ));
        }
        out.insert(symbol.to_owned(), (abi.to_owned(), signature.to_owned()));
    }
    Ok(out)
}

fn parse_host_ffi_hash_manifest_entries(value: &str) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    if value == "none" || value.trim().is_empty() {
        return Ok(out);
    }
    for entry in value.split(';') {
        let Some((symbol, payload)) = entry.split_once(':') else {
            return Err(format!(
                "invalid host_ffi_symbol_hashes entry `{entry}`; expected `symbol:fnv1a64:<hex>`"
            ));
        };
        if symbol.trim().is_empty() || payload.trim().is_empty() {
            return Err(format!(
                "invalid host_ffi_symbol_hashes entry `{entry}`; symbol and hash are required"
            ));
        }
        if !is_ffi_symbol_hash_token(payload) {
            return Err(format!(
                "invalid host_ffi_symbol_hashes entry `{entry}`; hash must use `fnv1a64:<hex>`"
            ));
        }
        out.insert(symbol.to_owned(), payload.to_owned());
    }
    Ok(out)
}

fn render_host_ffi_stubs(host_ffi_symbols: &BTreeMap<String, HostFfiSignature>) -> String {
    let mut out = String::new();
    for (symbol, signature) in host_ffi_symbols {
        out.push('\n');
        out.push_str(&render_host_ffi_stub(symbol, signature));
    }
    out
}

fn render_host_ffi_stub(symbol: &str, signature_info: &HostFfiSignature) -> String {
    let arg_count = signature_info.arg_count();
    let mut signature = String::new();
    if arg_count == 0 {
        signature.push_str("void");
    } else {
        for index in 0..arg_count {
            if index > 0 {
                signature.push_str(", ");
            }
            signature.push_str(&format!(
                "{} arg{index}",
                signature_info.arg_types[index].c_type()
            ));
        }
    }

    let body = if symbol.ends_with("color_bias") && arg_count >= 1 {
        "    return host_color_bias(arg0);".to_owned()
    } else if symbol.ends_with("speed_curve") && arg_count >= 1 {
        "    return host_speed_curve(arg0);".to_owned()
    } else if symbol.ends_with("radius_curve") && arg_count >= 1 {
        "    return host_radius_curve(arg0);".to_owned()
    } else if symbol.ends_with("mix_tick") && arg_count >= 2 {
        "    return host_mix_tick(arg0, arg1);".to_owned()
    } else if symbol == "host_network_connect_probe" && arg_count >= 3 {
        "    if (arg0 <= 0 || arg1 <= 0 || arg2 < 0) return 0;\n    return arg0 + arg1 + arg2;"
            .to_owned()
    } else if symbol == "host_network_open_tcp_stream" && arg_count >= 2 {
        "    if (arg0 <= 0 || arg1 < 0) return 0;\n    return arg0 + arg1 + 1;".to_owned()
    } else if symbol == "host_network_open_tcp_listener" && arg_count >= 3 {
        "    if (arg0 <= 0 || arg1 < 0 || arg2 < 0) return 0;\n    return arg0 + arg1 + arg2 + 1;"
            .to_owned()
    } else if symbol == "host_network_open_udp_datagram" && arg_count >= 2 {
        "    if (arg0 <= 0 && arg1 <= 0) return 0;\n    return arg0 + arg1 + 1;".to_owned()
    } else if symbol == "host_network_bind_udp_datagram" && arg_count >= 3 {
        "    if (arg0 <= 0 || arg1 < 0 || arg2 < 0) return 0;\n    return arg0 + arg1 + arg2 + 1;"
            .to_owned()
    } else if symbol == "host_network_accept_owned" && arg_count >= 3 {
        "    if (arg0 <= 0 || arg1 < 0 || arg2 < 0) return 0;\n    return arg0 + arg1 + arg2 + 1;"
            .to_owned()
    } else if symbol == "host_network_close_owned" && arg_count >= 1 {
        "    return arg0 > 0 ? 1 : 0;".to_owned()
    } else if symbol == "host_network_send_owned" && arg_count >= 3 {
        "    if (arg0 <= 0 || arg1 <= 0 || arg2 <= 0) return 0;\n    return arg0 + arg1 + arg2;"
            .to_owned()
    } else if symbol == "host_network_recv_owned" && arg_count >= 3 {
        "    if (arg0 <= 0 || arg1 <= 0 || arg2 <= 0) return 0;\n    return arg0 + arg1 + arg2;"
            .to_owned()
    } else if symbol == "host_network_recv_http_status_owned" && arg_count >= 3 {
        "    if (arg0 <= 0 || arg1 <= 0 || arg2 <= 0) return 0;\n    return 200;".to_owned()
    } else if symbol == "host_network_accept_probe" && arg_count >= 3 {
        "    if (arg0 <= 0 || arg1 < 0 || arg2 < 0) return 0;\n    return arg0 + arg1 + arg2;"
            .to_owned()
    } else if symbol == "host_network_close" && arg_count >= 1 {
        "    return arg0 > 0 ? 1 : 0;".to_owned()
    } else if symbol == "host_network_send_probe" && arg_count >= 3 {
        "    if (arg0 <= 0 || arg1 <= 0 || arg2 <= 0) return 0;\n    return arg0 + arg1 + arg2;"
            .to_owned()
    } else if symbol == "host_network_recv_probe" && arg_count >= 3 {
        "    if (arg0 <= 0 || arg1 <= 0 || arg2 <= 0) return 0;\n    return arg0 + arg1 + arg2;"
            .to_owned()
    } else if arg_count == 0 {
        "    return 0;".to_owned()
    } else if arg_count == 1 {
        "    return arg0;".to_owned()
    } else {
        let mut expr = String::new();
        for index in 0..arg_count {
            if index > 0 {
                expr.push_str(" + ");
            }
            expr.push_str(&format!("arg{index}"));
        }
        format!("    return {expr};")
    };

    format!(
        "{} {symbol}({signature}) {{\n{body}\n}}\n",
        signature_info.return_type.c_type()
    )
}

fn maybe_prepare_embedded_runtime_support(
    module: &YirModule,
    source: &str,
    frame_scale: usize,
) -> Result<Option<RuntimeFrameSupport>, String> {
    let has_tick = module
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "tick_i64");
    if !has_tick {
        return Ok(None);
    }

    let staticlib_path = ensure_runtime_host_staticlib_built()?;
    Ok(Some(RuntimeFrameSupport {
        staticlib_path,
        embedded_module_bytes: bytes_to_c_array(source.as_bytes()),
        frame_scale,
    }))
}

fn ensure_runtime_host_staticlib_built() -> Result<PathBuf, String> {
    let status = Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("yir-runtime-host")
        .status()
        .map_err(|error| format!("failed to invoke cargo build for yir-runtime-host: {error}"))?;
    if !status.success() {
        return Err("cargo build -p yir-runtime-host failed".to_owned());
    }

    let debug_path = PathBuf::from("target/debug/libyir_runtime_host.a");
    if debug_path.exists() {
        return Ok(debug_path);
    }

    Err(format!(
        "expected built runtime host staticlib at `{}`",
        debug_path.display()
    ))
}

fn objc_host_source(
    window_title: &str,
    window_width: usize,
    window_height: usize,
    fabric_worker_core: Option<usize>,
    fabric_table_id: Option<&str>,
    fabric_host_resource: Option<&str>,
    fabric_render_resource: Option<&str>,
    fabric_boot_plan: &str,
    fabric_boot_plan_len: usize,
    embedded_fallback_ppm_bytes: Option<&str>,
    runtime_frame_support: Option<&RuntimeFrameSupport>,
    host_ffi_stubs: &str,
) -> String {
    let affinity_tag = fabric_worker_core
        .map(|core| core.saturating_add(1))
        .unwrap_or(0);
    let affinity_setup = if let Some(core) = fabric_worker_core {
        format!(
            "    nuis_start_fabric_worker({});\n    fprintf(stderr, \"nuis: fabric worker thread requested on core hint {}\\n\");\n",
            affinity_tag, core
        )
    } else {
        String::new()
    };
    let affinity_teardown = if fabric_worker_core.is_some() {
        "    nuis_stop_fabric_worker();\n".to_owned()
    } else {
        String::new()
    };
    let fabric_table_id = fabric_table_id.unwrap_or("none");
    let fabric_host_resource = fabric_host_resource.unwrap_or("none");
    let fabric_render_resource = fabric_render_resource.unwrap_or("none");
    let runtime_mode = runtime_frame_support.is_some();
    let runtime_frame_scale = runtime_frame_support
        .map(|support| support.frame_scale)
        .unwrap_or(4);
    let embedded_runtime_module_bytes = runtime_frame_support
        .map(|support| support.embedded_module_bytes.as_str())
        .unwrap_or("");
    let runtime_support = if runtime_mode {
        format!(
            r#"
typedef struct {{
    unsigned char *ptr;
    uintptr_t len;
}} NuisRenderedBuffer;

extern int32_t nuis_render_embedded_yir_ppm(
    const unsigned char *source_ptr,
    uintptr_t source_len,
    uintptr_t scale,
    NuisRenderedBuffer *out_buffer
);

extern void nuis_rendered_buffer_free(unsigned char *ptr, uintptr_t len);
extern void nuis_rendered_buffer_reset(NuisRenderedBuffer *out_buffer);

static NSData *nuisGenerateRuntimeFrame(NSUInteger tick) {{
    @autoreleasepool {{
        setenv("NUIS_TICK", [[NSString stringWithFormat:@"%lu", (unsigned long)tick] UTF8String], 1);
        static const unsigned char kNuisEmbeddedYirModule[] = {{{embedded_runtime_module_bytes}}};
        NuisRenderedBuffer buffer;
        nuis_rendered_buffer_reset(&buffer);
        int32_t status = nuis_render_embedded_yir_ppm(
            kNuisEmbeddedYirModule,
            sizeof(kNuisEmbeddedYirModule),
            {runtime_frame_scale},
            &buffer
        );
        if (status != 0 || buffer.ptr == NULL || buffer.len == 0) {{
            fprintf(stderr, "nuis: embedded runtime frame generation failed with status %d\n", status);
            if (buffer.ptr != NULL) {{
                nuis_rendered_buffer_free(buffer.ptr, buffer.len);
            }}
            return nil;
        }}
        NSData *data = [NSData dataWithBytes:buffer.ptr length:buffer.len];
        nuis_rendered_buffer_free(buffer.ptr, buffer.len);
        return data;
    }}
}}
"#
        )
    } else {
        String::new()
    };
    let runtime_fields = if runtime_mode {
        r#"
@property(nonatomic, strong) NSImageView *imageView;
@property(nonatomic, assign) NSUInteger tick;
@property(nonatomic, strong) NSTimer *frameTimer;
"#
        .to_owned()
    } else {
        r#"
@property(nonatomic, strong) NSImageView *imageView;
"#
        .to_owned()
    };
    let runtime_bootstrap = if runtime_mode {
        r#"
    self.tick = 1;
"#
        .to_owned()
    } else {
        String::new()
    };
    let runtime_image_assignment = if runtime_mode {
        r#"
    self.imageView = imageView;
    self.frameTimer = [NSTimer scheduledTimerWithTimeInterval:(1.0 / 30.0)
                                                       repeats:YES
                                                         block:^(NSTimer *timer) {
        (void)timer;
        NSData *frameData = nuisGenerateRuntimeFrame(self.tick);
        if (frameData == nil) {
            return;
        }
        NSImage *runtimeImage = nuisImageFromPpmData(frameData);
        if (runtimeImage != nil) {
            [self.imageView setImage:runtimeImage];
            self.tick += 1;
        }
    }];
"#
        .to_owned()
    } else {
        String::new()
    };
    let runtime_teardown = if runtime_mode {
        r#"
    [self.frameTimer invalidate];
    self.frameTimer = nil;
"#
        .to_owned()
    } else {
        String::new()
    };
    let ppm_support = r#"
typedef struct {
    NSUInteger width;
    NSUInteger height;
    const unsigned char *pixels;
    NSUInteger pixel_length;
} NuisPpmView;

static void nuis_skip_ppm_ws_and_comments(const unsigned char *bytes, NSUInteger length, NSUInteger *index) {
    while (*index < length) {
        unsigned char byte = bytes[*index];
        if (byte == '#') {
            while (*index < length && bytes[*index] != '\n') {
                *index += 1;
            }
        } else if (byte == ' ' || byte == '\t' || byte == '\n' || byte == '\r') {
            *index += 1;
        } else {
            break;
        }
    }
}

static NSString *nuis_read_ppm_token(NSData *data, NSUInteger *index) {
    const unsigned char *bytes = data.bytes;
    NSUInteger length = data.length;
    nuis_skip_ppm_ws_and_comments(bytes, length, index);
    NSUInteger start = *index;
    while (*index < length) {
        unsigned char byte = bytes[*index];
        if (byte == ' ' || byte == '\t' || byte == '\n' || byte == '\r' || byte == '#') {
            break;
        }
        *index += 1;
    }
    if (start == *index) {
        return nil;
    }
    return [[NSString alloc] initWithBytes:&bytes[start] length:(*index - start) encoding:NSUTF8StringEncoding];
}

static BOOL nuis_parse_ppm(NSData *data, NuisPpmView *out_view) {
    NSUInteger index = 0;
    NSString *magic = nuis_read_ppm_token(data, &index);
    if (magic == nil || ![magic isEqualToString:@"P6"]) {
        return NO;
    }
    NSString *widthToken = nuis_read_ppm_token(data, &index);
    NSString *heightToken = nuis_read_ppm_token(data, &index);
    NSString *maxToken = nuis_read_ppm_token(data, &index);
    if (widthToken == nil || heightToken == nil || maxToken == nil) {
        return NO;
    }
    NSInteger width = [widthToken integerValue];
    NSInteger height = [heightToken integerValue];
    NSInteger maxValue = [maxToken integerValue];
    if (width <= 0 || height <= 0 || maxValue != 255) {
        return NO;
    }
    const unsigned char *bytes = data.bytes;
    NSUInteger length = data.length;
    if (index < length && (bytes[index] == ' ' || bytes[index] == '\t' || bytes[index] == '\n' || bytes[index] == '\r')) {
        index += 1;
    }
    NSUInteger expected = (NSUInteger)width * (NSUInteger)height * 3;
    if (length < index || (length - index) < expected) {
        return NO;
    }
    out_view->width = (NSUInteger)width;
    out_view->height = (NSUInteger)height;
    out_view->pixels = &bytes[index];
    out_view->pixel_length = expected;
    return YES;
}

static NSImage *nuisImageFromPpmData(NSData *ppmData) {
    NuisPpmView ppm;
    if (!nuis_parse_ppm(ppmData, &ppm)) {
        return nil;
    }
    NSBitmapImageRep *bitmap = [[NSBitmapImageRep alloc]
        initWithBitmapDataPlanes:NULL
                      pixelsWide:(NSInteger)ppm.width
                      pixelsHigh:(NSInteger)ppm.height
                   bitsPerSample:8
                 samplesPerPixel:3
                        hasAlpha:NO
                        isPlanar:NO
                  colorSpaceName:NSDeviceRGBColorSpace
                     bytesPerRow:(NSInteger)(ppm.width * 3)
                    bitsPerPixel:24];
    if (bitmap == nil || bitmap.bitmapData == NULL) {
        return nil;
    }
    memcpy(bitmap.bitmapData, ppm.pixels, ppm.pixel_length);
    NSImage *image = [[NSImage alloc] initWithSize:NSMakeSize(ppm.width, ppm.height)];
    [image addRepresentation:bitmap];
    return image;
}
"#;
    let initial_image_bootstrap = if runtime_mode {
        r#"
    NSData *frameData = nuisGenerateRuntimeFrame(0);
    if (frameData == nil) {
        fprintf(stderr, "failed to generate initial runtime frame\n");
        [NSApp terminate:nil];
        return;
    }
    NSImage *image = nuisImageFromPpmData(frameData);
    if (image == nil) {
        fprintf(stderr, "failed to decode initial runtime frame image\n");
        [NSApp terminate:nil];
        return;
    }
"#
        .to_owned()
    } else {
        let embedded_fallback_ppm_bytes = embedded_fallback_ppm_bytes.unwrap_or("");
        format!(
            r#"
    static const unsigned char kNuisFallbackFrameBytes[] = {{{embedded_fallback_ppm_bytes}}};
    NSData *ppmData = [NSData dataWithBytes:kNuisFallbackFrameBytes length:sizeof(kNuisFallbackFrameBytes)];
    NSImage *image = nuisImageFromPpmData(ppmData);
    if (image == nil) {{
        fprintf(stderr, "failed to load embedded fallback frame image\n");
        [NSApp terminate:nil];
        return;
    }}
"#
        )
    };
    format!(
        r###"#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>
#include <mach/mach.h>
#include <mach/thread_policy.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

extern int64_t nuis_yir_entry(void);

static char* nuis_host_text_slots[4096];
static int64_t nuis_host_text_len = 0;

static int64_t nuis_host_text_register(const char* text) {{
    if (text == NULL) return 0;
    if (nuis_host_text_len >= 4096) return 0;
    size_t size = strlen(text) + 1;
    char* copy = (char*)malloc(size);
    if (copy == NULL) return 0;
    memcpy(copy, text, size);
    nuis_host_text_slots[nuis_host_text_len] = copy;
    nuis_host_text_len += 1;
    return nuis_host_text_len;
}}

int64_t nuis_host_text_lift(const char* text) {{
    return nuis_host_text_register(text);
}}

static const char* nuis_host_text_lookup(int64_t handle) {{
    static char fallback[64];
    if (handle > 0 && handle <= nuis_host_text_len && nuis_host_text_slots[handle - 1] != NULL) {{
        return nuis_host_text_slots[handle - 1];
    }}
    if (handle == 0) return "";
    snprintf(fallback, sizeof(fallback), "%lld", (long long)handle);
    return fallback;
}}

void nuis_debug_print_i64(int64_t value) {{
    printf("%lld\n", (long long)value);
}}

void nuis_debug_print_bool(int32_t value) {{
    printf("%s\n", value ? "true" : "false");
}}

void nuis_debug_print_i32(int32_t value) {{
    printf("%d\n", value);
}}

void nuis_debug_print_f32(float value) {{
    printf("%g\n", value);
}}

void nuis_debug_print_f64(double value) {{
    printf("%g\n", value);
}}

int64_t host_color_bias(int64_t value) {{
    int64_t biased = value + 12;
    if (biased < 0) return 0;
    if (biased > 255) return 255;
    return biased;
}}

int64_t host_speed_curve(int64_t value) {{
    return value * 2 + 3;
}}

int64_t host_radius_curve(int64_t value) {{
    return (value * 3) / 2 + 8;
}}

int64_t host_mix_tick(int64_t base, int64_t tick) {{
    return base + tick;
}}
{host_ffi_stubs}
{runtime_support}
{ppm_support}

static void nuis_apply_fabric_affinity_hint(integer_t tag) {{
    if (tag <= 0) {{
        return;
    }}

    thread_affinity_policy_data_t policy;
    policy.affinity_tag = tag;
    kern_return_t status = thread_policy_set(
        mach_thread_self(),
        THREAD_AFFINITY_POLICY,
        (thread_policy_t)&policy,
        THREAD_AFFINITY_POLICY_COUNT
    );
    if (status != KERN_SUCCESS) {{
        fprintf(stderr, "nuis: failed to apply fabric affinity hint (kern_return_t=%d)\n", status);
    }}
}}

static atomic_bool gNuisFabricWorkerRunning = false;
static pthread_t gNuisFabricWorker;

typedef struct {{
    int kind;
    char action_class[16];
    char action_slot[16];
    char event_name[32];
    char table_id[32];
    char source[32];
    char target[32];
}} NuisFabricEvent;

enum {{
    NUIS_FABRIC_ACTION_UNKNOWN = 0,
    NUIS_FABRIC_ACTION_BIND_CORE = 1,
    NUIS_FABRIC_ACTION_HANDLE_TABLE = 2,
    NUIS_FABRIC_ACTION_OUTPUT_PIPE = 3,
    NUIS_FABRIC_ACTION_INPUT_PIPE = 4,
    NUIS_FABRIC_ACTION_MARKER = 5,
    NUIS_FABRIC_ACTION_COPY_WINDOW = 6,
    NUIS_FABRIC_ACTION_IMMUTABLE_WINDOW = 7,
    NUIS_FABRIC_ACTION_MOVE_VALUE = 8,
}};

typedef struct {{
    int handle_table_count;
    int output_pipe_count;
    int input_pipe_count;
    int marker_count;
    int window_count;
    int move_count;
    int bind_core_count;
}} NuisFabricDispatchState;

static NuisFabricDispatchState gNuisFabricDispatchState = {{0}};
static const NuisFabricEvent kNuisFabricBootPlan[] = {{
{fabric_boot_plan}}};
static const size_t kNuisFabricBootPlanLen = {fabric_boot_plan_len};

static void nuis_dispatch_handle_table(const NuisFabricEvent *event) {{
    gNuisFabricDispatchState.handle_table_count += 1;
    fprintf(
        stderr,
        "nuis: fabric dispatch handle_table class=%s slot=%s table=%s host=%s render=%s\n",
        event->action_class,
        event->action_slot,
        event->table_id,
        event->source,
        event->target
    );
}}

static void nuis_dispatch_output_pipe(const NuisFabricEvent *event) {{
    gNuisFabricDispatchState.output_pipe_count += 1;
    fprintf(
        stderr,
        "nuis: fabric dispatch output_pipe class=%s slot=%s egress=%s via=%s\n",
        event->action_class,
        event->action_slot,
        event->source,
        event->target
    );
}}

static void nuis_dispatch_input_pipe(const NuisFabricEvent *event) {{
    gNuisFabricDispatchState.input_pipe_count += 1;
    fprintf(
        stderr,
        "nuis: fabric dispatch input_pipe class=%s slot=%s ingress=%s into=%s\n",
        event->action_class,
        event->action_slot,
        event->source,
        event->target
    );
}}

static void nuis_dispatch_marker(const NuisFabricEvent *event) {{
    gNuisFabricDispatchState.marker_count += 1;
    fprintf(
        stderr,
        "nuis: fabric dispatch marker class=%s slot=%s event=%s on=%s\n",
        event->action_class,
        event->action_slot,
        event->event_name,
        event->source
    );
}}

static void nuis_dispatch_window(const NuisFabricEvent *event) {{
    gNuisFabricDispatchState.window_count += 1;
    fprintf(
        stderr,
        "nuis: fabric dispatch window class=%s slot=%s transfer=%s -> %s\n",
        event->action_class,
        event->action_slot,
        event->source,
        event->target
    );
}}

static void nuis_dispatch_move(const NuisFabricEvent *event) {{
    gNuisFabricDispatchState.move_count += 1;
    fprintf(
        stderr,
        "nuis: fabric dispatch move class=%s slot=%s value=%s -> %s\n",
        event->action_class,
        event->action_slot,
        event->source,
        event->target
    );
}}

static void nuis_dispatch_bind_core(const NuisFabricEvent *event) {{
    gNuisFabricDispatchState.bind_core_count += 1;
    fprintf(
        stderr,
        "nuis: fabric dispatch bind_core class=%s slot=%s worker=%s\n",
        event->action_class,
        event->action_slot,
        event->source
    );
}}

static void nuis_dispatch_host_signal(
    const char *event_name,
    const char *table_id,
    const char *source,
    const char *target
) {{
    fprintf(
        stderr,
        "nuis: fabric host signal `%s` table=%s source=%s target=%s\n",
        event_name,
        table_id,
        source,
        target
    );
}}

static void nuis_dispatch_fabric_event(const NuisFabricEvent *event) {{
    switch (event->kind) {{
        case NUIS_FABRIC_ACTION_HANDLE_TABLE:
            nuis_dispatch_handle_table(event);
            break;
        case NUIS_FABRIC_ACTION_OUTPUT_PIPE:
            nuis_dispatch_output_pipe(event);
            break;
        case NUIS_FABRIC_ACTION_INPUT_PIPE:
            nuis_dispatch_input_pipe(event);
            break;
        case NUIS_FABRIC_ACTION_MARKER:
            nuis_dispatch_marker(event);
            break;
        case NUIS_FABRIC_ACTION_COPY_WINDOW:
        case NUIS_FABRIC_ACTION_IMMUTABLE_WINDOW:
            nuis_dispatch_window(event);
            break;
        case NUIS_FABRIC_ACTION_MOVE_VALUE:
            nuis_dispatch_move(event);
            break;
        case NUIS_FABRIC_ACTION_BIND_CORE:
            nuis_dispatch_bind_core(event);
            break;
        default:
            fprintf(
                stderr,
                "nuis: fabric dispatch unknown action kind=%d event=%s table=%s source=%s target=%s\n",
                event->kind,
                event->event_name,
                event->table_id,
                event->source,
                event->target
            );
            break;
    }}
}}

static void nuis_run_fabric_boot_plan(void) {{
    for (size_t index = 0; index < kNuisFabricBootPlanLen; ++index) {{
        nuis_dispatch_fabric_event(&kNuisFabricBootPlan[index]);
    }}
}}

static void *nuis_fabric_worker_main(void *arg) {{
    integer_t tag = (integer_t)(intptr_t)arg;
    nuis_apply_fabric_affinity_hint(tag);
    fprintf(stderr, "nuis: fabric worker thread started with affinity tag %d\n", (int)tag);
    nuis_run_fabric_boot_plan();
    while (atomic_load(&gNuisFabricWorkerRunning)) {{
        usleep(1000 * 1000);
    }}
    fprintf(
        stderr,
        "nuis: fabric worker summary handle_table=%d output_pipe=%d input_pipe=%d marker=%d window=%d move=%d bind_core=%d\n",
        gNuisFabricDispatchState.handle_table_count,
        gNuisFabricDispatchState.output_pipe_count,
        gNuisFabricDispatchState.input_pipe_count,
        gNuisFabricDispatchState.marker_count,
        gNuisFabricDispatchState.window_count,
        gNuisFabricDispatchState.move_count,
        gNuisFabricDispatchState.bind_core_count
    );
    return NULL;
}}

static void nuis_start_fabric_worker(integer_t tag) {{
    if (tag <= 0) {{
        return;
    }}
    if (atomic_exchange(&gNuisFabricWorkerRunning, true)) {{
        return;
    }}
    int status = pthread_create(
        &gNuisFabricWorker,
        NULL,
        nuis_fabric_worker_main,
        (void *)(intptr_t)tag
    );
    if (status != 0) {{
        atomic_store(&gNuisFabricWorkerRunning, false);
        fprintf(stderr, "nuis: failed to start fabric worker thread (pthread status=%d)\n", status);
    }}
}}

static void nuis_stop_fabric_worker(void) {{
    if (!atomic_exchange(&gNuisFabricWorkerRunning, false)) {{
        return;
    }}
    pthread_join(gNuisFabricWorker, NULL);
}}

@interface NuisPreviewDelegate : NSObject <NSApplicationDelegate, NSWindowDelegate>
@property(nonatomic, strong) NSWindow *window;
{runtime_fields}
@end

@implementation NuisPreviewDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)notification {{
    (void)notification;
    nuis_dispatch_host_signal("window_boot", "{fabric_table_id}", "{fabric_host_resource}", "{fabric_render_resource}");
{initial_image_bootstrap}

    NSSize imageSize = [image size];
    CGFloat width = MAX(imageSize.width, {window_width}.0);
    CGFloat height = MAX(imageSize.height, {window_height}.0);
    NSRect windowRect = NSMakeRect(0, 0, width, height);
    self.window = [[NSWindow alloc]
        initWithContentRect:windowRect
                  styleMask:(NSWindowStyleMaskTitled |
                             NSWindowStyleMaskClosable |
                             NSWindowStyleMaskMiniaturizable |
                             NSWindowStyleMaskResizable)
                    backing:NSBackingStoreBuffered
                      defer:NO];
    [self.window setDelegate:self];
    [self.window center];
    [self.window setTitle:@"{window_title}"];

    NSImageView *imageView = [[NSImageView alloc] initWithFrame:windowRect];
    [imageView setImage:image];
    [imageView setImageScaling:NSImageScaleAxesIndependently];
    [imageView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
    [self.window setContentView:imageView];
{runtime_bootstrap}
{runtime_image_assignment}
    [self.window makeKeyAndOrderFront:nil];
    [NSApp activateIgnoringOtherApps:YES];
    nuis_dispatch_host_signal("window_ready", "{fabric_table_id}", "{fabric_host_resource}", "{fabric_render_resource}");
}}

- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)sender {{
    (void)sender;
    return YES;
}}

- (void)windowWillClose:(NSNotification *)notification {{
    (void)notification;
    [NSApp terminate:nil];
}}

- (void)applicationWillTerminate:(NSNotification *)notification {{
    (void)notification;
{runtime_teardown}
    nuis_dispatch_host_signal("shutdown", "{fabric_table_id}", "{fabric_render_resource}", "{fabric_host_resource}");
{affinity_teardown}}}

@end

int main(int argc, const char **argv) {{
    (void)argc;
    (void)argv;
    nuis_yir_entry();
{affinity_setup}
    nuis_dispatch_host_signal("boot", "{fabric_table_id}", "{fabric_host_resource}", "{fabric_render_resource}");

    @autoreleasepool {{
        NSApplication *app = [NSApplication sharedApplication];
        [app setActivationPolicy:NSApplicationActivationPolicyRegular];

        NuisPreviewDelegate *delegate = [[NuisPreviewDelegate alloc] init];
        [app setDelegate:delegate];
        [app run];
    }}

    return 0;
}}
"###
    )
}

#[cfg(test)]
mod tests {
    use super::{
        append_host_ffi_manifest_entries, collect_host_ffi_symbols, default_host_ffi_registry,
        host_ffi_registry_abis, host_ffi_registry_hash, is_ffi_symbol_hash_token,
        parse_host_ffi_registry_lines, parse_manifest_string_array, render_host_ffi_stubs,
        render_host_ffi_symbol_hash_manifest, render_host_ffi_symbol_manifest,
        verify_host_ffi_manifest_against_registry_lines, verify_host_ffi_manifest_lines,
        HostFfiArgType, HostFfiReturnType, HostFfiSignature,
    };
    use std::collections::BTreeMap;
    use yir_core::{Node, Operation, YirModule};

    fn cpu_node(name: &str, instruction: &str, args: &[&str]) -> Node {
        Node {
            name: name.to_owned(),
            resource: "cpu.main".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: instruction.to_owned(),
                args: args.iter().map(|arg| (*arg).to_owned()).collect(),
            },
        }
    }

    fn extern_node(name: &str, instruction: &str, symbol: &str, args: &[&str]) -> Node {
        let mut op_args = vec!["c".to_owned(), symbol.to_owned()];
        op_args.extend(args.iter().map(|arg| (*arg).to_owned()));
        cpu_node(
            name,
            instruction,
            &op_args.iter().map(String::as_str).collect::<Vec<_>>(),
        )
    }

    fn host_ffi_signature(
        abi: &str,
        return_type: HostFfiReturnType,
        arg_types: Vec<HostFfiArgType>,
    ) -> HostFfiSignature {
        HostFfiSignature {
            abi: abi.to_owned(),
            return_type,
            arg_types,
        }
    }

    fn i64_host_ffi_signature(abi: &str, arg_count: usize) -> HostFfiSignature {
        host_ffi_signature(
            abi,
            HostFfiReturnType::I64,
            vec![HostFfiArgType::I64; arg_count],
        )
    }

    fn insert_i64_host_ffi_symbol(
        symbols: &mut BTreeMap<String, HostFfiSignature>,
        abi: &str,
        symbol: &str,
        arg_count: usize,
    ) {
        symbols.insert(symbol.to_owned(), i64_host_ffi_signature(abi, arg_count));
    }

    fn registry_lines(lines: &[&str]) -> Vec<String> {
        lines.iter().map(|line| (*line).to_owned()).collect()
    }

    #[test]
    fn host_ffi_stub_tracks_i32_return_and_arg_type() {
        let mut module = YirModule::new("1");
        module.nodes.push(cpu_node("seed", "const_i32", &["7"]));
        module.nodes.push(extern_node(
            "curve",
            "extern_call_i32",
            "host_i32_curve",
            &["seed"],
        ));

        let symbols = collect_host_ffi_symbols(&module).unwrap();
        let signature = symbols.get("host_i32_curve").unwrap();
        assert_eq!(signature.return_type, HostFfiReturnType::I32);
        assert_eq!(signature.arg_types, vec![HostFfiArgType::I32]);
        assert_eq!(signature.render(), "i32(i32)");
        assert_eq!(signature.hash("host_i32_curve"), "fnv1a64:b0042e2b5ee2c2aa");

        let stubs = render_host_ffi_stubs(&symbols);
        assert!(stubs.contains("int32_t host_i32_curve(int32_t arg0)"));
    }

    #[test]
    fn host_ffi_manifest_hashes_are_self_verifying() {
        let mut module = YirModule::new("1");
        module.nodes.push(cpu_node("lhs", "const_i32", &["7"]));
        module.nodes.push(cpu_node("rhs", "const_i32", &["5"]));
        module.nodes.push(extern_node(
            "mix",
            "extern_call_i64",
            "host_i32_mix",
            &["lhs", "rhs"],
        ));

        let symbols = collect_host_ffi_symbols(&module).unwrap();
        let symbol_manifest = render_host_ffi_symbol_manifest(&symbols);
        let hash_manifest = render_host_ffi_symbol_hash_manifest(&symbols);

        assert_eq!(symbol_manifest, "host_i32_mix@c:i64(i32,i32)");
        assert!(hash_manifest.starts_with("host_i32_mix:fnv1a64:"));
        verify_host_ffi_manifest_lines(&symbol_manifest, &hash_manifest).unwrap();
    }

    #[test]
    fn host_ffi_manifest_hashes_reject_drift() {
        let mut module = YirModule::new("1");
        module.nodes.push(cpu_node("seed", "const_i32", &["7"]));
        module.nodes.push(extern_node(
            "curve",
            "extern_call_i32",
            "host_i32_curve",
            &["seed"],
        ));

        let symbols = collect_host_ffi_symbols(&module).unwrap();
        let symbol_manifest = render_host_ffi_symbol_manifest(&symbols);
        let error = verify_host_ffi_manifest_lines(
            &symbol_manifest,
            "host_i32_curve:fnv1a64:0000000000000000",
        )
        .err()
        .expect("mismatched host ffi hash should be rejected");

        assert!(error.contains("host ffi manifest hash mismatch for `host_i32_curve`"));
        assert!(error.contains("fnv1a64:b0042e2b5ee2c2aa"));
    }

    #[test]
    fn host_ffi_manifest_line_verifier_rejects_abi_drift() {
        let error = verify_host_ffi_manifest_lines(
            "host_i32_curve@nurs:i32(i32)",
            "host_i32_curve:fnv1a64:b0042e2b5ee2c2aa",
        )
        .err()
        .expect("manifest hash should bind the ABI as well as the signature");

        assert!(error.contains("host ffi manifest hash mismatch for `host_i32_curve`"));
    }

    #[test]
    fn host_ffi_manifest_registry_verifier_accepts_registered_symbol() {
        let symbol_line = "host_i32_curve@c:i32(i32)";
        let hash_line = "host_i32_curve:fnv1a64:b0042e2b5ee2c2aa";
        verify_host_ffi_manifest_against_registry_lines(
            symbol_line,
            hash_line,
            &registry_lines(&["c:ffi_symbol:host_i32_curve=i32(i32)"]),
        )
        .unwrap();
    }

    #[test]
    fn host_ffi_registry_manifest_parser_preserves_commas_inside_signatures() {
        let values = parse_manifest_string_array(
            r#"abi_capabilities = ["c:ffi_symbol:host_file_open=i64(i64,i64)", "c:ffi_symbol:host_stdout_write=i64(i64)"]"#,
            "abi_capabilities",
        )
        .expect("abi_capabilities array should parse");

        assert_eq!(
            values,
            vec![
                "c:ffi_symbol:host_file_open=i64(i64,i64)".to_owned(),
                "c:ffi_symbol:host_stdout_write=i64(i64)".to_owned(),
            ]
        );
    }

    #[test]
    fn parsed_host_ffi_registry_manifest_lines_are_registry_compatible() {
        let values = parse_manifest_string_array(
            r#"abi_capabilities = ["c:ffi_symbol:host_file_open=i64(i64,i64)|ffi_symbol:host_stdout_write=i64(i64)", "nurs:ffi_symbol:HostMath__speed_curve=i64(i64)"]"#,
            "abi_capabilities",
        )
        .expect("abi_capabilities array should parse");
        let registry = parse_host_ffi_registry_lines(&values).unwrap();

        assert!(registry.contains_key(&("c".to_owned(), "host_file_open".to_owned())));
        assert!(registry.contains_key(&("c".to_owned(), "host_stdout_write".to_owned())));
        assert!(registry.contains_key(&("nurs".to_owned(), "HostMath__speed_curve".to_owned())));
    }

    #[test]
    fn default_host_ffi_registry_loads_cpu_manifest_facades() {
        let registry_lines = default_host_ffi_registry();
        assert!(registry_lines.source.starts_with("cpu-manifest:"));
        let registry = parse_host_ffi_registry_lines(&registry_lines.lines).unwrap();
        assert!(registry.contains_key(&("c".to_owned(), "host_stdout_write".to_owned())));
        assert!(registry.contains_key(&("c".to_owned(), "host_file_open".to_owned())));
        assert!(registry.contains_key(&("c".to_owned(), "host_command_spawn_in".to_owned())));
    }

    #[test]
    fn host_ffi_manifest_entries_record_registry_source() {
        let mut symbols = BTreeMap::new();
        insert_i64_host_ffi_symbol(&mut symbols, "c", "host_stdout_write", 1);

        let mut manifest = Vec::new();
        append_host_ffi_manifest_entries(&mut manifest, &symbols).unwrap();

        assert!(manifest
            .iter()
            .any(|line| line == "host_ffi_symbols=host_stdout_write@c:i64(i64)"));
        assert!(manifest
            .iter()
            .any(|line| line.starts_with("host_ffi_symbol_hashes=host_stdout_write:fnv1a64:")));
        let footprint_hash = manifest
            .iter()
            .find_map(|line| line.strip_prefix("host_ffi_footprint_hash="))
            .expect("footprint hash should be recorded");
        assert!(is_ffi_symbol_hash_token(footprint_hash));
        assert!(manifest
            .iter()
            .any(|line| line == "host_ffi_used_symbols=1"));
        assert!(manifest.iter().any(|line| line == "host_ffi_used_abis=c"));
        assert!(manifest
            .iter()
            .any(|line| line.starts_with("host_ffi_registry_source=cpu-manifest:")));
        assert!(manifest
            .iter()
            .any(|line| line == "host_ffi_registry_abis=c,nurs"));
        let registry_hash = manifest
            .iter()
            .find_map(|line| line.strip_prefix("host_ffi_registry_hash="))
            .expect("registry hash should be recorded");
        assert!(is_ffi_symbol_hash_token(registry_hash));
        let registry_line_count = manifest
            .iter()
            .find_map(|line| line.strip_prefix("host_ffi_registry_lines="))
            .and_then(|value| value.parse::<usize>().ok())
            .expect("registry line count should be recorded");
        let registry_symbol_count = manifest
            .iter()
            .find_map(|line| line.strip_prefix("host_ffi_registry_symbols="))
            .and_then(|value| value.parse::<usize>().ok())
            .expect("registry symbol count should be recorded");
        assert!(registry_line_count > 0);
        assert!(registry_symbol_count > 0);
    }

    #[test]
    fn host_ffi_manifest_entries_record_no_registry_when_unused() {
        let mut manifest = Vec::new();
        append_host_ffi_manifest_entries(&mut manifest, &BTreeMap::new()).unwrap();

        assert_eq!(
            manifest,
            vec![
                "host_ffi_symbols=none".to_owned(),
                "host_ffi_symbol_hashes=none".to_owned(),
                "host_ffi_footprint_hash=none".to_owned(),
                "host_ffi_used_symbols=0".to_owned(),
                "host_ffi_used_abis=none".to_owned(),
                "host_ffi_registry_source=none".to_owned(),
                "host_ffi_registry_lines=0".to_owned(),
                "host_ffi_registry_symbols=0".to_owned(),
                "host_ffi_registry_abis=none".to_owned(),
                "host_ffi_registry_hash=none".to_owned(),
            ]
        );
    }

    #[test]
    fn host_ffi_registry_hash_is_order_stable() {
        let lhs = registry_lines(&[
            "c:ffi_symbol:host_stdout_write=i64(i64)",
            "nurs:ffi_symbol:HostMath__speed_curve=i64(i64)",
        ]);
        let rhs = registry_lines(&[
            "nurs:ffi_symbol:HostMath__speed_curve=i64(i64)",
            "c:ffi_symbol:host_stdout_write=i64(i64)",
        ]);

        assert_eq!(host_ffi_registry_hash(&lhs), host_ffi_registry_hash(&rhs));
        assert!(is_ffi_symbol_hash_token(&host_ffi_registry_hash(&lhs)));
    }

    #[test]
    fn host_ffi_registry_abis_are_sorted_and_deduplicated() {
        let registry = parse_host_ffi_registry_lines(&registry_lines(&[
            "nurs:ffi_symbol:HostMath__speed_curve=i64(i64)",
            "c:ffi_symbol:host_stdout_write=i64(i64)|ffi_symbol:host_stderr_write=i64(i64)",
        ]))
        .unwrap();

        assert_eq!(host_ffi_registry_abis(&registry), "c,nurs");
    }

    #[test]
    fn default_host_ffi_registry_accepts_all_builtin_stub_symbols() {
        let mut symbols = BTreeMap::new();
        for (symbol, arg_count) in [
            ("HostRenderCurves__color_bias", 1),
            ("HostRenderCurves__speed_curve", 1),
            ("HostRenderCurves__radius_curve", 1),
            ("HostRenderCurves__mix_tick", 2),
            ("HostMath__speed_curve", 1),
        ] {
            insert_i64_host_ffi_symbol(&mut symbols, "nurs", symbol, arg_count);
        }
        for (symbol, arg_count) in [
            ("host_speed_curve", 1),
            ("host_hashed_curve", 1),
            ("host_argv_count", 0),
            ("host_monotonic_time_ns", 0),
            ("host_network_connect_probe", 3),
            ("host_network_open_tcp_stream", 2),
            ("host_network_open_tcp_listener", 3),
            ("host_network_open_udp_datagram", 2),
            ("host_network_bind_udp_datagram", 3),
            ("host_network_accept_owned", 3),
            ("host_network_close_owned", 1),
            ("host_network_send_owned", 3),
            ("host_network_recv_owned", 3),
            ("host_network_recv_http_status_owned", 3),
            ("host_network_accept_probe", 3),
            ("host_network_close", 1),
            ("host_network_send_probe", 3),
            ("host_network_recv_probe", 3),
            ("host_stdout_write", 1),
            ("host_file_open", 2),
            ("host_serialize_i64_into", 3),
            ("host_command_spawn_in", 4),
        ] {
            insert_i64_host_ffi_symbol(&mut symbols, "c", symbol, arg_count);
        }
        symbols.insert(
            "host_i32_curve".to_owned(),
            host_ffi_signature("c", HostFfiReturnType::I32, vec![HostFfiArgType::I32]),
        );

        let symbol_manifest = render_host_ffi_symbol_manifest(&symbols);
        let hash_manifest = render_host_ffi_symbol_hash_manifest(&symbols);

        verify_host_ffi_manifest_lines(&symbol_manifest, &hash_manifest).unwrap();
        verify_host_ffi_manifest_against_registry_lines(
            &symbol_manifest,
            &hash_manifest,
            &default_host_ffi_registry().lines,
        )
        .unwrap();
    }

    #[test]
    fn host_ffi_manifest_registry_verifier_rejects_unregistered_symbol() {
        let error = verify_host_ffi_manifest_against_registry_lines(
            "host_unregistered@c:i64(i64)",
            "host_unregistered:fnv1a64:f8a191df2b6270f9",
            &registry_lines(&["c:ffi_symbol:host_i32_curve=i32(i32)"]),
        )
        .err()
        .expect("unregistered host ffi symbol should be rejected");

        assert!(error.contains("host ffi symbol `host_unregistered` ABI `c` is not registered"));
    }

    #[test]
    fn host_ffi_collection_rejects_conflicting_symbol_signatures() {
        let mut module = YirModule::new("1");
        module.nodes.push(extern_node(
            "curve_i64",
            "extern_call_i64",
            "host_curve",
            &["seed"],
        ));
        module.nodes.push(extern_node(
            "curve_i32",
            "extern_call_i32",
            "host_curve",
            &["seed"],
        ));

        let error = collect_host_ffi_symbols(&module)
            .err()
            .expect("same host symbol with different return width should be rejected");
        assert!(error.contains("host ffi symbol `host_curve` is used with conflicting signatures"));
        assert!(error.contains("i64(i64)"));
        assert!(error.contains("i32(i64)"));
    }
}
