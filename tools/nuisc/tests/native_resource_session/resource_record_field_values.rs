use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::resource_record_initializers::source;
use super::*;

#[test]
fn record_field_helpers_execute_once_and_preserve_field_order_for_120_events() {
    for (case, value) in [
        "Envelope { packet: packet(load_at(pixels, 0), mode % 2 == 0 && bytes_len(saved) == 48, buffer_len(pixels)), unused: mode < 3 && buffer_len(pixels) == 6 }",
        "Envelope { unused: mode < 3 && buffer_len(pixels) == 6, packet: packet(load_at(pixels, 0), mode % 2 == 0 && bytes_len(saved) == 48, buffer_len(pixels)) }",
        "Envelope { packet: envelope(load_at(pixels, 0), mode % 2 == 0 && bytes_len(saved) == 48, buffer_len(pixels)).packet, unused: mode < 3 && buffer_len(pixels) == 6 }",
        "Envelope { packet: packet(load_at(pixels, 0), true, buffer_len(pixels)), unused: mode < 3 && buffer_len(pixels) == 6 }",
        "Envelope { packet: packet(load_at(pixels, 0), !(mode % 2 == 0 || bytes_len(saved) != 48), buffer_len(pixels)), unused: mode < 3 && buffer_len(pixels) == 6 }",
    ].into_iter().enumerate() {
        for declaration in ["let local: Envelope", "let local", "const local: Parcel"] {
            let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); let __nuis_resource_predicate_left: i64 = mode; store_at(pixels, 0, mode); {declaration} = {value}; store_at(pixels, 0, 99); return Result.Ok(Canvas {{ width: local.packet.value + local.packet.left - local.packet.right + word(local.packet.flags.valid) + word(local.unused), pixels: move(saved) }});"))
                .replace("mod cpu Main {", "mod cpu Main { type Parcel = Envelope;");
            let program = compile(&source); let bridge = bridge(&program);
            assert_eq!(program.ast, nuisc::frontend::parse_nuis_ast(&source).unwrap());
            assert_eq!(program.yir, yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap());
            assert_eq!(bridge.llvm_ir.matches("call [4 x i64] @nuis_fn_packet(").count(), 1, "{case}/{declaration}");
            if case == 2 { assert_eq!(bridge.llvm_ir.matches("call [5 x i64] @nuis_fn_envelope(").count(), 1); }
            assert!(!helper_body(&bridge.llvm_ir, "packet").contains("@nuis_scheduler_owned_"));
            assert!(!bridge.llvm_ir.contains("; deferred lowering"));
            let library = native::Library::build(&source, &program, &bridge.llvm_ir);
            let (mut session, _) = NativeResourceSession::open(&program.yir, &registration(), library.checked_binding(&program.yir, &bridge), vec![9; 6], 3).unwrap();
            for mode in (0..6).cycle().take(120) {
                let flag = match case { 3 => true, 4 => mode % 2 != 0, _ => mode % 2 == 0 };
                session.event(mode).unwrap();
                assert_eq!(session.scalar_state(), [mode - 6 + if flag { 18 } else { 0 } + if mode < 3 { 9 } else { 0 }], "{case}/{declaration}/{mode}");
                assert_eq!(session.state(), [9; 6]); assert_eq!(library.live_blobs(), 0);
            }
            session.close(0).unwrap(); session.completion_status().unwrap(); assert_eq!(library.live_blobs(), 0);
        }
    }
}

