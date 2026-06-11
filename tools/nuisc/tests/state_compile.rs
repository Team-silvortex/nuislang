use std::path::Path;

use nuis_semantics::model::{NirExpr, NirStmt};

fn compiled_project(path: &str) -> nuisc::pipeline::PipelineArtifacts {
    nuisc::pipeline::compile_project(Path::new(path))
        .unwrap_or_else(|error| panic!("project `{path}` should compile: {error}"))
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
        .find(|node| node.op.module == "cpu" && node.op.instruction == "loop_while_i64_cond_chain")
        .expect("expected loop_while_i64_cond_chain node");
    assert_eq!(loop_node.op.args[3], "gt");
    assert_eq!(loop_node.op.args[4], "sub");
    assert_eq!(loop_node.op.args[6], "prev_current_gt");
    assert_eq!(loop_node.op.args[8], "add_prev_carry1");
    assert_eq!(loop_node.op.args[9], "keep");
    assert_eq!(loop_node.op.args[11], "prev_current_gt");
    assert_eq!(loop_node.op.args[13], "add_prev_current");
    assert_eq!(loop_node.op.args[14], "add_prev_carry0");
}
