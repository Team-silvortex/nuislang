use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use crate::registry::NustarPackageManifest;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementationContract {
    pub kind: String,
    pub loader_abi: String,
    pub entry_symbol: String,
    pub entry_signature: String,
    pub host_abi_struct: String,
    pub result_struct: String,
    pub status_convention: String,
    pub artifact_container: String,
    pub implementation_section: String,
    pub required_exports: Vec<String>,
    pub required_metadata: Vec<String>,
    pub link_mode: String,
    pub machine_abi_policy: String,
    pub notes: String,
}

pub fn validate_manifest_for_packaging(manifest: &NustarPackageManifest) -> Result<(), String> {
    if manifest.abi_profiles.is_empty() {
        return Err(format!(
            "nustar package `{}` must declare at least one ABI profile in `abi_profiles`",
            manifest.package_id
        ));
    }
    if manifest.abi_capabilities.is_empty() {
        return Err(format!(
            "nustar package `{}` must declare ABI capability mappings in `abi_capabilities`",
            manifest.package_id
        ));
    }

    let profile_set = manifest
        .abi_profiles
        .iter()
        .map(|value| value.trim().to_owned())
        .collect::<BTreeSet<_>>();
    if profile_set.len() != manifest.abi_profiles.len() {
        return Err(format!(
            "nustar package `{}` has duplicated ABI profile entries in `abi_profiles`",
            manifest.package_id
        ));
    }

    for profile in &manifest.abi_profiles {
        crate::registry::validate_manifest_abi(manifest, profile)?;
        crate::registry::validate_abi_capabilities(manifest, profile, &[], &[])?;
    }

    let mut capabilities_by_abi = BTreeMap::<String, Vec<(String, String)>>::new();
    for raw in &manifest.abi_capabilities {
        let Some((abi, _)) = raw.split_once(':') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_capabilities entry `{}`; expected `abi:kind:value[|kind:value...]`",
                manifest.package_id, raw
            ));
        };
        let abi = abi.trim();
        if !profile_set.contains(abi) {
            return Err(format!(
                "nustar package `{}` has abi_capabilities entry `{}` referencing undeclared ABI profile `{}`",
                manifest.package_id, raw, abi
            ));
        }
        let caps = raw
            .split_once(':')
            .map(|(_, caps)| caps)
            .unwrap_or_default();
        for cap in caps.split('|').map(str::trim).filter(|cap| !cap.is_empty()) {
            if let Some(pattern) = cap.strip_prefix("surface:") {
                capabilities_by_abi
                    .entry(abi.to_owned())
                    .or_default()
                    .push(("surface".to_owned(), pattern.trim().to_owned()));
            } else if let Some(pattern) = cap.strip_prefix("op:") {
                capabilities_by_abi
                    .entry(abi.to_owned())
                    .or_default()
                    .push(("op".to_owned(), pattern.trim().to_owned()));
            }
        }
    }
    validate_domain_capability_policy(manifest, &capabilities_by_abi)?;
    Ok(())
}

fn validate_domain_capability_policy(
    manifest: &NustarPackageManifest,
    capabilities_by_abi: &BTreeMap<String, Vec<(String, String)>>,
) -> Result<(), String> {
    let op_prefix = format!("{}.", manifest.domain_family);
    for profile in &manifest.abi_profiles {
        let profile = profile.trim();
        let caps = capabilities_by_abi
            .get(profile)
            .cloned()
            .unwrap_or_default();
        let mut has_op_capability = false;
        let mut has_surface_capability = false;
        for (kind, pattern) in caps {
            if kind == "op" {
                has_op_capability = true;
                if !pattern.starts_with(&op_prefix) {
                    return Err(format!(
                        "nustar package `{}` ABI `{}` has cross-domain op capability pattern `{}`; expected prefix `{}`",
                        manifest.package_id, profile, pattern, op_prefix
                    ));
                }
            } else if kind == "surface" {
                has_surface_capability = true;
                validate_surface_pattern_for_domain(
                    &manifest.package_id,
                    &manifest.domain_family,
                    profile,
                    &pattern,
                )?;
            }
        }
        if !has_op_capability {
            return Err(format!(
                "nustar package `{}` ABI `{}` must declare at least one `op:` capability",
                manifest.package_id, profile
            ));
        }
        if !manifest.support_surface.is_empty()
            && manifest.domain_family != "cpu"
            && !has_surface_capability
        {
            return Err(format!(
                "nustar package `{}` ABI `{}` must declare at least one `surface:` capability for domain `{}`",
                manifest.package_id, profile, manifest.domain_family
            ));
        }
    }
    Ok(())
}

