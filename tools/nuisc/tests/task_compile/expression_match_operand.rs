use super::*;

#[test]
fn compiles_task_async_await_match_operand_project() {
    let project = Path::new("../../examples/projects/task/task_async_await_match_operand_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task async await-match operand project should compile");
}

#[test]
fn lowers_task_async_await_match_operand_project_with_expression_position_async_control_flow() {
    let artifacts =
        compiled_project("../../examples/projects/task/task_async_await_match_operand_demo");

    let branch_pick = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "branch_pick")
        .expect("expected branch_pick function");
    assert!(branch_pick.is_async);
    assert!(branch_pick.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If {
                then_body,
                else_body,
                ..
            } if matches!(
                then_body.as_slice(),
                [NirStmt::Let {
                    name,
                    value: NirExpr::Await(inner),
                    ..
                }] if name == "value"
                    && matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "one")
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Let {
                    name,
                    value: NirExpr::Await(inner),
                    ..
                }] if name == "value"
                    && matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "two")
            )
        )
    }));
    assert!(matches!(
        branch_pick.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Var(name)))) if name == "value"
    ));

    let classify = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "classify")
        .expect("expected classify function");
    assert!(classify.is_async);
    assert!(classify.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::Await(inner),
            } if name == "value"
                && ty.render() == "i64"
                && matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "branch_pick")
        )
    }));
    assert!(classify.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If { then_body, else_body, .. }
                if matches!(then_body.as_slice(), [NirStmt::Return(Some(NirExpr::Binary { .. }))])
                && matches!(else_body.as_slice(), [NirStmt::Return(Some(NirExpr::Binary { .. }))])
        )
    }));

    let async_call_count = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "async_call")
        .count();
    let await_count = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "await")
        .count();
    assert_eq!(async_call_count, 4);
    assert_eq!(await_count, 4);

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
            if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "classify")
    ));
}
