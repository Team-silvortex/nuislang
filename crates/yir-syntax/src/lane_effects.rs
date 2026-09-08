use std::collections::{BTreeMap, BTreeSet};

use yir_core::{Edge, EdgeKind, YirModule};

pub(super) fn synthesize_lane_effect_edges(module: &mut YirModule) {
    let Some(order) = dependency_order(module) else {
        // Preserve invalid input for verification instead of attempting to repair it.
        return;
    };
    let mut effects = module
        .edges
        .iter()
        .filter(|edge| edge.kind == EdgeKind::Effect)
        .map(|edge| (edge.from.clone(), edge.to.clone()))
        .collect::<BTreeSet<_>>();
    let mut previous_by_queue = BTreeMap::new();
    // Serial lane sugar extends the existing partial order, never textual backedges.
    for index in order {
        let node = &module.nodes[index];
        let Some(lane) = module.node_lanes.get(&node.name) else {
            continue;
        };
        let queue = (node.resource.as_str(), lane.as_str());
        if let Some(previous) = previous_by_queue.insert(queue, node.name.as_str()) {
            let pair = (previous.to_owned(), node.name.clone());
            if effects.insert(pair.clone()) {
                module.edges.push(Edge {
                    kind: EdgeKind::Effect,
                    from: pair.0,
                    to: pair.1,
                });
            }
        }
    }
}

fn dependency_order(module: &YirModule) -> Option<Vec<usize>> {
    let indices = module
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.name.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let mut outgoing = vec![Vec::new(); module.nodes.len()];
    let mut indegree = vec![0usize; module.nodes.len()];
    for edge in &module.edges {
        let (Some(&from), Some(&to)) = (
            indices.get(edge.from.as_str()),
            indices.get(edge.to.as_str()),
        ) else {
            return None;
        };
        outgoing[from].push(to);
        indegree[to] += 1;
    }
    // Source position is only a tie-breaker among currently dependency-ready nodes.
    let mut ready = indegree
        .iter()
        .enumerate()
        .filter_map(|(index, &degree)| (degree == 0).then_some(index))
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(module.nodes.len());
    while let Some(index) = ready.pop_first() {
        order.push(index);
        for &next in &outgoing[index] {
            indegree[next] -= 1;
            if indegree[next] == 0 {
                ready.insert(next);
            }
        }
    }
    (order.len() == module.nodes.len()).then_some(order)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_effect(module: &YirModule, from: &str, to: &str) -> bool {
        module
            .edges
            .iter()
            .any(|edge| edge.kind == EdgeKind::Effect && edge.from == from && edge.to == to)
    }

    #[test]
    fn lane_sugar_respects_transitive_dependencies_in_every_declaration_order() {
        for names in [
            ["a", "b", "c"],
            ["a", "c", "b"],
            ["b", "a", "c"],
            ["b", "c", "a"],
            ["c", "a", "b"],
            ["c", "b", "a"],
        ] {
            let nodes = names
                .map(|name| format!("cpu.const_i64 {name} cpu0@work 1\n"))
                .concat();
            let module = crate::parse_module(&format!(
                "resource cpu0 cpu.arm64\n{nodes}edge dep a b\nedge lifetime b c\n"
            ))
            .unwrap();
            assert!(dependency_order(&module).is_some(), "{names:?}");
            assert!(has_effect(&module, "a", "b"));
            assert!(has_effect(&module, "b", "c"));
            assert!(!has_effect(&module, "c", "a"));
        }
    }

    #[test]
    fn lane_sugar_respects_paths_through_other_lanes() {
        let module = crate::parse_module(
            "resource cpu0 cpu.arm64\n\
             cpu.const_i64 c cpu0@left 1\n\
             cpu.const_i64 a cpu0@left 1\n\
             cpu.const_i64 b cpu0@right 1\n\
             edge dep a b\nedge dep b c\n",
        )
        .unwrap();
        assert!(has_effect(&module, "a", "c"));
        assert!(!has_effect(&module, "c", "a"));
        assert!(dependency_order(&module).is_some());
    }

    #[test]
    fn added_lane_edges_cannot_form_a_cycle_together() {
        let module = crate::parse_module(
            "resource cpu0 cpu.arm64\n\
             cpu.const_i64 b cpu0@left 1\n\
             cpu.const_i64 d cpu0@right 1\n\
             cpu.const_i64 a cpu0@left 1\n\
             cpu.const_i64 c cpu0@right 1\n\
             edge dep a d\nedge dep c b\n",
        )
        .unwrap();
        assert!(has_effect(&module, "a", "b"));
        assert!(has_effect(&module, "d", "c"));
        assert!(dependency_order(&module).is_some());
    }

    #[test]
    fn invalid_explicit_graphs_are_not_repaired_or_given_more_edges() {
        for edges in ["edge dep a b\nedge dep b a\n", "edge dep missing a\n"] {
            let module = crate::parse_module(&format!(
                "resource cpu0 cpu.arm64\ncpu.const_i64 a cpu0@work 1\n\
                 cpu.const_i64 b cpu0@work 2\n{edges}"
            ))
            .unwrap();
            assert!(dependency_order(&module).is_none());
            assert!(!module
                .edges
                .iter()
                .any(|edge| edge.kind == EdgeKind::Effect));
        }
    }
}
