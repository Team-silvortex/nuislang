use std::fmt;
use std::fs;

use super::{ResolvedGalaxyDependency, ResolvedGalaxyDocSummary};

fn write_joined_items<W: fmt::Write>(out: &mut W, items: &[String], sep: &str) -> fmt::Result {
    let mut first = true;
    for item in items {
        if !first {
            out.write_str(sep)?;
        }
        first = false;
        out.write_str(item)?;
    }
    Ok(())
}

pub fn render_resolved_galaxy_index(dependencies: &[ResolvedGalaxyDependency]) -> String {
    if dependencies.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    write_resolved_galaxy_index(&mut out, dependencies)
        .expect("writing resolved galaxy index to String should not fail");
    out
}

pub fn write_resolved_galaxy_index<W: fmt::Write>(
    out: &mut W,
    dependencies: &[ResolvedGalaxyDependency],
) -> fmt::Result {
    for item in dependencies {
        let doc_summary = summarize_resolved_galaxy_docs(item);
        write!(
            out,
            "{}\tpackage={}\tdirect={}\trequested_by=",
            item.name,
            item.package_id,
            if item.direct { "true" } else { "false" },
        )?;
        if item.requested_by.is_empty() {
            out.write_str("<none>")?;
        } else {
            write_joined_items(out, &item.requested_by, ",")?;
        }
        writeln!(
            out,
            "\tsource_modules={}\tauto_injectable={}\tdocumented_library_modules={}\tdocumented_items={}",
            item.source_modules.len(),
            if item.auto_injectable {
                "true"
            } else {
                "false"
            },
            doc_summary.documented_library_modules,
            doc_summary.documented_items
        )?;

        out.write_str("  library_modules=")?;
        if item.library_modules.is_empty() {
            out.write_str("<none>")?;
        } else {
            write_joined_items(out, &item.library_modules, ", ")?;
        }
        out.write_str("\n")?;
        if !doc_summary.library_module_items.is_empty() {
            out.write_str("  library_docs=")?;
            for (index, (module, items)) in doc_summary.library_module_items.iter().enumerate() {
                if index > 0 {
                    out.write_str(", ")?;
                }
                write!(out, "{module}:{items}")?;
            }
            out.write_str("\n")?;
        }

        out.write_str("  surfaces=")?;
        if item.surfaces.is_empty() {
            out.write_str("<none>")?;
        } else {
            write_joined_items(out, &item.surfaces, ", ")?;
        }
        out.write_str("\n")?;

        writeln!(
            out,
            "  library_import_policy={}",
            item.library_import_policy.as_str()
        )?;
        writeln!(out, "  manifest={}", item.manifest_path.display())?;

        out.write_str("  depends_on=")?;
        if item.depends_on.is_empty() {
            out.write_str("<none>")?;
        } else {
            write_joined_items(out, &item.depends_on, ", ")?;
        }
        out.write_str("\n")?;

        out.write_str("  blockers=")?;
        if item.auto_inject_blockers.is_empty() {
            out.write_str("<none>")?;
        } else {
            write_joined_items(out, &item.auto_inject_blockers, " | ")?;
        }
        out.write_str("\n")?;
    }
    Ok(())
}

pub(crate) fn summarize_resolved_galaxy_docs(
    item: &ResolvedGalaxyDependency,
) -> ResolvedGalaxyDocSummary {
    let mut documented_library_modules = 0usize;
    let mut documented_items = 0usize;
    let mut library_module_items = Vec::new();

    for (library_module, path) in item
        .library_modules
        .iter()
        .zip(item.resolved_library_paths.iter())
    {
        let Ok(source) = fs::read_to_string(path) else {
            continue;
        };
        let Ok(ast) = crate::frontend::parse_nuis_ast(&source) else {
            continue;
        };
        let index = crate::frontend::extract_ast_doc_index(&ast);
        let item_count = index.items.len();
        if item_count > 0 {
            documented_library_modules += 1;
            documented_items += item_count;
            library_module_items.push((library_module.clone(), item_count));
        }
    }

    ResolvedGalaxyDocSummary {
        documented_library_modules,
        documented_items,
        library_module_items,
    }
}
