use crate::lifecycle_execution::CompiledEntryTransferResult;

pub const NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1: &str = "nuis-runtime-lifecycle-entry-context-i64-v1";
pub const NUIS_LIFECYCLE_ENTRY_DISPATCH_ABI_V2: &str =
    "nuis-runtime-lifecycle-entry-dispatch-i64-v2";
pub const NATIVE_LIFECYCLE_ENTRY_CONTEXT_PROTOCOL: &str = "nuis-runtime-lifecycle-entry-context-v1";
pub const NATIVE_LIFECYCLE_ENTRY_CONTEXT_VERSION: u32 = 1;
const NATIVE_LIFECYCLE_ENTRY_CONTEXT_MAGIC: [u8; 8] = *b"NUISCTX1";
const CLOCK_ROOT_BINDING_ID: &str = "runtime.clock-root";
const GLM_ROOT_BINDING_ID: &str = "runtime.glm-root";

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeLifecycleEntryContextV1 {
    magic: [u8; 8],
    abi_version: u32,
    struct_size_bytes: u32,
    plan_identity: u64,
    execution_identity: u64,
    clock_root_handle: u64,
    glm_root_handle: u64,
    scheduler_handle: u64,
    lifecycle_hook_handle: u64,
}

impl NativeLifecycleEntryContextV1 {
    pub fn from_transfer(transfer: &CompiledEntryTransferResult) -> Result<Self, String> {
        let context = Self::derive_from_transfer(transfer)?;
        match transfer.entry_context_protocol.as_deref() {
            Some(NATIVE_LIFECYCLE_ENTRY_CONTEXT_PROTOCOL) => {}
            Some(_) => return Err("native-entry-context:protocol-unsupported".to_owned()),
            None => return Err("native-entry-context:protocol-missing".to_owned()),
        }
        let expected_identity = transfer
            .entry_context_identity_hash
            .as_deref()
            .ok_or_else(|| "native-entry-context:identity-missing".to_owned())?;
        if context.identity_hash() != expected_identity {
            return Err("native-entry-context:identity-mismatch".to_owned());
        }
        crate::runtime_dispatch_binding::validate_transfer_runtime_dispatch(transfer, &context)?;
        Ok(context)
    }

