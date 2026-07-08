use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_MAX_LINES: usize = 600;

fn exception_budgets() -> BTreeMap<&'static str, usize> {
    BTreeMap::from([
        ("crates/nuis-artifact/src/artifact.rs", 1801),
        ("crates/nuis-runtime/src/executor.rs", 2282),
        ("crates/nuis-runtime/src/loader.rs", 775),
        ("crates/nuis-semantics/src/model.rs", 2720),
        ("crates/yir-core/src/lib.rs", 3450),
        ("crates/yir-domain-cpu/src/lib.rs", 3217),
        ("crates/yir-domain-kernel/src/lib.rs", 1581),
        ("crates/yir-domain-shader/src/lib.rs", 6445),
        ("crates/yir-lower-contract/src/lib.rs", 2530),
        ("crates/yir-lower-llvm/src/lib.rs", 10922),
        ("crates/yir-lower-llvm/src/tests.rs", 2132),
        ("docs/grammar/nuis-ir.md", 678),
        ("docs/historical/nuislang-whitepaper-v0.44b.md", 639),
        ("docs/reference/nsld-linker-frontdoor.md", 1388),
        ("docs/reference/std-shader-kernel-project-contract.md", 770),
        ("docs/reference/yir-langref.md", 874),
        ("docs/reference/yir-tools-reference.md", 674),
        (
            "examples/projects/domains/net_session_recipe_demo/main.ns",
            1106,
        ),
        (
            "examples/projects/tooling/cli_compile_workflow_demo/main.ns",
            638,
        ),
        (
            "examples/projects/tooling/cli_project_build_report_demo/main.ns",
            609,
        ),
        (
            "examples/projects/tooling/cli_workflow_automation_demo/main.ns",
            629,
        ),
        ("examples/projects/window_controls_demo/main.ns", 619),
        ("stdlib/std/cli_build_pipeline_recipe.ns", 972),
        ("stdlib/std/cli_compile_workflow_recipe.ns", 1595),
        ("stdlib/std/cli_project_build_report_recipe.ns", 974),
        ("stdlib/std/cli_workflow_automation_recipe.ns", 918),
        ("stdlib/std/net_session_recipe.ns", 1267),
        ("stdlib/std/workflow_runtime.ns", 657),
        ("tools/nsbdr/src/main.rs", 634),
        ("tools/nsld/src/check.rs", 925),
        ("tools/nsld/src/closure.rs", 616),
        ("tools/nsld/src/display.rs", 1862),
        ("tools/nsld/src/final_executable_emit.rs", 672),
        ("tools/nsld/src/json.rs", 2220),
        ("tools/nsld/src/json_container.rs", 639),
        ("tools/nsld/src/main.rs", 633),
        ("tools/nsld/src/main_cli_tests.rs", 867),
        ("tools/nsld/src/main_container_verify_tests.rs", 710),
        ("tools/nsld/src/main_tests.rs", 3695),
        ("tools/nsld/src/object_image_dry_run.rs", 812),
        ("tools/nsld/src/object_writer_input.rs", 682),
        ("tools/nsld/src/reports.rs", 1109),
        ("tools/nuis/src/main.rs", 3244),
        (
            "tools/nuisc/src/frontend/tests_generic_method_bounds_control_flow.rs",
            815,
        ),
        ("tools/nuisc/src/frontend/tests_lambda_higher_order.rs", 946),
        (
            "tools/nuisc/src/frontend/tests_match_struct_bindings.rs",
            973,
        ),
        ("tools/nuisc/src/frontend/tests_packet_test_meta.rs", 615),
        ("tools/nuisc/src/frontend/tests_parse_annotations.rs", 1120),
        ("tools/nuisc/src/frontend/tests_try.rs", 1291),
        ("tools/nuisc/src/frontend/tests_types_async_window.rs", 938),
        ("tools/nuisc/src/lib_tests.rs", 2047),
        (
            "tools/nuisc/src/lowering/tests_async_network_runtime.rs",
            657,
        ),
        ("tools/nuisc/src/lowering/tests_loop_flow.rs", 1555),
        ("tools/nuisc/src/lowering/tests_loop_post_flow.rs", 1143),
        ("tools/nuisc/src/lowering/tests_loops_basic.rs", 626),
        ("tools/nuisc/src/lowering/tests_recursion.rs", 850),
        ("tools/nuisc/src/nir_verify/tests.rs", 1789),
        ("tools/nuisc/src/project/tests/abi_recommendation.rs", 1829),
        ("tools/nuisc/src/project/tests/galaxy_resolution.rs", 1139),
        (
            "tools/nuisc/src/project/tests/packet_data_contracts.rs",
            699,
        ),
        ("tools/nuisc/src/project/tests/planning_kernel.rs", 609),
        (
            "tools/nuisc/src/project/tests/shader_nova_contracts.rs",
            2115,
        ),
        ("tools/nuisc/src/registry_tests.rs", 1586),
        ("tools/nuisc/tests/glm_verify.rs", 1400),
        ("tools/nuisc/tests/memory_compile.rs", 1786),
        ("tools/nuisc/tests/network_compile.rs", 1446),
        ("tools/yir-pack-aot/src/main.rs", 4329),
    ])
}

fn should_check(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("rs" | "ns" | "toml" | "md")
    )
}

fn visit_files(root: &Path, dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|err| {
        panic!("failed to read directory {}: {err}", dir.display());
    });
    for entry in entries {
        let entry = entry.unwrap_or_else(|err| panic!("failed to read directory entry: {err}"));
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(&path);
        if rel.starts_with(".git") || rel.starts_with("target") {
            continue;
        }
        if path.is_dir() {
            visit_files(root, &path, files);
        } else if should_check(&path) {
            files.push(path);
        }
    }
}

fn line_count(path: &Path) -> usize {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    content.lines().count()
}

#[test]
fn repository_text_files_respect_line_budget() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to canonicalize repo root");
    let exception_budgets = exception_budgets();
    let mut files = Vec::new();
    visit_files(&repo_root, &repo_root, &mut files);
    files.sort();

    let mut violations = Vec::new();
    for path in files {
        let rel = path
            .strip_prefix(&repo_root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let lines = line_count(&path);
        let limit = exception_budgets
            .get(rel.as_str())
            .copied()
            .unwrap_or(DEFAULT_MAX_LINES);
        if lines > limit {
            let reason = if exception_budgets.contains_key(rel.as_str()) {
                format!("{rel}: {lines} lines exceeds exception budget {limit}")
            } else {
                format!("{rel}: {lines} lines exceeds default limit {DEFAULT_MAX_LINES}")
            };
            violations.push(reason);
        }
    }

    assert!(
        violations.is_empty(),
        "file line budget violations:\n{}",
        violations.join("\n")
    );
}
