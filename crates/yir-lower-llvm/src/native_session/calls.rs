use std::collections::{BTreeMap, BTreeSet};

use super::ScalarKind;
use yir_core::{Node, YirFunction, YirFunctionRole, YirValueOwnership};

pub(super) const MAX_FUNCTIONS: usize = 64;
pub(super) const MAX_TOTAL_NODES: usize = 16384;
const MAX_CALL_DEPTH: usize = 32;

pub(super) fn target<'a>(
    node: &Node,
    functions: &BTreeMap<&str, &'a YirFunction>,
) -> Result<Option<&'a YirFunction>, String> {
    let Some(kind) = node.op.instruction.strip_prefix("call_") else {
        return Ok(None);
    };
    ScalarKind::parse(kind)?;
    let callee = node
        .op
        .args
        .first()
        .ok_or("native scalar call missing target")?;
    let function = functions.get(callee.as_str()).copied().ok_or_else(|| {
        format!(
            "native scalar call `{}` references unknown helper `{callee}`",
            node.name
        )
    })?;
    let result = function.result.as_ref();
    if function.role != YirFunctionRole::Helper
        || !result
            .is_some_and(|result| result.ty == kind && result.ownership == YirValueOwnership::Value)
        || function.parameters.len() != node.op.args.len() - 1
    {
        return Err(format!(
            "native scalar call `{}` helper signature drift",
            node.name
        ));
    }
    Ok(Some(function))
}

// Walk the graph from leaves, not with an unbounded recursive host-stack traversal.
pub(super) fn validate_graph(graph: &BTreeMap<String, BTreeSet<String>>) -> Result<(), String> {
    let mut remaining = BTreeMap::new();
    let mut parents = BTreeMap::<&str, Vec<&str>>::new();
    let mut depths = BTreeMap::new();
    let mut ready = BTreeSet::new();
    for (name, callees) in graph {
        remaining.insert(name.as_str(), callees.len());
        depths.insert(name.as_str(), 1);
        if callees.is_empty() {
            ready.insert(name.as_str());
        }
        for callee in callees {
            parents.entry(callee).or_default().push(name);
        }
    }
    let mut visited = 0;
    while let Some(name) = ready.pop_first() {
        visited += 1;
        let depth = depths[name];
        if depth > MAX_CALL_DEPTH {
            return Err("native scalar bridge exceeds its call-depth bound".to_owned());
        }
        for parent in parents.get(name).into_iter().flatten() {
            let parent_depth = depths.get_mut(parent).expect("selected caller");
            *parent_depth = (*parent_depth).max(depth + 1);
            let count = remaining.get_mut(parent).expect("selected caller");
            *count -= 1;
            if *count == 0 {
                ready.insert(*parent);
            }
        }
    }
    if visited != graph.len() {
        return Err("native scalar bridge rejects recursive helper call cycles".to_owned());
    }
    Ok(())
}
