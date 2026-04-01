pub mod aot;
pub mod cli;
pub mod codegen_wasm;
pub mod engine;
pub mod errors;
pub mod frontend;
pub mod lowering;
pub mod nustar_binary;
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
                println!("  schema: {}", manifest.manifest_schema);
                println!("  domain: {}", manifest.domain_family);
                println!("  frontend: {}", manifest.frontend);
                println!("  crate: {}", manifest.entry_crate);
                println!("  binary_extension: {}", manifest.binary_extension);
                println!("  package_layout: {}", manifest.package_layout);
                println!("  machine_abi_policy: {}", manifest.machine_abi_policy);
                println!(
                    "  implementation_kinds: {}",
                    manifest.implementation_kinds.join(", ")
                );
                println!("  loader_entry: {}", manifest.loader_entry);
                println!("  loader_abi: {}", manifest.loader_abi);
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
        CommandKind::PackNustar { package_id, output } => {
            let manifest = registry::load_manifest(Path::new("nustar-packages"), &package_id)?;
            let blob = format!(
                "nustar_impl_stub\npackage={}\nfrontend={}\nentry_crate={}\n",
                manifest.package_id, manifest.frontend, manifest.entry_crate
            )
            .into_bytes();
            let binary = nustar_binary::default_binary(manifest, blob);
            nustar_binary::write_to_path(&output, &binary)?;
            println!("packed nustar binary: {}", output.display());
            println!("  package: {}", binary.manifest.package_id);
            println!("  extension: .nustar");
            println!("  format_version: {}", binary.format_version);
            println!("  abi: {}", binary.abi_tag);
            println!("  machine_arch: {}", binary.machine_arch);
            println!("  machine_os: {}", binary.machine_os);
            println!("  object_format: {}", binary.object_format);
            println!("  calling_abi: {}", binary.calling_abi);
            println!("  format: {}", binary.implementation_format);
            println!("  checksum: {}", binary.implementation_checksum);
            println!("  blob_bytes: {}", binary.implementation_blob.len());
        }
        CommandKind::InspectNustar { input } => {
            let binary = nustar_binary::read_from_path(&input)?;
            println!("nustar binary: {}", input.display());
            println!("  package: {}", binary.manifest.package_id);
            println!("  domain: {}", binary.manifest.domain_family);
            println!("  frontend: {}", binary.manifest.frontend);
            println!("  crate: {}", binary.manifest.entry_crate);
            println!("  loader_abi: {}", binary.manifest.loader_abi);
            println!("  loader_entry: {}", binary.manifest.loader_entry);
            println!("  format_version: {}", binary.format_version);
            println!("  abi: {}", binary.abi_tag);
            println!("  machine_arch: {}", binary.machine_arch);
            println!("  machine_os: {}", binary.machine_os);
            println!("  object_format: {}", binary.object_format);
            println!("  calling_abi: {}", binary.calling_abi);
            println!(
                "  machine_abi_compatible_with_host: {}",
                nustar_binary::machine_abi_matches_host(&binary)
            );
            println!("  format: {}", binary.implementation_format);
            println!("  checksum: {}", binary.implementation_checksum);
            println!("  profiles: {}", binary.manifest.profiles.join(", "));
            println!(
                "  resource_families: {}",
                binary.manifest.resource_families.join(", ")
            );
            println!(
                "  lowering_targets: {}",
                binary.manifest.lowering_targets.join(", ")
            );
            println!("  ops: {}", binary.manifest.ops.join(", "));
            println!("  blob_bytes: {}", binary.implementation_blob.len());
        }
        CommandKind::LoaderContract { package_id } => {
            let manifest = registry::load_manifest(Path::new("nustar-packages"), &package_id)?;
            let binary = nustar_binary::default_binary(manifest, Vec::new());
            println!("loader contract: {}", binary.manifest.package_id);
            println!("  loader_abi: {}", binary.manifest.loader_abi);
            println!("  loader_entry: {}", binary.manifest.loader_entry);
            println!(
                "  canonical_entry_signature: {}",
                nustar_binary::CANONICAL_ENTRY_SIGNATURE
            );
            println!(
                "  canonical_host_abi_struct: {}",
                nustar_binary::CANONICAL_HOST_ABI_STRUCT
            );
            println!(
                "  canonical_result_struct: {}",
                nustar_binary::CANONICAL_RESULT_STRUCT
            );
            println!(
                "  loader_status_convention: {}",
                nustar_binary::CANONICAL_LOADER_STATUS_CONVENTION
            );
            println!(
                "  machine_abi_policy: {}",
                binary.manifest.machine_abi_policy
            );
            println!("  host_machine_arch: {}", binary.machine_arch);
            println!("  host_machine_os: {}", binary.machine_os);
            println!("  host_object_format: {}", binary.object_format);
            println!("  host_calling_abi: {}", binary.calling_abi);
            for contract in nustar_binary::implementation_contracts(&binary) {
                println!("  kind: {}", contract.kind);
                println!("    loader_abi: {}", contract.loader_abi);
                println!("    entry_symbol: {}", contract.entry_symbol);
                println!("    entry_signature: {}", contract.entry_signature);
                println!("    host_abi_struct: {}", contract.host_abi_struct);
                println!("    result_struct: {}", contract.result_struct);
                println!("    status_convention: {}", contract.status_convention);
                println!("    machine_abi_policy: {}", contract.machine_abi_policy);
                println!("    notes: {}", contract.notes);
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
