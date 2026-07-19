use std::path::Path;

use yir_core::{ModRegistry, YirModule};
use yir_lower_llvm::BranchEffectLlvmEmitterRegistry;

use crate::{registry::NustarPackageManifest, NUSTAR_REGISTRY_ROOT};

struct AssembledCodegenRegistries {
    yir: ModRegistry,
    llvm_branch_effects: BranchEffectLlvmEmitterRegistry,
}

pub(crate) fn emit_module_with_loaded_nustar(
    module: &YirModule,
    package_ids: &[String],
) -> Result<String, String> {
    let manifests = package_ids
        .iter()
        .map(|package_id| {
            crate::registry::load_manifest(Path::new(NUSTAR_REGISTRY_ROOT), package_id)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let registries = assemble_codegen_registries(&manifests)?;
    yir_lower_llvm::emit_module_with_registries(
        module,
        &registries.yir,
        &registries.llvm_branch_effects,
    )
}

fn assemble_codegen_registries(
    manifests: &[NustarPackageManifest],
) -> Result<AssembledCodegenRegistries, String> {
    let mut yir = ModRegistry::new();
    let mut llvm_branch_effects = BranchEffectLlvmEmitterRegistry::new();
    for manifest in manifests {
        if !yir_verify::register_static_nustar_semantics(&manifest.yir_lowering_entry, &mut yir) {
            return Err(format!(
                "loaded nustar package `{}` has no static YIR semantic provider for `{}`",
                manifest.package_id, manifest.yir_lowering_entry
            ));
        }
        yir_lower_llvm::register_static_nustar_branch_effect_emitters(
            &manifest.yir_lowering_entry,
            &mut llvm_branch_effects,
        );
    }
    Ok(AssembledCodegenRegistries {
        yir,
        llvm_branch_effects,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_manifest_installs_its_branch_effect_emitters() {
        let cpu = crate::registry::load_manifest_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), "cpu")
            .unwrap();
        let registries = assemble_codegen_registries(&[cpu]).unwrap();
        assert!(registries.yir.lookup("cpu").is_some());
        assert!(registries.yir.lookup("shader").is_none());
        assert!(registries.llvm_branch_effects.contains("cpu", "load_value"));
        assert!(registries.llvm_branch_effects.contains("cpu", "free"));
    }

    #[test]
    fn unrelated_manifest_does_not_install_cpu_emitters() {
        let shader =
            crate::registry::load_manifest_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), "shader")
                .unwrap();
        let registries = assemble_codegen_registries(&[shader]).unwrap();
        assert!(registries.yir.lookup("shader").is_some());
        assert!(registries.yir.lookup("cpu").is_none());
        assert!(!registries.llvm_branch_effects.contains("cpu", "load_value"));
    }

    #[test]
    fn unknown_semantic_provider_fails_during_assembly() {
        let mut manifest =
            crate::registry::load_manifest_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), "shader")
                .unwrap();
        manifest.yir_lowering_entry = "probe.yir.lowering.v1".to_owned();
        let error = assemble_codegen_registries(&[manifest])
            .err()
            .expect("unknown provider should fail closed");
        assert!(error.contains("has no static YIR semantic provider"));
        assert!(error.contains("probe.yir.lowering.v1"));
    }

    #[test]
    fn every_indexed_nustar_has_a_static_semantic_provider() {
        let manifests =
            crate::registry::load_all_manifests(Path::new(NUSTAR_REGISTRY_ROOT)).unwrap();
        let registries = assemble_codegen_registries(&manifests).unwrap();
        for module in ["cpu", "data", "kernel", "network", "shader"] {
            assert!(
                registries.yir.lookup(module).is_some(),
                "missing semantic registration for {module}"
            );
        }
    }
}
