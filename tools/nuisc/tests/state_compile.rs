use std::path::Path;

use nuis_semantics::model::{NirExpr, NirStmt};

fn compiled_project(path: &str) -> nuisc::pipeline::PipelineArtifacts {
    nuisc::pipeline::compile_project(Path::new(path))
        .unwrap_or_else(|error| panic!("project `{path}` should compile: {error}"))
}

fn expect_const_i64_value(
    artifacts: &nuisc::pipeline::PipelineArtifacts,
    node_name: &str,
    value: &str,
) {
    assert!(
        artifacts.yir.nodes.iter().any(|node| {
            node.name == node_name
                && node.op.module == "cpu"
                && node.op.instruction == "const_i64"
                && node.op.args.last().is_some_and(|arg| arg == value)
        }),
        "expected const node `{node_name}` with value `{value}`"
    );
}

#[test]
fn compiles_generic_payload_alias_higher_order_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_payload_alias_higher_order_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("generic payload alias higher-order state project should compile");
}

#[test]
fn compiles_generic_payload_alias_method_hof_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_payload_alias_method_hof_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("generic payload alias method higher-order state project should compile");
}

#[test]
fn compiles_generic_callable_forwarding_hof_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_callable_forwarding_hof_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("generic callable forwarding higher-order state project should compile");
}

#[test]
fn lowers_generic_callable_forwarding_hof_state_project_with_forwarded_fn2_and_fn3_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_callable_forwarding_hof_demo",
    );

    for prefix in [
        "__hof_relay2_",
        "__hof_chain2_",
        "__hof_apply2_",
        "__hof_relay3_",
        "__hof_chain3_",
        "__hof_apply3_",
    ] {
        assert!(
            artifacts
                .nir
                .functions
                .iter()
                .any(|function| function.name.starts_with(prefix)
                    && function.name.ends_with("__i64")),
            "expected project to emit `{prefix}...__i64` higher-order helper"
        );
    }

    assert!(artifacts.nir.functions.iter().any(|function| {
        function.name.starts_with("__lambda_chain2_") && function.name.ends_with("__i64")
    }));
    assert!(artifacts.nir.functions.iter().any(|function| {
        function.name.starts_with("__lambda_chain3_") && function.name.ends_with("__i64")
    }));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name.starts_with("__lambda_main_")));

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("expected main function");
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::Call { callee, .. },
            } if name == "pair"
                && ty.render() == "i64"
                && callee.starts_with("__hof_relay2_")
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::Call { callee, .. },
            } if name == "triple"
                && ty.render() == "i64"
                && callee.starts_with("__hof_relay3_")
        )
    }));
}

#[test]
fn compiles_glm_borrow_end_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/glm_borrow_end_state_demo",
    );
    nuisc::pipeline::compile_project(project).expect("glm borrow_end state project should compile");
}

#[test]
fn lowers_glm_borrow_end_state_project_with_borrow_end_then_owner_write_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/glm_borrow_end_state_demo",
    );

    let borrow_ends = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "borrow_end")
        .count();
    assert!(borrow_ends >= 1, "expected explicit borrow closure path");

    let lowered_ops = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu")
        .map(|node| node.op.instruction.as_str())
        .collect::<Vec<_>>();
    let borrow_end_index = lowered_ops
        .iter()
        .position(|op| *op == "borrow_end")
        .expect("expected borrow_end op");
    let store_value_index = lowered_ops
        .iter()
        .position(|op| *op == "store_value")
        .expect("expected owner store_value op");
    assert!(
        borrow_end_index < store_value_index,
        "expected borrow_end to lower before owner write, got {lowered_ops:?}"
    );
}

#[test]
fn compiles_if_borrow_end_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/if_borrow_end_state_demo",
    );
    nuisc::pipeline::compile_project(project).expect("if borrow_end state project should compile");
}

#[test]
fn lowers_if_borrow_end_state_project_with_borrow_end_then_owner_write_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/if_borrow_end_state_demo",
    );

    let borrow_ends = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "borrow_end")
        .count();
    assert!(borrow_ends >= 1, "expected explicit borrow closure path");

    let lowered_ops = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu")
        .map(|node| node.op.instruction.as_str())
        .collect::<Vec<_>>();
    let borrow_end_index = lowered_ops
        .iter()
        .position(|op| *op == "borrow_end")
        .expect("expected borrow_end op");
    let store_value_index = lowered_ops
        .iter()
        .position(|op| *op == "store_value")
        .expect("expected owner store_value op");
    assert!(
        borrow_end_index < store_value_index,
        "expected borrow_end to lower before owner write, got {lowered_ops:?}"
    );
}

#[test]
fn compiles_match_borrow_end_shared_suffix_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_borrow_end_shared_suffix_state_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("match borrow_end shared suffix state project should compile");
}

#[test]
fn lowers_match_borrow_end_shared_suffix_state_project_with_shared_suffix_after_select_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_borrow_end_shared_suffix_state_demo",
    );

    let borrow_ends = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "borrow_end")
        .count();
    assert!(borrow_ends >= 1, "expected explicit borrow closure path");

    let lowered_ops = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu")
        .map(|node| node.op.instruction.as_str())
        .collect::<Vec<_>>();
    let select_index = lowered_ops
        .iter()
        .position(|op| *op == "select")
        .expect("expected select op for shared branch value");
    let borrow_end_index = lowered_ops
        .iter()
        .position(|op| *op == "borrow_end")
        .expect("expected borrow_end op");
    let shared_suffix_add_index = lowered_ops
        .iter()
        .rposition(|op| *op == "add")
        .expect("expected shared suffix add op");
    let store_value_index = lowered_ops
        .iter()
        .position(|op| *op == "store_value")
        .expect("expected owner store_value op");
    assert!(
        select_index < borrow_end_index,
        "expected branch select before shared borrow_end suffix, got {lowered_ops:?}"
    );
    assert!(
        borrow_end_index < shared_suffix_add_index,
        "expected shared suffix add after shared borrow_end, got {lowered_ops:?}"
    );
    assert!(
        shared_suffix_add_index < store_value_index,
        "expected shared suffix add before owner write, got {lowered_ops:?}"
    );
}

