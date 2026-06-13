use std::path::Path;

use nuis_semantics::model::{NirExpr, NirStmt};

fn compiled_project(path: &str) -> nuisc::pipeline::PipelineArtifacts {
    nuisc::pipeline::compile_project(Path::new(path))
        .unwrap_or_else(|error| panic!("project `{path}` should compile: {error}"))
}

#[test]
fn compiles_task_recursive_async_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_demo",
    );
    nuisc::pipeline::compile_project(project).expect("task recursive async project should compile");
}

#[test]
fn compiles_task_mutual_recursive_async_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_mutual_recursive_async_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task mutual recursive async project should compile");
}

#[test]
fn compiles_task_generic_recursive_async_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_generic_recursive_async_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task generic recursive async project should compile");
}

#[test]
fn lowers_task_generic_recursive_async_project_with_specialized_async_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_generic_recursive_async_demo",
    );

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
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_generic_mutual_recursive_async_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task generic mutual recursive async project should compile");
}

#[test]
fn compiles_task_recursive_async_result_family_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_result_family_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task recursive async result family project should compile");
}

#[test]
fn compiles_task_recursive_async_payload_alias_hof_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_payload_alias_hof_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task recursive async payload alias hof project should compile");
}

#[test]
fn compiles_task_async_observer_bridge_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_observer_bridge_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task async observer bridge project should compile");
}

#[test]
fn lowers_task_async_observer_bridge_project_with_await_and_task_observer_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_observer_bridge_demo",
    );

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

#[test]
fn compiles_task_async_if_expression_positions_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_if_expression_positions_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task async if-expression positions project should compile");
}

#[test]
fn lowers_task_async_if_expression_positions_project_with_async_if_expression_family() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_if_expression_positions_demo",
    );

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
                if matches!(then_body.as_slice(), [NirStmt::If { .. }])
                    && matches!(else_body.as_slice(), [NirStmt::If { .. }])
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

#[test]
fn compiles_task_async_await_match_operand_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_await_match_operand_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task async await-match operand project should compile");
}

#[test]
fn lowers_task_async_await_match_operand_project_with_expression_position_async_control_flow() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_await_match_operand_demo",
    );

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

#[test]
fn compiles_task_async_match_call_argument_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_match_call_argument_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task async match call-argument project should compile");
}

#[test]
fn lowers_task_async_match_call_argument_project_with_async_call_argument_control_flow() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_match_call_argument_demo",
    );

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
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_struct_field_match_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task async struct-field match project should compile");
}

#[test]
fn lowers_task_async_struct_field_match_project_with_async_struct_field_control_flow() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_struct_field_match_demo",
    );

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

#[test]
fn compiles_task_async_method_receiver_match_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_method_receiver_match_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task async method-receiver match project should compile");
}

#[test]
fn lowers_task_async_method_receiver_match_project_with_async_method_receiver_control_flow() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_method_receiver_match_demo",
    );

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
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_helper_expanded_match_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task async helper-expanded match project should compile");
}

#[test]
fn lowers_task_async_helper_expanded_match_project_with_nested_helper_expanded_control_flow() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_helper_expanded_match_demo",
    );

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

#[test]
fn compiles_task_async_while_flow_cond_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_flow_cond_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task async while flow-cond project should compile");
}

#[test]
fn lowers_task_async_while_flow_cond_project_with_async_flow_cond_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_flow_cond_demo",
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
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task async while post-flow project should compile");
}

#[test]
fn lowers_task_async_while_post_flow_project_with_async_post_flow_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_demo",
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
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_cond_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task async while post-flow cond project should compile");
}

#[test]
fn lowers_task_async_while_post_flow_cond_project_with_async_post_flow_cond_loop_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_cond_demo",
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
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_compound_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task async while post-flow compound project should compile");
}

#[test]
fn compiles_task_async_post_flow_recursive_branching_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_post_flow_recursive_branching_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task async post-flow recursive branching project should compile");
}

#[test]
fn rejects_task_async_memory_project_with_precise_sibling_carry_diagnostic() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_post_flow_memory_unsupported_demo",
    );
    let error = nuisc::pipeline::compile_project(project)
        .err()
        .expect("task async memory project should fail until lowering exists");
    assert!(error.contains(
        "references sibling carry `slot` before that carry is updated in the loop body"
    ));
}

