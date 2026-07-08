use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

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

#[path = "tests_branch_helpers/cancel_suffix.rs"]
mod cancel_suffix;
#[path = "tests_branch_helpers/dynamic_cancel.rs"]
mod dynamic_cancel;
#[path = "tests_branch_helpers/dynamic_join.rs"]
mod dynamic_join;
#[path = "tests_branch_helpers/dynamic_mutex_thread.rs"]
mod dynamic_mutex_thread;
#[path = "tests_branch_helpers/dynamic_spawn.rs"]
mod dynamic_spawn;
#[path = "tests_branch_helpers/dynamic_thread_spawn.rs"]
mod dynamic_thread_spawn;
#[path = "tests_branch_helpers/dynamic_timeout.rs"]
mod dynamic_timeout;
#[path = "tests_branch_helpers/mutex_guard_suffix.rs"]
mod mutex_guard_suffix;
#[path = "tests_branch_helpers/observer_branches.rs"]
mod observer_branches;
#[path = "tests_branch_helpers/primitives.rs"]
mod primitives;
#[path = "tests_branch_helpers/pure_branch.rs"]
mod pure_branch;
#[path = "tests_branch_helpers/task_result_suffix.rs"]
mod task_result_suffix;
#[path = "tests_branch_helpers/thread_suffix.rs"]
mod thread_suffix;
#[path = "tests_branch_helpers/timeout_cancel_suffix.rs"]
mod timeout_cancel_suffix;
#[path = "tests_branch_helpers/timeout_suffix.rs"]
mod timeout_suffix;
