use super::continuations::bridge;
use super::nested_returns::{registration, with_step};
use super::record_helpers::compile;
#[cfg(unix)]
use super::record_helpers::helper_body;
use super::scalar_shadowing::BODY;
use super::*;

const HELPERS: &str = r#"mod cpu Main {
  struct Flags { invalid: bool, early: bool, late: bool }
  struct Validation { flags: Flags, code: i64, amount: i64 }
  fn validate(value: i64, mode: i64) -> Validation {
    return Validation {
      amount: value + 10, code: 40 - mode,
      flags: Flags { late: mode % 2 != 0, early: mode % 2 == 0, invalid: mode < 0 },
    };
  }
"#;

pub(super) fn source() -> String {
    let body = BODY
        .replace(
            "let saved: Bytes = copy_bytes(pixels);",
            r#"
          let saved: Bytes = copy_bytes(pixels);
          if validate(width, mode).flags.invalid { return Result.Err(validate(width, mode).code); }
        "#,
        )
        .replace("width + 10;", "validate(width, mode).amount;")
        .replace("width + 20;", "validate(width, mode).amount + 10;")
        .replace("if mode == 0", "if validate(width, mode).flags.early")
        .replace("if mode == 2", "if !validate(width, mode).flags.late");
    with_step(&body).replace("mod cpu Main {", HELPERS)
}

#[test]
#[cfg(unix)]
fn boolean_record_predicates_preserve_all_owner_paths_for_120_events() {
    let source = source();
    let program = compile(&source);
    assert_eq!(
        program.yir,
        yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap()
    );
    let bridge = bridge(&program);
    let helper = helper_body(&bridge.llvm_ir, "validate");
    assert!(helper.contains("define [5 x i64]"));
    assert!(!helper.contains("@nuis_scheduler_owned_"));
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
fn boolean_validation_error_cleans_held_bytes_and_never_publishes_prefix_writes() {
    let source = source().replace(
        "if validate(width, mode).flags.invalid",
        "store_at(pixels, 5, 777); if validate(width, mode).flags.invalid",
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
    session.event(0).unwrap();
    assert_eq!(session.state(), [9; 6]);
    assert_eq!(session.scalar_state(), [113]);
    assert!(session.event(-7).unwrap_err().contains("code=47"));
    assert_eq!(session.state(), [9; 6]);
    assert_eq!(session.scalar_state(), [113]);
    assert_eq!(library.live_blobs(), 0);
    session.close(0).unwrap();
    assert!(session.completion_status().is_err());
    assert_eq!(library.live_blobs(), 0);
}

#[test]
#[cfg(unix)]
fn boolean_predicate_return_allocation_failures_keep_success_and_error_channels_distinct() {
    let source = source();
    let program = compile(&source);
    let bridge = bridge(&program);
    let mut faulty = bridge.llvm_ir.clone();
    for function in &program.yir.functions {
        if !function.name.starts_with("__nuis_owned_") {
            continue;
        }
        let body = helper_body(&bridge.llvm_ir, &function.name);
        let changed = body.replace(
            "@nuis_scheduler_owned_aggregate_alloc_v1(i64 4)",
            "@nuis_scheduler_owned_aggregate_alloc_v1(i64 0)",
        );
        faulty = faulty.replacen(body, &changed, 1);
    }
    assert_ne!(faulty, bridge.llvm_ir);
    let library = native::Library::build(&source, &program, &faulty);
    for mode in [-7, 0, 1, 2, 3] {
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
    }
}

#[test]
fn boolean_fields_keep_exact_predicate_error_code_and_scalar_operand_types() {
    let program = compile(&source());
    bridge(&program);
    for case in 0..4 {
        let mut changed = program.yir.clone();
        match case {
            0..=2 => {
                let name = ["invalid", "code", "amount"][case];
                let field = changed
                    .nodes
                    .iter_mut()
                    .find(|n| n.op.instruction == "field" && n.op.args[1] == name)
                    .unwrap();
                field.op.instruction = if case == 0 { "const_i64" } else { "const_bool" }.into();
                field.op.args = vec![if case == 0 { "1" } else { "true" }.into()];
            }
            _ => {
                let call = changed
                    .nodes
                    .iter_mut()
                    .find(|n| n.op.instruction == "call_owned_struct")
                    .unwrap();
                call.op.args[1] = call.op.args[1].replace("invalid:bool", "invalid:i64");
            }
        }
        yir_verify::verify_module(&changed).unwrap();
        let error =
            yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration())
                .unwrap_err();
        if case == 0 {
            assert!(error.contains("condition or layout drift"), "{error}");
        }
        if case == 1 {
            assert!(error.contains("Err requires an i64"), "{error}");
        }
    }
}

#[test]
fn boolean_return_fields_do_not_grant_mixed_kind_shadowing() {
    let source = with_step(
        r#"
      if mode == 0 { let width = validate(width, mode).flags.early; }
      return Result.Ok(Canvas { width: width, pixels: copy_bytes(pixels) });
    "#,
    )
    .replace("mod cpu Main {", HELPERS);
    let error = nuisc::pipeline::compile_resource_session_source(
        &source,
        &registration(),
        &Default::default(),
    )
    .err()
    .unwrap();
    assert!(
        error.contains("requires matching resolved i64/bool kinds"),
        "{error}"
    );
    assert!(error.contains("local=bool"), "{error}");
}

#[test]
fn boolean_helper_parameters_use_checked_resource_admission() {
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
      struct Choice { reject: bool }
      fn choose(flag: bool) -> Choice { return Choice { reject: flag }; }
    "#,
    );
    let program = compile(&source);
    let bridge = bridge(&program);
    assert!(bridge
        .llvm_ir
        .contains("define [1 x i64] @nuis_fn_choose(i1"));
}

