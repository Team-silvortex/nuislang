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
            let value: i64 = await if true {
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
            let value: i64 = await match 1 {
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

#[test]
fn lowers_recursive_async_call_into_schedule_boundary_and_helper_lane() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_down(current: i64) -> i64 {
            if current == 0 {
              return 0;
            }
            let tail: i64 = await sum_down(current - 1);
            return current + tail;
          }

          async fn main() -> i64 {
            return await sum_down(4);
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
    let call_i64_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| name == "sum_down")
        })
        .count();
    assert!(
        async_call_count >= 2,
        "expected recursive async lowering to emit schedule boundaries, found {async_call_count}"
    );
    assert!(
        call_i64_count >= 2,
        "expected recursive async lowering to emit helper-lowered calls, found {call_i64_count}"
    );
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:sum_down"));
}

#[test]
fn lowers_self_tail_recursive_async_function_into_loop_while_i64_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_next(current: i64, acc: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            return await sum_next(current - 1, acc + (current - 1));
          }

          async fn main() -> i64 {
            return await sum_next(4, 1);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    dbg!(yir
        .nodes
        .iter()
        .map(|node| format!("{}::{}", node.op.module, node.op.instruction))
        .collect::<Vec<_>>());

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "add_current");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node.op.args.first().is_some_and(|name| name == "sum_next")
        })
        .count();
    assert_eq!(
        self_async_call_count, 1,
        "expected only the outer entry async call to remain after self tail-recursive async rewrite"
    );
}

#[test]
fn lowers_branching_self_tail_recursive_async_function_into_loop_while_i64_cond_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_selected(current: i64, acc: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if current > 2 {
              return await sum_selected(current - 1, acc + (current - 1));
            } else {
              return await sum_selected(current - 1, acc + 0);
            }
          }

          async fn main() -> i64 {
            return await sum_selected(4, 0);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_cond_chain"
        })
        .expect("expected loop_while_scalar_cond_chain node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "prev_current_gt");
    assert_eq!(loop_node.op.args[8], "add_current");
    assert_eq!(loop_node.op.args[9], "keep");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "sum_selected")
        })
        .count();
    assert_eq!(
        self_async_call_count, 1,
        "expected only the outer entry async call to remain after branching self tail-recursive async rewrite"
    );
}

