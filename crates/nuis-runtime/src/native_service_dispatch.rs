use crate::NativeLifecycleEntryContextV1;

pub const NATIVE_RUNTIME_DISPATCH_TABLE_PROTOCOL: &str = "nuis-runtime-service-dispatch-table-v1";
pub const NATIVE_RUNTIME_DISPATCH_TABLE_VERSION: u32 = 1;
pub const NATIVE_RUNTIME_DISPATCH_CAP_CLOCK_ACKNOWLEDGE: u64 = 1 << 0;
pub const NATIVE_RUNTIME_DISPATCH_CAP_SCHEDULER_ACKNOWLEDGE: u64 = 1 << 1;
pub const NATIVE_RUNTIME_DISPATCH_KNOWN_CAPABILITIES: u64 =
    NATIVE_RUNTIME_DISPATCH_CAP_CLOCK_ACKNOWLEDGE
        | NATIVE_RUNTIME_DISPATCH_CAP_SCHEDULER_ACKNOWLEDGE;
pub const NATIVE_RUNTIME_DISPATCH_SLOT_CLOCK_ACKNOWLEDGE: u32 = 1;
pub const NATIVE_RUNTIME_DISPATCH_SLOT_SCHEDULER_ACKNOWLEDGE: u32 = 2;

pub const NATIVE_RUNTIME_DISPATCH_STATUS_ACKNOWLEDGED: i32 = 0;
pub const NATIVE_RUNTIME_DISPATCH_STATUS_INVALID_RECORD: i32 = 1;
pub const NATIVE_RUNTIME_DISPATCH_STATUS_CONTEXT_IDENTITY_MISMATCH: i32 = 2;
pub const NATIVE_RUNTIME_DISPATCH_STATUS_TABLE_IDENTITY_MISMATCH: i32 = 3;
pub const NATIVE_RUNTIME_DISPATCH_STATUS_UNKNOWN_SLOT: i32 = 4;
pub const NATIVE_RUNTIME_DISPATCH_STATUS_CAPABILITY_MISMATCH: i32 = 5;
pub const NATIVE_RUNTIME_DISPATCH_STATUS_CAPABILITY_DENIED: i32 = 6;

const NATIVE_RUNTIME_DISPATCH_TABLE_MAGIC: [u8; 8] = *b"NUISDSP1";

pub type NativeRuntimeDispatchHandlerV1 = unsafe extern "C" fn(
    *const NativeLifecycleEntryContextV1,
    *const NativeRuntimeDispatchTableV1,
    *const NativeRuntimeDispatchRequestV1,
    *mut NativeRuntimeDispatchResponseV1,
) -> i32;

/// Stable target-side record. It contains only ABI scalars and one mediated
/// function entry; no Rust object address or layout crosses the boundary.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NativeRuntimeDispatchTableV1 {
    magic: [u8; 8],
    abi_version: u32,
    struct_size_bytes: u32,
    identity: u64,
    context_identity: u64,
    capability_mask: u64,
    handler: NativeRuntimeDispatchHandlerV1,
}

impl NativeRuntimeDispatchTableV1 {
    pub fn from_context(context: &NativeLifecycleEntryContextV1) -> Self {
        let context_identity = context.identity_value();
        let capability_mask = capability_mask_for_context(context);
        Self {
            magic: NATIVE_RUNTIME_DISPATCH_TABLE_MAGIC,
            abi_version: NATIVE_RUNTIME_DISPATCH_TABLE_VERSION,
            struct_size_bytes: std::mem::size_of::<Self>() as u32,
            identity: derive_table_identity(context_identity, capability_mask),
            context_identity,
            capability_mask,
            handler: dispatch_runtime_service_v1,
        }
    }

