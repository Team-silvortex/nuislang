use yir_core::{DataMod, LegacyFabricMod, ModRegistry};

pub type RegisterNustarSemantics = fn(&mut ModRegistry);

#[derive(Clone, Copy)]
pub struct StaticNustarSemanticProvider {
    pub lowering_entry: &'static str,
    pub register_mods: RegisterNustarSemantics,
}

const STATIC_NUSTAR_SEMANTIC_PROVIDERS: &[StaticNustarSemanticProvider] = &[
    StaticNustarSemanticProvider {
        lowering_entry: "cpu.yir.lowering.v1",
        register_mods: register_cpu,
    },
    StaticNustarSemanticProvider {
        lowering_entry: "cpu.aarch64.yir.lowering.v1",
        register_mods: register_cpu,
    },
    StaticNustarSemanticProvider {
        lowering_entry: "data.yir.lowering.v1",
        register_mods: register_data,
    },
    StaticNustarSemanticProvider {
        lowering_entry: "kernel.yir.lowering.v1",
        register_mods: register_kernel,
    },
    StaticNustarSemanticProvider {
        lowering_entry: "network.yir.lowering.v1",
        register_mods: register_network,
    },
    StaticNustarSemanticProvider {
        lowering_entry: "shader.yir.lowering.v1",
        register_mods: register_shader,
    },
];

pub fn static_nustar_semantic_providers() -> &'static [StaticNustarSemanticProvider] {
    STATIC_NUSTAR_SEMANTIC_PROVIDERS
}

pub fn register_static_nustar_semantics(lowering_entry: &str, registry: &mut ModRegistry) -> bool {
    let Some(provider) = STATIC_NUSTAR_SEMANTIC_PROVIDERS
        .iter()
        .find(|provider| provider.lowering_entry == lowering_entry)
    else {
        return false;
    };
    (provider.register_mods)(registry);
    true
}

fn register_cpu(registry: &mut ModRegistry) {
    registry.register(yir_domain_cpu::CpuMod);
}

fn register_data(registry: &mut ModRegistry) {
    registry.register(DataMod);
    registry.register(LegacyFabricMod);
}

fn register_kernel(registry: &mut ModRegistry) {
    registry.register(yir_domain_kernel::KernelMod);
}

fn register_network(registry: &mut ModRegistry) {
    registry.register(yir_domain_network::NetworkMod);
}

fn register_shader(registry: &mut ModRegistry) {
    registry.register(yir_domain_shader::ShaderMod);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_provider_does_not_enable_unloaded_domains() {
        let mut registry = ModRegistry::new();
        assert!(register_static_nustar_semantics(
            "cpu.yir.lowering.v1",
            &mut registry
        ));
        assert!(registry.lookup("cpu").is_some());
        assert!(registry.lookup("shader").is_none());
    }

    #[test]
    fn data_provider_installs_its_legacy_alias() {
        let mut registry = ModRegistry::new();
        assert!(register_static_nustar_semantics(
            "data.yir.lowering.v1",
            &mut registry
        ));
        assert!(registry.lookup("data").is_some());
        assert!(registry.lookup("fabric").is_some());
    }

    #[test]
    fn unknown_provider_does_not_mutate_the_registry() {
        let mut registry = ModRegistry::new();
        assert!(!register_static_nustar_semantics(
            "probe.yir.lowering.v1",
            &mut registry
        ));
        assert!(registry.lookup("probe").is_none());
    }
}
