mod aot;
mod cli;
mod codegen_wasm;
mod engine;
mod errors;
mod frontend;
mod lowering;
mod pipeline;
mod registry;
mod render;

use std::env;

use cli::CommandKind;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let frontend = frontend::frontend_name();
    let backend = codegen_wasm::backend_name();
    let engine = engine::NuiscEngine {
        version: "0.44.b-draft",
        profile: "aot",
    };
    let command = cli::parse_args(env::args().skip(1))?;

    match command {
        CommandKind::Status => {
            let manifests = registry::discover(std::path::Path::new("nustar-packages"))?;
            println!(
                "nuisc compiler prototype: topology-first scheduler frontend ({frontend} -> {backend}, yir={}, profile={}, registered_nustar={})",
                engine.version,
                engine.profile,
                manifests.len()
            );
            for manifest in manifests {
                println!(
                    "  - {} [{}] via {} -> {}",
                    manifest.package_id, manifest.domain_family, manifest.frontend, manifest.entry_crate
                );
            }
        }
        CommandKind::Registry => {
            let manifests = registry::discover(std::path::Path::new("nustar-packages"))?;
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
                println!("  resource_families: {}", manifest.resource_families.join(", "));
                println!("  lowering_targets: {}", manifest.lowering_targets.join(", "));
                println!("  ops: {}", manifest.ops.join(", "));
            }
        }
        CommandKind::DumpNir { input } => {
            let artifacts = pipeline::compile_source_path(&input)?;
            print!("{}", render::render_nir(&artifacts.nir));
        }
        CommandKind::DumpYir { input } => {
            let artifacts = pipeline::compile_source_path(&input)?;
            print!("{}", render::render_yir(&artifacts.yir));
        }
        CommandKind::Compile { input, output_dir } => {
            let artifacts = pipeline::compile_source_path(&input)?;
            let written = aot::write_and_link(
                &input,
                &output_dir,
                &artifacts.nir,
                &artifacts.yir,
                &artifacts.llvm_ir,
            )?;
            println!("compiled nuis source: {}", input.display());
            println!("nir: {}", written.nir_path);
            println!("yir: {}", written.yir_path);
            println!("llvm_ir: {}", written.llvm_ir_path);
            println!("binary: {}", written.binary_path);
        }
    }

    Ok(())
}