#[test]
fn compiles_generic_shared_suffix_if_method_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_shared_suffix_if_method_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("generic shared suffix if-method state project should compile");
}

#[test]
fn lowers_generic_shared_suffix_if_method_state_project_with_select_then_method_suffix_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_shared_suffix_if_method_demo",
    );

    let select_nodes = artifacts
        .yir
        .nodes
        .iter()
        .filter(|stmt| {
            matches!(
                stmt,
                yir_core::Node {
                    op: yir_core::Operation { module, instruction, .. },
                    ..
                } if module == "cpu" && instruction == "select"
            )
        })
        .count();
    assert!(
        select_nodes >= 1,
        "expected shared-branch select in YIR lowering"
    );

    let select_index = artifacts
        .yir
        .nodes
        .iter()
        .position(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .expect("expected shared-branch select node");

    let post_select_value_ops = artifacts
        .yir
        .nodes
        .iter()
        .enumerate()
        .filter(|stmt| {
            matches!(
                stmt,
                (
                    index,
                    yir_core::Node {
                        op: yir_core::Operation { module, instruction, .. },
                        ..
                    }
                ) if *index > select_index
                    && module == "cpu"
                    && (instruction == "add" || instruction == "call_i64")
            )
        })
        .count();
    assert!(
        post_select_value_ops >= 1,
        "expected at least one value-composition op after select in YIR"
    );
}

#[test]
fn compiles_task_result_shared_suffix_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/task_result_shared_suffix_state_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task result shared suffix state project should compile");
}

#[test]
fn lowers_task_result_shared_suffix_state_project_with_select_then_suffix_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/task_result_shared_suffix_state_demo",
    );

    let lowered_ops = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu")
        .map(|node| node.op.instruction.as_str())
        .collect::<Vec<_>>();
    let task_value_index = lowered_ops
        .iter()
        .position(|op| *op == "task_value")
        .expect("expected task_value op");
    let select_index = lowered_ops
        .iter()
        .position(|op| *op == "select")
        .expect("expected select op for task-result branch");
    let suffix_add_index = lowered_ops
        .iter()
        .rposition(|op| *op == "add")
        .expect("expected suffix add op");
    assert!(
        task_value_index < select_index,
        "expected task_value to feed branch select, got {lowered_ops:?}"
    );
    assert!(
        select_index < suffix_add_index,
        "expected suffix add after branch select, got {lowered_ops:?}"
    );
}

#[test]
fn compiles_buffer_shared_suffix_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/buffer_shared_suffix_state_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("buffer shared suffix state project should compile");
}

#[test]
fn lowers_buffer_shared_suffix_state_project_with_select_then_store_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/buffer_shared_suffix_state_demo",
    );

    let lowered_ops = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu")
        .map(|node| node.op.instruction.as_str())
        .collect::<Vec<_>>();
    let select_index = lowered_ops
        .iter()
        .position(|op| *op == "select")
        .expect("expected select op for buffer branch");
    let final_store_at_index = lowered_ops
        .iter()
        .rposition(|op| *op == "store_at")
        .expect("expected final store_at op");
    let replay_load_index = lowered_ops
        .iter()
        .rposition(|op| *op == "load_at")
        .expect("expected replay load after shared store");
    assert!(
        select_index < final_store_at_index,
        "expected shared buffer store after branch select, got {lowered_ops:?}"
    );
    assert!(
        final_store_at_index < replay_load_index,
        "expected replay load after shared buffer store, got {lowered_ops:?}"
    );
}

#[test]
fn lowers_generic_payload_alias_method_hof_state_project_with_hof_and_lambda_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/generic_payload_alias_method_hof_demo",
    );

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("expected main function");
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee.starts_with("__hof_apply_payload")
    ));

    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| { function.name.starts_with("__hof_apply_payload") }));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| { function.name.starts_with("__lambda_main_") }));

    let hof = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_payload"))
        .expect("expected monomorphized higher-order helper");
    assert!(matches!(
        hof.body.first(),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
        }) if name == "payload" && ty.render() == "i64" && field == "value"
    ));
    assert!(hof.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::Call { callee, .. },
                ..
            } if callee.starts_with("__lambda_main_")
        )
    }));
}

#[test]
fn compiles_ordinary_mutual_recursive_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_mutual_recursive_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary mutual recursive state project should compile");
}

#[test]
fn compiles_ordinary_recursive_scalar_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive scalar state project should compile");
}

#[test]
fn lowers_ordinary_recursive_scalar_state_project_with_scalar_helper_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_call_graph_demo",
    );

    for lane in ["fn:step", "fn:odd", "fn:even"] {
        assert!(
            artifacts.yir.node_lanes.values().any(|value| value == lane),
            "expected scalar recursive project to emit lane `{lane}`"
        );
    }

    let step_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| name == "step")
        })
        .count();
    let recursive_calls = artifacts
        .yir
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
        step_calls >= 1,
        "expected scalar recursive project to preserve scalar helper calls"
    );
    assert!(
        recursive_calls >= 2,
        "expected scalar recursive project to preserve mutual recursion"
    );
}

#[test]
fn compiles_ordinary_recursive_i32_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_i32_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive i32 state project should compile");
}

#[test]
fn lowers_ordinary_recursive_i32_state_project_with_i32_helper_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_i32_call_graph_demo",
    );

    for lane in ["fn:step", "fn:odd", "fn:even"] {
        assert!(
            artifacts.yir.node_lanes.values().any(|value| value == lane),
            "expected i32 recursive project to emit lane `{lane}`"
        );
    }

    let step_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i32"
                && node.op.args.first().is_some_and(|name| name == "step")
        })
        .count();
    let recursive_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i32"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "odd" || name == "even")
        })
        .count();
    assert!(
        step_calls >= 1,
        "expected i32 recursive project to preserve i32 helper calls"
    );
    assert!(
        recursive_calls >= 2,
        "expected i32 recursive project to preserve i32 mutual recursion"
    );
    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| { node.op.module == "cpu" && node.op.instruction == "guard_return" }));
}

#[test]
fn compiles_ordinary_recursive_match_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_match_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive match state project should compile");
}

