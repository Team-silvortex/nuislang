pub(crate) const NSLD_LINK_INPUT_TABLE_SCHEMA: &str = "nuis-nsld-link-input-table-v1";
pub(crate) const NSLD_LINK_INPUT_TABLE_SCHEMA_VERSION: usize = 1;
pub(crate) const NSLD_LINK_INPUT_TABLE_KIND: &str = "lowering-sidecar-link-inputs";
pub(crate) const NSLD_LINK_INPUT_TABLE_PRODUCER: &str = "nsld";
pub(crate) const NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE: &str = "alpha-0.6.0";
pub(crate) const NSLD_LINK_UNIT_TABLE_SCHEMA: &str = "nuis-nsld-link-unit-table-v1";
pub(crate) const NSLD_LINK_UNIT_TABLE_SCHEMA_VERSION: usize = 1;
pub(crate) const NSLD_LINK_UNIT_TABLE_KIND: &str = "deterministic-link-units";
pub(crate) const NSLD_LINK_BUNDLE_SCHEMA: &str = "nuis-nsld-link-bundle-v1";
pub(crate) const NSLD_LINK_BUNDLE_SCHEMA_VERSION: usize = 1;
pub(crate) const NSLD_LINK_BUNDLE_KIND: &str = "hetero-static-link-bundle";
pub(crate) const NSLD_ASSEMBLE_PLAN_SCHEMA: &str = "nuis-nsld-assemble-plan-v1";
pub(crate) const NSLD_ASSEMBLE_PLAN_SCHEMA_VERSION: usize = 1;
pub(crate) const NSLD_ASSEMBLE_PLAN_KIND: &str = "deterministic-section-assembly-plan";
pub(crate) const NSLD_SECTION_MANIFEST_SCHEMA: &str = "nuis-nsld-section-manifest-v1";
pub(crate) const NSLD_SECTION_MANIFEST_SCHEMA_VERSION: usize = 1;
pub(crate) const NSLD_SECTION_MANIFEST_KIND: &str = "deterministic-section-manifest";

pub(crate) fn fnv1a64_hex(bytes: &[u8]) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("0x{hash:016x}")
}
