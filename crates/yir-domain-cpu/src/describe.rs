use super::describe_async::describe_cpu_async_node;
use super::describe_basic::describe_cpu_basic_node;
use super::describe_host::describe_cpu_host_node;
use super::describe_loops_control::describe_cpu_loops_control_node;
use super::describe_scalar_memory::describe_cpu_scalar_memory_node;
use super::*;

pub(super) fn describe_cpu_node(
    node: &Node,
    resource: &Resource,
) -> Result<InstructionSemantics, String> {
    require_cpu_resource(node, resource)?;
    if let Some(semantics) = describe_cpu_basic_node(node)? {
        return Ok(semantics);
    }
    if let Some(semantics) = describe_cpu_async_node(node)? {
        return Ok(semantics);
    }
    if let Some(semantics) = describe_cpu_scalar_memory_node(node)? {
        return Ok(semantics);
    }
    if let Some(semantics) = describe_cpu_host_node(node)? {
        return Ok(semantics);
    }
    if let Some(semantics) = describe_cpu_loops_control_node(node)? {
        return Ok(semantics);
    }
    Err(format!("unknown cpu instruction `{}`", node.op.instruction))
}
