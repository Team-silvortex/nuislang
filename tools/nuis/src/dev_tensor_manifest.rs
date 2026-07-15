use crate::{
    dev_tensor::{dev_tensor_coordinate_key, DevTensorCell},
    dev_tensor_data::DEV_TENSOR_CELLS,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevTensorManifestCoverage {
    pub(crate) status: &'static str,
    pub(crate) source: &'static str,
    pub(crate) manifest_module_count: usize,
    pub(crate) tracked_manifest_module_count: usize,
    pub(crate) manifest_backed_coordinate_count: usize,
    pub(crate) manifest_missing_module_count: usize,
    pub(crate) manifest_untracked_module_count: usize,
    pub(crate) first_gap: Option<String>,
    pub(crate) manifest_backed_coordinates: Vec<String>,
    pub(crate) manifest_missing_modules: Vec<String>,
    pub(crate) manifest_untracked_modules: Vec<String>,
}

pub(crate) fn dev_tensor_manifest_coverage() -> DevTensorManifestCoverage {
    match collect_manifest_modules() {
        Ok(manifest_modules) => manifest_coverage_from_modules(&manifest_modules, DEV_TENSOR_CELLS),
        Err(error) => DevTensorManifestCoverage {
            status: "unavailable",
            source: "stdlib/index.toml",
            manifest_module_count: 0,
            tracked_manifest_module_count: 0,
            manifest_backed_coordinate_count: 0,
            manifest_missing_module_count: 1,
            manifest_untracked_module_count: 0,
            first_gap: Some(error),
            manifest_backed_coordinates: Vec::new(),
            manifest_missing_modules: Vec::new(),
            manifest_untracked_modules: Vec::new(),
        },
    }
}

fn collect_manifest_modules() -> Result<BTreeSet<String>, String> {
    let stdlib_root = nuisc::stdlib_registry::resolve_stdlib_root()?;
    let layout = nuisc::stdlib_registry::load_stdlib_layout(&stdlib_root)?;
    Ok(layout
        .modules
        .into_iter()
        .map(|module| module.name)
        .collect())
}

fn manifest_coverage_from_modules(
    manifest_modules: &BTreeSet<String>,
    cells: &[DevTensorCell],
) -> DevTensorManifestCoverage {
    let tracked_manifest_modules = cells
        .iter()
        .filter(|cell| cell.architecture == "standard-library")
        .map(|cell| cell.module.to_owned())
        .collect::<BTreeSet<_>>();
    let manifest_backed_coordinates = cells
        .iter()
        .filter(|cell| {
            cell.architecture == "standard-library" && manifest_modules.contains(cell.module)
        })
        .map(|cell| dev_tensor_coordinate_key(cell.architecture, cell.module, cell.function))
        .collect::<Vec<_>>();
    let manifest_missing_modules = tracked_manifest_modules
        .difference(manifest_modules)
        .cloned()
        .collect::<Vec<_>>();
    let manifest_untracked_modules = manifest_modules
        .difference(&tracked_manifest_modules)
        .cloned()
        .collect::<Vec<_>>();
    let first_gap = manifest_missing_modules
        .first()
        .map(|module| format!("standard-library/{module}:manifest-missing"));
    DevTensorManifestCoverage {
        status: if manifest_missing_modules.is_empty() {
            "clean"
        } else {
            "gap"
        },
        source: "stdlib/index.toml",
        manifest_module_count: manifest_modules.len(),
        tracked_manifest_module_count: tracked_manifest_modules.len(),
        manifest_backed_coordinate_count: manifest_backed_coordinates.len(),
        manifest_missing_module_count: manifest_missing_modules.len(),
        manifest_untracked_module_count: manifest_untracked_modules.len(),
        first_gap,
        manifest_backed_coordinates,
        manifest_missing_modules,
        manifest_untracked_modules,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_coverage_backs_current_official_stdlib_cells() {
        let coverage = dev_tensor_manifest_coverage();
        assert_eq!(coverage.status, "clean");
        assert_eq!(coverage.source, "stdlib/index.toml");
        assert_eq!(coverage.manifest_missing_module_count, 0);
        assert!(coverage.manifest_module_count >= 5);
        assert!(coverage.tracked_manifest_module_count >= 3);
        assert!(coverage
            .manifest_backed_coordinates
            .iter()
            .any(|coordinate| coordinate == "standard-library/std/host-io-filesystem-text"));
        assert!(coverage
            .manifest_backed_coordinates
            .iter()
            .any(|coordinate| coordinate == "standard-library/pixelmagic/image-processing-lane"));
        assert!(coverage
            .manifest_backed_coordinates
            .iter()
            .any(|coordinate| coordinate == "standard-library/witsage/classical-ml-lane"));
        assert!(coverage
            .manifest_untracked_modules
            .iter()
            .any(|module| module == "core"));
        assert!(coverage
            .manifest_untracked_modules
            .iter()
            .any(|module| module == "ns-nova"));
    }
}
