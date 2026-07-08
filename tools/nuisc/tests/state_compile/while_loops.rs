use super::*;

#[test]
fn compiles_flow_branching_while_state_project() {
    let project = Path::new("../../examples/projects/state/flow_branching_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("flow branching while state project should compile");
}

#[test]
fn compiles_counted_while_state_project() {
    let project = Path::new("../../examples/projects/state/counted_while_demo");
    nuisc::pipeline::compile_project(project).expect("counted while state project should compile");
}

#[test]
fn lowers_counted_while_state_project_with_basic_loop_shape() {
    let artifacts = compiled_project("../../examples/projects/state/counted_while_demo");

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
    let project = Path::new("../../examples/projects/state/accumulating_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("accumulating while state project should compile");
}

#[test]
fn lowers_accumulating_while_state_project_with_single_carry_chain_shape() {
    let artifacts = compiled_project("../../examples/projects/state/accumulating_while_demo");

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
    let project = Path::new("../../examples/projects/state/chained_while_demo");
    nuisc::pipeline::compile_project(project).expect("chained while state project should compile");
}

#[test]
fn lowers_chained_while_state_project_with_multi_carry_chain_shape() {
    let artifacts = compiled_project("../../examples/projects/state/chained_while_demo");

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
    let project = Path::new("../../examples/projects/state/match_branching_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("match branching while state project should compile");
}

#[test]
fn lowers_match_branching_while_state_project_with_cond_loop_shape() {
    let artifacts = compiled_project("../../examples/projects/state/match_branching_while_demo");

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
    let project = Path::new("../../examples/projects/state/branching_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("branching while state project should compile");
}

#[test]
fn lowers_branching_while_state_project_with_plain_cond_loop_shape() {
    let artifacts = compiled_project("../../examples/projects/state/branching_while_demo");

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
    let project = Path::new("../../examples/projects/state/bool_match_branching_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("bool match branching while state project should compile");
}

#[test]
fn lowers_bool_match_branching_while_state_project_with_cond_loop_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/bool_match_branching_while_demo");

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
    let project = Path::new("../../examples/projects/state/lambda_match_branching_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("lambda match branching while state project should compile");
}

#[test]
fn lowers_lambda_match_branching_while_state_project_with_lambda_cond_loop_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/lambda_match_branching_while_demo");

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
    let project = Path::new("../../examples/projects/state/match_expr_branching_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("match expression branching while state project should compile");
}

#[test]
fn lowers_match_expr_branching_while_state_project_with_nested_if_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/match_expr_branching_while_demo");

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
    let artifacts = compiled_project("../../examples/projects/state/flow_branching_while_demo");

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
    let project = Path::new("../../examples/projects/state/equality_branching_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("equality branching while state project should compile");
}

#[test]
fn lowers_equality_branching_while_state_project_with_equality_flow_cond_loop_shape() {
    let artifacts = compiled_project("../../examples/projects/state/equality_branching_while_demo");

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
    let project =
        Path::new("../../examples/projects/state/lambda_match_flow_continuing_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("lambda match flow continuing while state project should compile");
}

#[test]
fn lowers_lambda_match_flow_continuing_while_state_project_with_lambda_flow_cond_loop_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/lambda_match_flow_continuing_while_demo");

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