    pub fn protocol(&self) -> &'static str {
        NATIVE_LIFECYCLE_ENTRY_CONTEXT_PROTOCOL
    }

    pub fn abi_version(&self) -> u32 {
        self.abi_version
    }

    pub fn struct_size_bytes(&self) -> u32 {
        self.struct_size_bytes
    }

    pub fn plan_identity(&self) -> u64 {
        self.plan_identity
    }

    pub fn execution_identity(&self) -> u64 {
        self.execution_identity
    }

    pub fn clock_root_handle(&self) -> u64 {
        self.clock_root_handle
    }

    pub fn glm_root_handle(&self) -> u64 {
        self.glm_root_handle
    }

    pub fn scheduler_handle(&self) -> u64 {
        self.scheduler_handle
    }

    pub fn lifecycle_hook_handle(&self) -> u64 {
        self.lifecycle_hook_handle
    }

    pub fn identity_hash(&self) -> String {
        format!("0x{:016x}", self.identity_value())
    }

    pub fn identity_value(&self) -> u64 {
        let mut bytes = Vec::with_capacity(self.struct_size_bytes as usize);
        bytes.extend_from_slice(&self.magic);
        bytes.extend_from_slice(&self.abi_version.to_le_bytes());
        bytes.extend_from_slice(&self.struct_size_bytes.to_le_bytes());
        bytes.extend_from_slice(&self.plan_identity.to_le_bytes());
        bytes.extend_from_slice(&self.execution_identity.to_le_bytes());
        bytes.extend_from_slice(&self.clock_root_handle.to_le_bytes());
        bytes.extend_from_slice(&self.glm_root_handle.to_le_bytes());
        bytes.extend_from_slice(&self.scheduler_handle.to_le_bytes());
        bytes.extend_from_slice(&self.lifecycle_hook_handle.to_le_bytes());
        fnv1a64(&bytes)
    }

    pub fn dispatch_table_identity(&self) -> u64 {
        crate::NativeRuntimeDispatchTableV1::from_context(self).identity()
    }

    pub fn dispatch_capability_mask(&self) -> u64 {
        crate::NativeRuntimeDispatchTableV1::from_context(self).capability_mask()
    }

    pub(crate) fn is_well_formed(&self) -> bool {
        self.magic == NATIVE_LIFECYCLE_ENTRY_CONTEXT_MAGIC
            && self.abi_version == NATIVE_LIFECYCLE_ENTRY_CONTEXT_VERSION
            && self.struct_size_bytes == std::mem::size_of::<Self>() as u32
            && self.plan_identity != 0
            && self.execution_identity != 0
            && self.clock_root_handle != 0
            && self.glm_root_handle != 0
            && self.scheduler_handle != 0
            && self.lifecycle_hook_handle != 0
    }

    pub(crate) fn derive_from_transfer(
        transfer: &CompiledEntryTransferResult,
    ) -> Result<Self, String> {
        if !transfer.ready {
            return Err("native-entry-context:entry-transfer-blocked".to_owned());
        }
        if !transfer
            .entry_abi_contract
            .as_deref()
            .is_some_and(is_supported_lifecycle_entry_abi)
        {
            return Err("native-entry-context:entry-abi-unsupported".to_owned());
        }
        crate::lifecycle_execution::validate_transfer_execution_identity(transfer)?;
        let plan_identity = parse_hash(&transfer.plan_identity_hash, "plan-identity")?;
        let execution_identity =
            parse_hash(&transfer.execution_identity_hash, "execution-identity")?;
        let scheduler_entry = required_text(transfer.scheduler_entry.as_deref(), "scheduler")?;
        let lifecycle_hook = required_text(transfer.lifecycle_hook.as_deref(), "lifecycle-hook")?;
        require_service(&transfer.activated_service_ids, CLOCK_ROOT_BINDING_ID)?;
        require_service(&transfer.activated_service_ids, GLM_ROOT_BINDING_ID)?;
        let execution_hash = &transfer.execution_identity_hash;
        Ok(Self {
            magic: NATIVE_LIFECYCLE_ENTRY_CONTEXT_MAGIC,
            abi_version: NATIVE_LIFECYCLE_ENTRY_CONTEXT_VERSION,
            struct_size_bytes: std::mem::size_of::<Self>() as u32,
            plan_identity,
            execution_identity,
            clock_root_handle: bound_handle(execution_hash, "clock-root", CLOCK_ROOT_BINDING_ID),
            glm_root_handle: bound_handle(execution_hash, "glm-root", GLM_ROOT_BINDING_ID),
            scheduler_handle: bound_handle(execution_hash, "scheduler", scheduler_entry),
            lifecycle_hook_handle: bound_handle(execution_hash, "lifecycle-hook", lifecycle_hook),
        })
    }

    #[cfg(test)]
    pub(crate) fn test_fixture() -> Self {
        Self {
            magic: NATIVE_LIFECYCLE_ENTRY_CONTEXT_MAGIC,
            abi_version: NATIVE_LIFECYCLE_ENTRY_CONTEXT_VERSION,
            struct_size_bytes: std::mem::size_of::<Self>() as u32,
            plan_identity: 0x0101,
            execution_identity: 0x1111_1111_1111_1111,
            clock_root_handle: 0x0202,
            glm_root_handle: 0x0303,
            scheduler_handle: 0x0404,
            lifecycle_hook_handle: 0x0505,
        }
    }
}

pub(crate) fn bind_transfer_context(
    transfer: &mut CompiledEntryTransferResult,
) -> Result<(), String> {
    let context = NativeLifecycleEntryContextV1::derive_from_transfer(transfer)?;
    transfer.entry_context_protocol = Some(NATIVE_LIFECYCLE_ENTRY_CONTEXT_PROTOCOL.to_owned());
    transfer.entry_context_identity_hash = Some(context.identity_hash());
    crate::runtime_dispatch_binding::materialize_transfer_runtime_dispatch(transfer, &context)?;
    Ok(())
}

pub fn is_supported_lifecycle_entry_abi(value: &str) -> bool {
    matches!(
        value,
        NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1 | NUIS_LIFECYCLE_ENTRY_DISPATCH_ABI_V2
    )
}

pub fn is_dispatch_aware_lifecycle_entry_abi(value: &str) -> bool {
    value == NUIS_LIFECYCLE_ENTRY_DISPATCH_ABI_V2
}

fn required_text<'a>(value: Option<&'a str>, field: &str) -> Result<&'a str, String> {
    value
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("native-entry-context:{field}-missing"))
}

fn require_service(services: &[String], service: &str) -> Result<(), String> {
    if services
        .iter()
        .filter(|candidate| candidate.as_str() == service)
        .count()
        == 1
    {
        Ok(())
    } else {
        Err(format!(
            "native-entry-context:service-binding-invalid:{service}"
        ))
    }
}

