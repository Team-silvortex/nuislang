use crate::registry::NustarPackageManifest;

#[path = "nustar_binary_codec.rs"]
mod codec;
#[path = "nustar_binary_contracts.rs"]
mod contracts;
#[path = "nustar_binary_manifest.rs"]
mod manifest;
#[cfg(test)]
#[path = "nustar_binary_tests.rs"]
mod tests;

pub use codec::{
    decode, default_binary, encode, machine_abi_matches_host, read_from_path, write_to_path,
};
pub use contracts::{implementation_contracts, ImplementationContract};
pub use manifest::validate_manifest_for_packaging;

const NUSTAR_MAGIC: &[u8; 8] = b"NUSTAR01";
const NUSTAR_FORMAT_VERSION: u16 = 1;

pub const CANONICAL_LOADER_ABI: &str = "nustar-loader-v1";
pub const CANONICAL_ENTRY_SYMBOL: &str = "nustar.bootstrap.v1";
pub const CANONICAL_HOST_ABI_STRUCT: &str = "NustarHostAbiV1";
pub const CANONICAL_RESULT_STRUCT: &str = "NustarBootstrapResultV1";
pub const CANONICAL_ENTRY_SIGNATURE: &str =
    "extern \"C\" fn(*const NustarHostAbiV1, *const u8, usize, *mut NustarBootstrapResultV1) -> i32";
pub const CANONICAL_LOADER_STATUS_CONVENTION: &str =
    "returns 0 on success; non-zero loader status on bootstrap failure";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarBinary {
    pub manifest: NustarPackageManifest,
    pub format_version: u16,
    pub abi_tag: String,
    pub machine_arch: String,
    pub machine_os: String,
    pub object_format: String,
    pub calling_abi: String,
    pub implementation_format: String,
    pub implementation_blob: Vec<u8>,
    pub implementation_checksum: u32,
}
