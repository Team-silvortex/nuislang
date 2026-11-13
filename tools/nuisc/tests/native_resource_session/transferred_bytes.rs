use super::*;
use yir_core::{
    guarded_owned_call,
    native_resource_session::{ResourceSessionRegistration, CHECKED_CONTRACT},
};

const SOURCE: &str = r#"
mod cpu Main {
  enum Result<T, E> { Ok(T), Err(E) }
  struct Canvas { width: i64, pixels: Bytes }
  fn start(pixels: ref Buffer, width: i64) -> Result<Canvas, i64> {
    return Result.Ok(Canvas { width: width, pixels: copy_bytes(pixels) });
  }
  fn step(pixels: ref Buffer, width: i64, mode: i64) -> Result<Canvas, i64> {
    let original: Bytes = copy_bytes(pixels);
    let saved: Bytes = move(original);
    store_at(pixels, 1, width);
    let spare: Bytes = copy_bytes(pixels);
    if mode == 0 {
      let count: i64 = bytes_len(saved) / 8;
      store_at(pixels, 0, 77);
      return Result.Ok(Canvas { width: width + count, pixels: move(saved) });
    }
    if mode == 1 {
      drop_bytes(saved);
      let count: i64 = bytes_len(spare) / 8;
      store_at(pixels, 0, 88);
      return Result.Ok(Canvas { width: width + count, pixels: move(spare) });
    }
    if mode == 2 {
      let count: i64 = (bytes_len(saved) + bytes_len(spare)) / 16;
      drop_bytes(spare);
      return Result.Ok(Canvas { width: width + count, pixels: move(saved) });
    }
    let count: i64 = bytes_len(spare) / 8;
    drop_bytes(saved);
    drop_bytes(spare);
    store_at(pixels, 0, 99);
    return Result.Ok(Canvas { width: width + count, pixels: copy_bytes(pixels) });
  }
  fn stop(pixels: ref Buffer, width: i64, reason: i64) -> Result<Canvas, i64> {
    return Result.Ok(Canvas { width: width, pixels: copy_bytes(pixels) });
  }
  fn main() -> i64 { return 0; }
}
"#;

fn registration() -> ResourceSessionRegistration {
    ResourceSessionRegistration {
        contract: CHECKED_CONTRACT.into(),
        id: "transferred".into(),
        callbacks: ["start", "step", "stop"].map(str::to_owned),
        min_words: 1,
        max_words: 4096,
        state_type: Some("Canvas".into()),
        result_type: Some("Result".into()),
    }
}

fn program() -> nuisc::pipeline::PipelineArtifacts {
    nuisc::pipeline::compile_resource_session_source(SOURCE, &registration(), &Default::default())
        .unwrap()
        .emit_llvm()
        .unwrap()
}

