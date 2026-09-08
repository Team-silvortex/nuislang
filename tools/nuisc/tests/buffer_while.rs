use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

const SOURCE: &str = include_str!("fixtures/buffer_while.ns");

struct Project(PathBuf);

impl Project {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let project = Self(std::env::temp_dir().join(format!(
            "nuis-buffer-while-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&project.0).unwrap();
        fs::write(project.0.join("main.ns"), source).unwrap();
        fs::write(
            project.0.join("nuis.toml"),
            concat!(
            "name = \"buffer-while\"\nentry = \"main.ns\"\nmodules = [\"main.ns\"]\n",
            "application_sessions = [\"counter open=start event=step close=stop state=state\"]\n"
        ),
        )
        .unwrap();
        project
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn check_execution(source: &str, expected: i32) {
    let compiled = nuisc::pipeline::compile_source(source).unwrap();
    assert!(compiled.yir.nodes.iter().any(|node| {
        node.op.instruction == "loop_while_i64_effect"
            && node.op.args.get(6).map(String::as_str) == Some("scoped_call")
    }));
    let rendered = nuisc::render::render_yir(&compiled.yir);
    let roundtrip = yir_syntax::parse_module(&rendered).unwrap();
    yir_verify::verify_module(&roundtrip).unwrap();
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &rendered,
        &yir_verify::default_registry(),
    )
    .unwrap();
    let result = compiled
        .yir
        .functions
        .iter()
        .find(|function| function.role == yir_core::YirFunctionRole::Entry)
        .unwrap()
        .result
        .as_ref()
        .unwrap();
    assert_eq!(
        trace.values.get(&result.node),
        Some(&yir_core::Value::Int(expected.into()))
    );
    let output = native_run(source, &compiled);
    assert_eq!(
        output.status.code(),
        Some(expected),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn native_run(source: &str, compiled: &nuisc::pipeline::PipelineArtifacts) -> std::process::Output {
    let project = Project::new(source);
    let artifact = nuisc::aot::write_and_link_with_source(
        &project.0.join("main.ns"),
        &project.0.join("out"),
        source,
        nuisc::aot::AotCompileProgram {
            ast: &compiled.ast,
            nir: &compiled.nir,
            yir: &compiled.yir,
            llvm_ir: Some(&compiled.llvm_ir),
        },
        &nuisc::aot::host_cpu_build_target(),
    )
    .unwrap();
    Command::new(&artifact.binary_path).output().unwrap()
}

#[test]
fn buffer_writes_execute_in_reference_and_native() {
    check_execution(SOURCE, 37);
    check_execution(
        &SOURCE.replace("let index: i64 = 0;", "let index: i64 = 8;"),
        8,
    );
    check_execution(
        &SOURCE.replace("let index: i64 = 0;", "let index: i64 = 7;"),
        33,
    );
    check_execution(
        &SOURCE
            .replace("let index: i64 = 0;", "let index: i64 = 7;")
            .replace("index < 8", "index > 0")
            .replace("index + 1", "index - 1"),
        25,
    );
    check_execution(
        &SOURCE.replace(
            "while index < 8",
            "let length: i64 = buffer_len(buffer); while index < length",
        ),
        37,
    );
}

fn selected_buffer_source(choose_left: bool) -> String {
    SOURCE.replace(
        "let buffer: ref Buffer = alloc_buffer(8, 0);",
        &format!("let left: ref Buffer = alloc_buffer(4, 0);\nlet right: ref Buffer = alloc_buffer(8, 0);\nlet buffer: ref Buffer = select_owned_ptr({choose_left}, move(left), move(right));"),
    )
}

#[test]
fn buffer_loop_division_and_remainder_match_native_execution() {
    for (expression, expected) in [
        ("seed + index / 2 + index % 2", 20),
        ("seed + (index - 10) / 2 + (index - 10) % 2", 9),
        ("(seed - index) / (0 - 1)", 7),
        ("(seed - index) % (0 - 1)", 8),
    ] {
        check_execution(&SOURCE.replace("seed + index * 3", expression), expected);
    }
}

#[test]
fn buffer_loop_invalid_integer_divisors_fail_without_panics_or_native_ub() {
    for operator in ["/", "%"] {
        for (expression, diagnostic) in [
            (format!("1 {operator} 0"), "zero"),
            (format!("seed {operator} (index - index)"), "zero"),
            (
                format!("(0 - 9223372036854775807 - 1) {operator} (0 - 1)"),
                "overflow",
            ),
        ] {
            let source = SOURCE.replace("seed + index * 3", &expression);
            let compiled = nuisc::pipeline::compile_source(&source).unwrap();
            assert!(compiled.llvm_ir.contains("integer_divisor_invalid"));
            let error = yir_runtime_host::execute_module_source_with_registry(
                &nuisc::render::render_yir(&compiled.yir),
                &yir_verify::default_registry(),
            )
            .unwrap_err();
            assert!(error.contains(diagnostic), "{error}");
            let output = native_run(&source, &compiled);
            assert!(
                !output.status.success(),
                "invalid integer operation must trap"
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
    }
}

#[test]
fn selected_buffers_keep_the_chosen_length_through_loop_captures() {
    check_execution(&selected_buffer_source(false), 37);
}

#[test]
fn slice_field_buffers_keep_length_metadata_for_checked_reads() {
    let source = SOURCE
        .replace(
            "let result: i64",
            "let view: Slice<i64> = bytes(buffer, 2, 4);\nlet result: i64",
        )
        .replace(
            "load_at(buffer, 0) + load_at(buffer, 7)",
            "view[0] + view[1]",
        );
    check_execution(&source, 31);
}

#[test]
fn multiple_stores_and_reads_stay_inside_the_iteration_in_source_order() {
    let source = SOURCE.replace("store_at(buffer, index, value);", "store_at(buffer, index, value);\nlet next: i64 = load_at(buffer, index) + 1;\nstore_at(buffer, index, next);");
    check_execution(&source, 39);
    let source = SOURCE
        .replace(
            "let value: i64 = seed + index * 3;",
            "let value: i64 = load_at(buffer, 0) + 1;",
        )
        .replace(
            "store_at(buffer, index, value);",
            "store_at(buffer, 0, value);",
        );
    check_execution(&source, 16);
    check_execution(
        &SOURCE.replace("return fill(4);", "return fill(4) + fill(5);"),
        76,
    );
}

#[test]
fn buffer_effect_order_does_not_depend_on_yir_declaration_order() {
    let project = Project::new(SOURCE);
    let checkpoint = nuisc::pipeline::resolve_compile_input(&project.0)
        .unwrap()
        .compile_to_verified_yir(&Default::default())
        .unwrap();
    let mut yir = checkpoint.yir().clone();
    yir.nodes.reverse();
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
    assert_eq!(state.fields[0].1, yir_core::Value::Int(37));
    session.close(vec![]).unwrap();
    session.completion_status().unwrap();
}

#[test]
fn helper_names_are_private_collision_safe_and_deterministic() {
    let source = SOURCE.replace(
        "fn main()",
        "fn __nuis_buffer_iteration_0() -> i64 { return 99; }\nfn main()",
    );
    let first = nuisc::pipeline::compile_source(&source).unwrap();
    let second = nuisc::pipeline::compile_source(&source).unwrap();
    let helpers = first
        .yir
        .functions
        .iter()
        .filter(|function| function.name.starts_with("__nuis_buffer_iteration_"))
        .collect::<Vec<_>>();
    assert!(helpers
        .iter()
        .any(|function| function.name == "__nuis_buffer_iteration_1"));
    assert!(helpers
        .iter()
        .all(|function| function.role == yir_core::YirFunctionRole::Helper));
    assert_eq!(
        nuisc::render::render_yir(&first.yir),
        nuisc::render::render_yir(&second.yir)
    );
}

#[test]
fn unsupported_buffer_loop_effects_carries_and_mutable_bounds_remain_rejected() {
    for source in [
        SOURCE.replace("index + 1", "index + 2"),
        SOURCE.replace("index < 8", "index <= 8"),
        SOURCE.replace("index < 8", "index < load_at(buffer, 0)"),
        SOURCE.replace("index < 8", "index < index + 8"),
        SOURCE.replace("let value: i64 = seed + index * 3;", "let seed: i64 = seed + index; let value: i64 = seed;"),
        SOURCE.replace("store_at(buffer, index, value);", "if index > 3 { break; } store_at(buffer, index, value);"),
        SOURCE.replace("store_at(buffer, index, value);", "let snapshot: Bytes = copy_bytes(buffer); drop_bytes(snapshot); store_at(buffer, index, value);"),
    ] {
        assert!(nuisc::pipeline::compile_source(&source).is_err(), "must reject\n{source}");
    }
}

#[test]
fn buffer_index_errors_fail_before_native_memory_access() {
    for source in [
        SOURCE.replace(
            "store_at(buffer, index, value)",
            "store_at(buffer, 8, value)",
        ),
        SOURCE.replace(
            "store_at(buffer, index, value)",
            "store_at(buffer, 0 - 1, value)",
        ),
        SOURCE.replace("load_at(buffer, 7)", "load_at(buffer, 8)"),
        selected_buffer_source(true),
    ] {
        let compiled = nuisc::pipeline::compile_source(&source).unwrap();
        assert!(compiled.llvm_ir.contains("buffer_index_invalid"));
        let error = yir_runtime_host::execute_module_source_with_registry(
            &nuisc::render::render_yir(&compiled.yir),
            &yir_verify::default_registry(),
        )
        .unwrap_err();
        assert!(error.contains("index"), "{error}");
        let output = native_run(&source, &compiled);
        assert!(
            !output.status.success(),
            "out-of-bounds native access must trap"
        );
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(
                matches!(output.status.signal(), Some(4 | 5)),
                "expected LLVM trap, got {}",
                output.status
            );
        }
    }
}

#[test]
fn callback_iterations_and_helpers_share_fuel_and_do_not_commit_failed_state() {
    let project = Project::new(SOURCE);
    let resolved = nuisc::pipeline::resolve_compile_input(&project.0).unwrap();
    let checkpoint = resolved
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
    let initial = session.state().clone();
    let error = session
        .event_budgeted(vec![yir_core::Value::Int(0)], 20)
        .unwrap_err();
    assert!(error.contains("step budget exhausted"), "{error}");
    assert_eq!(session.state(), &initial);
    assert!(session.event(vec![yir_core::Value::Int(0)]).is_err());
    session.close(vec![]).unwrap();
    assert!(session.completion_status().is_err());
}

#[test]
fn buffer_while_runs_in_a_registered_application_callback() {
    let project = Project::new(SOURCE);
    let resolved = nuisc::pipeline::resolve_compile_input(&project.0).unwrap();
    let checkpoint = resolved
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
    for (input, expected) in [(0, 37), (1, 105)] {
        session.event(vec![yir_core::Value::Int(input)]).unwrap();
        let yir_core::Value::Struct(state) = session.state() else {
            panic!("missing state")
        };
        assert_eq!(state.fields[0].1, yir_core::Value::Int(expected));
    }
    session.close(vec![]).unwrap();
    session.completion_status().unwrap();
    drop(session);
    checkpoint.emit_llvm().unwrap();
}
