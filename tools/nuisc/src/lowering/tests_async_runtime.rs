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

#[test]
fn lowers_await_stmt_into_cpu_await_node() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn main() {
            await data_profile_bind_core("FabricPlane");
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let await_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "await")
        .unwrap();
    let awaited = await_node.op.args.first().unwrap();
    assert!(yir.edges.iter().any(|edge| edge.from == *awaited
        && edge.to == await_node.name
        && matches!(edge.kind, EdgeKind::Effect)));
}

#[test]
fn lowers_async_call_with_explicit_schedule_boundary() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          async fn main() {
            await ping();
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let async_call = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .unwrap();
    let await_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "await")
        .unwrap();
    assert!(yir.edges.iter().any(|edge| {
        edge.from == async_call.name
            && edge.to == await_node.op.args[0]
            && matches!(edge.kind, EdgeKind::Effect)
    }));
}

#[test]
fn materializes_registered_scheduler_contract_nodes_for_cpu_modules() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            print(7);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let lane_contract = yir
        .nodes
        .iter()
        .find(|node| node.name == "scheduler_contract_cpu_lane_policy_type")
        .unwrap();
    let lane_capability_contract = yir
        .nodes
        .iter()
        .find(|node| node.name == "scheduler_contract_cpu_lane_capability_type")
        .unwrap();
    let bridge_capability_contract = yir
        .nodes
        .iter()
        .find(|node| node.name == "scheduler_contract_cpu_bridge_capability_type")
        .unwrap();
    let clock_contract = yir
        .nodes
        .iter()
        .find(|node| node.name == "scheduler_contract_cpu_clock_type")
        .unwrap();
    let result_lane_contract = yir
        .nodes
        .iter()
        .find(|node| node.name == "scheduler_contract_cpu_result_lane_type")
        .unwrap();
    let result_capability_contract = yir
        .nodes
        .iter()
        .find(|node| node.name == "scheduler_contract_cpu_result_capability_type")
        .unwrap();
    let observer_role_variant_contract = yir
        .nodes
        .iter()
        .find(|node| node.name == "scheduler_contract_cpu_observer_role_variant_type")
        .unwrap();
    let summary_capability_contract = yir
        .nodes
        .iter()
        .find(|node| node.name == "scheduler_contract_cpu_summary_capability_type")
        .unwrap();
    let summary_class_contract = yir
        .nodes
        .iter()
        .find(|node| node.name == "scheduler_contract_cpu_summary_class_type")
        .unwrap();
    let observer_source_class_contract = yir
        .nodes
        .iter()
        .find(|node| node.name == "scheduler_contract_cpu_observer_source_class_type")
        .unwrap();
    let observer_stage_class_contract = yir
        .nodes
        .iter()
        .find(|node| node.name == "scheduler_contract_cpu_observer_stage_class_type")
        .unwrap();
    let observer_scope_class_contract = yir
        .nodes
        .iter()
        .find(|node| node.name == "scheduler_contract_cpu_observer_scope_class_type")
        .unwrap();
    let observer_branch_class_contract = yir
        .nodes
        .iter()
        .find(|node| node.name == "scheduler_contract_cpu_observer_branch_class_type")
        .unwrap();
    assert_eq!(lane_contract.op.module, "cpu");
    assert_eq!(lane_capability_contract.op.module, "cpu");
    assert_eq!(bridge_capability_contract.op.module, "cpu");
    assert_eq!(clock_contract.op.module, "cpu");
    assert_eq!(result_lane_contract.op.module, "cpu");
    assert_eq!(result_capability_contract.op.module, "cpu");
    assert_eq!(observer_role_variant_contract.op.module, "cpu");
    assert_eq!(summary_capability_contract.op.module, "cpu");
    assert_eq!(summary_class_contract.op.module, "cpu");
    assert_eq!(observer_source_class_contract.op.module, "cpu");
    assert_eq!(observer_stage_class_contract.op.module, "cpu");
    assert_eq!(observer_scope_class_contract.op.module, "cpu");
    assert_eq!(observer_branch_class_contract.op.module, "cpu");
    assert_eq!(
        yir.node_lanes
            .get("scheduler_contract_cpu_lane_policy_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        yir.node_lanes
            .get("scheduler_contract_cpu_result_lane_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        yir.node_lanes
            .get("scheduler_contract_cpu_lane_capability_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        yir.node_lanes
            .get("scheduler_contract_cpu_bridge_capability_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        yir.node_lanes
            .get("scheduler_contract_cpu_result_capability_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        yir.node_lanes
            .get("scheduler_contract_cpu_observer_role_variant_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        yir.node_lanes
            .get("scheduler_contract_cpu_summary_capability_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        yir.node_lanes
            .get("scheduler_contract_cpu_summary_class_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        yir.node_lanes
            .get("scheduler_contract_cpu_observer_source_class_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        yir.node_lanes
            .get("scheduler_contract_cpu_observer_stage_class_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        yir.node_lanes
            .get("scheduler_contract_cpu_observer_scope_class_type")
            .map(String::as_str),
        Some("contract")
    );
    assert_eq!(
        yir.node_lanes
            .get("scheduler_contract_cpu_observer_branch_class_type")
            .map(String::as_str),
        Some("contract")
    );
    assert!(yir.edges.iter().any(|edge| {
        edge.from == "scheduler_contract_cpu_lane_policy_type"
            && matches!(edge.kind, EdgeKind::Dep | EdgeKind::CrossDomainExchange)
    }));
    assert!(lane_contract.op.args[0].contains("family=cpu;"));
    assert!(lane_capability_contract.op.args[0].contains("main=host-entry"));
    assert!(bridge_capability_contract.op.args[0]
        .contains("lane_bridge=cpu_bind_core_lane:host_main_lane|worker_lane"));
    assert!(clock_contract.op.args[0].contains("domain=cpu.clock.host.v1"));
    assert!(result_lane_contract.op.args[0].contains("entry=main"));
    assert!(result_capability_contract.op.args[0].contains("probe=result-ready-probe"));
    assert!(observer_role_variant_contract.op.args[0].contains("send_ready=send-ready-observer"));
    assert!(summary_capability_contract.op.args[0].contains("windowed=async-windowed-summary"));
    assert!(summary_class_contract.op.args[0].contains("transport_split=transport-split-summary"));
    assert!(observer_source_class_contract.op.args[0].contains("summary=summary-backed"));
    assert!(observer_stage_class_contract.op.args[0].contains("payload=observer-payload-stage"));
    assert!(
        observer_scope_class_contract.op.args[0].contains("bridge_visible=bridge-visible-scope")
    );
    assert!(observer_branch_class_contract.op.args[0].contains("fallback=fallback-branch"));
}

#[test]
fn lowers_await_expression_into_value_producing_boundary() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          async fn main() -> i64 {
            let value: i64 = await ping();
            return value;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let async_call = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .unwrap();
    let await_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "await")
        .unwrap();
    assert!(yir.edges.iter().any(|edge| {
        edge.from == async_call.name
            && edge.to == await_node.op.args[0]
            && matches!(edge.kind, EdgeKind::Effect)
    }));
    assert_eq!(await_node.op.args.len(), 1);
}

#[test]
fn sequences_borrow_end_before_free_in_expr_stmt_order() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let head: ref Node = move(alloc_node(10, null()));
            let head_ref: ref Node = borrow(head);
            let current: i64 = load_value(head_ref);
            borrow_end(head_ref);
            free(head);
            return current;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let borrow_end = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "borrow_end")
        .unwrap();
    let free = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "free")
        .unwrap();
    assert!(yir.edges.iter().any(|edge| {
        edge.from == borrow_end.name
            && edge.to == free.name
            && matches!(edge.kind, EdgeKind::Effect)
    }));
}

#[test]
fn sequences_store_at_before_free_in_expr_stmt_order() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(3, 0);
            store_at(buffer, 1, 7);
            let value: i64 = load_at(buffer, 1);
            free(buffer);
            return value;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let store = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "store_at")
        .unwrap();
    let free = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "free")
        .unwrap();
    assert!(path_exists(&yir, &store.name, &free.name));
}

#[test]
fn lowers_explicit_task_primitives_into_cpu_effect_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let task: Task<i64> = spawn(ping());
            cancel(task);
            return join(task);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "join"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "cancel"));
}

