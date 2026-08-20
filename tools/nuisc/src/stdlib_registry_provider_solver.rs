use std::collections::{BTreeMap, BTreeSet};

use super::stdlib_registry_provider_semver::{
    compare_candidate_versions, parse_requirement, VersionRequirement,
};
use super::{GalaxyResolutionProviderRequirement, StdlibIndexModule};

const MAX_PACKAGES: usize = 256;
const MAX_CANDIDATES_PER_PACKAGE: usize = 128;
const MAX_SEARCH_STEPS: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SolvedCandidate {
    pub(super) candidate: StdlibIndexModule,
    pub(super) direct: bool,
    pub(super) requested_by: Vec<String>,
}

#[derive(Debug, Clone)]
struct Constraint {
    requirement: Option<VersionRequirement>,
    requested_by: String,
}

#[derive(Debug, Clone, Default)]
struct SolverState {
    constraints: BTreeMap<String, Vec<Constraint>>,
    selected: BTreeMap<String, StdlibIndexModule>,
}

struct SolverContext<'a> {
    provider_id: &'a str,
    candidates: &'a BTreeMap<(String, String), StdlibIndexModule>,
    allow_ranges: bool,
}

pub(super) fn solve_candidates(
    provider_id: &str,
    candidates: &BTreeMap<(String, String), StdlibIndexModule>,
    requirements: &[GalaxyResolutionProviderRequirement],
    allow_ranges: bool,
) -> Result<Vec<SolvedCandidate>, String> {
    validate_bounds(candidates)?;
    if requirements.len() > MAX_PACKAGES {
        return Err(format!(
            "Galaxy request exceeds the deterministic closure limit of {MAX_PACKAGES} packages"
        ));
    }
    let mut state = SolverState::default();
    for requirement in requirements {
        let parsed = parse_requirement(&requirement.version_requirement)?;
        require_range_trust(&parsed, allow_ranges)?;
        state
            .constraints
            .entry(requirement.name.clone())
            .or_default()
            .push(Constraint {
                requirement: Some(parsed),
                requested_by: requirement.name.clone(),
            });
    }
    let context = SolverContext {
        provider_id,
        candidates,
        allow_ranges,
    };
    let mut steps = 0;
    let solved = search(&context, state, &mut steps)?;
    render_solution(requirements, solved.selected)
}

pub(super) fn parse_dependency_requirement(
    raw: &str,
) -> Result<(String, Option<VersionRequirement>), String> {
    let (name, requirement) = raw
        .split_once('=')
        .map(|(name, requirement)| (name, Some(requirement)))
        .unwrap_or((raw, None));
    validate_package_name(name)?;
    let requirement = requirement.map(parse_requirement).transpose()?;
    Ok((name.to_owned(), requirement))
}

fn search(
    context: &SolverContext<'_>,
    state: SolverState,
    steps: &mut usize,
) -> Result<SolverState, String> {
    *steps += 1;
    if *steps > MAX_SEARCH_STEPS {
        return Err(format!(
            "Galaxy provider `{}` exceeded the deterministic solver limit of {MAX_SEARCH_STEPS} search steps",
            context.provider_id
        ));
    }
    validate_selected(context, &state)?;
    let Some(name) = next_unresolved(&state) else {
        return Ok(state);
    };
    let constraints = &state.constraints[&name];
    let mut options = matching_candidates(context, &name, constraints)?;
    options.sort_by(|lhs, rhs| {
        compare_candidate_versions(&rhs.version, &lhs.version).then_with(|| lhs.path.cmp(&rhs.path))
    });

    let mut last_error = None;
    for candidate in options {
        let mut branch = state.clone();
        branch.selected.insert(name.clone(), candidate.clone());
        match add_dependencies(context, &mut branch, &candidate)
            .and_then(|()| search(context, branch, steps))
        {
            Ok(solved) => return Ok(solved),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| {
        format!(
            "Galaxy provider `{}` could not resolve `{name}`",
            context.provider_id
        )
    }))
}

fn next_unresolved(state: &SolverState) -> Option<String> {
    state
        .constraints
        .iter()
        .filter(|(name, _)| !state.selected.contains_key(*name))
        .min_by(|(lhs_name, lhs), (rhs_name, rhs)| {
            let lhs_unpinned = !lhs.iter().any(|item| item.requirement.is_some());
            let rhs_unpinned = !rhs.iter().any(|item| item.requirement.is_some());
            lhs_unpinned
                .cmp(&rhs_unpinned)
                .then_with(|| lhs_name.cmp(rhs_name))
        })
        .map(|(name, _)| name.clone())
}

fn matching_candidates(
    context: &SolverContext<'_>,
    name: &str,
    constraints: &[Constraint],
) -> Result<Vec<StdlibIndexModule>, String> {
    let available = context
        .candidates
        .iter()
        .filter(|((candidate_name, _), _)| candidate_name == name)
        .map(|(_, candidate)| candidate.clone())
        .collect::<Vec<_>>();
    if available.is_empty() {
        return Err(format!(
            "Galaxy provider `{}` has no candidate for dependency `{name}` requested by [{}]",
            context.provider_id,
            requester_list(constraints)
        ));
    }
    let pinned = constraints
        .iter()
        .filter_map(|constraint| constraint.requirement.as_ref())
        .collect::<Vec<_>>();
    if pinned.is_empty() && available.len() > 1 {
        return Err(format!(
            "Galaxy provider `{}` has ambiguous unpinned transitive dependency `{name}` requested by `{}`; candidates=[{}]",
            context.provider_id,
            requester_list(constraints),
            version_list(&available)
        ));
    }
    let matching = available
        .iter()
        .filter(|candidate| {
            pinned
                .iter()
                .all(|requirement| requirement.matches(&candidate.version))
        })
        .cloned()
        .collect::<Vec<_>>();
    if matching.is_empty() {
        if pinned.len() == 1 {
            if let Some(exact) = pinned[0].exact() {
                return Err(format!(
                    "Galaxy provider `{}` has no exact candidate `{name}={exact}`",
                    context.provider_id
                ));
            }
        }
        return Err(format!(
            "Galaxy provider `{}` has no candidate for `{name}` satisfying [{}]; available=[{}]",
            context.provider_id,
            requirement_list(&pinned),
            version_list(&available)
        ));
    }
    Ok(matching)
}

