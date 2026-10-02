use super::*;

#[derive(Clone, Debug)]
pub(super) struct Case {
    pub limit: u64,
    pub loops: u64,
    pub entries: u64,
    pub status: Option<i64>,
    pub remaining: [i64; 2],
    pub corrupt: Option<(usize, u64)>,
    pub alias: bool,
    pub semantic_trap: bool,
}

pub(super) fn check(source: &str, width: usize, cases: &[Case]) {
    let project = Project::with_source(source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let registry = yir_verify::default_registry();
    for case in cases {
        let bridge = yir_lower_llvm::native_session::emit_registered_with_work_limits(
            &compiled.yir,
            "counter",
            case.loops,
            case.entries,
        )
        .unwrap();
        assert!(!bridge
            .llvm_ir
            .contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
        assert!(!bridge
            .llvm_ir
            .contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
        let mut input = initial(width, case.limit);
        let args = [input[0], input[1], input[2], case.limit]
            .into_iter()
            .zip(&bridge.callbacks[0].arguments)
            .map(|(word, kind)| kind.unpack(word).unwrap())
            .collect();
        let (mut reference, _) =
            ApplicationSession::open_registered(&compiled.yir, &registry, "counter", args).unwrap();
        assert_eq!(state_words(reference.state()), input);
        if case.semantic_trap {
            assert!(reference.event(vec![]).is_err());
            assert_eq!(state_words(reference.state()), input);
        } else {
            reference.event(vec![]).unwrap();
        }
        if let Some((slot, word)) = case.corrupt {
            input[slot] = word;
        }
        let output = if case.status == Some(0) {
            state_words(reference.state())
                .into_iter()
                .map(|w| w as i64)
                .collect()
        } else if case.alias {
            input.iter().map(|w| *w as i64).collect()
        } else {
            vec![-700; width]
        };
        let repeats = if case.status.is_some() { 2 } else { 1 };
        let llvm = driver(&bridge, &input, case.alias, repeats);
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
        let run = dynamic_loop_guard::run_bounded(
            std::path::Path::new(&artifact.binary_path),
            &project.0,
        );
        assert_eq!(
            run.status.success(),
            case.status.is_some(),
            "{case:?}: {}\n{}",
            String::from_utf8_lossy(&run.stderr),
            String::from_utf8_lossy(&run.stdout)
        );
        #[cfg(unix)]
        if case.status.is_none() {
            use std::os::unix::process::ExitStatusExt;
            assert!(
                matches!(run.status.signal(), Some(4 | 5)),
                "{:?}",
                run.status
            );
        }
        let actual = String::from_utf8(run.stdout)
            .unwrap()
            .lines()
            .map(|line| line.parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        let mut expected = vec![case.status.unwrap_or(73), 91];
        expected.extend(output);
        expected.push(92);
        expected.extend(case.remaining);
        expected.extend([0, 0]);
        assert_eq!(actual, expected.repeat(repeats), "{case:?}");
    }
}

fn driver(bridge: &NativeSessionBridge, input: &[u64], alias: bool, repeats: usize) -> String {
    let mut llvm = String::new();
    for line in bridge.llvm_ir.lines() {
        if line == "  call void @llvm.trap()" {
            llvm.push_str("  call void @probe_trap()\n");
        }
        llvm.push_str(line);
        llvm.push('\n');
        if (line.starts_with("  store i64 ") && line.contains("ptr %nuis_helper_entries,"))
            || (line.starts_with("  store i64 %") && line.contains("ptr %nuis_loop_work,"))
        {
            // Only the probe is global; production budgets remain invocation-local.
            llvm.push_str(
                "  call void @probe_budget(ptr %nuis_loop_work, ptr %nuis_helper_entries)\n",
            );
        }
    }
    llvm = llvm.replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    llvm.push_str(multi_execution::ALLOCATION_PROBE);
    llvm.push_str(BUDGET_PROBE);
    let width = input.len();
    let storage = std::iter::once(91)
        .chain(
            input
                .iter()
                .map(|word| if alias { *word as i64 } else { -700 }),
        )
        .chain([92])
        .map(|word| format!("i64 {word}"))
        .collect::<Vec<_>>()
        .join(", ");
    llvm.push_str(&format!("\n@probe_out = internal global [{} x i64] [{storage}], align 8\ndefine void @probe_words() {{\n", width + 2));
    for slot in 0..width + 2 {
        llvm.push_str(&format!("  %p{slot} = getelementptr i64, ptr @probe_out, i64 {slot}\n  %v{slot} = load volatile i64, ptr %p{slot}\n  call void @nuis_debug_print_i64(i64 %v{slot})\n"));
    }
    for counter in [
        "probe_loops",
        "probe_entries",
        "probe_allocs",
        "probe_drops",
    ] {
        llvm.push_str(&format!("  %{counter} = load volatile i64, ptr @{counter}\n  call void @nuis_debug_print_i64(i64 %{counter})\n"));
    }
    llvm.push_str("  %flushed = call i32 @fflush(ptr null)\n  ret void\n}\ndefine void @probe_trap() {\n  call void @nuis_debug_print_i64(i64 73)\n  call void @probe_words()\n  ret void\n}\ndefine i64 @nuis_yir_entry() {\n  %out = getelementptr i64, ptr @probe_out, i64 1\n");
    if alias {
        llvm.push_str("  %args = getelementptr i64, ptr %out, i64 0\n");
    } else {
        llvm.push_str(&format!("  %args = alloca [{width} x i64], align 8\n"));
    }
    // Re-enter the same callback after exact exhaustion to prove fresh counters.
    for invocation in 0..repeats {
        for (slot, word) in input.iter().enumerate() {
            llvm.push_str(&format!("  %p{invocation}_{slot} = getelementptr i64, ptr %args, i64 {slot}\n  store volatile i64 {}, ptr %p{invocation}_{slot}\n", *word as i64));
        }
        llvm.push_str(&format!("  %status{invocation} = call i32 @{}(ptr %args, i64 {width}, ptr %out, i64 {width})\n  %wide{invocation} = zext i32 %status{invocation} to i64\n  call void @nuis_debug_print_i64(i64 %wide{invocation})\n  call void @probe_words()\n", bridge.callbacks[1].symbol));
    }
    llvm.push_str("  ret i64 0\n}\n");
    llvm
}

const BUDGET_PROBE: &str = "
@probe_loops = internal global i64 -1
@probe_entries = internal global i64 -1
define void @probe_budget(ptr %loops, ptr %entries) {
  %l = load i64, ptr %loops
  %e = load i64, ptr %entries
  store volatile i64 %l, ptr @probe_loops
  store volatile i64 %e, ptr @probe_entries
  ret void
}
";