#[test]
fn lowers_explicit_timeout_primitive_into_cpu_effect_node() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let task: Task<i64> = timeout(spawn(ping()), 16);
            return join(task);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "timeout"));
}

#[test]
fn lowers_data_result_primitives_into_data_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let result: DataResult<Pipe<i64>> = data_result(data_output_pipe(7));
            let moved: bool = data_moved(result);
            return data_value(result);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "data" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "data" && node.op.instruction == "is_moved"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "data" && node.op.instruction == "value"));
}

#[test]
fn lowers_shader_result_primitives_into_shader_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let result: ShaderResult<Pass> = shader_result(shader_begin_pass(
              shader_target("rgba8", 8, 8),
              shader_pipeline("flat", "triangle"),
              shader_viewport(8, 8)
            ));
            let ready: bool = shader_pass_ready(result);
            return 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "is_pass_ready"));
}

#[test]
fn lowers_kernel_result_primitives_into_kernel_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let result: KernelResult<i64> = kernel_result(kernel_profile_queue_depth("KernelUnit"));
            let ready: bool = kernel_config_ready(result);
            return kernel_value(result);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "is_config_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "value"));
}

#[test]
fn lowers_kernel_tensor_primitives_into_kernel_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let input = kernel_tensor(1, 3, "2,4,6");
            let weights = kernel_tensor(3, 2, "1,-2,3,0,2,1");
            let bias = kernel_tensor(1, 2, "-4,3");
            let projected = kernel_matmul(input, weights);
            let shifted = kernel_add_bias(projected, bias);
            let activated = kernel_relu(shifted);
            return kernel_reduce_sum(activated);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "tensor"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "matmul"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "add_bias"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "relu"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "kernel" && node.op.instruction == "reduce_sum"));
}
