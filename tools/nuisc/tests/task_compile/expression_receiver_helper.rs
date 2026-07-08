use super::*;

#[test]
fn compiles_task_async_method_receiver_match_project() {
    let project = Path::new("../../examples/projects/task/task_async_method_receiver_match_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task async method-receiver match project should compile");
}

#[test]
fn lowers_task_async_method_receiver_match_project_with_async_method_receiver_control_flow() {
    let artifacts =
        compiled_project("../../examples/projects/task/task_async_method_receiver_match_demo");

    let apply = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "apply")
        .expect("expected apply function");
    assert!(apply.is_async);
    assert!(apply.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If {
                then_body,
                else_body,
                ..
            } if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "impl.Addable.for.i64.add"
                    && args.len() == 2
                    && matches!(
                        &args[0],
                        NirExpr::Await(inner)
                            if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "one")
                    )
                    && matches!(&args[1], NirExpr::Int(3))
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "impl.Addable.for.i64.add"
                    && args.len() == 2
                    && matches!(
                        &args[0],
                        NirExpr::Await(inner)
                            if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "two")
                    )
                    && matches!(&args[1], NirExpr::Int(3))
            )
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
    assert_eq!(async_call_count, 3);
    assert_eq!(await_count, 3);

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
            if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "apply")
    ));
}

#[test]
fn compiles_task_async_helper_expanded_match_project() {
    let project = Path::new("../../examples/projects/task/task_async_helper_expanded_match_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task async helper-expanded match project should compile");
}

#[test]
fn lowers_task_async_helper_expanded_match_project_with_nested_helper_expanded_control_flow() {
    let artifacts =
        compiled_project("../../examples/projects/task/task_async_helper_expanded_match_demo");

    let expand = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "expand")
        .expect("expected expand function");
    assert!(expand.is_async);
    assert!(expand.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If {
                then_body,
                else_body,
                ..
            } if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))] if callee == "relay"
                    && args.len() == 1
                    && matches!(
                        &args[0],
                        NirExpr::Await(inner)
                            if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "one")
                    )
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))] if callee == "relay"
                    && args.len() == 1
                    && matches!(
                        &args[0],
                        NirExpr::Await(inner)
                            if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "two")
                    )
            )
        )
    }));

    let relay = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "relay")
        .expect("expected relay function");
    assert!(matches!(
        relay.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == "wrap"
                && args.len() == 1
                && matches!(&args[0], NirExpr::Call { callee, .. } if callee == "project")
    ));

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
    assert_eq!(async_call_count, 3);
    assert_eq!(await_count, 3);

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
            if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "expand")
    ));
}
