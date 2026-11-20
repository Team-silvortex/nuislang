use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::resource_record_initializers::source;
use super::*;

#[test]
fn record_literals_keep_nested_fields_declaration_kinds_and_read_order_for_120_events() {
    for (case, flag) in [
        "mode % 2 == 0 && bytes_len(saved) == 48",
        "!(mode % 2 == 0 || buffer_len(pixels) != 6)",
        "(mode % 2 == 0 && buffer_len(pixels) == 6) ^ (mode < 3)",
        "choose(mode % 2 == 0 && (bytes_len(saved) == 48 || load_at(pixels, 99) > 0)).value",
    ]
    .into_iter()
    .enumerate()
    {
        for declaration in ["let local: Packet", "let local", "const local: Alias"] {
            let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); let __nuis_resource_predicate_left: i64 = mode; store_at(pixels, 0, mode); {declaration} = Packet {{ right: buffer_len(pixels), flags: Flags {{ valid: {flag} }}, left: load_at(pixels, 0), value: word(mode < 3 && buffer_len(pixels) == 6) }}; store_at(pixels, 0, 99); return Result.Ok(Canvas {{ width: local.value + local.left - local.right + word(local.flags.valid), pixels: move(saved) }});"));
            let program = compile(&source);
            assert_eq!(
                program.ast,
                nuisc::frontend::parse_nuis_ast(&source).unwrap()
            );
            assert_eq!(
                program.yir,
                yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir))
                    .unwrap()
            );
            let bridge = bridge(&program);
            assert!(!bridge.llvm_ir.contains("; deferred lowering"));
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
                let even = mode % 2 == 0;
                let flag = match case {
                    1 => !even,
                    2 => even ^ (mode < 3),
                    _ => even,
                };
                session.event(mode).unwrap();
                assert_eq!(
                    session.scalar_state(),
                    [mode - 6 + if mode < 3 { 9 } else { 0 } + if flag { 9 } else { 0 }],
                    "{case}/{declaration}/{mode}"
                );
                assert_eq!(session.state(), [9; 6]);
                assert_eq!(library.live_blobs(), 0);
            }
            session.close(0).unwrap();
            session.completion_status().unwrap();
            assert_eq!(library.live_blobs(), 0);
        }
    }
}

#[test]
fn record_literals_keep_early_returns_and_projected_scalar_continuations() {
    for early in [false, true] {
        let declaration = "let local = Flags { valid: mode % 2 == 0 && buffer_len(pixels) == 6 };";
        let body = if early {
            format!("if mode >= 0 {{ let branch: Bytes = copy_bytes(pixels); {declaration} return Result.Ok(Canvas {{ width: word(local.valid), pixels: move(saved) }}); }} return Result.Ok(Canvas {{ width: width + 1, pixels: move(saved) }});")
        } else {
            format!("{declaration} let valid: bool = local.valid; if valid {{ return Result.Ok(Canvas {{ width: 9, pixels: move(saved) }}); }} return Result.Ok(Canvas {{ width: 1, pixels: move(saved) }});")
        };
        let source = source(&format!(
            "let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); {body}"
        ));
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
        for mode in [0, 1, -1].into_iter().cycle().take(120) {
            width = if early && mode < 0 {
                width + 1
            } else if mode % 2 == 0 {
                9
            } else if early {
                0
            } else {
                1
            };
            session.event(mode).unwrap();
            assert_eq!(session.scalar_state(), [width]);
            assert_eq!(session.state(), [9; 6]);
            assert_eq!(library.live_blobs(), 0);
        }
        session.close(0).unwrap();
        session.completion_status().unwrap();
    }
}

#[test]
fn record_literals_keep_error_and_allocation_failure_state_atomic() {
    for error_return in [false, true] {
        let returned = if error_return {
            "Result.Err(word(local.valid) + 17)"
        } else {
            "Result.Ok(Canvas { width: word(local.valid), pixels: move(saved) })"
        };
        let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); store_at(pixels, 0, 99); let local: Flags = Flags {{ valid: mode == 0 && bytes_len(saved) == 48 }}; return {returned};"));
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
                        assert!(!error.contains("application error"), "{error}");
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
fn record_literals_without_held_owners_keep_independent_registered_names() {
    let source = source("let local: Flags = Flags { valid: mode == 0 && buffer_len(pixels) == 6 }; return Result.Err(word(local.valid));")
        .replace("Flags", "Bits").replace("Result", "Outcome").replace("Canvas", "Frame");
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
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(session.scalar_state(), [3]);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        assert!(session.completion_status().is_err());
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
fn record_literals_with_record_valued_helper_fields_stage_scalar_snapshots() {
    let source = source("let saved: Bytes = copy_bytes(pixels); let local: Envelope = Envelope { packet: packet(0, mode == 0 && buffer_len(pixels) == 6, 0), unused: false }; return Result.Ok(Canvas { width: local.packet.value, pixels: move(saved) });");
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
    for mode in (0..2).cycle().take(120) {
        session.event(mode).unwrap();
        assert_eq!(session.scalar_state(), [if mode == 0 { 9 } else { 0 }]);
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
}