#[test]
fn lowers_multi_carry_prev_current_self_tail_recursive_async_function_into_loop_while_i64_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn accumulate(current: i64, sum: i64, prod: i64) -> i64 {
            if current <= 1 {
              return sum + prod;
            }
            return await accumulate(current - 1, sum + current, prod * current);
          }

          async fn main() -> i64 {
            return await accumulate(4, 0, 1);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "add_prev_current");
    assert_eq!(loop_node.op.args[8], "mul_prev_current");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "accumulate")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_carry_condition_branching_multi_carry_prev_current_self_tail_recursive_async_function_into_loop_while_i64_cond_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn accumulate(current: i64, sum: i64, prod: i64) -> i64 {
            if current <= 1 {
              return sum + prod;
            }
            if sum > 3 {
              return await accumulate(current - 1, sum + 0, prod + current);
            } else {
              return await accumulate(current - 1, sum + current, prod * current);
            }
          }

          async fn main() -> i64 {
            return await accumulate(4, 0, 1);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_cond_chain"
        })
        .expect("expected loop_while_scalar_cond_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "prev_carry0_gt");
    assert_eq!(loop_node.op.args[8], "keep");
    assert_eq!(loop_node.op.args[9], "add_prev_current");
    assert_eq!(loop_node.op.args[11], "prev_carry0_gt");
    assert_eq!(loop_node.op.args[13], "add_prev_current");
    assert_eq!(loop_node.op.args[14], "mul_prev_current");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "accumulate")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_cross_prev_carry_self_tail_recursive_async_function_into_loop_while_i64_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn accumulate(current: i64, sum: i64, prod: i64) -> i64 {
            if current <= 1 {
              return sum + prod;
            }
            return await accumulate(current - 1, sum + prod, prod + current);
          }

          async fn main() -> i64 {
            return await accumulate(4, 0, 1);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "add_prev_carry1");
    assert_eq!(loop_node.op.args[8], "add_prev_current");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "accumulate")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_branching_cross_prev_carry_self_tail_recursive_async_function_into_loop_while_i64_cond_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn accumulate(current: i64, sum: i64, prod: i64) -> i64 {
            if current <= 1 {
              return sum + prod;
            }
            if current > 2 {
              return await accumulate(current - 1, sum + prod, prod + current);
            } else {
              return await accumulate(current - 1, sum + 0, prod + sum);
            }
          }

          async fn main() -> i64 {
            return await accumulate(4, 0, 1);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_cond_chain"
        })
        .expect("expected loop_while_scalar_cond_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "prev_current_gt");
    assert_eq!(loop_node.op.args[8], "add_prev_carry1");
    assert_eq!(loop_node.op.args[9], "keep");
    assert_eq!(loop_node.op.args[11], "prev_current_gt");
    assert_eq!(loop_node.op.args[13], "add_prev_current");
    assert_eq!(loop_node.op.args[14], "add_prev_carry0");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "accumulate")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_identity_branching_self_tail_recursive_async_function_into_loop_while_i64_cond_chain_with_keep_prev_carry(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn accumulate(current: i64, acc: i64) -> i64 {
            if current <= 1 {
              return acc;
            }
            if current > 2 {
              return await accumulate(current - 1, acc + current);
            } else {
              return await accumulate(current - 1, acc);
            }
          }

          async fn main() -> i64 {
            return await accumulate(4, 0);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_cond_chain"
        })
        .expect("expected loop_while_scalar_cond_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "prev_current_gt");
    assert_eq!(loop_node.op.args[8], "add_prev_current");
    assert_eq!(loop_node.op.args[9], "keep_prev_carry");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "accumulate")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_early_break_self_tail_recursive_async_function_into_loop_while_scalar_flow_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_until(current: i64, acc: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if current > 2 {
              return acc;
            } else {
              return await sum_until(current - 1, acc + current);
            }
          }

          async fn main() -> i64 {
            return await sum_until(4, 0);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_chain"
        })
        .expect("expected loop_while_scalar_flow_chain node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[5], "prev_current_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_prev_current");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node.op.args.first().is_some_and(|name| name == "sum_until")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_early_break_branching_self_tail_recursive_async_function_into_loop_while_scalar_flow_cond_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_until(current: i64, acc: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if current > 2 {
              return acc;
            } else {
              if current > 1 {
                return await sum_until(current - 1, acc + current);
              } else {
                return await sum_until(current - 1, acc + 0);
              }
            }
          }

          async fn main() -> i64 {
            return await sum_until(4, 0);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[5], "prev_current_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "prev_current_gt");
    assert_eq!(loop_node.op.args[11], "add_prev_current");
    assert_eq!(loop_node.op.args[12], "keep");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node.op.args.first().is_some_and(|name| name == "sum_until")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_post_flow_break_self_tail_recursive_async_function_into_loop_while_scalar_post_flow_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_until(current: i64, acc: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if acc + current > 5 {
              return acc + current;
            }
            return await sum_until(current - 1, acc + current);
          }

          async fn main() -> i64 {
            return await sum_until(4, 0);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_post_flow_chain"
        })
        .expect("expected loop_while_scalar_post_flow_chain node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[5], "carry0_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_prev_current");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node.op.args.first().is_some_and(|name| name == "sum_until")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_post_flow_break_branching_aux_carry_self_tail_recursive_async_function_into_loop_while_scalar_post_flow_cond_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_until(current: i64, acc: i64, flag: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if acc + current > 5 {
              return acc + current;
            }
            if current > 1 {
              return await sum_until(current - 1, acc + current, flag + current);
            } else {
              return await sum_until(current - 1, acc + current, flag + 0);
            }
          }

          async fn main() -> i64 {
            return await sum_until(4, 0, 0);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let lowered_ops = yir
        .nodes
        .iter()
        .map(|node| format!("{}::{}", node.op.module, node.op.instruction))
        .collect::<Vec<_>>();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_post_flow_cond_chain"
        })
        .unwrap_or_else(|| {
            panic!("expected loop_while_scalar_post_flow_cond_chain node, got {lowered_ops:?}")
        });
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[5], "carry0_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "always");
    assert_eq!(loop_node.op.args[11], "add_prev_current");
    assert_eq!(loop_node.op.args[14], "prev_current_gt");
    assert_eq!(loop_node.op.args[16], "add_prev_current");
    assert_eq!(loop_node.op.args[17], "keep");
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node.op.args.first().is_some_and(|name| name == "sum_until")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_post_flow_break_identity_branching_aux_carry_self_tail_recursive_async_function_into_loop_while_scalar_post_flow_cond_chain_with_keep_prev_carry(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_until(current: i64, acc: i64, flag: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if acc + current > 5 {
              return acc + current;
            }
            if current > 2 {
              return await sum_until(current - 1, acc + current, flag + current);
            } else {
              return await sum_until(current - 1, acc + current, flag);
            }
          }

          async fn main() -> i64 {
            return await sum_until(4, 0, 0);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_post_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[5], "carry0_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert!(loop_node.op.args.iter().any(|arg| arg == "prev_current_gt"));
    assert!(loop_node.op.args.iter().any(|arg| arg == "add_prev_current"));
    assert!(loop_node.op.args.iter().any(|arg| arg == "keep_prev_carry"));
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node.op.args.first().is_some_and(|name| name == "sum_until")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_post_flow_break_nested_branching_aux_carry_self_tail_recursive_async_function_into_loop_while_scalar_post_flow_cond_chain(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_until(current: i64, acc: i64, flag: i64) -> i64 {
            if current == 0 {
              return acc;
            }
            if acc + current > 6 {
              return acc + current;
            }
            if current > 3 {
              return await sum_until(current - 1, acc + current, flag + current);
            } else {
              if current > 1 {
                return await sum_until(current - 1, acc + current, flag + current);
              } else {
                return await sum_until(current - 1, acc + current, flag + 0);
              }
            }
          }

          async fn main() -> i64 {
            return await sum_until(5, 0, 0);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_post_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[5], "carry0_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert!(loop_node.op.args.iter().any(|arg| arg == "or"));
    assert!(loop_node.op.args.iter().any(|arg| arg == "prev_current_gt"));
    assert!(loop_node
        .op
        .args
        .iter()
        .any(|arg| arg == "add_prev_current"));
    assert!(loop_node.op.args.iter().any(|arg| arg == "keep"));
    let self_async_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "async_call"
                && node.op.args.first().is_some_and(|name| name == "sum_until")
        })
        .count();
    assert_eq!(self_async_call_count, 1);
}

#[test]
fn lowers_async_while_with_await_step_and_pure_carry_into_async_loop_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 3 {
              let value: i64 = await step(value);
              let acc: i64 = acc + value;
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_async_chain"
        })
        .expect("expected loop_while_scalar_async_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[5], "add_current");
}

