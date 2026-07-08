use super::*;

#[test]
fn compiles_task_async_match_call_argument_project() {
    let project = Path::new("../../examples/projects/task/task_async_match_call_argument_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task async match call-argument project should compile");
}

#[test]
fn lowers_task_async_match_call_argument_project_with_async_call_argument_control_flow() {
    let artifacts =
        compiled_project("../../examples/projects/task/task_async_match_call_argument_demo");

    let call_pick = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "call_pick")
        .expect("expected call_pick function");
    assert!(call_pick.is_async);
    assert!(call_pick.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If {
                then_body,
                else_body,
                ..
            } if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "add"
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
                    if callee == "add"
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
            if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "call_pick")
    ));
}

#[test]
fn compiles_task_async_struct_field_match_project() {
    let project = Path::new("../../examples/projects/task/task_async_struct_field_match_demo");
    nuisc::pipeline::compile_project(project)
        .expect("task async struct-field match project should compile");
}

#[test]
fn lowers_task_async_struct_field_match_project_with_async_struct_field_control_flow() {
    let artifacts =
        compiled_project("../../examples/projects/task/task_async_struct_field_match_demo");

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
            NirStmt::If {
                then_body,
                else_body,
                ..
            } if matches!(
                then_body.as_slice(),
                [NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::StructLiteral { type_name, fields, .. },
                }] if name == "packet"
                    && ty.render() == "Packet"
                    && type_name == "Packet"
                    && fields.iter().any(|(field, value)| {
                        field == "value"
                            && matches!(
                                value,
                                NirExpr::Await(inner)
                                    if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "one")
                            )
                    })
                    && fields.iter().any(|(field, value)| field == "tag" && matches!(value, NirExpr::Var(name) if name == "seed"))
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::StructLiteral { type_name, fields, .. },
                }] if name == "packet"
                    && ty.render() == "Packet"
                    && type_name == "Packet"
                    && fields.iter().any(|(field, value)| {
                        field == "value"
                            && matches!(
                                value,
                                NirExpr::Await(inner)
                                    if matches!(inner.as_ref(), NirExpr::Call { callee, .. } if callee == "two")
                            )
                    })
                    && fields.iter().any(|(field, value)| field == "tag" && matches!(value, NirExpr::Var(name) if name == "seed"))
            )
        )
    }));
    assert!(matches!(
        packetize.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Var(name)))) if name == "packet"
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
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::FieldAccess { field, .. }))) if field == "value"
    ));
}
