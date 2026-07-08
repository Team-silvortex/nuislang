use super::*;

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
fn lowers_await_if_expression_branches_into_cpu_effect_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn one() -> i64 {
            return 1;
          }

          async fn two() -> i64 {
            return 2;
          }

          async fn main() -> i64 {
            let choose: i64 = cpu_input_i64("choose", 1, 0, 1, 1);
            let value: i64 = await if choose > 0 {
              one()
            } else {
              two()
            };
            return value;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let await_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "await")
        .count();

    assert_eq!(async_call_count, 2);
    assert_eq!(await_count, 2);
}

#[test]
fn lowers_await_match_expression_branches_into_cpu_effect_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn one() -> i64 {
            return 1;
          }

          async fn two() -> i64 {
            return 2;
          }

          async fn main() -> i64 {
            let choice: i64 = cpu_input_i64("choice", 1, 0, 2, 1);
            let value: i64 = await match choice {
              1 => { one() },
              _ => { two() }
            };
            return value;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let async_call_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let await_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "await")
        .count();

    assert_eq!(async_call_count, 2);
    assert_eq!(await_count, 2);
}
