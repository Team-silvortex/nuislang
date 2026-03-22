use std::collections::{BTreeMap, BTreeSet};

use yir_core::{FabricMod, ModRegistry, Resource, YirModule};

pub fn default_registry() -> ModRegistry {
    let mut registry = ModRegistry::new();
    registry.register(FabricMod);
    registry.register(yir_domain_cpu::CpuMod);
    registry.register(yir_domain_shader::ShaderMod);
    registry
}

pub fn verify_module(module: &YirModule) -> Result<(), String> {
    let registry = default_registry();
    verify_module_with_registry(module, &registry)
}

pub fn verify_module_with_registry(
    module: &YirModule,
    registry: &ModRegistry,
) -> Result<(), String> {
    if module.version.is_empty() {
        return Err("module version must not be empty".to_owned());
    }

    let mut resources = BTreeMap::<String, &Resource>::new();
    for resource in &module.resources {
        if resources.insert(resource.name.clone(), resource).is_some() {
            return Err(format!("duplicate resource `{}`", resource.name));
        }
    }

    let mut produced = BTreeSet::new();

    for node in &module.nodes {
        let resource = resources.get(&node.resource).copied().ok_or_else(|| {
            format!(
                "node `{}` references unknown resource `{}`",
                node.name, node.resource
            )
        })?;

        let module_impl = registry.lookup(&node.op.module).ok_or_else(|| {
            format!(
                "node `{}` references unregistered mod `{}`",
                node.name, node.op.module
            )
        })?;

        let semantics = module_impl.describe(node, resource)?;

        for dependency in semantics.dependencies {
            if dependency == node.name {
                return Err(format!("node `{}` may not depend on itself", node.name));
            }

            if !produced.contains(dependency.as_str()) {
                return Err(format!(
                    "node `{}` depends on `{dependency}` before it is defined",
                    node.name
                ));
            }
        }

        if !produced.insert(node.name.as_str()) {
            return Err(format!("duplicate node `{}`", node.name));
        }
    }

    Ok(())
}
