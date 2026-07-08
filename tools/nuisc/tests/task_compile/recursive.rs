use super::*;

#[test]
fn compiles_task_recursive_async_project() {
    let project = Path::new("../../examples/projects/task/task_recursive_async_demo");
    nuisc::pipeline::compile_project(project).expect("task recursive async project should compile");
}

#[test]
fn compiles_task_recursive_async_keep_prev_carry_project() {
    let project =
        Path::new("../../examples/projects/task/task_recursive_async_keep_prev_carry_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task recursive async keep-prev-carry project should compile");
}

#[test]
fn compiles_task_recursive_async_shared_suffix_project() {
    let project = Path::new("../../examples/projects/task/task_recursive_async_shared_suffix_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task recursive async shared-suffix project should compile");
}

#[test]
fn compiles_task_mutual_recursive_async_project() {
    let project = Path::new("../../examples/projects/task/task_mutual_recursive_async_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task mutual recursive async project should compile");
}

#[test]
fn compiles_task_generic_recursive_async_project() {
    let project = Path::new("../../examples/projects/task/task_generic_recursive_async_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task generic recursive async project should compile");
}

#[test]
fn lowers_task_recursive_async_keep_prev_carry_project_with_cond_chain_shape() {
    let artifacts =
        compiled_project("../../examples/projects/task/task_recursive_async_keep_prev_carry_demo");

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
fn lowers_task_recursive_async_shared_suffix_project_with_select_then_suffix_then_recursive_call_shape(
) {
    let artifacts =
        compiled_project("../../examples/projects/task/task_recursive_async_shared_suffix_demo");

    let accumulate = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "accumulate")
        .expect("expected accumulate function");
    assert!(accumulate.is_async);
    assert!(accumulate.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If {
                condition: NirExpr::Binary { .. },
                then_body,
                else_body,
            } if matches!(
                then_body.as_slice(),
                [
                    NirStmt::Let {
                        name,
                        ty: Some(ty),
                        value: NirExpr::Var(current_name),
                    },
                    NirStmt::Let {
                        name: branch_name,
                        ty: Some(branch_ty),
                        value: NirExpr::Var(base_name),
                    }
                ] if name == "base"
                    && ty.render() == "i64"
                    && current_name == "current"
                    && branch_name == "branch_value"
                    && branch_ty.render() == "i64"
                    && base_name == "base"
            ) && matches!(
                else_body.as_slice(),
                [
                    NirStmt::Let {
                        name,
                        ty: Some(ty),
                        value: NirExpr::Int(0),
                    }
                ] if name == "branch_value" && ty.render() == "i64"
            )
        )
    }));
    assert!(accumulate.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::Binary { .. },
            } if name == "widened" && ty.render() == "i64"
        )
    }));
    assert!(matches!(
        accumulate.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Await(inner))))
            if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "accumulate")
    ));

    let lowered_ops = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction != "text")
        .map(|node| node.op.instruction.as_str())
        .collect::<Vec<_>>();
    let select_index = lowered_ops
        .iter()
        .position(|op| *op == "select")
        .expect("expected select op for recursive shared-suffix branch");
    let first_add_after_select = lowered_ops
        .iter()
        .enumerate()
        .skip(select_index + 1)
        .find_map(|(index, op)| (*op == "add").then_some(index))
        .expect("expected shared suffix add after select");
    let async_call_index = lowered_ops
        .iter()
        .enumerate()
        .skip(first_add_after_select + 1)
        .find_map(|(index, op)| (*op == "async_call").then_some(index))
        .expect("expected recursive async_call after shared suffix");
    let await_index = lowered_ops
        .iter()
        .enumerate()
        .skip(async_call_index + 1)
        .find_map(|(index, op)| (*op == "await").then_some(index))
        .expect("expected await after recursive async_call");
    assert!(
        select_index < first_add_after_select,
        "expected shared suffix add after select, got {lowered_ops:?}"
    );
    assert!(
        first_add_after_select < async_call_index,
        "expected recursive async_call after shared suffix add, got {lowered_ops:?}"
    );
    assert!(
        async_call_index < await_index,
        "expected await after recursive async_call, got {lowered_ops:?}"
    );
}

#[test]
fn lowers_task_generic_recursive_async_project_with_specialized_async_loop_shape() {
    let artifacts =
        compiled_project("../../examples/projects/task/task_generic_recursive_async_demo");

    let specialized = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "bounce__i64")
        .expect("expected specialized async generic recursive helper");
    assert!(specialized.is_async);
    assert!(specialized.generic_params.is_empty());

    assert!(
        artifacts
            .yir
            .nodes
            .iter()
            .any(|node| node.op.module == "cpu" && node.op.instruction == "async_call"),
        "expected async_call node in compiled task generic recursive project"
    );
    assert!(
        artifacts
            .yir
            .nodes
            .iter()
            .any(|node| node.op.module == "cpu" && node.op.instruction == "await"),
        "expected await node in compiled task generic recursive project"
    );
}

#[test]
fn compiles_task_generic_mutual_recursive_async_project() {
    let project =
        Path::new("../../examples/projects/task/task_generic_mutual_recursive_async_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task generic mutual recursive async project should compile");
}

#[test]
fn compiles_task_recursive_async_result_family_project() {
    let project = Path::new("../../examples/projects/task/task_recursive_async_result_family_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task recursive async result family project should compile");
}

#[test]
fn compiles_task_recursive_async_payload_alias_hof_project() {
    let project =
        Path::new("../../examples/projects/task/task_recursive_async_payload_alias_hof_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task recursive async payload alias hof project should compile");
}

#[test]
fn compiles_task_async_observer_bridge_project() {
    let project = Path::new("../../examples/projects/task/task_async_observer_bridge_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task async observer bridge project should compile");
}

#[test]
fn lowers_task_async_observer_bridge_project_with_await_and_task_observer_shape() {
    let artifacts =
        compiled_project("../../examples/projects/task/task_async_observer_bridge_demo");

    let orchestrate = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "orchestrate")
        .expect("expected orchestrate function");
    assert!(orchestrate.is_async);
    assert!(orchestrate.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::Await(inner),
            } if name == "base"
                && ty.render() == "i64"
                && matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "sum_down")
        )
    }));
    assert!(orchestrate.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(_),
            } if name == "completed_result" && ty.render() == "TaskResult<i64>"
        )
    }));
    assert!(orchestrate.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(_),
            } if name == "timed_result" && ty.render() == "TaskResult<i64>"
        )
    }));
    assert!(orchestrate.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If {
                condition: NirExpr::Binary { .. },
                then_body,
                ..
            } if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Binary { .. }))]
            )
        )
    }));
    assert!(orchestrate.body.iter().any(|stmt| {
        matches!(stmt, NirStmt::Return(Some(NirExpr::Var(name))) if name == "base")
    }));

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
            if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "orchestrate")
    ));
}