#[test]
fn lowers_ordinary_recursive_match_state_project_into_recursive_helper_lanes() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_match_call_graph_demo",
    );

    let call_i64_count = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "call_i64")
        .count();
    assert!(
        call_i64_count >= 2,
        "expected recursive match project to emit recursive helper calls, found {call_i64_count}"
    );
    assert!(artifacts
        .yir
        .node_lanes
        .values()
        .any(|lane| lane == "fn:odd"));
    assert!(artifacts
        .yir
        .node_lanes
        .values()
        .any(|lane| lane == "fn:even"));
}

#[test]
fn compiles_ordinary_recursive_bool_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_bool_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive bool state project should compile");
}

#[test]
fn lowers_ordinary_recursive_bool_state_project_with_bool_helper_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_bool_call_graph_demo",
    );

    for lane in ["fn:flip", "fn:odd", "fn:even"] {
        assert!(
            artifacts.yir.node_lanes.values().any(|value| value == lane),
            "expected bool recursive project to emit lane `{lane}`"
        );
    }

    let flip_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_bool"
                && node.op.args.first().is_some_and(|name| name == "flip")
        })
        .count();
    let recursive_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_bool"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "odd" || name == "even")
        })
        .count();
    assert!(
        flip_calls >= 1,
        "expected bool recursive project to preserve bool helper calls"
    );
    assert!(
        recursive_calls >= 2,
        "expected bool recursive project to preserve mutual bool recursion"
    );
    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| { node.op.module == "cpu" && node.op.instruction == "guard_return" }));
}

#[test]
fn compiles_ordinary_recursive_higher_order_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_higher_order_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive higher-order state project should compile");
}

#[test]
fn lowers_ordinary_recursive_higher_order_state_project_with_named_helper_hof_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_higher_order_call_graph_demo",
    );

    assert!(artifacts
        .yir
        .node_lanes
        .values()
        .any(|lane| lane == "fn:dec"));
    assert!(artifacts
        .yir
        .node_lanes
        .values()
        .any(|lane| lane.starts_with("fn:__hof_apply_")));

    let dec_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| name == "dec")
        })
        .count();
    let hof_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name.starts_with("__hof_apply_"))
        })
        .count();
    assert!(
        dec_calls >= 1,
        "expected higher-order recursive project to preserve named helper calls"
    );
    assert!(
        hof_calls >= 2,
        "expected higher-order recursive project to preserve helper-lowered recursion"
    );
}

#[test]
fn compiles_ordinary_recursive_fn2_higher_order_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_fn2_higher_order_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive fn2 higher-order state project should compile");
}

#[test]
fn lowers_ordinary_recursive_fn2_higher_order_state_project_with_recursive_hof_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_fn2_higher_order_call_graph_demo",
    );

    assert!(artifacts
        .yir
        .node_lanes
        .values()
        .any(|lane| lane.starts_with("fn:__hof_apply2_")));
    let lambda_lane_count = artifacts
        .yir
        .node_lanes
        .values()
        .filter(|lane| {
            lane.starts_with("fn:__lambda_odd_") || lane.starts_with("fn:__lambda_even_")
        })
        .count();
    assert!(
        lambda_lane_count >= 2,
        "expected fn2 recursive project to emit lambda helper lanes, found {lambda_lane_count}"
    );

    let hof_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name.starts_with("__hof_apply2_"))
        })
        .count();
    let lambda_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("__lambda_odd_") || name.starts_with("__lambda_even_")
                })
        })
        .count();
    assert!(
        hof_calls >= 2,
        "expected fn2 recursive project to preserve helper-lowered recursion"
    );
    assert!(
        lambda_calls >= 2,
        "expected fn2 recursive project to preserve synthesized lambda calls"
    );
}

#[test]
fn compiles_ordinary_recursive_generic_higher_order_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_higher_order_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive generic higher-order state project should compile");
}

#[test]
fn lowers_ordinary_recursive_generic_higher_order_state_project_with_specialized_hof_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_higher_order_call_graph_demo",
    );

    assert!(artifacts.nir.functions.iter().any(|function| {
        function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
    }));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name.starts_with("__lambda_odd_")));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name.starts_with("__lambda_even_")));

    let hof_calls =
        artifacts
            .yir
            .nodes
            .iter()
            .filter(|node| {
                node.op.module == "cpu"
                    && node.op.instruction == "call_i64"
                    && node.op.args.first().is_some_and(|name| {
                        name.starts_with("__hof_apply_") && name.ends_with("__i64")
                    })
            })
            .count();
    let lambda_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("__lambda_odd_") || name.starts_with("__lambda_even_")
                })
        })
        .count();
    assert!(
        hof_calls >= 2,
        "expected generic higher-order recursive project to preserve specialized helper calls"
    );
    assert!(
        lambda_calls >= 2,
        "expected generic higher-order recursive project to preserve specialized lambda calls"
    );
}

#[test]
fn compiles_ordinary_recursive_generic_fn2_higher_order_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_fn2_higher_order_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive generic fn2 higher-order state project should compile");
}

#[test]
fn lowers_ordinary_recursive_generic_fn2_higher_order_state_project_with_specialized_hof_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_fn2_higher_order_call_graph_demo",
    );

    assert!(artifacts.nir.functions.iter().any(|function| {
        function.name.starts_with("__hof_apply2_") && function.name.ends_with("__i64")
    }));
    let lambda_lane_count = artifacts
        .yir
        .node_lanes
        .values()
        .filter(|lane| {
            lane.starts_with("fn:__lambda_odd_") || lane.starts_with("fn:__lambda_even_")
        })
        .count();
    assert!(
        lambda_lane_count >= 2,
        "expected generic fn2 recursive project to emit lambda helper lanes, found {lambda_lane_count}"
    );

    let hof_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("__hof_apply2_") && name.ends_with("__i64")
                })
        })
        .count();
    let lambda_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("__lambda_odd_") || name.starts_with("__lambda_even_")
                })
        })
        .count();
    assert!(
        hof_calls >= 2,
        "expected generic fn2 recursive project to preserve specialized helper calls"
    );
    assert!(
        lambda_calls >= 2,
        "expected generic fn2 recursive project to preserve synthesized lambda calls"
    );
}

