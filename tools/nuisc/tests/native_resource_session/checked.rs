use super::*;
use yir_core::native_resource_session::{ResourceSessionRegistration, CHECKED_CONTRACT};

const SOURCE: &str = include_str!(
    "../../../../examples/projects/domains/native_checked_canvas_session_demo/main.ns"
);
fn registration() -> ResourceSessionRegistration {
    ResourceSessionRegistration {
        contract: CHECKED_CONTRACT.to_owned(),
        id: "canvas".to_owned(),
        callbacks: ["canvas_open", "canvas_event", "canvas_close"].map(str::to_owned),
        min_words: 1,
        max_words: 4096,
        state_type: Some("Canvas".to_owned()),
        result_type: Some("Result".to_owned()),
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
fn checked_dimensions_return_nuis_errors_before_any_state_is_published() {
    let program = program(SOURCE);
    let registration = registration();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration)
            .unwrap();
    assert!(!bridge.llvm_ir.contains("; deferred lowering"));
    let library = native::Library::build(SOURCE, &program, &bridge.llvm_ir);
    for (width, code) in [
        (0, 1),
        (-1, 1),
        (i64::MIN, 1),
        (7, 2),
        (i64::MAX, 2),
        (4, 3),
        (5, 3),
    ] {
        let error = NativeResourceSession::open(
            &program.yir,
            &registration,
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            width,
        )
        .err()
        .unwrap();
        assert_eq!(
            error,
            format!("native resource application error (role=open, code={code})")
        );
        assert_eq!(library.live_blobs(), 0);
    }
    for (width, height) in [(1, 6), (2, 3), (3, 2), (6, 1)] {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration,
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            width,
        )
        .unwrap();
        assert_eq!(session.scalar_state(), [width, height, 0, 0]);
        for i in 0..80 {
            session.event(i).unwrap();
        }
        assert_eq!(session.scalar_state(), [width, height, 80, 2]);
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
#[cfg(unix)]
fn checked_event_and_close_errors_preserve_both_halves_and_remain_failed() {
    let program = program(SOURCE);
    let registration = registration();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration)
            .unwrap();
    // Even a write before Err only reaches the call-local borrowed snapshot.
    let dirty = SOURCE.replace("if color < 0", "store_at(pixels, 0, 123); if color < 0");
    let dirty_program = self::program(&dirty);
    let dirty_bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&dirty_program.yir, &registration)
            .unwrap();
    let library = native::Library::build(SOURCE, &program, &bridge.llvm_ir);
    let dirty_library = native::Library::build(&dirty, &dirty_program, &dirty_bridge.llvm_ir);
    for (p, b, lib) in [
        (&program, &bridge, &library),
        (&dirty_program, &dirty_bridge, &dirty_library),
    ] {
        let (mut session, _) = NativeResourceSession::open(
            &p.yir,
            &registration,
            lib.checked_binding(&p.yir, b),
            vec![9; 6],
            3,
        )
        .unwrap();
        assert!(session
            .event(-1)
            .unwrap_err()
            .contains("role=event, code=4"));
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(session.scalar_state(), [3, 2, 0, 0]);
        assert!(session.event(8).is_err());
        session.close(0).unwrap();
        assert!(session
            .completion_status()
            .unwrap_err()
            .contains("role=event, code=4"));
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(session.close(0).unwrap(), None);
        assert_eq!(lib.live_blobs(), 0);
    }
    let (mut session, _) = NativeResourceSession::open(
        &program.yir,
        &registration,
        library.checked_binding(&program.yir, &bridge),
        vec![9; 6],
        3,
    )
    .unwrap();
    session.event(77).unwrap();
    assert!(session
        .close(-1)
        .unwrap_err()
        .contains("role=close, code=7"));
    assert!(session.close(0).unwrap_err().contains("role=close, code=7"));
    assert_eq!(session.state(), [77, 9, 9, 9, 9, 9]);
    assert_eq!(session.scalar_state(), [3, 2, 1, 1]);
    assert!(session.completion_status().is_err());
    assert_eq!(library.live_blobs(), 0);
}

