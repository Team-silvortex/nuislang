use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::resource_record_initializers::source;
use super::*;

#[test]
fn record_local_fields_preserve_completed_reads_and_nominal_projections_for_120_events() {
    for (prior, field) in [
        ("let prior: Packet = packet(load_at(pixels, 0), mode % 2 == 0, buffer_len(pixels));", "prior"),
        ("let prior = packet(load_at(pixels, 0), mode % 2 == 0, buffer_len(pixels));", "prior"),
        ("const prior: Alias = packet(load_at(pixels, 0), mode % 2 == 0, buffer_len(pixels));", "prior"),
        ("let prior = envelope(load_at(pixels, 0), mode % 2 == 0, buffer_len(pixels));", "prior.packet"),
        ("let prior = Packet { left: load_at(pixels, 0), value: word(mode % 2 == 0), right: buffer_len(pixels), flags: Flags { valid: mode % 2 == 0 } };", "prior"),
        ("let first = packet(load_at(pixels, 0), mode % 2 == 0, buffer_len(pixels)); let prior: Packet = first;", "prior"),
    ] {
        for declaration in ["let local: Envelope", "let local", "const local: Parcel"] {
            let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); store_at(pixels, 0, mode); {prior} store_at(pixels, 0, 99); {declaration} = Envelope {{ packet: {field}, unused: mode < 3 && bytes_len(saved) == 48 }}; return Result.Ok(Canvas {{ width: local.packet.left - local.packet.right + local.packet.value + word(local.packet.flags.valid) + word(local.unused), pixels: move(saved) }});"))
                .replace("mod cpu Main {", "mod cpu Main { type Parcel = Envelope;");
            let program = compile(&source);
            assert_eq!(program.ast, nuisc::frontend::parse_nuis_ast(&source).unwrap());
            assert_eq!(program.yir, yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap());
            let bridge = bridge(&program);
            if prior.contains("packet(") {
                assert_eq!(bridge.llvm_ir.matches("call [4 x i64] @nuis_fn_packet(").count(), 1);
            }
            let library = native::Library::build(&source, &program, &bridge.llvm_ir);
            let (mut session, _) = NativeResourceSession::open(&program.yir, &registration(), library.checked_binding(&program.yir, &bridge), vec![9; 6], 3).unwrap();
            for mode in (0..6).cycle().take(120) {
                session.event(mode).unwrap();
                assert_eq!(session.scalar_state(), [mode - 6 + if mode % 2 == 0 { 18 } else { 0 } + if mode < 3 { 9 } else { 0 }], "{prior}/{declaration}/{mode}");
                assert_eq!(session.state(), [9; 6]);
                assert_eq!(library.live_blobs(), 0);
            }
            session.close(0).unwrap();
            session.completion_status().unwrap();
        }
    }
}

#[test]
fn record_local_fields_share_one_completed_value_and_continue_through_scalar_locals() {
    let source = source("let saved: Bytes = copy_bytes(pixels); let prior = packet(0, mode % 2 == 0, 0); let local = Pair { first: prior, second: prior, flag: mode < 3 && buffer_len(pixels) == 6 }; let amount: i64 = local.first.value + local.second.value; let valid: bool = local.flag; if valid { let branch: Bytes = copy_bytes(pixels); return Result.Ok(Canvas { width: amount, pixels: move(saved) }); } return Result.Ok(Canvas { width: amount + 1, pixels: move(saved) });")
        .replace("mod cpu Main {", "mod cpu Main { struct Pair { first: Packet, second: Packet, flag: bool }");
    let program = compile(&source);
    let bridge = bridge(&program);
    assert_eq!(
        bridge
            .llvm_ir
            .matches("call [4 x i64] @nuis_fn_packet(")
            .count(),
        1
    );
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
            [if mode % 2 == 0 { 18 } else { 0 } + if mode < 3 { 0 } else { 1 }]
        );
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
}

#[test]
fn record_local_fields_keep_branch_errors_and_failed_allocation_nonpublishing() {
    for returns_error in [false, true] {
        let returned = if returns_error {
            "Result.Err(local.packet.value + 17)"
        } else {
            "Result.Ok(Canvas { width: local.packet.value, pixels: move(saved) })"
        };
        let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); if mode >= 0 {{ let branch: Bytes = copy_bytes(pixels); let prior = packet(0, mode == 0, 0); store_at(pixels, 0, 99); let local = Envelope {{ packet: prior, unused: mode == 0 && bytes_len(saved) == 48 }}; return {returned}; }} return Result.Ok(Canvas {{ width: width + 1, pixels: move(saved) }});"));
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
                if fail || returns_error {
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
                assert_eq!(session.completion_status().is_err(), fail || returns_error);
                assert_eq!(library.live_blobs(), 0);
            }
        }
    }
}

#[test]
fn record_local_fields_without_held_owners_keep_independent_registered_identity() {
    let source = source("let prior = envelope(0, mode == 0, 0); let local = Envelope { packet: prior.packet, unused: mode == 0 && buffer_len(pixels) == 6 }; return Result.Err(local.packet.value + word(local.unused));")
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
    for (mode, code) in [(0, 18), (1, 0)] {
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
    }
}

#[test]
fn record_local_fields_after_an_earlier_lazy_field_preserve_scalar_liveness() {
    let source = source("let saved: Bytes = copy_bytes(pixels); let prior = packet(0, true, 0); let local = Envelope { unused: mode == 0 && buffer_len(pixels) == 6, packet: prior }; return Result.Ok(Canvas { width: local.packet.value + word(local.unused), pixels: move(saved) });");
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
