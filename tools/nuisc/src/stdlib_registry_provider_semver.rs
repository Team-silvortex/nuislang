use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct SemanticVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RequirementKind {
    Exact(String),
    Range {
        lower: SemanticVersion,
        lower_inclusive: bool,
        upper: SemanticVersion,
        upper_inclusive: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct VersionRequirement {
    canonical: String,
    kind: RequirementKind,
}

impl VersionRequirement {
    pub(super) fn canonical(&self) -> &str {
        &self.canonical
    }

    pub(super) fn is_range(&self) -> bool {
        matches!(self.kind, RequirementKind::Range { .. })
    }

    pub(super) fn exact(&self) -> Option<&str> {
        match &self.kind {
            RequirementKind::Exact(version) => Some(version),
            RequirementKind::Range { .. } => None,
        }
    }

    pub(super) fn matches(&self, candidate: &str) -> bool {
        match &self.kind {
            RequirementKind::Exact(version) => version == candidate,
            RequirementKind::Range {
                lower,
                lower_inclusive,
                upper,
                upper_inclusive,
            } => {
                let Ok(candidate) = SemanticVersion::parse(candidate) else {
                    return false;
                };
                bound_matches(candidate.cmp(lower), *lower_inclusive, true)
                    && bound_matches(candidate.cmp(upper), *upper_inclusive, false)
            }
        }
    }
}

impl SemanticVersion {
    fn parse(raw: &str) -> Result<Self, String> {
        let mut parts = raw.split('.');
        let major = parse_component(parts.next(), raw)?;
        let minor = parse_component(parts.next(), raw)?;
        let patch = parse_component(parts.next(), raw)?;
        if parts.next().is_some() {
            return Err(version_error(raw));
        }
        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    fn caret_upper(self, raw: &str) -> Result<Self, String> {
        if self.major > 0 {
            return Ok(Self {
                major: increment(self.major, raw)?,
                minor: 0,
                patch: 0,
            });
        }
        if self.minor > 0 {
            return Ok(Self {
                major: 0,
                minor: increment(self.minor, raw)?,
                patch: 0,
            });
        }
        Ok(Self {
            major: 0,
            minor: 0,
            patch: increment(self.patch, raw)?,
        })
    }

    fn tilde_upper(self, raw: &str) -> Result<Self, String> {
        Ok(Self {
            major: self.major,
            minor: increment(self.minor, raw)?,
            patch: 0,
        })
    }
}

impl fmt::Display for SemanticVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

pub(super) fn parse_requirement(raw: &str) -> Result<VersionRequirement, String> {
    let raw = raw.trim();
    if let Some(version) = raw.strip_prefix('^') {
        let lower = SemanticVersion::parse(version)?;
        let upper = lower.caret_upper(raw)?;
        return Ok(range_requirement(lower, true, upper, false));
    }
    if let Some(version) = raw.strip_prefix('~') {
        let lower = SemanticVersion::parse(version)?;
        let upper = lower.tilde_upper(raw)?;
        return Ok(range_requirement(lower, true, upper, false));
    }
    if raw.contains(',') || raw.starts_with(['>', '<']) {
        return parse_bounded_range(raw);
    }
    validate_exact_version(raw)?;
    Ok(VersionRequirement {
        canonical: raw.to_owned(),
        kind: RequirementKind::Exact(raw.to_owned()),
    })
}

pub(super) fn validate_candidate_version(raw: &str) -> Result<(), String> {
    validate_exact_version(raw)
}

pub(super) fn compare_candidate_versions(lhs: &str, rhs: &str) -> Ordering {
    match (SemanticVersion::parse(lhs), SemanticVersion::parse(rhs)) {
        (Ok(lhs), Ok(rhs)) => lhs.cmp(&rhs),
        _ => lhs.cmp(rhs),
    }
}

fn parse_bounded_range(raw: &str) -> Result<VersionRequirement, String> {
    let parts = raw.split(',').map(str::trim).collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err(format!(
            "Galaxy version range `{raw}` must contain exactly one lower and one upper bound"
        ));
    }
    let (lower, lower_inclusive) = parse_lower(parts[0], raw)?;
    let (upper, upper_inclusive) = parse_upper(parts[1], raw)?;
    match lower.cmp(&upper) {
        Ordering::Greater => return Err(empty_range_error(raw)),
        Ordering::Equal if !(lower_inclusive && upper_inclusive) => {
            return Err(empty_range_error(raw));
        }
        _ => {}
    }
    Ok(range_requirement(
        lower,
        lower_inclusive,
        upper,
        upper_inclusive,
    ))
}

fn parse_lower(raw: &str, whole: &str) -> Result<(SemanticVersion, bool), String> {
    if let Some(version) = raw.strip_prefix(">=") {
        return SemanticVersion::parse(version).map(|version| (version, true));
    }
    if let Some(version) = raw.strip_prefix('>') {
        return SemanticVersion::parse(version).map(|version| (version, false));
    }
    Err(format!(
        "Galaxy version range `{whole}` must begin with a `>` or `>=` lower bound"
    ))
}

fn parse_upper(raw: &str, whole: &str) -> Result<(SemanticVersion, bool), String> {
    if let Some(version) = raw.strip_prefix("<=") {
        return SemanticVersion::parse(version).map(|version| (version, true));
    }
    if let Some(version) = raw.strip_prefix('<') {
        return SemanticVersion::parse(version).map(|version| (version, false));
    }
    Err(format!(
        "Galaxy version range `{whole}` must end with a `<` or `<=` upper bound"
    ))
}

fn range_requirement(
    lower: SemanticVersion,
    lower_inclusive: bool,
    upper: SemanticVersion,
    upper_inclusive: bool,
) -> VersionRequirement {
    VersionRequirement {
        canonical: format!(
            "{}{},{}{}",
            if lower_inclusive { ">=" } else { ">" },
            lower,
            if upper_inclusive { "<=" } else { "<" },
            upper
        ),
        kind: RequirementKind::Range {
            lower,
            lower_inclusive,
            upper,
            upper_inclusive,
        },
    }
}

fn bound_matches(ordering: Ordering, inclusive: bool, lower: bool) -> bool {
    if lower {
        ordering == Ordering::Greater || (inclusive && ordering == Ordering::Equal)
    } else {
        ordering == Ordering::Less || (inclusive && ordering == Ordering::Equal)
    }
}

fn validate_exact_version(raw: &str) -> Result<(), String> {
    if raw.is_empty()
        || !raw
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return Err(format!(
            "exact Galaxy version `{raw}` must contain only ASCII letters, digits, `.`, `-`, or `_`"
        ));
    }
    Ok(())
}

fn parse_component(raw: Option<&str>, version: &str) -> Result<u64, String> {
    let Some(raw) = raw else {
        return Err(version_error(version));
    };
    if raw.is_empty()
        || !raw.bytes().all(|byte| byte.is_ascii_digit())
        || (raw.len() > 1 && raw.starts_with('0'))
    {
        return Err(version_error(version));
    }
    raw.parse().map_err(|_| version_error(version))
}

fn increment(value: u64, raw: &str) -> Result<u64, String> {
    value.checked_add(1).ok_or_else(|| {
        format!("Galaxy version requirement `{raw}` overflows its deterministic upper bound")
    })
}

fn version_error(raw: &str) -> String {
    format!(
        "Galaxy semantic version `{raw}` must use three numeric components without prerelease or build metadata"
    )
}

fn empty_range_error(raw: &str) -> String {
    format!("Galaxy version range `{raw}` has no selectable versions")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_ranges_are_canonical_and_bounded() {
        let caret = parse_requirement("^1.2.3").unwrap();
        assert_eq!(caret.canonical(), ">=1.2.3,<2.0.0");
        assert!(caret.matches("1.9.9"));
        assert!(!caret.matches("2.0.0"));

        let zero = parse_requirement("^0.2.3").unwrap();
        assert!(zero.matches("0.2.9"));
        assert!(!zero.matches("0.3.0"));

        let tilde = parse_requirement("~2.4.1").unwrap();
        assert!(tilde.matches("2.4.9"));
        assert!(!tilde.matches("2.5.0"));
    }

    #[test]
    fn malformed_or_unbounded_ranges_fail_closed() {
        assert!(parse_requirement(">=1.0.0").unwrap_err().contains("upper"));
        assert!(parse_requirement("1.*").is_err());
        assert!(parse_requirement("1.2").is_ok());
        assert!(parse_requirement("^1.2").is_err());
        assert!(parse_requirement(">=2.0.0,<1.0.0").is_err());
    }
}