#[test]
fn compiles_ordinary_recursive_generic_fn3_higher_order_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_fn3_higher_order_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive generic fn3 higher-order state project should compile");
}

#[test]
fn lowers_ordinary_recursive_generic_fn3_higher_order_state_project_with_specialized_hof_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_fn3_higher_order_call_graph_demo",
    );

    assert!(artifacts.nir.functions.iter().any(|function| {
        function.name.starts_with("__hof_apply3_") && function.name.ends_with("__i64")
    }));
    let lambda_lane_count = artifacts
        .yir
        .node_lanes
        .values()
        .filter(|lane| {
            lane.starts_with("fn:__lambda_odd_") || lane.starts_with("fn:__lambda_even_")
        })
        .count();
    assert!(
        lambda_lane_count >= 2,
        "expected generic fn3 recursive project to emit lambda helper lanes, found {lambda_lane_count}"
    );

    let hof_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("__hof_apply3_") && name.ends_with("__i64")
                })
        })
        .count();
    let lambda_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("__lambda_odd_") || name.starts_with("__lambda_even_")
                })
        })
        .count();
    assert!(
        hof_calls >= 2,
        "expected generic fn3 recursive project to preserve specialized helper calls"
    );
    assert!(
        lambda_calls >= 2,
        "expected generic fn3 recursive project to preserve synthesized lambda calls"
    );
}

#[test]
fn compiles_ordinary_recursive_generic_alias_higher_order_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_alias_higher_order_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive generic alias higher-order state project should compile");
}

#[test]
fn lowers_ordinary_recursive_generic_alias_higher_order_state_project_with_specialized_hof_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_alias_higher_order_call_graph_demo",
    );

    assert!(artifacts.nir.functions.iter().any(|function| {
        function.name.starts_with("__hof_apply_") && function.name.ends_with("__i64")
    }));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name.starts_with("__lambda_odd_")));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name.starts_with("__lambda_even_")));

    let hof_calls =
        artifacts
            .yir
            .nodes
            .iter()
            .filter(|node| {
                node.op.module == "cpu"
                    && node.op.instruction == "call_i64"
                    && node.op.args.first().is_some_and(|name| {
                        name.starts_with("__hof_apply_") && name.ends_with("__i64")
                    })
            })
            .count();
    let lambda_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("__lambda_odd_") || name.starts_with("__lambda_even_")
                })
        })
        .count();
    assert!(
        hof_calls >= 2,
        "expected generic alias higher-order recursive project to preserve specialized helper calls"
    );
    assert!(
        lambda_calls >= 2,
        "expected generic alias higher-order recursive project to preserve specialized lambda calls"
    );
}

#[test]
fn compiles_ordinary_recursive_generic_alias_fn3_higher_order_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_alias_fn3_higher_order_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive generic alias fn3 higher-order state project should compile");
}

#[test]
fn lowers_ordinary_recursive_generic_alias_fn3_higher_order_state_project_with_recursive_hof_shape()
{
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_alias_fn3_higher_order_call_graph_demo",
    );

    assert!(artifacts.nir.functions.iter().any(|function| {
        function.name.starts_with("__hof_apply3_") && function.name.ends_with("__i64")
    }));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| { function.name.starts_with("__lambda_odd_") }));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| { function.name.starts_with("__lambda_even_") }));

    let hof_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("__hof_apply3_") && name.ends_with("__i64")
                })
        })
        .count();
    let lambda_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name.starts_with("__lambda_odd_") || name.starts_with("__lambda_even_")
                })
        })
        .count();
    assert!(
        hof_calls >= 2,
        "expected recursive project to emit helper-lowered Fn3 higher-order calls, found {hof_calls}"
    );
    assert!(
        lambda_calls >= 2,
        "expected recursive project to emit synthesized lambda calls, found {lambda_calls}"
    );
}

#[test]
fn compiles_ordinary_recursive_composed_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_composed_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive composed state project should compile");
}

#[test]
fn lowers_ordinary_recursive_composed_state_project_with_composed_helper_lane_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_composed_call_graph_demo",
    );

    for lane in [
        "fn:odd",
        "fn:even",
        "fn:near_zero",
        "fn:step",
        "fn:__lambda_odd_0",
        "fn:__lambda_even_0",
        "fn:__hof_apply___lambda_odd_0",
        "fn:__hof_apply___lambda_even_0",
    ] {
        assert!(
            artifacts.yir.node_lanes.values().any(|value| value == lane),
            "expected composed recursive project to emit lane `{lane}`"
        );
    }

    let odd_hof_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "__hof_apply___lambda_odd_0")
        })
        .count();
    let even_hof_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "__hof_apply___lambda_even_0")
        })
        .count();
    let odd_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| name == "odd")
        })
        .count();
    let even_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| name == "even")
        })
        .count();
    let step_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i32"
                && node.op.args.first().is_some_and(|name| name == "step")
        })
        .count();
    assert!(
        odd_hof_calls >= 1,
        "expected composed recursive project to call odd helper hof lane"
    );
    assert!(
        even_hof_calls >= 1,
        "expected composed recursive project to call even helper hof lane"
    );
    assert!(
        odd_calls >= 1 && even_calls >= 1,
        "expected composed recursive project to preserve odd/even recursion"
    );
    assert!(
        step_calls >= 1,
        "expected composed recursive project to preserve scalar helper call graph"
    );
}

#[test]
fn compiles_ordinary_recursive_lambda_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_lambda_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive lambda state project should compile");
}

#[test]
fn lowers_ordinary_recursive_lambda_state_project_with_lambda_helper_lane_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_lambda_call_graph_demo",
    );

    for lane in [
        "fn:odd",
        "fn:even",
        "fn:__lambda_odd_0",
        "fn:__lambda_even_0",
        "fn:__hof_apply___lambda_odd_0",
        "fn:__hof_apply___lambda_even_0",
    ] {
        assert!(
            artifacts.yir.node_lanes.values().any(|value| value == lane),
            "expected lambda recursive project to emit lane `{lane}`"
        );
    }

    let hof_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name == "__hof_apply___lambda_odd_0" || name == "__hof_apply___lambda_even_0"
                })
        })
        .count();
    let odd_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| name == "odd")
        })
        .count();
    let even_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| name == "even")
        })
        .count();
    assert!(
        hof_calls >= 2,
        "expected lambda recursive project to emit both helper-lowered lambda calls"
    );
    assert!(
        odd_calls >= 1 && even_calls >= 1,
        "expected lambda recursive project to preserve odd/even recursion"
    );
}

