use crate::{
    dev_tensor::dev_tensor_coordinate_key,
    dev_tensor_data::{DEV_TENSOR_CELLS, DEV_TENSOR_EXPECTED_COORDINATES},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

const MILESTONE_MANIFEST_SOURCE: &str = "docs/reference/nuis-development-tensor.milestones.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevTensorMilestoneCoverage {
    pub(crate) status: &'static str,
    pub(crate) source: &'static str,
    pub(crate) schema: String,
    pub(crate) milestone_count: usize,
    pub(crate) milestone_coordinate_count: usize,
    pub(crate) milestone_required_coordinate_count: usize,
    pub(crate) milestone_missing_coordinate_count: usize,
    pub(crate) milestone_untracked_coordinate_count: usize,
    pub(crate) milestone_constant_drift_count: usize,
    pub(crate) first_gap: Option<String>,
    pub(crate) milestone_coordinates: Vec<String>,
    pub(crate) milestone_missing_coordinates: Vec<String>,
    pub(crate) milestone_untracked_coordinates: Vec<String>,
    pub(crate) milestone_constant_drift_coordinates: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevTensorDerivedExpectedCoordinate {
    pub(crate) architecture: String,
    pub(crate) module: String,
    pub(crate) function: String,
    pub(crate) milestone: String,
    pub(crate) required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevTensorDerivedExpectedCoordinates {
    pub(crate) source: &'static str,
    pub(crate) fallback_used: bool,
    pub(crate) error: Option<String>,
    pub(crate) coordinates: Vec<DevTensorDerivedExpectedCoordinate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DevTensorMilestoneManifest {
    schema: String,
    milestones: Vec<DevTensorMilestone>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DevTensorMilestone {
    id: String,
    required: bool,
    coordinates: Vec<String>,
}

pub(crate) fn dev_tensor_milestone_coverage() -> DevTensorMilestoneCoverage {
    let manifest = match load_milestone_manifest() {
        Ok(manifest) => manifest,
        Err(message) => {
            return DevTensorMilestoneCoverage {
                status: "gap",
                source: MILESTONE_MANIFEST_SOURCE,
                schema: "<unavailable>".to_owned(),
                milestone_count: 0,
                milestone_coordinate_count: 0,
                milestone_required_coordinate_count: 0,
                milestone_missing_coordinate_count: 1,
                milestone_untracked_coordinate_count: 0,
                milestone_constant_drift_count: 0,
                first_gap: Some(message),
                milestone_coordinates: Vec::new(),
                milestone_missing_coordinates: Vec::new(),
                milestone_untracked_coordinates: Vec::new(),
                milestone_constant_drift_coordinates: Vec::new(),
            };
        }
    };
    let milestone_coordinates = milestone_coordinate_records(&manifest);
    let milestone_coordinate_keys = milestone_coordinates
        .iter()
        .map(|record| record.key.clone())
        .collect::<BTreeSet<_>>();
    let cell_coordinates = DEV_TENSOR_CELLS
        .iter()
        .map(|cell| dev_tensor_coordinate_key(cell.architecture, cell.module, cell.function))
        .collect::<BTreeSet<_>>();
    let expected_coordinate_records = DEV_TENSOR_EXPECTED_COORDINATES
        .iter()
        .map(|coordinate| {
            (
                dev_tensor_coordinate_key(
                    coordinate.architecture,
                    coordinate.module,
                    coordinate.function,
                ),
                (coordinate.milestone.to_owned(), coordinate.required),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let expected_coordinate_keys = expected_coordinate_records
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let milestone_missing_coordinates = milestone_coordinates
        .iter()
        .filter(|record| !cell_coordinates.contains(&record.key))
        .map(|record| format!("{}:{}", record.key, required_label(record.required)))
        .collect::<Vec<_>>();
    let milestone_untracked_coordinates = cell_coordinates
        .difference(&milestone_coordinate_keys)
        .cloned()
        .collect::<Vec<_>>();
    let mut milestone_constant_drift_coordinates = milestone_coordinates
        .iter()
        .filter_map(
            |record| match expected_coordinate_records.get(&record.key) {
                Some((milestone, required))
                    if milestone == &record.milestone && *required == record.required =>
                {
                    None
                }
                Some((milestone, required)) => Some(format!(
                    "{}:manifest={}:{}:constant={}:{}",
                    record.key,
                    record.milestone,
                    required_label(record.required),
                    milestone,
                    required_label(*required)
                )),
                None => Some(format!("{}:missing-from-constant", record.key)),
            },
        )
        .collect::<Vec<_>>();
    milestone_constant_drift_coordinates.extend(
        expected_coordinate_keys
            .difference(&milestone_coordinate_keys)
            .map(|coordinate| format!("{coordinate}:missing-from-milestone-manifest")),
    );
    let milestone_required_coordinate_count = milestone_coordinates
        .iter()
        .filter(|record| record.required)
        .count();
    let status = if milestone_missing_coordinates.is_empty()
        && milestone_untracked_coordinates.is_empty()
        && milestone_constant_drift_coordinates.is_empty()
    {
        "clean"
    } else {
        "gap"
    };
    let first_gap = milestone_missing_coordinates
        .first()
        .or_else(|| milestone_untracked_coordinates.first())
        .or_else(|| milestone_constant_drift_coordinates.first())
        .cloned();
    DevTensorMilestoneCoverage {
        status,
        source: MILESTONE_MANIFEST_SOURCE,
        schema: manifest.schema,
        milestone_count: manifest.milestones.len(),
        milestone_coordinate_count: milestone_coordinates.len(),
        milestone_required_coordinate_count,
        milestone_missing_coordinate_count: milestone_missing_coordinates.len(),
        milestone_untracked_coordinate_count: milestone_untracked_coordinates.len(),
        milestone_constant_drift_count: milestone_constant_drift_coordinates.len(),
        first_gap,
        milestone_coordinates: milestone_coordinates
            .into_iter()
            .map(|record| {
                format!(
                    "{}:{}:{}",
                    record.milestone,
                    required_label(record.required),
                    record.key
                )
            })
            .collect(),
        milestone_missing_coordinates,
        milestone_untracked_coordinates,
        milestone_constant_drift_coordinates,
    }
}

pub(crate) fn expected_coordinates_from_milestones() -> DevTensorDerivedExpectedCoordinates {
    match load_milestone_manifest() {
        Ok(manifest) => DevTensorDerivedExpectedCoordinates {
            source: MILESTONE_MANIFEST_SOURCE,
            fallback_used: false,
            error: None,
            coordinates: milestone_coordinate_records(&manifest)
                .into_iter()
                .filter_map(|record| {
                    split_coordinate_key(&record.key).map(|(architecture, module, function)| {
                        DevTensorDerivedExpectedCoordinate {
                            architecture,
                            module,
                            function,
                            milestone: record.milestone,
                            required: record.required,
                        }
                    })
                })
                .collect(),
        },
        Err(error) => DevTensorDerivedExpectedCoordinates {
            source: "DEV_TENSOR_EXPECTED_COORDINATES",
            fallback_used: true,
            error: Some(error),
            coordinates: DEV_TENSOR_EXPECTED_COORDINATES
                .iter()
                .map(|coordinate| DevTensorDerivedExpectedCoordinate {
                    architecture: coordinate.architecture.to_owned(),
                    module: coordinate.module.to_owned(),
                    function: coordinate.function.to_owned(),
                    milestone: coordinate.milestone.to_owned(),
                    required: coordinate.required,
                })
                .collect(),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DevTensorMilestoneCoordinate {
    milestone: String,
    required: bool,
    key: String,
}

fn milestone_coordinate_records(
    manifest: &DevTensorMilestoneManifest,
) -> Vec<DevTensorMilestoneCoordinate> {
    manifest
        .milestones
        .iter()
        .flat_map(|milestone| {
            milestone
                .coordinates
                .iter()
                .map(|coordinate| DevTensorMilestoneCoordinate {
                    milestone: milestone.id.clone(),
                    required: milestone.required,
                    key: coordinate.clone(),
                })
        })
        .collect()
}

fn load_milestone_manifest() -> Result<DevTensorMilestoneManifest, String> {
    let source = fs::read_to_string(repo_root().join(MILESTONE_MANIFEST_SOURCE))
        .map_err(|err| format!("{MILESTONE_MANIFEST_SOURCE}:unreadable:{err}"))?;
    parse_milestone_manifest(&source)
}

fn parse_milestone_manifest(source: &str) -> Result<DevTensorMilestoneManifest, String> {
    let mut schema = String::new();
    let mut milestones = Vec::new();
    let mut current = None::<DevTensorMilestone>;
    let mut in_coordinates = false;
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[milestones]]" {
            if let Some(milestone) = current.take() {
                milestones.push(milestone);
            }
            current = Some(DevTensorMilestone {
                id: String::new(),
                required: true,
                coordinates: Vec::new(),
            });
            in_coordinates = false;
            continue;
        }
        if let Some(value) = string_assignment(line, "schema") {
            schema = value;
            continue;
        }
        if let Some(milestone) = current.as_mut() {
            if let Some(value) = string_assignment(line, "id") {
                milestone.id = value;
                continue;
            }
            if let Some(value) = bool_assignment(line, "required") {
                milestone.required = value;
                continue;
            }
            if line == "coordinates = [" {
                in_coordinates = true;
                continue;
            }
            if in_coordinates && line == "]" {
                in_coordinates = false;
                continue;
            }
            if in_coordinates {
                if let Some(coordinate) = quoted_list_value(line) {
                    milestone.coordinates.push(coordinate);
                }
            }
        }
    }
    if let Some(milestone) = current {
        milestones.push(milestone);
    }
    if schema.is_empty() {
        return Err(format!("{MILESTONE_MANIFEST_SOURCE}:missing-schema"));
    }
    if milestones.iter().any(|milestone| milestone.id.is_empty()) {
        return Err(format!("{MILESTONE_MANIFEST_SOURCE}:missing-milestone-id"));
    }
    if milestones
        .iter()
        .any(|milestone| milestone.coordinates.is_empty())
    {
        return Err(format!("{MILESTONE_MANIFEST_SOURCE}:empty-milestone"));
    }
    Ok(DevTensorMilestoneManifest { schema, milestones })
}

fn string_assignment(line: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    line.strip_prefix(&prefix)
        .and_then(|value| value.strip_prefix('"'))
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_owned)
}

fn bool_assignment(line: &str, key: &str) -> Option<bool> {
    let prefix = format!("{key} = ");
    line.strip_prefix(&prefix).and_then(|value| match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    })
}

fn quoted_list_value(line: &str) -> Option<String> {
    line.trim_end_matches(',')
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_owned)
}

fn required_label(required: bool) -> &'static str {
    if required {
        "required"
    } else {
        "optional"
    }
}

fn split_coordinate_key(key: &str) -> Option<(String, String, String)> {
    let mut parts = key.split('/');
    let architecture = parts.next()?;
    let module = parts.next()?;
    let function = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    Some((
        architecture.to_owned(),
        module.to_owned(),
        function.to_owned(),
    ))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn milestone_manifest_derives_expected_coordinates_without_drift() {
        let coverage = dev_tensor_milestone_coverage();
        assert_eq!(coverage.status, "clean");
        assert_eq!(coverage.source, MILESTONE_MANIFEST_SOURCE);
        assert_eq!(coverage.schema, "nuis-dev-tensor-milestones-v1");
        assert_eq!(
            coverage.milestone_coordinate_count,
            DEV_TENSOR_EXPECTED_COORDINATES.len()
        );
        assert_eq!(coverage.milestone_missing_coordinate_count, 0);
        assert_eq!(coverage.milestone_untracked_coordinate_count, 0);
        assert_eq!(coverage.milestone_constant_drift_count, 0);
        assert!(coverage.first_gap.is_none());
        assert!(coverage
            .milestone_coordinates
            .iter()
            .any(|coordinate| coordinate
                == "alpha-governance:required:developer-system/dev-tensor/architecture-module-function-progress-model"));
    }

    #[test]
    fn milestone_manifest_is_primary_expected_coordinate_source() {
        let derived = expected_coordinates_from_milestones();
        assert_eq!(derived.source, MILESTONE_MANIFEST_SOURCE);
        assert!(!derived.fallback_used);
        assert!(derived.error.is_none());
        assert_eq!(
            derived.coordinates.len(),
            DEV_TENSOR_EXPECTED_COORDINATES.len()
        );
        assert!(derived.coordinates.iter().any(|coordinate| {
            coordinate.architecture == "developer-system"
                && coordinate.module == "dev-tensor"
                && coordinate.function == "architecture-module-function-progress-model"
                && coordinate.milestone == "alpha-governance"
                && coordinate.required
        }));
    }
}
