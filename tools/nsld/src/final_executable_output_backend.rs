#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldBackendArtifactCandidates {
    pub(crate) candidate_count: usize,
    pub(crate) ready_count: usize,
    pub(crate) selection_status: String,
    pub(crate) ordered_candidates: Vec<String>,
    pub(crate) selected_candidate: Option<String>,
    pub(crate) selected_payload_path: Option<String>,
    pub(crate) selection_reason: String,
    pub(crate) first_unready: Option<String>,
    pub(crate) blockers: Vec<String>,
    pub(crate) issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsldBackendArtifactCandidate {
    key: String,
    priority: usize,
    ready: bool,
    missing_signals: Vec<String>,
    payload_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldBackendArtifactAssemblyBoundary {
    pub(crate) status: String,
    pub(crate) selected_payload_path: Option<String>,
    pub(crate) consumed: bool,
    pub(crate) first_blocker: Option<String>,
    pub(crate) blockers: Vec<String>,
    pub(crate) issues: Vec<String>,
}

pub(crate) fn nsld_backend_artifact_candidates(
    plan: &nuisc::linker::LinkPlan,
) -> NsldBackendArtifactCandidates {
    let mut candidates = plan
        .domain_units
        .iter()
        .filter(|unit| unit.kind == "heterogeneous")
        .filter_map(nsld_backend_artifact_candidate)
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        left.priority
            .cmp(&right.priority)
            .then_with(|| left.key.cmp(&right.key))
    });

    let candidate_count = candidates.len();
    let ready_count = candidates
        .iter()
        .filter(|candidate| candidate.ready)
        .count();
    let ordered_candidates = candidates
        .iter()
        .map(|candidate| candidate.key.clone())
        .collect::<Vec<_>>();
    let first_unready = candidates
        .iter()
        .find(|candidate| !candidate.ready)
        .map(|candidate| candidate.key.clone());
    let selected_candidate = candidates
        .iter()
        .find(|candidate| candidate.ready)
        .map(|candidate| candidate.key.clone());
    let selected_payload_path = candidates
        .iter()
        .find(|candidate| candidate.ready)
        .and_then(|candidate| candidate.payload_path.clone());
    let selection_status = if candidate_count == 0 {
        "no-candidates"
    } else if ready_count == candidate_count {
        "ready"
    } else if ready_count > 0 {
        "partial"
    } else {
        "blocked"
    }
    .to_owned();
    let selection_reason = nsld_backend_artifact_selection_reason(
        candidate_count,
        ready_count,
        selected_candidate.as_deref(),
        first_unready.as_deref(),
    )
    .to_owned();
    let mut blockers = Vec::new();
    let mut issues = Vec::new();
    for candidate in candidates.iter().filter(|candidate| !candidate.ready) {
        blockers.push(format!("nustar-backend-artifact:{}:unready", candidate.key));
        for signal in &candidate.missing_signals {
            blockers.push(format!(
                "nustar-backend-artifact:{}:missing:{}",
                candidate.key, signal
            ));
        }
        issues.push(format!(
            "Nustar backend artifact candidate {} is unready: missing {}",
            candidate.key,
            candidate.missing_signals.join(",")
        ));
    }

    NsldBackendArtifactCandidates {
        candidate_count,
        ready_count,
        selection_status,
        ordered_candidates,
        selected_candidate,
        selected_payload_path,
        selection_reason,
        first_unready,
        blockers,
        issues,
    }
}

pub(crate) fn nsld_backend_artifact_assembly_boundary(
    candidates: &NsldBackendArtifactCandidates,
    layout: &super::reports::NsldFinalExecutableLayoutPlanReport,
) -> NsldBackendArtifactAssemblyBoundary {
    let Some(selected_candidate) = candidates.selected_candidate.as_deref() else {
        return NsldBackendArtifactAssemblyBoundary {
            status: "not-applicable".to_owned(),
            selected_payload_path: None,
            consumed: false,
            first_blocker: None,
            blockers: Vec::new(),
            issues: Vec::new(),
        };
    };
    let selected_payload_path = candidates.selected_payload_path.clone();
    let consumed = selected_payload_path.as_ref().is_some_and(|path| {
        layout
            .payloads
            .iter()
            .any(|payload| payload.path == *path && payload.present)
    });
    if consumed {
        return NsldBackendArtifactAssemblyBoundary {
            status: "consumed-by-final-layout".to_owned(),
            selected_payload_path,
            consumed,
            first_blocker: None,
            blockers: Vec::new(),
            issues: Vec::new(),
        };
    }
    let blocker =
        format!("nustar-backend-artifact:{selected_candidate}:not-consumed-by-final-layout");
    NsldBackendArtifactAssemblyBoundary {
        status: "not-consumed-by-final-layout".to_owned(),
        selected_payload_path,
        consumed,
        first_blocker: Some(blocker.clone()),
        blockers: vec![blocker],
        issues: vec![format!(
            "selected backend artifact candidate {selected_candidate} is not consumed by final executable layout payloads"
        )],
    }
}

fn nsld_backend_artifact_selection_reason(
    candidate_count: usize,
    ready_count: usize,
    selected_candidate: Option<&str>,
    first_unready: Option<&str>,
) -> &'static str {
    if candidate_count == 0 {
        return "no-backend-artifact-candidates";
    }
    if selected_candidate.is_some() && ready_count == candidate_count {
        return "selected-first-ready-candidate";
    }
    if selected_candidate.is_some() {
        return "selected-first-ready-candidate-with-later-blockers";
    }
    if first_unready.is_some() {
        return "all-candidates-blocked";
    }
    "selection-unavailable"
}

fn nsld_backend_artifact_candidate(
    unit: &nuisc::linker::LinkPlanDomainUnit,
) -> Option<NsldBackendArtifactCandidate> {
    if unit.backend_family.is_none()
        && unit.target_device.is_none()
        && unit.selected_lowering_target.is_none()
    {
        return None;
    }
    let missing_signals = nsld_backend_artifact_missing_signals(unit);
    Some(NsldBackendArtifactCandidate {
        key: nsld_backend_artifact_key(unit),
        priority: unit.backend_priority.unwrap_or(usize::MAX),
        ready: missing_signals.is_empty(),
        missing_signals,
        payload_path: unit.artifact_payload_blob_path.clone(),
    })
}

fn nsld_backend_artifact_key(unit: &nuisc::linker::LinkPlanDomainUnit) -> String {
    format!(
        "{}:{}:{}",
        unit.domain_family,
        unit.backend_family.as_deref().unwrap_or("none"),
        unit.target_device.as_deref().unwrap_or("none")
    )
}

fn nsld_backend_artifact_missing_signals(unit: &nuisc::linker::LinkPlanDomainUnit) -> Vec<String> {
    let mut missing = Vec::new();
    if unit.backend_family.is_none() {
        missing.push("backend_family".to_owned());
    }
    if unit.target_device.is_none() {
        missing.push("target_device".to_owned());
    }
    if unit.artifact_payload_blob_path.is_none() {
        missing.push("artifact_payload_blob".to_owned());
    }
    if unit.artifact_payload_format.is_none() {
        missing.push("artifact_payload_format".to_owned());
    }
    if unit.artifact_bridge_stub_path.is_none() {
        missing.push("artifact_bridge_stub".to_owned());
    }
    missing
}