fn validate_surface_pattern_for_domain(
    package_id: &str,
    domain_family: &str,
    abi: &str,
    pattern: &str,
) -> Result<(), String> {
    let allowed = match domain_family {
        "cpu" => false,
        "data" => pattern.starts_with("data.profile."),
        "kernel" => pattern.starts_with("kernel.profile."),
        "shader" => pattern.starts_with("shader.profile.") || pattern == "shader.inline.wgsl.v1",
        other => pattern.starts_with(&format!("{other}.")),
    };
    if allowed {
        return Ok(());
    }
    Err(format!(
        "nustar package `{}` ABI `{}` has invalid surface capability pattern `{}` for domain `{}`",
        package_id, abi, pattern, domain_family
    ))
}

pub fn encode(binary: &NustarBinary) -> Vec<u8> {
    let manifest = render_manifest(&binary.manifest);
    let abi = binary.abi_tag.as_bytes();
    let machine_arch = binary.machine_arch.as_bytes();
    let machine_os = binary.machine_os.as_bytes();
    let object_format = binary.object_format.as_bytes();
    let calling_abi = binary.calling_abi.as_bytes();
    let format = binary.implementation_format.as_bytes();
    let blob = &binary.implementation_blob;

    let mut out = Vec::new();
    out.extend_from_slice(NUSTAR_MAGIC);
    out.extend_from_slice(&binary.format_version.to_le_bytes());
    out.extend_from_slice(&(manifest.len() as u32).to_le_bytes());
    out.extend_from_slice(&(abi.len() as u32).to_le_bytes());
    out.extend_from_slice(&(machine_arch.len() as u32).to_le_bytes());
    out.extend_from_slice(&(machine_os.len() as u32).to_le_bytes());
    out.extend_from_slice(&(object_format.len() as u32).to_le_bytes());
    out.extend_from_slice(&(calling_abi.len() as u32).to_le_bytes());
    out.extend_from_slice(&(format.len() as u32).to_le_bytes());
    out.extend_from_slice(&(blob.len() as u32).to_le_bytes());
    out.extend_from_slice(&binary.implementation_checksum.to_le_bytes());
    out.extend_from_slice(manifest.as_bytes());
    out.extend_from_slice(abi);
    out.extend_from_slice(machine_arch);
    out.extend_from_slice(machine_os);
    out.extend_from_slice(object_format);
    out.extend_from_slice(calling_abi);
    out.extend_from_slice(format);
    out.extend_from_slice(blob);
    out
}

