use super::nested_returns::{registration, with_step};
use super::*;

const BODY: &str = r#"
    let original: Bytes = copy_bytes(pixels);
    let saved: Bytes = move(original);
    if mode < 3 {
      store_at(pixels, 1, width);
      let local: Bytes = copy_bytes(pixels);
      let count: i64 = bytes_len(local) / 8;
      if mode == 0 {
        return Result.Ok(Canvas { width: width + 100, pixels: move(saved) });
      }
      if mode == 1 {
        return Result.Ok(Canvas { width: width + 200, pixels: move(local) });
      }
      store_at(pixels, 2, count + width);
    }
    let count: i64 = bytes_len(saved) / 8;
    drop_bytes(saved);
    store_at(pixels, 0, load_at(pixels, 0) + 1);
    return Result.Ok(Canvas { width: width + count, pixels: copy_bytes(pixels) });
"#;

pub(super) fn compile_body(body: &str) -> Result<nuisc::pipeline::PipelineArtifacts, String> {
    nuisc::pipeline::compile_resource_session_source(
        &with_step(body),
        &registration(),
        &Default::default(),
    )?
    .emit_llvm()
}

pub(super) fn bridge(
    program: &nuisc::pipeline::PipelineArtifacts,
) -> yir_lower_llvm::native_resource_session::NativeResourceBridge {
    yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration()).unwrap()
}

