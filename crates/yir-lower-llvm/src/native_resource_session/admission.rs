use std::collections::{BTreeMap, BTreeSet};
use yir_core::{native_resource_session::ResourceSessionRegistration, EdgeKind, YirModule};

mod guarded_helpers;

#[derive(Clone, PartialEq, Eq)]
pub(super) enum Kind {
    Buffer,
    I64,
    Bool,
    Bytes,
    Void,
    Record(String, Vec<(String, Kind)>),
}

pub(super) fn select(
    module: &YirModule,
    registration: &ResourceSessionRegistration,
) -> Result<YirModule, String> {
    yir_verify::verify_module(module)?;
    let roots = registration.bind(module)?;
    let mixed = registration.mixed_layout(module)?;
    let functions = guarded_helpers::reachable(module, &roots)?;
    let order = crate::topological_order(module)?;
    let nodes = module
        .nodes
        .iter()
        .map(|n| (n.name.as_str(), n))
        .collect::<BTreeMap<_, _>>();
    let mut selected_nodes = BTreeSet::new();
    let mut metadata = BTreeSet::new();
    for function in &functions {
        if !function
            .name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err("resource callback requires an LLVM-safe helper name".to_owned());
        }
        let result = function
            .result
            .as_ref()
            .and_then(|r| nodes.get(r.node.as_str()));
        let return_op = if mixed.is_some() {
            "return_owned_struct"
        } else {
            "return_owned_bytes"
        };
        if result.is_none_or(|node| {
            node.op.instruction != return_op
                && !(mixed.is_some() && node.op.instruction == "tail_call_owned_struct_return")
        }) {
            return Err("resource callback requires a direct owned Bytes return".to_owned());
        }
        for index in 0..function.parameters.len() {
            let instruction =
                match guarded_helpers::parameter_kind(&function.parameters[index], index)? {
                    Kind::Buffer => "param_buffer_ref",
                    Kind::I64 => "param_i64",
                    Kind::Bytes => "param_owned_bytes",
                    _ => unreachable!(),
                };
            let node = nodes[function.parameters[index].node.as_str()];
            if node.op.instruction != instruction || node.op.args != [index.to_string()] {
                return Err("resource callback parameter declaration drift".to_owned());
            }
        }
        let body = function
            .body_nodes
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if body.is_empty() || body.len() > 256 || body.len() != function.body_nodes.len() {
            return Err("resource callback exceeds its unique node bound".to_owned());
        }
        let lane = format!("fn:{}", function.name);
        if module.nodes.iter().any(|node| {
            module.node_lanes.get(&node.name) == Some(&lane) && !body.contains(node.name.as_str())
        }) {
            return Err("resource callback lane contains undeclared nodes".to_owned());
        }
        for edge in &module.edges {
            let from_body = body.contains(edge.from.as_str());
            let to_body = body.contains(edge.to.as_str());
            if from_body == to_body {
                continue;
            }
            // Contract text is immutable metadata, not external initialization.
            // Retain its node and edge; never erase it or admit executable lookalikes.
            let source = nodes[edge.from.as_str()];
            if !from_body
                && to_body
                && edge.kind == EdgeKind::Dep
                && module
                    .node_lanes
                    .get(&source.name)
                    .is_some_and(|lane| lane == "contract")
                && source.op.module == "cpu"
                && source.op.instruction == "text"
                && source.op.args.len() == 1
                && module
                    .resources
                    .iter()
                    .any(|r| r.name == source.resource && r.kind.is_family("cpu"))
                && !module
                    .functions
                    .iter()
                    .any(|f| f.body_nodes.contains(&source.name))
                && !module.edges.iter().any(|e| e.to == source.name)
            {
                metadata.insert(source.name.clone());
            } else {
                return Err(format!(
                    "resource callback has an external ordering or lifetime dependency: {} -> {} ({:?}) in {}",
                    edge.from, edge.to, edge.kind, function.name
                ));
            }
        }
        let ordered = order
            .iter()
            .filter(|n| body.contains(n.as_str()))
            .collect::<Vec<_>>();
        if ordered.len() != body.len()
            || ordered.last().map(|n| n.as_str())
                != function.result.as_ref().map(|r| r.node.as_str())
        {
            return Err("resource callback must end in one complete Bytes return".to_owned());
        }
        let mut values = BTreeMap::new();
        let mut owners = BTreeSet::new();
        let mut copies = 0;
        for name in ordered {
            let node = nodes[name.as_str()];
            if !selected_nodes.insert(name.clone())
                || module.node_lanes.get(name) != Some(&lane)
                || node.op.module != "cpu"
                || !module
                    .resources
                    .iter()
                    .any(|r| r.name == node.resource && r.kind.is_family("cpu"))
            {
                return Err("resource callback has shared, cross-lane or non-CPU nodes".to_owned());
            }
            let args = &node.op.args;
            let instruction = node.op.instruction.as_str();
            if matches!(
                instruction,
                "guard_call_owned_struct_return" | "tail_call_owned_struct_return"
            ) {
                let tail = instruction == "tail_call_owned_struct_return";
                if tail && function.result.as_ref().map(|r| &r.node) != Some(name) {
                    return Err("owned continuation must be the declared terminal return".into());
                }
                guarded_helpers::check(node, module, mixed.as_ref(), &values, &owners)?;
                if tail {
                    owners.clear();
                }
                values.insert(name.clone(), Kind::Void);
                continue;
            }
            if matches!(instruction, "return_owned_bytes" | "return_owned_struct")
                && function.result.as_ref().map(|r| &r.node) != Some(name)
            {
                return Err("resource callback has an undeclared early return".to_owned());
            }
            if let Some(layout) = &mixed {
                if super::mixed_admission::node(node, layout, &mut values, &mut owners)? {
                    continue;
                }
            }
            let (inputs, output): (&[Kind], Kind) = match instruction {
                "param_buffer_ref" | "param_i64" | "param_owned_bytes" => {
                    let index = function
                        .parameters
                        .iter()
                        .position(|p| p.node == *name)
                        .ok_or("resource callback parameter node drift")?;
                    if args != &[index.to_string()]
                        || (instruction == "param_buffer_ref") != (index == 0)
                    {
                        return Err("resource callback parameter node drift".to_owned());
                    }
                    let kind = guarded_helpers::parameter_kind(&function.parameters[index], index)?;
                    if kind == Kind::Bytes {
                        owners.insert(name.clone());
                    }
                    values.insert(name.clone(), kind);
                    continue;
                }
                "const_i64" => {
                    if args.len() != 1 || args[0].parse::<i64>().is_err() {
                        return Err("invalid resource callback integer literal".to_owned());
                    }
                    values.insert(name.clone(), Kind::I64);
                    continue;
                }
                "const_bool" if mixed.is_some() => {
                    if args.len() != 1 || !matches!(args[0].as_str(), "true" | "false") {
                        return Err("invalid resource callback boolean literal".to_owned());
                    }
                    values.insert(name.clone(), Kind::Bool);
                    continue;
                }
                "eq" | "ne" | "lt" | "le" | "gt" | "ge" if mixed.is_some() => {
                    (&[Kind::I64, Kind::I64], Kind::Bool)
                }
                "and" | "or" | "xor" if mixed.is_some() => (&[Kind::Bool, Kind::Bool], Kind::Bool),
                "not" if mixed.is_some() => (&[Kind::Bool], Kind::Bool),
                "add" | "sub" | "mul" | "div" | "rem" => (&[Kind::I64, Kind::I64], Kind::I64),
                "buffer_len" => (&[Kind::Buffer], Kind::I64),
                "load_at" => (&[Kind::Buffer, Kind::I64], Kind::I64),
                "store_at" => (&[Kind::Buffer, Kind::I64, Kind::I64], Kind::Void),
                "copy_buffer_owned" => (&[Kind::Buffer], Kind::Bytes),
                "move_owned_bytes" => (&[Kind::Bytes], Kind::Bytes),
                "owned_bytes_len" => (&[Kind::Bytes], Kind::I64),
                "drop_owned_bytes" | "return_owned_bytes" => (&[Kind::Bytes], Kind::Void),
                _ => return Err(format!("resource callback does not admit `{instruction}`")),
            };
            if args.len() != inputs.len()
                || args.iter().zip(inputs).any(|(arg, kind)| {
                    values.get(arg) != Some(kind) || (*kind == Kind::Bytes && !owners.contains(arg))
                })
            {
                return Err(format!(
                    "resource callback `{name}` has unavailable or incompatible inputs"
                ));
            }
            match instruction {
                "copy_buffer_owned" => {
                    copies += 1;
                    if copies > 8 {
                        return Err(
                            "resource callback exceeds its Bytes allocation bound".to_owned()
                        );
                    }
                    owners.insert(name.clone());
                }
                "move_owned_bytes" => {
                    owners.remove(&args[0]);
                    owners.insert(name.clone());
                }
                "drop_owned_bytes" | "return_owned_bytes" => {
                    owners.remove(&args[0]);
                }
                _ => {}
            }
            values.insert(name.clone(), output);
        }
        if !owners.is_empty() {
            return Err("resource callback leaks an owned Bytes value".to_owned());
        }
    }
    selected_nodes.extend(metadata);
    let mut selected = YirModule::new(module.version.clone());
    selected.functions = functions.into_iter().cloned().collect();
    selected.nodes = module
        .nodes
        .iter()
        .filter(|n| selected_nodes.contains(&n.name))
        .cloned()
        .collect();
    selected.edges = module
        .edges
        .iter()
        .filter(|e| selected_nodes.contains(&e.from) && selected_nodes.contains(&e.to))
        .cloned()
        .collect();
    selected.node_lanes = module
        .node_lanes
        .iter()
        .filter(|(n, _)| selected_nodes.contains(*n))
        .map(|(n, l)| (n.clone(), l.clone()))
        .collect();
    selected.resources = module
        .resources
        .iter()
        .filter(|r| selected.nodes.iter().any(|n| n.resource == r.name))
        .cloned()
        .collect();
    Ok(selected)
}
