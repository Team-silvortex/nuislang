//! Discover packet-backed draw sites on verified YIR. The native region
//! projects the packet and invokes a scoped submitter at the original draw;
//! Data effects and ownership remain in their original lanes.
use std::collections::{BTreeMap, BTreeSet};
use yir_core::{EdgeKind, Node, YirModule};

mod llvm;

pub fn emit_llvm(module: &YirModule) -> Option<(usize, String)> {
    let sites: Vec<_> = sites(module).into_values().collect();
    (!sites.is_empty()).then(|| (sites.len(), llvm::emit(&sites)))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Site {
    pub ordinal: u32,
    pub slot: u32,
    pub width: u32,
}

pub fn sites(module: &YirModule) -> BTreeMap<String, Site> {
    if !module.nodes.iter().any(|n| {
        n.op.module == "shader" && n.op.instruction == "draw_instanced" && n.op.args.len() == 5
    }) {
        return BTreeMap::new();
    }
    let nodes: BTreeMap<_, _> = module.nodes.iter().map(|n| (n.name.as_str(), n)).collect();
    let ordered_inputs: BTreeSet<_> = module
        .edges
        .iter()
        .filter(|e| {
            e.kind == EdgeKind::Effect
                && nodes
                    .get(e.to.as_str())
                    .is_some_and(|n| n.op.module == "data" && n.op.instruction == "input_pipe")
        })
        .map(|e| (e.from.as_str(), e.to.as_str()))
        .collect();
    module
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, draw)| {
            if draw.op.module != "shader"
                || draw.op.instruction != "draw_instanced"
                || draw.op.args.len() != 5
            {
                return None;
            }
            let lane = module.node_lanes.get(&draw.name)?;
            if !lane.starts_with("fn:") {
                return None;
            }
            let local = |name: &str, op: &str| -> Option<&Node> {
                nodes.get(name).copied().filter(|n| {
                    n.op.full_name() == op && module.node_lanes.get(&n.name) == Some(lane)
                })
            };
            let input = local(&draw.op.args[1], "data.input_pipe")?;
            let output = local(input.op.args.first()?, "data.output_pipe")?;
            let window = local(output.op.args.first()?, "data.immutable_window")?;
            if output.resource != input.resource
                || window.resource != output.resource
                || !ordered_inputs.contains(&(output.name.as_str(), input.name.as_str()))
            {
                return None;
            }
            let record = local(window.op.args.first()?, "cpu.struct")?;
            let width = u32::try_from(record.op.args.len().checked_sub(1)?).ok()?;
            if !(1..=4).contains(&width) {
                return None;
            }
            let set = local(&draw.op.args[4], "shader.bind_set")?;
            let mut uniform = None;
            let mut storage = None;
            for operand in set.op.args.get(1..)? {
                let b = nodes.get(operand.as_str())?;
                let slot: u32 = b.op.args.first()?.parse().ok()?;
                if slot > 30
                    || b.op.module != "shader"
                    || b.op.args.len() != 2
                    || module.node_lanes.get(&b.name) != Some(lane)
                {
                    return None;
                }
                match b.op.instruction.as_str() {
                    "uniform_binding" if uniform.is_none() && b.op.args[1] == input.name => {
                        uniform = Some(slot)
                    }
                    "storage_binding" if storage.is_none() => storage = Some(slot),
                    _ => return None,
                }
            }
            let slot = uniform?;
            if storage == Some(slot) {
                return None;
            }
            Some((
                draw.name.clone(),
                Site {
                    ordinal: index.try_into().ok()?,
                    slot,
                    width,
                },
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use yir_core::{Edge, Operation};

    fn graph() -> YirModule {
        let mut m = YirModule::new("test");
        for (name, domain, op, args) in [
            ("record", "cpu", "struct", vec!["Packet", "x=value"]),
            (
                "window",
                "data",
                "immutable_window",
                vec!["record", "0", "1"],
            ),
            ("send", "data", "output_pipe", vec!["window"]),
            ("receive", "data", "input_pipe", vec!["send"]),
            ("bind", "shader", "uniform_binding", vec!["2", "receive"]),
            ("storage", "shader", "storage_binding", vec!["3", "pixels"]),
            (
                "set",
                "shader",
                "bind_set",
                vec!["pipeline", "storage", "bind"],
            ),
            (
                "draw",
                "shader",
                "draw_instanced",
                vec!["pass", "receive", "3", "1", "set"],
            ),
        ] {
            m.node_lanes.insert(name.into(), "fn:render".into());
            m.nodes.push(Node {
                name: name.into(),
                resource: domain.into(),
                op: Operation {
                    module: domain.into(),
                    instruction: op.into(),
                    args: args.into_iter().map(String::from).collect(),
                },
            });
        }
        m.edges.push(Edge {
            kind: EdgeKind::Effect,
            from: "send".into(),
            to: "receive".into(),
        });
        m
    }

    #[test]
    fn only_outline_same_lane_same_received_window_and_distinct_fragment_slots() {
        let original = graph();
        assert_eq!(
            sites(&original)["draw"],
            Site {
                ordinal: 7,
                slot: 2,
                width: 1
            }
        );
        for case in 0..8 {
            let mut m = original.clone();
            match case {
                0 => m.edges.clear(),
                1 => {
                    m.node_lanes.insert("send".into(), "fn:other".into());
                }
                2 => m.nodes[4].op.args[1] = "window".into(),
                3 => m.nodes[5].op.args[0] = "2".into(),
                4 => m.nodes[4].op.args[0] = "dynamic".into(),
                5 => m.nodes[6].op.args.push("bind".into()),
                6 => m.nodes[5].op.instruction = "vertex_binding".into(),
                _ => m.nodes[3].resource = "foreign".into(),
            }
            assert!(emit_llvm(&m).is_none(), "case {case}");
        }
    }
}
