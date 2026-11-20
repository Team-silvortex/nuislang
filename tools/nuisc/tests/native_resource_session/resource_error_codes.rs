use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::*;

fn source(body: &str) -> String {
    super::boolean_short_circuit::helpers(body).replace("mod cpu Main {", r#"mod cpu Main {
        struct Code { value: i64 }
        fn word(flag: bool) -> i64 { if flag { return 9; } return 0; }
        fn code(flag: bool) -> Code { return Code { value: word(flag) }; }
        fn wide(flag: bool) -> i64 { if flag { return 9223372036854775807; } return 0 - 9223372036854775807 - 1; }
    "#)
}

#[test]
fn direct_resource_error_codes_preserve_values_committed_state_and_all_owners() {
    for (expression, codes) in [
        ("word(mode == 0 && bytes_len(saved) == 48)", [9, 0]),
        ("word(!(mode == 0 || bytes_len(saved) == 0))", [0, 9]),
        (
            "word((mode == 0 && load_at(pixels, 0) == 99) ^ (mode != 0 || bytes_len(saved) == 0))",
            [9, 9],
        ),
        (
            "code(mode == 0 && bytes_len(saved) == 48).value + 17",
            [26, 17],
        ),
        (
            "wide(mode == 0 && bytes_len(saved) == 48)",
            [i64::MAX, i64::MIN],
        ),
    ] {
        for early in [false, true] {
            let returned = format!("return Result.Err({expression});");
            let returned = if early {
                format!("if mode >= 0 {{ let local: Bytes = copy_bytes(pixels); {returned} }} return Result.Ok(Canvas {{ width: width + 100, pixels: move(saved) }});")
            } else {
                returned
            };
            let source = source(&format!(
                r#"
                if mode < 0 {{ store_at(pixels, 1, width); return Result.Ok(Canvas {{ width: width + 1, pixels: copy_bytes(pixels) }}); }}
                let __nuis_resource_predicate_left: i64 = mode;
                let saved: Bytes = copy_bytes(pixels);
                let spare: Bytes = copy_bytes(pixels);
                store_at(pixels, 0, 99);
                {returned}
            "#
            ));
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
            for mode in 0..2 {
                let (mut session, _) = NativeResourceSession::open(
                    &program.yir,
                    &registration(),
                    library.checked_binding(&program.yir, &bridge),
                    vec![9; 6],
                    3,
                )
                .unwrap();
                session.event(-1).unwrap();
                let error = session.event(mode).unwrap_err();
                assert!(
                    error.contains(&format!("code={}", codes[mode as usize])),
                    "{expression}/{early}/{mode}: {error}"
                );
                assert_eq!(session.state(), [9, 3, 9, 9, 9, 9]);
                assert_eq!(session.scalar_state(), [4]);
                assert_eq!(library.live_blobs(), 0);
                session.close(0).unwrap();
                assert!(session.completion_status().is_err());
                assert_eq!(library.live_blobs(), 0);
            }
        }
    }
}

#[test]
fn direct_resource_error_result_allocation_failure_is_not_an_application_code() {
    let source = source("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); store_at(pixels, 0, 99); return Result.Err(word(mode == 0 && bytes_len(saved) == 48));");
    let program = compile(&source);
    let bridge = bridge(&program);
    let mut llvm = bridge.llvm_ir.clone();
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
        let error = session.event(mode).unwrap_err();
        assert!(
            error.contains("status 2") && !error.contains("application error"),
            "{error}"
        );
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(session.scalar_state(), [3]);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        assert!(session.completion_status().is_err());
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
fn direct_resource_error_codes_keep_registered_identity_without_held_bytes() {
    let source = source("return Result.Err(word(mode == 0 && load_at(pixels, 0) == 9));")
        .replace("Result", "Outcome")
        .replace("Canvas", "Frame");
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
    assert!(program
        .yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "guard_call_owned_struct_return"));
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
    }
}

#[test]
fn returned_resource_buffer_length_composition_uses_guarded_initializers() {
    let source = source("let saved: Bytes = copy_bytes(pixels); return Result.Ok(Canvas { width: buffer_len(pixels) + word(mode >= 0 && bytes_len(saved) == 48), pixels: move(saved) });");
    let program = compile(&source);
    let bridge = bridge(&program);
    let library = native::Library::build(&source, &program, &bridge.llvm_ir);
    for length in [1, 6, 17] {
        let pixels = vec![9; length];
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            pixels.clone(),
            3,
        )
        .unwrap();
        for mode in [-1, 0].into_iter().cycle().take(120) {
            session.event(mode).unwrap();
            let expected = length as i64 + if mode >= 0 && length == 6 { 9 } else { 0 };
            assert_eq!(session.scalar_state(), [expected]);
            assert_eq!(session.state(), pixels);
            assert_eq!(library.live_blobs(), 0);
        }
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(library.live_blobs(), 0);
    }
}