#[test]
fn lowers_async_while_with_await_step_and_dynamic_buffer_index_read_carry() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            let buffer: ref Buffer = alloc_buffer(8, 9);
            while value < 3 {
              let value: i64 = await step(value);
              let acc: i64 = acc + load_at(buffer, value);
            }
            free(buffer);
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_async_chain"
        })
        .expect("expected loop_while_scalar_async_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[5], "add_read_at_dynamic_current");
}

#[test]
fn lowers_async_while_with_await_step_and_break_flow_control() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 3 {
              let value: i64 = await step(value);
              if value > 1 {
                break;
              }
              let acc: i64 = acc + value;
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_async_flow_chain"
        })
        .expect("expected loop_while_scalar_async_flow_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "current_gt");
    assert_eq!(loop_node.op.args[6], "break");
    assert_eq!(loop_node.op.args[8], "add_current");
}

#[test]
fn lowers_async_while_with_await_step_and_continue_flow_control() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 4 {
              let value: i64 = await step(value);
              if value == 2 {
                continue;
              }
              let acc: i64 = acc + value;
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_async_flow_chain"
        })
        .expect("expected loop_while_scalar_async_flow_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "current_eq");
    assert_eq!(loop_node.op.args[6], "continue");
    assert_eq!(loop_node.op.args[8], "add_current");
}

