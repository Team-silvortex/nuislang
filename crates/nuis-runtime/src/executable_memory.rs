use crate::{
    CompiledEntryTransferResult, NativeLifecycleEntryContextV1, NativeRuntimeDispatchFrameV1,
    NativeRuntimeDispatchTableV1, NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1,
};

pub const EXECUTABLE_MEMORY_ADAPTER_CONTRACT: &str = "nuis-executable-memory-adapter-v1";
pub const NATIVE_ENTRY_INVOCATION_PROTOCOL: &str = "nuis-native-entry-invocation-v1";
pub const NATIVE_ENTRY_INVOCATION_PERMIT_PROTOCOL: &str = "nuis-native-entry-invocation-permit-v1";
pub const NUIS_NATIVE_ENTRY_SECTION_KIND: &str = "nuis-native-entry-code";
pub const NUIS_MACHINE_ARCH_AARCH64: &str = "aarch64";
pub const NUIS_MACHINE_ARCH_X86_64: &str = "x86_64";

pub fn canonical_machine_arch(value: &str) -> Option<&'static str> {
    match value {
        "aarch64" | "arm64" => Some(NUIS_MACHINE_ARCH_AARCH64),
        "x86_64" | "amd64" => Some(NUIS_MACHINE_ARCH_X86_64),
        _ => None,
    }
}

pub fn native_host_machine_arch() -> Option<&'static str> {
    #[cfg(target_arch = "aarch64")]
    return Some(NUIS_MACHINE_ARCH_AARCH64);
    #[cfg(target_arch = "x86_64")]
    return Some(NUIS_MACHINE_ARCH_X86_64);
    #[allow(unreachable_code)]
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutableEntryRequest<'a> {
    pub execution_identity_hash: &'a str,
    pub section_id: &'a str,
    pub section_kind: &'a str,
    pub expected_code_hash: &'a str,
    pub entry_symbol: &'a str,
    pub entry_offset: usize,
    pub entry_size_bytes: usize,
    pub abi_contract: &'a str,
    pub target_machine_arch: &'a str,
    pub code_bytes: &'a [u8],
}

impl<'a> ExecutableEntryRequest<'a> {
    pub fn from_transfer(
        transfer: &'a CompiledEntryTransferResult,
        code_bytes: &'a [u8],
    ) -> Result<Self, String> {
        if !transfer.ready {
            return Err("executable-memory:entry-transfer-blocked".to_owned());
        }
        Ok(Self {
            execution_identity_hash: &transfer.execution_identity_hash,
            section_id: required_str(
                transfer.entry_section_id.as_deref(),
                "entry-section-missing",
            )?,
            section_kind: required_str(
                transfer.entry_section_kind.as_deref(),
                "entry-section-kind-missing",
            )?,
            expected_code_hash: required_str(
                transfer.entry_symbol_payload_hash.as_deref(),
                "entry-symbol-hash-missing",
            )?,
            entry_symbol: required_str(transfer.entry_symbol.as_deref(), "entry-symbol-missing")?,
            entry_offset: 0,
            entry_size_bytes: transfer
                .entry_symbol_size_bytes
                .filter(|size| *size > 0)
                .ok_or_else(|| "executable-memory:entry-symbol-size-missing".to_owned())?,
            abi_contract: required_str(
                transfer.entry_abi_contract.as_deref(),
                "entry-abi-missing",
            )?,
            target_machine_arch: required_str(
                transfer.entry_machine_arch.as_deref(),
                "entry-machine-arch-missing",
            )?,
            code_bytes,
        })
    }
}

fn required_str<'a>(value: Option<&'a str>, blocker: &str) -> Result<&'a str, String> {
    value
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("executable-memory:{blocker}"))
}

pub trait ExecutableMemoryAdapter {
    fn adapter_id(&self) -> &'static str;

