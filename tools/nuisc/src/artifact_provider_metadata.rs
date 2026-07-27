use std::collections::BTreeSet;

pub const ARTIFACT_PROVIDER_METADATA_SCOPE_CONTRACT: &str =
    "nuis-artifact-provider-metadata-scope-v1";

const SCOPE_PREFIX: &str = "@scope(";
const SCOPE_SEPARATOR: &str = ")|";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtifactProviderMetadataEntry<'a> {
    pub provider_value: &'a str,
    pub domain_family: Option<&'a str>,
    pub trace_id: Option<&'a str>,
}

impl ArtifactProviderMetadataEntry<'_> {
    pub fn matches(&self, domain_family: &str, trace_id: &str) -> bool {
        self.domain_family
            .is_none_or(|expected| expected == domain_family)
            && self.trace_id.is_none_or(|expected| expected == trace_id)
    }

    pub fn is_scoped(&self) -> bool {
        self.domain_family.is_some() || self.trace_id.is_some()
    }
}

pub fn parse_artifact_provider_metadata_entry(
    value: &str,
) -> Result<ArtifactProviderMetadataEntry<'_>, String> {
    if !value.starts_with(SCOPE_PREFIX) {
        return Ok(ArtifactProviderMetadataEntry {
            provider_value: value,
            domain_family: None,
            trace_id: None,
        });
    }
    let scoped = value
        .strip_prefix(SCOPE_PREFIX)
        .expect("scope prefix checked");
    let (selectors, provider_value) = scoped
        .split_once(SCOPE_SEPARATOR)
        .ok_or_else(|| "scoped metadata is missing `)|`".to_owned())?;
    if selectors.is_empty() {
        return Err("scoped metadata must declare at least one selector".to_owned());
    }
    if provider_value.is_empty() {
        return Err("scoped metadata has an empty provider value".to_owned());
    }

    let mut domain_family = None;
    let mut trace_id = None;
    for selector in selectors.split(',') {
        let (key, selector_value) = selector
            .split_once('=')
            .ok_or_else(|| format!("malformed metadata scope selector `{selector}`"))?;
        validate_scope_value(key, selector_value)?;
        match key {
            "domain" => {
                if domain_family.replace(selector_value).is_some() {
                    return Err("duplicate metadata scope selector `domain`".to_owned());
                }
            }
            "trace" => {
                if trace_id.replace(selector_value).is_some() {
                    return Err("duplicate metadata scope selector `trace`".to_owned());
                }
            }
            _ => return Err(format!("unsupported metadata scope selector `{key}`")),
        }
    }
    Ok(ArtifactProviderMetadataEntry {
        provider_value,
        domain_family,
        trace_id,
    })
}

pub fn validate_artifact_provider_metadata(values: &[String]) -> Result<(), String> {
    if values.len() > 64 {
        return Err("more than 64 artifact_provider_metadata entries".to_owned());
    }
    let mut seen = BTreeSet::new();
    for value in values {
        if value.is_empty()
            || value.len() > 512
            || value.contains([';', '\n', '\r'])
            || !value.is_ascii()
        {
            return Err(format!(
                "invalid artifact_provider_metadata entry `{value}`"
            ));
        }
        parse_artifact_provider_metadata_entry(value).map_err(|error| {
            format!("invalid artifact_provider_metadata entry `{value}`: {error}")
        })?;
        if !seen.insert(value) {
            return Err(format!(
                "duplicate artifact_provider_metadata entry `{value}`"
            ));
        }
    }
    Ok(())
}

pub fn project_artifact_provider_metadata_for_trace<'a>(
    values: &'a [String],
    domain_family: &str,
    trace_id: &str,
) -> Result<Vec<&'a str>, String> {
    values
        .iter()
        .map(|value| parse_artifact_provider_metadata_entry(value))
        .filter_map(|entry| match entry {
            Ok(entry) if entry.matches(domain_family, trace_id) => Some(Ok(entry.provider_value)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn validate_scope_value(key: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 192
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':'))
    {
        return Err(format!("invalid metadata scope selector `{key}={value}`"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unscoped_entries_remain_global() {
        let entry =
            parse_artifact_provider_metadata_entry("nuis.pixelmagic:filter-plan=default").unwrap();
        assert!(!entry.is_scoped());
        assert!(entry.matches("shader", "hetero-trace:shader:metal"));
        assert_eq!(entry.provider_value, "nuis.pixelmagic:filter-plan=default");
    }

    #[test]
    fn scoped_entries_match_domain_and_trace_together() {
        let entry = parse_artifact_provider_metadata_entry(
            "@scope(domain=shader,trace=hetero-trace:shader:metal:apple-silicon-gpu)|nuis.pixelmagic:filter-plan=threshold",
        )
        .unwrap();
        assert!(entry.is_scoped());
        assert!(entry.matches("shader", "hetero-trace:shader:metal:apple-silicon-gpu"));
        assert!(!entry.matches("shader", "hetero-trace:shader:metal:secondary-gpu"));
        assert!(!entry.matches("kernel", "hetero-trace:shader:metal:apple-silicon-gpu"));
    }

    #[test]
    fn projection_keeps_order_and_strips_scope_envelopes() {
        let values = vec![
            "@scope(trace=hetero-trace:shader:metal:first)|nuis.pixelmagic:filter-plan=first"
                .to_owned(),
            "nuis.other:key=global".to_owned(),
            "@scope(trace=hetero-trace:shader:metal:second)|nuis.pixelmagic:filter-plan=second"
                .to_owned(),
        ];
        assert_eq!(
            project_artifact_provider_metadata_for_trace(
                &values,
                "shader",
                "hetero-trace:shader:metal:first"
            )
            .unwrap(),
            ["nuis.pixelmagic:filter-plan=first", "nuis.other:key=global"]
        );
        assert_eq!(
            project_artifact_provider_metadata_for_trace(
                &values,
                "shader",
                "hetero-trace:shader:metal:second"
            )
            .unwrap(),
            [
                "nuis.other:key=global",
                "nuis.pixelmagic:filter-plan=second"
            ]
        );
    }

    #[test]
    fn malformed_or_unknown_scope_selectors_fail_closed() {
        for value in [
            "@scope()|nuis.pixelmagic:key=value",
            "@scope(domain)|nuis.pixelmagic:key=value",
            "@scope(package=nuis.pixelmagic)|nuis.pixelmagic:key=value",
            "@scope(domain=shader,domain=kernel)|nuis.pixelmagic:key=value",
        ] {
            assert!(
                parse_artifact_provider_metadata_entry(value).is_err(),
                "`{value}` must fail"
            );
        }
    }
}