#[test]
fn lowers_async_while_with_await_step_and_conditional_carry_flow_control() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = await step(value);
              if value > 3 {
                continue;
              }
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_async_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_async_flow_cond_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "current_gt");
    assert_eq!(loop_node.op.args[6], "continue");
    assert_eq!(loop_node.op.args[8], "current_gt");
    assert_eq!(loop_node.op.args[10], "add_current");
    assert_eq!(loop_node.op.args[11], "keep");
}

#[test]
fn lowers_async_while_with_compound_flow_control_and_conditional_carry() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = await step(value);
              if value > 1 {
                if value > 4 {
                  break;
                } else {
                }
              } else {
              }
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_async_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_async_flow_cond_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "and");
    assert_eq!(loop_node.op.args[5], "current_gt");
    assert_eq!(loop_node.op.args[7], "current_gt");
    assert_eq!(loop_node.op.args[9], "break");
    assert_eq!(loop_node.op.args[11], "current_gt");
    assert_eq!(loop_node.op.args[13], "add_current");
    assert_eq!(loop_node.op.args[14], "keep");
}

#[test]
fn lowers_async_recursive_boolean_break_then_branching_carry_into_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 7 {
              let value: i64 = await step(value);
              if value > 1 && value > 3 && value < 6 {
                break;
              } else {
              }
              if value > 4 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_async_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_async_flow_cond_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "and");
    assert_eq!(loop_node.op.args[5], "and");
    assert_eq!(loop_node.op.args[6], "current_gt");
    assert_eq!(loop_node.op.args[8], "current_gt");
    assert_eq!(loop_node.op.args[10], "current_lt");
    assert_eq!(loop_node.op.args[12], "break");
    assert_eq!(loop_node.op.args[14], "current_gt");
    assert_eq!(loop_node.op.args[16], "add_current");
    assert_eq!(loop_node.op.args[17], "keep");
}

#[test]
fn lowers_async_while_with_await_step_and_post_flow_break() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 8 {
              let value: i64 = await step(value);
              let acc: i64 = acc + value;
              if acc > 6 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_async_post_flow_chain"
        })
        .expect("expected loop_while_scalar_async_post_flow_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "carry0_gt");
    assert_eq!(loop_node.op.args[6], "break");
    assert_eq!(loop_node.op.args[8], "add_current");
}

#[test]
fn lowers_async_while_with_await_step_and_post_flow_conditional_break() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 7 {
              let value: i64 = await step(value);
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
              if acc > 5 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_async_post_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_async_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "carry0_gt");
    assert_eq!(loop_node.op.args[6], "break");
    assert_eq!(loop_node.op.args[8], "current_gt");
    assert_eq!(loop_node.op.args[10], "add_current");
    assert_eq!(loop_node.op.args[11], "keep");
}

#[test]
fn lowers_async_while_with_compound_post_flow_control_and_conditional_carry() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = await step(value);
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
              match acc {
                5 => { continue; },
                _ => {
                  if acc < 6 {
                    continue;
                  } else {
                  }
                }
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_async_post_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_async_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "or");
    assert_eq!(loop_node.op.args[5], "carry0_eq");
    assert_eq!(loop_node.op.args[7], "carry0_lt");
    assert_eq!(loop_node.op.args[9], "continue");
    assert_eq!(loop_node.op.args[11], "current_gt");
    assert_eq!(loop_node.op.args[13], "add_current");
    assert_eq!(loop_node.op.args[14], "keep");
}

