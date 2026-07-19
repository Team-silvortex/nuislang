use crate::{register_cpu_branch_effect_llvm_emitters, BranchEffectLlvmEmitterRegistry};

pub type RegisterNustarLlvmEmitters = fn(&mut BranchEffectLlvmEmitterRegistry);

#[derive(Clone, Copy)]
pub struct StaticNustarLlvmEmitterProvider {
    pub lowering_entry: &'static str,
    pub register_branch_effect_emitters: RegisterNustarLlvmEmitters,
}

const STATIC_NUSTAR_LLVM_EMITTER_PROVIDERS: &[StaticNustarLlvmEmitterProvider] = &[
    StaticNustarLlvmEmitterProvider {
        lowering_entry: "cpu.yir.lowering.v1",
        register_branch_effect_emitters: register_cpu_branch_effect_llvm_emitters,
    },
    StaticNustarLlvmEmitterProvider {
        lowering_entry: "cpu.aarch64.yir.lowering.v1",
        register_branch_effect_emitters: register_cpu_branch_effect_llvm_emitters,
    },
];

pub fn static_nustar_llvm_emitter_providers() -> &'static [StaticNustarLlvmEmitterProvider] {
    STATIC_NUSTAR_LLVM_EMITTER_PROVIDERS
}

pub fn register_static_nustar_branch_effect_emitters(
    lowering_entry: &str,
    registry: &mut BranchEffectLlvmEmitterRegistry,
) -> bool {
    let Some(provider) = STATIC_NUSTAR_LLVM_EMITTER_PROVIDERS
        .iter()
        .find(|provider| provider.lowering_entry == lowering_entry)
    else {
        return false;
    };
    (provider.register_branch_effect_emitters)(registry);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_provider_aliases_install_the_same_emitters() {
        for entry in ["cpu.yir.lowering.v1", "cpu.aarch64.yir.lowering.v1"] {
            let mut registry = BranchEffectLlvmEmitterRegistry::new();
            assert!(register_static_nustar_branch_effect_emitters(
                entry,
                &mut registry
            ));
            assert!(registry.contains("cpu", "load_value"));
            assert!(registry.contains("cpu", "free"));
        }
    }

    #[test]
    fn unknown_provider_does_not_mutate_the_registry() {
        let mut registry = BranchEffectLlvmEmitterRegistry::new();
        assert!(!register_static_nustar_branch_effect_emitters(
            "probe.yir.lowering.v1",
            &mut registry
        ));
        assert!(!registry.contains("cpu", "load_value"));
    }
}
