use std::collections::BTreeSet;
use std::path::Path;

use super::{
    NuisProjectManifest, ProjectAbiRequirement, ProjectGalaxyDependency, ProjectGalaxyImport,
    ProjectLink,
};

pub(super) fn parse_project_manifest(
    source: &str,
    path: &Path,
) -> Result<NuisProjectManifest, String> {
    let name = parse_required_string(source, "name", path)?;
    let entry = parse_required_string(source, "entry", path)?;
    let modules = parse_optional_string_array(source, "modules").unwrap_or_default();
    let tests = parse_optional_string_array(source, "tests").unwrap_or_default();
    let links = parse_optional_link_array(source, "links").unwrap_or_default();
    let abi_requirements = parse_optional_abi_array(source, "abi").unwrap_or_default();
    let galaxy_dependencies =
        parse_optional_galaxy_dependency_array(source, "galaxy").unwrap_or_default();
    let galaxy_imports =
        parse_optional_galaxy_import_array(source, "galaxy_imports").unwrap_or_default();
    validate_unique_galaxy_imports(&galaxy_imports, path)?;
    Ok(NuisProjectManifest {
        name,
        entry,
        modules,
        tests,
        links,
        abi_requirements,
        galaxy_dependencies,
        galaxy_imports,
    })
}

fn validate_unique_galaxy_imports(
    imports: &[ProjectGalaxyImport],
    path: &Path,
) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for item in imports {
        let key = format!("{}:{}", item.galaxy, item.library_module);
        if !seen.insert(key.clone()) {
            return Err(format!(
                "project manifest `{}` declares duplicate galaxy_imports entry `{}`",
                path.display(),
                key
            ));
        }
    }
    Ok(())
}

fn parse_required_string(source: &str, key: &str, path: &Path) -> Result<String, String> {
    parse_optional_string(source, key).ok_or_else(|| {
        format!(
            "project manifest `{}` is missing required field `{key}`",
            path.display()
        )
    })
}

fn parse_optional_string(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_quoted(rest);
        }
    }
    None
}

fn parse_optional_string_array(source: &str, key: &str) -> Option<Vec<String>> {
    let prefix = format!("{key} = ");
    let mut lines = source.lines();
    while let Some(raw_line) = lines.next() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let mut collected = rest.trim().to_owned();
            if !collected.contains(']') {
                for next_line in lines.by_ref() {
                    collected.push(' ');
                    collected.push_str(next_line.trim());
                    if next_line.contains(']') {
                        break;
                    }
                }
            }
            let body = collected.trim();
            let body = body.strip_prefix('[')?.strip_suffix(']')?;
            let mut values = Vec::new();
            for item in split_quoted_array_items(body)? {
                let item = item.trim();
                if item.is_empty() {
                    continue;
                }
                values.push(
                    parse_quoted(item)
                        .ok_or_else(|| format!("invalid string array value `{item}`"))
                        .ok()?,
                );
            }
            return Some(values);
        }
    }
    None
}

fn split_quoted_array_items(inner: &str) -> Option<Vec<&str>> {
    let mut items = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut start = 0;
    for (index, ch) in inner.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            ',' if !in_string => {
                items.push(&inner[start..index]);
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    if in_string || escaped {
        return None;
    }
    items.push(&inner[start..]);
    Some(items)
}

fn parse_optional_link_array(source: &str, key: &str) -> Option<Vec<ProjectLink>> {
    let values = parse_optional_string_array(source, key)?;
    let mut links = Vec::new();
    for value in values {
        let parts = value.split("->").map(str::trim).collect::<Vec<_>>();
        if parts.len() < 2 {
            return None;
        }
        let from = parts[0].to_owned();
        let rhs = parts[1];
        let (to, via) = if let Some((to, via)) = rhs.split_once(" via ") {
            (to.trim().to_owned(), Some(via.trim().to_owned()))
        } else {
            (rhs.to_owned(), None)
        };
        links.push(ProjectLink { from, to, via });
    }
    Some(links)
}

fn parse_optional_abi_array(source: &str, key: &str) -> Option<Vec<ProjectAbiRequirement>> {
    let values = parse_optional_string_array(source, key)?;
    let mut items = Vec::new();
    for value in values {
        let Some((domain, abi)) = value.split_once('=') else {
            return None;
        };
        let domain = domain.trim().to_owned();
        let abi = abi.trim().to_owned();
        if domain.is_empty() || abi.is_empty() {
            return None;
        }
        items.push(ProjectAbiRequirement { domain, abi });
    }
    Some(items)
}

fn parse_optional_galaxy_dependency_array(
    source: &str,
    key: &str,
) -> Option<Vec<ProjectGalaxyDependency>> {
    let values = parse_optional_string_array(source, key)?;
    let mut items = Vec::new();
    for value in values {
        let Some((name, version)) = value.split_once('=') else {
            return None;
        };
        let name = name.trim().to_owned();
        let version = version.trim().to_owned();
        if name.is_empty() || version.is_empty() {
            return None;
        }
        items.push(ProjectGalaxyDependency { name, version });
    }
    Some(items)
}

fn parse_optional_galaxy_import_array(source: &str, key: &str) -> Option<Vec<ProjectGalaxyImport>> {
    let values = parse_optional_string_array(source, key)?;
    let mut items = Vec::new();
    for value in values {
        let Some((galaxy, library_module)) = value.split_once(':') else {
            return None;
        };
        let galaxy = galaxy.trim().to_owned();
        let library_module = library_module.trim().to_owned();
        if galaxy.is_empty() || library_module.is_empty() {
            return None;
        }
        items.push(ProjectGalaxyImport {
            galaxy,
            library_module,
        });
    }
    Some(items)
}

fn parse_quoted(raw: &str) -> Option<String> {
    let raw = raw.trim();
    let inner = raw.strip_prefix('"')?.strip_suffix('"')?;
    Some(inner.to_owned())
}

pub(super) fn sanitize_ident(raw: &str) -> String {
    raw.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_array_parser_preserves_commas_inside_quoted_project_values() {
        let values = parse_optional_string_array(
            r#"modules = ["main.ns", "generated/report,with-comma.ns"]"#,
            "modules",
        )
        .expect("array should parse");

        assert_eq!(
            values,
            vec![
                "main.ns".to_owned(),
                "generated/report,with-comma.ns".to_owned()
            ]
        );
    }
}