fn add_dependencies(
    context: &SolverContext<'_>,
    state: &mut SolverState,
    candidate: &StdlibIndexModule,
) -> Result<(), String> {
    for raw in &candidate.depends_on {
        let (name, requirement) = parse_dependency_requirement(raw)?;
        if let Some(requirement) = &requirement {
            require_range_trust(requirement, context.allow_ranges)?;
        }
        state.constraints.entry(name).or_default().push(Constraint {
            requirement,
            requested_by: candidate.name.clone(),
        });
    }
    if state.constraints.len() > MAX_PACKAGES {
        return Err(format!(
            "Galaxy provider `{}` exceeded the deterministic closure limit of {MAX_PACKAGES} packages",
            context.provider_id
        ));
    }
    Ok(())
}

fn validate_selected(context: &SolverContext<'_>, state: &SolverState) -> Result<(), String> {
    for (name, candidate) in &state.selected {
        let Some(constraints) = state.constraints.get(name) else {
            continue;
        };
        let failed = constraints.iter().filter_map(|item| {
            item.requirement
                .as_ref()
                .filter(|requirement| !requirement.matches(&candidate.version))
        });
        let failed = failed
            .map(VersionRequirement::canonical)
            .collect::<Vec<_>>();
        if !failed.is_empty() {
            return Err(format!(
                "Galaxy provider `{}` selected `{name}={}`, but later constraints [{}] conflict",
                context.provider_id,
                candidate.version,
                failed.join(", ")
            ));
        }
    }
    Ok(())
}

fn render_solution(
    requirements: &[GalaxyResolutionProviderRequirement],
    selected: BTreeMap<String, StdlibIndexModule>,
) -> Result<Vec<SolvedCandidate>, String> {
    let direct = requirements
        .iter()
        .map(|requirement| requirement.name.clone())
        .collect::<BTreeSet<_>>();
    let mut requested_by = BTreeMap::<String, BTreeSet<String>>::new();
    for requirement in requirements {
        requested_by
            .entry(requirement.name.clone())
            .or_default()
            .insert(requirement.name.clone());
    }
    for candidate in selected.values() {
        for raw in &candidate.depends_on {
            let (name, _) = parse_dependency_requirement(raw)?;
            requested_by
                .entry(name)
                .or_default()
                .insert(candidate.name.clone());
        }
    }
    Ok(selected
        .into_iter()
        .map(|(name, candidate)| SolvedCandidate {
            direct: direct.contains(&name),
            requested_by: requested_by
                .remove(&name)
                .unwrap_or_default()
                .into_iter()
                .collect(),
            candidate,
        })
        .collect())
}

fn validate_bounds(
    candidates: &BTreeMap<(String, String), StdlibIndexModule>,
) -> Result<(), String> {
    if candidates.len() > MAX_PACKAGES * MAX_CANDIDATES_PER_PACKAGE {
        return Err(
            "Galaxy provider candidate set exceeds the deterministic global limit".to_owned(),
        );
    }
    let mut counts = BTreeMap::<&str, usize>::new();
    for (name, _) in candidates.keys() {
        let count = counts.entry(name).or_default();
        *count += 1;
        if *count > MAX_CANDIDATES_PER_PACKAGE {
            return Err(format!(
                "Galaxy package `{name}` exceeds the deterministic limit of {MAX_CANDIDATES_PER_PACKAGE} candidates"
            ));
        }
    }
    for candidate in candidates.values() {
        if candidate.depends_on.len() > MAX_PACKAGES {
            return Err(format!(
                "Galaxy candidate `{}={}` exceeds the deterministic dependency-edge limit of {MAX_PACKAGES}",
                candidate.name, candidate.version
            ));
        }
    }
    Ok(())
}

fn require_range_trust(requirement: &VersionRequirement, allow_ranges: bool) -> Result<(), String> {
    if requirement.is_range() && !allow_ranges {
        return Err(format!(
            "Galaxy version range `{}` requires a verified `{}` sidecar",
            requirement.canonical(),
            super::stdlib_registry_provider_trust::GALAXY_CANDIDATE_SET_FILE
        ));
    }
    Ok(())
}

fn validate_package_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return Err(format!(
            "transitive Galaxy dependency name `{name}` must contain only ASCII letters, digits, `.`, `-`, or `_`"
        ));
    }
    Ok(())
}

fn requester_list(constraints: &[Constraint]) -> String {
    constraints
        .iter()
        .map(|item| item.requested_by.as_str())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ")
}

fn requirement_list(requirements: &[&VersionRequirement]) -> String {
    requirements
        .iter()
        .map(|item| item.canonical())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ")
}

fn version_list(candidates: &[StdlibIndexModule]) -> String {
    let mut versions = candidates
        .iter()
        .map(|candidate| candidate.version.as_str())
        .collect::<Vec<_>>();
    versions.sort_by(|lhs, rhs| compare_candidate_versions(lhs, rhs).then_with(|| lhs.cmp(rhs)));
    versions.dedup();
    versions.join(", ")
}
