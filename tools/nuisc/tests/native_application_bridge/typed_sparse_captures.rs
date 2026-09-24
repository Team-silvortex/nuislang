use super::*;

const SOURCE: &str = include_str!("typed_sparse_captures.ns");
#[path = "typed_alias_capture_fixture.rs"]
mod aliases;

#[test]
fn typed_sparse_captures_execute_64_integer_state_with_four_private_arguments() {
    check_sparse_captures(SOURCE, None, 0);
}

#[test]
fn typed_sparse_captures_follow_aliases_inside_guarded_private_helpers() {
    check_sparse_captures(&aliases::source(), Some(&[2, 3]), 0);
}

#[test]
fn typed_sparse_captures_preserve_old_and_rebound_record_versions() {
    check_sparse_captures(&aliases::rebound_source(), Some(&[2, 3]), 30);
}

#[test]
fn typed_sparse_captures_keep_same_name_branch_aliases_and_suffixes_independent() {
    check_sparse_captures(&aliases::scoped_source(), Some(&[2, 2, 2, 3]), 0);
}

#[test]
fn typed_sparse_captures_preserve_branch_local_rebound_snapshots() {
    check_sparse_captures(&aliases::scoped_rebound_source(), Some(&[2, 2, 2, 3]), 30);
}

#[test]
fn typed_sparse_captures_compose_with_invariant_iteration_aliases() {
    check_sparse_captures(&aliases::loop_source(), Some(&[2, 3]), 0);
}

#[test]
fn typed_sparse_captures_project_wide_invariant_iteration_inputs() {
    let source = aliases::wide_loop_source();
    let compiled = nuisc::pipeline::compile_source(&source).unwrap();
    let iteration = compiled
        .yir
        .functions
        .iter()
        .find(|f| f.name.starts_with("__nuis_scalar_iteration_"))
        .unwrap();
    assert_eq!(iteration.parameters.len(), 4);
    check_sparse_captures(&source, Some(&[2, 3]), 0);
}

#[test]
fn typed_sparse_captures_preserve_cross_scope_terminal_snapshots() {
    check_sparse_captures(&aliases::terminal_snapshot_source(), Some(&[2, 3]), 30);
}

fn check_sparse_captures(source: &str, branch_sizes: Option<&[usize]>, offset: i64) {
    let project = Project::with_source(source);
    let mut compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    compiled.yir.nodes.reverse();
    compiled.yir.functions.reverse();
    for function in &mut compiled.yir.functions {
        function.body_nodes.reverse();
    }
    if let Some(branch_sizes) = branch_sizes {
        let mut sizes = compiled
            .yir
            .functions
            .iter()
            .filter(|f| f.name.starts_with("__nuis_scalar_branch"))
            .map(|f| f.parameters.len())
            .collect::<Vec<_>>();
        sizes.sort();
        assert_eq!(
            sizes,
            branch_sizes,
            "{}",
            nuisc::render::render_yir(&compiled.yir)
        );
    } else {
        let selection = compiled
            .yir
            .functions
            .iter()
            .find(|f| f.name.starts_with("__nuis_conditional_value"))
            .unwrap();
        assert_eq!(selection.parameters.len(), 4);
        assert_eq!(
            selection
                .parameters
                .iter()
                .map(|p| p.ty.as_str())
                .collect::<Vec<_>>(),
            ["bool", "i64", "i64", "i64"]
        );
    }
    let choose = compiled
        .yir
        .functions
        .iter()
        .find(|f| f.name == "choose")
        .unwrap();
    assert_eq!(choose.parameters.len(), 64);
    let bridge = emit_registered(&compiled.yir, "counter").unwrap();
    assert!(!bridge
        .llvm_ir
        .contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
    assert!(!bridge
        .llvm_ir
        .contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
    let mut llvm = bridge.llvm_ir.replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    llvm.push_str(multi_execution::ALLOCATION_PROBE);
    llvm.push_str("\ndefine i64 @nuis_yir_entry() {\n  %args = alloca [4 x i64], align 8\n  %words = alloca [66 x i64], align 8\n  %state = getelementptr i64, ptr %words, i64 1\n");
    let mut expected = Vec::new();
    let registry = yir_verify::default_registry();
    for (case, input) in [
        [12, 0, 1, 99],
        [-12, 0, 1, 99],
        [12, 3, 0, 99],
        [-15, -3, 0, 99],
    ]
    .into_iter()
    .enumerate()
    {
        let (mut reference, _) = ApplicationSession::open_registered(
            &compiled.yir,
            &registry,
            "counter",
            input.into_iter().map(Value::Int).collect(),
        )
        .unwrap();
        let mut output = (0..64).map(i64::from).collect::<Vec<_>>();
        output[0] = input[0];
        output[1] = input[1];
        output[2] = input[2];
        output[63] = input[3];
        assert_eq!(
            state_words(reference.state()),
            output.iter().map(|v| *v as u64).collect::<Vec<_>>()
        );
        for (slot, word) in input.into_iter().enumerate() {
            llvm.push_str(&format!("  %a{case}_{slot} = getelementptr i64, ptr %args, i64 {slot}\n  store volatile i64 {word}, ptr %a{case}_{slot}\n"));
        }
        for (slot, word) in [(0, 91), (65, 92)] {
            llvm.push_str(&format!("  %s{case}_{slot} = getelementptr i64, ptr %words, i64 {slot}\n  store volatile i64 {word}, ptr %s{case}_{slot}\n"));
        }
        for (role, export) in bridge.callbacks.iter().enumerate() {
            if role == 1 {
                reference.event(vec![]).unwrap();
                output[0] = if input[2] > 0 {
                    input[0]
                } else {
                    input[3] / input[1]
                } + offset;
            } else if role == 2 {
                reference.close(vec![]).unwrap();
                reference.completion_status().unwrap();
            }
            assert_eq!(
                state_words(reference.state()),
                output.iter().map(|v| *v as u64).collect::<Vec<_>>()
            );
            let (args, count) = if role == 0 {
                ("args", 4)
            } else {
                ("state", 64)
            };
            llvm.push_str(&format!("  %r{case}_{role} = call i32 @{}(ptr %{args}, i64 {count}, ptr %state, i64 64)\n  %w{case}_{role} = zext i32 %r{case}_{role} to i64\n  call void @nuis_debug_print_i64(i64 %w{case}_{role})\n", export.symbol));
            expected.push(0);
            for (slot, value) in std::iter::once(&91).chain(&output).chain([&92]).enumerate() {
                llvm.push_str(&format!("  %p{case}_{role}_{slot} = getelementptr i64, ptr %words, i64 {slot}\n  %v{case}_{role}_{slot} = load volatile i64, ptr %p{case}_{role}_{slot}\n  call void @nuis_debug_print_i64(i64 %v{case}_{role}_{slot})\n"));
                expected.push(*value);
            }
            for counter in ["probe_allocs", "probe_drops"] {
                llvm.push_str(&format!("  %{counter}{case}_{role} = load i64, ptr @{counter}\n  call void @nuis_debug_print_i64(i64 %{counter}{case}_{role})\n"));
                expected.push(0);
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
    let run =
        dynamic_loop_guard::run_bounded(std::path::Path::new(&artifact.binary_path), &project.0);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let actual = String::from_utf8(run.stdout)
        .unwrap()
        .lines()
        .map(|line| line.parse::<i64>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}
