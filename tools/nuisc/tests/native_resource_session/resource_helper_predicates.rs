use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::resource_predicates::source;
use super::*;

#[test]
fn resource_helper_predicates_preserve_selected_values_and_owners_for_120_events() {
    for (case, expression) in [
        "valid && positive(load_at(pixels, 3))",
        "!valid || choose(bytes_len(saved) < 48).value",
        "!!(valid || positive(load_at(pixels, 3) - 9))",
        "valid && (mode < 2 || choose(bytes_len(saved) == 48).value)",
        "(valid && choose(load_at(pixels, 3) == 9).value) || mode == 3",
        "valid && verdict(bytes_len(saved)).choice.value",
        "valid && load_at(pixels, offset(2)) == 9",
    ]
    .into_iter()
    .enumerate()
    {
        for form in ["explicit", "inferred", "const-alias"] {
            let source = source(expression).replace(
                "mod cpu Main {",
                r#"mod cpu Main {
              struct Verdict { choice: Choice, code: i64 }
              fn verdict(value: i64) -> Verdict {
                return Verdict { choice: choose(value == 48), code: value };
              }
            "#,
            );
            let source = match form {
                "inferred" => source.replace("let valid: bool", "let valid"),
                "const-alias" => source.replace("let valid: bool", "const valid: Decision"),
                _ => source,
            };
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
            let mut pixels = vec![9; 6];
            let mut width = 3;
            for mode in (0..6).cycle().take(120) {
                let decision = match case {
                    1 => mode % 2 != 0,
                    4 => mode % 2 == 0 || mode == 3,
                    _ => mode % 2 == 0,
                };
                if decision {
                    width += 10;
                } else {
                    pixels[1] = mode;
                    pixels[0] = width;
                    width += 1;
                }
                session.event(mode).unwrap();
                assert_eq!(session.state(), pixels, "{case}/{form}/{mode}");
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
fn resource_helper_predicate_errors_and_failed_returns_never_publish_or_leak() {
    for allocation_failure in [false, true] {
        let source = source("valid && choose(bytes_len(saved) == 48).value");
        let source = if allocation_failure {
            source
        } else {
            source
                .replace(
                    "Result.Ok(Canvas { width: width + 10, pixels: move(saved) })",
                    "Result.Err(73)",
                )
                .replace(
                    "Result.Ok(Canvas { width: width + 1, pixels: copy_bytes(pixels) })",
                    "Result.Err(74)",
                )
        };
        let program = compile(&source);
        let bridge = bridge(&program);
        let mut llvm = bridge.llvm_ir.clone();
        if allocation_failure {
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
            let error = session.event(mode).unwrap_err();
            assert!(
                error.contains(if allocation_failure {
                    "status 2"
                } else if mode == 0 {
                    "code=73"
                } else {
                    "code=74"
                }),
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
}

#[test]
fn resource_helper_predicates_reject_effects_hidden_logic_and_resource_signatures() {
    for (expression, helper) in [
        (
            "valid && noisy(load_at(pixels, 0))",
            "fn noisy(value: i64) -> bool { print(value); return value > 0; }",
        ),
        (
            "valid || choose((mode < 3 && noisy(load_at(pixels, 99))) ^ valid).value",
            "fn noisy(value: i64) -> bool { print(value); return value > 0; }",
        ),
        (
            "valid && read(pixels, bytes_len(saved))",
            "fn read(value: ref Buffer, size: i64) -> bool { return load_at(value, 0) > size; }",
        ),
        (
            "valid && count(saved, load_at(pixels, 0))",
            "fn count(value: ref Bytes, size: i64) -> bool { return bytes_len(value) > size; }",
        ),
    ] {
        let source =
            source(expression).replace("mod cpu Main {", &format!("mod cpu Main {{ {helper}"));
        assert!(
            nuisc::pipeline::compile_resource_session_source(
                &source,
                &registration(),
                &Default::default()
            )
            .is_err(),
            "{expression}"
        );
    }
}

#[test]
fn resource_helper_predicate_reads_observe_writes_before_the_selected_call() {
    let source = source("valid && positive(load_at(pixels, 0))")
        .replace(
            "if mode >= 0 {",
            "store_at(pixels, 0, mode - 2); if mode >= 0 {",
        )
        .replace(
            "store_at(pixels, 1, mode);",
            "store_at(pixels, 0, 99); store_at(pixels, 1, mode);",
        );
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
    let mut pixels = vec![9; 6];
    let mut width = 3;
    for mode in (0..6).cycle().take(120) {
        if mode == 4 {
            width += 10;
        } else {
            pixels[0] = width;
            pixels[1] = mode;
            width += 1;
        }
        session.event(mode).unwrap();
        assert_eq!(session.state(), pixels);
        assert_eq!(session.scalar_state(), [width]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}

#[test]
fn resource_helper_predicates_keep_one_typed_bounded_native_closure() {
    let program = compile(&source("valid && positive(load_at(pixels, 3))"));
    bridge(&program);
    let callee = program
        .yir
        .nodes
        .iter()
        .find(|n| n.op.instruction == "call_owned_struct")
        .expect("a projected record call survives scalar inlining")
        .op
        .args[0]
        .clone();
    for mutation in 0..3 {
        let mut changed = program.yir.clone();
        let helper = changed
            .functions
            .iter_mut()
            .find(|f| f.name == callee)
            .unwrap();
        assert_eq!(helper.parameters[0].ty, "bool");
        match mutation {
            0 => helper.parameters[0].ty = "i64".into(),
            1 => helper.parameters[0].ownership = yir_core::YirValueOwnership::Borrowed,
            _ => helper.result.as_mut().unwrap().ty = "i64".into(),
        }
        assert!(yir_lower_llvm::native_resource_session::emit_registered(
            &changed,
            &registration()
        )
        .is_err());
    }
    let helpers = (0..9)
        .map(|i| {
            format!(
                "fn chain{i}(value: bool) -> Choice {{ return Choice {{ value: {}(value).value }}; }}",
                if i == 8 {
                    "choose".into()
                } else {
                    format!("chain{}", i + 1)
                },
            )
        })
        .collect::<String>();
    let source = source("valid && chain0(load_at(pixels, 3) > 0).value")
        .replace("mod cpu Main {", &format!("mod cpu Main {{ {helpers}"));
    let program = nuisc::pipeline::compile_resource_session_source(
        &source,
        &registration(),
        &Default::default(),
    )
    .unwrap();
    // Record construction keeps the chain observable after scalar inlining.
    assert_eq!(
        program
            .yir()
            .functions
            .iter()
            .filter(|f| f.name.starts_with("chain"))
            .count(),
        9
    );
    let error =
        yir_lower_llvm::native_resource_session::emit_registered(program.yir(), &registration())
            .unwrap_err();
    assert!(
        error.contains("closure depth bound") || error.contains("closure bound"),
        "{error}"
    );
}
