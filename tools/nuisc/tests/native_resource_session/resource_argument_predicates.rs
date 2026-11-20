use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::resource_predicates::source;
use super::*;

fn helpers(expression: &str) -> String {
    source(expression).replace(
        "mod cpu Main {",
        r#"mod cpu Main {
      fn pick(left: i64, flag: bool, right: i64) -> Choice {
        return Choice { value: flag && left == 9 && right > 0 };
      }
      fn word(flag: bool) -> i64 { if flag { return 9; } return 0; }
      fn both(first: bool, second: bool) -> Choice { return Choice { value: first && second }; }
    "#,
    )
}

#[test]
fn resource_logical_arguments_preserve_120_events_per_declaration_form() {
    for (case, expression) in [
        "valid && choose(mode < 3 && load_at(pixels, 3) == 9).value",
        "choose(valid || bytes_len(saved) == 0).value",
        "!choose(!(valid && load_at(pixels, 3) == 9)).value",
        "choose(valid && choose(mode < 3 || bytes_len(saved) == 48).value).value",
        "pick(load_at(pixels, 3), valid && bytes_len(saved) == 48, load_at(pixels, 0)).value",
        "choose(valid && load_at(pixels, 3) == 9).value || mode == 3",
        "both(valid && load_at(pixels, 3) == 9, mode < 3 || bytes_len(saved) == 48).value",
        "load_at(pixels, 3) == word(valid && bytes_len(saved) == 48)",
    ]
    .into_iter()
    .enumerate()
    {
        for form in ["explicit", "inferred", "const-alias"] {
            let source = helpers(expression);
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
            let bridge = yir_lower_llvm::native_resource_session::emit_registered(
                &program.yir,
                &registration(),
            )
            .unwrap_or_else(|error| panic!("{case}/{form}: {error}"));
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
                    0 => mode % 2 == 0 && mode < 3,
                    5 => mode % 2 == 0 || mode == 3,
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
fn resource_logical_argument_snapshots_keep_write_order_and_source_names() {
    let source = helpers("pick(load_at(pixels, 0), valid && bytes_len(saved) == 48, load_at(pixels, 3)).value")
        .replace("if mode >= 0 {", "let __nuis_resource_predicate_left: bool = mode < 0; store_at(pixels, 0, mode + 9); if mode >= 0 {")
        .replace("store_at(pixels, 1, mode);", "if __nuis_resource_predicate_left { return Result.Err(91); } store_at(pixels, 0, 99); store_at(pixels, 1, mode);")
        .replace("drop_bytes(saved);", "if __nuis_resource_predicate_left { return Result.Err(91); } drop_bytes(saved);");
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
        if mode == 0 {
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
fn resource_logical_argument_errors_and_failed_returns_preserve_state_and_owners() {
    for expression in [
        "choose(valid && bytes_len(saved) == 48).value",
        "load_at(pixels, 3) == word(valid && bytes_len(saved) == 48)",
    ] {
        for allocation_failure in [false, true] {
            let source = helpers(expression);
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
}
