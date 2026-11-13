use super::continuations::{bridge, compile_body};
use super::nested_returns::{registration, with_step};
use super::*;

const BODY: &str = r#"
    let original: Bytes = copy_bytes(pixels);
    let saved: Bytes = move(original);
    let spare: Bytes = copy_bytes(pixels);
    if mode < 3 {
      store_at(pixels, 1, width);
      let local: Bytes = copy_bytes(pixels);
      let count: i64 = bytes_len(local) / 8;
      if mode == 0 { return Result.Ok(Canvas { width: width + 100, pixels: move(saved) }); }
      if mode == 1 { return Result.Ok(Canvas { width: width + 200, pixels: move(local) }); }
      store_at(pixels, 2, count + width);
    } else {
      store_at(pixels, 3, width + 10);
      let local: Bytes = copy_bytes(pixels);
      let count: i64 = bytes_len(local) / 8;
      if mode == 3 { return Result.Ok(Canvas { width: width + 300, pixels: move(saved) }); }
      if mode == 4 { return Result.Ok(Canvas { width: width + 400, pixels: move(local) }); }
      store_at(pixels, 4, count + width + 10);
    }
    let count: i64 = bytes_len(saved) / 8;
    drop_bytes(saved);
    drop_bytes(spare);
    store_at(pixels, 0, load_at(pixels, 0) + 1);
    return Result.Ok(Canvas { width: width + count, pixels: copy_bytes(pixels) });
"#;

