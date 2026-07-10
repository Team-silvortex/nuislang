use super::*;

#[test]
fn compiles_ordinary_recursive_generic_fn3_higher_order_state_project() {
    let project = Path::new(
        "../../examples/projects/state/ordinary_recursive_generic_fn3_higher_order_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive generic fn3 higher-order state project should compile");
}

#[test]
fn lowers_ordinary_recursive_generic_fn3_higher_order_state_project_with_specialized_hof_shape() {
    let artifacts = compiled_project(
        "../../examples/projects/state/ordinary_recursive_generic_fn3_higher_order_call_graph_demo",
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
        "../../examples/projects/state/ordinary_recursive_generic_alias_higher_order_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive generic alias higher-order state project should compile");
}

#[test]
fn lowers_ordinary_recursive_generic_alias_higher_order_state_project_with_specialized_hof_shape() {
    let artifacts = compiled_project(
        "../../examples/projects/state/ordinary_recursive_generic_alias_higher_order_call_graph_demo",
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
        "../../examples/projects/state/ordinary_recursive_generic_alias_fn3_higher_order_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive generic alias fn3 higher-order state project should compile");
}

#[test]
fn lowers_ordinary_recursive_generic_alias_fn3_higher_order_state_project_with_recursive_hof_shape()
{
    let artifacts = compiled_project(
        "../../examples/projects/state/ordinary_recursive_generic_alias_fn3_higher_order_call_graph_demo",
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
    let project =
        Path::new("../../examples/projects/state/ordinary_recursive_composed_call_graph_demo");
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive composed state project should compile");
}

#[test]
fn lowers_ordinary_recursive_composed_state_project_with_composed_helper_lane_shape() {
    let artifacts = compiled_project(
        "../../examples/projects/state/ordinary_recursive_composed_call_graph_demo",
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
    let project =
        Path::new("../../examples/projects/state/ordinary_recursive_lambda_call_graph_demo");
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive lambda state project should compile");
}

#[test]
fn lowers_ordinary_recursive_lambda_state_project_with_lambda_helper_lane_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/ordinary_recursive_lambda_call_graph_demo");

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
    let project =
        Path::new("../../examples/projects/state/ordinary_recursive_mixed_call_graph_demo");
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive mixed state project should compile");
}

#[test]
fn lowers_ordinary_recursive_mixed_state_project_with_bool_recursive_helper_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/ordinary_recursive_mixed_call_graph_demo");

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
    let bool_returns = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "return_bool")
        .count();
    let scalar_params = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "param_i32")
        .count();
    assert!(
        bool_returns >= 3,
        "expected mixed recursive project to emit bool helper returns"
    );
    assert!(
        scalar_params >= 3,
        "expected mixed recursive project to emit scalar helper parameters"
    );
}

#[test]
fn compiles_ordinary_recursive_generic_composed_state_project() {
    let project = Path::new(
        "../../examples/projects/state/ordinary_recursive_generic_composed_call_graph_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("ordinary recursive generic composed state project should compile");
}

#[test]
fn lowers_ordinary_recursive_generic_composed_state_project_with_specialized_hof_recursive_shape() {
    let artifacts = compiled_project(
        "../../examples/projects/state/ordinary_recursive_generic_composed_call_graph_demo",
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
