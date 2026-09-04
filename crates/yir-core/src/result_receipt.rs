use crate::{
    ExecutionState, Node, ProviderCompletionEvidencePolicy, ProviderCompletionRegistration, Value,
    YirResultFamily,
};

pub const PROVIDER_COMPLETION_RECEIPT_CONTRACT: &str = "nuis-yir-provider-completion-receipt-v1";
pub const PROVIDER_PHYSICAL_COMPLETION_CONTRACT: &str = "nuis-yir-provider-physical-completion-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderCompletionReceipt {
    pub token: i64,
    pub completion_clock: i64,
    pub root: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderCompletionWitness {
    pub family: YirResultFamily,
    pub resource: String,
    pub source: String,
    pub completion_clock: i64,
    pub clock_domain: String,
    pub clock_kind: ProviderCompletionClockKind,
    pub physical_source_clock_domain: Option<String>,
    pub physical_fence_source: Option<String>,
    pub physical_source_clock: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderCompletionClockKind {
    RuntimeOrder,
    PhysicalFence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderPhysicalCompletion {
    pub target_clock_domain: String,
    pub source_clock_domain: String,
    pub fence_source: String,
    pub source_clock: i64,
}

impl ProviderPhysicalCompletion {
    pub fn new(
        target_clock_domain: impl Into<String>,
        source_clock_domain: impl Into<String>,
        fence_source: impl Into<String>,
        source_clock: i64,
    ) -> Result<Self, String> {
        let value = Self {
            target_clock_domain: target_clock_domain.into(),
            source_clock_domain: source_clock_domain.into(),
            fence_source: fence_source.into(),
            source_clock,
        };
        validate_completion_identity("target clock domain", &value.target_clock_domain)?;
        validate_completion_identity("source clock domain", &value.source_clock_domain)?;
        validate_completion_identity("fence source", &value.fence_source)?;
        if value.source_clock <= 0 {
            return Err("physical provider completion clock must be positive".to_owned());
        }
        Ok(value)
    }

    pub fn to_wire(&self) -> String {
        format!(
            "{PROVIDER_PHYSICAL_COMPLETION_CONTRACT}|{}|{}|{}|{}",
            self.target_clock_domain,
            self.source_clock_domain,
            self.fence_source,
            self.source_clock
        )
    }

    pub fn parse(wire: &str) -> Result<Self, String> {
        let fields = wire.split('|').collect::<Vec<_>>();
        if fields.len() != 5 || fields[0] != PROVIDER_PHYSICAL_COMPLETION_CONTRACT {
            return Err("invalid physical provider completion wire contract".to_owned());
        }
        let source_clock = fields[4]
            .parse::<i64>()
            .map_err(|_| "invalid physical provider completion clock".to_owned())?;
        Self::new(fields[1], fields[2], fields[3], source_clock)
    }
}

impl ExecutionState {
    pub fn begin_registered_provider_completion(&mut self, node: &Node) -> Result<(), String> {
        if self
            .staged_provider_physical_completions
            .contains_key(&node.name)
            || !self.pending_provider_completions.insert(node.name.clone())
        {
            return Err(format!(
                "provider completion transaction for `{}` is already active",
                node.name
            ));
        }
        Ok(())
    }

    pub fn stage_provider_physical_completion(
        &mut self,
        node: &Node,
        completion: ProviderPhysicalCompletion,
    ) -> Result<(), String> {
        if !self.pending_provider_completions.contains(&node.name) {
            return Err(format!(
                "physical provider completion for `{}` has no active transaction",
                node.name
            ));
        }
        if self
            .staged_provider_physical_completions
            .contains_key(&node.name)
        {
            return Err(format!(
                "physical provider completion for `{}` was staged more than once",
                node.name
            ));
        }
        self.staged_provider_physical_completions
            .insert(node.name.clone(), completion);
        Ok(())
    }

    pub fn abort_registered_provider_completion(&mut self, node: &Node) {
        self.pending_provider_completions.remove(&node.name);
        self.staged_provider_physical_completions.remove(&node.name);
    }

    pub fn finish_registered_provider_completion(
        &mut self,
        registration: ProviderCompletionRegistration,
        node: &Node,
    ) -> Result<ProviderCompletionWitness, String> {
        if !self.pending_provider_completions.remove(&node.name) {
            return Err(format!(
                "provider completion transaction for `{}` is not active",
                node.name
            ));
        }
        let physical_completion = self.staged_provider_physical_completions.remove(&node.name);
        validate_completion_identity("registered clock domain", registration.clock_domain)?;
        match physical_completion {
            Some(completion) => self.record_physical_provider_completion(
                registration.family,
                registration.clock_domain,
                node,
                completion,
            ),
            None => match registration.evidence_policy {
                ProviderCompletionEvidencePolicy::RuntimeOrderAllowed => self
                    .record_runtime_provider_completion(
                        registration.family,
                        registration.clock_domain,
                        node,
                    ),
                ProviderCompletionEvidencePolicy::PhysicalFenceRequired => Err(format!(
                    "provider completion for `{}` requires physical fence evidence",
                    node.name
                )),
            },
        }
    }

    pub fn record_provider_completion(
        &mut self,
        family: YirResultFamily,
        node: &Node,
    ) -> Result<ProviderCompletionWitness, String> {
        self.record_runtime_provider_completion(family, "yir.runtime-order.clock.v1", node)
    }

    fn record_runtime_provider_completion(
        &mut self,
        family: YirResultFamily,
        clock_domain: &str,
        node: &Node,
    ) -> Result<ProviderCompletionWitness, String> {
        self.record_provider_completion_witness(
            family,
            clock_domain,
            node,
            ProviderCompletionClockKind::RuntimeOrder,
            None,
        )
    }

    fn record_physical_provider_completion(
        &mut self,
        family: YirResultFamily,
        clock_domain: &str,
        node: &Node,
        completion: ProviderPhysicalCompletion,
    ) -> Result<ProviderCompletionWitness, String> {
        let completion = ProviderPhysicalCompletion::new(
            completion.target_clock_domain,
            completion.source_clock_domain,
            completion.fence_source,
            completion.source_clock,
        )?;
        if completion.target_clock_domain != clock_domain {
            return Err(format!(
                "physical provider completion for `{}` targets clock domain `{}`, not registered domain `{clock_domain}`",
                node.name, completion.target_clock_domain
            ));
        }
        self.record_provider_completion_witness(
            family,
            clock_domain,
            node,
            ProviderCompletionClockKind::PhysicalFence,
            Some(completion),
        )
    }

    fn record_provider_completion_witness(
        &mut self,
        family: YirResultFamily,
        clock_domain: &str,
        node: &Node,
        clock_kind: ProviderCompletionClockKind,
        physical: Option<ProviderPhysicalCompletion>,
    ) -> Result<ProviderCompletionWitness, String> {
        if let Some(previous) = self.provider_completion_witnesses.get(&node.name) {
            let same_binding = previous.family == family
                && previous.resource == node.resource
                && previous.clock_domain == clock_domain
                && previous.clock_kind == clock_kind
                && previous.physical_source_clock_domain.as_deref()
                    == physical
                        .as_ref()
                        .map(|value| value.source_clock_domain.as_str())
                && previous.physical_fence_source.as_deref()
                    == physical.as_ref().map(|value| value.fence_source.as_str());
            if !same_binding {
                return Err(format!(
                    "provider completion source `{}` changed its registered clock binding",
                    node.name
                ));
            }
        }
        if let Some(physical) = physical.as_ref() {
            let frontier_key = (
                physical.source_clock_domain.clone(),
                physical.fence_source.clone(),
            );
            if self
                .provider_physical_clock_frontiers
                .get(&frontier_key)
                .is_some_and(|previous| physical.source_clock <= *previous)
            {
                return Err(format!(
                    "physical provider completion clock {} is stale for `{}`",
                    physical.source_clock, physical.fence_source
                ));
            }
        }
        let completion_clock = self
            .next_provider_completion_clocks
            .get(clock_domain)
            .copied()
            .unwrap_or_default()
            .checked_add(1)
            .ok_or_else(|| format!("provider completion clock `{clock_domain}` overflow"))?;
        let witness = ProviderCompletionWitness {
            family,
            resource: node.resource.clone(),
            source: node.name.clone(),
            completion_clock,
            clock_domain: clock_domain.to_owned(),
            clock_kind,
            physical_source_clock_domain: physical
                .as_ref()
                .map(|value| value.source_clock_domain.clone()),
            physical_fence_source: physical.as_ref().map(|value| value.fence_source.clone()),
            physical_source_clock: physical.as_ref().map(|value| value.source_clock),
        };
        self.next_provider_completion_clocks
            .insert(clock_domain.to_owned(), completion_clock);
        if let Some(physical) = physical {
            self.provider_physical_clock_frontiers.insert(
                (physical.source_clock_domain, physical.fence_source),
                physical.source_clock,
            );
        }
        self.provider_completion_witnesses
            .insert(node.name.clone(), witness.clone());
        Ok(witness)
    }

    pub fn provider_completion_witness(
        &self,
        family: YirResultFamily,
        resource: &str,
        source: &str,
    ) -> Result<Option<&ProviderCompletionWitness>, String> {
        let Some(witness) = self.provider_completion_witnesses.get(source) else {
            return Ok(None);
        };
        if witness.family != family {
            return Err(format!(
                "provider completion witness `{source}` belongs to {}, not {family}",
                witness.family
            ));
        }
        if witness.resource != resource {
            return Err(format!(
                "provider completion witness `{source}` belongs to resource `{}`, not `{resource}`",
                witness.resource
            ));
        }
        Ok(Some(witness))
    }

    pub fn provider_completion_witnesses(
        &self,
    ) -> &std::collections::BTreeMap<String, ProviderCompletionWitness> {
        &self.provider_completion_witnesses
    }
}

fn validate_completion_identity(label: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err(format!("physical provider {label} `{value}` is invalid"));
    }
    Ok(())
}

pub fn issue_provider_completion_receipt(
    family: YirResultFamily,
    resource: &str,
    source: &str,
    state: &str,
    completion_clock: i64,
) -> ProviderCompletionReceipt {
    let root = provider_completion_receipt_root(family, resource, source, state);
    ProviderCompletionReceipt {
        token: provider_completion_receipt_token(root, completion_clock),
        completion_clock,
        root,
    }
}

pub fn issue_observe_completion_receipt(
    node: &Node,
    state: &ExecutionState,
    family: YirResultFamily,
) -> Result<Option<ProviderCompletionReceipt>, String> {
    let completion_clock = match node.op.args.get(2) {
        Some(clock_name) => state.expect_int(clock_name)?,
        None => {
            let Some(witness) =
                state.provider_completion_witness(family, &node.resource, &node.op.args[0])?
            else {
                return Ok(None);
            };
            witness.completion_clock
        }
    };
    Ok(Some(issue_provider_completion_receipt(
        family,
        &node.resource,
        &node.op.args[0],
        &node.op.args[1],
        completion_clock,
    )))
}

pub fn provider_completion_receipt_root(
    family: YirResultFamily,
    resource: &str,
    source: &str,
    state: &str,
) -> i64 {
    let canonical =
        format!("{PROVIDER_COMPLETION_RECEIPT_CONTRACT}\n{family}\n{resource}\n{source}\n{state}");
    positive_i64(fnv1a64(canonical.as_bytes()))
}

pub fn provider_completion_receipt_token(root: i64, completion_clock: i64) -> i64 {
    positive_i64((root as u64) ^ (completion_clock as u64)) | 1
}

pub fn project_provider_completion_receipt(
    receipt: Option<&ProviderCompletionReceipt>,
    field: &str,
) -> Result<Value, String> {
    let receipt = receipt.ok_or_else(|| {
        format!("result has no `{PROVIDER_COMPLETION_RECEIPT_CONTRACT}` metadata")
    })?;
    let value = match field {
        "completion_token" => receipt.token,
        "completion_clock" => receipt.completion_clock,
        "completion_root" => receipt.root,
        other => {
            return Err(format!(
                "unknown provider completion receipt field `{other}`"
            ))
        }
    };
    Ok(Value::Int(value))
}

fn positive_i64(value: u64) -> i64 {
    ((value & i64::MAX as u64).max(1)) as i64
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_receipts_are_stable_nonzero_and_clock_bound() {
        let first = issue_provider_completion_receipt(
            YirResultFamily::Shader,
            "shader0",
            "frame",
            "frame_ready",
            7,
        );
        let repeated = issue_provider_completion_receipt(
            YirResultFamily::Shader,
            "shader0",
            "frame",
            "frame_ready",
            7,
        );
        let next = issue_provider_completion_receipt(
            YirResultFamily::Shader,
            "shader0",
            "frame",
            "frame_ready",
            8,
        );

        assert_eq!(first, repeated);
        assert!(first.root > 0);
        assert!(first.token > 0);
        assert_ne!(first.token, next.token);
        assert_eq!(first.root, next.root);
        assert_eq!(next.completion_clock, 8);
    }

    #[test]
    fn observe_receipt_uses_registered_completion_witness_when_clock_is_implicit() {
        let source = Node {
            name: "frame".to_owned(),
            resource: "shader0".to_owned(),
            op: crate::Operation::parse(
                "shader.draw_instanced",
                vec![
                    "pass".to_owned(),
                    "packet".to_owned(),
                    "3".to_owned(),
                    "1".to_owned(),
                ],
            )
            .unwrap(),
        };
        let observe = Node {
            name: "result".to_owned(),
            resource: "shader0".to_owned(),
            op: crate::Operation::parse(
                "shader.observe",
                vec!["frame".to_owned(), "frame_ready".to_owned()],
            )
            .unwrap(),
        };
        let mut state = ExecutionState::default();
        let witness = state
            .record_provider_completion(YirResultFamily::Shader, &source)
            .unwrap();
        let receipt = issue_observe_completion_receipt(&observe, &state, YirResultFamily::Shader)
            .unwrap()
            .unwrap();

        assert_eq!(witness.completion_clock, 1);
        assert_eq!(receipt.completion_clock, witness.completion_clock);
        assert_eq!(
            receipt,
            issue_provider_completion_receipt(
                YirResultFamily::Shader,
                "shader0",
                "frame",
                "frame_ready",
                1,
            )
        );
    }

    #[test]
    fn completion_witness_rejects_cross_resource_or_family_rebinding() {
        let source = Node {
            name: "frame".to_owned(),
            resource: "shader0".to_owned(),
            op: crate::Operation::parse("shader.const", vec!["1".to_owned()]).unwrap(),
        };
        let mut state = ExecutionState::default();
        state
            .record_provider_completion(YirResultFamily::Shader, &source)
            .unwrap();

        let resource_error = state
            .provider_completion_witness(YirResultFamily::Shader, "shader1", "frame")
            .unwrap_err();
        assert!(resource_error.contains("resource `shader0`, not `shader1`"));
        let family_error = state
            .provider_completion_witness(YirResultFamily::Kernel, "shader0", "frame")
            .unwrap_err();
        assert!(family_error.contains("belongs to shader, not kernel"));
    }

    #[test]
    fn physical_completion_wire_round_trips_and_rejects_invalid_identity() {
        let completion = ProviderPhysicalCompletion::new(
            "shader.clock.frame.v1",
            "apple.mach-continuous.v1",
            "metal.command-buffer.completed",
            91,
        )
        .unwrap();

        assert_eq!(
            ProviderPhysicalCompletion::parse(&completion.to_wire()).unwrap(),
            completion
        );
        assert!(ProviderPhysicalCompletion::new(
            "shader.clock.frame.v1",
            "bad|clock",
            "metal.command-buffer.completed",
            91,
        )
        .is_err());
        assert!(ProviderPhysicalCompletion::new(
            "shader.clock.frame.v1",
            "apple.mach-continuous.v1",
            "metal.command-buffer.completed",
            0,
        )
        .is_err());
    }

    #[test]
    fn physical_completion_maps_to_logical_order_and_rejects_stale_or_rebound_fences() {
        let source = Node {
            name: "frame".to_owned(),
            resource: "shader0".to_owned(),
            op: crate::Operation::parse("shader.const", vec!["1".to_owned()]).unwrap(),
        };
        let registration =
            ProviderCompletionRegistration::new(YirResultFamily::Shader, "shader.clock.frame.v1");
        let physical = |fence_source: &str, source_clock| {
            ProviderPhysicalCompletion::new(
                "shader.clock.frame.v1",
                "apple.mach-continuous.v1",
                fence_source,
                source_clock,
            )
            .unwrap()
        };
        let mut state = ExecutionState::default();

        state.begin_registered_provider_completion(&source).unwrap();
        state
            .stage_provider_physical_completion(
                &source,
                physical("metal.command-buffer.completed", 100),
            )
            .unwrap();
        let first = state
            .finish_registered_provider_completion(registration, &source)
            .unwrap();
        assert_eq!(first.completion_clock, 1);
        assert_eq!(first.clock_domain, "shader.clock.frame.v1");
        assert_eq!(first.clock_kind, ProviderCompletionClockKind::PhysicalFence);
        assert_eq!(first.physical_source_clock, Some(100));

        state.begin_registered_provider_completion(&source).unwrap();
        state
            .stage_provider_physical_completion(
                &source,
                physical("metal.command-buffer.completed", 101),
            )
            .unwrap();
        let second = state
            .finish_registered_provider_completion(registration, &source)
            .unwrap();
        assert_eq!(second.completion_clock, 2);
        assert_eq!(second.physical_source_clock, Some(101));

        state.begin_registered_provider_completion(&source).unwrap();
        state
            .stage_provider_physical_completion(
                &source,
                physical("metal.command-buffer.completed", 100),
            )
            .unwrap();
        let stale = state
            .finish_registered_provider_completion(registration, &source)
            .unwrap_err();
        assert!(stale.contains("clock 100 is stale"));

        state.begin_registered_provider_completion(&source).unwrap();
        state
            .stage_provider_physical_completion(
                &source,
                physical("vulkan.queue-fence.completed", 102),
            )
            .unwrap();
        let rebound = state
            .finish_registered_provider_completion(registration, &source)
            .unwrap_err();
        assert!(rebound.contains("changed its registered clock binding"));
    }

    #[test]
    fn completion_commit_revalidates_registered_and_provider_supplied_clock_identity() {
        let source = Node {
            name: "frame".to_owned(),
            resource: "shader0".to_owned(),
            op: crate::Operation::parse("shader.const", vec!["1".to_owned()]).unwrap(),
        };
        let mut state = ExecutionState::default();

        state.begin_registered_provider_completion(&source).unwrap();
        state
            .stage_provider_physical_completion(
                &source,
                ProviderPhysicalCompletion::new(
                    "shader.clock.frame.v1",
                    "apple.mach-continuous.v1",
                    "metal.command-buffer.completed",
                    1,
                )
                .unwrap(),
            )
            .unwrap();
        let invalid_registration = state
            .finish_registered_provider_completion(
                ProviderCompletionRegistration::new(YirResultFamily::Shader, "bad|clock"),
                &source,
            )
            .unwrap_err();
        assert!(invalid_registration.contains("registered clock domain"));

        state.begin_registered_provider_completion(&source).unwrap();
        let missing_physical = state
            .finish_registered_provider_completion(
                ProviderCompletionRegistration::physical_fence_required(
                    YirResultFamily::Shader,
                    "shader.clock.frame.v1",
                ),
                &source,
            )
            .unwrap_err();
        assert!(missing_physical.contains("requires physical fence evidence"));

        state.begin_registered_provider_completion(&source).unwrap();
        state
            .stage_provider_physical_completion(
                &source,
                ProviderPhysicalCompletion {
                    target_clock_domain: "shader.clock.frame.v1".to_owned(),
                    source_clock_domain: "bad|clock".to_owned(),
                    fence_source: "metal.command-buffer.completed".to_owned(),
                    source_clock: 1,
                },
            )
            .unwrap();
        let invalid_physical = state
            .finish_registered_provider_completion(
                ProviderCompletionRegistration::new(
                    YirResultFamily::Shader,
                    "shader.clock.frame.v1",
                ),
                &source,
            )
            .unwrap_err();
        assert!(invalid_physical.contains("source clock domain"));
        assert!(state.provider_completion_witnesses().is_empty());
    }
}