pub fn decode(bytes: &[u8], source: &Path) -> Result<NustarBinary, String> {
    if bytes.len() < 46 {
        return Err(format!(
            "`{}` is too short to be a nustar binary",
            source.display()
        ));
    }
    if &bytes[..8] != NUSTAR_MAGIC {
        return Err(format!(
            "`{}` does not start with the nustar binary magic",
            source.display()
        ));
    }

    let format_version = u16::from_le_bytes(bytes[8..10].try_into().unwrap());
    if format_version != NUSTAR_FORMAT_VERSION {
        return Err(format!(
            "`{}` has unsupported nustar format version {}; expected {}",
            source.display(),
            format_version,
            NUSTAR_FORMAT_VERSION
        ));
    }

    let manifest_len = u32::from_le_bytes(bytes[10..14].try_into().unwrap()) as usize;
    let abi_len = u32::from_le_bytes(bytes[14..18].try_into().unwrap()) as usize;
    let machine_arch_len = u32::from_le_bytes(bytes[18..22].try_into().unwrap()) as usize;
    let machine_os_len = u32::from_le_bytes(bytes[22..26].try_into().unwrap()) as usize;
    let object_format_len = u32::from_le_bytes(bytes[26..30].try_into().unwrap()) as usize;
    let calling_abi_len = u32::from_le_bytes(bytes[30..34].try_into().unwrap()) as usize;
    let format_len = u32::from_le_bytes(bytes[34..38].try_into().unwrap()) as usize;
    let blob_len = u32::from_le_bytes(bytes[38..42].try_into().unwrap()) as usize;
    let implementation_checksum = u32::from_le_bytes(bytes[42..46].try_into().unwrap());

    let expected = 46
        + manifest_len
        + abi_len
        + machine_arch_len
        + machine_os_len
        + object_format_len
        + calling_abi_len
        + format_len
        + blob_len;
    if bytes.len() != expected {
        return Err(format!(
            "`{}` has invalid nustar binary length: expected {}, got {}",
            source.display(),
            expected,
            bytes.len()
        ));
    }

    let manifest_start = 46;
    let abi_start = manifest_start + manifest_len;
    let machine_arch_start = abi_start + abi_len;
    let machine_os_start = machine_arch_start + machine_arch_len;
    let object_format_start = machine_os_start + machine_os_len;
    let calling_abi_start = object_format_start + object_format_len;
    let impl_format_start = calling_abi_start + calling_abi_len;
    let blob_start = impl_format_start + format_len;

    let manifest_source =
        std::str::from_utf8(&bytes[manifest_start..abi_start]).map_err(|error| {
            format!(
                "`{}` has invalid utf-8 in manifest segment: {error}",
                source.display()
            )
        })?;
    let abi_tag = std::str::from_utf8(&bytes[abi_start..machine_arch_start])
        .map_err(|error| {
            format!(
                "`{}` has invalid utf-8 in ABI tag segment: {error}",
                source.display()
            )
        })?
        .to_owned();
    let machine_arch = std::str::from_utf8(&bytes[machine_arch_start..machine_os_start])
        .map_err(|error| {
            format!(
                "`{}` has invalid utf-8 in machine arch segment: {error}",
                source.display()
            )
        })?
        .to_owned();
    let machine_os = std::str::from_utf8(&bytes[machine_os_start..object_format_start])
        .map_err(|error| {
            format!(
                "`{}` has invalid utf-8 in machine os segment: {error}",
                source.display()
            )
        })?
        .to_owned();
    let object_format = std::str::from_utf8(&bytes[object_format_start..calling_abi_start])
        .map_err(|error| {
            format!(
                "`{}` has invalid utf-8 in object format segment: {error}",
                source.display()
            )
        })?
        .to_owned();
    let calling_abi = std::str::from_utf8(&bytes[calling_abi_start..impl_format_start])
        .map_err(|error| {
            format!(
                "`{}` has invalid utf-8 in calling ABI segment: {error}",
                source.display()
            )
        })?
        .to_owned();
    let implementation_format = std::str::from_utf8(&bytes[impl_format_start..blob_start])
        .map_err(|error| {
            format!(
                "`{}` has invalid utf-8 in implementation format segment: {error}",
                source.display()
            )
        })?
        .to_owned();
    let implementation_blob = bytes[blob_start..].to_vec();
    let actual_checksum = checksum(&implementation_blob);
    if actual_checksum != implementation_checksum {
        return Err(format!(
            "`{}` has mismatched implementation checksum: header={}, actual={}",
            source.display(),
            implementation_checksum,
            actual_checksum
        ));
    }

    let manifest = parse_manifest_text(manifest_source, source)?;
    validate_manifest_for_packaging(&manifest).map_err(|error| {
        format!(
            "`{}` failed manifest ABI packaging validation: {error}",
            source.display()
        )
    })?;

    Ok(NustarBinary {
        manifest,
        format_version,
        abi_tag,
        machine_arch,
        machine_os,
        object_format,
        calling_abi,
        implementation_format,
        implementation_blob,
        implementation_checksum,
    })
}

pub fn write_to_path(path: &Path, binary: &NustarBinary) -> Result<(), String> {
    fs::write(path, encode(binary))
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))
}