#[test]
#[cfg(unix)]
fn nonterminal_join_preserves_owners_and_runs_one_shared_suffix() {
    let program = compile_body(BODY).unwrap();
    let roundtrip =
        yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap();
    assert_eq!(roundtrip, program.yir);
    let bridge = bridge(&program);
    assert!(!bridge.llvm_ir.contains("; deferred lowering"));
    let tails = roundtrip
        .nodes
        .iter()
        .filter(|n| n.op.instruction == "tail_call_owned_struct_return")
        .map(|n| yir_core::guarded_owned_call::parse_tail(&n.op.args).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(tails.len(), 2);
    assert_eq!(tails[0].callee, tails[1].callee);
    assert!(tails.iter().all(|t| t.transfers.len() == 1));
    // The effectful suffix is represented once, not cloned into both paths.
    assert_eq!(
        roundtrip
            .nodes
            .iter()
            .filter(|n| n.op.instruction == "load_at")
            .count(),
        1
    );
    let library = native::Library::build(&with_step(BODY), &program, &bridge.llvm_ir);
    let (mut session, _) = NativeResourceSession::open(
        &roundtrip,
        &registration(),
        library.checked_binding(&roundtrip, &bridge),
        vec![9; 6],
        3,
    )
    .unwrap();
    let mut expected = vec![9; 6];
    let mut width = 3;
    for mode in [0, 1, 2, 3].into_iter().cycle().take(120) {
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
            _ => {
                expected[0] += 1;
                width += 6;
            }
        }
        session.event(mode).unwrap();
        assert_eq!(session.state(), expected);
        assert_eq!(session.scalar_state(), [width]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}

#[test]
#[cfg(unix)]
fn continuation_allocation_failure_is_forwarded_without_unpacking_or_publication() {
    let program = compile_body(BODY).unwrap();
    let bridge = bridge(&program);
    let tail = program
        .yir
        .nodes
        .iter()
        .find(|n| n.op.instruction == "tail_call_owned_struct_return")
        .unwrap();
    let callee = &tail.op.args[0];
    let start = bridge
        .llvm_ir
        .find(&format!("define i64 @nuis_fn_{callee}("))
        .unwrap();
    let end = start + bridge.llvm_ir[start..].find("\n}").unwrap();
    let original = &bridge.llvm_ir[start..end];
    let faulty = original.replace(
        "@nuis_scheduler_owned_aggregate_alloc_v1(i64 4)",
        "@nuis_scheduler_owned_aggregate_alloc_v1(i64 0)",
    );
    assert_ne!(original, faulty);
    let llvm = bridge.llvm_ir.replacen(original, &faulty, 1);
    let library = native::Library::build(&with_step(BODY), &program, &llvm);
    for mode in [2, 3] {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        session.event(0).unwrap();
        session.event(1).unwrap();
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
fn inner_error_skips_shared_suffix_and_releases_ancestor_and_local_bytes() {
    let body = BODY.replace(
        "return Result.Ok(Canvas { width: width + 100, pixels: move(saved) });",
        "return Result.Err(17);",
    );
    let program = compile_body(&body).unwrap();
    let bridge = bridge(&program);
    let library = native::Library::build(&with_step(&body), &program, &bridge.llvm_ir);
    let (mut session, _) = NativeResourceSession::open(
        &program.yir,
        &registration(),
        library.checked_binding(&program.yir, &bridge),
        vec![9; 6],
        3,
    )
    .unwrap();
    session.event(2).unwrap();
    let pixels = session.state().to_vec();
    let scalars = session.scalar_state().to_vec();
    assert!(session.event(0).unwrap_err().contains("code=17"));
    assert_eq!(session.state(), pixels);
    assert_eq!(session.scalar_state(), scalars);
    assert_eq!(library.live_blobs(), 0);
    session.close(0).unwrap();
    assert!(session.completion_status().is_err());
    assert_eq!(library.live_blobs(), 0);
}

#[test]
#[cfg(unix)]
fn nested_open_arms_rejoin_in_order_without_repeating_the_root_suffix() {
    let body = r#"
      let saved: Bytes = copy_bytes(pixels);
      if mode < 3 {
        let local: Bytes = copy_bytes(pixels);
        if mode < 2 {
          if mode == 0 { return Result.Ok(Canvas { width: width, pixels: move(saved) }); }
          store_at(pixels, 1, load_at(pixels, 1) + 10);
        }
        store_at(pixels, 2, load_at(pixels, 2) + 100);
      }
      let count: i64 = bytes_len(saved) / 8;
      drop_bytes(saved);
      store_at(pixels, 0, load_at(pixels, 0) + 1);
      return Result.Ok(Canvas { width: width + count, pixels: copy_bytes(pixels) });
    "#;
    let program = compile_body(body).unwrap();
    let bridge = bridge(&program);
    assert_eq!(
        program
            .yir
            .nodes
            .iter()
            .filter(|n| n.op.instruction == "load_at")
            .count(),
        3
    );
    let library = native::Library::build(&with_step(body), &program, &bridge.llvm_ir);
    for (mode, expected, width) in [
        (0, [9, 9, 9], 3),
        (1, [10, 19, 109], 9),
        (2, [10, 9, 109], 9),
        (3, [10, 9, 9], 9),
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
        assert_eq!(&session.state()[..3], expected);
        assert_eq!(session.scalar_state(), [width]);
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
fn continuation_rejects_consumed_owners_rebinding_and_malformed_tail_contracts() {
    for body in [
        BODY.replace("store_at(pixels, 2, count + width);", "drop_bytes(saved);"),
        format!(
            "let mut carry: i64 = width; {}",
            BODY.replace(
                "let local: Bytes",
                "carry = width + 7; store_at(pixels, 5, carry); let local: Bytes"
            )
            .replace("width: width + count,", "width: carry + count,")
        ),
    ] {
        let result = compile_body(&body).and_then(|p| {
            yir_lower_llvm::native_resource_session::emit_registered(&p.yir, &registration())
        });
        assert!(result.is_err());
    }
    let program = compile_body(BODY).unwrap();
    for case in 0..6 {
        let mut changed = program.yir.clone();
        let tail = changed
            .nodes
            .iter()
            .position(|n| n.op.instruction == "tail_call_owned_struct_return")
            .unwrap();
        let call = yir_core::guarded_owned_call::parse_tail(&changed.nodes[tail].op.args).unwrap();
        let callee = call.callee.to_owned();
        let owner = call.transfers[0].clone();
        let count = call.inputs.len();
        match case {
            0 => {
                changed.nodes[tail].op.args.truncate(2 + count);
            }
            1 => {
                let name = &changed.nodes[tail].name;
                changed.edges.retain(|e| {
                    !(e.from == owner && e.to == *name && e.kind == yir_core::EdgeKind::Lifetime)
                });
            }
            2 => {
                changed
                    .functions
                    .iter_mut()
                    .find(|f| f.name == callee)
                    .unwrap()
                    .parameters[count]
                    .ownership = yir_core::YirValueOwnership::Borrowed;
            }
            3 => {
                changed.nodes[tail].op.args[1] = "Other{pixels:Bytes}".into();
            }
            4 => {
                let caller = changed
                    .functions
                    .iter()
                    .find(|f| f.body_nodes.contains(&changed.nodes[tail].name))
                    .unwrap();
                changed.nodes[tail].op.args[0] = caller.name.clone();
            }
            _ => {
                let root = changed
                    .functions
                    .iter_mut()
                    .find(|f| f.body_nodes.contains(&changed.nodes[tail].name))
                    .unwrap();
                root.result.as_mut().unwrap().node = root.parameters[0].node.clone();
            }
        }
        assert!(
            yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration())
                .is_err(),
            "{case}"
        );
    }
}

#[test]
#[cfg(unix)]
fn shared_suffix_checked_arithmetic_is_skipped_by_a_real_inner_return() {
    let body = BODY.replace(
        "let count: i64 = bytes_len(saved) / 8;",
        "let count: i64 = (bytes_len(saved) / 8) / mode;",
    );
    let program = compile_body(&body).unwrap();
    let bridge = bridge(&program);
    let library = native::Library::build(&with_step(&body), &program, &bridge.llvm_ir);
    let (mut session, _) = NativeResourceSession::open(
        &program.yir,
        &registration(),
        library.checked_binding(&program.yir, &bridge),
        vec![9; 6],
        3,
    )
    .unwrap();
    session.event(0).unwrap(); // Evaluating the shared suffix would divide by zero.
    assert_eq!(session.scalar_state(), [103]);
    assert_eq!(session.state(), &[9; 6]);
    session.event(2).unwrap();
    assert_eq!(session.scalar_state(), [106]);
    assert_eq!(session.state(), &[10, 103, 109, 9, 9, 9]);
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}

#[test]
#[cfg(unix)]
fn branch_local_bytes_do_not_require_a_fabricated_continuation_owner() {
    let body = r#"
      if mode < 2 {
        store_at(pixels, 0, 7);
        let local: Bytes = copy_bytes(pixels);
        if mode == 0 { return Result.Ok(Canvas { width: width, pixels: move(local) }); }
      }
      store_at(pixels, 0, load_at(pixels, 0) + 10);
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
    let library = native::Library::build(&with_step(body), &program, &bridge.llvm_ir);
    for (mode, value) in [(0, 7), (1, 17), (2, 19)] {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        session.event(mode).unwrap();
        assert_eq!(session.state(), &[value, 9, 9, 9, 9, 9]);
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(library.live_blobs(), 0);
    }
}
