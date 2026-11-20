use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::*;

pub(super) const TRAPS: &[&str] = &[
    "inline_read_and_skip",
    "inline_read_and_take",
    "inline_read_or_skip",
    "inline_read_or_take",
    "inline_read_not_and_skip",
    "inline_read_not_and_take",
    "inline_read_not_or_skip",
    "inline_read_not_or_take",
    "inline_i64_and_skip",
    "inline_i64_and_take",
    "inline_i64_or_skip",
    "inline_i64_or_take",
    "inline_i64_not_and_skip",
    "inline_i64_not_and_take",
    "inline_i64_not_or_skip",
    "inline_i64_not_or_take",
    "inline_i64_xor_skip",
    "inline_i64_xor_take",
    "inline_i64_nested_skip",
    "inline_i64_nested_take",
    "inline_i64_body_skip",
    "inline_i64_body_take",
    "inline_i64_conditional_skip",
    "inline_i64_conditional_take",
    "inline_i64_left_take",
    "inline_i64_right_take",
    "inline_i64_eager_left_take",
    "inline_i64_eager_right_take",
    "inline_i64_index_skip",
    "inline_i64_index_take",
    "inline_i64_pure_skip",
    "inline_i64_pure_take",
];

fn source(expression: &str) -> String {
    super::resource_predicates::source("inline_condition")
        .replace("let valid: bool = inline_condition;", "")
        .replace("if valid {", &format!("if {expression} {{"))
        .replace(
            "mod cpu Main {",
            "mod cpu Main { fn word(flag: bool) -> i64 { if flag { return 9; } return 0; }",
        )
}