#[test]
fn rejects_async_chained_while_with_future_sibling_carry_dependency() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn seed_slot() -> i64 {
            return 0;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            let slot: i64 = await seed_slot();
            let buffer: ref Buffer = alloc_buffer(4, 9);
            while value < 3 {
              let value: i64 = await step(value);
              let acc: i64 = acc + load_at(buffer, slot);
              if value > 1 {
                let slot: i64 = slot + value;
              } else {
                let slot: i64 = slot + 0;
              }
            }
            return acc + slot;
          }
        }
        "#,
    )
    .unwrap();

    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();
    assert!(error.contains(
        "references sibling carry `slot` before that carry is updated in the loop body"
    ));
}

#[test]
fn lowers_async_post_flow_continue_with_recursive_boolean_condition_into_post_flow_cond_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 7 {
              let value: i64 = await step(value);
              if value > 4 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
              if acc > 1 && acc > 3 && acc < 10 {
                continue;
              } else {
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_async_post_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_async_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[2], "step");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "and");
    assert_eq!(loop_node.op.args[5], "and");
    assert_eq!(loop_node.op.args[6], "carry0_gt");
    assert_eq!(loop_node.op.args[8], "carry0_gt");
    assert_eq!(loop_node.op.args[10], "carry0_lt");
    assert_eq!(loop_node.op.args[12], "continue");
    assert_eq!(loop_node.op.args[14], "current_gt");
    assert_eq!(loop_node.op.args[16], "add_current");
    assert_eq!(loop_node.op.args[17], "keep");
}

#[test]
fn lowers_async_kernel_observer_step_into_async_post_flow_break_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            let probe: KernelResult<i64> =
              kernel_result(kernel_profile_queue_depth("KernelUnit"));
            if kernel_config_ready(probe) {
              return value + kernel_value(probe);
            }
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = await step(value);
              let acc: i64 = acc + value;
              if acc > 8 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_async_post_flow_chain"
        })
        .expect("expected loop_while_scalar_async_post_flow_chain node");
    assert_eq!(loop_node.op.args[2], "step");
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
fn rejects_async_while_with_await_step_and_task_observer_flow_control() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 3 {
              let value: i64 = await step(value);
              let result: TaskResult<i64> = join_result(spawn(step(value)));
              if task_completed(result) {
                break;
              }
              let acc: i64 = acc + value;
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();

    assert!(error.contains(
        "async/task-driven `while` lowering currently recognizes only structured async loop shapes"
    ));
    assert!(error.contains(
        "general async backedge execution with task primitives inside arbitrary loop conditions/bodies is not lowered yet"
    ));
}

#[test]
fn lowers_mutually_recursive_async_calls_into_schedule_boundaries_and_helper_lanes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn odd(value: i64) -> i64 {
            if value == 0 {
              return 0;
            }
            return await even(value - 1);
          }

          async fn even(value: i64) -> i64 {
            if value == 0 {
              return 1;
            }
            return await odd(value - 1);
          }

          async fn main() -> i64 {
            return await even(4);
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
    let call_i64_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "odd" || name == "even")
        })
        .count();
    assert!(
        async_call_count >= 3,
        "expected async mutual recursion to emit schedule boundaries, found {async_call_count}"
    );
    assert!(
        call_i64_count >= 3,
        "expected async mutual recursion to emit helper-lowered calls, found {call_i64_count}"
    );
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:odd"));
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:even"));
}

#[test]
fn lowers_generic_recursive_async_call_into_schedule_boundary_and_specialized_helper_lane() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn bounce<T>(value: T, remaining: i64) -> T {
            if remaining == 0 {
              return value;
            }
            return await bounce(value, remaining - 1);
          }

          async fn main() -> i64 {
            return await bounce(7, 4);
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
    let specialized_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name.starts_with("bounce__i64"))
        })
        .count();
    assert!(
        async_call_count >= 2,
        "expected generic recursive async lowering to emit schedule boundaries, found {async_call_count}"
    );
    assert!(
        specialized_call_count >= 2,
        "expected generic recursive async lowering to emit specialized helper calls, found {specialized_call_count}"
    );
    assert!(yir
        .node_lanes
        .values()
        .any(|lane| lane.starts_with("fn:bounce__i64")));
}

