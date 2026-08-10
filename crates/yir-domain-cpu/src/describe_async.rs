use super::*;
use yir_core::{CPU_MUTEX_RUNTIME_METADATA, CPU_SHARED_MUTEX_RUNTIME_METADATA};

fn describe_mutex_effect(node: &Node, value_role: &str) -> Result<InstructionSemantics, String> {
    if node.op.args.len() == 1 {
        return Ok(InstructionSemantics::effect(vec![node.op.args[0].clone()]));
    }
    let expected_len = 1 + CPU_MUTEX_RUNTIME_METADATA.len();
    if node.op.args.len() != expected_len {
        return Err(format!(
            "node `{}` expects `cpu.{} <name> <resource> <{}> [{}]`",
            node.name,
            node.op.instruction,
            value_role,
            CPU_MUTEX_RUNTIME_METADATA.join(" ")
        ));
    }
    for (actual, expected) in node.op.args[1..].iter().zip(CPU_MUTEX_RUNTIME_METADATA) {
        if actual != expected {
            return Err(format!(
                "node `{}` has unsupported cpu.{} mutex metadata `{actual}`; expected `{expected}`",
                node.name, node.op.instruction
            ));
        }
    }
    Ok(InstructionSemantics::effect(vec![node.op.args[0].clone()]))
}

fn describe_shared_mutex_effect(
    node: &Node,
    dependency_count: usize,
) -> Result<InstructionSemantics, String> {
    let expected_len = dependency_count + CPU_SHARED_MUTEX_RUNTIME_METADATA.len();
    if node.op.args.len() != expected_len {
        return Err(format!(
            "node `{}` expects {} dependency argument(s) followed by shared-mutex metadata [{}]",
            node.name,
            dependency_count,
            CPU_SHARED_MUTEX_RUNTIME_METADATA.join(" ")
        ));
    }
    for (actual, expected) in node.op.args[dependency_count..]
        .iter()
        .zip(CPU_SHARED_MUTEX_RUNTIME_METADATA)
    {
        if actual != expected {
            return Err(format!(
                "node `{}` has unsupported cpu.{} shared-mutex metadata `{actual}`; expected `{expected}`",
                node.name, node.op.instruction
            ));
        }
    }
    Ok(InstructionSemantics::effect(
        node.op.args[..dependency_count].to_vec(),
    ))
}

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
        | "task_completed" | "task_timed_out" | "task_cancelled" | "task_failed" | "task_value" => {
            if node.op.args.len() != 1 {
                return Err(format!(
                    "node `{}` expects `cpu.{} <name> <resource> <input>`",
                    node.name, node.op.instruction
                ));
            }
            Ok(InstructionSemantics::effect(node.op.args.clone()))
        }
        "mutex_new" => describe_mutex_effect(node, "value"),
        "mutex_lock" | "mutex_unlock" | "mutex_value" => describe_mutex_effect(node, "input"),
        "mutex_share" | "mutex_permit" => describe_shared_mutex_effect(node, 2),
        "mutex_lease_replace" => describe_shared_mutex_effect(node, 2),
        "mutex_shared_close" | "mutex_permit_lock" | "mutex_lease_value" | "mutex_lease_unlock" => {
            describe_shared_mutex_effect(node, 1)
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

#[cfg(test)]
mod tests {
    use super::*;
    use yir_core::Operation;

    fn mutex_node(metadata: &[&str]) -> Node {
        let mut args = vec!["input".to_owned()];
        args.extend(metadata.iter().map(|value| (*value).to_owned()));
        Node {
            name: "lock".to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "mutex_lock".to_owned(),
                args,
            },
        }
    }

    #[test]
    fn accepts_scheduler_mutex_metadata_without_treating_it_as_dependencies() {
        let semantics = describe_cpu_async_node(&mutex_node(&CPU_MUTEX_RUNTIME_METADATA))
            .expect("scheduler mutex metadata")
            .expect("mutex semantics");
        assert_eq!(semantics.dependencies, vec!["input".to_owned()]);
    }

    #[test]
    fn preserves_legacy_single_input_mutex_nodes() {
        let semantics = describe_cpu_async_node(&mutex_node(&[]))
            .expect("legacy mutex node")
            .expect("mutex semantics");
        assert_eq!(semantics.dependencies, vec!["input".to_owned()]);
    }

    #[test]
    fn rejects_scheduler_mutex_metadata_drift() {
        let mut metadata = CPU_MUTEX_RUNTIME_METADATA;
        metadata[1] = "visibility=unbounded";
        let error = describe_cpu_async_node(&mutex_node(&metadata))
            .expect_err("visibility drift must fail closed");
        assert!(error.contains("unsupported cpu.mutex_lock mutex metadata"));
        assert!(error.contains("visibility=acquire-release-epoch-v1"));
    }

    #[test]
    fn shared_mutex_permit_metadata_keeps_both_dependencies() {
        let mut args = vec!["shared".to_owned(), "lane".to_owned()];
        args.extend(
            CPU_SHARED_MUTEX_RUNTIME_METADATA
                .iter()
                .map(|value| (*value).to_owned()),
        );
        let node = Node {
            name: "permit".to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "mutex_permit".to_owned(),
                args,
            },
        };
        let semantics = describe_cpu_async_node(&node)
            .expect("shared mutex metadata")
            .expect("shared mutex semantics");
        assert_eq!(semantics.dependencies, vec!["shared", "lane"]);
    }

    #[test]
    fn shared_mutex_share_metadata_keeps_static_cardinality_dependency() {
        let mut args = vec!["mutex".to_owned(), "cardinality".to_owned()];
        args.extend(
            CPU_SHARED_MUTEX_RUNTIME_METADATA
                .iter()
                .map(|value| (*value).to_owned()),
        );
        let node = Node {
            name: "shared".to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "mutex_share".to_owned(),
                args,
            },
        };
        let semantics = describe_cpu_async_node(&node)
            .expect("shared mutex metadata")
            .expect("shared mutex semantics");
        assert_eq!(semantics.dependencies, vec!["mutex", "cardinality"]);
    }

    #[test]
    fn rejects_shared_mutex_metadata_drift() {
        let mut metadata = CPU_SHARED_MUTEX_RUNTIME_METADATA;
        metadata[3] = "permit_cardinality=dynamic";
        let mut args = vec!["shared".to_owned(), "lane".to_owned()];
        args.extend(metadata.iter().map(|value| (*value).to_owned()));
        let node = Node {
            name: "permit".to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "mutex_permit".to_owned(),
                args,
            },
        };
        let error = describe_cpu_async_node(&node)
            .expect_err("shared permit metadata drift must fail closed");
        assert!(error.contains("unsupported cpu.mutex_permit shared-mutex metadata"));
        assert!(error.contains("permit_cardinality=share-literal-1-to-64-v1"));
    }

    #[test]
    fn rejects_shared_mutex_close_lifecycle_drift() {
        let mut metadata = CPU_SHARED_MUTEX_RUNTIME_METADATA;
        metadata[6] = "lifecycle=implicit-drop";
        let mut args = vec!["shared".to_owned()];
        args.extend(metadata.iter().map(|value| (*value).to_owned()));
        let node = Node {
            name: "close".to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "mutex_shared_close".to_owned(),
                args,
            },
        };
        let error = describe_cpu_async_node(&node)
            .expect_err("shared close lifecycle drift must fail closed");
        assert!(error.contains("unsupported cpu.mutex_shared_close shared-mutex metadata"));
        assert!(error.contains("lifecycle=explicit-close-revoke-v1"));
    }
}
