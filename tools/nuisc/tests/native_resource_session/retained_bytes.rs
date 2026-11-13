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
    let spare: Bytes = copy_bytes(pixels);
    let count: i64 = bytes_len(saved) / 8;
    if mode == 0 {
      store_at(pixels, 0, 77);
      return Result.Ok(Canvas { width: width + count, pixels: copy_bytes(pixels) });
    }
    if mode == 1 {
      drop_bytes(saved);
      drop_bytes(spare);
      store_at(pixels, 0, 88);
      return Result.Ok(Canvas { width: width + count, pixels: copy_bytes(pixels) });
    }
    if mode == 13 { return Result.Err(8); }
    let continued: i64 = bytes_len(saved) / 8;
    drop_bytes(spare);
    store_at(pixels, 0, 999);
    return Result.Ok(Canvas { width: width + continued, pixels: move(saved) });
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
        id: "retained".into(),
        callbacks: ["start", "step", "stop"].map(str::to_owned),
        min_words: 1,
        max_words: 4096,
        state_type: Some("Canvas".into()),
        result_type: Some("Result".into()),
    }
}

fn program(source: &str) -> nuisc::pipeline::PipelineArtifacts {
    nuisc::pipeline::compile_resource_session_source(source, &registration(), &Default::default())
        .unwrap()
        .emit_llvm()
        .unwrap()
}

#[test]
#[cfg(unix)]
fn caller_bytes_cleanup_is_taken_only_and_fallthrough_keeps_reads_and_transfer() {
    let program = program(SOURCE);
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
    assert_eq!(guards.len(), 3);
    assert!(guards.iter().all(|g| g.cleanup.len() == 2));
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
    for (index, mode) in [0, 2, 1, 3].into_iter().cycle().take(120).enumerate() {
        if mode == 0 {
            expected[0] = 77;
        }
        if mode == 1 {
            expected[0] = 88;
        }
        session.event(mode).unwrap();
        assert_eq!(session.state(), expected);
        assert_eq!(session.scalar_state(), [3 + 6 * (index as i64 + 1)]);
        assert_eq!(library.live_blobs(), 0);
    }
    let state = session.state().to_vec();
    let scalars = session.scalar_state().to_vec();
    assert!(session.event(13).unwrap_err().contains("code=8"));
    assert_eq!(session.state(), state);
    assert_eq!(session.scalar_state(), scalars);
    assert_eq!(library.live_blobs(), 0);
    session.close(0).unwrap();
    assert!(session.completion_status().is_err());
    assert_eq!(library.live_blobs(), 0);
}

#[test]
fn caller_cleanup_contract_rejects_missing_extra_duplicate_and_consumed_owners() {
    let program = program(SOURCE);
    let registration = registration();
    let roundtrip =
        yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap();
    assert_eq!(roundtrip, program.yir);
    yir_lower_llvm::native_resource_session::emit_registered(&roundtrip, &registration).unwrap();
    for case in 0..7 {
        let mut changed = program.yir.clone();
        let moved = changed
            .nodes
            .iter()
            .find(|n| n.op.instruction == "move_owned_bytes")
            .unwrap()
            .op
            .args[0]
            .clone();
        let guard = changed
            .nodes
            .iter_mut()
            .find(|n| n.op.instruction == "guard_call_owned_struct_return")
            .unwrap();
        let marker = guard
            .op
            .args
            .iter()
            .position(|a| a.starts_with(guarded_owned_call::CLEANUP_PREFIX))
            .unwrap();
        match case {
            0 => guard.op.args.truncate(marker),
            1 => {
                guard.op.args[marker] = "cleanup_bytes_v1=1".into();
                guard.op.args.pop();
            }
            2 => guard.op.args[marker + 2] = guard.op.args[marker + 1].clone(),
            3 => guard.op.args[marker + 1] = guard.op.args[3].clone(),
            4 => guard.op.args[marker + 1] = moved,
            5 => guard.op.args[marker + 1] = "missing".into(),
            _ => {
                let owner = guard.op.args[marker + 1].clone();
                let name = guard.name.clone();
                changed.edges.retain(|e| {
                    !(e.from == owner && e.to == name && e.kind == yir_core::EdgeKind::Lifetime)
                });
            }
        }
        assert!(
            yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration)
                .is_err(),
            "{case}"
        );
    }
    // The exiting edge cannot consume an owned value on the surviving edge.
    let mut leaking = program.yir.clone();
    let drop = leaking
        .nodes
        .iter_mut()
        .find(|n| n.op.instruction == "drop_owned_bytes")
        .unwrap();
    drop.op.instruction = "owned_bytes_len".into();
    let error = yir_lower_llvm::native_resource_session::emit_registered(&leaking, &registration)
        .unwrap_err();
    assert_eq!(error, "checked return leaks resource owners");
}

#[test]
#[cfg(unix)]
fn caller_bytes_are_released_before_selected_helper_allocation_failure() {
    let program = program(SOURCE);
    let registration = registration();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration)
            .unwrap();
    let helpers = program
        .yir
        .nodes
        .iter()
        .filter(|n| n.op.instruction == "guard_call_owned_struct_return")
        .map(|n| guarded_owned_call::parse(&n.op.args).unwrap().callee)
        .collect::<Vec<_>>();
    for (mode, helper, aggregate) in [
        (0, helpers[0], false),
        (0, helpers[0], true),
        (13, helpers[2], true),
    ] {
        let start = bridge
            .llvm_ir
            .find(&format!("define i64 @nuis_fn_{helper}("))
            .unwrap();
        let end = start + bridge.llvm_ir[start..].find("\n}").unwrap();
        let original = &bridge.llvm_ir[start..end];
        let body = original
            .lines()
            .map(|line| {
                if !aggregate {
                    if let Some((register, _)) =
                        line.split_once(" = call ptr @nuis_scheduler_owned_blob_copy_v1(")
                    {
                        return format!("{register} = inttoptr i64 0 to ptr");
                    }
                }
                if aggregate {
                    line.replace(
                        "@nuis_scheduler_owned_aggregate_alloc_v1(i64 4)",
                        "@nuis_scheduler_owned_aggregate_alloc_v1(i64 0)",
                    )
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert_ne!(body, original);
        let fault = bridge.llvm_ir.replacen(original, &body, 1);
        let library = native::Library::build(SOURCE, &program, &fault);
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration,
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        session.event(2).unwrap();
        assert_eq!(session.scalar_state(), [9]);
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(library.live_blobs(), 0);
        assert!(session.event(mode).unwrap_err().contains("status 2"));
        assert_eq!(session.scalar_state(), [9]);
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        assert!(session.completion_status().is_err());
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
fn caller_cleanup_rejects_consumed_capture_double_drop_and_shadowing() {
    for source in [
        SOURCE.replace(
            "pixels: copy_bytes(pixels) });\n    }",
            "pixels: move(saved) });\n    }",
        ),
        SOURCE.replace(
            "drop_bytes(saved);\n      drop_bytes(spare);",
            "drop_bytes(saved);\n      drop_bytes(saved);\n      drop_bytes(spare);",
        ),
        SOURCE.replace(
            "store_at(pixels, 0, 77);",
            "let saved: Bytes = copy_bytes(pixels);",
        ),
    ] {
        if let Ok(compiled) = nuisc::pipeline::compile_resource_session_source(
            &source,
            &registration(),
            &Default::default(),
        ) {
            assert!(yir_lower_llvm::native_resource_session::emit_registered(
                compiled.yir(),
                &registration()
            )
            .is_err());
        }
    }
}
