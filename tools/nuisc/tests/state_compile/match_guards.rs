use super::*;

#[test]
fn compiles_lambda_match_or_flow_continuing_while_state_project() {
    let project =
        Path::new("../../examples/projects/state/lambda_match_or_flow_continuing_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("lambda match or flow continuing while state project should compile");
}

#[test]
fn lowers_lambda_match_or_flow_continuing_while_state_project_with_or_flow_cond_loop_shape() {
    let artifacts = compiled_project(
        "../../examples/projects/state/lambda_match_or_flow_continuing_while_demo",
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
    let project = Path::new("../../examples/projects/state/match_guarded_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("match guarded while state project should compile");
}

#[test]
fn lowers_match_guarded_while_state_project_with_guarded_return_shape() {
    let artifacts = compiled_project("../../examples/projects/state/match_guarded_while_demo");

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
    let project = Path::new("../../examples/projects/state/match_guard_or_state_demo");
    nuisc::pipeline::compile_project(project).expect("match guard-or state project should compile");
}

#[test]
fn lowers_match_guard_or_state_project_with_guarded_return_shape() {
    let artifacts = compiled_project("../../examples/projects/state/match_guard_or_state_demo");

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "guard_return"));
}

#[test]
fn compiles_match_multi_guard_state_project() {
    let project = Path::new("../../examples/projects/state/match_multi_guard_state_demo");
    nuisc::pipeline::compile_project(project)
        .expect("match multi-guard state project should compile");
}

#[test]
fn lowers_match_multi_guard_state_project_with_guarded_return_shape() {
    let artifacts = compiled_project("../../examples/projects/state/match_multi_guard_state_demo");

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
    let project = Path::new("../../examples/projects/state/match_guard_range_state_demo");
    nuisc::pipeline::compile_project(project)
        .expect("match guard-range state project should compile");
}

#[test]
fn lowers_match_guard_range_state_project_with_guarded_return_shape() {
    let artifacts = compiled_project("../../examples/projects/state/match_guard_range_state_demo");

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "guard_return"));
}
