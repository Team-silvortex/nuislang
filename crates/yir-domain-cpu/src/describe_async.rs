use super::*;

pub(super) fn describe_cpu_async_node(node: &Node) -> Result<Option<InstructionSemantics>, String> {
    let semantics = match node.op.instruction.as_str() {
        "async_call" => {
            if node.op.args.is_empty() {
                return Err(format!(
                    "node `{}` expects `cpu.async_call <name> <resource> <callee> [arg...]`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::effect(
                node.op.args.iter().skip(1).cloned().collect(),
            ))
        }
        "spawn_task" | "spawn_thread" | "thread_spawn" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `cpu.{} <name> <resource> <callee> <result>`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::effect(vec![node.op.args[1].clone()]))
        }
        "join" | "cancel" | "join_result" | "thread_join" | "thread_join_result"
        | "task_completed" | "task_timed_out" | "task_cancelled" | "task_failed" | "task_value"
        | "mutex_lock" | "mutex_unlock" | "mutex_value" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `cpu.{} <name> <resource> <input>`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        "mutex_new" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `cpu.mutex_new <name> <resource> <value>`",
                    node.name
                ));
            }
            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        "timeout" | "ready_after" => {
            if node.op.args.len() != 2 {
                return Err(format!(
                    "node `{}` expects `cpu.{} <name> <resource> <task> <ticks>`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        _ => return Ok(None),
    };
    semantics.map(Some)
}
