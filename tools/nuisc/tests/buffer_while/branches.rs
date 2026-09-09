use super::*;

fn replacing_store(body: &str) -> String {
    SOURCE.replace("store_at(buffer, index, value);", body)
}

fn assert_traps(source: &str, diagnostic: &str) {
    let compiled = nuisc::pipeline::compile_source(source).unwrap();
    let error = yir_runtime_host::execute_module_source_with_registry(
        &nuisc::render::render_yir(&compiled.yir),
        &yir_verify::default_registry(),
    )
    .unwrap_err();
    assert!(error.contains(diagnostic), "{error}");
    let output = native_run(source, &compiled);
    assert!(
        !output.status.success(),
        "selected invalid branch must fail"
    );
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
fn nested_branches_preserve_local_reads_writes_and_loop_order() {
    let source = replacing_store(
        r#"
      store_at(buffer, index, value);
      if index % 2 == 0 {
        let old: i64 = load_at(buffer, index);
        if index < 4 { store_at(buffer, index, old + 2); }
        else { store_at(buffer, index, old + 3); }
      } else {
        let old: i64 = load_at(buffer, index);
        store_at(buffer, index, old + 4);
      }
    "#,
    );
    check_execution(&source, 43);
    check_execution(
        &source.replace("let index: i64 = 0;", "let index: i64 = 8;"),
        8,
    );
    check_execution(
        &source
            .replace("let index: i64 = 0;", "let index: i64 = 7;")
            .replace("index < 8", "index > 0")
            .replace("index + 1", "index - 1"),
        29,
    );
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
    session.event(vec![yir_core::Value::Int(0)]).unwrap();
    let yir_core::Value::Struct(state) = session.state() else {
        panic!("missing state")
    };
    assert_eq!(state.fields[0].1, yir_core::Value::Int(43));
    session.close(vec![]).unwrap();
    session.completion_status().unwrap();
}

#[test]
fn branch_condition_is_snapshotted_once_before_either_arm_mutates_it() {
    check_execution(
        &replacing_store(
            r#"
      store_at(buffer, index, value);
      if load_at(buffer, index) == value {
        store_at(buffer, index, value + 1);
      } else {
        store_at(buffer, index, value + 100);
      }
      let after: i64 = load_at(buffer, index);
      store_at(buffer, index, after + 2);
    "#,
        ),
        43,
    );
}

#[test]
fn empty_branches_and_branchless_iterations_remain_noops() {
    for (body, expected) in [
        ("if index < 4 { store_at(buffer, index, value); }", 12),
        (
            "if index < 4 {} else { store_at(buffer, index, value); }",
            33,
        ),
        (
            "if index < 4 {} else {} store_at(buffer, index, value);",
            37,
        ),
    ] {
        check_execution(&replacing_store(body), expected);
    }
}

#[test]
fn untaken_branches_do_not_read_write_or_evaluate_invalid_arithmetic() {
    for (bad, diagnostic) in [
        ("store_at(buffer, 8, value);", "index"),
        ("store_at(buffer, 0 - 1, value);", "index"),
        ("let invalid: i64 = load_at(buffer, 8); store_at(buffer, index, invalid);", "index"),
        ("let invalid: i64 = value / (seed - seed); store_at(buffer, index, invalid);", "zero"),
        ("let invalid: i64 = value % (seed - seed); store_at(buffer, index, invalid);", "zero"),
        ("let invalid: i64 = (0 - 9223372036854775807 - 1) / (0 - 1); store_at(buffer, index, invalid);", "overflow"),
        ("if load_at(buffer, 8) > 0 { store_at(buffer, index, value); }", "index"),
    ] {
        for bad_first in [false, true] {
            let body = if bad_first {
                format!("if seed <= 0 {{ {bad} }} else {{ store_at(buffer, index, value); }}")
            } else {
                format!("if seed > 0 {{ store_at(buffer, index, value); }} else {{ {bad} }}")
            };
            let source = replacing_store(&body);
            check_execution(&source, 37);
            assert_traps(&source.replace("return fill(4);", "return fill(0);"), diagnostic);
        }
    }
}

#[test]
fn branch_local_failures_and_nested_calls_share_callback_admission_and_fuel() {
    let source = replacing_store(
        r#"
      if seed > 0 {
        if index < 8 { store_at(buffer, index, value); }
      } else { store_at(buffer, 8, value); }
    "#,
    );
    let project = Project::new(&source);
    let checkpoint = nuisc::pipeline::resolve_compile_input(&project.0)
        .unwrap()
        .compile_to_verified_yir(&Default::default())
        .unwrap();
    let registry = yir_verify::default_registry();
    for exhausted in [false, true] {
        let (mut session, _) = yir_runtime_host::ApplicationSession::open_registered(
            checkpoint.yir(),
            &registry,
            "counter",
            vec![yir_core::Value::Int(4)],
        )
        .unwrap();
        session.event(vec![yir_core::Value::Int(0)]).unwrap();
        let last = session.state().clone();
        let error = if exhausted {
            session.event_budgeted(vec![yir_core::Value::Int(0)], 40)
        } else {
            session.event(vec![yir_core::Value::Int(-100)])
        }
        .unwrap_err();
        assert!(
            error.contains(if exhausted {
                "step budget exhausted"
            } else {
                "index"
            }),
            "{error}"
        );
        assert_eq!(session.state(), &last);
        assert!(session.event(vec![yir_core::Value::Int(0)]).is_err());
        session.close(vec![]).unwrap();
        assert!(session.completion_status().is_err());
    }
}

#[test]
fn outlined_branch_names_do_not_capture_user_functions_or_locals() {
    let source = replacing_store(
        r#"
      let __nuis_buffer_condition_0: i64 = load_at(buffer, index) + 3;
      if index < 4 {
        let __nuis_buffer_condition_1: i64 = value + __nuis_buffer_condition_0;
        store_at(buffer, index, __nuis_buffer_condition_1);
      } else { store_at(buffer, index, value); }
    "#,
    )
    .replace(
        "fn main()",
        "fn __nuis_buffer_branch_0() -> i64 { return 99; } fn main()",
    );
    check_execution(&source, 40);
    let first = nuisc::pipeline::compile_source(&source).unwrap();
    let second = nuisc::pipeline::compile_source(&source).unwrap();
    assert_eq!(
        nuisc::render::render_yir(&first.yir),
        nuisc::render::render_yir(&second.yir)
    );
    assert!(first
        .yir
        .functions
        .iter()
        .any(|function| function.name == "__nuis_buffer_branch_1"));
    assert!(!first
        .yir
        .functions
        .iter()
        .any(|function| function.name == "__nuis_buffer_branch_0"));
}

#[test]
fn branch_induction_and_ownership_operations_stay_fail_closed() {
    for bad in [
        "let index: i64 = index + 1; store_at(buffer, index, value);",
        "let allocated: ref Buffer = alloc_buffer(1, 0); free(allocated);",
        "let snapshot: Bytes = copy_bytes(buffer); drop_bytes(snapshot);",
        "free(buffer);",
        "break;",
        "continue;",
        "return 0;",
    ] {
        let source = replacing_store(&format!(
            "if index < 4 {{ {bad} }} else {{ store_at(buffer, index, value); }}"
        ));
        assert!(
            nuisc::pipeline::compile_source(&source).is_err(),
            "must reject {bad}"
        );
    }
}

#[test]
fn branch_carries_can_update_a_captured_function_parameter() {
    check_execution(
        &SOURCE.replace(
            "let value: i64 = seed + index * 3;",
            "let seed: i64 = seed + index; let value: i64 = seed;",
        ),
        44,
    );
    check_execution(&replacing_store(
        "if index < 4 { let seed: i64 = seed + 1; store_at(buffer, index, seed); } else { store_at(buffer, index, value); }"
    ), 42);
}

#[test]
fn scalar_only_branch_traps_are_neither_hoisted_nor_discarded() {
    for operator in ["/", "%"] {
        let source = replacing_store(&format!(
            "if seed <= 0 {{ let unused: i64 = 1 {operator} (seed - seed); }} \
            store_at(buffer, index, value);"
        ));
        check_execution(&source, 37);
        let selected = source.replace("return fill(4);", "return fill(0);");
        assert_traps(&selected, "zero");
        let mut compiled = nuisc::pipeline::compile_source(&selected).unwrap();
        compiled.yir.nodes.reverse();
        for function in &mut compiled.yir.functions {
            function.body_nodes.reverse();
        }
        let error = yir_runtime_host::execute_module_source_with_registry(
            &nuisc::render::render_yir(&compiled.yir),
            &yir_verify::default_registry(),
        )
        .unwrap_err();
        assert!(error.contains("zero"), "{error}");
    }
}