    fn prepare(&self, request: &ExecutableEntryRequest<'_>) -> ExecutableEntryPreparation;
}

pub struct ExecutableEntryPreparation {
    pub protocol: &'static str,
    pub adapter_id: &'static str,
    pub status: &'static str,
    pub ready: bool,
    pub execution_identity_hash: String,
    pub section_id: String,
    pub entry_symbol: String,
    pub target_machine_arch: String,
    pub host_machine_arch: Option<String>,
    pub machine_arch_status: &'static str,
    pub mapping_size_bytes: usize,
    pub protection_status: &'static str,
    pub entry_bounds_status: &'static str,
    pub blockers: Vec<String>,
    entry: Option<Box<dyn OneShotExecutableEntry>>,
}

impl std::fmt::Debug for ExecutableEntryPreparation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ExecutableEntryPreparation")
            .field("protocol", &self.protocol)
            .field("adapter_id", &self.adapter_id)
            .field("status", &self.status)
            .field("ready", &self.ready)
            .field("execution_identity_hash", &self.execution_identity_hash)
            .field("section_id", &self.section_id)
            .field("entry_symbol", &self.entry_symbol)
            .field("target_machine_arch", &self.target_machine_arch)
            .field("host_machine_arch", &self.host_machine_arch)
            .field("machine_arch_status", &self.machine_arch_status)
            .field("mapping_size_bytes", &self.mapping_size_bytes)
            .field("protection_status", &self.protection_status)
            .field("entry_bounds_status", &self.entry_bounds_status)
            .field("blockers", &self.blockers)
            .finish()
    }
}

impl ExecutableEntryPreparation {
    pub fn authorize(
        self,
        permit: NativeEntryInvocationPermit,
        context: NativeLifecycleEntryContextV1,
    ) -> Result<AuthorizedNativeEntry, Box<NativeEntryInvocationResult>> {
        let mut blockers = self.blockers.clone();
        if !self.ready || self.entry.is_none() {
            blockers.push("native-entry-authorization:preparation-blocked".to_owned());
        }
        if permit.execution_identity_hash != self.execution_identity_hash
            || permit.section_id != self.section_id
            || permit.entry_symbol != self.entry_symbol
            || permit.target_machine_arch != self.target_machine_arch
        {
            blockers.push("native-entry-authorization:permit-identity-mismatch".to_owned());
        }
        if permit.context_identity_hash != context.identity_hash() {
            blockers.push("native-entry-authorization:context-identity-mismatch".to_owned());
        }
        let dispatch_table = NativeRuntimeDispatchTableV1::from_context(&context);
        if permit.dispatch_table_identity != dispatch_table.identity()
            || permit.dispatch_capability_mask != dispatch_table.capability_mask()
        {
            blockers.push("native-entry-authorization:dispatch-identity-mismatch".to_owned());
        }
        blockers.sort();
        blockers.dedup();
        if !blockers.is_empty() {
            return Err(Box::new(NativeEntryInvocationResult::blocked(
                self.adapter_id,
                self.execution_identity_hash,
                self.section_id,
                self.entry_symbol,
                blockers,
            )));
        }
        Ok(AuthorizedNativeEntry {
            entry: self.entry.expect("ready preparation owns native entry"),
            frame: NativeRuntimeDispatchFrameV1::scheduler_acknowledgment(context),
        })
    }