#[test]
#[cfg(unix)]
fn paired_arms_preserve_snapshots_and_share_one_suffix_for_120_events() {
    let program = compile_body(BODY).unwrap();
    let module =
        yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap();
    assert_eq!(module, program.yir);
    let bridge = bridge(&program);
    let shared = module
        .functions
        .iter()
        .filter(|f| f.name.starts_with("__nuis_owned_continue_"))
        .collect::<Vec<_>>();
    assert_eq!(shared.len(), 1);
    let tails = module
        .nodes
        .iter()
        .filter(|n| n.op.instruction == "tail_call_owned_struct_return")
        .map(|n| yir_core::guarded_owned_call::parse_tail(&n.op.args).unwrap())
        .collect::<Vec<_>>();
    let joins = tails
        .iter()
        .filter(|c| c.callee == shared[0].name)
        .collect::<Vec<_>>();
    assert_eq!(joins.len(), 2);
    assert!(joins.iter().all(|c| c.transfers.len() == 2));
    assert_eq!(
        module
            .nodes
            .iter()
            .filter(|n| n.op.instruction == "load_at")
            .count(),
        1
    );
    let root = module.functions.iter().find(|f| f.name == "step").unwrap();
    assert_eq!(
        module
            .nodes
            .iter()
            .filter(|n| root.body_nodes.contains(&n.name)
                && n.op.instruction == "guard_call_owned_struct_return")
            .count(),
        1
    );
    assert_eq!(
        module
            .nodes
            .iter()
            .find(|n| n.name == root.result.as_ref().unwrap().node)
            .unwrap()
            .op
            .instruction,
        "tail_call_owned_struct_return"
    );
    let library = native::Library::build(&with_step(BODY), &program, &bridge.llvm_ir);
    let (mut session, _) = NativeResourceSession::open(
        &module,
        &registration(),
        library.checked_binding(&module, &bridge),
        vec![9; 6],
        3,
    )
    .unwrap();
    let mut expected = vec![9; 6];
    let mut width = 3;
    for mode in [0, 1, 2, 3, 4, 5].into_iter().cycle().take(120) {
        match mode {
            0 => width += 100,
            1 => {
                expected[1] = width;
                width += 200;
            }
            2 => {
                expected[1] = width;
                expected[2] = width + 6;
                expected[0] += 1;
                width += 6;
            }
            3 => width += 300,
            4 => {
                expected[3] = width + 10;
                width += 400;
            }
            _ => {
                expected[3] = width + 10;
                expected[4] = width + 16;
                expected[0] += 1;
                width += 6;
            }
        }
        session.event(mode).unwrap();
        assert_eq!(session.state(), expected, "mode={mode}");
        assert_eq!(session.scalar_state(), [width]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}

#[test]
#[cfg(unix)]
fn paired_suffix_allocation_failure_preserves_state_on_both_continuing_edges() {
    let program = compile_body(BODY).unwrap();
    let bridge = bridge(&program);
    let shared = program
        .yir
        .functions
        .iter()
        .find(|f| f.name.starts_with("__nuis_owned_continue_"))
        .unwrap();
    let start = bridge
        .llvm_ir
        .find(&format!("define i64 @nuis_fn_{}(", shared.name))
        .unwrap();
    let end = start + bridge.llvm_ir[start..].find("\n}").unwrap();
    let body = &bridge.llvm_ir[start..end];
    let faulty = body.replace(
        "@nuis_scheduler_owned_aggregate_alloc_v1(i64 4)",
        "@nuis_scheduler_owned_aggregate_alloc_v1(i64 0)",
    );
    assert_ne!(body, faulty);
    let llvm = bridge.llvm_ir.replacen(body, &faulty, 1);
    let library = native::Library::build(&with_step(BODY), &program, &llvm);
    for mode in [2, 5] {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        for early in [0, 1, 3, 4] {
            session.event(early).unwrap();
        }
        let pixels = session.state().to_vec();
        let scalars = session.scalar_state().to_vec();
        assert!(session.event(mode).unwrap_err().contains("status 2"));
        assert_eq!(session.state(), pixels);
        assert_eq!(session.scalar_state(), scalars);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        assert!(session.completion_status().is_err());
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
#[cfg(unix)]
fn either_arm_can_return_an_error_without_publishing_or_running_the_suffix() {
    let body = BODY
        .replace(
            "return Result.Ok(Canvas { width: width + 100, pixels: move(saved) });",
            "return Result.Err(17);",
        )
        .replace(
            "return Result.Ok(Canvas { width: width + 300, pixels: move(saved) });",
            "return Result.Err(18);",
        );
    let program = compile_body(&body).unwrap();
    let bridge = bridge(&program);
    let library = native::Library::build(&with_step(&body), &program, &bridge.llvm_ir);
    for (mode, error) in [(0, "code=17"), (3, "code=18")] {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        session.event(2).unwrap();
        session.event(5).unwrap();
        let pixels = session.state().to_vec();
        let scalars = session.scalar_state().to_vec();
        assert!(session.event(mode).unwrap_err().contains(error));
        assert_eq!(session.state(), pixels);
        assert_eq!(session.scalar_state(), scalars);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        assert!(session.completion_status().is_err());
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
#[cfg(unix)]
fn unselected_arm_and_early_exit_suffix_arithmetic_remain_lazy() {
    let body = BODY.replacen(
        "let count: i64 = bytes_len(local) / 8;",
        "let count: i64 = (bytes_len(local) / 8) / (mode - 5);",
        1,
    );
    let body = body
        .replace(
            "let count: i64 = bytes_len(local) / 8;",
            "let count: i64 = (bytes_len(local) / 8) / mode;",
        )
        .replace(
            "let count: i64 = bytes_len(saved) / 8;",
            "let count: i64 = (bytes_len(saved) / 8) / (mode % 3);",
        );
    let program = compile_body(&body).unwrap();
    let bridge = bridge(&program);
    let library = native::Library::build(&with_step(&body), &program, &bridge.llvm_ir);
    for (mode, width, expected) in [
        (0, 103, [9, 9, 9, 9, 9, 9]),
        (3, 303, [9, 9, 9, 9, 9, 9]),
        (2, 6, [10, 3, 1, 9, 9, 9]),
        (5, 6, [10, 9, 9, 13, 14, 9]),
    ] {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        session.event(mode).unwrap();
        assert_eq!(session.scalar_state(), [width]);
        assert_eq!(session.state(), expected);
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
#[cfg(unix)]
fn buffer_predicate_is_not_retested_and_branch_locals_need_no_shared_owner() {
    let body = r#"
      if load_at(pixels, 0) == 9 {
        store_at(pixels, 0, 10);
        let local: Bytes = copy_bytes(pixels);
      } else {
        store_at(pixels, 0, 20);
        let local: Bytes = copy_bytes(pixels);
      }
      store_at(pixels, 1, load_at(pixels, 1) + 1);
      return Result.Ok(Canvas { width: width, pixels: copy_bytes(pixels) });
    "#;
    let program = compile_body(body).unwrap();
    let bridge = bridge(&program);
    assert!(program
        .yir
        .nodes
        .iter()
        .filter(|n| n.op.instruction == "tail_call_owned_struct_return")
        .all(|n| yir_core::guarded_owned_call::parse_tail(&n.op.args)
            .unwrap()
            .transfers
            .is_empty()));
    assert_eq!(
        program
            .yir
            .nodes
            .iter()
            .filter(|n| n.op.instruction == "load_at")
            .count(),
        2
    );
    let library = native::Library::build(&with_step(body), &program, &bridge.llvm_ir);
    let (mut session, _) = NativeResourceSession::open(
        &program.yir,
        &registration(),
        library.checked_binding(&program.yir, &bridge),
        vec![9; 6],
        3,
    )
    .unwrap();
    for (first, second) in [(10, 10), (20, 11), (20, 12)] {
        session.event(0).unwrap();
        assert_eq!(session.state(), [first, second, 9, 9, 9, 9]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
}

#[test]
fn paired_join_rejects_rebinding_consumption_and_drift_on_either_owner_edge() {
    for side in [
        "store_at(pixels, 2, count + width);",
        "store_at(pixels, 4, count + width + 10);",
    ] {
        for replacement in [
            "drop_bytes(saved);",
            "carry = width + 7; store_at(pixels, 5, carry);",
            "let renamed: Bytes = move(saved);",
        ] {
            let body = format!(
                "let mut carry: i64 = width; {}",
                BODY.replace(side, replacement)
                    .replace("width: width + count,", "width: carry + count,")
            );
            let result = compile_body(&body).and_then(|p| {
                yir_lower_llvm::native_resource_session::emit_registered(&p.yir, &registration())
            });
            assert!(result.is_err(), "{side}: {replacement}");
        }
    }
    let program = compile_body(BODY).unwrap();
    let shared = program
        .yir
        .functions
        .iter()
        .find(|f| f.name.starts_with("__nuis_owned_continue_"))
        .unwrap();
    let tails = program
        .yir
        .nodes
        .iter()
        .filter(|n| {
            n.op.instruction == "tail_call_owned_struct_return" && n.op.args[0] == shared.name
        })
        .collect::<Vec<_>>();
    assert_eq!(tails.len(), 2);
    for tail in tails {
        let call = yir_core::guarded_owned_call::parse_tail(&tail.op.args).unwrap();
        for owner in call.transfers {
            let mut changed = program.yir.clone();
            changed.edges.retain(|e| {
                !(e.from == *owner && e.to == tail.name && e.kind == yir_core::EdgeKind::Lifetime)
            });
            assert!(yir_lower_llvm::native_resource_session::emit_registered(
                &changed,
                &registration()
            )
            .is_err());
        }
        let mut changed = program.yir.clone();
        changed
            .nodes
            .iter_mut()
            .find(|n| n.name == tail.name)
            .unwrap()
            .op
            .args
            .truncate(2 + call.inputs.len());
        assert!(yir_lower_llvm::native_resource_session::emit_registered(
            &changed,
            &registration()
        )
        .is_err());
    }
}

#[test]
fn paired_shared_suffixes_obey_the_existing_closure_depth_budget() {
    for count in [3, 4] {
        let mut body = "let saved: Bytes = copy_bytes(pixels);".to_owned();
        for _ in 0..count {
            body.push_str(
                "if mode == 0 { store_at(pixels, 0, 1); } else { store_at(pixels, 0, 2); }",
            );
        }
        body.push_str("return Result.Ok(Canvas { width: width, pixels: move(saved) });");
        let result = compile_body(&body).and_then(|p| {
            yir_lower_llvm::native_resource_session::emit_registered(&p.yir, &registration())
        });
        if count == 3 {
            result.unwrap();
        } else {
            assert!(result.unwrap_err().contains("closure depth bound"));
        }
    }
}

#[test]
#[cfg(unix)]
fn nested_pairs_mix_terminal_and_continuing_arms_without_extra_suffix_work() {
    let body = r#"
      let saved: Bytes = copy_bytes(pixels);
      if mode < 2 {
        if mode == 0 {
          return Result.Ok(Canvas { width: 10, pixels: move(saved) });
        } else {
          store_at(pixels, 0, 20);
        }
        store_at(pixels, 1, 21);
      } else {
        if mode == 2 {
          return Result.Ok(Canvas { width: 30, pixels: copy_bytes(pixels) });
        } else {
          return Result.Ok(Canvas { width: 40, pixels: move(saved) });
        }
      }
      drop_bytes(saved);
      store_at(pixels, 2, 99);
      return Result.Ok(Canvas { width: 50, pixels: copy_bytes(pixels) });
    "#;
    let program = compile_body(body).unwrap();
    let bridge = bridge(&program);
    assert_eq!(
        program
            .yir
            .functions
            .iter()
            .filter(|f| f.name.starts_with("__nuis_owned_continue_"))
            .count(),
        2
    );
    let library = native::Library::build(&with_step(body), &program, &bridge.llvm_ir);
    for (mode, width, pixels) in [
        (0, 10, [9, 9, 9, 9, 9, 9]),
        (1, 50, [20, 21, 99, 9, 9, 9]),
        (2, 30, [9, 9, 9, 9, 9, 9]),
        (3, 40, [9, 9, 9, 9, 9, 9]),
    ] {
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
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
#[cfg(unix)]
fn a_complete_pair_needs_no_fabricated_suffix_or_trailing_source_return() {
    let body = r#"
      let saved: Bytes = copy_bytes(pixels);
      if mode == 0 {
        return Result.Ok(Canvas { width: width + 1, pixels: move(saved) });
      } else {
        store_at(pixels, 0, 20);
        return Result.Ok(Canvas { width: width + 2, pixels: copy_bytes(pixels) });
      }
    "#;
    let program = compile_body(body).unwrap();
    let bridge = bridge(&program);
    assert!(!program
        .yir
        .functions
        .iter()
        .any(|f| f.name.starts_with("__nuis_owned_continue_")));
    let library = native::Library::build(&with_step(body), &program, &bridge.llvm_ir);
    let (mut session, _) = NativeResourceSession::open(
        &program.yir,
        &registration(),
        library.checked_binding(&program.yir, &bridge),
        vec![9; 6],
        3,
    )
    .unwrap();
    session.event(0).unwrap();
    assert_eq!(session.scalar_state(), [4]);
    assert_eq!(session.state(), [9; 6]);
    session.event(1).unwrap();
    assert_eq!(session.scalar_state(), [6]);
    assert_eq!(session.state(), [20, 9, 9, 9, 9, 9]);
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}
