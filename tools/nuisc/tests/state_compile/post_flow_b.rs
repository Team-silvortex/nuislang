use super::*;

#[test]
fn compiles_equality_while_state_project() {
    let project = Path::new("../../examples/projects/state/equality_while_demo");
    nuisc::pipeline::compile_project(project).expect("equality while state project should compile");
}

#[test]
fn lowers_equality_while_state_project_with_equality_post_flow_shape() {
    let artifacts = compiled_project("../../examples/projects/state/equality_while_demo");

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
    let project = Path::new("../../examples/projects/state/inequality_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("inequality while state project should compile");
}

#[test]
fn lowers_inequality_while_state_project_with_inequality_loop_shape() {
    let artifacts = compiled_project("../../examples/projects/state/inequality_while_demo");

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
    let project = Path::new("../../examples/projects/state/post_flow_continuing_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("post-flow continuing while state project should compile");
}

#[test]
fn lowers_post_flow_continuing_while_state_project_with_post_flow_continue_loop_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/post_flow_continuing_while_demo");

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
    let project = Path::new("../../examples/projects/state/carried_breaking_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("carried breaking while state project should compile");
}

#[test]
fn lowers_carried_breaking_while_state_project_with_carried_break_flow_loop_shape() {
    let artifacts = compiled_project("../../examples/projects/state/carried_breaking_while_demo");

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
    let project = Path::new("../../examples/projects/state/double_branching_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("double branching while state project should compile");
}

#[test]
fn lowers_double_branching_while_state_project_with_double_carry_cond_loop_shape() {
    let artifacts = compiled_project("../../examples/projects/state/double_branching_while_demo");

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
    let project =
        Path::new("../../examples/projects/state/post_flow_branching_continuing_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("post-flow branching continuing while state project should compile");
}

#[test]
fn lowers_post_flow_branching_continuing_while_state_project_with_post_flow_continue_cond_loop_shape(
) {
    let artifacts =
        compiled_project("../../examples/projects/state/post_flow_branching_continuing_while_demo");

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