pub fn read_from_path(path: &Path) -> Result<NustarBinary, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    decode(&bytes, path)
}

pub fn default_binary(
    manifest: NustarPackageManifest,
    implementation_blob: Vec<u8>,
) -> NustarBinary {
    let implementation_checksum = checksum(&implementation_blob);
    NustarBinary {
        manifest,
        format_version: NUSTAR_FORMAT_VERSION,
        abi_tag: "nustar-abi-v1".to_owned(),
        machine_arch: current_machine_arch().to_owned(),
        machine_os: current_machine_os().to_owned(),
        object_format: current_object_format().to_owned(),
        calling_abi: current_calling_abi().to_owned(),
        implementation_format: "nustar-impl-stub-v1".to_owned(),
        implementation_blob,
        implementation_checksum,
    }
}

pub fn machine_abi_matches_host(binary: &NustarBinary) -> bool {
    binary.machine_arch == current_machine_arch()
        && binary.machine_os == current_machine_os()
        && binary.object_format == current_object_format()
        && binary.calling_abi == current_calling_abi()
}

pub fn implementation_contracts(binary: &NustarBinary) -> Vec<ImplementationContract> {
    binary
        .manifest
        .implementation_kinds
        .iter()
        .map(|kind| implementation_contract(binary, kind))
        .collect()
}

fn canonical_entry_signature(binary: &NustarBinary, kind: &str) -> String {
    match kind {
        "native-dylib" => format!(
            "{CANONICAL_ENTRY_SIGNATURE} // machine={} / {} / {} / {}",
            binary.machine_arch, binary.machine_os, binary.object_format, binary.calling_abi
        ),
        "llvm-bc" => format!(
            "{CANONICAL_ENTRY_SIGNATURE} // lowered under {} to {} / {} / {} / {}",
            binary.manifest.machine_abi_policy,
            binary.machine_arch,
            binary.machine_os,
            binary.object_format,
            binary.calling_abi
        ),
        _ => CANONICAL_ENTRY_SIGNATURE.to_owned(),
    }
}

