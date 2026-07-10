use super::*;

#[test]
fn compiles_task_async_if_expression_positions_project() {
    let project = Path::new("../../examples/projects/task/task_async_if_expression_positions_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task async if-expression positions project should compile");
}

#[test]
fn lowers_task_async_if_expression_positions_project_with_async_if_expression_family() {
    let artifacts =
        compiled_project("../../examples/projects/task/task_async_if_expression_positions_demo");

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
            NirStmt::If { then_body, else_body, .. }
                if matches!(
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

    let call_pick = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "call_pick")
        .expect("expected call_pick function");
    assert!(call_pick.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If { then_body, else_body, .. }
                if matches!(
                    then_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                        if callee == "add_pair"
                            && args.len() == 2
                            && matches!(
                                &args[0],
                                NirExpr::Await(inner)
                                    if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "one")
                            )
                            && matches!(&args[1], NirExpr::Int(5))
                ) && matches!(
                    else_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                        if callee == "add_pair"
                            && args.len() == 2
                            && matches!(
                                &args[0],
                                NirExpr::Await(inner)
                                    if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "two")
                            )
                            && matches!(&args[1], NirExpr::Int(5))
                )
        )
    }));

    let packetize = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "packetize")
        .expect("expected packetize function");
    assert!(packetize.is_async);
    assert!(packetize.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If { then_body, else_body, .. }
                if matches!(
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
    assert!(packetize.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::StructLiteral { fields, .. },
            } if name == "packet"
                && ty.render() == "Packet"
                && fields.iter().any(|(field, value)| {
                    field == "value" && matches!(value, NirExpr::Var(name) if name == "value")
                })
                && fields.iter().any(|(field, value)| {
                    field == "tag" && matches!(value, NirExpr::Var(name) if name == "tag")
                })
        )
    }));
    assert!(matches!(
        packetize.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Var(name)))) if name == "packet"
    ));

    let apply = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "apply")
        .expect("expected apply function");
    assert!(apply.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If { then_body, else_body, .. }
                if matches!(
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

    let expand = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "expand")
        .expect("expected expand function");
    assert!(expand.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If { then_body, else_body, .. }
                if matches!(
                    then_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                        if callee == "relay"
                            && args.len() == 1
                            && matches!(
                                &args[0],
                                NirExpr::Await(inner)
                                    if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "one")
                            )
                ) && matches!(
                    else_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                        if callee == "relay"
                            && args.len() == 1
                            && matches!(
                                &args[0],
                                NirExpr::Await(inner)
                                    if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "two")
                            )
                )
        )
    }));

    let main = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("expected main function");
    assert!(main.is_async);
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::Await(inner),
            } if name == "packet"
                && ty.render() == "Packet"
                && matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "packetize")
        )
    }));
}