#[test]
fn lowers_task_async_post_flow_recursive_branching_project_with_post_flow_recursive_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_post_flow_recursive_branching_demo",
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
        .any(|arg| arg == "prev_current_gt"));
    assert!(loop_node
        .op
        .args
        .iter()
        .any(|arg| arg == "add_prev_current"));
    assert!(loop_node.op.args.iter().any(|arg| arg == "keep"));
}

#[test]
fn lowers_task_async_while_post_flow_compound_project_with_async_post_flow_compound_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_compound_demo",
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

#[test]
fn compiles_task_runtime_project() {
    let project =
        Path::new("/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_runtime_demo");
    nuisc::pipeline::compile_project(project).expect("task runtime project should compile");
}

#[test]
fn lowers_task_runtime_project_with_completed_timeout_and_cancelled_shapes() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_runtime_demo",
    );

    let capture_lifecycle = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_lifecycle")
        .expect("expected capture_task_lifecycle function");
    assert!(capture_lifecycle.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "completed_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(task.as_ref(), NirExpr::Var(task_name) if task_name == "completed_task")
        )
    }));
    assert!(capture_lifecycle.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuTimeout { task, .. },
            } if name == "timed_task"
                && ty.render() == "Task<i64>"
                && matches!(task.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "ping")
        )
    }));
    assert!(capture_lifecycle.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuCancel(inner),
            } if name == "cancelled_task"
                && ty.render() == "Task<i64>"
                && matches!(inner.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "ping")
        )
    }));

    let encode_timed_out = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "encode_timed_out")
        .expect("expected encode_timed_out function");
    assert!(matches!(
        encode_timed_out.body.first(),
        Some(NirStmt::If {
            condition: NirExpr::CpuTaskTimedOut(_),
            ..
        })
    ));

    let encode_cancelled = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "encode_cancelled")
        .expect("expected encode_cancelled function");
    assert!(matches!(
        encode_cancelled.body.first(),
        Some(NirStmt::If {
            condition: NirExpr::CpuTaskCancelled(_),
            ..
        })
    ));
}

#[test]
fn compiles_task_status_observe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_status_observe_demo",
    );
    nuisc::pipeline::compile_project(project).expect("task status observe project should compile");
}

#[test]
fn lowers_task_status_observe_project_with_status_observer_shapes() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_status_observe_demo",
    );

    for (name, predicate) in [
        ("capture_completed", "completed"),
        ("capture_timed_out", "timed_out"),
        ("capture_cancelled", "cancelled"),
    ] {
        let function = artifacts
            .nir
            .functions
            .iter()
            .find(|function| function.name == name)
            .unwrap_or_else(|| panic!("expected {name} function"));
        match predicate {
            "completed" => assert!(matches!(
                function.body.first(),
                Some(NirStmt::If {
                    condition: NirExpr::CpuTaskCompleted(_),
                    ..
                })
            )),
            "timed_out" => assert!(matches!(
                function.body.first(),
                Some(NirStmt::If {
                    condition: NirExpr::CpuTaskTimedOut(_),
                    ..
                })
            )),
            "cancelled" => assert!(matches!(
                function.body.first(),
                Some(NirStmt::If {
                    condition: NirExpr::CpuTaskCancelled(_),
                    ..
                })
            )),
            _ => unreachable!(),
        }
    }

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
                value: NirExpr::CpuJoinResult(_),
            } if name == "completed_result" && ty.render() == "TaskResult<i64>"
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(_),
            } if name == "timed_result" && ty.render() == "TaskResult<i64>"
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(_),
            } if name == "cancelled_result" && ty.render() == "TaskResult<i64>"
        )
    }));
}

#[test]
fn compiles_task_completed_observe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_completed_observe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task completed observe project should compile");
}

#[test]
fn lowers_task_completed_observe_project_with_join_result_and_task_value_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_completed_observe_demo",
    );

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
                value: NirExpr::CpuJoinResult(task),
            } if name == "result"
                && ty.render() == "TaskResult<i64>"
                && matches!(task.as_ref(), NirExpr::Var(task_name) if task_name == "task")
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If {
                condition: NirExpr::CpuTaskCompleted(_),
                then_body,
                ..
            } if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::CpuTaskValue(_)))]
            )
        )
    }));
}

