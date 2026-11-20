use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::*;

pub(super) const TRAPS: &[(&str, &str, bool)] = &[
    (
        "i64_and_skip",
        "word(mode != 0 && load_at(pixels, 99) > 0)",
        false,
    ),
    (
        "i64_and_take",
        "word(mode == 0 && load_at(pixels, 99) > 0)",
        false,
    ),
    (
        "i64_or_skip",
        "word(mode == 0 || load_at(pixels, 99) > 0)",
        true,
    ),
    (
        "i64_or_take",
        "word(mode != 0 || load_at(pixels, 99) > 0)",
        false,
    ),
    (
        "i64_not_and_skip",
        "word(!(mode != 0 && load_at(pixels, 99) > 0))",
        true,
    ),
    (
        "i64_not_and_take",
        "word(!(mode == 0 && load_at(pixels, 99) > 0))",
        false,
    ),
    (
        "i64_not_or_skip",
        "word(!(mode == 0 || load_at(pixels, 99) > 0))",
        false,
    ),
    (
        "i64_not_or_take",
        "word(!(mode != 0 || load_at(pixels, 99) > 0))",
        false,
    ),
    (
        "i64_xor_skip",
        "word((mode != 0 && load_at(pixels, 99) > 0) ^ (bytes_len(saved) == 48))",
        true,
    ),
    (
        "i64_xor_take",
        "word((mode == 0 && load_at(pixels, 99) > 0) ^ (bytes_len(saved) == 48))",
        false,
    ),
    (
        "i64_nested_skip",
        "word(mode == 0 || (bytes_len(saved) > 0 && load_at(pixels, 99) > 0))",
        true,
    ),
    (
        "i64_nested_take",
        "word(mode != 0 || (bytes_len(saved) > 0 && load_at(pixels, 99) > 0))",
        false,
    ),
    (
        "i64_body_skip",
        "word(mode != 0 && divide(bytes_len(saved) - 48))",
        false,
    ),
    (
        "i64_body_take",
        "word(mode == 0 && divide(bytes_len(saved) - 48))",
        false,
    ),
    (
        "i64_conditional_skip",
        "ratio(mode != 0 && bytes_len(saved) == 48, mode)",
        false,
    ),
    (
        "i64_conditional_take",
        "ratio(mode == 0 && bytes_len(saved) == 48, mode)",
        false,
    ),
    (
        "i64_left_take",
        "load_at(pixels, 99) + word(mode != 0 && bytes_len(saved) > 0)",
        false,
    ),
    (
        "i64_right_take",
        "ratio(mode != 0 && bytes_len(saved) > 0, load_at(pixels, 99))",
        false,
    ),
    (
        "i64_eager_left_take",
        "100 / mode + word(mode != 0 && bytes_len(saved) > 0)",
        false,
    ),
    (
        "i64_eager_right_take",
        "word(mode != 0 && bytes_len(saved) > 0) + 100 / mode",
        false,
    ),
    (
        "i64_index_skip",
        "load_at(pixels, word(mode != 0 && load_at(pixels, 99) > 0))",
        true,
    ),
    (
        "i64_index_take",
        "load_at(pixels, word(mode == 0 && load_at(pixels, 99) > 0))",
        false,
    ),
    ("i64_pure_skip", "word(mode != 0 && divide(mode))", false),
    ("i64_pure_take", "word(mode == 0 && divide(mode))", false),
];

