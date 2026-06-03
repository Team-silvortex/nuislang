use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_MAX_LINES: usize = 600;

fn exception_budgets() -> BTreeMap<&'static str, usize> {
    BTreeMap::from([
        ("crates/nuis-semantics/src/model.rs", 2479),
        ("crates/yir-core/src/lib.rs", 3331),
        ("crates/yir-domain-cpu/src/lib.rs", 2905),
        ("crates/yir-domain-kernel/src/lib.rs", 1575),
        ("crates/yir-domain-shader/src/lib.rs", 7212),
        ("crates/yir-lower-contract/src/lib.rs", 886),
        ("crates/yir-lower-llvm/src/lib.rs", 5357),
        ("crates/yir-syntax/src/lib.rs", 550),
        ("crates/yir-verify/src/lib.rs", 5442),
        ("docs/grammar/nuis-ir.md", 678),
        ("docs/historical/nuislang-whitepaper-v0.44b.md", 639),
        ("docs/reference/std-net-layering-contract.md", 645),
        ("docs/reference/std-shader-kernel-project-contract.md", 751),
        ("docs/reference/yir-langref.md", 874),
        ("docs/reference/yir-tools-reference.md", 626),
        (
            "examples/projects/domains/net_session_recipe_demo/main.ns",
            1106,
        ),
        ("examples/projects/window_controls_demo/main.ns", 619),
        ("stdlib/std/net_session_recipe.ns", 1106),
        ("tools/nuis/src/cli.rs", 618),
        ("tools/nuis/src/galaxy.rs", 1819),
        ("tools/nuis/src/main.rs", 3413),
        ("tools/nuisc/src/aot.rs", 4265),
        ("tools/nuisc/src/cache.rs", 716),
        ("tools/nuisc/src/frontend/mod.rs", 27155),
        ("tools/nuisc/src/frontend/parser.rs", 1864),
        ("tools/nuisc/src/lib.rs", 1276),
        ("tools/nuisc/src/lowering.rs", 11446),
        ("tools/nuisc/src/nir_verify.rs", 1767),
        ("tools/nuisc/src/nustar_binary.rs", 1112),
        ("tools/nuisc/src/optimize.rs", 1793),
        ("tools/nuisc/src/project.rs", 7620),
        ("tools/nuisc/src/registry.rs", 1651),
        ("tools/nuisc/src/render.rs", 1749),
        ("tools/yir-pack-aot/src/main.rs", 1519),
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
