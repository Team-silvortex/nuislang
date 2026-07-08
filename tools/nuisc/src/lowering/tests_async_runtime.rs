use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;
use yir_core::EdgeKind;

fn path_exists(yir: &yir_core::YirModule, from: &str, to: &str) -> bool {
    let mut frontier = vec![from.to_owned()];
    let mut seen = std::collections::BTreeSet::new();
    while let Some(current) = frontier.pop() {
        if current == to {
            return true;
        }
        if !seen.insert(current.clone()) {
            continue;
        }
        for edge in &yir.edges {
            if edge.from == current {
                frontier.push(edge.to.clone());
            }
        }
    }
    false
}

#[path = "tests_async_runtime/await_scheduler.rs"]
mod await_scheduler;
#[path = "tests_async_runtime/compound_flow.rs"]
mod compound_flow;
#[path = "tests_async_runtime/domain_primitives.rs"]
mod domain_primitives;
#[path = "tests_async_runtime/flow_breaks.rs"]
mod flow_breaks;
#[path = "tests_async_runtime/kernel_tensor.rs"]
mod kernel_tensor;
#[path = "tests_async_runtime/memory_order.rs"]
mod memory_order;
#[path = "tests_async_runtime/network_kernel_paths.rs"]
mod network_kernel_paths;
#[path = "tests_async_runtime/post_flow.rs"]
mod post_flow;
#[path = "tests_async_runtime/recursive_helpers.rs"]
mod recursive_helpers;
#[path = "tests_async_runtime/tail_recursion.rs"]
mod tail_recursion;
#[path = "tests_async_runtime/task_primitives.rs"]
mod task_primitives;
#[path = "tests_async_runtime/while_flow.rs"]
mod while_flow;
