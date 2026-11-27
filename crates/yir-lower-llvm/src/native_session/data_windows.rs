//! Function-local, one-element Data windows over inline value packets.
//! Verification of the original graph precedes this check. In particular the
//! verifier requires the correct dep/xfer edge for every semantic operand.
use super::function::FunctionAdmission;
use crate::types::{LlvmLoweringState, LlvmValueRef, StructLlvmValueRef};
use std::collections::{BTreeMap, BTreeSet};
use yir_core::{Edge, EdgeKind, Node, YirModule};

pub(crate) fn is_site(node: &Node) -> bool {
    node.op.module == "data"
        && matches!(
            node.op.instruction.as_str(),
            "immutable_window" | "read_window" | "output_pipe" | "input_pipe"
        )
}

pub(super) struct Edges<'a> {
    incoming: BTreeMap<&'a str, Vec<&'a Edge>>,
    outgoing: BTreeMap<&'a str, Vec<&'a Edge>>,
    owned: BTreeSet<&'a str>,
}

impl<'a> Edges<'a> {
    pub fn new(module: &'a YirModule, nodes: &BTreeMap<&str, &Node>) -> Self {
        let mut this = Self {
            incoming: BTreeMap::new(),
            outgoing: BTreeMap::new(),
            owned: BTreeSet::new(),
        };
        if !module.nodes.iter().any(is_site) {
            return this;
        }
        for f in &module.functions {
            this.owned.extend(f.body_nodes.iter().map(String::as_str));
        }
        for e in &module.edges {
            if is_site(nodes[e.to.as_str()]) {
                this.incoming.entry(&e.to).or_default().push(e);
            }
            if is_site(nodes[e.from.as_str()]) {
                this.outgoing.entry(&e.from).or_default().push(e);
            }
        }
        this
    }

    pub fn check(
        &self,
        a: &FunctionAdmission<'_>,
        n: &Node,
        body: &BTreeSet<&str>,
        retained: &mut BTreeSet<String>,
    ) -> Result<(), String> {
        let op = n.op.instruction.as_str();
        let capture = op == "immutable_window";
        let (base, bounds): (&str, &[usize]) = match n.op.args.as_slice() {
            [b, _, _] if capture => (b, &[0, 1]),
            [b, _] if !capture => (b, &[0]),
            [b] if matches!(op, "output_pipe" | "input_pipe") => (b, &[]),
            _ => return Err("invalid native inline Data window operands".into()),
        };
        let producer = a.nodes[base];
        let local_source = match op {
            "immutable_window" => producer.op.full_name() == "cpu.struct",
            "output_pipe" => producer.op.full_name() == "data.immutable_window",
            "input_pipe" => producer.op.full_name() == "data.output_pipe",
            "read_window" => {
                matches!(
                    producer.op.instruction.as_str(),
                    "immutable_window" | "input_pipe"
                ) && producer.op.module == "data"
            }
            _ => false,
        };
        if !body.contains(base) || !local_source || (!capture && producer.resource != n.resource) {
            return Err(format!(
                "native Data {op} `{}` has incompatible local producer `{base}`",
                n.name
            ));
        }
        for (i, (arg, &expected)) in n.op.args[1..].iter().zip(bounds).enumerate() {
            let value = arg.parse::<usize>().ok().or_else(|| {
                let p = *a.nodes.get(arg.as_str())?;
                if p.op.full_name() != "cpu.const_i64" || p.op.args.len() != 1 {
                    return None;
                }
                if !body.contains(arg.as_str())
                    && !(arg.starts_with("project_profile_data_")
                        && a.module.node_lanes.get(arg).is_some_and(|l| l == "profile")
                        && !self.owned.contains(arg.as_str())
                        && !a.incoming.contains_key(arg.as_str()))
                {
                    return None;
                }
                p.op.args[0].parse::<i64>().ok()?.try_into().ok()
            });
            if value != Some(expected) {
                let label = if !capture {
                    "index"
                } else if i == 0 {
                    "offset"
                } else {
                    "length"
                };
                return Err(format!("native inline Data window `{}` requires constant {label} {expected}, got `{arg}`", n.name));
            }
        }
        for e in self.incoming.get(n.name.as_str()).into_iter().flatten() {
            if body.contains(e.from.as_str()) {
                continue;
            }
            let p = a.nodes[e.from.as_str()];
            let bound = e.kind == EdgeKind::CrossDomainExchange
                && p.name.parse::<usize>().is_err()
                && n.op.args[1..].contains(&p.name);
            let contract = e.kind == EdgeKind::Dep
                && p.op.full_name() == "cpu.text"
                && p.op.args.len() == 1
                && p.name.starts_with("scheduler_contract_data_")
                && p.name.ends_with("_type")
                && a.module
                    .node_lanes
                    .get(&p.name)
                    .is_some_and(|l| l == "contract")
                && !self.owned.contains(p.name.as_str())
                && !a.incoming.contains_key(p.name.as_str());
            if !(bound || contract) {
                return Err(format!(
                    "native inline Data window `{}` rejects external {} from `{}`",
                    n.name,
                    e.kind.as_str(),
                    p.name
                ));
            }
            retained.insert(p.name.clone());
        }
        let outgoing = self
            .outgoing
            .get(n.name.as_str())
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let mut receiver = None;
        for e in outgoing {
            let dst = a.nodes[e.to.as_str()];
            let reads = matches!(e.kind, EdgeKind::Dep | EdgeKind::CrossDomainExchange);
            let consumes = dst.resource == n.resource
                && dst.op.module == "data"
                && dst.op.args.first() == Some(&n.name);
            let valid = match op {
                "immutable_window" => {
                    consumes && matches!(dst.op.instruction.as_str(), "read_window" | "output_pipe")
                }
                "output_pipe" => consumes && dst.op.instruction == "input_pipe",
                "input_pipe" => consumes && dst.op.instruction == "read_window",
                _ => true, // read_window has produced an ordinary packet.
            };
            if !body.contains(e.to.as_str()) || (reads && !valid) {
                return Err(format!(
                    "native Data {op} `{}` cannot hand off to {} `{}`",
                    n.name,
                    dst.op.full_name(),
                    dst.name
                ));
            }
            if op == "output_pipe" && reads && receiver.replace(&e.to).is_some() {
                return Err("native Data output_pipe requires a single receiver".into());
            }
        }
        if op == "output_pipe"
            && !receiver.is_some_and(|r| {
                outgoing
                    .iter()
                    .any(|e| e.kind == EdgeKind::Effect && &e.to == r)
            })
        {
            return Err(
                "native Data pipe pair requires one receiver and its explicit effect edge".into(),
            );
        }
        Ok(())
    }
}

