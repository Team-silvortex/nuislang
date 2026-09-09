use super::*;
use scalar_helpers::{assert_traps, with_helpers};

const HELPERS: &str = r#"
  fn leaf(value: i64) -> i64 { return value; }
  fn agrees(value: i64) -> bool {
    if value < 0 { return false; }
    if value == 0 { return true; }
    return leaf(value) > 0;
  }
  fn candidate(seed: i64, index: i64) -> i64 {
    const scale: i64 = 3;
    if agrees(index) {
      if index == 0 { return leaf(seed); }
      let checked: i64 = 21 / index;
      if checked < 0 { return 0; }
    } else { return 1 / index; }
    let value: i64 = seed + leaf(index) * scale;
    return leaf(value);
  }
"#;

fn source(helpers: &str) -> String {
    with_helpers(
        &SOURCE.replace("seed + index * 3", "candidate(seed, index)"),
        helpers,
    )
}

#[test]
fn scalar_control_nested_early_returns_stay_inside_callees() {
    let source = source(HELPERS);
    check_execution(&source, 37);
    check_execution(
        &source.replace("let index: i64 = 0;", "let index: i64 = 8;"),
        8,
    );
    check_execution(
        &source.replace("let index: i64 = 0;", "let index: i64 = 7;"),
        33,
    );
    check_execution(
        &source
            .replace("let index: i64 = 0;", "let index: i64 = 7;")
            .replace("index < 8", "index > 0")
            .replace("index + 1", "index - 1"),
        25,
    );
    let mut compiled = nuisc::pipeline::compile_source(&source).unwrap();
    for instruction in ["call_i64", "call_bool", "guard_return"] {
        assert!(compiled
            .yir
            .nodes
            .iter()
            .any(|node| node.op.instruction == instruction));
    }
    assert!(compiled
        .yir
        .functions
        .iter()
        .any(|function| function.name.starts_with("__nuis_scalar_continue_")));
    let canonical = nuisc::render::render_yir(&compiled.yir);
    assert_eq!(
        canonical,
        nuisc::render::render_yir(&nuisc::pipeline::compile_source(&source).unwrap().yir)
    );
    compiled.yir.nodes.reverse();
    for function in &mut compiled.yir.functions {
        function.body_nodes.reverse();
    }
    yir_runtime_host::execute_module_source_with_registry(
        &nuisc::render::render_yir(&compiled.yir),
        &yir_verify::default_registry(),
    )
    .unwrap();
    assert_eq!(native_run(&source, &compiled).status.code(), Some(37));
}

#[test]
fn scalar_control_untaken_arms_skip_local_math_arguments_and_callees() {
    for (body, diagnostic) in [
        ("return leaf(1 / index);", "zero"),
        ("let unused: i64 = 1 % index; return seed;", "zero"),
        ("return divide(seed, index);", "zero"),
        (
            "return divide(0 - 9223372036854775807 - 1, 0 - 1);",
            "overflow",
        ),
        (
            "return remainder(0 - 9223372036854775807 - 1, 0 - 1);",
            "overflow",
        ),
        (
            "if divide(1, index) > 0 { return seed; } return index;",
            "zero",
        ),
    ] {
        let helpers = format!(
            r#"
          fn leaf(v: i64) -> i64 {{ return v; }}
          fn divide(a: i64, b: i64) -> i64 {{ return a / b; }}
          fn remainder(a: i64, b: i64) -> i64 {{ return a % b; }}
          fn candidate(seed: i64, index: i64) -> i64 {{
            if seed > 0 {{ return seed + index * 3; }} else {{ {body} }}
          }}
        "#
        );
        let source = source(&helpers);
        check_execution(&source, 37);
        assert_traps(
            &source.replace("return fill(4);", "return fill(0);"),
            diagnostic,
        );
        // Exercise the opposite guarded arm, not just an unselected else.
        check_execution(
            &source.replace("seed > 0", "seed <= 0").replace(
                &format!("return seed + index * 3; }} else {{ {body}"),
                &format!("{body} }} else {{ return seed + index * 3;"),
            ),
            37,
        );
    }
}

#[test]
fn scalar_control_bool_returns_and_fallthrough_do_not_speculate() {
    let helpers = r#"
      fn choose(seed: i64, index: i64) -> bool {
        if seed > 0 {
          if index == 0 { return true; }
          return 21 / index >= 0;
        }
        let unused: i64 = 1 / index;
        return false;
      }
      fn candidate(seed: i64, index: i64) -> i64 {
        if choose(seed, index) { return seed + index * 3; }
        return 1 % index;
      }
    "#;
    let source = source(helpers);
    check_execution(&source, 37);
    assert_traps(
        &source.replace("return fill(4);", "return fill(0);"),
        "zero",
    );
}

