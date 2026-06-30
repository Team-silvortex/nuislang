use std::{fs, path::Path};

use crate::registry::NustarPackageManifest;

use super::manifest::{parse_manifest_text, render_manifest, validate_manifest_for_packaging};
use super::{NustarBinary, NUSTAR_FORMAT_VERSION, NUSTAR_MAGIC};

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
