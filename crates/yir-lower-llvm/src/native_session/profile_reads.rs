use std::collections::{BTreeMap, BTreeSet};

use super::function::FunctionAdmission;
use yir_core::{Edge, EdgeKind, Node, YirModule};

/// Only explicit operand reads are importable. A node name is not a grant
/// to erase an ordering, lifetime, exchange or initialization edge.
#[derive(Default)]
pub(super) struct ProfileReads<'a>(BTreeMap<&'a str, BTreeSet<&'a str>>);

impl<'a> ProfileReads<'a> {
    pub(super) fn inspect(admission: &FunctionAdmission<'a>) -> Self {
        let module = admission.module;
        let mut literals = module
            .nodes
            .iter()
            .filter(|n| {
                n.name.starts_with("project_profile_shader_")
                    && module
                        .node_lanes
                        .get(&n.name)
                        .is_some_and(|l| l == "profile")
                    && n.op.module == "cpu"
                    && n.op.instruction == "const_i64"
                    && matches!(n.op.args.as_slice(), [v] if v.parse::<i64>().is_ok())
                    && admission
                        .resources
                        .get(n.resource.as_str())
                        .is_some_and(|r| r.kind.is_family("cpu"))
                    && !admission.incoming.contains_key(n.name.as_str())
            })
            .map(|n| n.name.as_str())
            .collect::<BTreeSet<_>>();
        if literals.is_empty() {
            return Self::default();
        }
        for f in &module.functions {
            for name in &f.body_nodes {
                literals.remove(name.as_str());
            }
        }

        let mut reads = Self::default();
        let mut ordered = Vec::new();
        for edge in &module.edges {
            if !literals.contains(edge.from.as_str()) {
                continue;
            }
            if edge.kind != EdgeKind::Dep {
                ordered.push((edge.to.as_str(), edge.from.as_str()));
                continue;
            }
            let src = admission.nodes[edge.from.as_str()];
            let dst = admission.nodes[edge.to.as_str()];
            if src.resource == dst.resource && numeric_use(dst, &edge.from) {
                reads.0.entry(&edge.to).or_default().insert(&edge.from);
            }
        }
        for (dst, src) in ordered {
            if let Some(reads) = reads.0.get_mut(dst) {
                reads.remove(src);
            }
        }
        reads.0.retain(|_, sources| !sources.is_empty());
        reads
    }

    pub(super) fn permits(&self, dst: &str, src: &str) -> bool {
        self.0.get(dst).is_some_and(|sources| sources.contains(src))
    }

    pub(super) fn localize(
        &self,
        selected: &mut YirModule,
        original: &BTreeMap<&str, &Node>,
    ) -> Result<(), String> {
        if self.0.is_empty() {
            return Ok(());
        }
        let indices = selected
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.name.clone(), i))
            .collect::<BTreeMap<_, _>>();
        let mut fresh = 0;
        for f in &mut selected.functions {
            // Canonical order keeps generated names independent of body enumeration.
            let uses: BTreeMap<_, _> = f
                .body_nodes
                .iter()
                .filter_map(|dst| self.0.get(dst.as_str()).map(|sources| (dst, sources)))
                .collect();
            let sources: BTreeSet<_> = uses.values().flat_map(|s| s.iter().copied()).collect();
            if f.body_nodes.len() + sources.len() > super::function::MAX_FUNCTION_NODES
                || selected.nodes.len() + sources.len() > super::calls::MAX_TOTAL_NODES
            {
                return Err(
                    "native scalar bridge exceeds node bounds after profile literal import".into(),
                );
            }
            let mut literals = BTreeMap::new();
            let lane = format!("fn:{}", f.name);
            for src in sources {
                let name = loop {
                    let name = format!("__nuis_profile_i64_{fresh}");
                    fresh += 1;
                    if !original.contains_key(name.as_str()) {
                        break name;
                    }
                };
                let mut literal = original[src].clone();
                literal.name = name.clone();
                selected.nodes.push(literal);
                selected.node_lanes.insert(name.clone(), lane.clone());
                literals.insert(src, name);
            }
            for (dst, sources) in uses {
                for &src in sources {
                    let local = &literals[src];
                    for operand in &mut selected.nodes[indices[dst]].op.args {
                        if operand == src {
                            operand.clone_from(local);
                        }
                    }
                    selected.edges.push(Edge {
                        kind: EdgeKind::Dep,
                        from: local.clone(),
                        to: dst.clone(),
                    });
                }
            }
            f.body_nodes.extend(literals.into_values());
        }
        Ok(())
    }
}

// These operations have exactly two value operands: no callee, field label,
// destination, or embedded action can be mistaken for a read.
fn numeric_use(node: &Node, source: &str) -> bool {
    node.op.module == "cpu"
        && matches!(
            node.op.instruction.as_str(),
            "add"
                | "sub"
                | "mul"
                | "add_i64"
                | "sub_i64"
                | "mul_i64"
                | "eq"
                | "ne"
                | "lt"
                | "le"
                | "gt"
                | "ge"
                | "eq_i64"
                | "ne_i64"
                | "lt_i64"
                | "le_i64"
                | "gt_i64"
                | "ge_i64"
        )
        && matches!(node.op.args.as_slice(), [a,b] if a == source || b == source)
}