#[test]
fn lowers_generic_mutually_recursive_async_calls_into_schedule_boundaries_and_specialized_helper_lanes(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn odd<T>(value: T, remaining: i64) -> T {
            if remaining == 0 {
              return value;
            }
            return await even(value, remaining - 1);
          }

          async fn even<T>(value: T, remaining: i64) -> T {
            if remaining == 0 {
              return value;
            }
            return await odd(value, remaining - 1);
          }

          async fn main() -> i64 {
            return await even(7, 4);
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
    let specialized_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("odd__i64") || name.starts_with("even__i64")
                })
        })
        .count();
    assert!(
        async_call_count >= 3,
        "expected generic async mutual recursion to emit schedule boundaries, found {async_call_count}"
    );
    assert!(
        specialized_call_count >= 3,
        "expected generic async mutual recursion to emit specialized helper calls, found {specialized_call_count}"
    );
    assert!(yir
        .node_lanes
        .values()
        .any(|lane| lane.starts_with("fn:odd__i64")));
    assert!(yir
        .node_lanes
        .values()
        .any(|lane| lane.starts_with("fn:even__i64")));
}

#[test]
fn lowers_recursive_async_with_generic_payload_alias_higher_order_body_into_specialized_helper_lanes(
) {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;
          type Mapper<T> = Fn1<T, T>;

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          struct Just<T> {
            value: T,
          }

          fn apply_payload<T: Addable>(value: JustAlias<T>, f: Mapper<T>) -> T {
            match value {
              JustAlias<T>(payload) => {
                return f(payload);
              }
              _ => {
                return value.value;
              }
            }
          }

          async fn climb(value: i64, remaining: i64) -> i64 {
            if remaining == 0 {
              return apply_payload(
                JustAlias<i64>(value),
                |x: i64| -> i64 { return x.add(1); }
              );
            }
            return await climb(value, remaining - 1);
          }

          async fn main() -> i64 {
            return await climb(7, 4);
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
    let recursive_call_count = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| name == "climb")
        })
        .count();
    assert!(
        async_call_count >= 2,
        "expected async recursive higher-order lowering to emit schedule boundaries, found {async_call_count}"
    );
    assert!(
        recursive_call_count >= 2,
        "expected async recursive higher-order lowering to emit recursive helper calls, found {recursive_call_count}"
    );
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:climb"));
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
fn sequences_borrowed_next_traversal_before_borrow_end_and_free() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let tail: ref Node = move(alloc_node(30, null()));
            let head: ref Node = alloc_node(10, tail);
            let head_ref: ref Node = borrow(head);
            let next_ptr: ref Node = load_next(head_ref);
            let tail_ref: ref Node = borrow(next_ptr);
            let current: i64 = load_value(tail_ref);
            borrow_end(tail_ref);
            borrow_end(head_ref);
            free(head);
            return current;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let load_next = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "load_next")
        .unwrap();
    let load_value = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "load_value")
        .unwrap();
    let borrow_ends = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "borrow_end")
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let free = yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "free")
        .unwrap();

    assert!(path_exists(&yir, &load_next.name, &load_value.name));
    assert!(borrow_ends
        .iter()
        .all(|borrow_end| path_exists(&yir, &load_value.name, borrow_end)));
    assert!(borrow_ends
        .iter()
        .all(|borrow_end| path_exists(&yir, borrow_end, &free.name)));
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
fn lowers_recursive_async_result_family_observation_path_into_cpu_effect_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn sum_down(seed: i64, remaining: i64) -> i64 {
            if remaining == 0 {
              return seed;
            }
            return await sum_down(seed + 1, remaining - 1);
          }

          fn encode_timed_out(result: TaskResult<i64>) -> i64 {
            if task_timed_out(result) {
              return 1;
            }
            return 0;
          }

          fn encode_cancelled(result: TaskResult<i64>) -> i64 {
            if task_cancelled(result) {
              return 1;
            }
            return 0;
          }

          fn encode_value(result: TaskResult<i64>) -> i64 {
            if task_completed(result) {
              return task_value(result);
            }
            return 0;
          }

          fn main() -> i64 {
            let completed_result: TaskResult<i64> = join_result(spawn(sum_down(7, 4)));
            let timed_result: TaskResult<i64> =
              join_result(timeout(spawn(sum_down(7, 4)), 0));
            let cancelled_result: TaskResult<i64> =
              join_result(cancel(spawn(sum_down(7, 4))));

            return encode_value(completed_result)
              + encode_timed_out(timed_result)
              + encode_cancelled(cancelled_result);
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
        .any(|node| node.op.module == "cpu" && node.op.instruction == "join_result"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_completed"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_value"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_timed_out"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_cancelled"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "timeout"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "cancel"));
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
fn lowers_network_result_primitives_into_network_nodes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let result: NetworkResult<i64> =
              network_result(network_profile_send_window("NetworkUnit"));
            let send_ready: bool = network_send_ready(result);
            let recv_ready: bool = network_recv_ready(result);
            let config_ready: bool = network_config_ready(result);
            return network_value(result);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_send_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_recv_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_config_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "value"));
}