    fn blocked(
        adapter_id: &'static str,
        request: &ExecutableEntryRequest<'_>,
        blockers: Vec<String>,
    ) -> Self {
        Self {
            protocol: EXECUTABLE_MEMORY_ADAPTER_CONTRACT,
            adapter_id,
            status: "blocked",
            ready: false,
            execution_identity_hash: request.execution_identity_hash.to_owned(),
            section_id: request.section_id.to_owned(),
            entry_symbol: request.entry_symbol.to_owned(),
            target_machine_arch: request.target_machine_arch.to_owned(),
            host_machine_arch: native_host_machine_arch().map(str::to_owned),
            machine_arch_status: machine_arch_status(request.target_machine_arch),
            mapping_size_bytes: 0,
            protection_status: "not-mapped",
            entry_bounds_status: "blocked",
            blockers,
            entry: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NativeEntryInvocationPermit {
    protocol: &'static str,
    execution_identity_hash: String,
    section_id: String,
    entry_symbol: String,
    target_machine_arch: String,
    context_identity_hash: String,
    dispatch_table_identity: u64,
    dispatch_capability_mask: u64,
}

impl NativeEntryInvocationPermit {
    pub fn from_transfer(
        transfer: &CompiledEntryTransferResult,
        context: &NativeLifecycleEntryContextV1,
    ) -> Result<Self, String> {
        if !transfer.ready {
            return Err("native-entry-permit:entry-transfer-blocked".to_owned());
        }
        let target_machine_arch = permit_required(
            transfer.entry_machine_arch.as_deref(),
            "target-machine-arch-missing",
        )?;
        if canonical_machine_arch(target_machine_arch) != Some(target_machine_arch) {
            return Err("native-entry-permit:target-machine-arch-unsupported".to_owned());
        }
        if native_host_machine_arch() != Some(target_machine_arch) {
            return Err("native-entry-permit:host-machine-arch-mismatch".to_owned());
        }
        if transfer.entry_abi_contract.as_deref() != Some(NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1) {
            return Err("native-entry-permit:entry-abi-unsupported".to_owned());
        }
        let expected_context = NativeLifecycleEntryContextV1::from_transfer(transfer)?;
        if expected_context != *context {
            return Err("native-entry-permit:context-identity-mismatch".to_owned());
        }
        let dispatch_table = NativeRuntimeDispatchTableV1::from_context(context);
        Ok(Self {
            protocol: NATIVE_ENTRY_INVOCATION_PERMIT_PROTOCOL,
            execution_identity_hash: transfer.execution_identity_hash.clone(),
            section_id: permit_required(transfer.entry_section_id.as_deref(), "section-missing")?
                .to_owned(),
            entry_symbol: permit_required(transfer.entry_symbol.as_deref(), "symbol-missing")?
                .to_owned(),
            target_machine_arch: target_machine_arch.to_owned(),
            context_identity_hash: context.identity_hash(),
            dispatch_table_identity: dispatch_table.identity(),
            dispatch_capability_mask: dispatch_table.capability_mask(),
        })
    }

    pub fn protocol(&self) -> &'static str {
        self.protocol
    }

    pub fn context_identity_hash(&self) -> &str {
        &self.context_identity_hash
    }

    pub fn dispatch_table_identity(&self) -> u64 {
        self.dispatch_table_identity
    }

    pub fn dispatch_capability_mask(&self) -> u64 {
        self.dispatch_capability_mask
    }
}

pub struct AuthorizedNativeEntry {
    entry: Box<dyn OneShotExecutableEntry>,
    frame: NativeRuntimeDispatchFrameV1,
}

impl AuthorizedNativeEntry {
    /// # Safety
    /// The mapped bytes must be trusted AOT output implementing the declared entry ABI.
    pub unsafe fn invoke(mut self) -> NativeEntryInvocationResult {
        unsafe { self.entry.invoke(&mut self.frame) }
    }
}

fn permit_required<'a>(value: Option<&'a str>, blocker: &str) -> Result<&'a str, String> {
    value
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("native-entry-permit:{blocker}"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeEntryInvocationResult {
    pub protocol: &'static str,
    pub adapter_id: &'static str,
    pub status: &'static str,
    pub invoked: bool,
    pub execution_identity_hash: String,
    pub section_id: String,
    pub entry_symbol: String,
    pub return_value: Option<i64>,
    pub dispatch_table_identity: Option<u64>,
    pub dispatch_capability_mask: Option<u64>,
    pub dispatch_slot: Option<u32>,
    pub dispatch_status_code: Option<i32>,
    pub dispatch_acknowledged: bool,
    pub blockers: Vec<String>,
}

impl NativeEntryInvocationResult {
    fn blocked(
        adapter_id: &'static str,
        execution_identity_hash: String,
        section_id: String,
        entry_symbol: String,
        blockers: Vec<String>,
    ) -> Self {
        Self {
            protocol: NATIVE_ENTRY_INVOCATION_PROTOCOL,
            adapter_id,
            status: "blocked",
            invoked: false,
            execution_identity_hash,
            section_id,
            entry_symbol,
            return_value: None,
            dispatch_table_identity: None,
            dispatch_capability_mask: None,
            dispatch_slot: None,
            dispatch_status_code: None,
            dispatch_acknowledged: false,
            blockers,
        }
    }
}

trait OneShotExecutableEntry {
    unsafe fn invoke(
        self: Box<Self>,
        frame: &mut NativeRuntimeDispatchFrameV1,
    ) -> NativeEntryInvocationResult;
}

#[derive(Debug, Default)]
pub struct NativeHostExecutableMemoryAdapter;

impl ExecutableMemoryAdapter for NativeHostExecutableMemoryAdapter {
    fn adapter_id(&self) -> &'static str {
        "nuis.native-host-executable-memory.v1"
    }

    fn prepare(&self, request: &ExecutableEntryRequest<'_>) -> ExecutableEntryPreparation {
        let blockers = validate_request(request);
        if !blockers.is_empty() {
            return ExecutableEntryPreparation::blocked(self.adapter_id(), request, blockers);
        }
        prepare_native_host_entry(self.adapter_id(), request)
    }
}

fn validate_request(request: &ExecutableEntryRequest<'_>) -> Vec<String> {
    let mut blockers = Vec::new();
    if !valid_hash(request.execution_identity_hash) {
        blockers.push("executable-memory:execution-identity-invalid".to_owned());
    }
    if request.section_id.is_empty() || request.section_kind.is_empty() {
        blockers.push("executable-memory:section-identity-missing".to_owned());
    }
    if request.section_kind != NUIS_NATIVE_ENTRY_SECTION_KIND {
        blockers.push("executable-memory:section-kind-unsupported".to_owned());
    }
    if request.entry_symbol.is_empty() {
        blockers.push("executable-memory:entry-symbol-missing".to_owned());
    }
    if request.abi_contract != NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1 {
        blockers.push("executable-memory:entry-abi-unsupported".to_owned());
    }
    match machine_arch_status(request.target_machine_arch) {
        "verified-host-match" => {}
        "target-unsupported" => {
            blockers.push("executable-memory:target-machine-arch-unsupported".to_owned())
        }
        "host-unavailable" => {
            blockers.push("executable-memory:host-machine-arch-unavailable".to_owned())
        }
        _ => blockers.push("executable-memory:host-machine-arch-mismatch".to_owned()),
    }
    if request.code_bytes.is_empty()
        || !valid_hash(request.expected_code_hash)
        || fnv1a64_hex(request.code_bytes) != request.expected_code_hash
    {
        blockers.push("executable-memory:section-bytes-unverified".to_owned());
    }
    let entry_end = request.entry_offset.checked_add(request.entry_size_bytes);
    if request.entry_size_bytes == 0 || entry_end.is_none_or(|end| end > request.code_bytes.len()) {
        blockers.push("executable-memory:entry-range-invalid".to_owned());
    }
    #[cfg(target_arch = "aarch64")]
    if !request.entry_offset.is_multiple_of(4) {
        blockers.push("executable-memory:entry-alignment-invalid".to_owned());
    }
    blockers.sort();
    blockers.dedup();
    blockers
}

fn machine_arch_status(target_machine_arch: &str) -> &'static str {
    let Some(target) = canonical_machine_arch(target_machine_arch) else {
        return "target-unsupported";
    };
    if target != target_machine_arch {
        return "target-not-canonical";
    }
    match native_host_machine_arch() {
        Some(host) if host == target => "verified-host-match",
        Some(_) => "host-mismatch",
        None => "host-unavailable",
    }
}

