use std::collections::BTreeMap;

pub(crate) const PROVIDER_ADAPTER_BINDING_CONTRACT: &str =
    "nuis-provider-request-adapter-binding-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderAdapterBinding {
    pub(crate) provider_family: String,
    pub(crate) execution_requirement: String,
}

pub(crate) fn parse_adapter_binding(
    fields: &BTreeMap<String, String>,
    prefix: &str,
) -> Option<Option<ProviderAdapterBinding>> {
    let Some(contract) = fields.get(&format!("{prefix}contract")) else {
        return Some(None);
    };
    (contract == PROVIDER_ADAPTER_BINDING_CONTRACT).then_some(())?;
    let binding = ProviderAdapterBinding {
        provider_family: fields.get(&format!("{prefix}provider_family"))?.clone(),
        execution_requirement: fields
            .get(&format!("{prefix}execution_requirement"))?
            .clone(),
    };
    (binding.provider_family.split_once(':').is_some()
        && matches!(
            binding.execution_requirement.as_str(),
            "real-device" | "any"
        ))
    .then_some(Some(binding))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_registered_request_adapter_binding() {
        let fields = BTreeMap::from([
            (
                "adapter_contract".to_owned(),
                PROVIDER_ADAPTER_BINDING_CONTRACT.to_owned(),
            ),
            (
                "adapter_provider_family".to_owned(),
                "metal:apple-silicon-gpu".to_owned(),
            ),
            (
                "adapter_execution_requirement".to_owned(),
                "real-device".to_owned(),
            ),
        ]);
        let binding = parse_adapter_binding(&fields, "adapter_")
            .expect("valid descriptor")
            .expect("binding");
        assert_eq!(binding.provider_family, "metal:apple-silicon-gpu");
    }

    #[test]
    fn rejects_invalid_request_adapter_binding() {
        let fields = BTreeMap::from([
            (
                "adapter_contract".to_owned(),
                PROVIDER_ADAPTER_BINDING_CONTRACT.to_owned(),
            ),
            ("adapter_provider_family".to_owned(), "metal".to_owned()),
            (
                "adapter_execution_requirement".to_owned(),
                "maybe".to_owned(),
            ),
        ]);
        assert!(parse_adapter_binding(&fields, "adapter_").is_none());
    }
}
