use super::continuations::bridge;
use super::nested_returns::{registration, with_step};
use super::record_helpers::compile;
#[cfg(unix)]
use super::record_helpers::helper_body;
use super::*;

pub(super) fn helpers(body: &str) -> String {
    with_step(body).replace(
        "mod cpu Main {",
        r#"mod cpu Main {
          type Decision = bool;
          struct Choice { value: bool }
          fn choose(flag: bool) -> Choice { return Choice { value: flag }; }
          fn offset(value: i64) -> i64 { return value + 1; }
          fn positive(value: i64) -> bool { return choose(offset(value) > 1).value; }
        "#,
    )
}

fn source() -> String {
    helpers(
        r#"
      let saved: Bytes = copy_bytes(pixels);
      let gate: bool = mode % 2 != 0;
      if mode < 2 {
        let gate: bool = !gate && positive(width);
        store_at(pixels, 1, width + 10);
        if gate { return Result.Ok(Canvas { width: width + 110, pixels: move(saved) }); }
      } else {
        const gate: Decision = !gate || choose(width < 0).value;
        store_at(pixels, 2, width + 20);
        if gate { return Result.Ok(Canvas { width: width + 220, pixels: move(saved) }); }
      }
      if !gate { return Result.Err(99); }
      drop_bytes(saved);
      store_at(pixels, 0, load_at(pixels, 0) + 1);
      return Result.Ok(Canvas { width: width + 1, pixels: copy_bytes(pixels) });
    "#,
    )
}