#[cfg(all(unix, any(target_arch = "aarch64", target_arch = "x86_64")))]
fn prepare_native_host_entry(
    adapter_id: &'static str,
    request: &ExecutableEntryRequest<'_>,
) -> ExecutableEntryPreparation {
    match unix::MappedExecutableRegion::new(request.code_bytes) {
        Ok(region) => ExecutableEntryPreparation {
            protocol: EXECUTABLE_MEMORY_ADAPTER_CONTRACT,
            adapter_id,
            status: "ready",
            ready: true,
            execution_identity_hash: request.execution_identity_hash.to_owned(),
            section_id: request.section_id.to_owned(),
            entry_symbol: request.entry_symbol.to_owned(),
            target_machine_arch: request.target_machine_arch.to_owned(),
            host_machine_arch: native_host_machine_arch().map(str::to_owned),
            machine_arch_status: "verified-host-match",
            mapping_size_bytes: request.code_bytes.len(),
            protection_status: "sealed-read-execute",
            entry_bounds_status: "verified",
            blockers: Vec::new(),
            entry: Some(Box::new(UnixExecutableEntry {
                adapter_id,
                execution_identity_hash: request.execution_identity_hash.to_owned(),
                section_id: request.section_id.to_owned(),
                entry_symbol: request.entry_symbol.to_owned(),
                entry_offset: request.entry_offset,
                region,
            })),
        },
        Err(blocker) => ExecutableEntryPreparation::blocked(adapter_id, request, vec![blocker]),
    }
}

