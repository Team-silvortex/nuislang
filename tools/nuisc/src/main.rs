mod codegen_wasm;
mod errors;
mod ir;
mod parser;
mod registry;

use std::{env, path::Path};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let frontend = parser::frontend_name();
    let backend = codegen_wasm::backend_name();
    let engine = ir::NuiscEngine {
        version: "0.44.b-draft",
        profile: "aot",
    };
    let command = env::args().nth(1).unwrap_or_else(|| "status".to_owned());

    match command.as_str() {
        "status" => {
            let manifests = registry::discover(Path::new("nustar-packages"))?;
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
        "registry" => {
            let manifests = registry::discover(Path::new("nustar-packages"))?;
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
        other => {
            return Err(format!(
                "unknown nuisc command `{other}`; expected `status` or `registry`"
            ));
        }
    }

    Ok(())
}
