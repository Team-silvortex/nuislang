use super::*;
use std::path::Component;

pub(super) fn validate_galaxy_token(
    field: &str,
    value: &str,
    context: &Path,
) -> Result<(), String> {
    if value.is_empty() || value == "." || value == ".." || value.starts_with('.') {
        return Err(format!(
            "`{}` has unsafe galaxy {field} `{}`",
            context.display(),
            value
        ));
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'+'))
    {
        return Err(format!(
            "`{}` has unsafe galaxy {field} `{}`; expected ascii alphanumeric, `.`, `_`, `-`, or `+`",
            context.display(),
            value
        ));
    }
    if value.contains('/') || value.contains('\\') {
        return Err(format!(
            "`{}` has unsafe galaxy {field} `{}`; path separators are not allowed",
            context.display(),
            value
        ));
    }
    Ok(())
}

pub(super) fn validate_relative_bundle_path(
    field: &str,
    value: &str,
    context: &Path,
) -> Result<PathBuf, String> {
    if value.is_empty() {
        return Err(format!("`{}` has empty galaxy {field}", context.display()));
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(format!(
            "`{}` has unsafe galaxy {field} `{}`; expected relative path",
            context.display(),
            value
        ));
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(format!(
                    "`{}` has unsafe galaxy {field} `{}`; parent, root, and prefix components are not allowed",
                    context.display(),
                    value
                ));
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(format!(
            "`{}` has unsafe galaxy {field} `{}`; expected file path",
            context.display(),
            value
        ));
    }
    Ok(normalized)
}

pub(super) fn validate_path_under_root(
    field: &str,
    path: &Path,
    root: &Path,
    context: &Path,
) -> Result<(), String> {
    let normalized_path = normalize_existing_or_lexical_path(path)?;
    let normalized_root = normalize_existing_or_lexical_path(root)?;
    if !normalized_path.starts_with(&normalized_root) {
        return Err(format!(
            "`{}` has unsafe galaxy {field} `{}`; expected path under `{}`",
            context.display(),
            path.display(),
            root.display()
        ));
    }
    Ok(())
}

fn normalize_existing_or_lexical_path(path: &Path) -> Result<PathBuf, String> {
    if let Ok(canonical) = path.canonicalize() {
        return Ok(canonical);
    }
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