#[test]
#[cfg(unix)]
fn selected_bytes_transfer_preserves_snapshots_and_fallthrough_ownership() {
    let program = program();
    let registration = registration();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration)
            .unwrap();
    assert!(!bridge.llvm_ir.contains("; deferred lowering"));
    let guards = program
        .yir
        .nodes
        .iter()
        .filter(|n| n.op.instruction == "guard_call_owned_struct_return")
        .map(|n| guarded_owned_call::parse(&n.op.args).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        guards
            .iter()
            .map(|g| (g.transfers.len(), g.cleanup.len()))
            .collect::<Vec<_>>(),
        [(1, 1), (1, 1), (2, 0)]
    );
    for guard in &guards {
        let helper = program
            .yir
            .functions
            .iter()
            .find(|f| f.name == guard.callee)
            .unwrap();
        assert!(!program.yir.nodes.iter().any(
            |n| helper.body_nodes.contains(&n.name) && n.op.instruction == "copy_buffer_owned"
        ));
    }
    let library = native::Library::build(SOURCE, &program, &bridge.llvm_ir);
    let (mut session, _) = NativeResourceSession::open(
        &program.yir,
        &registration,
        library.checked_binding(&program.yir, &bridge),
        vec![9; 6],
        3,
    )
    .unwrap();
    let mut expected = vec![9; 6];
    for (index, mode) in [0, 1, 2, 3].into_iter().cycle().take(120).enumerate() {
        if mode == 1 || mode == 3 {
            expected[1] = 3 + 6 * index as i64;
        }
        if mode == 3 {
            expected[0] = 99;
        }
        session.event(mode).unwrap();
        assert_eq!(session.state(), expected);
        assert_eq!(session.scalar_state(), [3 + 6 * (index as i64 + 1)]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}

#[test]
#[cfg(unix)]
fn selected_helper_can_consume_transferred_bytes_before_an_error_return() {
    let source = SOURCE.replace(
        "return Result.Ok(Canvas { width: width + count, pixels: move(saved) });",
        "drop_bytes(saved); return Result.Err(count);",
    );
    let registration = registration();
    let program = nuisc::pipeline::compile_resource_session_source(
        &source,
        &registration,
        &Default::default(),
    )
    .unwrap()
    .emit_llvm()
    .unwrap();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration)
            .unwrap();
    let library = native::Library::build(&source, &program, &bridge.llvm_ir);
    for mode in [0, 2] {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration,
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        session.event(3).unwrap();
        let pixels = session.state().to_vec();
        let scalars = session.scalar_state().to_vec();
        assert!(session.event(mode).unwrap_err().contains("code=6"));
        assert_eq!(session.state(), pixels);
        assert_eq!(session.scalar_state(), scalars);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        assert!(session.completion_status().is_err());
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
fn transfer_partition_signature_and_helper_consumption_are_revalidated() {
    let program = program();
    let registration = registration();
    let roundtrip =
        yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap();
    assert_eq!(roundtrip, program.yir);
    yir_lower_llvm::native_resource_session::emit_registered(&roundtrip, &registration).unwrap();
    for case in 0..8 {
        let mut changed = program.yir.clone();
        let guard_index = changed
            .nodes
            .iter()
            .position(|n| n.op.instruction == "guard_call_owned_struct_return")
            .unwrap();
        let call = guarded_owned_call::parse(&changed.nodes[guard_index].op.args).unwrap();
        let owner = call.transfers[0].clone();
        let helper_name = call.callee.to_owned();
        let input_count = call.inputs.len();
        let marker = 3 + input_count;
        let guard = &mut changed.nodes[guard_index];
        match case {
            0 => {
                guard.op.args.drain(marker..marker + 2);
            }
            1 => {
                guard.op.args[marker + 3] = owner.clone();
            }
            2 => {
                guard.op.args[marker + 1] = guard.op.args[3].clone();
            }
            3 => {
                guard.op.args[marker + 1] = "unknown_owner".into();
            }
            4 => {
                let name = guard.name.clone();
                changed.edges.retain(|e| {
                    !(e.from == owner && e.to == name && e.kind == yir_core::EdgeKind::Lifetime)
                });
            }
            5 => {
                changed
                    .functions
                    .iter_mut()
                    .find(|f| f.name == helper_name)
                    .unwrap()
                    .parameters[input_count]
                    .ownership = yir_core::YirValueOwnership::Borrowed;
            }
            6 => {
                changed
                    .functions
                    .iter_mut()
                    .find(|f| f.name == helper_name)
                    .unwrap()
                    .parameters[input_count]
                    .ty = "i64".into();
            }
            _ => {
                let moved_from = changed
                    .nodes
                    .iter()
                    .find(|n| n.name == owner)
                    .unwrap()
                    .op
                    .args[0]
                    .clone();
                changed.nodes[guard_index].op.args[marker + 1] = moved_from;
            }
        }
        assert!(
            yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration)
                .is_err(),
            "{case}"
        );
    }
    // Keep the ownership trailer valid but make its callee abandon the input.
    let mut leaking = program.yir.clone();
    let call = leaking
        .nodes
        .iter()
        .find(|n| n.op.instruction == "guard_call_owned_struct_return")
        .unwrap();
    let helper = leaking
        .functions
        .iter()
        .find(|f| f.name == call.op.args[1])
        .unwrap();
    let borrowed = helper.parameters[0].node.clone();
    let moved = leaking
        .nodes
        .iter_mut()
        .find(|n| helper.body_nodes.contains(&n.name) && n.op.instruction == "move_owned_bytes")
        .unwrap();
    moved.op.instruction = "copy_buffer_owned".into();
    moved.op.args = vec![borrowed.clone()];
    let name = moved.name.clone();
    for kind in [yir_core::EdgeKind::Dep, yir_core::EdgeKind::Lifetime] {
        leaking.edges.push(yir_core::Edge {
            kind,
            from: borrowed.clone(),
            to: name.clone(),
        });
    }
    let error = yir_lower_llvm::native_resource_session::emit_registered(&leaking, &registration)
        .unwrap_err();
    assert_eq!(error, "checked return leaks resource owners");
}

#[test]
#[cfg(unix)]
fn transferred_bytes_are_released_when_selected_result_allocation_fails() {
    let program = program();
    let registration = registration();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration)
            .unwrap();
    let guards = program
        .yir
        .nodes
        .iter()
        .filter(|n| n.op.instruction == "guard_call_owned_struct_return")
        .map(|n| guarded_owned_call::parse(&n.op.args).unwrap())
        .collect::<Vec<_>>();
    for (mode, guard) in guards.iter().enumerate() {
        let start = bridge
            .llvm_ir
            .find(&format!("define i64 @nuis_fn_{}(", guard.callee))
            .unwrap();
        let end = start + bridge.llvm_ir[start..].find("\n}").unwrap();
        let original = &bridge.llvm_ir[start..end];
        let body = original.replace(
            "@nuis_scheduler_owned_aggregate_alloc_v1(i64 4)",
            "@nuis_scheduler_owned_aggregate_alloc_v1(i64 0)",
        );
        assert_ne!(body, original);
        let faulty = bridge.llvm_ir.replacen(original, &body, 1);
        let library = native::Library::build(SOURCE, &program, &faulty);
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration,
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        session.event(3).unwrap();
        let pixels = session.state().to_vec();
        let scalars = session.scalar_state().to_vec();
        assert!(session.event(mode as i64).unwrap_err().contains("status 2"));
        assert_eq!(session.state(), pixels);
        assert_eq!(session.scalar_state(), scalars);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        assert!(session.completion_status().is_err());
        assert_eq!(library.live_blobs(), 0);
    }
}
