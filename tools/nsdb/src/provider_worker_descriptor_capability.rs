pub(crate) const PROVIDER_WORKER_DESCRIPTOR_CAPABILITY_CONTRACT: &str =
    "nuis-provider-worker-descriptor-capability-v1";
pub(crate) const PROVIDER_WORKER_DESCRIPTOR_ENVELOPE_LIMIT: usize = 32;
pub(crate) const PROVIDER_WORKER_ADAPTER_CONTROL_ROLE: &str = "control.adapter";
pub(crate) const PROVIDER_WORKER_OUTPUT_DESCRIPTOR_CAPABILITY_CONTRACT: &str =
    "nuis-provider-worker-output-descriptor-capability-v1";
pub(crate) const PROVIDER_WORKER_OUTPUT_DESCRIPTOR_ENVELOPE_LIMIT: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProviderWorkerDescriptorCapability {
    pub(crate) contract: &'static str,
    pub(crate) max_semantic_descriptors: usize,
    pub(crate) max_control_descriptors: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProviderWorkerOutputDescriptorCapability {
    pub(crate) contract: &'static str,
    pub(crate) max_output_descriptors: usize,
}

impl ProviderWorkerOutputDescriptorCapability {
    pub(crate) fn validate(self) -> Result<(), String> {
        if self.contract != PROVIDER_WORKER_OUTPUT_DESCRIPTOR_CAPABILITY_CONTRACT
            || self.max_output_descriptors == 0
            || self.max_output_descriptors > PROVIDER_WORKER_OUTPUT_DESCRIPTOR_ENVELOPE_LIMIT
        {
            return Err("provider worker output descriptor capability is invalid".to_owned());
        }
        Ok(())
    }
}

impl ProviderWorkerDescriptorCapability {
    pub(crate) fn total_limit(self) -> usize {
        self.max_semantic_descriptors + self.max_control_descriptors
    }

    pub(crate) fn validate(self) -> Result<(), String> {
        if self.contract != PROVIDER_WORKER_DESCRIPTOR_CAPABILITY_CONTRACT
            || self.max_semantic_descriptors == 0
            || self.max_control_descriptors == 0
            || self.total_limit() > PROVIDER_WORKER_DESCRIPTOR_ENVELOPE_LIMIT
        {
            return Err("provider worker descriptor capability is invalid".to_owned());
        }
        Ok(())
    }

    pub(crate) fn validate_roles(self, roles: &[&str]) -> Result<(), String> {
        self.validate()?;
        let control_count = roles
            .iter()
            .filter(|role| **role == PROVIDER_WORKER_ADAPTER_CONTROL_ROLE)
            .count();
        let semantic_count = roles.len().saturating_sub(control_count);
        if control_count > self.max_control_descriptors {
            return Err(format!(
                "provider worker control descriptor count {control_count} exceeds negotiated limit {}",
                self.max_control_descriptors
            ));
        }
        if semantic_count > self.max_semantic_descriptors {
            return Err(format!(
                "provider worker semantic descriptor count {semantic_count} exceeds negotiated limit {}",
                self.max_semantic_descriptors
            ));
        }
        if roles.len() > self.total_limit() {
            return Err(format!(
                "provider worker total descriptor count {} exceeds negotiated limit {}",
                roles.len(),
                self.total_limit()
            ));
        }
        if control_count > 0 && roles.last().copied() != Some(PROVIDER_WORKER_ADAPTER_CONTROL_ROLE)
        {
            return Err("provider worker control descriptor must be unique and last".to_owned());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capability(semantic: usize, control: usize) -> ProviderWorkerDescriptorCapability {
        ProviderWorkerDescriptorCapability {
            contract: PROVIDER_WORKER_DESCRIPTOR_CAPABILITY_CONTRACT,
            max_semantic_descriptors: semantic,
            max_control_descriptors: control,
        }
    }

    #[test]
    fn accepts_registered_semantic_and_trailing_control_capacity() {
        capability(3, 1)
            .validate_roles(&["input.0", "input.1", "input.2", "control.adapter"])
            .expect("registered descriptor shape");
    }

    #[test]
    fn rejects_capacity_overflow_before_transport() {
        let error = capability(2, 1)
            .validate_roles(&["input.0", "input.1", "input.2"])
            .expect_err("semantic overflow");
        assert!(error.contains("exceeds negotiated limit 2"));
    }

    #[test]
    fn rejects_non_trailing_control_descriptor() {
        let error = capability(3, 1)
            .validate_roles(&["input.0", "control.adapter", "input.1"])
            .expect_err("control ordering");
        assert!(error.contains("unique and last"));
    }

    #[test]
    fn output_capacity_rejects_zero_or_physical_overflow() {
        for max_output_descriptors in [0, PROVIDER_WORKER_OUTPUT_DESCRIPTOR_ENVELOPE_LIMIT + 1] {
            let capability = ProviderWorkerOutputDescriptorCapability {
                contract: PROVIDER_WORKER_OUTPUT_DESCRIPTOR_CAPABILITY_CONTRACT,
                max_output_descriptors,
            };
            assert!(capability.validate().is_err());
        }
    }
}
