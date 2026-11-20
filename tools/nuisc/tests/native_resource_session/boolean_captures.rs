use super::continuations::bridge;
use super::nested_returns::{registration, with_step};
use super::record_helpers::compile;
#[cfg(unix)]
use super::record_helpers::helper_body;
use super::scalar_shadowing::BODY;
use super::*;

fn helpers(body: &str) -> String {
    with_step(body).replace(
        "mod cpu Main {",
        r#"mod cpu Main {
      type Decision = bool;
      struct Choice { reject: bool }
      fn choose(flag: bool) -> Choice { return Choice { reject: flag }; }
    "#,
    )
}

fn source() -> String {
    helpers(
        &BODY
            .replace(
                "let saved: Bytes = copy_bytes(pixels);",
                r#"
          let saved: Bytes = copy_bytes(pixels);
          let early = choose(mode % 2 == 0).reject;
          let reject = choose(mode < 0).reject;
        "#,
            )
            .replace("if mode == 0", "if early")
            .replace("if mode == 2", "if early")
            .replace(
                "store_at(pixels, 1, width);",
                "store_at(pixels, 1, width); if reject { return Result.Err(47); }",
            )
            .replace(
                "store_at(pixels, 2, width);",
                "store_at(pixels, 2, width); if reject { return Result.Err(47); }",
            ),
    )
}

