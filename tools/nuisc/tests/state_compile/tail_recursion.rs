use super::*;

#[test]
fn compiles_tail_recursive_sum_state_project() {
    let project = Path::new("../../examples/projects/state/tail_recursive_sum_demo");
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive sum state project should compile");
}

#[test]
fn lowers_tail_recursive_sum_state_project_with_chain_loop_shape() {
    let artifacts = compiled_project("../../examples/projects/state/tail_recursive_sum_demo");

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
    let project = Path::new("../../examples/projects/state/tail_recursive_factorial_demo");
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive factorial state project should compile");
}

#[test]
fn lowers_tail_recursive_factorial_state_project_with_multiplicative_chain_shape() {
    let artifacts = compiled_project("../../examples/projects/state/tail_recursive_factorial_demo");

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
    let project =
        Path::new("../../examples/projects/state/tail_recursive_factorial_affine_mul_demo");
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive factorial affine mul state project should compile");
}

#[test]
fn lowers_tail_recursive_factorial_affine_mul_state_project_with_affine_multiplicative_chain_shape()
{
    let artifacts =
        compiled_project("../../examples/projects/state/tail_recursive_factorial_affine_mul_demo");

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
    let project =
        Path::new("../../examples/projects/state/tail_recursive_factorial_scaled_mul_demo");
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive factorial scaled mul state project should compile");
}

#[test]
fn lowers_tail_recursive_factorial_scaled_mul_state_project_with_scaled_multiplicative_chain_shape()
{
    let artifacts =
        compiled_project("../../examples/projects/state/tail_recursive_factorial_scaled_mul_demo");

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
    let project =
        Path::new("../../examples/projects/state/counted_while_multi_carry_scaled_mul_demo");
    nuisc::pipeline::compile_project(project)
        .expect("counted while multi-carry scaled mul state project should compile");
}

#[test]
fn lowers_counted_while_multi_carry_scaled_mul_state_project_with_multi_carry_scaled_mul_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/counted_while_multi_carry_scaled_mul_demo");

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
    let project =
        Path::new("../../examples/projects/state/counted_while_multi_carry_state_factor_mul_demo");
    nuisc::pipeline::compile_project(project)
        .expect("counted while multi-carry state-factor mul state project should compile");
}

#[test]
fn lowers_counted_while_multi_carry_state_factor_mul_state_project_with_state_factor_mul_shape() {
    let artifacts = compiled_project(
        "../../examples/projects/state/counted_while_multi_carry_state_factor_mul_demo",
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
        "../../examples/projects/state/counted_while_multi_carry_state_plus_invariant_factor_mul_demo",
    );
    nuisc::pipeline::compile_project(project).expect(
        "counted while multi-carry state-plus-invariant-factor mul state project should compile",
    );
}

#[test]
fn lowers_counted_while_multi_carry_state_plus_invariant_factor_mul_state_project_with_state_plus_invariant_factor_mul_shape(
) {
    let artifacts = compiled_project(
        "../../examples/projects/state/counted_while_multi_carry_state_plus_invariant_factor_mul_demo",
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
    let project = Path::new("../../examples/projects/state/tail_recursive_cross_carry_demo");
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive cross-carry state project should compile");
}

#[test]
fn lowers_tail_recursive_cross_carry_state_project_with_cross_carry_chain_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/tail_recursive_cross_carry_demo");

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
    let project = Path::new("../../examples/projects/state/tail_recursive_branching_demo");
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive branching state project should compile");
}

#[test]
fn lowers_tail_recursive_branching_state_project_with_branching_cond_loop_shape() {
    let artifacts = compiled_project("../../examples/projects/state/tail_recursive_branching_demo");

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
    let project = Path::new("../../examples/projects/state/tail_recursive_keep_prev_carry_demo");
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive keep-prev-carry state project should compile");
}

#[test]
fn lowers_tail_recursive_keep_prev_carry_state_project_with_branching_cond_loop_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/tail_recursive_keep_prev_carry_demo");

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
    let project = Path::new("../../examples/projects/state/tail_recursive_multi_carry_demo");
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive multi-carry state project should compile");
}

#[test]
fn lowers_tail_recursive_multi_carry_state_project_with_multi_carry_chain_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/tail_recursive_multi_carry_demo");

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
    let project =
        Path::new("../../examples/projects/state/tail_recursive_carry_condition_multi_carry_demo");
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive carry-condition multi-carry state project should compile");
}

#[test]
fn lowers_tail_recursive_carry_condition_multi_carry_state_project_with_carry_condition_cond_loop_shape(
) {
    let artifacts = compiled_project(
        "../../examples/projects/state/tail_recursive_carry_condition_multi_carry_demo",
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
    let project =
        Path::new("../../examples/projects/state/tail_recursive_branching_cross_carry_demo");
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive branching cross-carry state project should compile");
}

#[test]
fn lowers_tail_recursive_branching_cross_carry_state_project_with_cond_loop_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/tail_recursive_branching_cross_carry_demo");

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
    let project =
        Path::new("../../examples/projects/state/tail_recursive_branching_multi_carry_demo");
    nuisc::pipeline::compile_project(project)
        .expect("tail recursive branching multi-carry state project should compile");
}

#[test]
fn lowers_tail_recursive_branching_multi_carry_state_project_with_cond_loop_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/tail_recursive_branching_multi_carry_demo");

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