#[test]
fn inline_resource_conditions_preserve_selected_values_and_owners_for_120_events() {
    for (case, expression) in [
        "valid && load_at(pixels, 3) == 9",
        "!(valid || bytes_len(saved) == 0)",
        "(valid && load_at(pixels, 3) == 9) == (mode < 3)",
        "(valid && load_at(pixels, 3) == 9) ^ (mode < 3 || bytes_len(saved) == 0)",
        "word((valid && bytes_len(saved) == 48) ^ (mode < 3)) == 9",
        "choose(mode < 3 && positive(load_at(pixels, 3))).value",
        "!(mode < 3 && (bytes_len(saved) == 0 || load_at(pixels, 3) == 9))",
        "word(valid && bytes_len(saved) == 48) < load_at(pixels, 3)",
    ]
    .into_iter()
    .enumerate()
    {
        let source = source(expression);
        let program = nuisc::pipeline::compile_resource_session_source(
            &source,
            &registration(),
            &Default::default(),
        )
        .and_then(|program| program.emit_llvm())
        .unwrap_or_else(|error| panic!("{case}: {error}"));
        assert_eq!(
            program.ast,
            nuisc::frontend::parse_nuis_ast(&source).unwrap()
        );
        assert_eq!(
            program.yir,
            yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap()
        );
        let bridge = bridge(&program);
        assert!(!bridge.llvm_ir.contains("; deferred lowering"));
        assert_eq!(
            program
                .yir
                .nodes
                .iter()
                .filter(|n| n.op.instruction == "store_at")
                .count(),
            2
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
        let mut width = 3;
        let mut pixels = vec![9; 6];
        for mode in (0..6).cycle().take(120) {
            let even = mode % 2 == 0;
            let small = mode < 3;
            let selected = match case {
                0 => even,
                1 | 7 => !even,
                2 => even == small,
                3 | 4 => even ^ small,
                5 => small,
                6 => !small,
                _ => unreachable!(),
            };
            if selected {
                width += 10;
            } else {
                pixels[1] = mode;
                pixels[0] = width;
                width += 1;
            }
            session.event(mode).unwrap();
            assert_eq!(session.state(), pixels, "{case}/{mode}");
            assert_eq!(session.scalar_state(), [width], "{case}/{mode}");
            assert_eq!(library.live_blobs(), 0);
        }
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
fn inline_resource_conditions_keep_both_arms_local_owners_and_one_suffix() {
    let source = super::boolean_short_circuit::helpers(
        r#"
        let saved: Bytes = copy_bytes(pixels);
        let spare: Bytes = copy_bytes(pixels);
        if mode < 3 && bytes_len(saved) == 48 {
            store_at(pixels, 1, width);
            let local: Bytes = copy_bytes(pixels);
            if mode == 0 { return Result.Ok(Canvas { width: width + 100, pixels: move(saved) }); }
            if mode == 1 { return Result.Ok(Canvas { width: width + 200, pixels: move(local) }); }
            store_at(pixels, 2, width + 6);
        } else {
            store_at(pixels, 3, width + 10);
            let local: Bytes = copy_bytes(pixels);
            if mode == 3 { return Result.Ok(Canvas { width: width + 300, pixels: move(saved) }); }
            if mode == 4 { return Result.Ok(Canvas { width: width + 400, pixels: move(local) }); }
            store_at(pixels, 4, width + 16);
        }
        drop_bytes(saved);
        drop_bytes(spare);
        store_at(pixels, 0, load_at(pixels, 0) + 1);
        return Result.Ok(Canvas { width: width + 6, pixels: copy_bytes(pixels) });
    "#,
    );
    let program = compile(&source);
    let bridge = bridge(&program);
    assert_eq!(
        program
            .yir
            .nodes
            .iter()
            .filter(|n| n.op.instruction == "store_at")
            .count(),
        5
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
        match mode {
            0 => width += 100,
            1 => {
                pixels[1] = width;
                width += 200;
            }
            2 => {
                pixels[1] = width;
                pixels[2] = width + 6;
                pixels[0] += 1;
                width += 6;
            }
            3 => width += 300,
            4 => {
                pixels[3] = width + 10;
                width += 400;
            }
            _ => {
                pixels[3] = width + 10;
                pixels[4] = width + 16;
                pixels[0] += 1;
                width += 6;
            }
        }
        session.event(mode).unwrap();
        assert_eq!(session.state(), pixels, "{mode}");
        assert_eq!(session.scalar_state(), [width]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}

#[test]
fn inline_resource_condition_snapshot_preserves_writes_and_source_names() {
    let source = source("load_at(pixels, 0) == mode && bytes_len(saved) == 48")
        .replace("if mode >= 0 {", "let __nuis_resource_predicate_left: i64 = mode; store_at(pixels, 0, mode); if mode >= 0 {")
        .replace("return Result.Ok(Canvas { width: width + 10", "store_at(pixels, 0, 99); if __nuis_resource_predicate_left != mode { return Result.Err(91); } return Result.Ok(Canvas { width: width + 10");
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
fn inline_resource_condition_errors_and_failed_results_preserve_state_and_release_owners() {
    for allocation_failure in [false, true] {
        let source = source("valid && bytes_len(saved) == 48");
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
fn returned_resource_state_fields_use_ordered_short_circuit_initializers() {
    let source = super::boolean_short_circuit::helpers(
        r#"
        let saved: Bytes = copy_bytes(pixels);
        return Result.Ok(Canvas {
            width: width + word(mode >= 0 && bytes_len(saved) == 48),
            pixels: move(saved)
        });
    "#,
    )
    .replace(
        "mod cpu Main {",
        "mod cpu Main { fn word(flag: bool) -> i64 { if flag { return 9; } return 0; }",
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
    for mode in 0..120 {
        session.event(mode).unwrap();
        assert_eq!(session.scalar_state(), [3 + (mode + 1) * 9]);
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}

#[test]
fn inline_resource_condition_snapshots_share_the_existing_guard_input_budget() {
    for count in [1, 17, 29, 30] {
        let declarations = (0..count)
            .map(|i| format!("let b{i}: bool = width > {i};"))
            .collect::<String>();
        let predicate = (0..count)
            .map(|i| format!("b{i}"))
            .collect::<Vec<_>>()
            .join(" ^ ");
        let source = super::boolean_short_circuit::helpers(&format!(
            r#"
            let saved: Bytes = copy_bytes(pixels);
            {declarations}
            if ({predicate}) && bytes_len(saved) == 48 {{ return Result.Err(8); }}
            return Result.Ok(Canvas {{ width: width, pixels: move(saved) }});
        "#
        ));
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
            assert!(result.unwrap_err().contains("signature metadata"));
        }
    }
}
