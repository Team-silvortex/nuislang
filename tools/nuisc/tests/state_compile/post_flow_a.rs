use super::*;

#[test]
fn compiles_flow_continuing_while_state_project() {
    let project = Path::new("../../examples/projects/state/flow_continuing_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("flow continuing while state project should compile");
}

#[test]
fn lowers_flow_continuing_while_state_project_with_continue_flow_cond_loop_shape() {
    let artifacts = compiled_project("../../examples/projects/state/flow_continuing_while_demo");

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
    let project = Path::new("../../examples/projects/state/post_flow_branching_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("post-flow branching while state project should compile");
}

#[test]
fn compiles_tail_recursive_post_flow_branching_state_project() {
    let project =
        Path::new("../../examples/projects/state/tail_recursive_post_flow_branching_demo");
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive post-flow branching state project should compile");
}

#[test]
fn compiles_tail_recursive_post_flow_dynamic_prev_carry_state_project() {
    let project =
        Path::new("../../examples/projects/state/tail_recursive_post_flow_dynamic_prev_carry_demo");
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive post-flow dynamic prev-carry state project should compile");
}

#[test]
fn lowers_tail_recursive_post_flow_dynamic_prev_carry_state_project_with_recursive_post_flow_cond_loop_shape(
) {
    let artifacts = compiled_project(
        "../../examples/projects/state/tail_recursive_post_flow_dynamic_prev_carry_demo",
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
    let artifacts =
        compiled_project("../../examples/projects/state/tail_recursive_post_flow_branching_demo");

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
    let artifacts =
        compiled_project("../../examples/projects/state/post_flow_branching_while_demo");

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
    let project = Path::new("../../examples/projects/state/post_flow_breaking_while_demo");
    nuisc::pipeline::compile_project(project)
        .expect("post-flow breaking while state project should compile");
}

#[test]
fn lowers_post_flow_breaking_while_state_project_with_post_flow_break_loop_shape() {
    let artifacts = compiled_project("../../examples/projects/state/post_flow_breaking_while_demo");

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
    let project = Path::new("../../examples/projects/state/bounded_while_demo");
    nuisc::pipeline::compile_project(project).expect("bounded while state project should compile");
}

#[test]
fn lowers_bounded_while_state_project_with_bounded_post_flow_shape() {
    let artifacts = compiled_project("../../examples/projects/state/bounded_while_demo");

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