#[test]
fn lowers_async_network_result_recursive_control_flow_observation_path() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn consume_network_result(result: NetworkResult<i64>) -> i64 {
            if network_send_ready(result) || network_recv_ready(result) {
              return network_value(result) + 1;
            }
            if network_config_ready(result) {
              return network_value(result) + 7;
            }
            return 0;
          }

          fn main() -> i64 {
            let primary: NetworkResult<i64> =
              network_result(network_profile_send_window("NetworkUnit"));
            let fallback: NetworkResult<i64> =
              network_result(network_profile_recv_window("NetworkUnit"));
            let config_only: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));

            let primary_task: Task<i64> = spawn(consume_network_result(primary));
            let fallback_task: Task<i64> = spawn(consume_network_result(fallback));
            let config_task: Task<i64> = spawn(consume_network_result(config_only));

            let primary_result: TaskResult<i64> = join_result(primary_task);
            let fallback_result: TaskResult<i64> = join_result(fallback_task);
            let config_result: TaskResult<i64> = join_result(config_task);

            if task_completed(primary_result) {
              return task_value(primary_result);
            }
            if task_completed(fallback_result) {
              return task_value(fallback_result);
            }
            if task_completed(config_result) {
              return task_value(config_result);
            }
            return 0;
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
        .any(|node| node.op.module == "cpu" && node.op.instruction == "join_result"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_send_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_recv_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_config_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "value"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_network_lane_policy_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_network_result_capability_type"));
}

#[test]
fn lowers_async_network_result_task_policy_observation_path() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn consume_network_result(result: NetworkResult<i64>) -> i64 {
            if network_send_ready(result) {
              return network_value(result) + 1;
            }
            if network_recv_ready(result) {
              return network_value(result) + 2;
            }
            if network_config_ready(result) {
              return network_value(result) + 3;
            }
            return 0;
          }

          fn main() -> i64 {
            let primary: NetworkResult<i64> =
              network_result(network_profile_send_window("NetworkUnit"));
            let fallback: NetworkResult<i64> =
              network_result(network_profile_recv_window("NetworkUnit"));
            let config_only: NetworkResult<i64> =
              network_result(network_profile_bind_core("NetworkUnit"));

            let completed_result: TaskResult<i64> =
              join_result(spawn(consume_network_result(primary)));
            let timed_result: TaskResult<i64> =
              join_result(timeout(spawn(consume_network_result(fallback)), 0));
            let cancelled_result: TaskResult<i64> =
              join_result(cancel(spawn(consume_network_result(config_only))));

            if task_completed(completed_result)
              && task_timed_out(timed_result)
              && task_cancelled(cancelled_result) {
              return task_value(completed_result);
            }
            return 0;
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
        .any(|node| node.op.module == "cpu" && node.op.instruction == "join_result"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "timeout"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "cancel"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_completed"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_timed_out"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_cancelled"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "task_value"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_send_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_recv_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_config_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_network_lane_policy_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_network_result_capability_type"));
}

