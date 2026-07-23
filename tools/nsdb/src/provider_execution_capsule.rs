pub(crate) const PROVIDER_EXECUTION_CAPSULE_CONTRACT: &str = "nuis-provider-execution-capsule-v1";
pub(crate) const PROVIDER_EXECUTION_CAPSULE_REGISTRY_SOURCE: &str =
    "builtin-nustar-provider-execution-capsule-registry";
pub(crate) const PROVIDER_EXECUTION_CAPSULE_INVOCATION_MODE: &str =
    "worker-authorized-parent-adapter-v1";
pub(crate) const PROVIDER_EXECUTION_CAPSULE_INVOKER_CONTRACT: &str =
    "nuis-provider-execution-capsule-invoker-v1";
pub(crate) const PROVIDER_EXECUTION_CAPSULE_INVOKER_REGISTRY_SOURCE: &str =
    "builtin-nustar-provider-execution-capsule-invoker-registry";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderExecutionCapsuleRegistration {
    pub(crate) contract: &'static str,
    pub(crate) registry_source: &'static str,
    pub(crate) capsule_id: String,
    pub(crate) capsule_token: String,
    pub(crate) invocation_mode: &'static str,
    pub(crate) input_roles: Vec<String>,
    pub(crate) output_roles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderExecutionCapsuleInvokerRegistration {
    pub(crate) contract: &'static str,
    pub(crate) registry_source: &'static str,
    pub(crate) invoker_id: String,
    pub(crate) invoker_token: String,
    pub(crate) output_carrier_contract: &'static str,
}

pub(crate) fn register_provider_execution_capsule(
    provider_family: &str,
    adapter_id: &str,
    operation_token: &str,
    input_roles: &[String],
    output_roles: &[String],
) -> Option<ProviderExecutionCapsuleRegistration> {
    if provider_family.split_once(':').is_none()
        || !is_capsule_token(adapter_id)
        || !operation_token.starts_with("operation:")
        || !operation_token
            .strip_prefix("operation:")?
            .bytes()
            .all(|byte| byte.is_ascii_digit())
        || input_roles.len() > 16
        || output_roles.is_empty()
        || output_roles.len() > 16
        || !input_roles.iter().all(|role| is_capsule_token(role))
        || !output_roles.iter().all(|role| is_capsule_token(role))
    {
        return None;
    }
    let input_manifest = input_roles.join(",");
    let output_manifest = output_roles.join(",");
    let identity = format!(
        "{provider_family}:{adapter_id}:{operation_token}:{input_manifest}:{output_manifest}"
    );
    let identity_hash = stable_capsule_scalar(identity.as_bytes());
    Some(ProviderExecutionCapsuleRegistration {
        contract: PROVIDER_EXECUTION_CAPSULE_CONTRACT,
        registry_source: PROVIDER_EXECUTION_CAPSULE_REGISTRY_SOURCE,
        capsule_id: format!("capsule:{identity_hash}"),
        capsule_token: format!("capsule-token:{identity_hash}"),
        invocation_mode: PROVIDER_EXECUTION_CAPSULE_INVOCATION_MODE,
        input_roles: input_roles.to_vec(),
        output_roles: output_roles.to_vec(),
    })
}

pub(crate) fn register_provider_execution_capsule_invoker(
    capsule: &ProviderExecutionCapsuleRegistration,
    adapter_id: &str,
) -> Option<ProviderExecutionCapsuleInvokerRegistration> {
    if capsule.contract != PROVIDER_EXECUTION_CAPSULE_CONTRACT
        || capsule.registry_source != PROVIDER_EXECUTION_CAPSULE_REGISTRY_SOURCE
        || !is_capsule_token(adapter_id)
        || capsule.output_roles.is_empty()
    {
        return None;
    }
    let identity = format!(
        "{}:{}:{}:{}",
        capsule.capsule_id,
        adapter_id,
        capsule.capsule_token,
        capsule.output_roles.join(",")
    );
    let identity_hash = stable_capsule_scalar(identity.as_bytes());
    Some(ProviderExecutionCapsuleInvokerRegistration {
        contract: PROVIDER_EXECUTION_CAPSULE_INVOKER_CONTRACT,
        registry_source: PROVIDER_EXECUTION_CAPSULE_INVOKER_REGISTRY_SOURCE,
        invoker_id: format!("capsule-invoker:{identity_hash}"),
        invoker_token: format!("invoker-token:{identity_hash}"),
        output_carrier_contract: "nuis-provider-worker-output-descriptor-v1",
    })
}

pub(crate) fn render_capsule_roles(roles: &[String]) -> String {
    if roles.is_empty() {
        "none".to_owned()
    } else {
        roles.join(",")
    }
}

fn is_capsule_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b':' | b'_' | b'-'))
}

fn stable_capsule_scalar(bytes: &[u8]) -> i64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    i64::try_from(hash & i64::MAX as u64)
        .unwrap_or(i64::MAX)
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capsule_registration_is_open_ended_and_role_bound() {
        let inputs = vec!["input.0".to_owned(), "input.1".to_owned()];
        let outputs = vec!["output.result".to_owned()];
        let first = register_provider_execution_capsule(
            "spirv:vulkan-gpu",
            "spirv.vulkan.real-device",
            "operation:42",
            &inputs,
            &outputs,
        )
        .expect("capsule");
        let repeated = register_provider_execution_capsule(
            "spirv:vulkan-gpu",
            "spirv.vulkan.real-device",
            "operation:42",
            &inputs,
            &outputs,
        )
        .expect("capsule");
        let other = register_provider_execution_capsule(
            "spirv:vulkan-gpu",
            "spirv.vulkan.real-device",
            "operation:42",
            &["input.0".to_owned()],
            &outputs,
        )
        .expect("other capsule");
        assert_eq!(first, repeated);
        assert_ne!(first.capsule_token, other.capsule_token);
        assert_eq!(first.contract, PROVIDER_EXECUTION_CAPSULE_CONTRACT);
        assert_eq!(first.invocation_mode, "worker-authorized-parent-adapter-v1");
        let invoker = register_provider_execution_capsule_invoker(&first, "adapter.generic")
            .expect("invoker");
        let repeated_invoker =
            register_provider_execution_capsule_invoker(&first, "adapter.generic")
                .expect("repeated invoker");
        assert_eq!(invoker, repeated_invoker);
        assert_eq!(
            invoker.contract,
            PROVIDER_EXECUTION_CAPSULE_INVOKER_CONTRACT
        );
        assert_eq!(
            invoker.output_carrier_contract,
            "nuis-provider-worker-output-descriptor-v1"
        );
    }

    #[test]
    fn capsule_registration_rejects_invalid_roles_or_operation_identity() {
        assert!(register_provider_execution_capsule(
            "spirv:vulkan-gpu",
            "spirv.vulkan.real-device",
            "operation:not-a-scalar",
            &[],
            &["output.result".to_owned()],
        )
        .is_none());
        assert!(register_provider_execution_capsule(
            "spirv:vulkan-gpu",
            "spirv.vulkan.real-device",
            "operation:42",
            &["input/invalid".to_owned()],
            &["output.result".to_owned()],
        )
        .is_none());
    }
}
