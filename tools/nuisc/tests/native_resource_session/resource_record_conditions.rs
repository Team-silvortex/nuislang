use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::{compile, helper_body};
use super::resource_record_initializers::source;
use super::*;

#[test]
fn record_conditions_preserve_values_completed_reads_and_binding_forms_for_120_events() {
    for (case, condition) in [
        "prior.flags.valid && bytes_len(saved) == 48",
        "!(prior.flags.valid || bytes_len(saved) == 0)",
        "(prior.flags.valid && bytes_len(saved) == 48) ^ (prior.left < 3)",
        "choose(prior.flags.valid && load_at(pixels, prior.right - 6) == 99).value",
        "prior.flags.valid && (bytes_len(saved) == prior.right * 8 || load_at(pixels, 99) > 0)",
        "(prior.flags.valid && buffer_len(pixels) == prior.right) == (prior.left < 3)",
        "word(prior.flags.valid && bytes_len(saved) == 48) < load_at(pixels, 1)",
        "load_at(pixels, word(prior.flags.valid && bytes_len(saved) == 48) / 9) == 9",
    ]
    .into_iter()
    .enumerate()
    {
        for declaration in ["let prior: Alias", "let prior", "const prior: Packet"] {
            let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); store_at(pixels, 0, mode); let outer = envelope(load_at(pixels, 0), mode % 2 == 0, buffer_len(pixels)); {declaration} = outer.packet; store_at(pixels, 0, 99); if {condition} {{ let local: Bytes = copy_bytes(pixels); return Result.Ok(Canvas {{ width: width + 10, pixels: move(saved) }}); }} return Result.Ok(Canvas {{ width: width + 1, pixels: move(saved) }});"));
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
                let even = mode % 2 == 0;
                let selected = match case {
                    1 | 6 => !even,
                    2 => even ^ (mode < 3),
                    5 => even == (mode < 3),
                    _ => even,
                };
                width += if selected { 10 } else { 1 };
                session.event(mode).unwrap();
                assert_eq!(
                    session.scalar_state(),
                    [width],
                    "{case}/{declaration}/{mode}"
                );
                assert_eq!(session.state(), [9; 6]);
                assert_eq!(library.live_blobs(), 0);
            }
            session.close(0).unwrap();
            session.completion_status().unwrap();
        }
    }
}

#[test]
fn record_conditions_keep_both_arms_local_owners_and_shared_fallthrough() {
    let source = source(
        r#"
        let saved: Bytes = copy_bytes(pixels);
        let spare: Bytes = copy_bytes(pixels);
        let prior = packet(0, mode < 3, 0);
        if prior.flags.valid && bytes_len(saved) == 48 {
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
        drop_bytes(saved); drop_bytes(spare);
        store_at(pixels, 0, load_at(pixels, 0) + 1);
        return Result.Ok(Canvas { width: width + 6, pixels: copy_bytes(pixels) });
    "#,
    );
    let program = compile(&source);
    assert_eq!(
        program
            .yir
            .nodes
            .iter()
            .filter(|n| n.op.instruction == "store_at")
            .count(),
        5
    );
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
        assert_eq!(session.scalar_state(), [width]);
        assert_eq!(session.state(), pixels);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
}

#[test]
fn record_conditions_keep_branch_local_errors_and_failed_allocations_atomic() {
    for error in [false, true] {
        let then_value = if error {
            "Result.Err(26)"
        } else {
            "Result.Ok(Canvas { width: 9, pixels: move(saved) })"
        };
        let else_value = if error {
            "Result.Err(17)"
        } else {
            "Result.Ok(Canvas { width: 0, pixels: move(saved) })"
        };
        let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); if mode >= 0 {{ let branch: Bytes = copy_bytes(pixels); let prior = packet(0, mode == 0, 0); store_at(pixels, 0, 99); if prior.flags.valid && bytes_len(saved) == 48 {{ return {then_value}; }} return {else_value}; }} return Result.Ok(Canvas {{ width: width + 1, pixels: move(saved) }});"));
        let program = compile(&source);
        let bridge = bridge(&program);
        for fail in [false, true] {
            let llvm = result_allocation_ir(&program, &bridge.llvm_ir, fail);
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
                if fail || error {
                    let text = result.unwrap_err();
                    assert!(
                        text.contains(if fail {
                            "status 2"
                        } else if mode == 0 {
                            "code=26"
                        } else {
                            "code=17"
                        }),
                        "{text}"
                    );
                    if fail {
                        assert!(!text.contains("application error"));
                    }
                    assert_eq!(session.scalar_state(), [3]);
                } else {
                    result.unwrap();
                    assert_eq!(session.scalar_state(), [if mode == 0 { 9 } else { 0 }]);
                }
                assert_eq!(session.state(), [9; 6]);
                assert_eq!(library.live_blobs(), 0);
                session.close(0).unwrap();
                assert_eq!(session.completion_status().is_err(), fail || error);
            }
        }
    }
}

