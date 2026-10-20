use crate::backend_artifacts::{
    emit_kernel_backend_artifacts, emit_shader_backend_artifacts, render_shader_backend_validation,
};
use crate::host_compile::{c_shim_source, compile_native_appkit_binary, compile_native_binary};
use crate::host_ffi::{
    append_host_ffi_manifest_entries, collect_host_ffi_symbols, validate_host_ffi_symbols,
};
use crate::host_ffi_stub::render_host_ffi_stubs;
use crate::host_frames::{
    cleanup_stale_fallback_frame, maybe_emit_prerendered_frame,
    maybe_prepare_embedded_runtime_support,
};
use crate::host_objc::{objc_host_source, ObjcHostSourceSpec};
use crate::host_plan::{
    extract_cpu_window_spec, extract_fabric_boot_plan, extract_primary_fabric_binding,
    render_fabric_boot_plan,
};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};
use yir_lower_contract::analyze_kernel_lowering;
use yir_lower_contract::analyze_shader_lowering;
use yir_lower_llvm::emit_module;
use yir_verify::verify_module;

mod backend_artifacts;
mod backend_descriptor;
mod host_application_script;
mod host_compile;
mod host_ffi;
mod host_ffi_stub;
mod host_frames;
mod host_native_session;
mod host_objc;
mod host_plan;
mod host_runtime_frame;
mod host_runtime_library;
mod host_source;
mod host_text_runtime;
mod host_window_session;
mod shader_codegen;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let input = args.next().ok_or_else(|| {
        "usage: yir-pack-aot <module.yir> <output-dir> [frame-scale] [--headless | --native-session ID]".to_owned()
    })?;
    let output_dir = args.next().ok_or_else(|| {
        "usage: yir-pack-aot <module.yir> <output-dir> [frame-scale] [--headless | --native-session ID]".to_owned()
    })?;
    let (frame_scale, headless, native) = host_application_script::parse_options(args)?;

    let source =
        fs::read_to_string(&input).map_err(|error| format!("failed to read `{input}`: {error}"))?;
    let module = yir_syntax::parse_module(&source)?;
    verify_module(&module)?;
    if let Some(id) = native {
        return host_native_session::build(
            &module,
            &source,
            Path::new(&input),
            Path::new(&output_dir),
            &id,
        );
    }
    if headless && module.application_sessions.is_empty() {
        return Err("headless packaging requires a registered application session".to_owned());
    }
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

    let mut manifest = vec![format!("module={input}")];
    if !headless {
        let llvm_ir = emit_module(&module)?;
        fs::write(&ll_path, llvm_ir)
            .map_err(|error| format!("failed to write `{}`: {error}", ll_path.display()))?;
        fs::write(&shim_path, c_shim_source(&host_ffi_symbols))
            .map_err(|error| format!("failed to write `{}`: {error}", shim_path.display()))?;
        manifest.push(format!("llvm_ir={}", ll_path.display()));
    }
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
        manifest.push(if headless {
            "fabric_worker_core_mode=not-applied-by-headless-host".to_owned()
        } else {
            "fabric_worker_core_mode=macos_affinity_worker_thread".to_owned()
        });
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
    let frame_bundle =
        if headless || (runtime_frame_support.is_some() && !shader_requires_prerender_fallback) {
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

    if headless {
        manifest.push("render_mode=application_session_observation".to_owned());
        manifest.push(format!(
            "application_yir_fnv1a64={}",
            yir_core::provider_runtime_ipc::hash_bytes(source.as_bytes())
        ));
    } else if runtime_frame_support.is_some() {
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

    if headless {
        let runtime = runtime_frame_support
            .as_ref()
            .ok_or("headless packaging requires embedded runtime support")?;
        let host = output_dir.join(format!("{stem}_headless.c"));
        manifest.extend(host_application_script::build(
            &host,
            &runtime.staticlib_path,
            &exe_path,
            &runtime.embedded_module_bytes,
        )?);
    } else if use_window_host {
        let fabric_boot_plan = extract_fabric_boot_plan(
            &module,
            primary_fabric_binding.as_ref(),
            shader_contract
                .fabric_core_bindings
                .first()
                .map(|binding| binding.resource.as_str()),
        );
        let rendered_fabric_boot_plan = render_fabric_boot_plan(&fabric_boot_plan);
        fs::write(
            &host_path,
            objc_host_source(ObjcHostSourceSpec {
                window_title: window_spec
                    .as_ref()
                    .map(|spec| spec.title.as_str())
                    .unwrap_or(stem),
                window_width: window_spec.as_ref().map(|spec| spec.width).unwrap_or(640),
                window_height: window_spec.as_ref().map(|spec| spec.height).unwrap_or(480),
                fabric_worker_core: shader_contract
                    .fabric_core_bindings
                    .first()
                    .map(|binding| binding.core_index),
                fabric_table_id: primary_fabric_binding
                    .as_ref()
                    .map(|binding| binding.table_id.as_str()),
                fabric_host_resource: primary_fabric_binding
                    .as_ref()
                    .map(|binding| binding.host_resource.as_str()),
                fabric_render_resource: primary_fabric_binding
                    .as_ref()
                    .map(|binding| binding.render_resource.as_str()),
                fabric_boot_plan: &rendered_fabric_boot_plan,
                fabric_boot_plan_len: fabric_boot_plan.len(),
                embedded_fallback_ppm_bytes: frame_bundle
                    .as_ref()
                    .map(|bundle| bundle.embedded_ppm_bytes.as_str()),
                runtime_frame_support: runtime_frame_support.as_ref(),
                host_ffi_stubs: &host_ffi_stub_source,
            }),
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
                "window_session_contract={}",
                host_window_session::CONTRACT
            ));
            manifest.push(format!(
                "window_parent_contract={}",
                host_window_session::PARENT_CONTRACT
            ));
            manifest.push(format!(
                "window_cancellation_contract={}",
                host_window_session::CANCELLATION_CONTRACT
            ));
            manifest.push(format!(
                "application_provider_drain_contract={}",
                host_window_session::PROVIDER_DRAIN_CONTRACT
            ));
            manifest.push(format!(
                "frame_export_contract={}",
                host_runtime_frame::FRAME_EXPORT_CONTRACT
            ));
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

#[cfg(test)]
use crate::host_ffi::{
    default_host_ffi_registry, host_ffi_registry_abis, host_ffi_registry_hash,
    parse_host_ffi_registry_lines, parse_manifest_string_array,
    render_host_ffi_symbol_hash_manifest, render_host_ffi_symbol_manifest,
    verify_host_ffi_manifest_against_registry, verify_host_ffi_manifest_against_registry_lines,
    verify_host_ffi_manifest_lines, HostFfiArgType, HostFfiReturnType, HostFfiSignature,
};
#[cfg(test)]
use yir_core::ffi::is_ffi_symbol_hash_token;

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;

#[cfg(test)]
mod packaging_sources_tests;