fn implementation_contract(binary: &NustarBinary, kind: &str) -> ImplementationContract {
    let canonical_export = binary.manifest.loader_entry.clone();
    match kind {
        "native-dylib" => ImplementationContract {
            kind: kind.to_owned(),
            loader_abi: binary.manifest.loader_abi.clone(),
            entry_symbol: canonical_export.clone(),
            entry_signature: canonical_entry_signature(binary, kind),
            host_abi_struct: CANONICAL_HOST_ABI_STRUCT.to_owned(),
            result_struct: CANONICAL_RESULT_STRUCT.to_owned(),
            status_convention: CANONICAL_LOADER_STATUS_CONVENTION.to_owned(),
            artifact_container: format!("native shared library ({})", binary.object_format),
            implementation_section: ".nustar.impl.native-dylib".to_owned(),
            required_exports: vec![
                canonical_export,
                "nustar.manifest.v1".to_owned(),
                "nustar.loader_abi.v1".to_owned(),
            ],
            required_metadata: vec![
                format!("machine_arch={}", binary.machine_arch),
                format!("machine_os={}", binary.machine_os),
                format!("object_format={}", binary.object_format),
                format!("calling_abi={}", binary.calling_abi),
            ],
            link_mode: "host-dynamic-load".to_owned(),
            machine_abi_policy: binary.manifest.machine_abi_policy.clone(),
            notes: "expects a host-loadable shared library exporting the canonical loader entry with the canonical host/result structs".to_owned(),
        },
        "llvm-bc" => ImplementationContract {
            kind: kind.to_owned(),
            loader_abi: binary.manifest.loader_abi.clone(),
            entry_symbol: canonical_export.clone(),
            entry_signature: canonical_entry_signature(binary, kind),
            host_abi_struct: CANONICAL_HOST_ABI_STRUCT.to_owned(),
            result_struct: CANONICAL_RESULT_STRUCT.to_owned(),
            status_convention: CANONICAL_LOADER_STATUS_CONVENTION.to_owned(),
            artifact_container: "llvm-bitcode-module".to_owned(),
            implementation_section: ".nustar.impl.llvm-bc".to_owned(),
            required_exports: vec![
                canonical_export,
                "nustar.manifest.v1".to_owned(),
                "nustar.loader_abi.v1".to_owned(),
            ],
            required_metadata: vec![
                "llvm_bitcode_version=opaque-pointer-compatible".to_owned(),
                format!("lowering_target_machine={}", binary.machine_arch),
                format!("lowering_object_format={}", binary.object_format),
                format!("lowering_calling_abi={}", binary.calling_abi),
            ],
            link_mode: "nuisc-link-or-lower".to_owned(),
            machine_abi_policy: binary.manifest.machine_abi_policy.clone(),
            notes: "expects LLVM bitcode carrying the canonical loader entry symbol and the same bootstrap signature for later lowering/link integration".to_owned(),
        },
        "native-stub" => ImplementationContract {
            kind: kind.to_owned(),
            loader_abi: binary.manifest.loader_abi.clone(),
            entry_symbol: canonical_export,
            entry_signature: canonical_entry_signature(binary, kind),
            host_abi_struct: CANONICAL_HOST_ABI_STRUCT.to_owned(),
            result_struct: CANONICAL_RESULT_STRUCT.to_owned(),
            status_convention: CANONICAL_LOADER_STATUS_CONVENTION.to_owned(),
            artifact_container: "opaque stub payload".to_owned(),
            implementation_section: ".nustar.impl.stub".to_owned(),
            required_exports: vec!["nustar.manifest.v1".to_owned()],
            required_metadata: vec!["prototype_only=true".to_owned()],
            link_mode: "non-loadable".to_owned(),
            machine_abi_policy: binary.manifest.machine_abi_policy.clone(),
            notes: "prototype-only placeholder implementation; may be inspected and packaged but does not provide executable domain code".to_owned(),
        },
        other => ImplementationContract {
            kind: kind.to_owned(),
            loader_abi: binary.manifest.loader_abi.clone(),
            entry_symbol: binary.manifest.loader_entry.clone(),
            entry_signature: canonical_entry_signature(binary, other),
            host_abi_struct: CANONICAL_HOST_ABI_STRUCT.to_owned(),
            result_struct: CANONICAL_RESULT_STRUCT.to_owned(),
            status_convention: CANONICAL_LOADER_STATUS_CONVENTION.to_owned(),
            artifact_container: "custom-container".to_owned(),
            implementation_section: format!(".nustar.impl.{other}"),
            required_exports: vec![
                binary.manifest.loader_entry.clone(),
                "nustar.manifest.v1".to_owned(),
                "nustar.loader_abi.v1".to_owned(),
            ],
            required_metadata: vec!["custom_kind_requires_explicit_loader_adapter=true".to_owned()],
            link_mode: "custom".to_owned(),
            machine_abi_policy: binary.manifest.machine_abi_policy.clone(),
            notes: format!(
                "custom implementation kind `{other}` must still satisfy the canonical loader ABI and entry contract"
            ),
        },
    }
}

