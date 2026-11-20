use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::resource_record_initializers::source;
use super::*;

#[test]
fn local_record_scalar_initializers_preserve_truth_tables_reads_and_declarations_for_120_events() {
    for (case, kind, expression) in [
        (0, "bool", "prior.packet.flags.valid && bytes_len(saved) == 48"),
        (1, "bool", "!(prior.packet.flags.valid || bytes_len(saved) == 0)"),
        (2, "bool", "(prior.packet.flags.valid && bytes_len(saved) == 48) ^ (prior.packet.left < 3)"),
        (3, "bool", "choose(prior.packet.flags.valid && load_at(pixels, prior.packet.right - 6) == 99).value"),
        (4, "bool", "prior.packet.flags.valid && (bytes_len(saved) == prior.packet.right * 8 || load_at(pixels, 99) > 0)"),
        (5, "i64", "word(prior.packet.flags.valid && buffer_len(pixels) == prior.packet.right)"),
        (6, "i64", "prior.packet.left + word(prior.packet.flags.valid && bytes_len(saved) == prior.packet.right * 8) - prior.packet.right"),
        (7, "i64", "packet(prior.packet.left, prior.packet.flags.valid && bytes_len(saved) == 48, prior.packet.right).value + load_at(pixels, 0)"),
        (8, "i64", "load_at(pixels, word(prior.packet.flags.valid && bytes_len(saved) == 48) / 9)"),
    ] {
        let alias = if kind == "bool" { "Flag" } else { "Number" };
        for declaration in [format!("let value: {kind}"), "let value".into(), format!("const value: {alias}")] {
            let returned = if kind == "bool" { "word(value)" } else { "value" };
            let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); store_at(pixels, 0, mode); let prior = envelope(load_at(pixels, 0), mode % 2 == 0, buffer_len(pixels)); store_at(pixels, 0, 99); {declaration} = {expression}; return Result.Ok(Canvas {{ width: {returned}, pixels: move(saved) }});"))
                .replace("mod cpu Main {", "mod cpu Main { type Flag = bool; type Number = i64;");
            let program = compile(&source);
            assert_eq!(program.ast, nuisc::frontend::parse_nuis_ast(&source).unwrap());
            assert_eq!(program.yir, yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap());
            let bridge = bridge(&program);
            let library = native::Library::build(&source, &program, &bridge.llvm_ir);
            let (mut session, _) = NativeResourceSession::open(&program.yir, &registration(), library.checked_binding(&program.yir, &bridge), vec![9; 6], 3).unwrap();
            for mode in (0..6).cycle().take(120) {
                let even = mode % 2 == 0;
                let flag = match case { 1 => !even, 2 => even ^ (mode < 3), _ => even };
                let expected = match case { 6 => mode + if even { 9 } else { 0 } - 6, 7 => if even { 108 } else { 99 }, 8 => if even { 9 } else { 99 }, _ => if flag { 9 } else { 0 } };
                session.event(mode).unwrap();
                assert_eq!(session.scalar_state(), [expected], "{case}/{declaration}/{mode}");
                assert_eq!(session.state(), [9; 6]); assert_eq!(library.live_blobs(), 0);
            }
            session.close(0).unwrap(); session.completion_status().unwrap();
        }
    }
}

#[test]
fn local_record_scalar_initializers_keep_branch_local_shadows_and_outer_fallthrough() {
    for declaration in ["let valid: bool", "let valid", "const valid: Flag"] {
        let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let valid: bool = mode >= 0; if mode >= 0 {{ let branch: Bytes = copy_bytes(pixels); let prior = packet(0, mode % 2 == 0, 0); {declaration} = (prior.flags.valid && bytes_len(saved) == 48) ^ valid; return Result.Ok(Canvas {{ width: word(valid), pixels: move(saved) }}); }} return Result.Ok(Canvas {{ width: word(valid), pixels: move(saved) }});"))
            .replace("mod cpu Main {", "mod cpu Main { type Flag = bool;");
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
        for mode in [-1, 0, 1, 2].into_iter().cycle().take(120) {
            session.event(mode).unwrap();
            assert_eq!(session.scalar_state(), [if mode == 1 { 9 } else { 0 }]);
            assert_eq!(session.state(), [9; 6]);
            assert_eq!(library.live_blobs(), 0);
        }
        session.close(0).unwrap();
        session.completion_status().unwrap();
    }
}

#[test]
fn local_record_scalar_initializers_keep_errors_and_failed_allocations_atomic() {
    for error_return in [false, true] {
        let returned = if error_return {
            "Result.Err(value + 17)"
        } else {
            "Result.Ok(Canvas { width: value, pixels: move(saved) })"
        };
        let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); if mode >= 0 {{ let branch: Bytes = copy_bytes(pixels); let prior = packet(0, mode == 0, 0); store_at(pixels, 0, 99); let value: i64 = word(prior.flags.valid && bytes_len(saved) == 48); return {returned}; }} return Result.Ok(Canvas {{ width: width + 1, pixels: move(saved) }});"));
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
fn local_record_scalar_initializers_preserve_independent_borrowed_only_error_identity() {
    let source = source("let prior = envelope(0, mode == 0, 0); let code: i64 = word(prior.packet.flags.valid && buffer_len(pixels) == 6); return Result.Err(code);")
        .replace("Packet", "Snapshot").replace("Flags", "Bits").replace("Canvas", "Frame").replace("Result", "Outcome");
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
    }
}

#[test]
fn local_record_scalar_initializers_share_paths_without_widening_capture_bounds() {
    for count in [1, 17, 29, 30] {
        let fields = (0..count)
            .map(|i| format!("f{i}: i64"))
            .collect::<Vec<_>>()
            .join(", ");
        let values = (0..count)
            .map(|i| format!("f{i}: width + {i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let args = (0..count)
            .map(|i| format!("prior.f{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let prior = Wide {{ {values} }}; let value: i64 = tally({args}, mode == 0 && buffer_len(pixels) == 6, prior.f0); return Result.Ok(Canvas {{ width: value, pixels: move(saved) }});"))
            .replace("mod cpu Main {", &format!("mod cpu Main {{ struct Wide {{ {fields} }} fn tally({fields}, flag: bool, again: i64) -> i64 {{ return f0 + f{} + word(flag) + again; }}", count - 1));
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
fn local_record_scalar_initializers_also_admit_direct_resource_conditions() {
    let source = source("let saved: Bytes = copy_bytes(pixels); let prior = packet(0, mode == 0, 0); if prior.flags.valid && bytes_len(saved) == 48 { return Result.Ok(Canvas { width: width + 1, pixels: move(saved) }); } return Result.Ok(Canvas { width: width, pixels: move(saved) });");
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
    for mode in (0..6).cycle().take(120) {
        width += i64::from(mode == 0);
        session.event(mode).unwrap();
        assert_eq!(session.scalar_state(), [width]);
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
}