#[test]
#[cfg(unix)]
fn checked_error_channel_keeps_full_i64_and_never_publishes_outputs() {
    use yir_core::native_resource_session::CHECKED_CAPABILITY_TAG;
    let source = SOURCE
        .replace("return Result.Err(1)", "return Result.Err(width)")
        .replace("if width <= 0", "if width != 3");
    let program = program(&source);
    let registration = registration();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration)
            .unwrap();
    let library = native::Library::build(&source, &program, &bridge.llvm_ir);
    let callback = library.checked_callbacks(&bridge)[0];
    for width in [0, -1, i64::MIN, i64::MAX] {
        let mut pixels = [9; 6];
        let mut output = [71; 6];
        let mut scalars = [81; 4];
        let mut error = 91;
        let status = unsafe {
            callback(
                CHECKED_CAPABILITY_TAG,
                pixels.as_mut_ptr(),
                6,
                scalars.as_ptr(),
                0,
                width,
                output.as_mut_ptr(),
                6,
                scalars.as_mut_ptr(),
                4,
                &mut error,
            )
        };
        assert_eq!(status, 4);
        assert_eq!(error, width);
        assert_eq!(pixels, [9; 6]);
        assert_eq!(output, [71; 6]);
        assert_eq!(scalars, [81; 4]);
        assert_eq!(library.live_blobs(), 0);
    }
    for bad in [0, 1, 2, 3] {
        let mut pixels = [9; 6];
        let mut output = [71; 6];
        let mut scalars = [81; 4];
        let mut error = 91;
        let status = unsafe {
            callback(
                if bad == 0 { 0 } else { CHECKED_CAPABILITY_TAG },
                pixels.as_mut_ptr(),
                6,
                scalars.as_ptr(),
                if bad == 1 { 1 } else { 0 },
                0,
                output.as_mut_ptr(),
                6,
                scalars.as_mut_ptr(),
                if bad == 2 { 3 } else { 4 },
                if bad == 3 {
                    std::ptr::null_mut()
                } else {
                    &mut error
                },
            )
        };
        assert_eq!(status, 1);
        assert_eq!(error, 91);
        assert_eq!(output, [71; 6]);
        assert_eq!(scalars, [81; 4]);
    }
}

#[test]
fn checked_layout_and_guard_admission_reject_drift_and_live_owner_exits() {
    use yir_core::native_resource_session::MixedResourceLayout;
    let program = program(SOURCE);
    let registration = registration();
    let layout = registration.mixed_layout(&program.yir).unwrap().unwrap();
    for encoded in [
        layout
            .encoded
            .replace("Result.Err{value:i64}", "Result.Err{value:Bytes}"),
        layout
            .encoded
            .replace("Result.Ok{value:Canvas", "Result.Ok{other:Canvas"),
        layout.encoded.replace("tag:i64", "tag:bool"),
        layout.encoded.replace("Result.Err", "Result.Other"),
        layout.encoded.replace("width:i64", "width:f64"),
    ] {
        assert!(MixedResourceLayout::parse_outcome(&encoded, "Canvas", "Result").is_err());
    }
    let mut changed = program.yir.clone();
    let guard = changed
        .nodes
        .iter_mut()
        .find(|n| n.op.instruction == "guard_return")
        .unwrap();
    guard.op.args[2] = layout.encoded.replace("height:i64", "depth:i64");
    assert!(
        yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration).is_err()
    );
    let source = SOURCE
        .replace(
            "if width <= 0",
            "let spare: Bytes = copy_bytes(pixels); if width <= 0",
        )
        .replace(
            "if width > buffer_len(pixels)",
            "drop_bytes(spare); if width > buffer_len(pixels)",
        );
    let mut retained = nuisc::pipeline::compile_resource_session_source(
        &source,
        &registration,
        &Default::default(),
    )
    .unwrap()
    .emit_llvm()
    .unwrap()
    .yir;
    yir_lower_llvm::native_resource_session::emit_registered(&retained, &registration).unwrap();
    let guard = retained
        .nodes
        .iter_mut()
        .find(|n| n.op.args.iter().any(|a| a.starts_with("cleanup_bytes_v1=")))
        .unwrap();
    let marker = guard
        .op
        .args
        .iter()
        .position(|a| a.starts_with("cleanup_bytes_v1="))
        .unwrap();
    guard.op.args.truncate(marker);
    assert!(
        yir_lower_llvm::native_resource_session::emit_registered(&retained, &registration).is_err()
    );
    let mut wrong = registration.clone();
    wrong.result_type = None;
    assert!(wrong.validate().is_err());
    wrong.result_type = Some("Other".to_owned());
    assert!(wrong.bind(&program.yir).is_err());
    let mut registered = program.yir.clone();
    registered.resource_sessions.push(registration.clone());
    assert_eq!(
        yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&registered)).unwrap(),
        registered
    );
}