#[test]
fn compiles_ordinary_recursive_mixed_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_mixed_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive mixed state project should compile");
}

#[test]
fn lowers_ordinary_recursive_mixed_state_project_with_bool_recursive_helper_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_mixed_call_graph_demo",
    );

    for lane in ["fn:odd", "fn:even", "fn:is_zero", "fn:step"] {
        assert!(
            artifacts.yir.node_lanes.values().any(|value| value == lane),
            "expected mixed recursive project to emit lane `{lane}`"
        );
    }

    let bool_recursive_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_bool"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "odd" || name == "even")
        })
        .count();
    let step_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i32"
                && node.op.args.first().is_some_and(|name| name == "step")
        })
        .count();
    assert!(
        bool_recursive_calls >= 2,
        "expected mixed recursive project to preserve bool-returning mutual recursion"
    );
    assert!(
        step_calls >= 1,
        "expected mixed recursive project to preserve scalar step helper"
    );
    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| { node.op.module == "cpu" && node.op.instruction == "guard_return" }));
}

#[test]
fn compiles_ordinary_recursive_generic_composed_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_composed_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive generic composed state project should compile");
}

#[test]
fn lowers_ordinary_recursive_generic_composed_state_project_with_specialized_hof_recursive_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/ordinary_recursive_generic_composed_call_graph_demo",
    );

    for function in [
        "__hof_apply___lambda_odd_0__i64",
        "__hof_apply___lambda_even_0__i64",
        "__lambda_odd_0",
        "__lambda_even_0",
    ] {
        assert!(
            artifacts
                .nir
                .functions
                .iter()
                .any(|item| item.name == function),
            "expected generic composed recursive project to emit `{function}`"
        );
    }

    let hof_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| {
                    name == "__hof_apply___lambda_odd_0__i64"
                        || name == "__hof_apply___lambda_even_0__i64"
                })
        })
        .count();
    let odd_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| name == "odd")
        })
        .count();
    let even_calls = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| name == "even")
        })
        .count();
    assert!(
        hof_calls >= 2,
        "expected generic composed recursive project to emit specialized helper-lowered calls"
    );
    assert!(
        odd_calls >= 1 && even_calls >= 1,
        "expected generic composed recursive project to preserve odd/even recursion"
    );
}

#[test]
fn compiles_tail_recursive_sum_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_sum_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive sum state project should compile");
}

#[test]
fn lowers_tail_recursive_sum_state_project_with_chain_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_sum_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "add_current");
}

#[test]
fn compiles_tail_recursive_factorial_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_factorial_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive factorial state project should compile");
}

#[test]
fn lowers_tail_recursive_factorial_state_project_with_multiplicative_chain_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_factorial_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "mul_prev_current");
}

#[test]
fn compiles_tail_recursive_factorial_affine_mul_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_factorial_affine_mul_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive factorial affine mul state project should compile");
}

#[test]
fn lowers_tail_recursive_factorial_affine_mul_state_project_with_affine_multiplicative_chain_shape()
{
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_factorial_affine_mul_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "mul_prev_current_plus_invariant");
    assert!(loop_node.op.args[7].starts_with("int_"));
}

#[test]
fn compiles_tail_recursive_factorial_scaled_mul_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_factorial_scaled_mul_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive factorial scaled mul state project should compile");
}

#[test]
fn lowers_tail_recursive_factorial_scaled_mul_state_project_with_scaled_multiplicative_chain_shape()
{
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_factorial_scaled_mul_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "mul_scaled_current_plus_invariant");
    assert!(loop_node.op.args[7].starts_with("int_"));
    assert!(loop_node.op.args[8].starts_with("mul_"));
}

#[test]
fn compiles_counted_while_multi_carry_scaled_mul_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/counted_while_multi_carry_scaled_mul_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("counted while multi-carry scaled mul state project should compile");
}

#[test]
fn lowers_counted_while_multi_carry_scaled_mul_state_project_with_multi_carry_scaled_mul_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/counted_while_multi_carry_scaled_mul_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "add_current");
    assert_eq!(loop_node.op.args[8], "mul_scaled_current_plus_carry0");
    assert!(loop_node.op.args[9].starts_with("int_"));
}

#[test]
fn compiles_counted_while_multi_carry_state_factor_mul_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/counted_while_multi_carry_state_factor_mul_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("counted while multi-carry state-factor mul state project should compile");
}

#[test]
fn lowers_counted_while_multi_carry_state_factor_mul_state_project_with_state_factor_mul_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/counted_while_multi_carry_state_factor_mul_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "add_current");
    assert_eq!(
        loop_node.op.args[8],
        "mul_scaled_by_current_current_plus_carry0"
    );
}

#[test]
fn compiles_counted_while_multi_carry_state_plus_invariant_factor_mul_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/counted_while_multi_carry_state_plus_invariant_factor_mul_demo",
    );
    nuisc::pipeline::compile_project(project).expect(
        "counted while multi-carry state-plus-invariant-factor mul state project should compile",
    );
}

#[test]
fn lowers_counted_while_multi_carry_state_plus_invariant_factor_mul_state_project_with_state_plus_invariant_factor_mul_shape(
) {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/counted_while_multi_carry_state_plus_invariant_factor_mul_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "add_current");
    assert_eq!(
        loop_node.op.args[8],
        "mul_scaled_by_current_plus_factor_invariant_current_plus_carry0"
    );
    assert!(loop_node.op.args[9].starts_with("int_"));
}

#[test]
fn compiles_tail_recursive_cross_carry_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_cross_carry_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive cross-carry state project should compile");
}

#[test]
fn lowers_tail_recursive_cross_carry_state_project_with_cross_carry_chain_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_cross_carry_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "add_prev_carry1");
    assert_eq!(loop_node.op.args[8], "add_prev_current");
}

#[test]
fn compiles_tail_recursive_branching_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_branching_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive branching state project should compile");
}