#[test]
fn scalar_control_function_entry_arguments_still_evaluate_before_early_return() {
    for (args, diagnostic) in [
        ("load_at(buffer, 8), 1 / (seed - seed)", "index"),
        ("1 / (seed - seed), load_at(buffer, 8)", "zero"),
    ] {
        let source = with_helpers(&SOURCE.replace("seed + index * 3", &format!("candidate({args})")),
            "fn candidate(seed: i64, index: i64) -> i64 { if seed > 0 { return 3; } return index; }");
        assert_traps(&source, diagnostic);
    }
}

#[test]
fn scalar_control_branch_scopes_do_not_capture_generated_or_sibling_bindings() {
    let source = source(
        r#"
      fn __nuis_scalar_branch_0(v: i64) -> i64 { return v; }
      fn __nuis_scalar_continue_0(v: i64) -> i64 { return v; }
      fn candidate(seed: i64, index: i64) -> i64 {
        if index == 0 {
          const __nuis_scalar_condition_0: i64 = 21;
          let __nuis_scalar_result_0: i64 = __nuis_scalar_condition_0 / seed;
          if __nuis_scalar_result_0 > 0 { return __nuis_scalar_branch_0(seed); }
        } else {
          const __nuis_scalar_condition_0: i64 = 21;
          let __nuis_scalar_result_0: i64 = __nuis_scalar_condition_0 / index;
          if __nuis_scalar_result_0 < 0 { return 0; }
        }
        const __nuis_scalar_condition_0: i64 = 3;
        return __nuis_scalar_continue_0(seed + index * __nuis_scalar_condition_0);
      }
    "#,
    );
    check_execution(&source, 37);
}

#[test]
fn scalar_control_callbacks_share_fuel_and_keep_failed_state_uncommitted() {
    let project = Project::new(&source(HELPERS));
    let checkpoint = nuisc::pipeline::resolve_compile_input(&project.0)
        .unwrap()
        .compile_to_verified_yir(&Default::default())
        .unwrap();
    let mut yir = checkpoint.yir().clone();
    yir.nodes.reverse();
    for function in &mut yir.functions {
        function.body_nodes.reverse();
    }
    let registry = yir_verify::default_registry();
    let (mut session, _) = yir_runtime_host::ApplicationSession::open_registered(
        &yir,
        &registry,
        "counter",
        vec![yir_core::Value::Int(4)],
    )
    .unwrap();
    for (input, expected) in [(0, 37), (1, 105)] {
        session.event(vec![yir_core::Value::Int(input)]).unwrap();
        let yir_core::Value::Struct(state) = session.state() else {
            panic!("missing state");
        };
        assert_eq!(state.fields[0].1, yir_core::Value::Int(expected));
    }
    let last = session.state().clone();
    let error = session
        .event_budgeted(vec![yir_core::Value::Int(0)], 80)
        .unwrap_err();
    assert!(error.contains("step budget exhausted"), "{error}");
    assert_eq!(session.state(), &last);
    assert!(session.event(vec![yir_core::Value::Int(0)]).is_err());
    session.close(vec![]).unwrap();
    assert!(session.completion_status().is_err());
}

#[test]
fn scalar_control_selected_branch_math_keeps_source_failure_order() {
    for (first, last, diagnostic) in [
        (
            "1 / index",
            "(0 - 9223372036854775807 - 1) / (0 - 1)",
            "zero",
        ),
        (
            "(0 - 9223372036854775807 - 1) / (0 - 1)",
            "1 / index",
            "overflow",
        ),
    ] {
        let helpers = format!(
            "fn candidate(seed: i64, index: i64) -> i64 {{ \
            if seed > 0 {{ let unused: i64 = {first}; return {last}; }} return seed; }}"
        );
        assert_traps(&source(&helpers), diagnostic);
    }
}

#[test]
fn scalar_control_selected_callee_failure_keeps_last_callback_state() {
    let helpers = HELPERS.replace(
        "const scale: i64 = 3;",
        "if seed < 0 { return 1 / index; } const scale: i64 = 3;",
    );
    let project = Project::new(&source(&helpers));
    let checkpoint = nuisc::pipeline::resolve_compile_input(&project.0)
        .unwrap()
        .compile_to_verified_yir(&Default::default())
        .unwrap();
    let registry = yir_verify::default_registry();
    let (mut session, _) = yir_runtime_host::ApplicationSession::open_registered(
        checkpoint.yir(),
        &registry,
        "counter",
        vec![yir_core::Value::Int(4)],
    )
    .unwrap();
    session.event(vec![yir_core::Value::Int(0)]).unwrap();
    let last = session.state().clone();
    let error = session.event(vec![yir_core::Value::Int(-100)]).unwrap_err();
    assert!(error.contains("zero"), "{error}");
    assert_eq!(session.state(), &last);
    assert!(session.event(vec![yir_core::Value::Int(0)]).is_err());
    session.close(vec![]).unwrap();
    assert!(session.completion_status().is_err());
}
