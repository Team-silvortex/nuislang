use super::*;
use std::path::Path;

pub(super) struct Invocation {
    pub arguments: Vec<Value>,
    pub state: Option<Vec<i64>>,
    pub leaf: Option<[i64; 2]>,
    pub iterations: i64,
    pub reference_error: Option<&'static str>,
}

pub(super) fn execute(source: &str, cases: &[Invocation], loop_label: &str, checked: bool) {
    execute_with_predicates(source, cases, loop_label, checked, None);
}

pub(super) fn execute_with_predicates(
    source: &str,
    cases: &[Invocation],
    loop_label: &str,
    checked: bool,
    predicate_counts: Option<&[i64]>,
) {
    if let Some(counts) = predicate_counts {
        assert_eq!(counts.len(), cases.len());
    }
    assert!(!cases.is_empty());
    let trap = cases.last().unwrap().state.is_none();
    assert!(cases[..cases.len() - 1]
        .iter()
        .all(|case| case.state.is_some()));
    let project = Project::with_source(source);
    let mut compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    compiled.yir.nodes.reverse();
    compiled.yir.functions.reverse();
    for function in &mut compiled.yir.functions {
        function.body_nodes.reverse();
    }
    let bridge = emit_registered(&compiled.yir, "counter").unwrap();
    assert!(bridge.llvm_ir.contains("native_loop_preflight"));
    assert_eq!(bridge.llvm_ir.contains("integer_divisor_invalid"), checked);
    let mut llvm = bridge
        .llvm_ir
        .replacen(
            "define i64 @nuis_yir_entry()",
            "define i64 @unused_native_entry()",
            1,
        )
        .replace(
            "call ptr @nuis_scheduler_owned_aggregate_alloc_v1(",
            "call ptr @probe_alloc(",
        )
        .replace(
            "call void @nuis_scheduler_owned_aggregate_drop_v1(",
            "call void @probe_drop(",
        );
    // Observe actual execution without replacing preflight or process traps.
    if predicate_counts.is_some() {
        llvm = predicate_probe::instrument(&llvm, loop_label);
    }
    let labels = llvm
        .match_indices(&format!("\n{loop_label}"))
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    assert!(!labels.is_empty(), "{loop_label}");
    for (slot, label) in labels.into_iter().enumerate().rev() {
        let insertion = label + llvm[label..].find(":\n").unwrap() + 2;
        llvm.insert_str(insertion, &format!("  %probe_count_{slot} = load volatile i64, ptr @probe_iterations\n  %probe_next_{slot} = add i64 %probe_count_{slot}, 1\n  store volatile i64 %probe_next_{slot}, ptr @probe_iterations\n"));
    }
    let leaf = llvm.find("define i64 @nuis_fn_leaf(").unwrap();
    let insertion = leaf + llvm[leaf..].find("{\n").unwrap() + 2;
    llvm.insert_str(insertion, "  call void @nuis_debug_print_i64(i64 93)\n  call void @nuis_debug_print_i64(i64 %arg0)\n  call void @nuis_debug_print_i64(i64 %arg1)\n  %probe_trips = load volatile i64, ptr @probe_iterations\n  call void @nuis_debug_print_i64(i64 %probe_trips)\n  %flushed = call i32 @fflush(ptr null)\n");
    llvm = llvm.replace(
        "  call void @llvm.trap()",
        "  call void @probe_trap_evidence()\n  call void @llvm.trap()",
    );
    llvm.push_str(multi_execution::ALLOCATION_PROBE);
    llvm.push_str("\n@probe_iterations = internal global i64 0, align 8\ndefine void @probe_trap_evidence() {\n  call void @nuis_debug_print_i64(i64 73)\n  %trips = load volatile i64, ptr @probe_iterations\n  call void @nuis_debug_print_i64(i64 %trips)\n  %flushed = call i32 @fflush(ptr null)\n  ret void\n}\ndefine i64 @nuis_yir_entry() {\n");
    let argc = bridge.callbacks[0].arguments.len();
    let outc = bridge.state_fields.len();
    llvm.push_str(&format!(
        "  %args = alloca [{argc} x i64], align 8\n  %out = alloca [{outc} x i64], align 8\n"
    ));
    let registry = yir_verify::default_registry();
    let mut oracle = Vec::new();
    for (case_index, case) in cases.iter().enumerate() {
        let reference = || {
            ApplicationSession::open_registered(
                &compiled.yir,
                &registry,
                "counter",
                case.arguments.clone(),
            )
        };
        if let Some(state) = &case.state {
            assert_eq!(state.len(), outc);
            assert!(case.reference_error.is_none());
            let (session, _) = reference().unwrap();
            assert_eq!(
                state_words(session.state()),
                state.iter().map(|word| *word as u64).collect::<Vec<_>>(),
                "case {case_index}"
            );
        } else if let Some(expected_error) = case.reference_error {
            let error = reference().err().expect("invalid reference arithmetic");
            assert!(error.contains(expected_error), "{error}");
        } else {
            // Reference fuel is not evidence for native induction preflight.
            assert!(case.leaf.is_none());
            assert_eq!(case.iterations, 0);
        }
        if let Some([value, divisor]) = case.leaf {
            oracle.extend([93, value, divisor, case.iterations]);
        }
        if let Some(state) = &case.state {
            oracle.push(0);
            oracle.extend(state);
            oracle.extend([0, case.iterations]);
        } else {
            if let Some(counts) = predicate_counts {
                oracle.extend([83, counts[case_index]]);
            }
            oracle.extend([73, case.iterations]);
        }
        llvm.push_str("  store volatile i64 0, ptr @probe_iterations\n");
        if predicate_counts.is_some() {
            llvm.push_str("  store volatile i64 0, ptr @probe_predicates\n");
        }
        let words = case
            .arguments
            .iter()
            .flat_map(state_words)
            .collect::<Vec<_>>();
        assert_eq!(words.len(), argc);
        for (slot, word) in words.into_iter().enumerate() {
            llvm.push_str(&format!("  %p{case_index}_{slot} = getelementptr i64, ptr %args, i64 {slot}\n  store volatile i64 {word}, ptr %p{case_index}_{slot}, align 8\n"));
        }
        llvm.push_str(&format!("  %status{case_index} = call i32 @{}(ptr %args, i64 {argc}, ptr %out, i64 {outc})\n  %wide{case_index} = zext i32 %status{case_index} to i64\n  call void @nuis_debug_print_i64(i64 %wide{case_index})\n", bridge.callbacks[0].symbol));
        for slot in 0..outc {
            llvm.push_str(&format!("  %o{case_index}_{slot} = getelementptr i64, ptr %out, i64 {slot}\n  %v{case_index}_{slot} = load i64, ptr %o{case_index}_{slot}, align 8\n  call void @nuis_debug_print_i64(i64 %v{case_index}_{slot})\n"));
        }
        llvm.push_str(&format!("  %allocs{case_index} = load i64, ptr @probe_allocs\n  %drops{case_index} = load i64, ptr @probe_drops\n  %live{case_index} = sub i64 %allocs{case_index}, %drops{case_index}\n  call void @nuis_debug_print_i64(i64 %live{case_index})\n  %trips{case_index} = load volatile i64, ptr @probe_iterations\n  call void @nuis_debug_print_i64(i64 %trips{case_index})\n  %flush{case_index} = call i32 @fflush(ptr null)\n"));
        if let Some(counts) = predicate_counts {
            llvm.push_str("  call void @probe_predicate_evidence()\n");
            if case.state.is_some() {
                oracle.extend([83, counts[case_index]]);
            }
        }
    }
    llvm.push_str("  ret i64 0\n}\n");
    let artifact = nuisc::aot::write_and_link_with_source(
        &project.0.join("main.ns"),
        &project.0.join("out"),
        source,
        nuisc::aot::AotCompileProgram {
            ast: &compiled.ast,
            nir: &compiled.nir,
            yir: &compiled.yir,
            llvm_ir: Some(&llvm),
        },
        &nuisc::aot::host_cpu_build_target(),
    )
    .unwrap();
    let run = dynamic_loop_guard::run_bounded(Path::new(&artifact.binary_path), &project.0);
    assert_eq!(
        run.status.success(),
        !trap,
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    if trap {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(
                matches!(run.status.signal(), Some(4 | 5)),
                "{:?}",
                run.status
            );
        }
    }
    let actual = String::from_utf8(run.stdout)
        .unwrap()
        .lines()
        .map(|line| line.parse::<i64>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(actual, oracle);
}