/// Keep the opaque window out of the CPU register map. Its backing consists
/// solely of immutable SSA words, not addresses or independently owned values.
#[derive(Default)]
pub(crate) struct LocalHandles {
    windows: BTreeMap<String, StructLlvmValueRef>,
    pipes: BTreeMap<String, StructLlvmValueRef>,
}

pub(crate) fn lower(
    n: &Node,
    handles: &mut LocalHandles,
    state: &mut LlvmLoweringState,
) -> Result<bool, String> {
    if !handles.pipes.is_empty()
        && n.op.module == "cpu"
        && (n.op.instruction == "guard_return" || n.op.instruction.starts_with("return_"))
    {
        return Err("native Data pipe handoff cannot straddle a callback or helper return".into());
    }
    if !is_site(n) {
        return Ok(false);
    }
    let base = &n.op.args[0];
    if n.op.instruction == "immutable_window" {
        let Some(LlvmValueRef::Struct(packet)) = state.registers.get(base) else {
            return Err(format!(
                "native Data window packet `{base}` did not materialize"
            ));
        };
        let mut names = BTreeSet::new();
        if !(1..=super::MAX_SCALAR_SLOTS).contains(&packet.fields.len())
            || !packet.fields.iter().all(|(name, value)| {
                !name.is_empty() && names.insert(name) && matches!(value, LlvmValueRef::I64(_))
            })
        {
            return Err(
                "native Data window requires a bounded flat packet of distinct i64 fields".into(),
            );
        }
        handles.windows.insert(n.name.clone(), packet.clone());
    } else if n.op.instruction == "output_pipe" {
        let packet = handles
            .windows
            .get(base)
            .ok_or("native Data output has no bound window")?;
        super::pipe_events::emit(n, packet, state)?;
        handles.pipes.insert(n.name.clone(), packet.clone());
    } else if n.op.instruction == "input_pipe" {
        let packet = handles
            .pipes
            .remove(base)
            .ok_or("native Data input has no unconsumed output")?;
        super::pipe_events::emit(n, &packet, state)?;
        handles.windows.insert(n.name.clone(), packet);
    } else {
        let snapshot = handles.windows.get(base).ok_or_else(|| {
            format!("native Data window `{base}` was not bound in this invocation")
        })?;
        state
            .registers
            .insert(n.name.clone(), LlvmValueRef::Struct(snapshot.clone()));
    }
    Ok(true)
}
