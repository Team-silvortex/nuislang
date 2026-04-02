use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use nuis_semantics::model::AstModule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisProjectManifest {
    pub name: String,
    pub entry: String,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectModule {
    pub path: PathBuf,
    pub ast: AstModule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedProject {
    pub root: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest: NuisProjectManifest,
    pub entry_path: PathBuf,
    pub entry_source: String,
    pub modules: Vec<ProjectModule>,
}

pub fn is_project_input(path: &Path) -> bool {
    path.is_dir() || path.file_name().and_then(|name| name.to_str()) == Some("nuis.toml")
}

pub fn load_project(input: &Path) -> Result<LoadedProject, String> {
    let manifest_path = if input.is_dir() {
        input.join("nuis.toml")
    } else {
        input.to_path_buf()
    };
    let root = manifest_path
        .parent()
        .ok_or_else(|| format!("project manifest `{}` has no parent directory", manifest_path.display()))?
        .to_path_buf();
    let source = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("failed to read `{}`: {error}", manifest_path.display()))?;
    let manifest = parse_project_manifest(&source, &manifest_path)?;

    let module_specs = if manifest.modules.is_empty() {
        vec![manifest.entry.clone()]
    } else {
        manifest.modules.clone()
    };
    let mut seen_paths = BTreeSet::new();
    let mut modules = Vec::new();
    for spec in module_specs {
        let path = root.join(&spec);
        if !seen_paths.insert(path.clone()) {
            continue;
        }
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
        let ast = crate::frontend::parse_nuis_ast(&source)?;
        modules.push(ProjectModule { path, ast });
    }

    let entry_path = root.join(&manifest.entry);
    let entry_source = fs::read_to_string(&entry_path)
        .map_err(|error| format!("failed to read `{}`: {error}", entry_path.display()))?;

    validate_project_modules(&modules)?;
    validate_project_unit_bindings(&modules)?;
    validate_project_uses(&modules)?;

    Ok(LoadedProject {
        root,
        manifest_path,
        manifest,
        entry_path,
        entry_source,
        modules,
    })
}

fn validate_project_modules(modules: &[ProjectModule]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for module in modules {
        let key = (module.ast.domain.clone(), module.ast.unit.clone());
        if !seen.insert(key.clone()) {
            return Err(format!(
                "duplicate project mod definition for `mod {} {}`",
                key.0, key.1
            ));
        }
    }
    Ok(())
}

fn validate_project_unit_bindings(modules: &[ProjectModule]) -> Result<(), String> {
    for module in modules {
        let manifest = crate::registry::load_manifest_for_domain(
            Path::new("nustar-packages"),
            &module.ast.domain,
        )?;
        crate::registry::validate_unit_binding(&[manifest], &module.ast.domain, &module.ast.unit)?;
    }
    Ok(())
}

fn validate_project_uses(modules: &[ProjectModule]) -> Result<(), String> {
    let local_units = modules
        .iter()
        .map(|module| (module.ast.domain.clone(), module.ast.unit.clone()))
        .collect::<BTreeSet<_>>();
    for module in modules {
        for item in &module.ast.uses {
            if local_units.contains(&(item.domain.clone(), item.unit.clone())) {
                continue;
            }
            let manifest = crate::registry::load_manifest_for_domain(
                Path::new("nustar-packages"),
                &item.domain,
            )?;
            crate::registry::validate_unit_binding(&[manifest], &item.domain, &item.unit)?;
        }
    }
    Ok(())
}

fn parse_project_manifest(source: &str, path: &Path) -> Result<NuisProjectManifest, String> {
    let name = parse_required_string(source, "name", path)?;
    let entry = parse_required_string(source, "entry", path)?;
    let modules = parse_optional_string_array(source, "modules").unwrap_or_default();
    Ok(NuisProjectManifest { name, entry, modules })
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
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let body = rest.trim();
            let body = body.strip_prefix('[')?.strip_suffix(']')?;
            let mut values = Vec::new();
            for item in body.split(',') {
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

fn parse_quoted(raw: &str) -> Option<String> {
    let raw = raw.trim();
    let inner = raw.strip_prefix('"')?.strip_suffix('"')?;
    Some(inner.to_owned())
}
