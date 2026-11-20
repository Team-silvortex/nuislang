use super::continuations::bridge;
use super::nested_returns::{registration, with_step};
use super::record_helpers::compile;
#[cfg(unix)]
use super::record_helpers::helper_body;
use super::*;

fn source() -> String {
    super::boolean_records::source()
        .replace(
            "fn validate(value: i64, mode: i64) -> Validation {",
            r#"
      fn classify(invalid: bool, amount: i64, early: bool, code: i64, late: bool) -> Validation {
        return Validation {
          amount: amount, code: code,
          flags: Flags { late: late, early: early, invalid: invalid },
        };
      }
      struct Choice { reject: bool }
      fn choose(flag: bool) -> Choice { return Choice { reject: flag }; }
      fn validate(value: i64, mode: i64) -> Validation {
    "#,
        )
        .replace(
            r#"return Validation {
      amount: value + 10, code: 40 - mode,
      flags: Flags { late: mode % 2 != 0, early: mode % 2 == 0, invalid: mode < 0 },
    };"#,
            "return classify(mode < 0, value + 10, mode % 2 == 0, 40 - mode, mode % 2 != 0);",
        )
        .replace(
            "if validate(width, mode).flags.invalid",
            "if choose(validate(width, mode).flags.invalid).reject",
        )
}