fn parse_hash(value: &str, field: &str) -> Result<u64, String> {
    value
        .strip_prefix("0x")
        .filter(|digits| digits.len() == 16)
        .and_then(|digits| u64::from_str_radix(digits, 16).ok())
        .ok_or_else(|| format!("native-entry-context:{field}-invalid"))
}

fn bound_handle(execution_identity: &str, kind: &str, value: &str) -> u64 {
    fnv1a64(
        format!("{NATIVE_LIFECYCLE_ENTRY_CONTEXT_PROTOCOL}\t{execution_identity}\t{kind}\t{value}")
            .as_bytes(),
    )
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
    use crate::{COMPILED_ENTRY_TRANSFER_PROTOCOL, NUIS_NATIVE_ENTRY_SECTION_KIND};

    fn ready_transfer() -> CompiledEntryTransferResult {
        let plan_identity_hash = "0x0101010101010101";
        let image_hash = "0x2222222222222222";
        let execution_identity_hash = format!(
            "0x{:016x}",
            fnv1a64(
                format!(
                    "{}\t{}\t{}\t{}",
                    crate::LIFECYCLE_BOOTSTRAP_EXECUTION_IDENTITY_CONTRACT,
                    plan_identity_hash,
                    image_hash,
                    256
                )
                .as_bytes()
            )
        );
        let mut transfer = CompiledEntryTransferResult {
            protocol: COMPILED_ENTRY_TRANSFER_PROTOCOL,
            status: "transfer-ready",
            ready: true,
            plan_identity_hash: plan_identity_hash.to_owned(),
            execution_identity_hash,
            execution_identity_contract: crate::LIFECYCLE_BOOTSTRAP_EXECUTION_IDENTITY_CONTRACT,
            image_hash: Some(image_hash.to_owned()),
            image_size_bytes: Some(256),
            entry_symbol: Some("main".to_owned()),
            entry_section_id: Some("sec.native-entry".to_owned()),
            entry_section_kind: Some(NUIS_NATIVE_ENTRY_SECTION_KIND.to_owned()),
            entry_section_offset: Some(0),
            entry_section_size_bytes: Some(8),
            entry_section_payload_hash: Some("0x3333333333333333".to_owned()),
            entry_abi_contract: Some(NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1.to_owned()),
            entry_machine_arch: Some("aarch64".to_owned()),
            entry_symbol_offset: Some(0),
            entry_symbol_size_bytes: Some(8),
            entry_symbol_payload_hash: Some("0x4444444444444444".to_owned()),
            entry_context_protocol: None,
            entry_context_identity_hash: None,
            lifecycle_hook: Some("on_process_start".to_owned()),
            scheduler_entry: Some("nuis.scheduler.loop.v1".to_owned()),
            runtime_dispatch_import: None,
            consumed_mapped_section_count: 1,
            consumed_applied_relocation_count: 1,
            activated_service_ids: vec![
                CLOCK_ROOT_BINDING_ID.to_owned(),
                GLM_ROOT_BINDING_ID.to_owned(),
            ],
            trace: Vec::new(),
            blockers: Vec::new(),
        };
        bind_transfer_context(&mut transfer).unwrap();
        transfer
    }

    #[test]
    fn fixed_context_binds_transfer_services_and_identity() {
        let transfer = ready_transfer();
        let context = NativeLifecycleEntryContextV1::from_transfer(&transfer).unwrap();
        assert_eq!(std::mem::size_of_val(&context), 64);
        assert_eq!(context.struct_size_bytes(), 64);
        assert_eq!(context.abi_version(), 1);
        assert_ne!(context.clock_root_handle(), context.glm_root_handle());
        assert_eq!(
            transfer.entry_context_identity_hash.as_deref(),
            Some(context.identity_hash().as_str())
        );
    }

    #[test]
    fn context_identity_claim_drift_fails_closed() {
        let mut transfer = ready_transfer();
        transfer.entry_context_identity_hash = Some("0xaaaaaaaaaaaaaaaa".to_owned());
        assert_eq!(
            NativeLifecycleEntryContextV1::from_transfer(&transfer).unwrap_err(),
            "native-entry-context:identity-mismatch"
        );
    }

    #[test]
    fn missing_required_service_cannot_form_context() {
        let mut transfer = ready_transfer();
        transfer
            .activated_service_ids
            .retain(|service| service != GLM_ROOT_BINDING_ID);
        assert_eq!(
            NativeLifecycleEntryContextV1::derive_from_transfer(&transfer).unwrap_err(),
            "native-entry-context:service-binding-invalid:runtime.glm-root"
        );
    }
}
