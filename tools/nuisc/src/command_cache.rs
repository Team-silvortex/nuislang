use std::path::PathBuf;

use crate::cache;
use crate::command_helpers::resolve_compile_input;
use crate::json_report::json_escape;

pub(crate) fn run_cache_status(
    input: Option<PathBuf>,
    all: bool,
    verbose_cache: bool,
    json: bool,
) -> Result<(), String> {
    if all {
        let workspace_root = std::env::current_dir()
            .map_err(|error| format!("failed to resolve current directory: {error}"))?;
        let summary = cache::compile_cache_inventory_summary(&workspace_root)?;
        if json {
            print!(
                "{{\"kind\":\"compile_cache_inventory\",\"workspace_root\":\"{}\",\"roots_count\":{},\"entries\":{},\"files\":{},\"bytes\":{},\"roots\":[",
                json_escape(&summary.workspace_root.display().to_string()),
                summary.roots.len(),
                summary.total_entries,
                summary.total_files,
                summary.total_bytes
            );
            for (root_index, inventory) in summary.roots.iter().enumerate() {
                if root_index > 0 {
                    print!(",");
                }
                print!(
                    "{{\"root\":\"{}\",\"entries\":{},\"files\":{},\"bytes\":{}",
                    json_escape(&inventory.root.display().to_string()),
                    inventory.entry_count,
                    inventory.total_files,
                    inventory.total_bytes
                );
                if verbose_cache {
                    print!(",\"items\":[");
                    for (entry_index, entry) in inventory.entries.iter().enumerate() {
                        if entry_index > 0 {
                            print!(",");
                        }
                        print!(
                            "{{\"key\":\"{}\",\"files\":{},\"bytes\":{},\"dir\":\"{}\"}}",
                            json_escape(&entry.key),
                            entry.file_count,
                            entry.total_bytes,
                            json_escape(&entry.entry_dir.display().to_string())
                        );
                    }
                    print!("]");
                }
                print!("}}");
            }
            println!("]}}");
        } else {
            println!("compile cache inventory");
            println!("  workspace_root: {}", summary.workspace_root.display());
            println!("  roots: {}", summary.roots.len());
            println!("  entries: {}", summary.total_entries);
            println!("  files: {}", summary.total_files);
            println!("  bytes: {}", summary.total_bytes);
            for inventory in summary.roots {
                println!("  root: {}", inventory.root.display());
                println!("    entries: {}", inventory.entry_count);
                println!("    files: {}", inventory.total_files);
                println!("    bytes: {}", inventory.total_bytes);
                if verbose_cache {
                    for entry in inventory.entries {
                        println!(
                            "    entry: {} files={} bytes={} dir={}",
                            entry.key,
                            entry.file_count,
                            entry.total_bytes,
                            entry.entry_dir.display()
                        );
                    }
                }
            }
        }
    } else {
        let input = input.expect("cache-status input must exist when --all is not set");
        let resolved = resolve_compile_input(&input)?;
        let status = cache::compile_cache_status_with_plan(
            &input,
            resolved.project.as_ref(),
            resolved.project_plan.as_ref(),
        )?;
        if json {
            print!(
                "{{\"kind\":\"compile_cache_status\",\"input\":\"{}\",\"root\":\"{}\",\"key\":\"{}\",\"state\":\"{}\",\"entry_dir\":\"{}\",\"files\":{},\"bytes\":{},\"fingerprint_inputs\":{}",
                json_escape(&input.display().to_string()),
                json_escape(&status.root.display().to_string()),
                json_escape(&status.key),
                if status.entry_exists { "present" } else { "missing" },
                json_escape(&status.entry_dir.display().to_string()),
                status.file_count,
                status.total_bytes,
                status.input_labels.len()
            );
            if verbose_cache {
                print!(",\"inputs\":[");
                for (index, label) in status.input_labels.iter().enumerate() {
                    if index > 0 {
                        print!(",");
                    }
                    print!("\"{}\"", json_escape(label));
                }
                print!("]");
            }
            println!("}}");
        } else {
            println!("compile cache status: {}", input.display());
            println!("  root: {}", status.root.display());
            println!("  key: {}", status.key);
            println!(
                "  state: {}",
                if status.entry_exists {
                    "present"
                } else {
                    "missing"
                }
            );
            println!("  entry_dir: {}", status.entry_dir.display());
            println!("  files: {}", status.file_count);
            println!("  bytes: {}", status.total_bytes);
            println!("  fingerprint_inputs: {}", status.input_labels.len());
            if verbose_cache {
                for label in status.input_labels {
                    println!("  input: {}", label);
                }
            }
        }
    }

    Ok(())
}