fn render_manifest(manifest: &NustarPackageManifest) -> String {
    format!(
        "manifest_schema = \"{}\"\npackage_id = \"{}\"\ndomain_family = \"{}\"\nfrontend = \"{}\"\nentry_crate = \"{}\"\nast_entry = \"{}\"\nnir_entry = \"{}\"\nyir_lowering_entry = \"{}\"\npart_verify_entry = \"{}\"\nast_surface = {}\nnir_surface = {}\nyir_lowering = {}\npart_verify = {}\nbinary_extension = \"{}\"\npackage_layout = \"{}\"\nmachine_abi_policy = \"{}\"\nabi_profiles = {}\nabi_capabilities = {}\nimplementation_kinds = {}\nloader_entry = \"{}\"\nloader_abi = \"{}\"\nhost_ffi_surface = {}\nhost_ffi_abis = {}\nhost_ffi_bridge = \"{}\"\nsupport_surface = {}\nsupport_profile_slots = {}\nprofiles = {}\nresource_families = {}\nunit_types = {}\nlowering_targets = {}\nops = {}\n",
        manifest.manifest_schema,
        manifest.package_id,
        manifest.domain_family,
        manifest.frontend,
        manifest.entry_crate,
        manifest.ast_entry,
        manifest.nir_entry,
        manifest.yir_lowering_entry,
        manifest.part_verify_entry,
        render_array(&manifest.ast_surface),
        render_array(&manifest.nir_surface),
        render_array(&manifest.yir_lowering),
        render_array(&manifest.part_verify),
        manifest.binary_extension,
        manifest.package_layout,
        manifest.machine_abi_policy,
        render_array(&manifest.abi_profiles),
        render_array(&manifest.abi_capabilities),
        render_array(&manifest.implementation_kinds),
        manifest.loader_entry,
        manifest.loader_abi,
        render_array(&manifest.host_ffi_surface),
        render_array(&manifest.host_ffi_abis),
        manifest.host_ffi_bridge,
        render_array(&manifest.support_surface),
        render_array(&manifest.support_profile_slots),
        render_array(&manifest.profiles),
        render_array(&manifest.resource_families),
        render_array(&manifest.unit_types),
        render_array(&manifest.lowering_targets),
        render_array(&manifest.ops),
    )
}

fn render_array(values: &[String]) -> String {
    let quoted = values
        .iter()
        .map(|value| format!("\"{}\"", value))
        .collect::<Vec<_>>();
    format!("[{}]", quoted.join(", "))
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0u32, |acc, byte| {
        acc.wrapping_mul(16777619).wrapping_add(*byte as u32)
    })
}

fn current_machine_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => other,
    }
}

fn current_machine_os() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    }
}

fn current_object_format() -> &'static str {
    match std::env::consts::OS {
        "macos" => "mach-o",
        "linux" => "elf",
        "windows" => "coff",
        _ => "unknown",
    }
}

fn current_calling_abi() -> &'static str {
    match (current_machine_arch(), current_machine_os()) {
        ("arm64", "darwin") => "aapcs64-darwin",
        ("arm64", _) => "aapcs64",
        ("x86_64", "windows") => "win64",
        ("x86_64", _) => "sysv64",
        _ => "unknown",
    }
}

fn parse_manifest_text(source: &str, path: &Path) -> Result<NustarPackageManifest, String> {
    Ok(NustarPackageManifest {
        manifest_schema: parse_required_string(source, "manifest_schema", path)?,
        package_id: parse_required_string(source, "package_id", path)?,
        domain_family: parse_required_string(source, "domain_family", path)?,
        frontend: parse_required_string(source, "frontend", path)?,
        entry_crate: parse_required_string(source, "entry_crate", path)?,
        ast_entry: parse_required_string(source, "ast_entry", path)?,
        nir_entry: parse_required_string(source, "nir_entry", path)?,
        yir_lowering_entry: parse_required_string(source, "yir_lowering_entry", path)?,
        part_verify_entry: parse_required_string(source, "part_verify_entry", path)?,
        ast_surface: parse_string_array(source, "ast_surface", path)?,
        nir_surface: parse_string_array(source, "nir_surface", path)?,
        yir_lowering: parse_string_array(source, "yir_lowering", path)?,
        part_verify: parse_string_array(source, "part_verify", path)?,
        binary_extension: parse_required_string(source, "binary_extension", path)?,
        package_layout: parse_required_string(source, "package_layout", path)?,
        machine_abi_policy: parse_required_string(source, "machine_abi_policy", path)?,
        abi_profiles: parse_optional_string_array(source, "abi_profiles").unwrap_or_default(),
        abi_capabilities: parse_optional_string_array(source, "abi_capabilities")
            .unwrap_or_default(),
        implementation_kinds: parse_string_array(source, "implementation_kinds", path)?,
        loader_entry: parse_required_string(source, "loader_entry", path)?,
        loader_abi: parse_required_string(source, "loader_abi", path)?,
        host_ffi_surface: parse_string_array(source, "host_ffi_surface", path)?,
        host_ffi_abis: parse_string_array(source, "host_ffi_abis", path)?,
        host_ffi_bridge: parse_required_string(source, "host_ffi_bridge", path)?,
        support_surface: parse_optional_string_array(source, "support_surface").unwrap_or_default(),
        support_profile_slots: parse_optional_string_array(source, "support_profile_slots")
            .unwrap_or_default(),
        profiles: parse_string_array(source, "profiles", path)?,
        resource_families: parse_string_array(source, "resource_families", path)?,
        unit_types: parse_string_array(source, "unit_types", path)?,
        lowering_targets: parse_string_array(source, "lowering_targets", path)?,
        ops: parse_string_array(source, "ops", path)?,
    })
}

