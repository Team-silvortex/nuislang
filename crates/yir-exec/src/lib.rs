use std::collections::BTreeMap;

use yir_core::{ExecutionState, Resource, Value, YirModule};
use yir_verify::{default_registry, verify_module_with_registry};

#[derive(Debug, Default)]
pub struct ExecutionTrace {
    pub events: Vec<String>,
    pub values: BTreeMap<String, Value>,
}

pub fn execute_module(module: &YirModule) -> Result<ExecutionTrace, String> {
    let registry = default_registry();
    verify_module_with_registry(module, &registry)?;

    let resources = module
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource))
        .collect::<BTreeMap<String, &Resource>>();

    let mut state = ExecutionState::default();

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

        let value = module_impl.execute(node, resource, &mut state)?;
        state.values.insert(node.name.clone(), value);
    }

    Ok(ExecutionTrace {
        events: state.events,
        values: state.values,
    })
}