#[test]
fn lowers_tail_recursive_branching_state_project_with_branching_cond_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_branching_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_cond_chain"
        })
        .expect("expected loop_while_scalar_cond_chain node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "prev_current_gt");
    expect_const_i64_value(&artifacts, &loop_node.op.args[7], "2");
    assert_eq!(loop_node.op.args[8], "add_current");
    assert_eq!(loop_node.op.args[9], "keep");
}

#[test]
fn compiles_tail_recursive_keep_prev_carry_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_keep_prev_carry_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive keep-prev-carry state project should compile");
}

#[test]
fn lowers_tail_recursive_keep_prev_carry_state_project_with_branching_cond_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_keep_prev_carry_demo",
    );

    let loop_node = artifacts
        .yir
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
}

#[test]
fn compiles_tail_recursive_multi_carry_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_multi_carry_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive multi-carry state project should compile");
}

#[test]
fn lowers_tail_recursive_multi_carry_state_project_with_multi_carry_chain_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_multi_carry_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "add_prev_current");
    assert_eq!(loop_node.op.args[8], "mul_prev_current");
}

#[test]
fn compiles_tail_recursive_carry_condition_multi_carry_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_carry_condition_multi_carry_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive carry-condition multi-carry state project should compile");
}

#[test]
fn lowers_tail_recursive_carry_condition_multi_carry_state_project_with_carry_condition_cond_loop_shape(
) {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_carry_condition_multi_carry_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_cond_chain"
        })
        .expect("expected loop_while_scalar_cond_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "prev_carry0_gt");
    expect_const_i64_value(&artifacts, &loop_node.op.args[7], "3");
    assert_eq!(loop_node.op.args[8], "keep");
    assert_eq!(loop_node.op.args[9], "add_prev_current");
    assert_eq!(loop_node.op.args[11], "prev_carry0_gt");
    expect_const_i64_value(&artifacts, &loop_node.op.args[12], "3");
    assert_eq!(loop_node.op.args[13], "add_prev_current");
    assert_eq!(loop_node.op.args[14], "mul_prev_current");
}

#[test]
fn compiles_tail_recursive_branching_cross_carry_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_branching_cross_carry_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive branching cross-carry state project should compile");
}

#[test]
fn lowers_tail_recursive_branching_cross_carry_state_project_with_cond_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_branching_cross_carry_demo",
    );

    let loop_node = artifacts
        .yir
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
}

#[test]
fn compiles_tail_recursive_branching_multi_carry_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_branching_multi_carry_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive branching multi-carry state project should compile");
}

#[test]
fn lowers_tail_recursive_branching_multi_carry_state_project_with_cond_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_branching_multi_carry_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_cond_chain"
        })
        .expect("expected loop_while_scalar_cond_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "prev_current_gt");
    expect_const_i64_value(&artifacts, &loop_node.op.args[7], "2");
    assert_eq!(loop_node.op.args[8], "add_prev_current");
    assert_eq!(loop_node.op.args[9], "keep");
    assert_eq!(loop_node.op.args[11], "prev_current_gt");
    expect_const_i64_value(&artifacts, &loop_node.op.args[12], "2");
    assert_eq!(loop_node.op.args[13], "mul_prev_current");
    assert_eq!(loop_node.op.args[14], "add_prev_current");
}

#[test]
fn compiles_flow_branching_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/flow_branching_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("flow branching while state project should compile");
}

#[test]
fn compiles_counted_while_state_project() {
    let project =
        Path::new("/Users/Shared/chroot/dev/nuislang/examples/projects/state/counted_while_demo");
    nuisc::pipeline::compile_project(project).expect("counted while state project should compile");
}

#[test]
fn lowers_counted_while_state_project_with_basic_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/counted_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64")
        .expect("expected loop_while_i64 node");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "add");
}

#[test]
fn compiles_accumulating_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/accumulating_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("accumulating while state project should compile");
}

#[test]
fn lowers_accumulating_while_state_project_with_single_carry_chain_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/accumulating_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[6], "add_current");
}

#[test]
fn compiles_chained_while_state_project() {
    let project =
        Path::new("/Users/Shared/chroot/dev/nuislang/examples/projects/state/chained_while_demo");
    nuisc::pipeline::compile_project(project).expect("chained while state project should compile");
}

#[test]
fn lowers_chained_while_state_project_with_multi_carry_chain_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/chained_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_chain")
        .expect("expected loop_while_scalar_chain node");
    assert_eq!(loop_node.op.args[6], "add_current");
    assert_eq!(loop_node.op.args[8], "add_carry0");
}

#[test]
fn compiles_match_branching_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_branching_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("match branching while state project should compile");
}

#[test]
fn lowers_match_branching_while_state_project_with_cond_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_branching_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_cond_chain"
        })
        .expect("expected loop_while_scalar_cond_chain node");
    assert_eq!(loop_node.op.args[6], "current_eq");
    assert_eq!(loop_node.op.args[8], "add_current");
    assert_eq!(loop_node.op.args[9], "keep");
}

#[test]
fn compiles_branching_while_state_project() {
    let project =
        Path::new("/Users/Shared/chroot/dev/nuislang/examples/projects/state/branching_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("branching while state project should compile");
}

#[test]
fn lowers_branching_while_state_project_with_plain_cond_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/branching_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_cond_chain"
        })
        .expect("expected loop_while_scalar_cond_chain node");
    assert_eq!(loop_node.op.args[6], "current_gt");
    expect_const_i64_value(&artifacts, &loop_node.op.args[7], "2");
    assert_eq!(loop_node.op.args[8], "add_current");
    assert_eq!(loop_node.op.args[9], "keep");
}

#[test]
fn compiles_bool_match_branching_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/bool_match_branching_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("bool match branching while state project should compile");
}

#[test]
fn lowers_bool_match_branching_while_state_project_with_cond_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/bool_match_branching_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_cond_chain"
        })
        .expect("expected loop_while_scalar_cond_chain node");
    assert_eq!(loop_node.op.args[6], "current_gt");
    assert_eq!(loop_node.op.args[8], "add_current");
    assert_eq!(loop_node.op.args[9], "keep");
}