#[test]
fn record_conditions_keep_borrowed_only_registered_identity() {
    let source = source("let prior = envelope(0, mode == 0, 0); if prior.packet.flags.valid && buffer_len(pixels) == 6 { return Result.Err(9); } return Result.Err(17);")
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
    for (mode, code) in [(0, 9), (1, 17)] {
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
fn record_conditions_share_scalar_paths_without_widening_capture_bounds() {
    for (nested, count, limit) in [
        (false, 1, 29),
        (false, 29, 29),
        (false, 30, 29),
        (true, 17, 27),
        (true, 27, 27),
        (true, 28, 27),
    ] {
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
        let condition = if nested {
            format!("tally({args}, mode == 0 && buffer_len(pixels) == 6, prior.f0) > 0")
        } else {
            let comparisons = (0..count)
                .map(|i| format!("(prior.f{i} > 0)"))
                .collect::<Vec<_>>()
                .join(" ^ ");
            format!("(({comparisons}) ^ (prior.f0 < 0)) && buffer_len(pixels) == 6")
        };
        let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let prior = Wide {{ {values} }}; if {condition} {{ return Result.Ok(Canvas {{ width: 9, pixels: move(saved) }}); }} return Result.Ok(Canvas {{ width: 0, pixels: move(saved) }});"))
            .replace("mod cpu Main {", &format!("mod cpu Main {{ struct Wide {{ {fields} }} fn tally({fields}, flag: bool, again: i64) -> i64 {{ return f0 + f{} + word(flag) + again; }}", count - 1));
        let result = nuisc::pipeline::compile_resource_session_source(
            &source,
            &registration(),
            &Default::default(),
        )
        .and_then(|p| p.emit_llvm())
        .and_then(|p| {
            yir_lower_llvm::native_resource_session::emit_registered(&p.yir, &registration())?;
            Ok(p)
        });
        if count <= limit {
            let program = result.unwrap_or_else(|e| panic!("{nested}/{count}: {e}"));
            if count == limit {
                let maximum = program
                    .yir
                    .nodes
                    .iter()
                    .filter(|n| n.op.instruction == "guard_call_owned_struct_return")
                    .map(|n| {
                        yir_core::guarded_owned_call::parse(&n.op.args)
                            .unwrap()
                            .inputs
                            .len()
                    })
                    .max();
                assert_eq!(
                    maximum,
                    Some(33),
                    "generated bool/call temporaries count as inputs"
                );
            }
        } else {
            let e = result
                .err()
                .expect("the extra capture must exceed the guard bound");
            assert!(e.contains("signature metadata"), "{e}");
        }
    }
}

#[test]
fn local_record_return_fields_keep_selected_reads_and_terminal_cleanup() {
    for (prefix, fields, base) in [
        ("", "width: word(prior.flags.valid && bytes_len(saved) == 48), pixels: move(saved)", 0),
        ("let spare: Bytes = copy_bytes(pixels);", "pixels: move(saved), width: prior.left + word(prior.flags.valid && bytes_len(spare) == 48 && load_at(pixels, mode * 99) == 17)", 9),
    ] {
        let source = source(&format!("let saved: Bytes = copy_bytes(pixels); {prefix} let prior = packet(load_at(pixels, 0), mode == 0, 0); store_at(pixels, 0, 17); return Result.Ok(Canvas {{ {fields} }});"));
        let program = compile(&source);
        let bridge = bridge(&program);
        let library = native::Library::build(&source, &program, &bridge.llvm_ir);
        let (mut session, _) = NativeResourceSession::open(&program.yir, &registration(),
            library.checked_binding(&program.yir, &bridge), vec![9; 6], 3).unwrap();
        for mode in (0..2).cycle().take(120) {
            session.event(mode).unwrap();
            assert_eq!(session.scalar_state(), [base + if mode == 0 { 9 } else { 0 }]);
            assert_eq!(session.state(), [9; 6]);
            assert_eq!(library.live_blobs(), 0);
        }
        session.close(0).unwrap();
        session.completion_status().unwrap();
    }
}

#[test]
fn local_record_return_fields_still_reject_reads_after_move() {
    let source = source("let saved: Bytes = copy_bytes(pixels); let prior = packet(0, false, 0); return Result.Ok(Canvas { pixels: move(saved), width: word(prior.flags.valid && bytes_len(saved) == 48) });");
    let error = nuisc::pipeline::compile_resource_session_source(
        &source,
        &registration(),
        &Default::default(),
    )
    .err()
    .expect("even an unselected resource read must respect field move order");
    assert!(
        error.contains("moved") || error.contains("consumed"),
        "{error}"
    );
}

#[test]
fn checked_error_codes_retain_local_record_values_and_selected_reads() {
    let cases: [(&str, [i64; 2]); 3] = [
        (
            r#"let saved: Bytes = copy_bytes(pixels);
            let prior = packet(0, mode == 0, 0);
            return Result.Err(word(prior.flags.valid && bytes_len(saved) == 48));"#,
            [9, 0],
        ),
        (
            r#"let saved: Bytes = copy_bytes(pixels);
            let spare: Bytes = copy_bytes(pixels);
            let prior = packet(load_at(pixels, 0), mode == 0, 0);
            store_at(pixels, 0, 17);
            return Result.Err(prior.left - 5000000000 + word(prior.flags.valid &&
                bytes_len(saved) == 48 && load_at(pixels, mode * 99) == 17));"#,
            [-4_999_999_982, -4_999_999_991],
        ),
        (
            r#"if mode >= 0 {
                let prior = envelope(0, mode == 0, 0);
                return Result.Err(word(prior.packet.flags.valid && load_at(pixels, mode * 99) == 9));
            }
            return Result.Err(17);"#,
            [9, 0],
        ),
    ];
    assert_checked_errors(cases);
}

fn assert_checked_errors(cases: impl IntoIterator<Item = (&'static str, [i64; 2])>) {
    for (body, expected) in cases {
        let source = source(body);
        let program = compile(&source);
        let bridge = bridge(&program);
        for fail in [false, true] {
            let llvm = result_allocation_ir(&program, &bridge.llvm_ir, fail);
            let library = native::Library::build(&source, &program, &llvm);
            for (mode, code) in expected.into_iter().enumerate() {
                let (mut session, _) = NativeResourceSession::open(
                    &program.yir,
                    &registration(),
                    library.checked_binding(&program.yir, &bridge),
                    vec![9; 6],
                    3,
                )
                .unwrap();
                let err = session.event(mode as i64).unwrap_err();
                if fail {
                    assert!(
                        err.contains("status 2") && !err.contains("application error"),
                        "{err}"
                    );
                } else {
                    assert!(
                        err.contains(&format!(
                            "native resource application error (role=event, code={code})"
                        )),
                        "{err}"
                    );
                }
                assert_eq!(session.scalar_state(), [3]);
                assert_eq!(session.state(), [9; 6]);
                assert_eq!(library.live_blobs(), 0);
                session.close(0).unwrap();
                assert!(session.completion_status().is_err());
            }
        }
    }
}

fn result_allocation_ir(
    program: &nuisc::pipeline::PipelineArtifacts,
    original: &str,
    fail: bool,
) -> String {
    let mut llvm = original.to_owned();
    if fail {
        for f in &program.yir.functions {
            if f.name.starts_with("__nuis_owned_") {
                let body = helper_body(original, &f.name);
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
        assert_ne!(llvm, original);
    }
    llvm
}

#[test]
fn scalar_projections_survive_an_intervening_resource_guard_in_checked_errors() {
    let cases: [(&str, [i64; 2]); 3] = [
        ("let saved: Bytes = copy_bytes(pixels); let prior = packet(mode, true, 0); let valid = mode == 0 && bytes_len(saved) == 48; return Result.Err(prior.left + word(valid));", [9, 1]),
        (r#"let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels);
            let prior = envelope(load_at(pixels, 0), mode == 0, 0);
            store_at(pixels, 0, 17);
            let valid = mode == 0 && load_at(pixels, mode * 99) == 17;
            return Result.Err(prior.packet.left + prior.packet.left +
                word(valid && prior.packet.flags.valid && bytes_len(spare) == 48));"#, [27, 18]),
        (r#"let prior = packet(mode - 5000000000, false, 0);
            const valid: bool = mode != 0 || load_at(pixels, mode * 99) == 9;
            return Result.Err(prior.left + word(valid));"#, [-4_999_999_991, -4_999_999_990]),
    ];
    assert_checked_errors(cases);
}

#[test]
fn guarded_error_capture_cost_depends_on_distinct_used_leaves() {
    let fields = (0..40)
        .map(|i| format!("f{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let values = (0..40)
        .map(|i| format!("f{i}: mode + {i}"))
        .collect::<Vec<_>>()
        .join(", ");
    for distinct in [false, true] {
        let sum = (0..40)
            .map(|i| format!("prior.f{}", if distinct { i } else { 39 }))
            .collect::<Vec<_>>()
            .join(" + ");
        let body = format!("let saved: Bytes = copy_bytes(pixels); let prior = Wide {{ {values} }};
            let valid = mode == 0 && bytes_len(saved) == 48; return Result.Err({sum} + word(valid));");
        let source = source(&body).replace(
            "mod cpu Main {",
            &format!("mod cpu Main {{ struct Wide {{ {fields} }}"),
        );
        let result = nuisc::pipeline::compile_resource_session_source(
            &source,
            &registration(),
            &Default::default(),
        )
        .and_then(|program| program.emit_llvm())
        .and_then(|program| {
            yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration())
        });
        if distinct {
            let error = result
                .err()
                .expect("forty scalar captures exceed the existing input bound");
            assert!(
                error.contains("guarded owned call has invalid arity"),
                "{error}"
            );
        } else {
            result.unwrap_or_else(|error| {
                panic!("forty uses of one field need one capture: {error}")
            });
        }
    }
}

#[test]
fn returned_state_projections_cross_a_resource_guard_without_moving_owners_early() {
    for (body, widths, pixel) in [
        ("let saved: Bytes = copy_bytes(pixels); let prior = packet(mode, true, 0); let valid = mode == 0 && bytes_len(saved) == 48; return Result.Ok(Canvas { width: prior.left + word(valid), pixels: move(saved) });", [9, 1], 9),
        (r#"let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels);
            let prior = envelope(load_at(pixels, 0), mode == 0, 0);
            store_at(pixels, 0, 17);
            let valid = mode == 0 && load_at(pixels, mode * 99) == 17;
            return Result.Ok(Canvas { pixels: move(saved), width: prior.packet.left + prior.packet.left
                + word(valid && prior.packet.flags.valid && bytes_len(spare) == 48) });"#, [27, 18], 9),
        (r#"let prior = packet(load_at(pixels, 0), true, 0); store_at(pixels, 0, 17);
            const valid: bool = mode != 0 || load_at(pixels, mode * 99) == 17;
            return Result.Ok(Canvas { width: prior.left + word(valid), pixels: copy_bytes(pixels) });"#, [18, 18], 17),
    ] {
        let source = source(body);
        let program = compile(&source);
        let bridge = bridge(&program);
        for fail in [false, true] {
            let llvm = result_allocation_ir(&program, &bridge.llvm_ir, fail);
            let library = native::Library::build(&source, &program, &llvm);
            for (mode, width) in widths.into_iter().enumerate() {
                let (mut session, _) = NativeResourceSession::open(&program.yir, &registration(),
                    library.checked_binding(&program.yir, &bridge), vec![9; 6], 3).unwrap();
                let result = session.event(mode as i64);
                if fail {
                    assert!(result.unwrap_err().contains("status 2"));
                    assert_eq!(session.scalar_state(), [3]);
                    assert_eq!(session.state(), [9; 6]);
                } else {
                    result.unwrap();
                    assert_eq!(session.scalar_state(), [width]);
                    assert_eq!(session.state(), [pixel, 9, 9, 9, 9, 9]);
                }
                assert_eq!(library.live_blobs(), 0);
                session.close(0).unwrap();
                assert_eq!(session.completion_status().is_err(), fail);
            }
        }
    }
}

#[test]
fn cross_guard_store_with_a_lazy_resource_value_is_not_made_eager() {
    assert_stored_states([
        (
            r#"let saved: Bytes = copy_bytes(pixels); let prior = packet(mode, false, 0);
            let valid = mode == 0 && bytes_len(saved) == 48;
            store_at(pixels, 0, word(valid && load_at(pixels, mode * 99) == 9));
            return Result.Ok(Canvas { width: prior.left + load_at(pixels, 0), pixels: move(saved) });"#,
            1,
            [9, 1],
            [9, 9],
        ),
        (
            r#"let prior = packet(load_at(pixels, 0), true, 0);
            store_at(pixels, 0, load_at(pixels, 0) + 1);
            store_at(pixels, load_at(pixels, 1) - 9,
                load_at(pixels, 0) + prior.left + word(mode != 0 || load_at(pixels, mode * 99) == 10));
            store_at(pixels, 0, load_at(pixels, 0) * 2);
            return Result.Ok(Canvas { width: prior.left + load_at(pixels, 0), pixels: copy_bytes(pixels) });"#,
            3,
            [65, 65],
            [56, 56],
        ),
    ]);
}

#[test]
fn cross_guard_store_with_a_lazy_resource_index_updates_only_the_selected_pixel() {
    assert_store_case(
        r#"let saved: Bytes = copy_bytes(pixels); let prior = packet(mode, false, 0);
        let valid = mode == 0 && bytes_len(saved) == 48;
        store_at(pixels, word(valid && load_at(pixels, mode * 99) == 9) / 9, 17);
        return Result.Ok(Canvas { width: prior.left + load_at(pixels, 0) + 2 * load_at(pixels, 1), pixels: move(saved) });"#,
        1,
        [(43, [9; 6]), (36, [9; 6])],
    );
    assert_store_case(
        r#"let prior = packet(load_at(pixels, 0), true, 0);
        store_at(pixels, word(mode % 2 == 0 && load_at(pixels, mode % 2 * 99) == prior.left) / 9,
            load_at(pixels, (mode + 1) % 2) + 1 + word(mode >= 2 || load_at(pixels, mode / 2 * 99) == 8));
        return Result.Ok(Canvas { width: prior.left + load_at(pixels, 0) + 2 * load_at(pixels, 1), pixels: copy_bytes(pixels) });"#,
        1,
        [
            (38, [9, 10, 9, 9, 9, 9]),
            (37, [10, 9, 9, 9, 9, 9]),
            (56, [9, 19, 9, 9, 9, 9]),
            (46, [19, 9, 9, 9, 9, 9]),
        ],
    );
}

#[test]
fn guarded_store_native_trace_finishes_index_before_starting_value() {
    let source = source(r#"
        store_at(pixels,
            probe(2, word(mode % 2 == 0 && probe(1, load_at(pixels, mode % 2 * 99)) == 9) / 9),
            probe(4, 1 + word(probe(3, mode) >= 2 || probe(5, load_at(pixels, mode / 2 * 99)) == 9)));
        return Result.Ok(Canvas { width: probe(0, 0), pixels: copy_bytes(pixels) });"#)
        .replace("mod cpu Main {", "mod cpu Main { @noinline fn probe(tag: i64, x: i64) -> i64 { return (x + tag) - tag; }");
    let program = compile(&source);
    let bridge = bridge(&program);
    // Only the identity probe is instrumented. The actual compiled guards, loads,
    // address calculation and store are unchanged. Tag zero reports and resets.
    let body = helper_body(&bridge.llvm_ir, "probe");
    let llvm = bridge.llvm_ir.replacen(
        body,
        r#"define i64 @nuis_fn_probe(i64 %tag, i64 %x) {
entry:
  %old = load volatile i64, ptr @store_test_trace
  %shift = mul i64 %old, 10
  %append = add i64 %shift, %tag
  %report = icmp eq i64 %tag, 0
  %next = select i1 %report, i64 0, i64 %append
  store volatile i64 %next, ptr @store_test_trace
  %result = select i1 %report, i64 %old, i64 %x
  ret i64 %result"#,
        1,
    ) + "\n@store_test_trace = internal global i64 0\n";
    let library = native::Library::build(&source, &program, &llvm);
    // 1 = selected index read, 2 = index done, 3 = value starts,
    // 5 = selected value read, 4 = value done.
    for (mode, trace) in [12354, 2354, 1234, 234].into_iter().enumerate() {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        session.event(mode as i64).unwrap();
        assert_eq!(session.scalar_state(), [trace], "mode {mode}");
        let mut pixels = [9; 6];
        pixels[(mode + 1) % 2] = 10;
        assert_eq!(session.state(), pixels);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        session.completion_status().unwrap();
    }
}

#[test]
fn cross_guard_store_followed_by_a_projected_readback_local_remains_outside() {
    let source = source(
        r#"let saved: Bytes = copy_bytes(pixels); let prior = packet(mode, false, 0);
        let valid = mode == 0 && bytes_len(saved) == 48;
        store_at(pixels, word(valid && load_at(pixels, mode * 99) == 9) / 9, 17);
        let observed: i64 = load_at(pixels, 0) + prior.left;
        return Result.Ok(Canvas { width: observed, pixels: move(saved) });"#,
    );
    let error = nuisc::pipeline::compile_resource_session_source(
        &source,
        &registration(),
        &Default::default(),
    )
    .and_then(|program| program.emit_llvm())
    .and_then(|program| {
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration())
    })
    .err()
    .expect("a nonterminal scalar declaration is not a checked return tail");
    assert!(error.contains("prior"), "{error}");
}

#[test]
fn guarded_stores_execute_before_projected_error_and_cleanup() {
    assert_checked_errors([
        (
            r#"let saved: Bytes = copy_bytes(pixels);
        let prior = packet(load_at(pixels, 0), false, 0);
        let valid = mode == 0 && bytes_len(saved) == 48;
        store_at(pixels, prior.right, load_at(pixels, 0) + 1 + word(valid));
        return Result.Err(prior.left + load_at(pixels, 0));"#,
            [28, 19],
        ),
        (
            r#"let saved: Bytes = copy_bytes(pixels); let prior = packet(load_at(pixels, 0), false, 0);
        store_at(pixels, 0, prior.left + word(mode == 0 && load_at(pixels, mode * 99) == 9 && bytes_len(saved) == 48));
        return Result.Err(prior.left + load_at(pixels, 0));"#,
            [27, 18],
        ),
    ]);
}

#[test]
fn guarded_stores_precede_projected_returns_once_on_both_edges() {
    assert_stored_states([
        ("let saved: Bytes = copy_bytes(pixels); let prior = packet(mode, true, 0); let valid = mode == 0 && bytes_len(saved) == 48; store_at(pixels, 0, word(valid)); return Result.Ok(Canvas { width: prior.left + word(valid), pixels: move(saved) });", 1, [9, 1], [9, 9]),
        (r#"let spare: Bytes = copy_bytes(pixels);
            let prior = packet(load_at(pixels, 0), false, buffer_len(pixels) - 6);
            let valid = mode == 0 && load_at(pixels, mode * 99) == 9;
            store_at(pixels, prior.right, load_at(pixels, 0) + 1 + word(valid));
            store_at(pixels, prior.right, load_at(pixels, 0) * 2 + prior.left);
            return Result.Ok(Canvas { width: prior.left + load_at(pixels, 0), pixels: copy_bytes(pixels) });"#,
            2, [56, 38], [47, 29]),
    ]);
}

fn assert_stored_states(
    cases: impl IntoIterator<Item = (&'static str, usize, [i64; 2], [i64; 2])>,
) {
    for (body, stores, widths, pixels) in cases {
        assert_store_case(
            body,
            stores,
            [0, 1].map(|m| (widths[m], [pixels[m], 9, 9, 9, 9, 9])),
        );
    }
}

fn assert_store_case<const N: usize>(body: &str, stores: usize, expected: [(i64, [i64; 6]); N]) {
    let source = source(body);
    let program = compile(&source);
    assert_eq!(
        program
            .yir
            .nodes
            .iter()
            .filter(|node| node.op.instruction == "store_at")
            .count(),
        stores
    );
    let bridge = bridge(&program);
    for fail in [false, true] {
        let llvm = result_allocation_ir(&program, &bridge.llvm_ir, fail);
        let library = native::Library::build(&source, &program, &llvm);
        for (mode, (width, pixels)) in expected.into_iter().enumerate() {
            let (mut session, _) = NativeResourceSession::open(
                &program.yir,
                &registration(),
                library.checked_binding(&program.yir, &bridge),
                vec![9; 6],
                3,
            )
            .unwrap();
            let result = session.event(mode as i64);
            if fail {
                assert!(result.unwrap_err().contains("status 2"));
                assert_eq!(session.scalar_state(), [3]);
                assert_eq!(session.state(), [9; 6]);
            } else {
                result.unwrap();
                assert_eq!(session.scalar_state(), [width]);
                assert_eq!(session.state(), pixels);
            }
            assert_eq!(library.live_blobs(), 0);
            session.close(0).unwrap();
            assert_eq!(session.completion_status().is_err(), fail);
        }
    }
}