#[test]
fn compiles_task_compare_observe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_compare_observe_demo",
    );
    nuisc::pipeline::compile_project(project).expect("task compare observe project should compile");
}

#[test]
fn lowers_task_compare_observe_project_with_direct_and_observed_join_shapes() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_compare_observe_demo",
    );

    let capture_direct = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_direct_value")
        .expect("expected capture_direct_value function");
    assert!(capture_direct.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Return(Some(NirExpr::CpuJoin(task)))
                if matches!(task.as_ref(), NirExpr::Var(task_name) if task_name == "task")
        )
    }));

    let capture_observed = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_observed_value")
        .expect("expected capture_observed_value function");
    assert!(capture_observed.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(_),
            } if name == "result" && ty.render() == "TaskResult<i64>"
        )
    }));
    assert!(capture_observed.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If {
                condition: NirExpr::CpuTaskCompleted(_),
                then_body,
                ..
            } if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::CpuTaskValue(_)))]
            )
        )
    }));
}

#[test]
fn compiles_task_memory_roundtrip_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_roundtrip_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task memory roundtrip project should compile");
}

#[test]
fn compiles_task_memory_result_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_result_branch_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task memory result branch project should compile");
}

#[test]
fn compiles_task_memory_result_batch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_result_batch_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task memory result batch project should compile");
}

#[test]
fn compiles_task_memory_session_policy_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_session_policy_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task memory session policy project should compile");
}

#[test]
fn lowers_task_memory_session_policy_project_with_task_memory_session_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_session_policy_demo",
    );

    let capture_session = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_memory_session")
        .expect("expected capture_task_memory_session function");
    assert!(capture_session.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "primary_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(
                    task.as_ref(),
                    NirExpr::CpuSpawn { callee, .. } if callee == "primary"
                )
        )
    }));
    assert!(capture_session.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "secondary_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(
                    task.as_ref(),
                    NirExpr::CpuSpawn { callee, .. } if callee == "secondary"
                )
        )
    }));
    assert!(capture_session.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "fallback_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(
                    task.as_ref(),
                    NirExpr::CpuTimeout { task: inner, .. }
                        if matches!(inner.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "primary")
                )
        )
    }));
    let stage_session = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "stage_session_value")
        .expect("expected stage_session_value function");
    assert!(
        stage_session.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::AllocBuffer { .. },
                } if name == "scratch"
                    && ty.name == "Buffer"
                    && ty.is_ref
            )
        }),
        "expected alloc_buffer staging let: {:?}",
        stage_session.body,
    );
    assert!(
        stage_session
            .body
            .iter()
            .any(|stmt| { matches!(stmt, NirStmt::Expr(NirExpr::StoreAt { .. })) }),
        "expected store_at staging expr: {:?}",
        stage_session.body,
    );
    assert!(
        stage_session
            .body
            .iter()
            .any(|stmt| { matches!(stmt, NirStmt::Expr(NirExpr::Free(_))) }),
        "expected free staging expr: {:?}",
        stage_session.body,
    );
}

#[test]
fn compiles_task_memory_session_packet_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_memory_session_packet_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task memory session packet project should compile");
}

#[test]
fn compiles_task_result_policy_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_policy_branch_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task result policy branch project should compile");
}

#[test]
fn lowers_task_result_policy_branch_project_with_branch_selection_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_policy_branch_demo",
    );

    let select_value = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "select_value")
        .expect("expected select_value function");
    assert_eq!(
        select_value
            .body
            .iter()
            .filter(|stmt| matches!(stmt, NirStmt::If { .. }))
            .count(),
        3
    );
    assert!(matches!(
        select_value.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Int(0))))
    ));

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_result_policy")
        .expect("expected capture_task_result_policy function");
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "primary_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(task.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "primary")
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "secondary_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(task.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "secondary")
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "fallback_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(
                    task.as_ref(),
                    NirExpr::CpuTimeout { task: inner, .. }
                        if matches!(inner.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "primary")
                )
        )
    }));
}