fn parse_required_string(source: &str, key: &str, path: &Path) -> Result<String, String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let trimmed = rest.trim();
            if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
                return Ok(trimmed[1..trimmed.len() - 1].to_owned());
            }
            return Err(format!(
                "`{}` has invalid string value for `{key}`",
                path.display()
            ));
        }
    }
    Err(format!(
        "`{}` is missing required key `{key}`",
        path.display()
    ))
}

fn parse_string_array(source: &str, key: &str, path: &Path) -> Result<Vec<String>, String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let trimmed = rest.trim();
            if !(trimmed.starts_with('[') && trimmed.ends_with(']')) {
                return Err(format!(
                    "`{}` has invalid array value for `{key}`",
                    path.display()
                ));
            }
            let inner = &trimmed[1..trimmed.len() - 1];
            if inner.trim().is_empty() {
                return Ok(Vec::new());
            }
            let mut items = Vec::new();
            for part in inner.split(',') {
                let value = part.trim();
                if !(value.starts_with('"') && value.ends_with('"') && value.len() >= 2) {
                    return Err(format!(
                        "`{}` has invalid array item for `{key}`",
                        path.display()
                    ));
                }
                items.push(value[1..value.len() - 1].to_owned());
            }
            return Ok(items);
        }
    }
    Err(format!(
        "`{}` is missing required key `{key}`",
        path.display()
    ))
}

