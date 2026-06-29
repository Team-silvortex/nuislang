use std::path::PathBuf;

use crate::command_helpers::{compile_command_input, print_project_context, resolve_compile_input};
use crate::execution_inspect_report::{inspect_execution_json, render_execution_report};
use crate::inspect_report::{
    collect_benchmark_inventory, collect_doc_indexes, inspect_benchmarks_json, inspect_docs_json,
    inspect_galaxy_doc_summary, inspect_galaxy_docs_json, inspect_stdlib_doc_summary,
    inspect_stdlib_docs_json, summarize_doc_indexes, write_json_output,
};
use crate::project;

pub(crate) fn run_inspect_execution(input: PathBuf, json: bool) -> Result<(), String> {
    if json {
        println!("{}", inspect_execution_json(&input)?);
    } else {
        println!("{}", render_execution_report(&input)?);
    }

    Ok(())
}

pub(crate) fn run_inspect_benchmarks(input: PathBuf, json: bool) -> Result<(), String> {
    let compiled = compile_command_input(&input)?;
    let benchmarks = collect_benchmark_inventory(&compiled.artifacts);
    if json {
        println!("{}", inspect_benchmarks_json(&input, &compiled.artifacts));
        return Ok(());
    }
    print_project_context(&compiled.resolved);
    println!("benchmark inventory: {}", input.display());
    println!(
        "  domain_unit: {}::{}",
        compiled.artifacts.nir.domain, compiled.artifacts.nir.unit
    );
    println!("  benchmark_count: {}", benchmarks.len());
    for entry in benchmarks {
        println!("  benchmark: {}", entry.symbol);
        println!("    label: {}", entry.label);
        println!(
            "    async: {}",
            if entry.is_async { "true" } else { "false" }
        );
        println!("    return_type: {}", entry.return_type);
        println!(
            "    warmup_iters: {}",
            entry
                .warmup_iters
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_owned())
        );
        println!(
            "    measure_iters: {}",
            entry
                .measure_iters
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_owned())
        );
        println!(
            "    timeout_ms: {}",
            entry
                .timeout_ms
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_owned())
        );
        println!(
            "    clock_domain: {}",
            entry.clock_domain.as_deref().unwrap_or("-")
        );
        println!(
            "    clock_policy: {}",
            entry.clock_policy.as_deref().unwrap_or("-")
        );
    }

    Ok(())
}

pub(crate) fn run_inspect_docs(
    input: PathBuf,
    json: bool,
    output: Option<PathBuf>,
) -> Result<(), String> {
    let indexes = collect_doc_indexes(&input)?;
    if json {
        let payload = inspect_docs_json(&input, &indexes);
        if let Some(path) = output {
            write_json_output(&path, &payload)?;
            println!("wrote doc index: {}", path.display());
            println!("  source: {}", input.display());
            println!("  bytes: {}", payload.len());
        } else {
            println!("{payload}");
        }
        return Ok(());
    }
    if project::is_project_input(&input) {
        let resolved = resolve_compile_input(&input)?;
        print_project_context(&resolved);
    }
    let summaries = summarize_doc_indexes(&indexes);
    let total_items = summaries
        .iter()
        .map(|summary| summary.item_count)
        .sum::<usize>();
    println!("doc index: {}", input.display());
    println!("  module_count: {}", summaries.len());
    println!("  documented_item_count: {}", total_items);
    for (index, summary) in indexes.iter().zip(summaries.iter()) {
        println!("  module: {}", summary.module_path);
        println!("    documented_items: {}", summary.item_count);
        for item in &index.items {
            println!("    item: {} {}", item.kind, item.path);
            if let Some(signature) = &item.signature {
                println!("      signature: {}", signature);
            }
            for line in &item.docs {
                println!("      doc: {}", line);
            }
        }
    }

    Ok(())
}

pub(crate) fn run_inspect_galaxy_docs(galaxy: String, json: bool) -> Result<(), String> {
    let summary = inspect_galaxy_doc_summary(&galaxy)?;
    if json {
        println!("{}", inspect_galaxy_docs_json(&summary));
        return Ok(());
    }
    println!("galaxy doc index: {}", summary.galaxy);
    println!("  package_id: {}", summary.package_id);
    println!("  library_module_count: {}", summary.library_module_count);
    println!(
        "  documented_library_module_count: {}",
        summary.documented_library_module_count
    );
    println!("  documented_item_count: {}", summary.documented_item_count);
    for module in summary.modules {
        println!("  library_module: {}", module.library_module);
        println!("    module_path: {}", module.module_path);
        println!("    documented_items: {}", module.documented_item_count);
        for item in module.doc_index.items {
            println!("    item: {} {}", item.kind, item.path);
            if let Some(signature) = item.signature {
                println!("      signature: {}", signature);
            }
            for line in item.docs {
                println!("      doc: {}", line);
            }
        }
    }

    Ok(())
}

pub(crate) fn run_inspect_stdlib_docs(json: bool) -> Result<(), String> {
    let summary = inspect_stdlib_doc_summary()?;
    if json {
        println!("{}", inspect_stdlib_docs_json(&summary));
        return Ok(());
    }
    println!("stdlib doc index");
    println!("  galaxy_count: {}", summary.galaxy_count);
    println!(
        "  documented_galaxy_count: {}",
        summary.documented_galaxy_count
    );
    println!("  documented_item_count: {}", summary.documented_item_count);
    for galaxy in summary.galaxies {
        println!("  galaxy: {}", galaxy.galaxy);
        println!("    package_id: {}", galaxy.package_id);
        println!("    library_module_count: {}", galaxy.library_module_count);
        println!(
            "    documented_library_module_count: {}",
            galaxy.documented_library_module_count
        );
        println!(
            "    documented_item_count: {}",
            galaxy.documented_item_count
        );
    }

    Ok(())
}