#[test]
fn compiles_lambda_match_branching_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/lambda_match_branching_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("lambda match branching while state project should compile");
}

#[test]
fn lowers_lambda_match_branching_while_state_project_with_lambda_cond_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/lambda_match_branching_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_cond_chain"
        })
        .expect("expected loop_while_scalar_cond_chain node");
    assert_eq!(loop_node.op.args[6], "current_gt");
    expect_const_i64_value(&artifacts, &loop_node.op.args[7], "2");
    assert_eq!(loop_node.op.args[8], "add_current");
    assert_eq!(loop_node.op.args[9], "keep");
}

#[test]
fn compiles_match_expr_branching_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_expr_branching_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("match expression branching while state project should compile");
}

#[test]
fn lowers_match_expr_branching_while_state_project_with_nested_if_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_expr_branching_while_demo",
    );

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("expected main function");
    assert!(matches!(
        &main.body[1],
        NirStmt::While { body, .. }
            if matches!(
                body.as_slice(),
                [NirStmt::If {
                    condition: NirExpr::Binary { .. },
                    then_body,
                    else_body,
                }] if matches!(
                    then_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(7)))]
                ) && matches!(
                    else_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(9)))]
                )
            )
    ));
}

#[test]
fn lowers_flow_branching_while_state_project_with_flow_cond_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/flow_branching_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "current_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "current_gt");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn compiles_equality_branching_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/equality_branching_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("equality branching while state project should compile");
}

#[test]
fn lowers_equality_branching_while_state_project_with_equality_flow_cond_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/equality_branching_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "current_ne");
    expect_const_i64_value(&artifacts, &loop_node.op.args[6], "3");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "current_eq");
    expect_const_i64_value(&artifacts, &loop_node.op.args[10], "3");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn compiles_lambda_match_flow_continuing_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/lambda_match_flow_continuing_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("lambda match flow continuing while state project should compile");
}

#[test]
fn lowers_lambda_match_flow_continuing_while_state_project_with_lambda_flow_cond_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/lambda_match_flow_continuing_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "current_lt");
    expect_const_i64_value(&artifacts, &loop_node.op.args[6], "3");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "current_gt");
    expect_const_i64_value(&artifacts, &loop_node.op.args[10], "4");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn compiles_lambda_match_or_flow_continuing_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/lambda_match_or_flow_continuing_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("lambda match or flow continuing while state project should compile");
}

#[test]
fn lowers_lambda_match_or_flow_continuing_while_state_project_with_or_flow_cond_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/lambda_match_or_flow_continuing_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "or");
    assert_eq!(loop_node.op.args[6], "current_eq");
    expect_const_i64_value(&artifacts, &loop_node.op.args[7], "1");
    assert_eq!(loop_node.op.args[8], "current_eq");
    expect_const_i64_value(&artifacts, &loop_node.op.args[9], "2");
    assert_eq!(loop_node.op.args[10], "continue");
    assert_eq!(loop_node.op.args[12], "current_gt");
    expect_const_i64_value(&artifacts, &loop_node.op.args[13], "4");
    assert_eq!(loop_node.op.args[14], "add_current");
    assert_eq!(loop_node.op.args[15], "keep");
}

#[test]
fn compiles_match_guarded_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_guarded_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("match guarded while state project should compile");
}

#[test]
fn lowers_match_guarded_while_state_project_with_guarded_return_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_guarded_while_demo",
    );

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("expected main function");
    assert!(matches!(
        &main.body[1],
        NirStmt::While { body, .. }
            if matches!(
                body.as_slice(),
                [NirStmt::If {
                    condition: NirExpr::Binary { .. },
                    then_body,
                    else_body,
                }] if matches!(
                    then_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(7)))]
                ) && matches!(
                    else_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(9)))]
                )
            )
    ));
}

#[test]
fn compiles_match_guard_or_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_guard_or_state_demo",
    );
    nuisc::pipeline::compile_project(project).expect("match guard-or state project should compile");
}

#[test]
fn lowers_match_guard_or_state_project_with_guarded_return_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_guard_or_state_demo",
    );

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "guard_return"));
}

#[test]
fn compiles_match_multi_guard_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_multi_guard_state_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("match multi-guard state project should compile");
}

#[test]
fn lowers_match_multi_guard_state_project_with_guarded_return_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_multi_guard_state_demo",
    );

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("expected main function");
    assert!(matches!(
        &main.body[2],
        NirStmt::While { body, .. }
            if matches!(
                body.as_slice(),
                [NirStmt::If {
                    then_body,
                    else_body,
                    ..
                }] if matches!(
                    then_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Int(5)))]
                ) && matches!(
                    else_body.as_slice(),
                    [NirStmt::If {
                        then_body,
                        else_body,
                        ..
                    }] if matches!(
                        then_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(7)))]
                    ) && matches!(
                        else_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(9)))]
                    )
                )
            )
    ));
}

#[test]
fn compiles_match_guard_range_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_guard_range_state_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("match guard-range state project should compile");
}

#[test]
fn lowers_match_guard_range_state_project_with_guarded_return_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_guard_range_state_demo",
    );

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "guard_return"));
}

#[test]
fn compiles_flow_continuing_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/flow_continuing_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("flow continuing while state project should compile");
}

#[test]
fn lowers_flow_continuing_while_state_project_with_continue_flow_cond_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/flow_continuing_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "current_lt");
    expect_const_i64_value(&artifacts, &loop_node.op.args[6], "3");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "current_gt");
    expect_const_i64_value(&artifacts, &loop_node.op.args[10], "4");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn compiles_post_flow_branching_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_branching_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("post-flow branching while state project should compile");
}

#[test]
fn compiles_tail_recursive_post_flow_branching_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_post_flow_branching_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive post-flow branching state project should compile");
}

#[test]
fn compiles_tail_recursive_post_flow_dynamic_prev_carry_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_post_flow_dynamic_prev_carry_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive post-flow dynamic prev-carry state project should compile");
}

