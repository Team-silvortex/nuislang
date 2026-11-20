use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::*;

pub(super) fn source(body: &str) -> String {
    super::boolean_short_circuit::helpers(body).replace("mod cpu Main {", r#"mod cpu Main {
        struct Flags { valid: bool }
        struct Packet { value: i64, left: i64, right: i64, flags: Flags }
        struct Envelope { packet: Packet, unused: bool }
        type Alias = Packet;
        fn word(flag: bool) -> i64 { if flag { return 9; } return 0; }
        fn packet(left: i64, flag: bool, right: i64) -> Packet {
            return Packet { value: word(flag), left: left, right: right, flags: Flags { valid: flag } };
        }
        fn envelope(left: i64, flag: bool, right: i64) -> Envelope {
            return Envelope { packet: packet(left, flag, right), unused: !flag };
        }
    "#)
}

#[test]
fn record_local_initializers_preserve_nested_values_argument_order_and_owners_for_120_events() {
    for (case, value) in [
        "packet(load_at(pixels, 0), mode % 2 == 0 && bytes_len(saved) == 48, buffer_len(pixels))",
        "packet(load_at(pixels, 0), !(mode % 2 == 0 || bytes_len(saved) == 0), buffer_len(pixels))",
        "packet(load_at(pixels, 0), (mode % 2 == 0 && buffer_len(pixels) == 6) ^ (mode < 3), buffer_len(pixels))",
        "envelope(load_at(pixels, 0), mode % 2 == 0 && bytes_len(saved) == 48, buffer_len(pixels)).packet",
        "packet(load_at(pixels, 0), choose(mode % 2 == 0 && bytes_len(saved) == 48).value, buffer_len(pixels))",
        "packet(load_at(pixels, 0), mode % 2 == 0 && bytes_len(saved) == 48, word(mode < 3 && buffer_len(pixels) == 6))",
    ].into_iter().enumerate() {
        for declaration in ["let local: Packet", "let local", "const local: Alias"] {
            let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); let __nuis_resource_predicate_left: i64 = mode; store_at(pixels, 0, mode); {declaration} = {value}; store_at(pixels, 0, 99); return Result.Ok(Canvas {{ width: local.value + local.left - local.right + word(local.flags.valid), pixels: move(saved) }});"));
            let program = compile(&source);
            assert_eq!(program.ast, nuisc::frontend::parse_nuis_ast(&source).unwrap());
            assert_eq!(program.yir, yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap());
            let bridge = bridge(&program);
            assert!(!bridge.llvm_ir.contains("; deferred lowering"));
            let body = helper_body(&bridge.llvm_ir, "packet");
            assert!(body.contains("define [4 x i64]"), "{body}");
            assert!(!body.contains("@nuis_scheduler_owned_"), "{body}");
            let library = native::Library::build(&source, &program, &bridge.llvm_ir);
            let (mut session, _) = NativeResourceSession::open(&program.yir, &registration(), library.checked_binding(&program.yir, &bridge), vec![9; 6], 3).unwrap();
            for mode in (0..6).cycle().take(120) {
                let even = mode % 2 == 0;
                let flag = match case { 1 => !even, 2 => even ^ (mode < 3), _ => even };
                let right = if case == 5 { if mode < 3 { 9 } else { 0 } } else { 6 };
                session.event(mode).unwrap();
                assert_eq!(session.scalar_state(), [mode - right + if flag { 18 } else { 0 }], "{case}/{declaration}/{mode}");
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
fn record_local_initializers_keep_early_returns_and_scalar_projected_continuations() {
    for early in [false, true] {
        let declaration = "let local = packet(0, mode % 2 == 0 && buffer_len(pixels) == 6, 0);";
        let body = if early {
            format!("if mode >= 0 {{ let branch: Bytes = copy_bytes(pixels); {declaration} return Result.Ok(Canvas {{ width: local.value, pixels: move(saved) }}); }} return Result.Ok(Canvas {{ width: width + 1, pixels: move(saved) }});")
        } else {
            format!("{declaration} let value: i64 = local.value; let valid: bool = local.flags.valid; if valid {{ return Result.Ok(Canvas {{ width: value, pixels: move(saved) }}); }} return Result.Ok(Canvas {{ width: value + 1, pixels: move(saved) }});")
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
fn record_local_initializers_keep_application_errors_and_allocation_failures_atomic() {
    for error_return in [false, true] {
        let returned = if error_return {
            "Result.Err(local.value + 17)"
        } else {
            "Result.Ok(Canvas { width: local.value, pixels: move(saved) })"
        };
        let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); store_at(pixels, 0, 99); let local: Packet = packet(0, mode == 0 && bytes_len(saved) == 48, 0); return {returned};"));
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
fn record_local_initializers_do_not_widen_native_value_record_slots() {
    for count in [64, 65] {
        let fields = (0..count)
            .map(|i| format!("f{i}: i64"))
            .collect::<Vec<_>>()
            .join(", ");
        let values = (0..count)
            .map(|i| format!("f{i}: word(flag)"))
            .collect::<Vec<_>>()
            .join(", ");
        let source = source("let saved: Bytes = copy_bytes(pixels); let local: Wide = wide(mode == 0 && buffer_len(pixels) == 6); return Result.Ok(Canvas { width: local.f0, pixels: move(saved) });")
            .replace("mod cpu Main {", &format!("mod cpu Main {{ struct Wide {{ {fields} }} fn wide(flag: bool) -> Wide {{ return Wide {{ {values} }}; }}"));
        let program = compile(&source);
        let result =
            yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration());
        if count == 64 {
            result.unwrap();
        } else {
            assert!(result.is_err());
        }
    }
}

#[test]
fn record_local_initializers_without_held_owners_keep_registered_error_identity() {
    let source = source("let local: Packet = packet(0, mode == 0 && buffer_len(pixels) == 6, 0); return Result.Err(local.value);")
        .replace("Packet", "Snapshot").replace("Flags", "Bits")
        .replace("Result", "Outcome").replace("Canvas", "Frame");
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
    assert!(!program
        .yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "call_owned_struct"
            && node
                .op
                .args
                .get(1)
                .is_some_and(|layout| layout.contains(":Bytes"))));
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
fn record_local_literal_short_circuit_fields_use_ordered_staging() {
    let source = source("let saved: Bytes = copy_bytes(pixels); let local: Flags = Flags { valid: mode == 0 && buffer_len(pixels) == 6 }; return Result.Ok(Canvas { width: word(local.valid), pixels: move(saved) });");
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