#[test]
fn compiles_task_fallback_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_fallback_branch_demo",
    );
    nuisc::pipeline::compile_project(project).expect("task fallback branch project should compile");
}

#[test]
fn lowers_task_fallback_branch_project_with_timeout_fallback_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_fallback_branch_demo",
    );

    let select_value = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "select_value")
        .expect("expected select_value function");
    assert_eq!(
        select_value
            .body
            .iter()
            .filter(|stmt| matches!(stmt, NirStmt::If { .. }))
            .count(),
        2
    );
    assert!(matches!(
        select_value.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Int(0))))
    ));

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_fallback")
        .expect("expected capture_task_fallback function");
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuTimeout { task, .. },
            } if name == "primary_task"
                && ty.render() == "Task<i64>"
                && matches!(task.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "primary")
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuSpawn { callee, .. },
            } if name == "fallback_task" && ty.render() == "Task<i64>" && callee == "fallback"
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "primary_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(task.as_ref(), NirExpr::Var(task_name) if task_name == "primary_task")
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "fallback_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(task.as_ref(), NirExpr::Var(task_name) if task_name == "fallback_task")
        )
    }));
}

#[test]
fn compiles_task_result_family_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_family_branch_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task result family branch project should compile");
}

#[test]
fn lowers_task_result_family_branch_project_with_result_family_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_family_branch_demo",
    );

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_result_family")
        .expect("expected capture_task_result_family function");
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "completed_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(task.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "ping")
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "timed_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(
                    task.as_ref(),
                    NirExpr::CpuTimeout { task: inner, .. }
                        if matches!(inner.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "ping")
                )
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "cancelled_result"
                && ty.render() == "TaskResult<i64>"
                && matches!(
                    task.as_ref(),
                    NirExpr::CpuCancel(inner)
                        if matches!(inner.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "ping")
                )
        )
    }));

    let encode_cancelled = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "encode_cancelled")
        .expect("expected encode_cancelled function");
    assert!(matches!(
        encode_cancelled.body.first(),
        Some(NirStmt::If {
            condition: NirExpr::CpuTaskCancelled(_),
            then_body,
            ..
        }) if matches!(then_body.as_slice(), [NirStmt::Return(Some(NirExpr::Int(1)))])
    ));

    let encode_value = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "encode_value")
        .expect("expected encode_value function");
    assert!(matches!(
        encode_value.body.first(),
        Some(NirStmt::If {
            then_body,
            ..
        }) if matches!(
            then_body.as_slice(),
            [NirStmt::Return(Some(NirExpr::CpuTaskValue(_)))]
        )
    ));
}

#[test]
fn compiles_task_batch_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_batch_branch_demo",
    );
    nuisc::pipeline::compile_project(project).expect("task batch branch project should compile");
}

#[test]
fn lowers_task_batch_branch_project_with_batch_summary_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_batch_branch_demo",
    );

    let capture_batch = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_batch")
        .expect("expected capture_task_batch function");
    for (task_name, callee) in [
        ("alpha_task", "alpha"),
        ("beta_task", "beta"),
        ("gamma_task", "gamma"),
    ] {
        assert!(capture_batch.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::CpuSpawn { callee: stmt_callee, .. },
                } if name == task_name && ty.render() == "Task<i64>" && stmt_callee == callee
            )
        }));
    }
    for (result_name, task_name) in [
        ("alpha_result", "alpha_task"),
        ("beta_result", "beta_task"),
        ("gamma_result", "gamma_task"),
    ] {
        assert!(capture_batch.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::CpuJoinResult(task),
                } if name == result_name
                    && ty.render() == "TaskResult<i64>"
                    && matches!(task.as_ref(), NirExpr::Var(bound_task) if bound_task == task_name)
            )
        }));
    }
    assert!(matches!(
        capture_batch.body.last(),
        Some(NirStmt::Return(Some(NirExpr::StructLiteral { type_name, fields, .. })))
            if type_name == "TaskBatchSummary"
                && fields.iter().any(|(field, value)| {
                    field == "alpha_completed" && matches!(value, NirExpr::Call { callee, .. } if callee == "encode_completed")
                })
                && fields.iter().any(|(field, value)| {
                    field == "batch_value" && matches!(value, NirExpr::Binary { .. })
                })
    ));
}

