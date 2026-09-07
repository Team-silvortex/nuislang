use std::collections::{BTreeMap, BTreeSet};
use yir_core::{ModRegistry, YirModule};

/// Conservative function-root closure: all callback body nodes, all incoming
/// graph edges (including ordering edges), described value dependencies, and
/// every statically named function operand. No operation/backend allowlist is
/// used here. Unrelated globals are not implicit roots in this explicit mode.
pub(super) fn rooted_nodes(
    module: &YirModule,
    registry: &ModRegistry,
    roots: &[&str],
) -> Result<(BTreeSet<String>, BTreeSet<String>), String> {
    let functions = module
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect::<BTreeMap<_, _>>();
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let resources = module
        .resources
        .iter()
        .map(|resource| (resource.name.as_str(), resource))
        .collect::<BTreeMap<_, _>>();
    let mut incoming: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for edge in &module.edges {
        incoming.entry(&edge.to).or_default().push(&edge.from);
    }
    let mut reachable = BTreeSet::new();
    let mut called = BTreeSet::new();
    let mut pending = Vec::new();
    for root in roots {
        let function = functions
            .get(root)
            .ok_or_else(|| format!("unknown session root `{root}`"))?;
        called.insert((*root).to_owned());
        pending.extend(function.body_nodes.iter().cloned());
    }
    while let Some(name) = pending.pop() {
        if !reachable.insert(name.clone()) {
            continue;
        }
        let node = nodes
            .get(name.as_str())
            .ok_or_else(|| format!("unknown session dependency `{name}`"))?;
        if let Some(edges) = incoming.get(name.as_str()) {
            pending.extend(edges.iter().map(|name| (*name).to_owned()));
        }
        let resource = resources
            .get(node.resource.as_str())
            .ok_or("unknown session resource")?;
        let semantics = registry
            .lookup(&node.op.module)
            .ok_or("unregistered session module")?
            .describe(node, resource)?;
        pending.extend(semantics.dependencies);
        for operand in &node.op.args {
            if let Some(function) = functions.get(operand.as_str()) {
                if called.insert(operand.clone()) {
                    pending.extend(function.body_nodes.iter().cloned());
                }
            }
        }
    }
    Ok((reachable, called))
}
