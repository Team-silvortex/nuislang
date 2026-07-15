use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DevTensorDriftCheckSpec {
    pub(crate) id: &'static str,
    pub(crate) path: &'static str,
    pub(crate) required_patterns: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevTensorDriftCheck {
    pub(crate) id: &'static str,
    pub(crate) path: &'static str,
    pub(crate) passed: bool,
    pub(crate) missing_patterns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevTensorDriftSummary {
    pub(crate) check_count: usize,
    pub(crate) passed_count: usize,
    pub(crate) failed_count: usize,
    pub(crate) status: &'static str,
    pub(crate) first_failed_check: Option<&'static str>,
    pub(crate) checks: Vec<DevTensorDriftCheck>,
}

pub(crate) fn dev_tensor_drift_summary<'a>(
    specs: impl IntoIterator<Item = &'a DevTensorDriftCheckSpec>,
) -> DevTensorDriftSummary {
    let checks = specs
        .into_iter()
        .map(run_dev_tensor_drift_check)
        .collect::<Vec<_>>();
    let check_count = checks.len();
    let passed_count = checks.iter().filter(|check| check.passed).count();
    let failed_count = check_count.saturating_sub(passed_count);
    let first_failed_check = checks
        .iter()
        .find(|check| !check.passed)
        .map(|check| check.id);
    DevTensorDriftSummary {
        check_count,
        passed_count,
        failed_count,
        status: if failed_count == 0 { "clean" } else { "drift" },
        first_failed_check,
        checks,
    }
}

fn run_dev_tensor_drift_check(spec: &DevTensorDriftCheckSpec) -> DevTensorDriftCheck {
    let path = repo_root().join(spec.path);
    let source = fs::read_to_string(path).unwrap_or_default();
    let missing_patterns = spec
        .required_patterns
        .iter()
        .filter(|pattern| !source.contains(**pattern))
        .map(|pattern| (*pattern).to_owned())
        .collect::<Vec<_>>();
    DevTensorDriftCheck {
        id: spec.id,
        path: spec.path,
        passed: missing_patterns.is_empty(),
        missing_patterns,
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
