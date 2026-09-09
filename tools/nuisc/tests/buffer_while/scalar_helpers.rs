use super::*;

pub(super) fn with_helpers(source: &str, helpers: &str) -> String {
    source.replace("fn main()", &format!("{helpers}\nfn main()"))
}

const HELPERS: &str = r#"
  fn compose(seed: i64, index: i64) -> i64 { return plus(seed, tripled(index)); }
  fn agrees(left: i64, right: i64) -> bool { return boolean(left == right); }
  fn boolean(value: bool) -> bool { return value; }
  fn plus(left: i64, right: i64) -> i64 { return identity(left) + identity(right); }
  fn tripled(value: i64) -> i64 { const factor: i64 = 3; return identity(value) * factor; }
  fn identity(value: i64) -> i64 { return value; }
  fn unrelated(value: i64) -> i64 { return value + 99; }
"#;

fn composed_source() -> String {
    with_helpers(
        &SOURCE
            .replace("seed + index * 3", "compose(seed, index)")
            .replace(
                "store_at(buffer, index, value);",
                r#"
      store_at(buffer, identity(index), value);
      if agrees(load_at(buffer, index), value) {
        store_at(buffer, index, plus(load_at(buffer, index), identity(1)));
      } else { store_at(buffer, 8, value); }
    "#,
            ),
        HELPERS,
    )
}

fn assert_real_calls(compiled: &nuisc::pipeline::PipelineArtifacts) {
    for (name, instruction) in [
        ("compose", "call_i64"),
        ("plus", "call_i64"),
        ("tripled", "call_i64"),
        ("identity", "call_i64"),
        ("agrees", "call_bool"),
        ("boolean", "call_bool"),
    ] {
        assert!(compiled
            .yir
            .functions
            .iter()
            .any(|function| function.name == name));
        assert!(
            compiled.yir.nodes.iter().any(|node| {
                node.op.instruction == instruction
                    && node.op.args.first().map(String::as_str) == Some(name)
            }),
            "missing direct {instruction} {name}"
        );
    }
    assert!(!compiled
        .yir
        .functions
        .iter()
        .any(|function| function.name == "unrelated"));
}

#[test]
fn scalar_helper_dag_executes_real_calls_with_ordered_buffer_arguments() {
    let source = composed_source();
    check_execution(&source, 39);
    check_execution(
        &source.replace("let index: i64 = 0;", "let index: i64 = 8;"),
        8,
    );
    check_execution(
        &source.replace("let index: i64 = 0;", "let index: i64 = 7;"),
        34,
    );
    check_execution(
        &source
            .replace("let index: i64 = 0;", "let index: i64 = 7;")
            .replace("index < 8", "index > 0")
            .replace("index + 1", "index - 1"),
        26,
    );
    let compiled = nuisc::pipeline::compile_source(&source).unwrap();
    assert_real_calls(&compiled);
    let second = nuisc::pipeline::compile_source(&source).unwrap();
    assert_eq!(
        nuisc::render::render_yir(&compiled.yir),
        nuisc::render::render_yir(&second.yir)
    );
}

pub(super) fn assert_traps(source: &str, diagnostic: &str) {
    let mut compiled = nuisc::pipeline::compile_source(source).unwrap();
    compiled.yir.nodes.reverse();
    for function in &mut compiled.yir.functions {
        function.body_nodes.reverse();
    }
    let error = yir_runtime_host::execute_module_source_with_registry(
        &nuisc::render::render_yir(&compiled.yir),
        &yir_verify::default_registry(),
    )
    .unwrap_err();
    assert!(error.contains(diagnostic), "{error}");
    let output = native_run(source, &compiled);
    assert!(!output.status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(
            matches!(output.status.signal(), Some(4 | 5)),
            "{}",
            output.status
        );
    }
}

#[test]
fn untaken_scalar_helper_calls_do_not_evaluate_arguments_or_callee_traps() {
    for (expression, diagnostic) in [
        ("identity(load_at(buffer, 8))", "index"),
        ("identity(1 / (seed - seed))", "zero"),
        ("divide(value, seed - seed)", "zero"),
        ("divide(0 - 9223372036854775807 - 1, 0 - 1)", "overflow"),
        ("remainder(value, seed - seed)", "zero"),
        ("unused_trap(seed - seed)", "zero"),
    ] {
        let helpers = format!(
            r#"{HELPERS}
          fn divide(a: i64, b: i64) -> i64 {{ return a / b; }}
          fn remainder(a: i64, b: i64) -> i64 {{ return a % b; }}
          fn unused_trap(zero: i64) -> i64 {{ let unused: i64 = 1 / zero; return zero; }}
        "#
        );
        let source = with_helpers(&SOURCE.replace(
            "store_at(buffer, index, value);",
            &format!("if seed > 0 {{ store_at(buffer, index, value); }} else {{ \
                if agrees(index, index) {{ let bad: i64 = {expression}; store_at(buffer, index, bad); }} }}"),
        ), &helpers);
        check_execution(&source, 37);
        assert_traps(
            &source.replace("return fill(4);", "return fill(0);"),
            diagnostic,
        );
    }
}