#[test]
#[cfg(unix)]
fn short_circuit_shadows_preserve_four_owner_paths_for_120_events_per_form() {
    for form in ["explicit", "inferred", "alias"] {
        let source = match form {
            "inferred" => source().replace("let gate: bool", "let gate"),
            "alias" => source().replace("let gate: bool", "let gate: Decision"),
            _ => source(),
        };
        let program = compile(&source);
        assert_eq!(
            program.ast,
            nuisc::frontend::parse_nuis_ast(&source).unwrap()
        );
        assert_eq!(
            program.yir,
            yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap()
        );
        let bridge = bridge(&program);
        assert!(!bridge.llvm_ir.contains("; deferred lowering"));
        for function in &program.yir.functions {
            if function.result.as_ref().is_some_and(|r| r.ty == "bool") {
                let body = helper_body(&bridge.llvm_ir, &function.name);
                assert!(body.starts_with("define i1"));
                assert!(!body.contains("@nuis_scheduler_owned_"));
            }
        }
        let library = native::Library::build(&source, &program, &bridge.llvm_ir);
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        let mut pixels = vec![9; 6];
        let mut width = 3;
        for mode in [0, 1, 2, 3].into_iter().cycle().take(120) {
            match mode {
                0 => width += 110,
                1 => {
                    pixels[1] = width + 10;
                    pixels[0] += 1;
                    width += 1;
                }
                2 => width += 220,
                _ => {
                    pixels[2] = width + 20;
                    pixels[0] += 1;
                    width += 1;
                }
            }
            session.event(mode).unwrap();
            assert_eq!(session.state(), pixels, "{form} mode={mode}");
            assert_eq!(session.scalar_state(), [width]);
            assert_eq!(library.live_blobs(), 0);
        }
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
#[cfg(unix)]
fn boolean_helper_errors_and_return_failures_keep_state_and_release_owners() {
    for allocation_failure in [false, true] {
        let source = if allocation_failure {
            source()
        } else {
            source()
                .replace(
                    "return Result.Ok(Canvas { width: width + 110, pixels: move(saved) });",
                    "return Result.Err(width + 110);",
                )
                .replace(
                    "return Result.Ok(Canvas { width: width + 220, pixels: move(saved) });",
                    "return Result.Err(width + 220);",
                )
        };
        let program = compile(&source);
        let bridge = bridge(&program);
        let mut llvm = bridge.llvm_ir.clone();
        if allocation_failure {
            for function in &program.yir.functions {
                if function.name.starts_with("__nuis_owned_") {
                    let body = helper_body(&bridge.llvm_ir, &function.name);
                    llvm = llvm.replacen(
                        body,
                        &body.replace(
                            "@nuis_scheduler_owned_aggregate_alloc_v1(i64 4)",
                            "@nuis_scheduler_owned_aggregate_alloc_v1(i64 0)",
                        ),
                        1,
                    );
                }
            }
            assert_ne!(llvm, bridge.llvm_ir);
        }
        let library = native::Library::build(&source, &program, &llvm);
        for mode in 0..4 {
            if !allocation_failure && mode % 2 != 0 {
                continue;
            }
            let (mut session, _) = NativeResourceSession::open(
                &program.yir,
                &registration(),
                library.checked_binding(&program.yir, &bridge),
                vec![9; 6],
                3,
            )
            .unwrap();
            if !allocation_failure {
                session.event(1).unwrap();
            }
            let pixels = session.state().to_vec();
            let scalars = session.scalar_state().to_vec();
            let error = session.event(mode).unwrap_err();
            assert!(
                error.contains(if allocation_failure {
                    "status 2"
                } else if mode == 0 {
                    "code=114"
                } else {
                    "code=224"
                }),
                "{error}"
            );
            assert_eq!(session.state(), pixels);
            assert_eq!(session.scalar_state(), scalars);
            assert_eq!(library.live_blobs(), 0);
            session.close(0).unwrap();
            assert!(session.completion_status().is_err());
            assert_eq!(library.live_blobs(), 0);
        }
    }
}

#[test]
fn boolean_helpers_reject_signature_type_effect_and_external_dependency_drift() {
    let program = compile(&source());
    bridge(&program);
    let branch = program
        .yir
        .functions
        .iter()
        .find(|f| {
            f.body_nodes.iter().any(|name| {
                program
                    .yir
                    .nodes
                    .iter()
                    .any(|n| &n.name == name && n.op.instruction == "guard_return")
            }) && f.result.as_ref().is_some_and(|r| r.ty == "bool")
        })
        .unwrap();
    for case in 0..17 {
        let mut changed = program.yir.clone();
        let guard = changed
            .nodes
            .iter()
            .position(|n| branch.body_nodes.contains(&n.name) && n.op.instruction == "guard_return")
            .unwrap();
        let bool_param = branch
            .parameters
            .iter()
            .find(|p| p.ty == "bool")
            .unwrap()
            .node
            .clone();
        let int_param = branch
            .parameters
            .iter()
            .find(|p| p.ty == "i64")
            .unwrap()
            .node
            .clone();
        let call = changed
            .nodes
            .iter()
            .position(|n| n.op.instruction == "call_bool" && n.op.args[0] == branch.name)
            .unwrap();
        let helper = changed
            .functions
            .iter_mut()
            .find(|f| f.name == branch.name)
            .unwrap();
        match case {
            0 => helper.parameters[0].ownership = yir_core::YirValueOwnership::Owned,
            1 => helper.result.as_mut().unwrap().ownership = yir_core::YirValueOwnership::Owned,
            2 => helper.result.as_mut().unwrap().ty = "i64".into(),
            3 => changed.nodes[call].op.args[0] = "missing".into(),
            4 => {
                changed.nodes[call].op.args.pop();
            }
            5 => changed.nodes[call].op.args.swap(1, 2),
            6 => changed.nodes[guard].op.args[0] = int_param.clone(),
            7 => changed.nodes[guard].op.args[1] = int_param.clone(),
            8 => {
                let returned = helper.result.as_ref().unwrap().node.clone();
                changed
                    .nodes
                    .iter_mut()
                    .find(|n| n.name == returned)
                    .unwrap()
                    .op
                    .args[0] = int_param.clone();
            }
            9 => {
                let node = &mut changed.nodes[guard];
                node.op.instruction = "print".into();
                node.op.args = vec![int_param.clone()];
            }
            10 => changed.edges.push(yir_core::Edge {
                kind: yir_core::EdgeKind::Lifetime,
                from: registration().bind(&program.yir).unwrap()[1].parameters[0]
                    .node
                    .clone(),
                to: bool_param.clone(),
            }),
            11 => {
                let node = changed
                    .nodes
                    .iter_mut()
                    .find(|n| {
                        branch.body_nodes.contains(&n.name) && n.op.instruction == "call_bool"
                    })
                    .unwrap();
                node.op.args[1] = bool_param.clone();
            }
            12..=13 => {
                let select = changed
                    .nodes
                    .iter_mut()
                    .find(|n| n.op.instruction == "select")
                    .unwrap();
                select.op.args[if case == 12 { 0 } else { 2 }] = changed
                    .functions
                    .iter()
                    .flat_map(|f| &f.parameters)
                    .find(|p| {
                        p.ty == "i64"
                            && changed.node_lanes.get(&p.node)
                                == changed.node_lanes.get(&select.name)
                    })
                    .unwrap()
                    .node
                    .clone();
            }
            14 => {
                let node = changed
                    .nodes
                    .iter_mut()
                    .find(|n| {
                        branch.body_nodes.contains(&n.name) && n.op.instruction == "const_bool"
                    })
                    .unwrap();
                node.op.instruction = "const_i64".into();
                node.op.args = vec!["0".into()];
            }
            15..=16 => {
                let node = changed
                    .nodes
                    .iter_mut()
                    .find(|n| branch.body_nodes.contains(&n.name) && n.op.instruction == "ne")
                    .unwrap();
                node.op.instruction = if case == 15 { "lt" } else { "ne_i64" }.into();
            }
            _ => unreachable!(),
        }
        assert!(
            yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration())
                .is_err(),
            "case={case}"
        );
    }
}

#[test]
#[cfg(unix)]
fn short_circuit_snapshot_survives_later_buffer_writes() {
    let source = helpers(
        r#"
      let saved: Bytes = copy_bytes(pixels);
      let ready = load_at(pixels, 0) == 9;
      let decision = ready && positive(width);
      if mode < 2 {
        store_at(pixels, 0, 0);
        if mode == 0 { return Result.Ok(Canvas { width: width + 100, pixels: move(saved) }); }
      }
      if !decision { return Result.Err(71); }
      drop_bytes(saved);
      return Result.Ok(Canvas { width: offset(width), pixels: copy_bytes(pixels) });
    "#,
    );
    let program = compile(&source);
    let bridge = bridge(&program);
    let library = native::Library::build(&source, &program, &bridge.llvm_ir);
    for (mode, pixels, width) in [(0, [9; 6], 103), (1, [0, 9, 9, 9, 9, 9], 4), (2, [9; 6], 4)] {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        session.event(mode).unwrap();
        assert_eq!(session.state(), pixels);
        assert_eq!(session.scalar_state(), [width]);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
#[cfg(unix)]
fn nested_logical_helpers_preserve_the_selected_truth_table() {
    let source = helpers(
        r#"
      let saved: Bytes = copy_bytes(pixels);
      let decision = mode == 0 || (mode == 1 && positive(width));
      if decision { return Result.Ok(Canvas { width: width + 10, pixels: move(saved) }); }
      drop_bytes(saved);
      return Result.Ok(Canvas { width: offset(width), pixels: copy_bytes(pixels) });
    "#,
    );
    let program = compile(&source);
    let bridge = bridge(&program);
    let library = native::Library::build(&source, &program, &bridge.llvm_ir);
    let (mut session, _) = NativeResourceSession::open(
        &program.yir,
        &registration(),
        library.checked_binding(&program.yir, &bridge),
        vec![9; 6],
        3,
    )
    .unwrap();
    let mut width = 3;
    for mode in [0, 1, 2, 3].into_iter().cycle().take(120) {
        width += if mode < 2 { 10 } else { 1 };
        session.event(mode).unwrap();
        assert_eq!(session.scalar_state(), [width]);
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}

fn helper_chain(count: usize, parallel: bool) -> yir_core::YirModule {
    let source = helpers("let gate = mode != 0 && choose(width > 0).value; if gate { return Result.Err(3); } return Result.Ok(Canvas { width: width, pixels: copy_bytes(pixels) });");
    let mut module = compile(&source).yir;
    let root_index = module
        .functions
        .iter()
        .position(|f| f.name == "step")
        .unwrap();
    let root_result = module.functions[root_index]
        .result
        .as_ref()
        .unwrap()
        .node
        .clone();
    let call_index = module
        .nodes
        .iter()
        .position(|n| {
            module.functions[root_index].body_nodes.contains(&n.name)
                && n.op.instruction == "call_bool"
        })
        .unwrap();
    let original = module.nodes[call_index].clone();
    let template = module
        .functions
        .iter()
        .find(|f| f.name == original.op.args[0])
        .unwrap()
        .clone();
    assert_eq!(
        template
            .parameters
            .iter()
            .map(|p| p.ty.as_str())
            .collect::<Vec<_>>(),
        ["bool", "i64"]
    );
    module.nodes[call_index].op.args[0] = "bound_flag0".into();
    // Build the post-optimization graph explicitly: source identity wrappers
    // may be erased and therefore cannot witness a native closure limit.
    for i in 0..count {
        let name = format!("bound_flag{i}");
        let lane = format!("fn:{name}");
        let mut function = template.clone();
        function.name = name.clone();
        function.body_nodes.clear();
        for (index, parameter) in function.parameters.iter_mut().enumerate() {
            let mut node = module
                .nodes
                .iter()
                .find(|n| n.name == template.parameters[index].node)
                .unwrap()
                .clone();
            node.name = format!("{name}_param{index}");
            parameter.node = node.name.clone();
            function.body_nodes.push(node.name.clone());
            module.node_lanes.insert(node.name.clone(), lane.clone());
            module.nodes.push(node);
        }
        let mut value = function.parameters[0].node.clone();
        if !parallel && i + 1 < count {
            let mut call = original.clone();
            call.name = format!("{name}_call");
            call.op.args = vec![
                format!("bound_flag{}", i + 1),
                function.parameters[0].node.clone(),
                function.parameters[1].node.clone(),
            ];
            value = call.name.clone();
            for input in &call.op.args[1..] {
                module.edges.push(yir_core::Edge {
                    kind: yir_core::EdgeKind::Dep,
                    from: input.clone(),
                    to: call.name.clone(),
                });
            }
            function.body_nodes.push(call.name.clone());
            module.node_lanes.insert(call.name.clone(), lane.clone());
            module.nodes.push(call);
        }
        let mut returned = module
            .nodes
            .iter()
            .find(|n| n.name == template.result.as_ref().unwrap().node)
            .unwrap()
            .clone();
        returned.name = format!("{name}_return");
        returned.op.args = vec![value];
        module.edges.push(yir_core::Edge {
            kind: yir_core::EdgeKind::Dep,
            from: returned.op.args[0].clone(),
            to: returned.name.clone(),
        });
        function.result.as_mut().unwrap().node = returned.name.clone();
        function.body_nodes.push(returned.name.clone());
        module.node_lanes.insert(returned.name.clone(), lane);
        module.nodes.push(returned);
        module.functions.push(function);
        if parallel && i > 0 {
            let mut call = original.clone();
            call.name = format!("bound_parallel{i}");
            call.op.args[0] = name;
            for input in &call.op.args[1..] {
                module.edges.push(yir_core::Edge {
                    kind: yir_core::EdgeKind::Dep,
                    from: input.clone(),
                    to: call.name.clone(),
                });
            }
            module.functions[root_index]
                .body_nodes
                .push(call.name.clone());
            module
                .node_lanes
                .insert(call.name.clone(), "fn:step".into());
            module.edges.push(yir_core::Edge {
                kind: yir_core::EdgeKind::Dep,
                from: call.name.clone(),
                to: root_result.clone(),
            });
            module.nodes.push(call);
        }
    }
    module
}

#[test]
fn scalar_record_and_resource_helpers_share_one_closure_budget() {
    for (count, parallel, allowed) in [
        (7, false, true),
        (8, false, false),
        (29, true, true),
        (30, true, false),
    ] {
        let module = helper_chain(count, parallel);
        let result =
            yir_lower_llvm::native_resource_session::emit_registered(&module, &registration());
        if allowed {
            result.unwrap();
        } else {
            assert!(result
                .err()
                .unwrap_or_else(|| panic!(
                    "unexpectedly admitted count={count}, parallel={parallel}"
                ))
                .contains("closure"));
        }
    }
    let mut cycle = helper_chain(2, false);
    cycle
        .nodes
        .iter_mut()
        .find(|n| n.op.instruction == "call_bool" && n.op.args[0] == "bound_flag1")
        .unwrap()
        .op
        .args[0] = "bound_flag0".into();
    assert!(
        yir_lower_llvm::native_resource_session::emit_registered(&cycle, &registration())
            .unwrap_err()
            .contains("recursive")
    );
}
#[test]
fn short_circuit_resource_prefix_negation_needs_no_separate_initializer() {
    let declaration = "const gate: Decision = !gate || choose(width < 0).value;";
    let wrapped = source().replace(
        declaration,
        "const gate: Decision = !(!gate || choose(width < 0).value);",
    );
    bridge(&compile(&wrapped));

    let separated = source().replace(
        declaration,
        "const selected: Decision = !gate || choose(width < 0).value; const gate: Decision = !selected;",
    );
    bridge(&compile(&separated));
}

#[test]
fn wrapped_resource_read_rhs_uses_owned_branch_normalization() {
    for op in ["&&", "||"] {
        for rhs in ["load_at(pixels, 99) > 0", "bytes_len(saved) > 0"] {
            let source = helpers(&format!(
                r#"
              let saved: Bytes = copy_bytes(pixels);
              let decision = (mode != 0 {op} (mode < 3 && {rhs})) ^ (mode == 0);
              if decision {{ return Result.Ok(Canvas {{ width: width + 1, pixels: move(saved) }}); }}
              return Result.Ok(Canvas {{ width: width, pixels: move(saved) }});
            "#
            ));
            assert!(!bridge(&compile(&source))
                .llvm_ir
                .contains("; deferred lowering"));
        }
    }
}
