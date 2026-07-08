use super::*;

#[test]
fn compiles_generic_payload_alias_higher_order_state_project() {
    let project =
        Path::new("../../examples/projects/state/generic_payload_alias_higher_order_demo");
    nuisc::pipeline::compile_project(project)
        .expect("generic payload alias higher-order state project should compile");
}

#[test]
fn compiles_generic_payload_alias_method_hof_state_project() {
    let project = Path::new("../../examples/projects/state/generic_payload_alias_method_hof_demo");
    nuisc::pipeline::compile_project(project)
        .expect("generic payload alias method higher-order state project should compile");
}

#[test]
fn compiles_generic_callable_forwarding_hof_state_project() {
    let project = Path::new("../../examples/projects/state/generic_callable_forwarding_hof_demo");
    nuisc::pipeline::compile_project(project)
        .expect("generic callable forwarding higher-order state project should compile");
}

#[test]
fn lowers_generic_callable_forwarding_hof_state_project_with_forwarded_fn2_and_fn3_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/generic_callable_forwarding_hof_demo");

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
    let project = Path::new("../../examples/projects/state/glm_borrow_end_state_demo");
    nuisc::pipeline::compile_project(project).expect("glm borrow_end state project should compile");
}

#[test]
fn lowers_glm_borrow_end_state_project_with_borrow_end_then_owner_write_shape() {
    let artifacts = compiled_project("../../examples/projects/state/glm_borrow_end_state_demo");

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
    let project = Path::new("../../examples/projects/state/if_borrow_end_state_demo");
    nuisc::pipeline::compile_project(project).expect("if borrow_end state project should compile");
}

#[test]
fn lowers_if_borrow_end_state_project_with_borrow_end_then_owner_write_shape() {
    let artifacts = compiled_project("../../examples/projects/state/if_borrow_end_state_demo");

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
    let project =
        Path::new("../../examples/projects/state/match_borrow_end_shared_suffix_state_demo");
    nuisc::pipeline::compile_project(project)
        .expect("match borrow_end shared suffix state project should compile");
}

#[test]
fn lowers_match_borrow_end_shared_suffix_state_project_with_shared_suffix_after_select_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/match_borrow_end_shared_suffix_state_demo");

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
    let project = Path::new("../../examples/projects/state/generic_shared_suffix_if_method_demo");
    nuisc::pipeline::compile_project(project)
        .expect("generic shared suffix if-method state project should compile");
}

#[test]
fn lowers_generic_shared_suffix_if_method_state_project_with_select_then_method_suffix_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/generic_shared_suffix_if_method_demo");

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
    let project = Path::new("../../examples/projects/state/task_result_shared_suffix_state_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task result shared suffix state project should compile");
}

#[test]
fn lowers_task_result_shared_suffix_state_project_with_select_then_suffix_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/task_result_shared_suffix_state_demo");

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
    let project = Path::new("../../examples/projects/state/buffer_shared_suffix_state_demo");
    nuisc::pipeline::compile_project(project)
        .expect("buffer shared suffix state project should compile");
}

#[test]
fn lowers_buffer_shared_suffix_state_project_with_select_then_store_shape() {
    let artifacts =
        compiled_project("../../examples/projects/state/buffer_shared_suffix_state_demo");

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
    let artifacts =
        compiled_project("../../examples/projects/state/generic_payload_alias_method_hof_demo");

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
        Some(NirStmt::If { then_body, .. })
            if matches!(
                then_body.first(),
                Some(NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::FieldAccess { field, .. }
                        | NirExpr::VariantFieldAccess { field, .. },
                }) if name == "payload" && ty.render() == "i64" && field == "value"
            )
    ));
    assert!(matches!(
        hof.body.first(),
        Some(NirStmt::If { then_body, .. })
            if then_body.iter().any(|stmt| {
                matches!(
                    stmt,
                    NirStmt::Let {
                        value: NirExpr::Call { callee, .. },
                        ..
                    } if callee.starts_with("__lambda_main_")
                )
            })
    ));
}