#[test]
fn scalar_helper_arguments_and_unused_callee_math_keep_failure_order() {
    let helpers = r#"
      fn combine(a: i64, b: i64) -> i64 { return a + b; }
      fn checked(value: i64) -> i64 { let unused: i64 = 1 / value; return 1 % value; }
    "#;
    for (expression, diagnostic) in [
        ("combine(load_at(buffer, 8), 1 / (seed - seed))", "index"),
        ("combine(1 / (seed - seed), load_at(buffer, 8))", "zero"),
        ("checked(seed - seed)", "zero"),
    ] {
        let source = with_helpers(&SOURCE.replace("seed + index * 3", expression), helpers);
        assert_traps(&source, diagnostic);
    }
}

#[test]
fn scalar_helper_effects_recursion_and_dynamic_headers_stay_fail_closed() {
    for helpers in [
        "fn candidate(v: i64) -> i64 { print(v); return v; }",
        "fn candidate(v: i64) -> i64 { let b: ref Buffer = alloc_buffer(1, v); let x: i64 = load_at(b, 0); free(b); return x; }",
        "fn candidate(v: i64) -> i64 { return impure(v); } fn impure(v: i64) -> i64 { print(v); return v; }",
        "fn candidate(v: i64) -> i64 { return candidate(v - 1) + 1; }",
        "fn candidate(v: i64) -> i64 { return cycle(v) + 1; } fn cycle(v: i64) -> i64 { return candidate(v) + 1; }",
        "fn candidate(v: i64) -> i64 { if v > 0 { print(v); return v; } return 0; }",
        "async fn candidate(v: i64) -> i64 { return v; }",
    ] {
        let source = with_helpers(&SOURCE.replace("seed + index * 3", "candidate(seed)"), helpers);
        assert!(nuisc::pipeline::compile_source(&source).is_err(), "must reject {helpers}");
    }
    for source in [
        with_helpers(
            &SOURCE.replace("seed + index * 3", "reading(buffer, index)"),
            "fn reading(b: ref Buffer, i: i64) -> i64 { return load_at(b, i); }",
        ),
        with_helpers(&SOURCE.replace("index < 8", "index < identity(8)"), HELPERS),
    ] {
        assert!(nuisc::pipeline::compile_source(&source).is_err());
    }
}

#[test]
fn scalar_helper_callbacks_share_fuel_and_keep_failed_state_uncommitted() {
    let source = composed_source();
    let project = Project::new(&source);
    let checkpoint = nuisc::pipeline::resolve_compile_input(&project.0)
        .unwrap()
        .compile_to_verified_yir(&Default::default())
        .unwrap();
    let mut yir = checkpoint.yir().clone();
    yir.nodes.reverse();
    for function in &mut yir.functions {
        function.body_nodes.reverse();
    }
    yir_verify::verify_module(&yir).unwrap();
    let registry = yir_verify::default_registry();
    let (mut session, _) = yir_runtime_host::ApplicationSession::open_registered(
        &yir,
        &registry,
        "counter",
        vec![yir_core::Value::Int(4)],
    )
    .unwrap();
    for (input, expected) in [(0, 39), (1, 111)] {
        session.event(vec![yir_core::Value::Int(input)]).unwrap();
        let yir_core::Value::Struct(state) = session.state() else {
            panic!("missing state")
        };
        assert_eq!(state.fields[0].1, yir_core::Value::Int(expected));
    }
    let last = session.state().clone();
    let error = session
        .event_budgeted(vec![yir_core::Value::Int(0)], 50)
        .unwrap_err();
    assert!(error.contains("step budget exhausted"), "{error}");
    assert_eq!(session.state(), &last);
    assert!(session.event(vec![yir_core::Value::Int(0)]).is_err());
    session.close(vec![]).unwrap();
    assert!(session.completion_status().is_err());
}

#[test]
fn scalar_helper_callee_failure_preserves_last_callback_state_without_retry() {
    let source = with_helpers(
        &SOURCE.replace(
            "store_at(buffer, index, value);",
            r#"
          if seed > 0 { store_at(buffer, index, value); }
          else { store_at(buffer, index, divide(value, seed - seed)); }
        "#,
        ),
        "fn divide(value: i64, divisor: i64) -> i64 { return value / divisor; }",
    );
    let project = Project::new(&source);
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