#[test]
fn compiles_task_result_batch_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_batch_branch_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task result batch branch project should compile");
}

#[test]
fn lowers_task_result_batch_branch_project_with_result_batch_summary_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_batch_branch_demo",
    );

    let capture_batch = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_result_batch")
        .expect("expected capture_task_result_batch function");
    for (result_name, callee) in [
        ("alpha_result", "alpha"),
        ("beta_result", "beta"),
        ("gamma_result", "gamma"),
    ] {
        assert!(capture_batch.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::CpuJoinResult(task),
                } if name == result_name
                    && ty.render() == "TaskResult<i64>"
                    && matches!(task.as_ref(), NirExpr::CpuSpawn { callee: stmt_callee, .. } if stmt_callee == callee)
            )
        }));
    }
    assert!(matches!(
        capture_batch.body.last(),
        Some(NirStmt::Return(Some(NirExpr::StructLiteral { type_name, fields, .. })))
            if type_name == "TaskResultBatchSummary"
                && fields.iter().any(|(field, value)| {
                    field == "alpha_completed" && matches!(value, NirExpr::Call { callee, .. } if callee == "encode_completed")
                })
                && fields.iter().any(|(field, value)| {
                    field == "batch_value" && matches!(value, NirExpr::Binary { .. })
                })
    ));
}

#[test]
fn compiles_task_windowed_batch_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_windowed_batch_branch_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task windowed batch branch project should compile");
}

#[test]
fn lowers_task_windowed_batch_branch_project_with_windowed_summary_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_windowed_batch_branch_demo",
    );

    let capture_batch = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_batch")
        .expect("expected capture_task_batch function");
    for (task_name, callee) in [
        ("alpha_task", "alpha"),
        ("beta_task", "beta"),
        ("gamma_task", "gamma"),
    ] {
        assert!(capture_batch.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::CpuSpawn { callee: stmt_callee, .. },
                } if name == task_name && ty.render() == "Task<i64>" && stmt_callee == callee
            )
        }));
    }
    for (result_name, task_name) in [
        ("alpha_result", "alpha_task"),
        ("beta_result", "beta_task"),
        ("gamma_result", "gamma_task"),
    ] {
        assert!(capture_batch.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::CpuJoinResult(task),
                } if name == result_name
                    && ty.render() == "TaskResult<i64>"
                    && matches!(task.as_ref(), NirExpr::Var(bound_task) if bound_task == task_name)
            )
        }));
    }

    let capture_windowed = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_windowed_batch")
        .expect("expected capture_task_windowed_batch function");
    assert!(capture_windowed.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::Call { callee, .. },
            } if name == "summary"
                && ty.render() == "TaskBatchSummary"
                && callee == "capture_task_batch"
        )
    }));
    assert!(matches!(
        capture_windowed.body.last(),
        Some(NirStmt::Return(Some(NirExpr::StructLiteral { type_name, fields, .. })))
            if type_name == "TaskWindowedBatchSummary"
                && fields.iter().any(|(field, value)| {
                    field == "preview_value" && matches!(value, NirExpr::Binary { .. })
                })
                && fields.iter().any(|(field, value)| {
                    field == "final_value"
                        && matches!(
                            value,
                            NirExpr::FieldAccess { field: inner_field, .. } if inner_field == "batch_value"
                        )
                })
    ));
}

#[test]
fn compiles_task_result_windowed_batch_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_windowed_batch_branch_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task result windowed batch branch project should compile");
}

