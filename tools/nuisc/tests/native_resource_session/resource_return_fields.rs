use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::*;

fn source(body: &str) -> String {
    super::boolean_short_circuit::helpers(body).replace(
        "mod cpu Main {",
        "mod cpu Main { fn word(flag: bool) -> i64 { if flag { return 9; } return 0; }",
    )
}

#[test]
fn returned_resource_fields_keep_source_order_and_owners_for_120_events() {
    for (prefix, fields) in [
        ("let saved: Bytes = copy_bytes(pixels);", "width: width + word(mode % 2 == 0 && bytes_len(saved) == 48), pixels: move(saved)"),
        ("", "pixels: copy_bytes(pixels), width: width + word(mode % 2 == 0 && load_at(pixels, 3) == 9)"),
        ("let saved: Bytes = copy_bytes(pixels);", "pixels: move(saved), width: width + word(mode % 2 == 0 && load_at(pixels, 3) == 9)"),
    ] {
        for early in [false, true] {
            let body = format!("{prefix} return Result.Ok(Canvas {{ {fields} }});");
            let body = if early { format!("if mode >= 0 {{ {body} }} return Result.Err(77);") } else { body };
            let source = source(&body);
            let program = compile(&source);
            assert_eq!(program.ast, nuisc::frontend::parse_nuis_ast(&source).unwrap());
            assert_eq!(program.yir, yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap());
            let bridge = bridge(&program);
            assert!(!bridge.llvm_ir.contains("; deferred lowering"));
            let library = native::Library::build(&source, &program, &bridge.llvm_ir);
            let (mut session, _) = NativeResourceSession::open(&program.yir, &registration(), library.checked_binding(&program.yir, &bridge), vec![9; 6], 3).unwrap();
            let mut width = 3;
            for mode in (0..6).cycle().take(120) {
                if mode % 2 == 0 { width += 9; }
                session.event(mode).unwrap();
                assert_eq!(session.state(), [9; 6]);
                assert_eq!(session.scalar_state(), [width]);
                assert_eq!(library.live_blobs(), 0);
            }
            session.close(0).unwrap();
            session.completion_status().unwrap();
            assert_eq!(library.live_blobs(), 0);
        }
    }
}

#[test]
fn returned_resource_fields_failed_result_allocation_preserves_state_and_releases_staged_bytes() {
    for (prefix, fields) in [
        (
            "",
            "width: width + word(mode == 0 && bytes_len(saved) == 48), pixels: move(saved)",
        ),
        (
            "",
            "pixels: move(saved), width: width + word(mode == 0 && load_at(pixels, 3) == 9)",
        ),
        (
            "",
            "pixels: copy_bytes(pixels), width: width + word(mode == 0 && bytes_len(saved) == 48)",
        ),
        (
            "let prior = choose(mode == 0);",
            "pixels: move(saved), width: width + word(prior.value && bytes_len(spare) == 48)",
        ),
    ] {
        let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); {prefix} store_at(pixels, 0, 99); return Result.Ok(Canvas {{ {fields} }});"));
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
            assert!(session.event(mode).unwrap_err().contains("status 2"));
            assert_eq!(session.state(), [9; 6]);
            assert_eq!(session.scalar_state(), [3]);
            assert_eq!(library.live_blobs(), 0);
            session.close(0).unwrap();
            assert!(session.completion_status().is_err());
            assert_eq!(library.live_blobs(), 0);
        }
    }
}

#[test]
fn returned_resource_fields_follow_registered_layouts_not_canvas_or_result_names() {
    for checked in [false, true] {
        let value = "Frame { score: word(mode % 2 == 0 && load_at(pixels, 3) == 9), data: copy_bytes(pixels), ticks: ticks + word(mode < 3 && load_at(pixels, 3) == 9) }";
        let initial = "Frame { ticks: seed, data: copy_bytes(pixels), score: 0 }";
        let final_state = "Frame { score: score, ticks: ticks, data: copy_bytes(pixels) }";
        let wrap = |value: &str| {
            if checked {
                format!("Outcome.Ok({value})")
            } else {
                value.to_owned()
            }
        };
        let result = if checked {
            "Outcome<Frame, i64>"
        } else {
            "Frame"
        };
        let source = format!(
            r#"mod cpu Main {{
          enum Outcome<T, E> {{ Ok(T), Err(E) }}
          struct Frame {{ ticks: i64, data: Bytes, score: i64 }}
          fn word(flag: bool) -> i64 {{ if flag {{ return 9; }} return 0; }}
          fn start(pixels: ref Buffer, seed: i64) -> {result} {{ return {}; }}
          fn step(pixels: ref Buffer, ticks: i64, score: i64, mode: i64) -> {result} {{ return {}; }}
          fn stop(pixels: ref Buffer, ticks: i64, score: i64, reason: i64) -> {result} {{ return {}; }}
          fn main() -> i64 {{ return 0; }}
        }}"#,
            wrap(initial),
            wrap(value),
            wrap(final_state)
        );
        let mut registration = registration();
        registration.state_type = Some("Frame".into());
        registration.result_type = checked.then(|| "Outcome".into());
        if !checked {
            registration.contract = yir_core::native_resource_session::MIXED_CONTRACT.into();
        }
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
        let binding = if checked {
            library.checked_binding(&program.yir, &bridge)
        } else {
            library.mixed_binding(&program.yir, &bridge)
        };
        let (mut session, _) =
            NativeResourceSession::open(&program.yir, &registration, binding, vec![9; 6], 3)
                .unwrap();
        let mut ticks = 3;
        for mode in (0..6).cycle().take(120) {
            if mode < 3 {
                ticks += 9;
            }
            session.event(mode).unwrap();
            assert_eq!(
                session.scalar_state(),
                [ticks, if mode % 2 == 0 { 9 } else { 0 }]
            );
            assert_eq!(session.state(), [9; 6]);
            assert_eq!(library.live_blobs(), 0);
        }
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
fn returned_resource_fields_do_not_read_moved_owners_even_under_false_gates() {
    let source = source("let saved: Bytes = copy_bytes(pixels); return Result.Ok(Canvas { pixels: move(saved), width: word(false && bytes_len(saved) > 0) });");
    let error = nuisc::pipeline::compile_resource_session_source(
        &source,
        &registration(),
        &Default::default(),
    )
    .err()
    .expect("read after move must fail");
    assert!(
        error.contains("moved") || error.contains("consumed"),
        "{error}"
    );
}

#[test]
fn returned_resource_error_codes_use_guarded_initializers_before_cleanup() {
    let source = source("let saved: Bytes = copy_bytes(pixels); return Result.Err(word(mode >= 0 && bytes_len(saved) == 48));");
    let program = compile(&source);
    let bridge = bridge(&program);
    let library = native::Library::build(&source, &program, &bridge.llvm_ir);
    for (mode, code) in [(-1, 0), (0, 9)] {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        let error = session.event(mode).unwrap_err();
        assert!(error.contains(&format!("code={code}")), "{error}");
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(session.scalar_state(), [3]);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        assert!(session.completion_status().is_err());
        assert_eq!(library.live_blobs(), 0);
    }
}
