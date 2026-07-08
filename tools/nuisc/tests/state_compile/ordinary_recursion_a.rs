use super::*;

#[test]
fn compiles_ordinary_mutual_recursive_state_project() {
    let project = Path::new("../../examples/projects/state/ordinary_mutual_recursive_demo");
    nuisc::pipeline::compile_project(project)
        .expect("ordinary mutual recursive state project should compile");
}

#[test]
fn compiles_ordinary_recursive_scalar_state_project() {
    let project = Path::new("../../examples/projects/state/ordinary_recursive_call_graph_demo");
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive scalar state project should compile");
}

#[test]
fn lowers_ordinary_recursive_scalar_state_project_with_scalar_helper_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/ordinary_recursive_call_graph_demo");

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
    let project = Path::new("../../examples/projects/state/ordinary_recursive_i32_call_graph_demo");
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive i32 state project should compile");
}

#[test]
fn lowers_ordinary_recursive_i32_state_project_with_i32_helper_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/ordinary_recursive_i32_call_graph_demo");

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
    let project =
        Path::new("../../examples/projects/state/ordinary_recursive_match_call_graph_demo");
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive match state project should compile");
}

#[test]
fn lowers_ordinary_recursive_match_state_project_into_recursive_helper_lanes() {
    let artifacts =
        compiled_project("../../examples/projects/state/ordinary_recursive_match_call_graph_demo");

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
    let project =
        Path::new("../../examples/projects/state/ordinary_recursive_bool_call_graph_demo");
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive bool state project should compile");
}

#[test]
fn lowers_ordinary_recursive_bool_state_project_with_bool_helper_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/ordinary_recursive_bool_call_graph_demo");

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
    let bool_returns = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "return_bool")
        .count();
    let selects = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "select")
        .count();
    assert!(
        bool_returns >= 3,
        "expected bool recursive project to emit bool helper returns"
    );
    assert!(
        selects >= 3,
        "expected bool recursive project to lower bool branches through select nodes"
    );
}

#[test]
fn compiles_ordinary_recursive_higher_order_state_project() {
    let project =
        Path::new("../../examples/projects/state/ordinary_recursive_higher_order_call_graph_demo");
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive higher-order state project should compile");
}

#[test]
fn lowers_ordinary_recursive_higher_order_state_project_with_named_helper_hof_shape() {
    let artifacts = compiled_project(
        "../../examples/projects/state/ordinary_recursive_higher_order_call_graph_demo",
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
        "../../examples/projects/state/ordinary_recursive_fn2_higher_order_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive fn2 higher-order state project should compile");
}

#[test]
fn lowers_ordinary_recursive_fn2_higher_order_state_project_with_recursive_hof_shape() {
    let artifacts = compiled_project(
        "../../examples/projects/state/ordinary_recursive_fn2_higher_order_call_graph_demo",
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
        "../../examples/projects/state/ordinary_recursive_generic_higher_order_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive generic higher-order state project should compile");
}

#[test]
fn lowers_ordinary_recursive_generic_higher_order_state_project_with_specialized_hof_shape() {
    let artifacts = compiled_project(
        "../../examples/projects/state/ordinary_recursive_generic_higher_order_call_graph_demo",
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
        "../../examples/projects/state/ordinary_recursive_generic_fn2_higher_order_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive generic fn2 higher-order state project should compile");
}

#[test]
fn lowers_ordinary_recursive_generic_fn2_higher_order_state_project_with_specialized_hof_shape() {
    let artifacts = compiled_project(
        "../../examples/projects/state/ordinary_recursive_generic_fn2_higher_order_call_graph_demo",
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