pub(crate) fn run_clean_cache(input: Option<PathBuf>, all: bool, json: bool) -> Result<(), String> {
    if all {
        let workspace_root = std::env::current_dir()
            .map_err(|error| format!("failed to resolve current directory: {error}"))?;
        let cleaned = cache::clean_compile_cache_summary(&workspace_root)?;
        if json {
            print!(
                "{{\"kind\":\"compile_cache_cleaned\",\"workspace_root\":\"{}\",\"cleaned_roots\":{},\"removed_entries\":{},\"removed_bytes\":{},\"roots\":[",
                json_escape(&cleaned.workspace_root.display().to_string()),
                cleaned.cleaned_roots.len(),
                cleaned.removed_entries,
                cleaned.removed_bytes
            );
            for (index, root) in cleaned.cleaned_roots.iter().enumerate() {
                if index > 0 {
                    print!(",");
                }
                print!(
                    "{{\"root\":\"{}\",\"removed_entries\":{},\"removed_bytes\":{}}}",
                    json_escape(&root.root.display().to_string()),
                    root.removed_entries,
                    root.removed_bytes
                );
            }
            println!("]}}");
        } else {
            println!("compile cache cleaned");
            println!("  workspace_root: {}", cleaned.workspace_root.display());
            println!("  cleaned_roots: {}", cleaned.cleaned_roots.len());
            println!("  removed_entries: {}", cleaned.removed_entries);
            println!("  removed_bytes: {}", cleaned.removed_bytes);
            for root in cleaned.cleaned_roots {
                println!("  root: {}", root.root.display());
                println!("    removed_entries: {}", root.removed_entries);
                println!("    removed_bytes: {}", root.removed_bytes);
            }
        }
    } else {
        let input = input.expect("clean-cache input must exist when --all is not set");
        let resolved = resolve_compile_input(&input)?;
        let cleaned = cache::clean_compile_cache_with_plan(
            &input,
            resolved.project.as_ref(),
            resolved.project_plan.as_ref(),
        )?;
        if json {
            println!(
                "{{\"kind\":\"compile_cache_cleaned\",\"input\":\"{}\",\"root\":\"{}\",\"removed_entries\":{},\"removed_bytes\":{}}}",
                json_escape(&input.display().to_string()),
                json_escape(&cleaned.root.display().to_string()),
                cleaned.removed_entries,
                cleaned.removed_bytes
            );
        } else {
            println!("compile cache cleaned: {}", input.display());
            println!("  root: {}", cleaned.root.display());
            println!("  removed_entries: {}", cleaned.removed_entries);
            println!("  removed_bytes: {}", cleaned.removed_bytes);
        }
    }

    Ok(())
}

pub(crate) fn run_prune_cache(
    input: Option<PathBuf>,
    all: bool,
    keep: usize,
    json: bool,
) -> Result<(), String> {
    if all {
        let workspace_root = std::env::current_dir()
            .map_err(|error| format!("failed to resolve current directory: {error}"))?;
        let pruned = cache::prune_compile_cache_summary(&workspace_root, keep)?;
        if json {
            print!(
                "{{\"kind\":\"compile_cache_pruned\",\"workspace_root\":\"{}\",\"keep\":{},\"pruned_roots\":{},\"kept_entries\":{},\"removed_entries\":{},\"removed_bytes\":{},\"roots\":[",
                json_escape(&pruned.workspace_root.display().to_string()),
                keep,
                pruned.pruned_roots.len(),
                pruned.kept_entries,
                pruned.removed_entries,
                pruned.removed_bytes
            );
            for (index, root) in pruned.pruned_roots.iter().enumerate() {
                if index > 0 {
                    print!(",");
                }
                print!(
                    "{{\"root\":\"{}\",\"kept_entries\":{},\"removed_entries\":{},\"removed_bytes\":{}}}",
                    json_escape(&root.root.display().to_string()),
                    root.kept_entries,
                    root.removed_entries,
                    root.removed_bytes
                );
            }
            println!("]}}");
        } else {
            println!("compile cache pruned");
            println!("  workspace_root: {}", pruned.workspace_root.display());
            println!("  keep: {}", keep);
            println!("  pruned_roots: {}", pruned.pruned_roots.len());
            println!("  kept_entries: {}", pruned.kept_entries);
            println!("  removed_entries: {}", pruned.removed_entries);
            println!("  removed_bytes: {}", pruned.removed_bytes);
            for root in pruned.pruned_roots {
                println!("  root: {}", root.root.display());
                println!("    kept_entries: {}", root.kept_entries);
                println!("    removed_entries: {}", root.removed_entries);
                println!("    removed_bytes: {}", root.removed_bytes);
            }
        }
    } else {
        let input = input.expect("cache-prune input must exist when --all is not set");
        let resolved = resolve_compile_input(&input)?;
        let pruned = cache::prune_compile_cache_with_plan(
            &input,
            resolved.project.as_ref(),
            resolved.project_plan.as_ref(),
            keep,
        )?;
        if json {
            println!(
                "{{\"kind\":\"compile_cache_pruned\",\"input\":\"{}\",\"root\":\"{}\",\"keep\":{},\"kept_entries\":{},\"removed_entries\":{},\"removed_bytes\":{}}}",
                json_escape(&input.display().to_string()),
                json_escape(&pruned.root.display().to_string()),
                keep,
                pruned.kept_entries,
                pruned.removed_entries,
                pruned.removed_bytes
            );
        } else {
            println!("compile cache pruned: {}", input.display());
            println!("  root: {}", pruned.root.display());
            println!("  keep: {}", keep);
            println!("  kept_entries: {}", pruned.kept_entries);
            println!("  removed_entries: {}", pruned.removed_entries);
            println!("  removed_bytes: {}", pruned.removed_bytes);
        }
    }

    Ok(())
}