#[cfg(not(all(unix, any(target_arch = "aarch64", target_arch = "x86_64"))))]
fn prepare_native_host_entry(
    adapter_id: &'static str,
    request: &ExecutableEntryRequest<'_>,
) -> ExecutableEntryPreparation {
    ExecutableEntryPreparation::blocked(
        adapter_id,
        request,
        vec!["executable-memory:platform-adapter-unavailable".to_owned()],
    )
}

#[cfg(all(unix, any(target_arch = "aarch64", target_arch = "x86_64")))]
struct UnixExecutableEntry {
    adapter_id: &'static str,
    execution_identity_hash: String,
    section_id: String,
    entry_symbol: String,
    entry_offset: usize,
    region: unix::MappedExecutableRegion,
}

#[cfg(all(unix, any(target_arch = "aarch64", target_arch = "x86_64")))]
impl OneShotExecutableEntry for UnixExecutableEntry {
    unsafe fn invoke(
        self: Box<Self>,
        frame: &mut NativeRuntimeDispatchFrameV1,
    ) -> NativeEntryInvocationResult {
        let entry_address = unsafe { self.region.entry_address(self.entry_offset) };
        // SAFETY: preparation verifies the entry range and ABI, the mapping is RX,
        // and ownership of the mapping remains live until the call returns.
        let entry: extern "C" fn(*const NativeLifecycleEntryContextV1) -> i64 =
            unsafe { std::mem::transmute(entry_address) };
        let return_value = entry(frame.context_pointer());
        let dispatch_response = frame.response();
        NativeEntryInvocationResult {
            protocol: NATIVE_ENTRY_INVOCATION_PROTOCOL,
            adapter_id: self.adapter_id,
            status: "invoked",
            invoked: true,
            execution_identity_hash: self.execution_identity_hash,
            section_id: self.section_id,
            entry_symbol: self.entry_symbol,
            return_value: Some(return_value),
            dispatch_table_identity: Some(frame.table().identity()),
            dispatch_capability_mask: Some(frame.table().capability_mask()),
            dispatch_slot: Some(dispatch_response.slot()),
            dispatch_status_code: Some(dispatch_response.status_code()),
            dispatch_acknowledged: dispatch_response.acknowledged(),
            blockers: Vec::new(),
        }
    }
}

#[cfg(all(unix, any(target_arch = "aarch64", target_arch = "x86_64")))]
mod unix {
    use std::{ffi::c_void, ptr::NonNull};

    pub(super) struct MappedExecutableRegion {
        pointer: NonNull<c_void>,
        size_bytes: usize,
    }

