use std::path::{Path, PathBuf};

use crate::command_helpers::{
    compile_command_input, print_project_context, print_required_nustar_context,
    resolve_compile_input, success_logs_enabled,
};
use crate::inspect_report::{collect_benchmark_inventory, write_compile_doc_index};
use crate::{aot, cache, lowering, pipeline, project, registry, render};

pub(crate) fn run_dump_ast(input: PathBuf) -> Result<(), String> {
    let compiled = compile_command_input(&input)?;
    print_project_context(&compiled.resolved);
    print!("{}", render::render_ast(&compiled.artifacts.ast));

    Ok(())
}

pub(crate) fn run_dump_nir(input: PathBuf) -> Result<(), String> {
    let compiled = compile_command_input(&input)?;
    print_project_context(&compiled.resolved);
    print_required_nustar_context(&compiled.artifacts)?;
    print!("{}", render::render_nir(&compiled.artifacts.nir));

    Ok(())
}

pub(crate) fn run_dump_yir(input: PathBuf) -> Result<(), String> {
    let compiled = compile_command_input(&input)?;
    print_project_context(&compiled.resolved);
    print_required_nustar_context(&compiled.artifacts)?;
    print!("{}", render::render_yir(&compiled.artifacts.yir));

    Ok(())
}

pub(crate) fn run_check(input: PathBuf) -> Result<(), String> {
    let resolved = resolve_compile_input(&input)?;
    let artifacts = resolved.compile()?;
    let benchmarks = collect_benchmark_inventory(&artifacts);
    if success_logs_enabled() {
        println!("checked nuis source: {}", input.display());
        if let Some(project) = &resolved.project {
            println!("project: {}", project::describe_project(project));
        }
        if let Some(plan) = &resolved.project_plan {
            println!(
                "project_plan: {}",
                project::describe_project_compilation_plan(plan)
            );
            println!(
                "project_abi_graph: {}",
                project::render_project_abi_graph_line(&plan.abi_resolution)
            );
        }
        println!(
            "loaded_nustar: {}",
            artifacts
                .loaded_nustar
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!("nir_functions: {}", artifacts.nir.functions.len());
        println!("nir_benchmarks: {}", benchmarks.len());
        if !benchmarks.is_empty() {
            println!(
                "benchmark_symbols: {}",
                benchmarks
                    .iter()
                    .map(|entry| entry.symbol.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        println!("yir_nodes: {}", artifacts.yir.nodes.len());
        println!("yir_edges: {}", artifacts.yir.edges.len());
        println!("llvm_ir_bytes: {}", artifacts.llvm_ir.len());
    }

    Ok(())
}

pub(crate) fn run_compile(
    input: PathBuf,
    output_dir: PathBuf,
    verbose_cache: bool,
    cpu_abi: Option<String>,
    target: Option<String>,
) -> Result<(), String> {
    let resolved = resolve_compile_input(&input)?;
    let cpu_target = aot::resolve_cpu_build_target(
        Path::new("nustar-packages"),
        resolved
            .project_plan
            .as_ref()
            .map(|plan| &plan.abi_resolution),
        cpu_abi.as_deref(),
        target.as_deref(),
    )?;
    let cache_key = cache::compute_compile_cache_key_with_plan(
        &input,
        resolved.project.as_ref(),
        resolved.project_plan.as_ref(),
    )?;
    let cache_hit = cache::lookup_compile_cache(&cache_key)?;
    let compile_fresh = || -> Result<(aot::CompileArtifacts, Vec<String>), String> {
        let artifacts = resolved.compile_with_options(&pipeline::PipelineCompileOptions {
            lowering_target: Some(lowering::LoweringTargetConfig::from_cpu_build_target(
                &cpu_target,
            )),
        })?;
        let written = aot::write_and_link(
            &resolved.effective_input_path,
            &output_dir,
            &artifacts.ast,
            &artifacts.nir,
            &artifacts.yir,
            &artifacts.llvm_ir,
            &cpu_target,
        )?;
        let _ = cache::store_compile_cache(&cache_key, &output_dir)?;
        Ok((written, artifacts.loaded_nustar))
    };
    let (written, loaded_nustar, used_cache_restore) = if let Some(entry) = &cache_hit {
        match cache::restore_compile_cache(entry, &output_dir)
            .and_then(|_| aot::verify_build_manifest(&output_dir.join("nuis.build.manifest.toml")))
        {
            Ok(restored_manifest) => {
                let written = aot::compile_artifacts_for_output_dir_with_packaging_mode(
                    &resolved.effective_input_path,
                    &output_dir,
                    &restored_manifest.packaging_mode,
                )?;
                (written, restored_manifest.loaded_nustar, true)
            }
            Err(_) => {
                let (written, loaded_nustar) = compile_fresh()?;
                (written, loaded_nustar, false)
            }
        }
    } else {
        let (written, loaded_nustar) = compile_fresh()?;
        (written, loaded_nustar, false)
    };
    let project_metadata =
        if let (Some(project), Some(plan)) = (&resolved.project, &resolved.project_plan) {
            Some(project::write_project_metadata(&output_dir, project, plan)?)
        } else {
            None
        };
    let project_text_handle_rewrite = resolved
        .project
        .as_ref()
        .map(project::summarize_project_text_handle_rewrites)
        .transpose()?;
    let doc_index = write_compile_doc_index(&input, &output_dir)?;
    let build_manifest = aot::write_build_manifest(
        &output_dir,
        &written,
        &aot::BuildManifestContext {
            input_path: input.display().to_string(),
            output_dir: output_dir.display().to_string(),
            loaded_nustar: loaded_nustar.clone(),
            compile_cache: Some(aot::BuildManifestCacheInfo {
                status: if used_cache_restore {
                    "hit".to_owned()
                } else {
                    "miss".to_owned()
                },
                key: cache_key.key.clone(),
                root: cache_key.root.display().to_string(),
            }),
            project: resolved
                .project
                .as_ref()
                .zip(resolved.project_plan.as_ref())
                .map(|(project, plan)| aot::BuildManifestProjectInfo {
                    name: project.manifest.name.clone(),
                    abi_mode: if plan.abi_resolution.explicit {
                        "explicit".to_owned()
                    } else {
                        "auto-recommended".to_owned()
                    },
                    abi_graph_summary: Some(project::render_project_abi_graph_line(
                        &plan.abi_resolution,
                    )),
                    abi_entries: plan
                        .abi_resolution
                        .requirements
                        .iter()
                        .map(|item| (item.domain.clone(), item.abi.clone()))
                        .collect::<Vec<_>>(),
                    plan_summary: Some(project::describe_project_compilation_plan(plan)),
                    effective_input: Some(plan.effective_input_path.display().to_string()),
                    text_handle_rewrite_helper_hits: project_text_handle_rewrite
                        .map(|summary| summary.helper_hits)
                        .unwrap_or(0),
                    text_handle_rewrite_local_hits: project_text_handle_rewrite
                        .map(|summary| summary.local_hits)
                        .unwrap_or(0),
                    manifest_copy_path: project_metadata
                        .as_ref()
                        .map(|item| item.manifest_copy_path.clone()),
                    plan_index_path: project_metadata
                        .as_ref()
                        .map(|item| item.plan_index_path.clone()),
                    organization_index_path: project_metadata
                        .as_ref()
                        .map(|item| item.organization_index_path.clone()),
                    exchange_index_path: project_metadata
                        .as_ref()
                        .map(|item| item.exchange_index_path.clone()),
                    modules_index_path: project_metadata
                        .as_ref()
                        .map(|item| item.modules_index_path.clone()),
                    docs_index_path: project_metadata
                        .as_ref()
                        .map(|item| item.docs_index_path.clone()),
                    docs_module_count: project_metadata
                        .as_ref()
                        .map(|item| item.docs_summary.modules)
                        .unwrap_or(0),
                    docs_documented_module_count: project_metadata
                        .as_ref()
                        .map(|item| item.docs_summary.documented_modules)
                        .unwrap_or(0),
                    docs_documented_item_count: project_metadata
                        .as_ref()
                        .map(|item| item.docs_summary.documented_items)
                        .unwrap_or(0),
                    imports_index_path: project_metadata
                        .as_ref()
                        .map(|item| item.imports_index_path.clone()),
                    imports_library_count: project_metadata
                        .as_ref()
                        .map(|item| item.imports_summary.libraries)
                        .unwrap_or(0),
                    imports_visible_library_count: project_metadata
                        .as_ref()
                        .map(|item| item.imports_summary.visible_libraries)
                        .unwrap_or(0),
                    imports_visible_module_count: project_metadata
                        .as_ref()
                        .map(|item| item.imports_summary.visible_modules)
                        .unwrap_or(0),
                    imports_documented_visible_module_count: project_metadata
                        .as_ref()
                        .map(|item| item.imports_summary.documented_visible_modules)
                        .unwrap_or(0),
                    imports_documented_visible_item_count: project_metadata
                        .as_ref()
                        .map(|item| item.imports_summary.documented_visible_items)
                        .unwrap_or(0),
                    galaxy_index_path: project_metadata
                        .as_ref()
                        .map(|item| item.galaxy_index_path.clone()),
                    galaxy_count: project_metadata
                        .as_ref()
                        .map(|item| item.galaxy_summary.galaxies)
                        .unwrap_or(0),
                    galaxy_documented_count: project_metadata
                        .as_ref()
                        .map(|item| item.galaxy_summary.documented_galaxies)
                        .unwrap_or(0),
                    galaxy_documented_library_module_count: project_metadata
                        .as_ref()
                        .map(|item| item.galaxy_summary.documented_library_modules)
                        .unwrap_or(0),
                    galaxy_documented_item_count: project_metadata
                        .as_ref()
                        .map(|item| item.galaxy_summary.documented_items)
                        .unwrap_or(0),
                    links_index_path: project_metadata
                        .as_ref()
                        .map(|item| item.links_index_path.clone()),
                    packet_index_path: project_metadata
                        .as_ref()
                        .map(|item| item.packet_index_path.clone()),
                    host_ffi_index_path: project_metadata
                        .as_ref()
                        .map(|item| item.host_ffi_index_path.clone()),
                    abi_index_path: project_metadata
                        .as_ref()
                        .map(|item| item.abi_index_path.clone()),
                }),
            doc_index: Some(doc_index.clone()),
            cpu_target: cpu_target.clone(),
        },
    )?;
    if success_logs_enabled() {
        println!("compiled nuis source: {}", input.display());
        println!(
            "compile_cache: {} ({})",
            if used_cache_restore { "hit" } else { "miss" },
            cache_key.key
        );
        println!("compile_cache_inputs: {}", cache_key.input_labels.len());
        if verbose_cache {
            for label in &cache_key.input_labels {
                println!("  compile_cache_input: {}", label);
            }
        }
        if let Some(project) = &resolved.project {
            println!("project: {}", project::describe_project(project));
            if let Ok(graph) = project::describe_project_abi_graph(project) {
                println!("project_abi_graph: {}", graph);
            }
        }
        if let Some(plan) = &resolved.project_plan {
            println!(
                "project_plan: {}",
                project::describe_project_compilation_plan(plan)
            );
            println!(
                "project_abi_graph: {}",
                project::render_project_abi_graph_line(&plan.abi_resolution)
            );
        }
        println!(
            "loaded_nustar: {}",
            loaded_nustar
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!("cpu_target_abi: {}", cpu_target.abi);
        println!(
            "cpu_target_machine: {}-{}",
            cpu_target.machine_arch, cpu_target.machine_os
        );
        println!("cpu_target_clang: {}", cpu_target.clang_target);
        println!(
            "cpu_target_cross: {}",
            if cpu_target.cross_compile {
                "true"
            } else {
                "false"
            }
        );
        if let Some(plan) = &resolved.project_plan {
            for item in &plan.abi_resolution.requirements {
                println!("abi: {}={}", item.domain, item.abi);
                if let Ok(manifest) =
                    registry::load_manifest_for_domain(Path::new("nustar-packages"), &item.domain)
                {
                    if let Ok(target) = registry::registered_abi_target(&manifest, &item.abi) {
                        println!(
                            "  abi_target_machine: {}-{}",
                            target.machine_arch, target.machine_os
                        );
                        println!("  abi_target_object: {}", target.object_format);
                        println!("  abi_target_calling: {}", target.calling_abi);
                        println!("  abi_target_clang: {}", target.clang_target);
                        if let Some(backend) = target.backend_family {
                            println!("  abi_target_backend: {}", backend);
                        }
                        if let Some(vendor) = target.vendor {
                            println!("  abi_target_vendor: {}", vendor);
                        }
                        if let Some(device_class) = target.device_class {
                            println!("  abi_target_device: {}", device_class);
                        }
                        println!(
                            "  abi_target_host_adaptive: {}",
                            if target.host_adaptive {
                                "true"
                            } else {
                                "false"
                            }
                        );
                    }
                }
            }
        }
        println!("ast: {}", written.ast_path);
        println!("nir: {}", written.nir_path);
        println!("yir: {}", written.yir_path);
        println!("llvm_ir: {}", written.llvm_ir_path);
        println!("packaging_mode: {}", written.packaging_mode);
        println!("binary: {}", written.binary_path);
        println!(
            "compiled_artifact: {}",
            output_dir.join("nuis.compiled.artifact").display()
        );
        println!("doc_index: {}", doc_index.path);
        println!("doc_index_modules: {}", doc_index.module_count);
        println!(
            "doc_index_documented_items: {}",
            doc_index.documented_item_count
        );
        println!("build_manifest: {}", build_manifest);
        if let Some(metadata) = &project_metadata {
            println!("project_manifest: {}", metadata.manifest_copy_path);
            println!("project_plan_index: {}", metadata.plan_index_path);
            println!("project_organization: {}", metadata.organization_index_path);
            println!("project_exchange: {}", metadata.exchange_index_path);
            println!("project_modules: {}", metadata.modules_index_path);
            println!("project_docs: {}", metadata.docs_index_path);
            println!("project_imports: {}", metadata.imports_index_path);
            println!("project_galaxy: {}", metadata.galaxy_index_path);
            println!("project_links: {}", metadata.links_index_path);
            println!("project_packet: {}", metadata.packet_index_path);
            println!("project_host_ffi: {}", metadata.host_ffi_index_path);
            println!("project_abi: {}", metadata.abi_index_path);
        }
    }

    Ok(())
}