#[test]
fn lowers_task_result_windowed_batch_branch_project_with_result_windowed_summary_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_result_windowed_batch_branch_demo",
    );

    let capture_batch = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_result_batch")
        .expect("expected capture_task_result_batch function");
    for (result_name, callee) in [
        ("alpha_result", "alpha"),
        ("beta_result", "beta"),
        ("gamma_result", "gamma"),
    ] {
        assert!(capture_batch.body.iter().any(|stmt| {
            matches!(
                stmt,
                NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value: NirExpr::CpuJoinResult(task),
                } if name == result_name
                    && ty.render() == "TaskResult<i64>"
                    && matches!(task.as_ref(), NirExpr::CpuSpawn { callee: stmt_callee, .. } if stmt_callee == callee)
            )
        }));
    }

    let encode_value = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "encode_value")
        .expect("expected encode_value function");
    assert!(matches!(
        encode_value.body.first(),
        Some(NirStmt::If {
            then_body,
            ..
        }) if matches!(
            then_body.as_slice(),
            [NirStmt::Return(Some(NirExpr::CpuTaskValue(_)))]
        )
    ));

    let capture_windowed = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_result_windowed_batch")
        .expect("expected capture_task_result_windowed_batch function");
    assert!(capture_windowed.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::Call { callee, .. },
            } if name == "summary"
                && ty.render() == "TaskResultBatchSummary"
                && callee == "capture_task_result_batch"
        )
    }));
    assert!(matches!(
        capture_windowed.body.last(),
        Some(NirStmt::Return(Some(NirExpr::StructLiteral { type_name, fields, .. })))
            if type_name == "TaskResultWindowedBatchSummary"
                && fields.iter().any(|(field, value)| {
                    field == "preview_value" && matches!(value, NirExpr::Binary { .. })
                })
                && fields.iter().any(|(field, value)| {
                    field == "final_value"
                        && matches!(
                            value,
                            NirExpr::FieldAccess { field: inner_field, .. } if inner_field == "batch_value"
                        )
                })
    ));
}

#[test]
fn compiles_task_lifecycle_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_lifecycle_branch_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task lifecycle branch project should compile");
}

#[test]
fn lowers_task_lifecycle_branch_project_with_timeout_branch_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_lifecycle_branch_demo",
    );

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
                value: NirExpr::CpuTimeout { task, .. },
            } if name == "task"
                && ty.render() == "Task<i64>"
                && matches!(task.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "ping")
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(_),
            } if name == "result" && ty.render() == "TaskResult<i64>"
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If {
                then_body,
                else_body,
                ..
            } if matches!(
                then_body.as_slice(),
                [
                    NirStmt::Let { name, .. },
                    NirStmt::Print(_),
                    NirStmt::Return(Some(_))
                ] if name == "summary"
            ) && matches!(
                else_body.as_slice(),
                [
                    NirStmt::Let { name, .. },
                    NirStmt::Print(_),
                    NirStmt::Return(Some(_))
                ] if name == "summary"
            )
        )
    }));
}

#[test]
fn compiles_task_cancel_branch_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_cancel_branch_demo",
    );
    nuisc::pipeline::compile_project(project).expect("task cancel branch project should compile");
}

#[test]
fn lowers_task_cancel_branch_project_with_cancelled_branch_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_cancel_branch_demo",
    );

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
                value: NirExpr::CpuCancel(inner),
            } if name == "task"
                && ty.render() == "Task<i64>"
                && matches!(inner.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "ping")
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(task),
            } if name == "result"
                && ty.render() == "TaskResult<i64>"
                && matches!(task.as_ref(), NirExpr::Var(task_name) if task_name == "task")
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If {
                condition: NirExpr::CpuTaskCancelled(_),
                then_body,
                else_body,
            } if matches!(
                then_body.as_slice(),
                [
                    NirStmt::Let { name, .. },
                    NirStmt::Print(_),
                    NirStmt::Return(Some(_))
                ] if name == "summary"
            ) && matches!(
                else_body.as_slice(),
                [
                    NirStmt::Let { name, .. },
                    NirStmt::Print(_),
                    NirStmt::Return(Some(_))
                ] if name == "summary"
            )
        )
    }));
}

#[test]
fn compiles_task_cli_tooling_project() {
    let project =
        Path::new("/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_cli_tooling_demo");
    nuisc::pipeline::compile_project(project).expect("task cli tooling project should compile");
}