    impl MappedExecutableRegion {
        pub(super) fn new(bytes: &[u8]) -> Result<Self, String> {
            // SAFETY: mmap receives a null hint, valid flags, and a nonzero length.
            let pointer = unsafe {
                libc::mmap(
                    std::ptr::null_mut(),
                    bytes.len(),
                    libc::PROT_READ | libc::PROT_WRITE,
                    libc::MAP_PRIVATE | libc::MAP_ANON,
                    -1,
                    0,
                )
            };
            if pointer == libc::MAP_FAILED {
                return Err(format!(
                    "executable-memory:mmap-failed:{}",
                    std::io::Error::last_os_error()
                ));
            }
            let pointer = NonNull::new(pointer).expect("mmap success returns non-null storage");
            // SAFETY: the new RW mapping is at least bytes.len() and does not overlap input.
            unsafe {
                std::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    pointer.as_ptr().cast::<u8>(),
                    bytes.len(),
                );
                flush_instruction_cache(pointer.as_ptr(), bytes.len());
            }
            // SAFETY: pointer is page-aligned mmap storage owned by this function.
            if unsafe {
                libc::mprotect(
                    pointer.as_ptr(),
                    bytes.len(),
                    libc::PROT_READ | libc::PROT_EXEC,
                )
            } != 0
            {
                let error = std::io::Error::last_os_error();
                // SAFETY: the mapping is still owned and has not escaped.
                unsafe { libc::munmap(pointer.as_ptr(), bytes.len()) };
                return Err(format!("executable-memory:mprotect-failed:{error}"));
            }
            Ok(Self {
                pointer,
                size_bytes: bytes.len(),
            })
        }

        pub(super) unsafe fn entry_address(&self, offset: usize) -> *mut u8 {
            unsafe { self.pointer.as_ptr().cast::<u8>().add(offset) }
        }
    }

    impl Drop for MappedExecutableRegion {
        fn drop(&mut self) {
            // SAFETY: this object uniquely owns the live mmap region.
            unsafe { libc::munmap(self.pointer.as_ptr(), self.size_bytes) };
        }
    }

    #[cfg(target_vendor = "apple")]
    unsafe fn flush_instruction_cache(pointer: *mut c_void, size_bytes: usize) {
        unsafe extern "C" {
            fn sys_icache_invalidate(start: *mut c_void, len: usize);
        }
        // SAFETY: the range is the initialized executable mapping.
        unsafe { sys_icache_invalidate(pointer, size_bytes) };
    }

    #[cfg(all(target_arch = "aarch64", not(target_vendor = "apple")))]
    unsafe fn flush_instruction_cache(pointer: *mut c_void, size_bytes: usize) {
        use std::arch::asm;
        let start = pointer as usize;
        let end = start + size_bytes;
        let ctr: usize;
        unsafe {
            asm!("mrs {ctr}, ctr_el0", ctr = out(reg) ctr, options(nostack, preserves_flags))
        };
        if ctr & (1 << 28) == 0 {
            let line = 4usize << ((ctr >> 16) & 0xf);
            let mut cursor = start & !(line - 1);
            while cursor < end {
                unsafe {
                    asm!("dc cvau, {cursor}", cursor = in(reg) cursor, options(nostack, preserves_flags))
                };
                cursor += line;
            }
        }
        unsafe { asm!("dsb ish", options(nostack, preserves_flags)) };
        if ctr & (1 << 29) == 0 {
            let line = 4usize << (ctr & 0xf);
            let mut cursor = start & !(line - 1);
            while cursor < end {
                unsafe {
                    asm!("ic ivau, {cursor}", cursor = in(reg) cursor, options(nostack, preserves_flags))
                };
                cursor += line;
            }
        }
        unsafe {
            asm!("dsb ish", "isb", options(nostack, preserves_flags));
        }
    }

    #[cfg(target_arch = "x86_64")]
    unsafe fn flush_instruction_cache(_pointer: *mut c_void, _size_bytes: usize) {}
}

fn valid_hash(value: &str) -> bool {
    value.len() == 18
        && value.starts_with("0x")
        && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

#[cfg(test)]
#[path = "executable_memory_tests.rs"]
mod tests;