#[test]
#[cfg(unix)]
fn boolean_captures_preserve_four_paths_for_120_events_per_declaration() {
    for declaration in ["let", "let_typed", "const_aliased", "branch_local"] {
        let mut source = source()
            .replace(
                "let early =",
                match declaration {
                    "let_typed" => "let early: bool =",
                    "const_aliased" => "const early: Decision =",
                    _ => "let early =",
                },
            )
            .replace(
                "let reject =",
                match declaration {
                    "let_typed" => "let reject: bool =",
                    "const_aliased" => "const reject: Decision =",
                    _ => "let reject =",
                },
            );
        if declaration == "branch_local" {
            source = source
                .replace("let early = choose(mode % 2 == 0).reject;", "")
                .replace(
                    "store_at(pixels, 1, width);",
                    "let early: bool = choose(mode == 0).reject; store_at(pixels, 1, width);",
                )
                .replace(
                    "store_at(pixels, 2, width);",
                    "const early: Decision = choose(mode == 2).reject; store_at(pixels, 2, width);",
                );
        }
        let program = compile(&source);
        assert_eq!(
            program.yir,
            yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap()
        );
        let bridge = bridge(&program);
        let continuations = program
            .yir
            .functions
            .iter()
            .filter(|f| f.name.starts_with("__nuis_owned_continue_"))
            .collect::<Vec<_>>();
        assert!(!continuations.is_empty());
        for function in continuations {
            assert_eq!(
                function
                    .parameters
                    .iter()
                    .filter(|p| p.ty == "bool")
                    .count(),
                if declaration == "branch_local" { 1 } else { 2 }
            );
            if declaration == "branch_local" {
                assert!(!function.parameters.iter().any(|p| p.name == "early"));
            }
        }
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
        for mode in [0, 1, 2, 3].into_iter().cycle().take(120) {
            match mode {
                0 => width += 110,
                1 => {
                    pixels[1] = width + 10;
                    pixels[0] += 1;
                    width += 1;
                }
                2 => width += 220,
                _ => {
                    pixels[2] = width + 20;
                    pixels[0] += 1;
                    width += 1;
                }
            }
            session.event(mode).unwrap();
            assert_eq!(session.state(), pixels, "{declaration} mode={mode}");
            assert_eq!(session.scalar_state(), [width]);
            assert_eq!(library.live_blobs(), 0);
        }
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
#[cfg(unix)]
fn captured_boolean_snapshot_survives_buffer_writes_and_shared_continuation() {
    let source = helpers(
        r#"
      let saved: Bytes = copy_bytes(pixels);
      let unchanged: bool = choose(load_at(pixels, 0) == 9).reject;
      if mode == 0 {
        store_at(pixels, 0, 99);
        if width == 0 { return Result.Ok(Canvas { width: width, pixels: move(saved) }); }
      }
      if unchanged {
        drop_bytes(saved);
        return Result.Ok(Canvas { width: width + 1, pixels: copy_bytes(pixels) });
      }
      return Result.Ok(Canvas { width: width, pixels: move(saved) });
    "#,
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
    for _ in 0..2 {
        session.event(0).unwrap();
        assert_eq!(session.state(), [99, 9, 9, 9, 9, 9]);
        assert_eq!(session.scalar_state(), [4]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
}

#[test]
#[cfg(unix)]
fn boolean_capture_errors_and_failed_returns_preserve_committed_state_and_owners() {
    let source = source();
    let program = compile(&source);
    let bridge = bridge(&program);
    for fault in [false, true] {
        let mut llvm = bridge.llvm_ir.clone();
        if fault {
            for function in &program.yir.functions {
                if !function.name.starts_with("__nuis_owned_") {
                    continue;
                }
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
            assert_ne!(llvm, bridge.llvm_ir);
        }
        let library = native::Library::build(&source, &program, &llvm);
        for mode in [-7, 0, 1, 2, 3] {
            if !fault && mode != -7 {
                continue;
            }
            let (mut session, _) = NativeResourceSession::open(
                &program.yir,
                &registration(),
                library.checked_binding(&program.yir, &bridge),
                vec![9; 6],
                3,
            )
            .unwrap();
            if !fault {
                session.event(1).unwrap();
            }
            let pixels = session.state().to_vec();
            let scalars = session.scalar_state().to_vec();
            assert!(session.event(mode).unwrap_err().contains(if fault {
                "status 2"
            } else {
                "code=47"
            }));
            assert_eq!(session.state(), pixels);
            assert_eq!(session.scalar_state(), scalars);
            assert_eq!(library.live_blobs(), 0);
            session.close(0).unwrap();
            assert!(session.completion_status().is_err());
            assert_eq!(library.live_blobs(), 0);
        }
    }
}

#[test]
fn guarded_and_tail_boolean_captures_reject_type_and_ownership_drift() {
    let program = compile(&source());
    bridge(&program);
    for tail in [false, true] {
        let instruction = if tail {
            "tail_call_owned_struct_return"
        } else {
            "guard_call_owned_struct_return"
        };
        let original = program
            .yir
            .nodes
            .iter()
            .find(|n| {
                n.op.instruction == instruction && {
                    let call = if tail {
                        yir_core::guarded_owned_call::parse_tail(&n.op.args)
                    } else {
                        yir_core::guarded_owned_call::parse(&n.op.args)
                    }
                    .unwrap();
                    program
                        .yir
                        .functions
                        .iter()
                        .find(|f| f.name == call.callee)
                        .unwrap()
                        .parameters
                        .iter()
                        .any(|p| p.ty == "bool")
                }
            })
            .unwrap();
        let call = if tail {
            yir_core::guarded_owned_call::parse_tail(&original.op.args)
        } else {
            yir_core::guarded_owned_call::parse(&original.op.args)
        }
        .unwrap();
        let helper = program
            .yir
            .functions
            .iter()
            .find(|f| f.name == call.callee)
            .unwrap();
        let index = helper
            .parameters
            .iter()
            .position(|p| p.ty == "bool")
            .unwrap();
        for case in 0..7 {
            let mut changed = program.yir.clone();
            match case {
                0 => {
                    let node = changed
                        .nodes
                        .iter_mut()
                        .find(|n| n.name == original.name)
                        .unwrap();
                    node.op.args[(if tail { 2 } else { 3 }) + index] = call.inputs[1].clone();
                }
                1 => {
                    let node = changed
                        .nodes
                        .iter_mut()
                        .find(|n| n.name == original.name)
                        .unwrap();
                    node.op.args[(if tail { 2 } else { 3 }) + index] = "missing_capture".into();
                }
                2..=4 => {
                    let parameter = &mut changed
                        .functions
                        .iter_mut()
                        .find(|f| f.name == helper.name)
                        .unwrap()
                        .parameters[index];
                    if case == 2 {
                        parameter.ty = "i64".into();
                    } else {
                        parameter.ownership = if case == 3 {
                            yir_core::YirValueOwnership::Owned
                        } else {
                            yir_core::YirValueOwnership::Borrowed
                        };
                    }
                }
                5 => {
                    changed
                        .nodes
                        .iter_mut()
                        .find(|n| n.name == helper.parameters[index].node)
                        .unwrap()
                        .op
                        .instruction = "param_i64".into();
                }
                _ => {
                    let node = changed
                        .nodes
                        .iter_mut()
                        .find(|n| n.name == original.name)
                        .unwrap();
                    let marker = node
                        .op
                        .args
                        .iter()
                        .position(|v| {
                            v.starts_with("transfer_bytes_v1=")
                                || v.starts_with("cleanup_bytes_v1=")
                        })
                        .unwrap();
                    node.op.args[marker + 1] = call.inputs[index].clone();
                }
            }
            assert!(
                yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration())
                    .is_err(),
                "tail={tail} case={case}"
            );
        }
    }
}

#[test]
fn boolean_captures_keep_exported_root_mixed_shadow_and_write_boundaries() {
    let source = source();
    let program = compile(&source);
    let mut changed = program.yir.clone();
    changed
        .functions
        .iter_mut()
        .find(|f| f.name == "step")
        .unwrap()
        .parameters[1]
        .ty = "bool".into();
    assert!(registration().bind(&changed).is_err());
    for (replacement, expected) in [
        (
            "let early: i64 = 0; store_at(pixels, 1, width);",
            "requires matching resolved i64/bool kinds",
        ),
        (
            "early = false; store_at(pixels, 1, width);",
            "cannot rebind captured",
        ),
    ] {
        let modified = source
            .replace("let early =", "let mut early =")
            .replace("store_at(pixels, 1, width);", replacement)
            .replace(
                "drop_bytes(saved);",
                "if early { return Result.Err(9); } drop_bytes(saved);",
            );
        let error = nuisc::pipeline::compile_resource_session_source(
            &modified,
            &registration(),
            &Default::default(),
        )
        .err()
        .unwrap_or_else(|| panic!("unexpected captured-write/shadow admission: {replacement}"));
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn boolean_captures_share_the_guarded_input_budget() {
    for count in [1, 17, 30, 31] {
        let declarations = (0..count)
            .map(|i| format!("let b{i}: bool = width > {i};"))
            .collect::<String>();
        let predicate = (0..count)
            .map(|i| format!("b{i}"))
            .collect::<Vec<_>>()
            .join(" ^ ");
        let source = helpers(&format!(
            r#"
          let saved: Bytes = copy_bytes(pixels);
          {declarations}
          if mode == 0 {{ if {predicate} {{ return Result.Err(8); }} }}
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
        if count <= 30 {
            result.unwrap();
        } else {
            assert!(result.unwrap_err().contains("signature metadata"));
        }
    }
}