#[test]
fn lowers_tail_recursive_post_flow_dynamic_prev_carry_state_project_with_recursive_post_flow_cond_loop_shape(
) {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_post_flow_dynamic_prev_carry_demo",
    );

    let loop_node = artifacts
        .yir
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
    assert!(loop_node
        .op
        .args
        .iter()
        .any(|arg| arg == "add_read_at_dynamic_prev_carry1"));
    assert!(loop_node
        .op
        .args
        .iter()
        .any(|arg| arg.starts_with("alloc_buffer_")));
    assert!(loop_node.op.args.iter().any(|arg| arg == "prev_current_gt"));
    assert!(loop_node
        .op
        .args
        .iter()
        .any(|arg| arg == "add_prev_current"));
    assert!(loop_node.op.args.iter().any(|arg| arg == "keep"));
}

#[test]
fn lowers_tail_recursive_post_flow_branching_state_project_with_recursive_post_flow_cond_loop_shape(
) {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/tail_recursive_post_flow_branching_demo",
    );

    let loop_node = artifacts
        .yir
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
    assert!(
        loop_node.op.args.iter().any(|arg| arg == "or"),
        "expected recursive boolean carry condition"
    );
    assert!(
        loop_node.op.args.iter().any(|arg| arg == "prev_current_gt"),
        "expected nested branch to reference previous current"
    );
    assert!(
        loop_node
            .op
            .args
            .iter()
            .any(|arg| arg == "add_prev_current"),
        "expected carry update to use previous current"
    );
    assert!(
        loop_node.op.args.iter().any(|arg| arg == "keep"),
        "expected fallback carry branch to keep prior value"
    );
}

#[test]
fn lowers_post_flow_branching_while_state_project_with_post_flow_cond_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_branching_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_post_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "carry0_gt");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "current_gt");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}

#[test]
fn compiles_post_flow_breaking_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_breaking_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("post-flow breaking while state project should compile");
}

#[test]
fn lowers_post_flow_breaking_while_state_project_with_post_flow_break_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_breaking_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_post_flow_chain"
        })
        .expect("expected loop_while_scalar_post_flow_chain node");
    assert_eq!(loop_node.op.args[5], "carry0_gt");
    expect_const_i64_value(&artifacts, &loop_node.op.args[6], "6");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn compiles_bounded_while_state_project() {
    let project =
        Path::new("/Users/Shared/chroot/dev/nuislang/examples/projects/state/bounded_while_demo");
    nuisc::pipeline::compile_project(project).expect("bounded while state project should compile");
}

#[test]
fn lowers_bounded_while_state_project_with_bounded_post_flow_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/bounded_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_post_flow_chain"
        })
        .expect("expected loop_while_scalar_post_flow_chain node");
    assert_eq!(loop_node.op.args[3], "le");
    assert_eq!(loop_node.op.args[4], "add");
    assert_eq!(loop_node.op.args[5], "carry0_ge");
    expect_const_i64_value(&artifacts, &loop_node.op.args[6], "6");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn compiles_equality_while_state_project() {
    let project =
        Path::new("/Users/Shared/chroot/dev/nuislang/examples/projects/state/equality_while_demo");
    nuisc::pipeline::compile_project(project).expect("equality while state project should compile");
}

#[test]
fn lowers_equality_while_state_project_with_equality_post_flow_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/equality_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_post_flow_chain"
        })
        .expect("expected loop_while_scalar_post_flow_chain node");
    assert_eq!(loop_node.op.args[5], "carry0_eq");
    expect_const_i64_value(&artifacts, &loop_node.op.args[6], "6");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn compiles_inequality_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/inequality_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("inequality while state project should compile");
}

#[test]
fn lowers_inequality_while_state_project_with_inequality_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/inequality_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64")
        .expect("expected loop_while_i64 node");
    assert_eq!(loop_node.op.args[3], "ne");
    assert_eq!(loop_node.op.args[4], "add");
}

#[test]
fn compiles_post_flow_continuing_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_continuing_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("post-flow continuing while state project should compile");
}

#[test]
fn lowers_post_flow_continuing_while_state_project_with_post_flow_continue_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_continuing_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_post_flow_chain"
        })
        .expect("expected loop_while_scalar_post_flow_chain node");
    assert_eq!(loop_node.op.args[5], "carry0_lt");
    expect_const_i64_value(&artifacts, &loop_node.op.args[6], "6");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn compiles_carried_breaking_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/carried_breaking_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("carried breaking while state project should compile");
}

#[test]
fn lowers_carried_breaking_while_state_project_with_carried_break_flow_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/carried_breaking_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_flow_chain"
        })
        .expect("expected loop_while_scalar_flow_chain node");
    assert_eq!(loop_node.op.args[5], "carry0_gt");
    expect_const_i64_value(&artifacts, &loop_node.op.args[6], "6");
    assert_eq!(loop_node.op.args[7], "break");
    assert_eq!(loop_node.op.args[9], "add_current");
}

#[test]
fn compiles_double_branching_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/double_branching_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("double branching while state project should compile");
}

#[test]
fn lowers_double_branching_while_state_project_with_double_carry_cond_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/double_branching_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_cond_chain"
        })
        .expect("expected loop_while_scalar_cond_chain node");
    assert_eq!(loop_node.op.args[6], "current_gt");
    expect_const_i64_value(&artifacts, &loop_node.op.args[7], "1");
    assert_eq!(loop_node.op.args[8], "add_current");
    assert_eq!(loop_node.op.args[9], "keep");
    assert_eq!(loop_node.op.args[11], "carry0_eq");
    expect_const_i64_value(&artifacts, &loop_node.op.args[12], "5");
    assert_eq!(loop_node.op.args[13], "add_carry0");
    assert_eq!(loop_node.op.args[14], "keep");
}

#[test]
fn compiles_post_flow_branching_continuing_while_state_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_branching_continuing_while_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("post-flow branching continuing while state project should compile");
}

#[test]
fn lowers_post_flow_branching_continuing_while_state_project_with_post_flow_continue_cond_loop_shape(
) {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_branching_continuing_while_demo",
    );

    let loop_node = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "loop_while_scalar_post_flow_cond_chain"
        })
        .expect("expected loop_while_scalar_post_flow_cond_chain node");
    assert_eq!(loop_node.op.args[5], "carry0_lt");
    assert_eq!(loop_node.op.args[7], "continue");
    assert_eq!(loop_node.op.args[9], "current_gt");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[12], "keep");
}