fn parse_optional_string_array(source: &str, key: &str) -> Option<Vec<String>> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let trimmed = rest.trim();
            if !(trimmed.starts_with('[') && trimmed.ends_with(']')) {
                return None;
            }
            let inner = &trimmed[1..trimmed.len() - 1];
            if inner.trim().is_empty() {
                return Some(Vec::new());
            }
            let mut values = Vec::new();
            for part in inner.split(',') {
                let item = part.trim();
                if !(item.starts_with('"') && item.ends_with('"') && item.len() >= 2) {
                    return None;
                }
                values.push(item[1..item.len() - 1].to_owned());
            }
            return Some(values);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct PolicyCase {
        name: &'static str,
        domain: &'static str,
        support_surface: Vec<&'static str>,
        abi_capabilities: Vec<&'static str>,
        expect_ok: bool,
        expect_error_contains: &'static str,
    }

    fn make_manifest(domain: &str) -> NustarPackageManifest {
        NustarPackageManifest {
            manifest_schema: "nustar-manifest-v1".to_owned(),
            package_id: format!("test.{domain}"),
            domain_family: domain.to_owned(),
            frontend: format!("nustar-{domain}"),
            entry_crate: format!("crates/yir-domain-{domain}"),
            ast_entry: format!("{domain}.ast.bootstrap.v1"),
            nir_entry: format!("{domain}.nir.bootstrap.v1"),
            yir_lowering_entry: format!("{domain}.yir.lowering.v1"),
            part_verify_entry: format!("{domain}.verify.partial.v1"),
            ast_surface: vec![format!("{domain}.mod-ast.v1")],
            nir_surface: vec![format!("nir.{domain}.surface.v1")],
            yir_lowering: vec![format!("yir.{domain}.lowering.v1")],
            part_verify: vec![format!("verify.{domain}.contract.v1")],
            binary_extension: "nustar".to_owned(),
            package_layout: "single-envelope".to_owned(),
            machine_abi_policy: "exact-match".to_owned(),
            abi_profiles: vec![format!("{domain}.abi.v1")],
            abi_capabilities: Vec::new(),
            implementation_kinds: vec!["native-stub".to_owned()],
            loader_entry: "nustar.bootstrap.v1".to_owned(),
            loader_abi: "nustar-loader-v1".to_owned(),
            host_ffi_surface: Vec::new(),
            host_ffi_abis: Vec::new(),
            host_ffi_bridge: "none".to_owned(),
            support_surface: Vec::new(),
            support_profile_slots: Vec::new(),
            profiles: vec!["aot".to_owned()],
            resource_families: vec![domain.to_owned()],
            unit_types: vec!["Main".to_owned()],
            lowering_targets: vec!["native".to_owned()],
            ops: vec![format!("{domain}.const")],
        }
    }

    #[test]
    fn capability_policy_table() {
        let cases = vec![
            PolicyCase {
                name: "reject cross-domain op for shader",
                domain: "shader",
                support_surface: vec!["shader.profile.packet.v1"],
                abi_capabilities: vec!["shader.abi.v1:surface:shader.profile.*|op:cpu.*"],
                expect_ok: false,
                expect_error_contains: "cross-domain op capability pattern",
            },
            PolicyCase {
                name: "reject invalid data surface prefix",
                domain: "data",
                support_surface: vec!["data.profile.bind-core.v1"],
                abi_capabilities: vec!["data.abi.v1:surface:shader.profile.*|op:data.*"],
                expect_ok: false,
                expect_error_contains: "invalid surface capability pattern",
            },
            PolicyCase {
                name: "reject missing surface capability for kernel",
                domain: "kernel",
                support_surface: vec!["kernel.profile.bind-core.v1"],
                abi_capabilities: vec!["kernel.abi.v1:op:kernel.*"],
                expect_ok: false,
                expect_error_contains: "must declare at least one `surface:` capability",
            },
            PolicyCase {
                name: "reject surface capability in cpu domain",
                domain: "cpu",
                support_surface: vec![],
                abi_capabilities: vec!["cpu.abi.v1:surface:cpu.profile.*|op:cpu.*"],
                expect_ok: false,
                expect_error_contains: "invalid surface capability pattern",
            },
            PolicyCase {
                name: "accept valid shader capability policy",
                domain: "shader",
                support_surface: vec!["shader.profile.packet.v1", "shader.inline.wgsl.v1"],
                abi_capabilities: vec![
                    "shader.abi.v1:surface:shader.profile.*|surface:shader.inline.wgsl.v1|op:shader.*",
                ],
                expect_ok: true,
                expect_error_contains: "",
            },
            PolicyCase {
                name: "accept valid cpu capability policy",
                domain: "cpu",
                support_surface: vec![],
                abi_capabilities: vec!["cpu.abi.v1:op:cpu.*"],
                expect_ok: true,
                expect_error_contains: "",
            },
        ];

        for case in cases {
            let mut manifest = make_manifest(case.domain);
            manifest.support_surface = case
                .support_surface
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            manifest.abi_capabilities = case
                .abi_capabilities
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>();

            let result = validate_manifest_for_packaging(&manifest);
            if case.expect_ok {
                assert!(
                    result.is_ok(),
                    "{}: unexpected error: {result:?}",
                    case.name
                );
            } else {
                let error = result.unwrap_err();
                assert!(
                    error.contains(case.expect_error_contains),
                    "{}: unexpected error: {}",
                    case.name,
                    error
                );
            }
        }
    }

    #[test]
    fn reject_missing_capability_mapping_for_one_profile() {
        let mut manifest = make_manifest("data");
        manifest.abi_profiles = vec!["data.abi.v1".to_owned(), "data.abi.alt.v1".to_owned()];
        manifest.support_surface = vec!["data.profile.bind-core.v1".to_owned()];
        manifest.abi_capabilities = vec!["data.abi.v1:surface:data.profile.*|op:data.*".to_owned()];

        let error = validate_manifest_for_packaging(&manifest).unwrap_err();
        assert!(
            error.contains("has no abi_capabilities mapping"),
            "unexpected error: {error}"
        );
    }
}
