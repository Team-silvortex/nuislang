use std::path::PathBuf;

use crate::command_compile;
use crate::project_metadata_report::{
    inspect_project_metadata, inspect_project_metadata_json,
    render_project_metadata_compact_summary, render_project_metadata_paths,
    render_project_metadata_summary, repair_project_metadata_target,
};

pub(crate) fn run_inspect_project_metadata(
    input: PathBuf,
    json: bool,
    summary: bool,
    paths_only: bool,
) -> Result<(), String> {
    let metadata = inspect_project_metadata(&input)?;
    if json {
        println!("{}", inspect_project_metadata_json(&metadata));
        return Ok(());
    }
    if summary {
        println!("{}", render_project_metadata_compact_summary(&metadata));
        return Ok(());
    }
    if paths_only {
        println!("{}", render_project_metadata_paths(&metadata));
        return Ok(());
    }
    println!("{}", render_project_metadata_summary(&metadata));

    Ok(())
}

pub(crate) fn run_repair_project_metadata(input: PathBuf, dry_run: bool) -> Result<(), String> {
    let (project_input, output_dir) = repair_project_metadata_target(&input)?;
    if dry_run {
        println!("project metadata repair plan");
        println!("  source: {}", input.display());
        println!("  input: {}", project_input.display());
        println!("  output_dir: {}", output_dir.display());
        println!(
            "  command: nuisc compile \"{}\" \"{}\"",
            project_input.display(),
            output_dir.display()
        );
        return Ok(());
    }
    command_compile::run_compile(
        project_input.clone(),
        output_dir.clone(),
        false,
        None,
        None,
        None,
    )?;
    let repaired_manifest = output_dir.join("nuis.build.manifest.toml");
    let repaired_summary = inspect_project_metadata(&repaired_manifest)?;
    println!(
        "project metadata repaired: input={} output_dir={}",
        project_input.display(),
        output_dir.display()
    );
    println!(
        "{}",
        render_project_metadata_compact_summary(&repaired_summary)
    );

    Ok(())
}
