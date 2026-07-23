use super::*;

use crate::data_mod::{parse_data_flow_state, require_data_resource};

pub(crate) fn describe_data_node(
    node: &Node,
    resource: &Resource,
) -> Result<InstructionSemantics, String> {
    if node.op.instruction != "move" {
        require_data_resource(node, resource)?;
    }
    match node.op.instruction.as_str() {
        "move" => describe_move(node),
        "copy_window" | "immutable_window" => describe_window_slice(node),
        "read_window" => describe_read_window(node),
        "write_window" => describe_write_window(node),
        "freeze_window" => {
            describe_single_dep(node, "data.freeze_window <name> <resource> <window>")
        }
        "marker" => describe_marker(node),
        "output_pipe" => describe_single_effect(node, "data.output_pipe <name> <resource> <input>"),
        "input_pipe" => describe_single_effect(node, "data.input_pipe <name> <resource> <pipe>"),
        "observe" => describe_observe(node),
        "is_ready" | "is_moved" | "is_windowed" | "value" => describe_result_access(node),
        "handle_table" => describe_handle_table(node),
        "provider_request_ingress" => describe_provider_request_ingress(node),
        "bind_core" => describe_bind_core(node),
        other => Err(format!("unknown data instruction `{other}`")),
    }
}

fn describe_move(node: &Node) -> Result<InstructionSemantics, String> {
    if node.op.args.len() != 2 {
        return Err(format!(
            "node `{}` expects `data.move <name> <resource> <input> <to>`",
            node.name
        ));
    }
    Ok(InstructionSemantics::effect(vec![node.op.args[0].clone()]))
}

fn describe_window_slice(node: &Node) -> Result<InstructionSemantics, String> {
    if node.op.args.len() != 3 {
        return Err(format!(
            "node `{}` expects `data.{} <name> <resource> <input> <offset> <len>`",
            node.name, node.op.instruction
        ));
    }
    let mut deps = vec![node.op.args[0].clone()];
    if node.op.args[1].parse::<usize>().is_err() {
        deps.push(node.op.args[1].clone());
    }
    if node.op.args[2].parse::<usize>().is_err() {
        deps.push(node.op.args[2].clone());
    }
    Ok(InstructionSemantics::pure(deps))
}

fn describe_read_window(node: &Node) -> Result<InstructionSemantics, String> {
    if node.op.args.len() != 2 {
        return Err(format!(
            "node `{}` expects `data.read_window <name> <resource> <window> <index>`",
            node.name
        ));
    }
    let mut deps = vec![node.op.args[0].clone()];
    if node.op.args[1].parse::<usize>().is_err() {
        deps.push(node.op.args[1].clone());
    }
    Ok(InstructionSemantics::pure(deps))
}

fn describe_write_window(node: &Node) -> Result<InstructionSemantics, String> {
    if node.op.args.len() != 3 {
        return Err(format!(
            "node `{}` expects `data.write_window <name> <resource> <window> <index> <value>`",
            node.name
        ));
    }
    let mut deps = vec![node.op.args[0].clone(), node.op.args[2].clone()];
    if node.op.args[1].parse::<usize>().is_err() {
        deps.push(node.op.args[1].clone());
    }
    Ok(InstructionSemantics::pure(deps))
}

fn describe_single_dep(node: &Node, usage: &str) -> Result<InstructionSemantics, String> {
    if node.op.args.len() != 1 {
        return Err(format!("node `{}` expects `{usage}`", node.name));
    }
    Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
}

fn describe_marker(node: &Node) -> Result<InstructionSemantics, String> {
    if node.op.args.len() != 1 {
        return Err(format!(
            "node `{}` expects `data.marker <name> <resource> <tag>`",
            node.name
        ));
    }
    Ok(InstructionSemantics::pure(Vec::new()))
}

fn describe_single_effect(node: &Node, usage: &str) -> Result<InstructionSemantics, String> {
    if node.op.args.len() != 1 {
        return Err(format!("node `{}` expects `{usage}`", node.name));
    }
    Ok(InstructionSemantics::effect(vec![node.op.args[0].clone()]))
}

fn describe_observe(node: &Node) -> Result<InstructionSemantics, String> {
    if node.op.args.len() != 2 {
        return Err(format!(
            "node `{}` expects `data.observe <name> <resource> <input> <state>`",
            node.name
        ));
    }
    parse_data_flow_state(&node.op.args[1]).map_err(|error| {
        format!(
            "node `{}` has invalid data observe state: {error}",
            node.name
        )
    })?;
    Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
}

fn describe_result_access(node: &Node) -> Result<InstructionSemantics, String> {
    if node.op.args.len() != 1 {
        return Err(format!(
            "node `{}` expects `data.{} <name> <resource> <result>`",
            node.name, node.op.instruction
        ));
    }
    Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
}

fn describe_handle_table(node: &Node) -> Result<InstructionSemantics, String> {
    if node.op.args.is_empty() {
        return Err(format!(
            "node `{}` expects `data.handle_table <name> <resource> <slot=resource> [slot=resource...]`",
            node.name
        ));
    }
    for entry in &node.op.args {
        if entry.split_once('=').is_none() {
            return Err(format!(
                "node `{}` has invalid handle-table entry `{}`",
                node.name, entry
            ));
        }
    }
    Ok(InstructionSemantics::pure(Vec::new()))
}

fn describe_provider_request_ingress(node: &Node) -> Result<InstructionSemantics, String> {
    if !matches!(node.op.args.len(), 5 | 8) {
        return Err(format!(
            "node `{}` expects 5 legacy or 8 capsule arguments for `data.provider_request_ingress`",
            node.name
        ));
    }
    Ok(InstructionSemantics::effect(node.op.args.clone()))
}

fn describe_bind_core(node: &Node) -> Result<InstructionSemantics, String> {
    if node.op.args.len() != 1 {
        return Err(format!(
            "node `{}` expects `data.bind_core <name> <resource> <core_index>`",
            node.name
        ));
    }
    node.op.args[0].parse::<usize>().map_err(|_| {
        format!(
            "node `{}` has invalid fabric core index `{}`",
            node.name, node.op.args[0]
        )
    })?;
    Ok(InstructionSemantics::effect(Vec::new()))
}