#[test]
#[cfg(unix)]
fn checked_allocation_failure_is_not_absence_or_an_application_error() {
    let program = program(SOURCE);
    let registration = registration();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration)
            .unwrap();
    let no_bytes = bridge
        .llvm_ir
        .lines()
        .map(|line| {
            if let Some((name, _)) =
                line.split_once(" = call ptr @nuis_scheduler_owned_blob_copy_v1(")
            {
                format!("{name} = inttoptr i64 0 to ptr")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let no_aggregate = bridge.llvm_ir.replace(
        "@nuis_scheduler_owned_aggregate_alloc_v1(i64 7)",
        "@nuis_scheduler_owned_aggregate_alloc_v1(i64 0)",
    );
    for (ir, errors_survive) in [(no_bytes, true), (no_aggregate, false)] {
        assert_ne!(ir, bridge.llvm_ir);
        let library = native::Library::build(SOURCE, &program, &ir);
        for width in [0, 3] {
            let error = NativeResourceSession::open(
                &program.yir,
                &registration,
                library.checked_binding(&program.yir, &bridge),
                vec![9; 6],
                width,
            )
            .err()
            .unwrap();
            if errors_survive && width == 0 {
                assert!(error.contains("role=open, code=1"), "{error}");
            } else {
                assert!(error.contains("status 2"), "{error}");
            }
            assert_eq!(library.live_blobs(), 0);
        }
    }
}

#[test]
#[cfg(unix)]
fn checked_result_identity_and_slots_are_not_canvas_specific() {
    let source = SOURCE
        .replace("Result", "Reply")
        .replace("Canvas", "Surface")
        .replace("Ok(T), Err(E)", "Err(E), Ok(T)")
        .replace("    pixels: Bytes,\n", "")
        .replace("struct Surface {", "struct Surface {\n    pixels: Bytes,");
    let mut registration = registration();
    registration.state_type = Some("Surface".to_owned());
    registration.result_type = Some("Reply".to_owned());
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
    let error = NativeResourceSession::open(
        &program.yir,
        &registration,
        library.checked_binding(&program.yir, &bridge),
        vec![9; 6],
        0,
    )
    .err()
    .unwrap();
    assert!(error.contains("role=open, code=1"), "{error}");
    let (mut session, _) = NativeResourceSession::open(
        &program.yir,
        &registration,
        library.checked_binding(&program.yir, &bridge),
        vec![9; 6],
        2,
    )
    .unwrap();
    session.event(101).unwrap();
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(session.scalar_state(), [2, 3, 1, 1]);
    assert_eq!(session.state(), [101, 9, 9, 9, 9, 9]);
    assert_eq!(library.live_blobs(), 0);
}