fn source(expression: &str) -> String {
    super::resource_predicates::source("choice == 9")
        .replace("mod cpu Main {", r#"mod cpu Main {
            type Word = i64;
            struct Number { value: i64 }
            fn word(flag: bool) -> i64 { if flag { return 9; } return 0; }
            fn pick(left: i64, flag: bool, right: i64) -> i64 { if flag { return left; } return right; }
            fn number(flag: bool) -> Number { return Number { value: word(flag) }; }
            fn index(flag: bool) -> i64 { if flag { return 3; } return 4; }
        "#)
        .replace("let valid: bool = choice == 9;", &format!("let choice: i64 = {expression}; let valid: bool = choice == 9;"))
}

#[test]
fn direct_i64_resource_arguments_preserve_values_and_owners_for_120_events() {
    for (case, expression) in [
        "word(valid && load_at(pixels, 3) == 9)",
        "word(!(valid || bytes_len(saved) == 0))",
        "word((valid && bytes_len(saved) == 48) ^ (mode < 3))",
        "offset(word(valid && bytes_len(saved) == 48)) - 1",
        "pick(load_at(pixels, 3), valid && bytes_len(saved) == 48, 0)",
        "number(valid && bytes_len(saved) == 48).value",
        "load_at(pixels, index(valid && bytes_len(saved) == 48))",
        "pick(word(valid && bytes_len(saved) == 48), mode < 3, word(!valid && bytes_len(saved) == 48))",
    ].into_iter().enumerate() {
        for form in ["explicit", "inferred", "const-alias"] {
            let source = source(expression);
            let source = match form {
                "inferred" => source.replace("let choice: i64", "let choice"),
                "const-alias" => source.replace("let choice: i64", "const choice: Word"),
                _ => source,
            };
            let program = nuisc::pipeline::compile_resource_session_source(&source, &registration(), &Default::default())
                .and_then(|program| program.emit_llvm())
                .unwrap_or_else(|error| panic!("{case}/{form}: {error}"));
            assert_eq!(program.ast, nuisc::frontend::parse_nuis_ast(&source).unwrap());
            assert_eq!(program.yir, yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap());
            let bridge = bridge(&program);
            assert!(!bridge.llvm_ir.contains("; deferred lowering"));
            assert_eq!(program.yir.nodes.iter().filter(|n| n.op.instruction == "store_at").count(), 2);
            let library = native::Library::build(&source, &program, &bridge.llvm_ir);
            let (mut session, _) = NativeResourceSession::open(&program.yir, &registration(), library.checked_binding(&program.yir, &bridge), vec![9; 6], 3).unwrap();
            let mut pixels = vec![9; 6];
            let mut width = 3;
            for mode in (0..6).cycle().take(120) {
                let even = mode % 2 == 0;
                let decision = match case {
                    1 => !even,
                    2 => even ^ (mode < 3),
                    6 => true,
                    7 => if mode < 3 { even } else { !even },
                    _ => even,
                };
                if decision { width += 10; } else { pixels[1] = mode; pixels[0] = width; width += 1; }
                session.event(mode).unwrap();
                assert_eq!(session.state(), pixels, "{case}/{form}/{mode}");
                assert_eq!(session.scalar_state(), [width], "{case}/{form}/{mode}");
                assert_eq!(library.live_blobs(), 0);
            }
            session.close(0).unwrap();
            session.completion_status().unwrap();
            assert_eq!(library.live_blobs(), 0);
        }
    }
}

#[test]
fn direct_i64_resource_arguments_keep_full_width_values() {
    let source = source(
        "pick(9223372036854775807, valid && bytes_len(saved) == 48, 0 - 9223372036854775807 - 1)",
    )
    .replace("choice == 9", "choice > 0")
    .replace("width: width + 10", "width: choice");
    // The continuation is outside the inner binding's scope: publish its value
    // through an explicit store, not an accidental capture beyond that scope.
    let source = source.replace("store_at(pixels, 1, mode);", "store_at(pixels, 1, choice);");
    let program = compile(&source);
    let bridge = bridge(&program);
    let library = native::Library::build(&source, &program, &bridge.llvm_ir);
    for mode in [0, 1] {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        session.event(mode).unwrap();
        if mode == 0 {
            assert_eq!(session.scalar_state(), [i64::MAX]);
            assert_eq!(session.state(), [9; 6]);
        } else {
            assert_eq!(session.scalar_state(), [4]);
            assert_eq!(session.state(), [3, i64::MIN, 9, 9, 9, 9]);
        }
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
fn direct_i64_initializers_preserve_old_scope_writes_and_temporary_hygiene() {
    let source = source("pick(load_at(pixels, 0), choice == 9 && bytes_len(saved) == 48, 0)")
        .replace("if mode >= 0 {", "let choice: i64 = 9; let __nuis_resource_predicate_left: i64 = mode; store_at(pixels, 0, 9); if mode >= 0 {")
        .replace("store_at(pixels, 1, mode);", "store_at(pixels, 0, 99); if __nuis_resource_predicate_left != mode { return Result.Err(91); } store_at(pixels, 1, mode);");
    let program = compile(&source);
    let bridge = bridge(&program);
    let library = native::Library::build(&source, &program, &bridge.llvm_ir);
    let (mut session, _) = NativeResourceSession::open(
        &program.yir,
        &registration(),
        library.checked_binding(&program.yir, &bridge),
        vec![7; 6],
        3,
    )
    .unwrap();
    for mode in 0..120 {
        session.event(mode).unwrap();
        assert_eq!(session.state(), [7; 6]);
        assert_eq!(session.scalar_state(), [3 + (mode + 1) * 10]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}

#[test]
fn direct_i64_errors_and_failed_results_preserve_state_and_release_owners() {
    for allocation_failure in [false, true] {
        let source = source("word(valid && bytes_len(saved) == 48)");
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
fn direct_inline_resource_conditions_reach_guarded_lowering() {
    let expression = "word((valid && bytes_len(saved) == 48) ^ (mode < 3))";
    let source = source(expression)
        .replace(
            &format!("let choice: i64 = {expression}; let valid: bool = choice == 9;"),
            "",
        )
        .replace("if valid {", &format!("if {expression} == 9 {{"));
    assert!(!bridge(&compile(&source))
        .llvm_ir
        .contains("; deferred lowering"));
}

#[test]
fn direct_i64_arguments_retain_the_source_closure_depth_bound() {
    let source = source("pick(word(valid && bytes_len(saved) == 48), mode < 3 || load_at(pixels, 3) == 0, word(!valid && bytes_len(saved) == 48))");
    let error = nuisc::pipeline::compile_resource_session_source(
        &source,
        &registration(),
        &Default::default(),
    )
    .err()
    .expect("over-budget resource closure must remain rejected");
    assert!(
        error.contains("resource return helpers exceed the source closure depth bound"),
        "{error}"
    );
}
