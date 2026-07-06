use std::path::Path;

use crate::{
    aot, frontend, json_bool_field, json_optional_i64_field, json_optional_string_field,
    json_string_array_field, json_string_field, json_usize_field, pipeline, project,
    stdlib_registry,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BenchmarkInventoryEntry {
    pub(crate) symbol: String,
    pub(crate) label: String,
    pub(crate) is_async: bool,
    pub(crate) return_type: String,
    pub(crate) warmup_iters: Option<i64>,
    pub(crate) measure_iters: Option<i64>,
    pub(crate) timeout_ms: Option<i64>,
    pub(crate) clock_domain: Option<String>,
    pub(crate) clock_policy: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocIndexModuleSummary {
    pub(crate) module_path: String,
    pub(crate) item_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GalaxyDocModuleSummary {
    pub(crate) library_module: String,
    pub(crate) module_path: String,
    pub(crate) documented_item_count: usize,
    pub(crate) doc_index: frontend::AstDocIndex,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GalaxyDocSummary {
    pub(crate) galaxy: String,
    pub(crate) package_id: String,
    pub(crate) library_module_count: usize,
    pub(crate) documented_library_module_count: usize,
    pub(crate) documented_item_count: usize,
    pub(crate) modules: Vec<GalaxyDocModuleSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StdlibDocSummary {
    pub(crate) galaxy_count: usize,
    pub(crate) documented_galaxy_count: usize,
    pub(crate) documented_item_count: usize,
    pub(crate) galaxies: Vec<GalaxyDocSummary>,
}

pub(crate) fn collect_benchmark_inventory(
    artifacts: &pipeline::PipelineArtifacts,
) -> Vec<BenchmarkInventoryEntry> {
    frontend::collect_nir_benchmarks(&artifacts.nir)
        .into_iter()
        .map(|function| BenchmarkInventoryEntry {
            symbol: format!(
                "{}::{}::{}",
                artifacts.nir.domain, artifacts.nir.unit, function.name
            ),
            label: function
                .benchmark_name
                .clone()
                .unwrap_or_else(|| function.name.clone()),
            is_async: function.is_async,
            return_type: function
                .return_type
                .as_ref()
                .map(|ty| ty.render())
                .unwrap_or_else(|| "()".to_owned()),
            warmup_iters: function.benchmark_warmup_iters,
            measure_iters: function.benchmark_measure_iters,
            timeout_ms: function.benchmark_timeout_ms,
            clock_domain: function
                .benchmark_clock_domain
                .map(|domain| domain.as_str().to_owned()),
            clock_policy: function
                .benchmark_clock_policy
                .map(|policy| policy.as_str().to_owned()),
        })
        .collect()
}

pub(crate) fn inspect_benchmarks_json(
    input: &Path,
    artifacts: &pipeline::PipelineArtifacts,
) -> String {
    let benchmarks = collect_benchmark_inventory(artifacts);
    let entries = benchmarks
        .iter()
        .map(|entry| {
            let fields = vec![
                json_string_field("symbol", &entry.symbol),
                json_string_field("label", &entry.label),
                json_bool_field("async", entry.is_async),
                json_string_field("return_type", &entry.return_type),
                json_optional_i64_field("warmup_iters", entry.warmup_iters),
                json_optional_i64_field("measure_iters", entry.measure_iters),
                json_optional_i64_field("timeout_ms", entry.timeout_ms),
                json_optional_string_field("clock_domain", entry.clock_domain.as_deref()),
                json_optional_string_field("clock_policy", entry.clock_policy.as_deref()),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    let fields = vec![
        json_string_field("kind", "nuis_benchmark_inventory"),
        json_string_field("input", &input.display().to_string()),
        json_string_field("domain", &artifacts.nir.domain),
        json_string_field("unit", &artifacts.nir.unit),
        json_usize_field("benchmark_count", benchmarks.len()),
        format!("\"benchmarks\":[{}]", entries),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn collect_doc_indexes(input: &Path) -> Result<Vec<frontend::AstDocIndex>, String> {
    if project::is_project_input(input) {
        let project = project::load_project(input)?;
        let mut indexes = project
            .modules
            .iter()
            .map(|module| frontend::extract_ast_doc_index(&module.ast))
            .collect::<Vec<_>>();
        indexes.sort_by(|lhs, rhs| lhs.module_path.cmp(&rhs.module_path));
        return Ok(indexes);
    }

    let source = std::fs::read_to_string(input)
        .map_err(|error| format!("failed to read `{}`: {error}", input.display()))?;
    let ast = frontend::parse_nuis_ast(&source)?;
    Ok(vec![frontend::extract_ast_doc_index(&ast)])
}

pub(crate) fn summarize_doc_indexes(
    indexes: &[frontend::AstDocIndex],
) -> Vec<DocIndexModuleSummary> {
    indexes
        .iter()
        .map(|index| DocIndexModuleSummary {
            module_path: index.module_path.clone(),
            item_count: index.items.len(),
        })
        .collect()
}

pub(crate) fn inspect_docs_json(input: &Path, indexes: &[frontend::AstDocIndex]) -> String {
    let modules = indexes
        .iter()
        .map(|index| {
            let items = index
                .items
                .iter()
                .map(|item| {
                    let fields = vec![
                        json_string_field("kind", &item.kind),
                        json_string_field("path", &item.path),
                        json_string_array_field("docs", &item.docs),
                        json_optional_string_field("signature", item.signature.as_deref()),
                    ];
                    format!("{{{}}}", fields.join(","))
                })
                .collect::<Vec<_>>()
                .join(",");
            let fields = vec![
                json_string_field("module_path", &index.module_path),
                json_usize_field("item_count", index.items.len()),
                format!("\"items\":[{}]", items),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    let total_items = indexes.iter().map(|index| index.items.len()).sum::<usize>();
    let fields = vec![
        json_string_field("kind", "nuis_doc_index"),
        json_string_field("input", &input.display().to_string()),
        json_usize_field("module_count", indexes.len()),
        json_usize_field("documented_item_count", total_items),
        format!("\"modules\":[{}]", modules),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn write_json_output(path: &Path, payload: &str) -> Result<(), String> {
    std::fs::write(path, payload)
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))
}

pub(crate) fn inspect_galaxy_doc_summary(galaxy: &str) -> Result<GalaxyDocSummary, String> {
    let stdlib_root = stdlib_registry::resolve_stdlib_root()?;
    let manifest = stdlib_registry::load_stdlib_module_manifest(&stdlib_root, galaxy)?;
    let module_root = stdlib_root.join(galaxy);
    let mut modules = Vec::new();

    for library_module in &manifest.library_modules {
        let path = module_root.join(library_module);
        let source = std::fs::read_to_string(&path)
            .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
        let ast = frontend::parse_nuis_ast(&source)?;
        let doc_index = frontend::extract_ast_doc_index(&ast);
        let documented_item_count = doc_index.items.len();
        modules.push(GalaxyDocModuleSummary {
            library_module: library_module.clone(),
            module_path: doc_index.module_path.clone(),
            documented_item_count,
            doc_index,
        });
    }

    let documented_library_module_count = modules
        .iter()
        .filter(|module| module.documented_item_count > 0)
        .count();
    let documented_item_count = modules
        .iter()
        .map(|module| module.documented_item_count)
        .sum::<usize>();

    Ok(GalaxyDocSummary {
        galaxy: manifest.name,
        package_id: manifest.package_id,
        library_module_count: modules.len(),
        documented_library_module_count,
        documented_item_count,
        modules,
    })
}

pub(crate) fn inspect_galaxy_docs_json(summary: &GalaxyDocSummary) -> String {
    let modules = summary
        .modules
        .iter()
        .map(|module| {
            let items = module
                .doc_index
                .items
                .iter()
                .map(|item| {
                    format!(
                        "{{{},{},{},{}}}",
                        json_string_field("kind", &item.kind),
                        json_string_field("path", &item.path),
                        json_string_array_field("docs", &item.docs),
                        json_optional_string_field("signature", item.signature.as_deref()),
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let items_field = format!("\"items\":[{}]", items);
            format!(
                "{{{},{},{},{}}}",
                json_string_field("library_module", &module.library_module),
                json_string_field("module_path", &module.module_path),
                json_usize_field("documented_item_count", module.documented_item_count),
                items_field
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let documented_items = json_usize_field("documented_item_count", summary.documented_item_count);
    let modules_field = format!("{documented_items},\"modules\":[{modules}]");
    format!(
        "{{{},{},{},{},{},{}}}",
        json_string_field("kind", "nuis_galaxy_doc_index"),
        json_string_field("galaxy", &summary.galaxy),
        json_string_field("package_id", &summary.package_id),
        json_usize_field("library_module_count", summary.library_module_count),
        json_usize_field(
            "documented_library_module_count",
            summary.documented_library_module_count
        ),
        modules_field
    )
}

pub(crate) fn inspect_stdlib_doc_summary() -> Result<StdlibDocSummary, String> {
    let stdlib_root = stdlib_registry::resolve_stdlib_root()?;
    let layout = stdlib_registry::load_stdlib_layout(&stdlib_root)?;
    let mut galaxies = Vec::new();
    for module in layout.modules {
        galaxies.push(inspect_galaxy_doc_summary(&module.name)?);
    }
    let documented_galaxy_count = galaxies
        .iter()
        .filter(|galaxy| galaxy.documented_item_count > 0)
        .count();
    let documented_item_count = galaxies
        .iter()
        .map(|galaxy| galaxy.documented_item_count)
        .sum::<usize>();
    Ok(StdlibDocSummary {
        galaxy_count: galaxies.len(),
        documented_galaxy_count,
        documented_item_count,
        galaxies,
    })
}

pub(crate) fn inspect_stdlib_docs_json(summary: &StdlibDocSummary) -> String {
    let galaxies = summary
        .galaxies
        .iter()
        .map(|galaxy| {
            format!(
                "{{{},{},{},{},{}}}",
                json_string_field("galaxy", &galaxy.galaxy),
                json_string_field("package_id", &galaxy.package_id),
                json_usize_field("library_module_count", galaxy.library_module_count),
                json_usize_field(
                    "documented_library_module_count",
                    galaxy.documented_library_module_count
                ),
                json_usize_field("documented_item_count", galaxy.documented_item_count)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{{},{},{},{},{}}}",
        json_string_field("kind", "nuis_stdlib_doc_index"),
        json_usize_field("galaxy_count", summary.galaxy_count),
        json_usize_field("documented_galaxy_count", summary.documented_galaxy_count),
        json_usize_field("documented_item_count", summary.documented_item_count),
        format!("\"galaxies\":[{}]", galaxies)
    )
}

pub(crate) fn collect_doc_indexes_from_manifest_input(
    manifest_verify: &aot::BuildManifestVerifyReport,
) -> Result<Vec<frontend::AstDocIndex>, String> {
    collect_doc_indexes(Path::new(&manifest_verify.input))
}

pub(crate) fn write_compile_doc_index(
    input: &Path,
    output_dir: &Path,
) -> Result<aot::BuildManifestDocIndexInfo, String> {
    let indexes = collect_doc_indexes(input)?;
    let payload = inspect_docs_json(input, &indexes);
    let output_path = output_dir.join("nuis.doc-index.json");
    write_json_output(&output_path, &payload)?;
    Ok(aot::BuildManifestDocIndexInfo {
        path: output_path.display().to_string(),
        module_count: indexes.len(),
        documented_item_count: indexes.iter().map(|index| index.items.len()).sum(),
    })
}