    pub fn protocol(&self) -> &'static str {
        NATIVE_RUNTIME_DISPATCH_TABLE_PROTOCOL
    }

    pub fn abi_version(&self) -> u32 {
        self.abi_version
    }

    pub fn struct_size_bytes(&self) -> u32 {
        self.struct_size_bytes
    }

    pub fn identity(&self) -> u64 {
        self.identity
    }

    pub fn context_identity(&self) -> u64 {
        self.context_identity
    }

    pub fn capability_mask(&self) -> u64 {
        self.capability_mask
    }

    pub fn handler(&self) -> NativeRuntimeDispatchHandlerV1 {
        self.handler
    }

    pub fn dispatch(
        &self,
        context: &NativeLifecycleEntryContextV1,
        request: &NativeRuntimeDispatchRequestV1,
    ) -> NativeRuntimeDispatchResponseV1 {
        let mut response = NativeRuntimeDispatchResponseV1::blocked(
            self.identity,
            context.identity_value(),
            request.slot,
            NATIVE_RUNTIME_DISPATCH_STATUS_INVALID_RECORD,
        );
        // SAFETY: all pointers originate from live references to fixed-layout ABI records.
        unsafe {
            (self.handler)(context, self, request, &mut response);
        }
        response
    }

    fn is_well_formed(&self) -> bool {
        self.magic == NATIVE_RUNTIME_DISPATCH_TABLE_MAGIC
            && self.abi_version == NATIVE_RUNTIME_DISPATCH_TABLE_VERSION
            && self.struct_size_bytes == std::mem::size_of::<Self>() as u32
            && self.capability_mask & !NATIVE_RUNTIME_DISPATCH_KNOWN_CAPABILITIES == 0
            && self.identity == derive_table_identity(self.context_identity, self.capability_mask)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeRuntimeDispatchRequestV1 {
    abi_version: u32,
    struct_size_bytes: u32,
    table_identity: u64,
    context_identity: u64,
    slot: u32,
    reserved: u32,
    requested_capability: u64,
}

impl NativeRuntimeDispatchRequestV1 {
    pub fn acknowledge(
        context: &NativeLifecycleEntryContextV1,
        table: &NativeRuntimeDispatchTableV1,
        slot: u32,
    ) -> Self {
        Self {
            abi_version: NATIVE_RUNTIME_DISPATCH_TABLE_VERSION,
            struct_size_bytes: std::mem::size_of::<Self>() as u32,
            table_identity: table.identity,
            context_identity: context.identity_value(),
            slot,
            reserved: 0,
            requested_capability: capability_for_slot(slot).unwrap_or(0),
        }
    }

    pub fn slot(&self) -> u32 {
        self.slot
    }

    pub fn requested_capability(&self) -> u64 {
        self.requested_capability
    }

    fn is_well_formed(&self) -> bool {
        self.abi_version == NATIVE_RUNTIME_DISPATCH_TABLE_VERSION
            && self.struct_size_bytes == std::mem::size_of::<Self>() as u32
            && self.reserved == 0
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeRuntimeDispatchResponseV1 {
    abi_version: u32,
    struct_size_bytes: u32,
    table_identity: u64,
    context_identity: u64,
    slot: u32,
    acknowledged: u32,
    acknowledged_capability: u64,
    status_code: i32,
    reserved: u32,
}

impl NativeRuntimeDispatchResponseV1 {
    pub fn acknowledged(&self) -> bool {
        self.acknowledged == 1 && self.status_code == NATIVE_RUNTIME_DISPATCH_STATUS_ACKNOWLEDGED
    }

    pub fn status_code(&self) -> i32 {
        self.status_code
    }

    pub fn slot(&self) -> u32 {
        self.slot
    }

    pub fn acknowledged_capability(&self) -> u64 {
        self.acknowledged_capability
    }

    fn blocked(table_identity: u64, context_identity: u64, slot: u32, status_code: i32) -> Self {
        Self {
            abi_version: NATIVE_RUNTIME_DISPATCH_TABLE_VERSION,
            struct_size_bytes: std::mem::size_of::<Self>() as u32,
            table_identity,
            context_identity,
            slot,
            acknowledged: 0,
            acknowledged_capability: 0,
            status_code,
            reserved: 0,
        }
    }
}

unsafe extern "C" fn dispatch_runtime_service_v1(
    context: *const NativeLifecycleEntryContextV1,
    table: *const NativeRuntimeDispatchTableV1,
    request: *const NativeRuntimeDispatchRequestV1,
    response: *mut NativeRuntimeDispatchResponseV1,
) -> i32 {
    if response.is_null() {
        return NATIVE_RUNTIME_DISPATCH_STATUS_INVALID_RECORD;
    }
    if context.is_null() || table.is_null() || request.is_null() {
        // SAFETY: response was checked non-null and points to the caller-owned record.
        unsafe {
            (*response).status_code = NATIVE_RUNTIME_DISPATCH_STATUS_INVALID_RECORD;
        }
        return NATIVE_RUNTIME_DISPATCH_STATUS_INVALID_RECORD;
    }

    // SAFETY: the ABI caller promises pointers to readable records for this call.
    let (context, table, request, response) =
        unsafe { (&*context, &*table, &*request, &mut *response) };
    *response = NativeRuntimeDispatchResponseV1::blocked(
        table.identity,
        context.identity_value(),
        request.slot,
        NATIVE_RUNTIME_DISPATCH_STATUS_INVALID_RECORD,
    );

    let status = validate_dispatch(context, table, request);
    if status != NATIVE_RUNTIME_DISPATCH_STATUS_ACKNOWLEDGED {
        response.status_code = status;
        return status;
    }

    let capability = capability_for_slot(request.slot).expect("validated dispatch slot");
    response.acknowledged = 1;
    response.acknowledged_capability = capability;
    response.status_code = NATIVE_RUNTIME_DISPATCH_STATUS_ACKNOWLEDGED;
    record_service_acknowledgment();
    NATIVE_RUNTIME_DISPATCH_STATUS_ACKNOWLEDGED
}

fn validate_dispatch(
    context: &NativeLifecycleEntryContextV1,
    table: &NativeRuntimeDispatchTableV1,
    request: &NativeRuntimeDispatchRequestV1,
) -> i32 {
    if !context.is_well_formed() || !table.is_well_formed() || !request.is_well_formed() {
        return NATIVE_RUNTIME_DISPATCH_STATUS_INVALID_RECORD;
    }
    let context_identity = context.identity_value();
    if table.context_identity != context_identity || request.context_identity != context_identity {
        return NATIVE_RUNTIME_DISPATCH_STATUS_CONTEXT_IDENTITY_MISMATCH;
    }
    if request.table_identity != table.identity {
        return NATIVE_RUNTIME_DISPATCH_STATUS_TABLE_IDENTITY_MISMATCH;
    }
    let Some(capability) = capability_for_slot(request.slot) else {
        return NATIVE_RUNTIME_DISPATCH_STATUS_UNKNOWN_SLOT;
    };
    if request.requested_capability != capability {
        return NATIVE_RUNTIME_DISPATCH_STATUS_CAPABILITY_MISMATCH;
    }
    if table.capability_mask & capability == 0 {
        return NATIVE_RUNTIME_DISPATCH_STATUS_CAPABILITY_DENIED;
    }
    NATIVE_RUNTIME_DISPATCH_STATUS_ACKNOWLEDGED
}

fn capability_mask_for_context(context: &NativeLifecycleEntryContextV1) -> u64 {
    let mut mask = 0;
    if context.clock_root_handle() != 0 {
        mask |= NATIVE_RUNTIME_DISPATCH_CAP_CLOCK_ACKNOWLEDGE;
    }
    if context.scheduler_handle() != 0 {
        mask |= NATIVE_RUNTIME_DISPATCH_CAP_SCHEDULER_ACKNOWLEDGE;
    }
    mask
}

fn capability_for_slot(slot: u32) -> Option<u64> {
    match slot {
        NATIVE_RUNTIME_DISPATCH_SLOT_CLOCK_ACKNOWLEDGE => {
            Some(NATIVE_RUNTIME_DISPATCH_CAP_CLOCK_ACKNOWLEDGE)
        }
        NATIVE_RUNTIME_DISPATCH_SLOT_SCHEDULER_ACKNOWLEDGE => {
            Some(NATIVE_RUNTIME_DISPATCH_CAP_SCHEDULER_ACKNOWLEDGE)
        }
        _ => None,
    }
}

fn derive_table_identity(context_identity: u64, capability_mask: u64) -> u64 {
    let mut bytes = Vec::with_capacity(NATIVE_RUNTIME_DISPATCH_TABLE_PROTOCOL.len() + 20);
    bytes.extend_from_slice(NATIVE_RUNTIME_DISPATCH_TABLE_PROTOCOL.as_bytes());
    bytes.extend_from_slice(&NATIVE_RUNTIME_DISPATCH_TABLE_VERSION.to_le_bytes());
    bytes.extend_from_slice(&context_identity.to_le_bytes());
    bytes.extend_from_slice(&capability_mask.to_le_bytes());
    fnv1a64(&bytes)
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
static SERVICE_ACKNOWLEDGMENT_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
#[cfg(test)]
static TEST_DISPATCH_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
fn record_service_acknowledgment() {
    SERVICE_ACKNOWLEDGMENT_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(not(test))]
fn record_service_acknowledgment() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_exposes_context_bound_capability_scoped_acknowledgments() {
        let _guard = TEST_DISPATCH_LOCK.lock().unwrap();
        SERVICE_ACKNOWLEDGMENT_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
        let context = NativeLifecycleEntryContextV1::test_fixture();
        let table = NativeRuntimeDispatchTableV1::from_context(&context);
        assert_eq!(std::mem::size_of::<NativeRuntimeDispatchTableV1>(), 48);
        assert_eq!(std::mem::size_of::<NativeRuntimeDispatchRequestV1>(), 40);
        assert_eq!(std::mem::size_of::<NativeRuntimeDispatchResponseV1>(), 48);
        assert_eq!(table.abi_version(), 1);
        assert_eq!(table.context_identity(), context.identity_value());
        assert_eq!(table.identity(), context.dispatch_table_identity());
        assert_eq!(
            table.capability_mask(),
            NATIVE_RUNTIME_DISPATCH_KNOWN_CAPABILITIES
        );

        for (slot, capability) in [
            (
                NATIVE_RUNTIME_DISPATCH_SLOT_CLOCK_ACKNOWLEDGE,
                NATIVE_RUNTIME_DISPATCH_CAP_CLOCK_ACKNOWLEDGE,
            ),
            (
                NATIVE_RUNTIME_DISPATCH_SLOT_SCHEDULER_ACKNOWLEDGE,
                NATIVE_RUNTIME_DISPATCH_CAP_SCHEDULER_ACKNOWLEDGE,
            ),
        ] {
            let request = NativeRuntimeDispatchRequestV1::acknowledge(&context, &table, slot);
            let response = table.dispatch(&context, &request);
            assert!(response.acknowledged());
            assert_eq!(response.acknowledged_capability(), capability);
        }
        assert_eq!(
            SERVICE_ACKNOWLEDGMENT_COUNT.load(std::sync::atomic::Ordering::SeqCst),
            2
        );
    }

    #[test]
    fn unknown_or_identity_mismatched_dispatch_invokes_no_service_slot() {
        let _guard = TEST_DISPATCH_LOCK.lock().unwrap();
        SERVICE_ACKNOWLEDGMENT_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
        let context = NativeLifecycleEntryContextV1::test_fixture();
        let table = NativeRuntimeDispatchTableV1::from_context(&context);

        let unknown = NativeRuntimeDispatchRequestV1::acknowledge(&context, &table, u32::MAX);
        let response = table.dispatch(&context, &unknown);
        assert!(!response.acknowledged());
        assert_eq!(
            response.status_code(),
            NATIVE_RUNTIME_DISPATCH_STATUS_UNKNOWN_SLOT
        );

        let mut mismatched = NativeRuntimeDispatchRequestV1::acknowledge(
            &context,
            &table,
            NATIVE_RUNTIME_DISPATCH_SLOT_CLOCK_ACKNOWLEDGE,
        );
        mismatched.context_identity ^= 1;
        let response = table.dispatch(&context, &mismatched);
        assert!(!response.acknowledged());
        assert_eq!(
            response.status_code(),
            NATIVE_RUNTIME_DISPATCH_STATUS_CONTEXT_IDENTITY_MISMATCH
        );
        assert_eq!(
            SERVICE_ACKNOWLEDGMENT_COUNT.load(std::sync::atomic::Ordering::SeqCst),
            0
        );
    }

    #[test]
    fn capability_or_table_identity_drift_fails_before_acknowledgment() {
        let _guard = TEST_DISPATCH_LOCK.lock().unwrap();
        SERVICE_ACKNOWLEDGMENT_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
        let context = NativeLifecycleEntryContextV1::test_fixture();
        let table = NativeRuntimeDispatchTableV1::from_context(&context);
        let mut request = NativeRuntimeDispatchRequestV1::acknowledge(
            &context,
            &table,
            NATIVE_RUNTIME_DISPATCH_SLOT_SCHEDULER_ACKNOWLEDGE,
        );
        request.requested_capability = NATIVE_RUNTIME_DISPATCH_CAP_CLOCK_ACKNOWLEDGE;
        let response = table.dispatch(&context, &request);
        assert_eq!(
            response.status_code(),
            NATIVE_RUNTIME_DISPATCH_STATUS_CAPABILITY_MISMATCH
        );

        request.requested_capability = NATIVE_RUNTIME_DISPATCH_CAP_SCHEDULER_ACKNOWLEDGE;
        request.table_identity ^= 1;
        let response = table.dispatch(&context, &request);
        assert_eq!(
            response.status_code(),
            NATIVE_RUNTIME_DISPATCH_STATUS_TABLE_IDENTITY_MISMATCH
        );

        let mut denied_table = table;
        denied_table.capability_mask &= !NATIVE_RUNTIME_DISPATCH_CAP_SCHEDULER_ACKNOWLEDGE;
        denied_table.identity =
            derive_table_identity(denied_table.context_identity, denied_table.capability_mask);
        let denied = NativeRuntimeDispatchRequestV1::acknowledge(
            &context,
            &denied_table,
            NATIVE_RUNTIME_DISPATCH_SLOT_SCHEDULER_ACKNOWLEDGE,
        );
        let response = denied_table.dispatch(&context, &denied);
        assert_eq!(
            response.status_code(),
            NATIVE_RUNTIME_DISPATCH_STATUS_CAPABILITY_DENIED
        );
        assert_eq!(
            SERVICE_ACKNOWLEDGMENT_COUNT.load(std::sync::atomic::Ordering::SeqCst),
            0
        );
    }
}
