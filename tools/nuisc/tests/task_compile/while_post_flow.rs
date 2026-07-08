use super::*;

#[test]
fn compiles_task_async_while_flow_cond_project() {
    let project = Path::new("../../examples/projects/task/task_async_while_flow_cond_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task async while flow-cond project should compile");
}

#[test]
fn lowers_task_async_while_flow_cond_project_with_async_flow_cond_loop_shape() {
    let artifacts =
        compiled_project("../../examples/projects/task/task_async_while_flow_cond_demo");

    let accumulate = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "accumulate")
        .expect("expected accumulate function");
    assert!(accumulate.is_async);
    assert!(accumulate
        .body
        .iter()
        .any(|stmt| { matches!(stmt, NirStmt::While { .. }) }));

    let loop_node = artifacts
        .yir
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

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("expected main function");
    assert!(main.is_async);
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Await(inner))))
            if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "accumulate")
    ));
}

#[test]
fn compiles_task_async_while_post_flow_project() {
    let project = Path::new("../../examples/projects/task/task_async_while_post_flow_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task async while post-flow project should compile");
}

#[test]
fn lowers_task_async_while_post_flow_project_with_async_post_flow_loop_shape() {
    let artifacts =
        compiled_project("../../examples/projects/task/task_async_while_post_flow_demo");

    let accumulate = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "accumulate")
        .expect("expected accumulate function");
    assert!(accumulate.is_async);
    assert!(accumulate
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::While { .. })));

    let loop_node = artifacts
        .yir
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

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("expected main function");
    assert!(main.is_async);
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Await(inner))))
            if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "accumulate")
    ));
}

#[test]
fn compiles_task_async_while_post_flow_cond_project() {
    let project = Path::new("../../examples/projects/task/task_async_while_post_flow_cond_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task async while post-flow cond project should compile");
}

#[test]
fn lowers_task_async_while_post_flow_cond_project_with_async_post_flow_cond_loop_shape() {
    let artifacts =
        compiled_project("../../examples/projects/task/task_async_while_post_flow_cond_demo");

    let accumulate = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "accumulate")
        .expect("expected accumulate function");
    assert!(accumulate.is_async);
    assert!(accumulate
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::While { .. })));

    let loop_node = artifacts
        .yir
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

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("expected main function");
    assert!(main.is_async);
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Await(inner))))
            if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "accumulate")
    ));
}

#[test]
fn compiles_task_async_while_post_flow_compound_project() {
    let project =
        Path::new("../../examples/projects/task/task_async_while_post_flow_compound_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task async while post-flow compound project should compile");
}

#[test]
fn compiles_task_async_post_flow_recursive_branching_project() {
    let project =
        Path::new("../../examples/projects/task/task_async_post_flow_recursive_branching_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task async post-flow recursive branching project should compile");
}

#[test]
fn compiles_task_async_post_flow_keep_prev_carry_project() {
    let project =
        Path::new("../../examples/projects/task/task_async_post_flow_keep_prev_carry_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task async post-flow keep-prev-carry project should compile");
}

#[test]
fn compiles_task_async_post_flow_shared_suffix_loop_control_project() {
    let project = Path::new(
        "../../examples/projects/task/task_async_post_flow_shared_suffix_loop_control_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task async post-flow shared-suffix loop-control project should compile");
}

#[test]
fn rejects_task_async_memory_project_with_precise_sibling_carry_diagnostic() {
    let project =
        Path::new("../../examples/projects/task/task_async_post_flow_memory_unsupported_demo");
    let error = nuisc::pipeline::compile_project(project)
        .err()
        .expect("task async memory project should fail until lowering exists");
    assert!(error
        .contains("references sibling carry `slot` before that carry is updated in the loop body"));
}

#[test]
fn rejects_task_async_post_flow_shared_suffix_loop_control_project_with_precise_shape_diagnostic() {
    let project = Path::new(
        "../../examples/invalid/projects/bad_task_async_post_flow_shared_suffix_loop_control",
    );
    let error = nuisc::pipeline::compile_project(project).err().expect(
        "task async post-flow shared-suffix loop-control project should fail until lowering exists",
    );
    assert!(error.contains(
        "structured `while` lowering recognized loop state `value` and a loop-control `if`"
    ));
    assert!(error.contains(
        "control condition is not reducible to supported loop-state/carry boolean tests"
    ));
}

#[test]
fn lowers_task_async_post_flow_shared_suffix_loop_control_project_with_cond_chain_shape() {
    let artifacts = compiled_project(
        "../../examples/projects/task/task_async_post_flow_shared_suffix_loop_control_demo",
    );

    let accumulate = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "accumulate")
        .expect("expected accumulate function");
    assert!(accumulate.is_async);
    assert!(accumulate
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::While { .. })));

    let loop_node = artifacts
        .yir
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
    assert_eq!(
        loop_node.op.args[10],
        "add_scaled_by_current_plus_current_times_factor_group_current_plus_factor_invariant_times_factor_invariant_times_terms_current_plus_current_plus_current_plus_invariant"
    );
    assert!(loop_node.op.args[11].starts_with("int_"));
    assert!(loop_node.op.args[12].starts_with("int_"));
    assert!(loop_node.op.args[13].starts_with("int_"));
    assert_eq!(
        loop_node.op.args[14],
        "add_scaled_by_current_plus_current_times_factor_group_current_plus_factor_invariant_times_factor_invariant_times_terms_current_plus_current_plus_invariant"
    );
    assert!(loop_node.op.args[15].starts_with("int_"));
    assert!(loop_node.op.args[16].starts_with("int_"));
    assert!(loop_node.op.args[17].starts_with("add_"));
}

#[test]
fn lowers_task_async_post_flow_recursive_branching_project_with_post_flow_recursive_shape() {
    let artifacts = compiled_project(
        "../../examples/projects/task/task_async_post_flow_recursive_branching_demo",
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
    assert!(loop_node.op.args.iter().any(|arg| arg == "prev_current_gt"));
    assert!(loop_node
        .op
        .args
        .iter()
        .any(|arg| arg == "add_prev_current"));
    assert!(loop_node.op.args.iter().any(|arg| arg == "keep"));
}

#[test]
fn lowers_task_async_post_flow_keep_prev_carry_project_with_post_flow_recursive_shape() {
    let artifacts =
        compiled_project("../../examples/projects/task/task_async_post_flow_keep_prev_carry_demo");

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
    assert!(loop_node.op.args.iter().any(|arg| arg == "prev_current_gt"));
    assert!(loop_node
        .op
        .args
        .iter()
        .any(|arg| arg == "add_prev_current"));
    assert!(loop_node.op.args.iter().any(|arg| arg == "keep_prev_carry"));
}

#[test]
fn lowers_task_async_while_post_flow_compound_project_with_async_post_flow_compound_shape() {
    let artifacts =
        compiled_project("../../examples/projects/task/task_async_while_post_flow_compound_demo");

    let accumulate = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "accumulate")
        .expect("expected accumulate function");
    assert!(accumulate.is_async);
    assert!(accumulate
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::While { .. })));

    let loop_node = artifacts
        .yir
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

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("expected main function");
    assert!(main.is_async);
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Await(inner))))
            if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "accumulate")
    ));
}
