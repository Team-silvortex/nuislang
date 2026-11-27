//! Shared pure helper validation, independent of a callback registration.

use std::collections::{BTreeMap, BTreeSet};
use yir_core::{Node, YirFunction, YirModule};

pub(crate) fn target<'a>(module: &'a YirModule, node: &Node) -> Result<&'a YirFunction, String> {
    let functions = module
        .functions
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect();
    let nodes = module.nodes.iter().map(|n| (n.name.as_str(), n)).collect();
    super::calls::target(node, &functions, &nodes)?
        .ok_or_else(|| "expected a native value helper call".to_owned())
}

pub(crate) fn validate(
    module: &YirModule,
    function: &YirFunction,
    contract_metadata: &BTreeSet<String>,
) -> Result<(), String> {
    let registry = yir_verify::default_registry();
    let mut incoming = BTreeMap::<&str, Vec<&str>>::new();
    for edge in &module.edges {
        // The caller has independently checked these immutable metadata edges.
        // They stay in the emitted module; only executable ingress is checked here.
        if contract_metadata.contains(&edge.from) {
            continue;
        }
        incoming.entry(&edge.to).or_default().push(&edge.from);
    }
    super::function::FunctionAdmission {
        module,
        nodes: module.nodes.iter().map(|n| (n.name.as_str(), n)).collect(),
        resources: module
            .resources
            .iter()
            .map(|r| (r.name.as_str(), r))
            .collect(),
        incoming,
        profile_reads: Default::default(),
        registry: &registry,
        literal_prints: None,
    }
    .validate(function, false, None)
}
