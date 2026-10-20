use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_MAX_LINES: usize = 800;
const TEST_MAX_LINES: usize = 1000;
const MARKDOWN_MAX_LINES: usize = 2000;

fn exception_budgets() -> BTreeMap<&'static str, usize> {
    BTreeMap::from([
        (
            "examples/projects/domains/net_session_recipe_demo/main.ns",
            1106,
        ),
        ("stdlib/std/cli_build_pipeline_recipe.ns", 1029),
        ("stdlib/std/cli_compile_workflow_recipe.ns", 1659),
        ("stdlib/std/cli_project_build_report_recipe.ns", 1032),
        ("stdlib/std/cli_workflow_automation_recipe.ns", 930),
        ("stdlib/std/net_session_recipe.ns", 1267),
        ("tools/nuisc/src/frontend/tests_try.rs", 1291),
        ("tools/nuisc/src/lowering/tests_loop_post_flow.rs", 1143),
        ("tools/nuisc/src/nir_verify/tests.rs", 1787),
        (
            "tools/nuisc/src/project/tests/shader_nova_contracts.rs",
            2115,
        ),
        ("tools/nuisc/tests/glm_verify.rs", 1400),
        ("tools/nuisc/tests/memory_compile.rs", 1747),
        ("tools/nuisc/tests/network_compile.rs", 1446),
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
        if rel.starts_with(".git")
            || rel.starts_with("target")
            || rel
                .components()
                .any(|component| component.as_os_str() == ".nuis")
        {
            continue;
        }
        if path.is_dir() {
            visit_files(root, &path, files);
        } else if should_check(&path) {
            files.push(path);
        }
    }
}

#[test]
fn historical_line_exceptions_are_still_needed_and_tight() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (relative, budget) in exception_budgets() {
        let path = root.join(relative);
        let lines = line_count(&path);
        assert!(
            lines > default_limit_for(Path::new(relative)),
            "{relative}: remove the obsolete exception after shrinking below the default limit"
        );
        assert_eq!(
            lines, budget,
            "{relative}: lower the exception budget when the file shrinks"
        );
    }
}

fn line_count(path: &Path) -> usize {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    content.lines().count()
}

fn default_limit_for(path: &Path) -> usize {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("md") => MARKDOWN_MAX_LINES,
        Some("rs") if is_test_source(path) => TEST_MAX_LINES,
        _ => DEFAULT_MAX_LINES,
    }
}

fn is_test_source(path: &Path) -> bool {
    if path.components().any(|part| {
        part.as_os_str()
            .to_str()
            .is_some_and(|name| name == "tests" || name.ends_with("_tests"))
    }) {
        return true;
    }
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| {
            stem == "tests" || stem.starts_with("tests_") || stem.ends_with("_tests")
        })
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
        let default_limit = default_limit_for(&path);
        let limit = exception_budgets
            .get(rel.as_str())
            .copied()
            .unwrap_or(default_limit);
        if lines > limit {
            let reason = if exception_budgets.contains_key(rel.as_str()) {
                format!("{rel}: {lines} lines exceeds exception budget {limit}")
            } else {
                format!("{rel}: {lines} lines exceeds default limit {default_limit}")
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
