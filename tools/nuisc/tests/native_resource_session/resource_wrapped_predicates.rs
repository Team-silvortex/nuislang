use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::resource_predicates::source;
use super::*;

fn helpers(expression: &str) -> String {
    source(expression).replace(
        "mod cpu Main {",
        "mod cpu Main { fn word(flag: bool) -> i64 { if flag { return 9; } return 0; }",
    )
}

#[test]
fn wrapped_resource_predicates_keep_truth_tables_and_owners_for_120_events() {
    for (case, expression) in [
        "(valid && load_at(pixels, 3) == 9) == (mode < 3)",
        "valid != (mode < 3 && bytes_len(saved) == 48)",
        "(valid && load_at(pixels, 3) == 9) ^ (mode < 3 || bytes_len(saved) == 0)",
        "!((valid && load_at(pixels, 3) == 9) == (mode < 3))",
        "((valid && load_at(pixels, 3) == 9) ^ (mode < 3)) == valid",
        "choose((valid && load_at(pixels, 3) == 9) ^ (mode < 3)).value",
        "((valid && load_at(pixels, 3) == 9) == (mode < 3)) || bytes_len(saved) == 0",
        "load_at(pixels, 3) == word((valid && bytes_len(saved) == 48) ^ (mode < 3))",
        "word(valid && bytes_len(saved) == 48) < load_at(pixels, 3)",
        "load_at(pixels, 3) <= word(valid && bytes_len(saved) == 48)",
        "word(valid && bytes_len(saved) == 48) > word(mode < 3 || bytes_len(saved) == 0)",
        "word(valid && bytes_len(saved) == 48) >= load_at(pixels, 3)",
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
            .unwrap_or_else(|e| panic!("{case}/{form}: {e}"));
            assert!(!bridge.llvm_ir.contains("; deferred lowering"));
            assert_eq!(
                program
                    .yir
                    .nodes
                    .iter()
                    .filter(|n| n.op.instruction == "store_at")
                    .count(),
                2,
                "wrapper extraction must not duplicate the suffix"
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
            let mut pixels = vec![9; 6];
            let mut width = 3;
            for mode in (0..6).cycle().take(120) {
                let even = mode % 2 == 0;
                let small = mode < 3;
                let decision = match case {
                    0 | 6 => even == small,
                    1 | 2 | 3 | 5 | 7 => even != small,
                    4 => !small,
                    8 => !even,
                    9 | 11 => even,
                    10 => even && !small,
                    _ => unreachable!(),
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
fn resource_wrapper_snapshots_preserve_writes_shadowing_and_name_hygiene() {
    let source = helpers("(load_at(pixels, 0) == mode) == (valid && bytes_len(saved) == 48)")
        .replace("if mode >= 0 {", "let __nuis_resource_predicate_left: bool = mode < 0; store_at(pixels, 0, mode); if mode >= 0 {")
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
    let mut width = 3;
    let mut pixels = vec![9; 6];
    for mode in (0..6).cycle().take(120) {
        if mode % 2 == 0 {
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
fn resource_wrapper_errors_and_failed_results_preserve_state_and_release_owners() {
    for allocation_failure in [false, true] {
        let source =
            helpers("((valid && load_at(pixels, 3) == 9) == (mode < 3)) ^ (bytes_len(saved) < 48)");
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
fn resource_wrappers_reach_direct_i64_initializers() {
    let expression = "word((valid && bytes_len(saved) == 48) ^ (mode < 3))";
    let source = helpers(&format!("{expression} == 9")).replace(
        &format!("let valid: bool = {expression} == 9;"),
        &format!("let choice: i64 = {expression}; let valid: bool = choice == 9;"),
    );
    assert!(!bridge(&compile(&source))
        .llvm_ir
        .contains("; deferred lowering"));
}
