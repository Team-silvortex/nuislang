use std::fs;

pub(crate) fn handle_project_lock_abi(input: std::path::PathBuf) -> Result<(), String> {
    let project = nuisc::project::load_project(&input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let manifest_source = fs::read_to_string(&project.manifest_path).map_err(|error| {
        format!(
            "failed to read `{}`: {error}",
            project.manifest_path.display()
        )
    })?;
    let updated = upsert_abi_block(&manifest_source, &plan.abi_resolution.requirements);
    if updated == manifest_source {
        println!(
            "project abi already locked: {}",
            project.manifest_path.display()
        );
    } else {
        fs::write(&project.manifest_path, updated).map_err(|error| {
            format!(
                "failed to write `{}`: {error}",
                project.manifest_path.display()
            )
        })?;
        println!("locked project abi: {}", project.manifest_path.display());
    }
    println!(
        "project_plan: {}",
        nuisc::project::describe_project_compilation_plan(&plan)
    );
    println!(
        "  mode: {}",
        if plan.abi_resolution.explicit {
            "explicit (normalized)"
        } else {
            "auto -> explicit"
        }
    );
    for item in plan.abi_resolution.requirements {
        println!("  abi: {}={}", item.domain, item.abi);
    }
    Ok(())
}

pub(crate) fn upsert_abi_block(
    source: &str,
    requirements: &[nuisc::project::ProjectAbiRequirement],
) -> String {
    let mut entries = requirements
        .iter()
        .map(|item| (item.domain.clone(), item.abi.clone()))
        .collect::<Vec<_>>();
    entries.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
    let block = render_abi_block(&entries);

    if let Some((start, end)) = find_abi_block_span(source) {
        let mut out = String::new();
        out.push_str(&source[..start]);
        out.push_str(&block);
        out.push_str(&source[end..]);
        out
    } else if source.ends_with('\n') {
        format!("{source}\n{block}")
    } else {
        format!("{source}\n\n{block}")
    }
}

fn render_abi_block(entries: &[(String, String)]) -> String {
    let mut out = String::new();
    out.push_str("abi = [\n");
    for (domain, abi) in entries {
        out.push_str(&format!("  \"{}={}\",\n", domain, abi));
    }
    out.push_str("]\n");
    out
}

pub(crate) fn find_abi_block_span(source: &str) -> Option<(usize, usize)> {
    let mut offset = 0usize;
    let mut start = None::<usize>;
    let mut depth = 0i32;
    let mut seen_open = false;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if start.is_none() && trimmed.starts_with("abi") && trimmed.contains('=') {
            start = Some(offset);
            depth += line.matches('[').count() as i32;
            depth -= line.matches(']').count() as i32;
            seen_open = line.contains('[');
            if seen_open && depth <= 0 {
                return Some((start?, offset + line.len()));
            }
        } else if start.is_some() {
            depth += line.matches('[').count() as i32;
            depth -= line.matches(']').count() as i32;
            if line.contains('[') {
                seen_open = true;
            }
            if seen_open && depth <= 0 {
                return Some((start?, offset + line.len()));
            }
        }
        offset += line.len();
    }
    start.map(|s| (s, source.len()))
}