#[test]
fn boolean_record_leaves_share_the_existing_slot_budget() {
    for count in [1, 17, 64, 65] {
        let fields = (0..count)
            .map(|i| format!("f{i}: bool"))
            .collect::<Vec<_>>()
            .join(",");
        let values = (0..count)
            .map(|i| format!("f{i}: ready"))
            .collect::<Vec<_>>()
            .join(",");
        let source = with_step(&format!(r#"
          let saved: Bytes = copy_bytes(pixels);
          if flags(width).f{} {{ return Result.Err(9); }}
          return Result.Ok(Canvas {{ width: width, pixels: move(saved) }});
        "#, count - 1)).replace("mod cpu Main {", &format!(r#"mod cpu Main {{
          struct Flags {{ {fields} }}
          fn flags(value: i64) -> Flags {{ let ready: bool = value < 0; return Flags {{ {values} }}; }}
        "#));
        let program = compile(&source);
        let result =
            yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration());
        assert_eq!(
            result.is_ok(),
            count <= 64,
            "count={count}: {:?}",
            result.err()
        );
    }
}

#[test]
#[cfg(unix)]
fn boolean_helper_equality_and_inequality_keep_the_full_truth_table() {
    let source = with_step(
        r#"
      let saved: Bytes = copy_bytes(pixels);
      if validate(width, mode).flags.invalid == validate(width, mode).flags.early {
        store_at(pixels, 0, 11);
      } else { store_at(pixels, 0, 22); }
      if validate(width, mode).flags.invalid != validate(width, mode).flags.early {
        store_at(pixels, 1, 33);
      } else { store_at(pixels, 1, 44); }
      drop_bytes(saved);
      return Result.Ok(Canvas { width: width, pixels: copy_bytes(pixels) });
    "#,
    )
    .replace("mod cpu Main {", HELPERS);
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
    for (mode, first, second) in [(-2, 11, 44), (-1, 22, 33), (0, 22, 33), (1, 11, 44)] {
        session.event(mode).unwrap();
        assert_eq!(session.state(), [first, second, 9, 9, 9, 9]);
        assert_eq!(session.scalar_state(), [3]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
}

#[test]
fn boolean_equality_admission_rejects_ordering_and_implicit_integer_coercion() {
    let program = compile(&source());
    let flag = program
        .yir
        .nodes
        .iter()
        .find(|n| n.op.instruction == "field" && n.op.args[1] == "late")
        .unwrap();
    let comparison = program
        .yir
        .nodes
        .iter()
        .find(|n| n.op.instruction == "eq" && n.op.args.contains(&flag.name))
        .unwrap();
    let lane = &program.yir.node_lanes[&comparison.name];
    let integer = program
        .yir
        .nodes
        .iter()
        .find(|n| {
            n.op.instruction == "param_i64" && program.yir.node_lanes.get(&n.name) == Some(lane)
        })
        .unwrap();
    for case in 0..6 {
        let mut changed = program.yir.clone();
        let node = changed
            .nodes
            .iter_mut()
            .find(|n| n.name == comparison.name)
            .unwrap();
        if case < 4 {
            node.op.instruction = ["lt", "le", "gt", "ge"][case].into();
        } else {
            node.op.args[case - 4] = integer.name.clone();
            changed.edges.push(yir_core::Edge {
                kind: yir_core::EdgeKind::Dep,
                from: integer.name.clone(),
                to: comparison.name.clone(),
            });
        }
        yir_verify::verify_module(&changed).unwrap();
        let error =
            yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration())
                .unwrap_err();
        assert!(
            error.contains("unavailable or incompatible inputs"),
            "case={case}: {error}"
        );
    }
}