#[test]
fn lowers_async_kernel_result_recursive_control_flow_observation_path() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn consume_kernel_result(result: KernelResult<i64>) -> i64 {
            if kernel_config_ready(result) && kernel_value(result) > 3 {
              return kernel_value(result) + 10;
            }
            if kernel_config_ready(result) {
              return kernel_value(result) + 2;
            }
            return 0;
          }

          fn main() -> i64 {
            let primary: KernelResult<i64> =
              kernel_result(kernel_profile_queue_depth("KernelUnit"));
            let fallback: KernelResult<i64> =
              kernel_result(kernel_profile_batch_lanes("KernelUnit"));

            let primary_task: Task<i64> = spawn(consume_kernel_result(primary));
            let fallback_task: Task<i64> = spawn(consume_kernel_result(fallback));

            let primary_result: TaskResult<i64> = join_result(primary_task);
            let fallback_result: TaskResult<i64> = join_result(fallback_task);

            if task_completed(primary_result) && task_value(primary_result) > 0 {
              return task_value(primary_result);
            }
            if task_completed(fallback_result) {
              return task_value(fallback_result);
            }
            return 0;
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
        .any(|node| node.op.module == "cpu" && node.op.instruction == "join_result"));
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
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_kernel_lane_policy_type"));
}

#[test]
fn lowers_async_multidomain_result_orchestration_path() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn consume_network_result(result: NetworkResult<i64>) -> i64 {
            if network_send_ready(result) || network_recv_ready(result) {
              return network_value(result) + 1;
            }
            if network_config_ready(result) {
              return network_value(result) + 3;
            }
            return 0;
          }

          async fn consume_kernel_result(result: KernelResult<i64>) -> i64 {
            if kernel_config_ready(result) && kernel_value(result) > 2 {
              return kernel_value(result) + 5;
            }
            if kernel_config_ready(result) {
              return kernel_value(result) + 1;
            }
            return 0;
          }

          async fn orchestrate(seed: i64) -> i64 {
            let payload: DataResult<i64> =
              data_result(data_input_pipe(data_output_pipe(seed)));
            let base: i64 = if data_ready(payload) {
              data_value(payload)
            } else {
              0
            };

            let net_probe: NetworkResult<i64> =
              network_result(network_profile_send_window("NetworkUnit"));
            let kernel_probe: KernelResult<i64> =
              kernel_result(kernel_profile_batch_lanes("KernelUnit"));

            let network_task: Task<i64> = spawn(consume_network_result(net_probe));
            let kernel_task: Task<i64> = spawn(consume_kernel_result(kernel_probe));

            let network_result_joined: TaskResult<i64> = join_result(network_task);
            let kernel_result_joined: TaskResult<i64> = join_result(kernel_task);

            if task_completed(network_result_joined) && task_completed(kernel_result_joined) {
              return base + task_value(network_result_joined) + task_value(kernel_result_joined);
            }
            if task_completed(network_result_joined) {
              return base + task_value(network_result_joined);
            }
            if task_completed(kernel_result_joined) {
              return base + task_value(kernel_result_joined);
            }
            return base;
          }

          async fn main() {
            await orchestrate(7);
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();

    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "async_call"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "await"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "spawn_task"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "join_result"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "data" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "data" && node.op.instruction == "is_ready"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "data" && node.op.instruction == "value"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "observe"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "network" && node.op.instruction == "is_send_ready"));
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
        .any(|node| node.name == "scheduler_contract_network_lane_policy_type"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name == "scheduler_contract_kernel_lane_policy_type"));
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
