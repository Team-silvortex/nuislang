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
            let mut llvm = bridge.llvm_ir.clone();
            if fail {
                for f in &program.yir.functions {
                    if f.name.starts_with("__nuis_owned_") {
                        let body = helper_body(&bridge.llvm_ir, &f.name);
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
fn record_conditions_do_not_yet_admit_local_record_operands_in_returned_state() {
    let source = source("let saved: Bytes = copy_bytes(pixels); let prior = packet(0, mode == 0, 0); return Result.Ok(Canvas { width: word(prior.flags.valid && bytes_len(saved) == 48), pixels: move(saved) });");
    let error = nuisc::pipeline::compile_resource_session_source(
        &source,
        &registration(),
        &Default::default(),
    )
    .err()
    .expect("returned State needs local-record operand admission");
    assert!(
        error.contains("short-circuit RHS requires guarded lowering"),
        "{error}"
    );
}