#[test]
fn lowers_task_cli_tooling_project_with_timeout_and_host_io_shape() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_cli_tooling_demo",
    );

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
                value: NirExpr::CpuExternCall { callee, .. },
            } if name == "argv_count"
                && ty.render() == "i64"
                && callee == "host_argv_count"
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuTimeout { task, .. },
            } if name == "task"
                && ty.render() == "Task<i64>"
                && matches!(task.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "ping")
        )
    }));
    assert!(main.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::If {
                condition: NirExpr::CpuTaskCompleted(_),
                then_body,
                ..
            } if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
                    if callee == "emit_completed_cli"
            )
        )
    }));
    assert!(matches!(
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "emit_timeout_cli"
    ));

    let emit_completed = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "emit_completed_cli")
        .expect("expected emit_completed_cli function");
    assert!(emit_completed.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuExternCall { callee, .. },
            } if name == "stdout_code"
                && ty.render() == "i64"
                && callee == "host_stdout_write"
        )
    }));

    let emit_timeout = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "emit_timeout_cli")
        .expect("expected emit_timeout_cli function");
    assert!(emit_timeout.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuExternCall { callee, .. },
            } if name == "stderr_code"
                && ty.render() == "i64"
                && callee == "host_stderr_write"
        )
    }));
}

#[test]
fn compiles_task_scheduler_observe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_scheduler_observe_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task scheduler observe project should compile");
}

#[test]
fn lowers_task_scheduler_observe_project_with_scheduler_and_timeout_shapes() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_scheduler_observe_demo",
    );

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_scheduler_project")
        .expect("expected capture_task_scheduler_project function");
    assert!(capture
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::Expr(NirExpr::CpuBindCore(0)))));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuTimeout { task, .. },
            } if name == "task"
                && ty.render() == "Task<i64>"
                && matches!(task.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "ping")
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuTickI64 { .. },
            } if name == "scheduler_tick" && ty.render() == "i64"
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuExternCall { callee, .. },
            } if name == "monotonic_ns"
                && ty.render() == "i64"
                && callee == "host_monotonic_time_ns"
        )
    }));
}

#[test]
fn compiles_task_clock_observe_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_clock_observe_demo",
    );
    nuisc::pipeline::compile_project(project).expect("task clock observe project should compile");
}

#[test]
fn lowers_task_clock_observe_project_with_clock_host_observer_shapes() {
    let artifacts = compiled_project(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_clock_observe_demo",
    );

    let capture = artifacts
        .nir
        .functions
        .iter()
        .find(|function| function.name == "capture_task_clock_project")
        .expect("expected capture_task_clock_project function");
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuTimeout { task, .. },
            } if name == "task"
                && ty.render() == "Task<i64>"
                && matches!(task.as_ref(), NirExpr::CpuSpawn { callee, .. } if callee == "ping")
        )
    }));
    assert!(capture.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))
                if fields.iter().any(|(field, value)| {
                    field == "global_domain_id" && matches!(value, NirExpr::Binary { .. })
                }) && fields.iter().any(|(field, value)| {
                    field == "global_epoch_ns"
                        && matches!(
                            value,
                            NirExpr::CpuExternCall { callee, .. } if callee == "host_clock_epoch_ns"
                        )
                }) && fields.iter().any(|(field, value)| {
                    field == "monotonic_ns"
                        && matches!(
                            value,
                            NirExpr::CpuExternCall { callee, .. } if callee == "host_monotonic_time_ns"
                        )
                }) && fields.iter().any(|(field, value)| {
                    field == "global_tick" && matches!(value, NirExpr::CpuTickI64 { .. })
                }) && fields.iter().any(|(field, value)| {
                    field == "global_scale_ppm"
                        && matches!(
                            value,
                            NirExpr::CpuExternCall { callee, .. } if callee == "host_clock_scale_ppm"
                        )
                })
        )
    }));
}

#[test]
fn compiles_task_httpish_response_packet_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_httpish_response_packet_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task httpish response packet project should compile");
}

#[test]
fn compiles_task_httpish_session_policy_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_httpish_session_policy_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task httpish session policy project should compile");
}

#[test]
fn compiles_task_httpish_response_slots_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_httpish_response_slots_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task httpish response slots project should compile");
}

#[test]
fn compiles_task_httpish_header_session_project() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_httpish_header_session_demo",
    );
    nuisc::pipeline::compile_project(project)
        .expect("task httpish header session project should compile");
}