#[test]
#[cfg(unix)]
fn mixed_boolean_parameters_preserve_four_owner_paths_for_120_events() {
    let source = source();
    let program = compile(&source);
    assert_eq!(
        program.yir,
        yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap()
    );
    let bridge = bridge(&program);
    let signature = helper_body(&bridge.llvm_ir, "classify")
        .lines()
        .next()
        .unwrap();
    assert_eq!(signature.matches("i1 ").count(), 3, "{signature}");
    assert_eq!(signature.matches("i64 ").count(), 2, "{signature}");
    for name in ["choose", "classify", "validate"] {
        assert!(!helper_body(&bridge.llvm_ir, name).contains("@nuis_scheduler_owned_"));
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
        assert_eq!(session.state(), pixels, "mode={mode}");
        assert_eq!(session.scalar_state(), [width]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}

#[test]
#[cfg(unix)]
fn boolean_parameters_keep_application_and_allocation_failures_atomic() {
    let source = source().replace(
        "if choose(validate(width, mode).flags.invalid).reject",
        "store_at(pixels, 5, 777); if choose(validate(width, mode).flags.invalid).reject",
    );
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
            assert!(session.event(mode).unwrap_err().contains(if fault {
                "status 2"
            } else {
                "code=47"
            }));
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
fn boolean_parameter_calls_reject_wrong_kinds_arity_and_unready_values() {
    let program = compile(&source());
    bridge(&program);
    for case in 0..8 {
        let mut changed = program.yir.clone();
        let callee = if case < 4 || case == 7 {
            "choose"
        } else {
            "classify"
        };
        let call = changed
            .nodes
            .iter()
            .position(|n| n.op.instruction == "call_owned_struct" && n.op.args[0] == callee)
            .unwrap();
        match case {
            0 => {
                let buffer = registration().bind(&changed).unwrap()[1].parameters[0]
                    .node
                    .clone();
                changed.nodes[call].op.args[2] = buffer;
            }
            1 => {
                changed.nodes[call].op.args.pop();
            }
            2 => {
                let arg = changed.nodes[call].op.args[2].clone();
                changed.nodes[call].op.args.push(arg);
            }
            3 => {
                changed.nodes[call].op.args[2] = "not_yet_ready".into();
            }
            4 => {
                changed.nodes[call].op.args.swap(2, 3);
            }
            5..=6 => {
                let arg = changed.nodes[call].op.args[if case == 5 { 2 } else { 3 }].clone();
                let operand = changed.nodes.iter_mut().find(|n| n.name == arg).unwrap();
                operand.op.instruction = if case == 5 { "const_i64" } else { "const_bool" }.into();
                operand.op.args = vec![if case == 5 { "1" } else { "true" }.into()];
                yir_verify::verify_module(&changed).unwrap();
            }
            7 => {
                let arg = changed.nodes[call].op.args[2].clone();
                let operand = changed.nodes.iter_mut().find(|n| n.name == arg).unwrap();
                operand.op.instruction = "const_i64".into();
                operand.op.args = vec!["1".into()];
                yir_verify::verify_module(&changed).unwrap();
            }
            _ => unreachable!(),
        }
        assert!(
            yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration())
                .is_err(),
            "case={case}"
        );
    }
}

#[test]
fn boolean_parameters_share_the_existing_argument_budget() {
    for count in [1, 17, 64, 65] {
        let parameters = (0..count)
            .map(|i| format!("b{i}: bool"))
            .collect::<Vec<_>>()
            .join(",");
        let arguments = (0..count)
            .map(|i| if i % 2 == 0 { "true" } else { "false" })
            .collect::<Vec<_>>()
            .join(",");
        let source = with_step(&format!(
            r#"
          let saved: Bytes = copy_bytes(pixels);
          if choose({arguments}).reject {{ return Result.Err(8); }}
          return Result.Ok(Canvas {{ width: width, pixels: move(saved) }});
        "#
        ))
        .replace(
            "mod cpu Main {",
            &format!(
                r#"mod cpu Main {{
          struct Choice {{ reject: bool }}
          fn choose({parameters}) -> Choice {{ return Choice {{ reject: b0 }}; }}
        "#
            ),
        );
        let program = compile(&source);
        let result =
            yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration());
        if count <= 64 {
            result.unwrap();
        } else {
            assert!(result.unwrap_err().contains("function/argument bounds"));
        }
    }
}

#[test]
#[cfg(unix)]
fn direct_boolean_comparison_and_alias_parameters_execute_both_values() {
    let source = with_step(
        r#"
      let saved: Bytes = copy_bytes(pixels);
      if choose(width < 1).reject { return Result.Err(8); }
      return Result.Ok(Canvas { width: width, pixels: move(saved) });
    "#,
    )
    .replace(
        "mod cpu Main {",
        r#"mod cpu Main {
      type Decision = bool;
      struct Choice { reject: bool }
      fn choose(flag: Decision) -> Choice { return Choice { reject: flag }; }
    "#,
    );
    let program = compile(&source);
    let bridge = bridge(&program);
    let library = native::Library::build(&source, &program, &bridge.llvm_ir);
    for width in [0, 3] {
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            width,
        )
        .unwrap();
        let result = session.event(0);
        if width == 0 {
            assert!(result.unwrap_err().contains("code=8"));
        } else {
            result.unwrap();
        }
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(session.scalar_state(), [width]);
        assert_eq!(library.live_blobs(), 0);
        session.close(0).unwrap();
        assert_eq!(session.completion_status().is_err(), width == 0);
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
fn boolean_local_captures_use_typed_resource_outlines() {
    let source = with_step(
        r#"
      let saved: Bytes = copy_bytes(pixels);
      let reject: bool = choose(width < 1).reject;
      if mode == 0 {
        if reject { return Result.Err(8); }
      }
      return Result.Ok(Canvas { width: width, pixels: move(saved) });
    "#,
    )
    .replace(
        "mod cpu Main {",
        r#"mod cpu Main {
      struct Choice { reject: bool }
      fn choose(flag: bool) -> Choice { return Choice { reject: flag }; }
    "#,
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
    let bridge = result.unwrap();
    assert!(bridge
        .llvm_ir
        .lines()
        .any(|line| line.starts_with("define i64 @nuis_fn___nuis_owned_")
            && line.contains("i1 %arg")));
}

#[test]
fn boolean_parameter_signatures_reject_other_kinds_ownership_and_declaration_drift() {
    let program = compile(&source());
    bridge(&program);
    for case in 0..8 {
        let mut changed = program.yir.clone();
        let helper = changed
            .functions
            .iter_mut()
            .find(|f| f.name == "choose")
            .unwrap();
        let parameter = &mut helper.parameters[0];
        let node_name = parameter.node.clone();
        match case {
            0 => parameter.ownership = yir_core::YirValueOwnership::Owned,
            1 => parameter.ownership = yir_core::YirValueOwnership::Borrowed,
            2..=5 => parameter.ty = ["i32", "f64", "Bytes", "ref Buffer"][case - 2].into(),
            _ => {}
        }
        if case >= 6 {
            let node = changed
                .nodes
                .iter_mut()
                .find(|n| n.name == node_name)
                .unwrap();
            if case == 6 {
                node.op.instruction = "param_i64".into();
            } else {
                node.op.args = vec!["1".into()];
            }
        }
        assert!(
            yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration())
                .is_err(),
            "case={case}"
        );
    }
}

#[test]
#[cfg(unix)]
fn boolean_arguments_observe_buffer_reads_before_later_writes() {
    let source = with_step(
        r#"
      let saved: Bytes = copy_bytes(pixels);
      if choose(load_at(pixels, 0) == 9).reject {
        store_at(pixels, 0, 99);
        return Result.Ok(Canvas { width: width + 1, pixels: copy_bytes(pixels) });
      }
      return Result.Ok(Canvas { width: width, pixels: move(saved) });
    "#,
    )
    .replace(
        "mod cpu Main {",
        r#"mod cpu Main {
      struct Choice { reject: bool }
      fn choose(flag: bool) -> Choice { return Choice { reject: flag }; }
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
    assert_eq!(library.live_blobs(), 0);
}