#[test]
fn record_field_helpers_compose_multiple_calls_and_later_scalar_continuations() {
    let source = source("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); let local = Pair { first: packet(0, mode % 2 == 0 && bytes_len(saved) == 48, 0), second: envelope(0, mode < 3 && buffer_len(pixels) == 6, 0).packet }; let amount: i64 = local.first.value + local.second.value; let flag: bool = local.first.flags.valid; if flag { let branch: Bytes = copy_bytes(pixels); return Result.Ok(Canvas { width: amount, pixels: move(saved) }); } return Result.Ok(Canvas { width: amount + 1, pixels: move(saved) });")
        .replace("mod cpu Main {", "mod cpu Main { struct Pair { first: Packet, second: Packet }");
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
    for mode in (0..6).cycle().take(120) {
        session.event(mode).unwrap();
        assert_eq!(
            session.scalar_state(),
            [if mode % 2 == 0 { 9 } else { 1 } + if mode < 3 { 9 } else { 0 }]
        );
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
}

#[test]
fn record_field_helpers_keep_early_errors_and_allocation_failures_atomic() {
    for error_return in [false, true] {
        let returned = if error_return {
            "Result.Err(local.packet.value + 17)"
        } else {
            "Result.Ok(Canvas { width: local.packet.value, pixels: move(saved) })"
        };
        let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); if mode >= 0 {{ let branch: Bytes = copy_bytes(pixels); store_at(pixels, 0, 99); let local = Envelope {{ packet: packet(0, mode == 0 && bytes_len(saved) == 48, 0), unused: mode < 3 && buffer_len(pixels) == 6 }}; return {returned}; }} return Result.Ok(Canvas {{ width: width + 1, pixels: move(saved) }});"));
        let program = compile(&source);
        let bridge = bridge(&program);
        for fail in [false, true] {
            let mut llvm = bridge.llvm_ir.clone();
            if fail {
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
            for mode in [0, 1] {
                let (mut session, _) = NativeResourceSession::open(
                    &program.yir,
                    &registration(),
                    library.checked_binding(&program.yir, &bridge),
                    vec![9; 6],
                    3,
                )
                .unwrap();
                let result = session.event(mode);
                if fail || error_return {
                    let error = result.unwrap_err();
                    assert!(
                        error.contains(if fail {
                            "status 2"
                        } else if mode == 0 {
                            "code=26"
                        } else {
                            "code=17"
                        }),
                        "{error}"
                    );
                    if fail {
                        assert!(!error.contains("application error"));
                    }
                    assert_eq!(session.scalar_state(), [3]);
                } else {
                    result.unwrap();
                    assert_eq!(session.scalar_state(), [if mode == 0 { 9 } else { 0 }]);
                }
                assert_eq!(session.state(), [9; 6]);
                assert_eq!(library.live_blobs(), 0);
                session.close(0).unwrap();
                assert_eq!(session.completion_status().is_err(), fail || error_return);
                assert_eq!(library.live_blobs(), 0);
            }
        }
    }
}

#[test]
fn record_field_helpers_without_held_owners_keep_registered_error_identity() {
    let source = source("let local = Envelope { packet: packet(0, mode == 0 && buffer_len(pixels) == 6, 0), unused: mode < 3 && buffer_len(pixels) == 6 }; return Result.Err(local.packet.value);")
        .replace("Packet", "Snapshot").replace("Envelope", "Parcel").replace("Result", "Outcome").replace("Canvas", "Frame");
    let mut registration = registration();
    registration.state_type = Some("Frame".into());
    registration.result_type = Some("Outcome".into());
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
    for (mode, code) in [(0, 9), (1, 0)] {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration,
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        assert!(session
            .event(mode)
            .unwrap_err()
            .contains(&format!("code={code}")));
        assert_eq!(session.scalar_state(), [3]);
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        assert!(session.completion_status().is_err());
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
fn record_field_locals_before_later_resource_logic_use_local_scope_proof() {
    let source = source("let saved: Bytes = copy_bytes(pixels); let prior: Packet = packet(0, true, 0); let local: Envelope = Envelope { packet: prior, unused: mode == 0 && buffer_len(pixels) == 6 }; return Result.Ok(Canvas { width: local.packet.value + word(local.unused), pixels: move(saved) });");
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
    for mode in (0..6).cycle().take(120) {
        session.event(mode).unwrap();
        assert_eq!(session.scalar_state(), [if mode == 0 { 18 } else { 9 }]);
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
}
