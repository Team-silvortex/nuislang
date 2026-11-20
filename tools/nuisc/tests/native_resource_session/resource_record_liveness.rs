use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::resource_record_initializers::source;
use super::*;

#[test]
fn record_liveness_preserves_late_fields_completed_reads_and_nested_paths_for_120_events() {
    for (prior, field) in [
        ("let prior: Packet = packet(load_at(pixels, 0), mode % 2 == 0, buffer_len(pixels));", "prior"),
        ("let prior = packet(load_at(pixels, 0), mode % 2 == 0, buffer_len(pixels));", "prior"),
        ("const prior: Alias = packet(load_at(pixels, 0), mode % 2 == 0, buffer_len(pixels));", "prior"),
        ("let prior = envelope(load_at(pixels, 0), mode % 2 == 0, buffer_len(pixels));", "prior.packet"),
        ("let prior = Packet { left: load_at(pixels, 0), value: word(mode % 2 == 0), right: buffer_len(pixels), flags: Flags { valid: mode % 2 == 0 } };", "prior"),
        ("let first = packet(load_at(pixels, 0), mode % 2 == 0, buffer_len(pixels)); let prior: Packet = first;", "prior"),
    ] {
        for declaration in ["let local: Envelope", "let local", "const local: Parcel"] {
            let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); store_at(pixels, 0, mode); {prior} store_at(pixels, 0, 99); {declaration} = Envelope {{ unused: mode < 3 && bytes_len(saved) == 48, packet: {field} }}; return Result.Ok(Canvas {{ width: local.packet.left - local.packet.right + local.packet.value + word(local.packet.flags.valid) + word(local.unused), pixels: move(saved) }});"))
                .replace("mod cpu Main {", "mod cpu Main { type Parcel = Envelope;");
            let program = compile(&source);
            assert_eq!(program.ast, nuisc::frontend::parse_nuis_ast(&source).unwrap());
            assert_eq!(program.yir, yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap());
            let bridge = bridge(&program);
            if prior.contains("packet(") { assert_eq!(bridge.llvm_ir.matches("call [4 x i64] @nuis_fn_packet(").count(), 1); }
            let library = native::Library::build(&source, &program, &bridge.llvm_ir);
            let (mut session, _) = NativeResourceSession::open(&program.yir, &registration(), library.checked_binding(&program.yir, &bridge), vec![9; 6], 3).unwrap();
            for mode in (0..6).cycle().take(120) {
                session.event(mode).unwrap();
                assert_eq!(session.scalar_state(), [mode - 6 + if mode % 2 == 0 { 18 } else { 0 } + if mode < 3 { 9 } else { 0 }], "{prior}/{declaration}/{mode}");
                assert_eq!(session.state(), [9; 6]); assert_eq!(library.live_blobs(), 0);
            }
            session.close(0).unwrap(); session.completion_status().unwrap();
        }
    }
}

#[test]
fn record_liveness_shares_overlapping_paths_across_nested_and_multiple_lazy_fields() {
    let source = source("let saved: Bytes = copy_bytes(pixels); let prior = envelope(0, mode % 2 == 0, 0); let local = Both { gate: mode < 3 && bytes_len(saved) == 48, outer: Wrapped { flag: mode % 2 == 0 && buffer_len(pixels) == 6, value: prior }, inner: prior.packet }; let amount: i64 = local.outer.value.packet.value + local.inner.value + word(local.outer.flag); let valid: bool = local.gate; if valid { let branch: Bytes = copy_bytes(pixels); return Result.Ok(Canvas { width: amount, pixels: move(saved) }); } return Result.Ok(Canvas { width: amount + 1, pixels: move(saved) });")
        .replace("mod cpu Main {", "mod cpu Main { struct Wrapped { flag: bool, value: Envelope } struct Both { gate: bool, outer: Wrapped, inner: Packet }");
    let program = compile(&source);
    let bridge = bridge(&program);
    assert_eq!(
        bridge
            .llvm_ir
            .matches("call [5 x i64] @nuis_fn_envelope(")
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
            [if mode % 2 == 0 { 27 } else { 0 } + if mode < 3 { 0 } else { 1 }]
        );
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
}

#[test]
fn record_liveness_keeps_branch_errors_and_failed_allocation_atomic() {
    for returns_error in [false, true] {
        let returned = if returns_error {
            "Result.Err(local.packet.value + 17)"
        } else {
            "Result.Ok(Canvas { width: local.packet.value, pixels: move(saved) })"
        };
        let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); if mode >= 0 {{ let branch: Bytes = copy_bytes(pixels); let prior = packet(0, mode == 0, 0); store_at(pixels, 0, 99); let local = Envelope {{ unused: prior.flags.valid && bytes_len(saved) == 48, packet: prior }}; return {returned}; }} return Result.Ok(Canvas {{ width: width + 1, pixels: move(saved) }});"))
            .replace("width: local.packet.value,", "width: word(local.unused),")
            .replace("Result.Err(local.packet.value + 17)", "Result.Err(word(local.unused) + 17)");
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
fn record_liveness_without_held_owners_keeps_registered_identity() {
    let source = source("let prior = envelope(0, mode == 0, 0); let local = Envelope { unused: prior.packet.flags.valid && buffer_len(pixels) == 6, packet: prior.packet }; return Result.Err(local.packet.value + word(local.unused));")
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
fn record_liveness_reuses_repeated_paths_without_widening_guard_input_bounds() {
    for count in [1, 17, 29, 30] {
        let fields = (0..count)
            .map(|i| format!("f{i}: i64"))
            .collect::<Vec<_>>()
            .join(", ");
        let values = (0..count)
            .map(|i| format!("f{i}: width + {i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let prior = Wide {{ {values} }}; let local = Pair {{ gate: mode == 0 && buffer_len(pixels) == 6, first: prior, second: prior }}; return Result.Ok(Canvas {{ width: local.first.f0 + local.second.f{} + word(local.gate), pixels: move(saved) }});", count - 1))
            .replace("mod cpu Main {", &format!("mod cpu Main {{ struct Wide {{ {fields} }} struct Pair {{ gate: bool, first: Wide, second: Wide }}"));
        let result = nuisc::pipeline::compile_resource_session_source(
            &source,
            &registration(),
            &Default::default(),
        )
        .and_then(|program| program.emit_llvm())
        .and_then(|program| {
            yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration())
        });
        if count <= 29 {
            result.unwrap_or_else(|error| panic!("{count}: {error}"));
        } else {
            let error = result.unwrap_err();
            assert!(error.contains("signature metadata"), "{error}");
        }
    }
}

#[test]
fn record_liveness_admits_local_scalar_projection_inside_resource_predicates() {
    let source = source("let saved: Bytes = copy_bytes(pixels); let prior = packet(0, true, 0); let local = Envelope { unused: prior.flags.valid && bytes_len(saved) == 48, packet: prior }; return Result.Ok(Canvas { width: local.packet.value + word(local.unused), pixels: move(saved) });");
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
        assert_eq!(session.scalar_state(), [18]);
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
}
