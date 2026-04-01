pub mod aot;
pub mod cli;
pub mod codegen_wasm;
pub mod engine;
pub mod errors;
pub mod frontend;
pub mod lowering;
pub mod pipeline;
pub mod registry;
pub mod render;

use std::path::Path;

pub use cli::CommandKind;

pub fn run(command: CommandKind) -> Result<(), String> {
    let frontend = frontend::frontend_name();
    let backend = codegen_wasm::backend_name();
    let engine = engine::default_engine();

    match command {
        CommandKind::Status => {
            let index = registry::load_index(Path::new("nustar-packages"))?;
            println!(
                "nuisc compiler core: topology-first scheduler frontend ({frontend} -> {backend}, yir={}, profile={}, indexed_nustar={})",
                engine.version,
                engine.profile,
                index.len()
            );
            for entry in index {
                println!(
                    "  - {} [{}] -> {}",
                    entry.package_id,
                    entry.domain_family,
                    registry::manifest_path(Path::new("nustar-packages"), &entry).display()
                );
            }
        }
        CommandKind::Registry => {
            let manifests = registry::load_all_manifests(Path::new("nustar-packages"))?;
            if manifests.is_empty() {
                let placeholder_error = errors::NuiscError {
                    message: "no nustar packages discovered",
                };
                return Err(placeholder_error.message.to_owned());
            }
            for manifest in manifests {
                println!("package: {}", manifest.package_id);
                println!("  domain: {}", manifest.domain_family);
                println!("  frontend: {}", manifest.frontend);
                println!("  crate: {}", manifest.entry_crate);
                println!("  profiles: {}", manifest.profiles.join(", "));
                println!(
                    "  resource_families: {}",
                    manifest.resource_families.join(", ")
                );
                println!(
                    "  lowering_targets: {}",
                    manifest.lowering_targets.join(", ")
                );
                println!("  ops: {}", manifest.ops.join(", "));
            }
        }
        CommandKind::Bindings { input } => {
            let artifacts = pipeline::compile_source_path(&input)?;
            let plan = registry::plan_bindings(Path::new("nustar-packages"), &artifacts.yir)?;
            println!("binding plan for: {}", input.display());
            for binding in plan.bindings {
                println!("package: {}", binding.package_id);
                println!("  domain: {}", binding.domain_family);
                println!("  frontend: {}", binding.frontend);
                println!("  crate: {}", binding.entry_crate);
                println!(
                    "  matched_resources: {}",
                    if binding.matched_resources.is_empty() {
                        "<none>".to_owned()
                    } else {
                        binding.matched_resources.join(", ")
                    }
                );
                println!("  matched_ops: {}", binding.matched_ops.join(", "));
                if !binding.undeclared_ops.is_empty() {
                    println!("  undeclared_ops: {}", binding.undeclared_ops.join(", "));
                }
            }
        }
        CommandKind::DumpNir { input } => {
            let artifacts = pipeline::compile_source_path(&input)?;
            let required =
                registry::load_required_manifests(Path::new("nustar-packages"), &artifacts.yir)?;
            eprintln!(
                "nuisc: lazily loaded nustar = {}",
                required
                    .iter()
                    .map(|manifest| manifest.package_id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            print!("{}", render::render_nir(&artifacts.nir));
        }
        CommandKind::DumpYir { input } => {
            let artifacts = pipeline::compile_source_path(&input)?;
            let required =
                registry::load_required_manifests(Path::new("nustar-packages"), &artifacts.yir)?;
            eprintln!(
                "nuisc: lazily loaded nustar = {}",
                required
                    .iter()
                    .map(|manifest| manifest.package_id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            print!("{}", render::render_yir(&artifacts.yir));
        }
        CommandKind::Check { input } => {
            let artifacts = pipeline::compile_source_path(&input)?;
            let required =
                registry::load_required_manifests(Path::new("nustar-packages"), &artifacts.yir)?;
            println!("checked nuis source: {}", input.display());
            println!(
                "loaded_nustar: {}",
                required
                    .iter()
                    .map(|manifest| manifest.package_id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!("nir_functions: {}", artifacts.nir.functions.len());
            println!("yir_nodes: {}", artifacts.yir.nodes.len());
            println!("yir_edges: {}", artifacts.yir.edges.len());
            println!("llvm_ir_bytes: {}", artifacts.llvm_ir.len());
        }
        CommandKind::Compile { input, output_dir } => {
            let artifacts = pipeline::compile_source_path(&input)?;
            let required =
                registry::load_required_manifests(Path::new("nustar-packages"), &artifacts.yir)?;
            let written = aot::write_and_link(
                &input,
                &output_dir,
                &artifacts.nir,
                &artifacts.yir,
                &artifacts.llvm_ir,
            )?;
            println!("compiled nuis source: {}", input.display());
            println!(
                "loaded_nustar: {}",
                required
                    .iter()
                    .map(|manifest| manifest.package_id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!("nir: {}", written.nir_path);
            println!("yir: {}", written.yir_path);
            println!("llvm_ir: {}", written.llvm_ir_path);
            println!("binary: {}", written.binary_path);
        }
    }

    Ok(())
}
