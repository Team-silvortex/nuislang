use std::path::{Component, Path, PathBuf};

fn normalize_manifest_path(path: &Path) -> Result<PathBuf, String> {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => out.push(part),
            Component::RootDir => out.push(component.as_os_str()),
            Component::Prefix(prefix) => out.push(prefix.as_os_str()),
            Component::ParentDir => {
                return Err(format!(
                    "path `{}` contains parent-directory traversal",
                    path.display()
                ));
            }
        }
    }
    Ok(out)
}

pub(crate) fn validate_manifest_path_in_output_dir(
    field: &str,
    value: &str,
    output_dir: &str,
    context: &Path,
) -> Result<(), String> {
    let output_path = Path::new(output_dir);
    let candidate_path = Path::new(value);
    if output_path.is_absolute() != candidate_path.is_absolute() {
        return Err(format!(
            "`{}` has unsafe {field} `{}`; path kind must match output_dir `{}`",
            context.display(),
            value,
            output_dir
        ));
    }
    let normalized_output = normalize_manifest_path(output_path).map_err(|error| {
        format!(
            "`{}` has unsafe output_dir `{}` while validating {field}: {error}",
            context.display(),
            output_dir
        )
    })?;
    let normalized_candidate = normalize_manifest_path(candidate_path).map_err(|error| {
        format!(
            "`{}` has unsafe {field} `{}`: {error}",
            context.display(),
            value
        )
    })?;
    if !normalized_candidate.starts_with(&normalized_output) {
        return Err(format!(
            "`{}` has unsafe {field} `{}`; expected path under output_dir `{}`",
            context.display(),
            value,
            output_dir
        ));
    }
    Ok(())
}
