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
    let registries = assemble_codegen_registries(&manifests);
    yir_lower_llvm::emit_module_with_registries(
        module,
        &registries.yir,
        &registries.llvm_branch_effects,
    )
}

fn assemble_codegen_registries(manifests: &[NustarPackageManifest]) -> AssembledCodegenRegistries {
    let mut llvm_branch_effects = BranchEffectLlvmEmitterRegistry::new();
    for manifest in manifests {
        yir_lower_llvm::register_static_nustar_branch_effect_emitters(
            &manifest.yir_lowering_entry,
            &mut llvm_branch_effects,
        );
    }
    AssembledCodegenRegistries {
        yir: yir_verify::default_registry(),
        llvm_branch_effects,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_manifest_installs_its_branch_effect_emitters() {
        let cpu = crate::registry::load_manifest_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), "cpu")
            .unwrap();
        let registries = assemble_codegen_registries(&[cpu]);
        assert!(registries.llvm_branch_effects.contains("cpu", "load_value"));
        assert!(registries.llvm_branch_effects.contains("cpu", "free"));
    }

    #[test]
    fn unrelated_manifest_does_not_install_cpu_emitters() {
        let shader =
            crate::registry::load_manifest_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), "shader")
                .unwrap();
        let registries = assemble_codegen_registries(&[shader]);
        assert!(!registries.llvm_branch_effects.contains("cpu", "load_value"));
    }
}
