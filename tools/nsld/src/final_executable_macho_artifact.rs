use crate::content_hash_cache::{file_fingerprint, FileFingerprint};
use crate::{
    final_executable_macho_object::{
        build_macho_host_object_handoff, validate_macho_host_object_handoff,
        MachOArm64PrivateShellProduct,
    },
    reports::NsldExecutableFinalizerInputSummary,
};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, OnceLock,
    },
};

const MACH_O_64_HEADER_SIZE: usize = 32;
const MACH_O_64_LE_MAGIC: [u8; 4] = [0xcf, 0xfa, 0xed, 0xfe];
const MACH_O_FAT_BE_MAGIC: [u8; 4] = [0xca, 0xfe, 0xba, 0xbe];
const MACH_O_FAT_64_BE_MAGIC: [u8; 4] = [0xca, 0xfe, 0xba, 0xbf];
const MACH_O_CPU_TYPE_ARM64: u32 = 0x0100_000c;
const MACH_O_FILE_TYPE_EXECUTE: u32 = 2;
const MACH_O_LOAD_COMMAND_SEGMENT_64: u32 = 0x19;
const MACH_O_LOAD_COMMAND_UNIXTHREAD: u32 = 0x5;
const MACH_O_LOAD_COMMAND_MAIN: u32 = 0x8000_0028;
static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static VALIDATED_IMAGE_CACHE: OnceLock<Mutex<Option<ValidatedImageCacheEntry>>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidatedImageCacheKey {
    artifact_path: PathBuf,
    artifact_fingerprint: FileFingerprint,
    packaging_mode: String,
    machine_arch: String,
    machine_os: String,
    object_format: String,
    calling_abi: String,
    binary_name: String,
    binary_bytes: usize,
}

#[derive(Clone)]
struct ValidatedImageCacheEntry {
    key: ValidatedImageCacheKey,
    result: Result<Arc<[u8]>, String>,
}

pub(crate) fn macho_artifact_image_validation_issues(
    plan: &nuisc::linker::LinkPlan,
) -> Vec<String> {
    validated_macho_artifact_image(plan)
        .err()
        .into_iter()
        .map(|error| format!("compiled-artifact-native-handoff:{error}"))
        .collect()
}

pub(crate) fn macho_artifact_input_summary(
    plan: &nuisc::linker::LinkPlan,
) -> Result<Option<NsldExecutableFinalizerInputSummary>, String> {
    macho_artifact_private_shell_product(plan).map(|product| Some(product.summary))
}

pub(crate) fn macho_artifact_private_shell_product(
    plan: &nuisc::linker::LinkPlan,
) -> Result<MachOArm64PrivateShellProduct, String> {
    let artifact_path = Path::new(&plan.compiled_artifact.path);
    let artifact = nuisc::aot::parse_nuis_compiled_artifact(artifact_path).map_err(|error| {
        format!(
            "failed to parse compiled artifact `{}`: {error}",
            artifact_path.display()
        )
    })?;
    build_macho_host_object_handoff(&artifact, plan)
}

pub(crate) fn materialize_macho_artifact_image(
    plan: &nuisc::linker::LinkPlan,
    output_path: &Path,
) -> Result<(), String> {
    let image = validated_macho_artifact_image(plan)?;
    let expected_name = Path::new(&plan.compiled_artifact.binary_name);
    if output_path.file_name() != expected_name.file_name() {
        return Err(format!(
            "final output file name `{}` does not match compiled artifact binary name `{}`",
            output_path.display(),
            plan.compiled_artifact.binary_name
        ));
    }
    atomic_write_executable(output_path, image.as_ref())
}

fn validated_macho_artifact_image(plan: &nuisc::linker::LinkPlan) -> Result<Arc<[u8]>, String> {
    let key_before = validated_image_cache_key(plan)?;
    let cache = VALIDATED_IMAGE_CACHE.get_or_init(|| Mutex::new(None));
    if let Some(result) = cache
        .lock()
        .map_err(|_| "compiled artifact image cache lock is poisoned".to_owned())?
        .as_ref()
        .filter(|entry| entry.key == key_before)
        .map(|entry| entry.result.clone())
    {
        return result;
    }

    let result = load_and_validate_macho_artifact_image(plan).map(Arc::<[u8]>::from);
    let key_after = validated_image_cache_key(plan)?;
    let result = if key_before == key_after {
        result
    } else {
        Err("compiled artifact changed while its host image was being validated".to_owned())
    };
    *cache
        .lock()
        .map_err(|_| "compiled artifact image cache lock is poisoned".to_owned())? =
        Some(ValidatedImageCacheEntry {
            key: key_after,
            result: result.clone(),
        });
    result
}

fn validated_image_cache_key(
    plan: &nuisc::linker::LinkPlan,
) -> Result<ValidatedImageCacheKey, String> {
    let artifact_path = PathBuf::from(&plan.compiled_artifact.path);
    Ok(ValidatedImageCacheKey {
        artifact_fingerprint: file_fingerprint(&artifact_path)?,
        artifact_path,
        packaging_mode: plan.packaging_mode.clone(),
        machine_arch: plan.cpu_target.machine_arch.clone(),
        machine_os: plan.cpu_target.machine_os.clone(),
        object_format: plan.cpu_target.object_format.clone(),
        calling_abi: plan.cpu_target.calling_abi.clone(),
        binary_name: plan.compiled_artifact.binary_name.clone(),
        binary_bytes: plan.compiled_artifact.binary_bytes,
    })
}

fn load_and_validate_macho_artifact_image(
    plan: &nuisc::linker::LinkPlan,
) -> Result<Vec<u8>, String> {
    if plan.packaging_mode != "native-cpu-llvm" {
        return Err(format!(
            "unsupported packaging mode `{}`; expected `native-cpu-llvm`",
            plan.packaging_mode
        ));
    }
    if canonical_arch(&plan.cpu_target.machine_arch) != Some("aarch64") {
        return Err(format!(
            "unsupported target architecture `{}`; expected `aarch64`",
            plan.cpu_target.machine_arch
        ));
    }
    if canonical_os(&plan.cpu_target.machine_os) != "macos" {
        return Err(format!(
            "unsupported target OS `{}`; expected `macos`",
            plan.cpu_target.machine_os
        ));
    }
    if canonical_object_format(&plan.cpu_target.object_format) != "mach-o" {
        return Err(format!(
            "unsupported object format `{}`; expected `mach-o`",
            plan.cpu_target.object_format
        ));
    }

    let artifact_path = Path::new(&plan.compiled_artifact.path);
    let artifact = nuisc::aot::parse_nuis_compiled_artifact(artifact_path).map_err(|error| {
        format!(
            "failed to parse compiled artifact `{}`: {error}",
            artifact_path.display()
        )
    })?;
    if artifact.packaging_mode != plan.packaging_mode {
        return Err(format!(
            "packaging mode mismatch: plan={}, artifact={}",
            plan.packaging_mode, artifact.packaging_mode
        ));
    }
    if canonical_arch(&artifact.cpu_target_machine_arch) != Some("aarch64")
        || canonical_os(&artifact.cpu_target_machine_os) != "macos"
        || canonical_object_format(&artifact.cpu_target_object_format) != "mach-o"
    {
        return Err(format!(
            "compiled artifact target mismatch: arch={} os={} object={}",
            artifact.cpu_target_machine_arch,
            artifact.cpu_target_machine_os,
            artifact.cpu_target_object_format
        ));
    }
    if artifact.cpu_target_calling_abi != plan.cpu_target.calling_abi {
        return Err(format!(
            "calling ABI mismatch: plan={}, artifact={}",
            plan.cpu_target.calling_abi, artifact.cpu_target_calling_abi
        ));
    }
    validate_macho_host_object_handoff(&artifact, plan)?;
    if artifact.binary_name != plan.compiled_artifact.binary_name {
        return Err(format!(
            "binary name mismatch: plan={}, artifact={}",
            plan.compiled_artifact.binary_name, artifact.binary_name
        ));
    }
    if artifact.binary_bytes != artifact.binary_blob.len() {
        return Err(format!(
            "compiled artifact binary size mismatch: declared={}, actual={}",
            artifact.binary_bytes,
            artifact.binary_blob.len()
        ));
    }
    if plan.compiled_artifact.binary_bytes != 0
        && plan.compiled_artifact.binary_bytes != artifact.binary_blob.len()
    {
        return Err(format!(
            "link-plan binary size mismatch: plan={}, artifact={}",
            plan.compiled_artifact.binary_bytes,
            artifact.binary_blob.len()
        ));
    }
    validate_macho_arm64_executable(&artifact.binary_blob)?;
    Ok(artifact.binary_blob)
}

fn validate_macho_arm64_executable(bytes: &[u8]) -> Result<(), String> {
    match bytes.get(..4) {
        Some(magic) if magic == MACH_O_64_LE_MAGIC => validate_thin_macho_arm64_executable(bytes),
        Some(magic) if magic == MACH_O_FAT_BE_MAGIC => {
            validate_fat_macho_arm64_executable(bytes, false)
        }
        Some(magic) if magic == MACH_O_FAT_64_BE_MAGIC => {
            validate_fat_macho_arm64_executable(bytes, true)
        }
        _ => Err(
            "Mach-O image magic is neither MH_MAGIC_64 nor a supported universal Mach-O".to_owned(),
        ),
    }
}

fn validate_thin_macho_arm64_executable(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() < MACH_O_64_HEADER_SIZE {
        return Err(format!(
            "Mach-O image is truncated: expected at least {MACH_O_64_HEADER_SIZE} bytes, found {}",
            bytes.len()
        ));
    }
    if bytes[..4] != MACH_O_64_LE_MAGIC {
        return Err("Mach-O image magic is not little-endian MH_MAGIC_64".to_owned());
    }
    let cpu_type = read_u32_le(bytes, 4)?;
    if cpu_type != MACH_O_CPU_TYPE_ARM64 {
        return Err(format!(
            "Mach-O CPU type is 0x{cpu_type:08x}; expected ARM64"
        ));
    }
    let file_type = read_u32_le(bytes, 12)?;
    if file_type != MACH_O_FILE_TYPE_EXECUTE {
        return Err(format!(
            "Mach-O file type is {file_type}; expected MH_EXECUTE"
        ));
    }
    let command_count = read_u32_le(bytes, 16)? as usize;
    let command_span = read_u32_le(bytes, 20)? as usize;
    let command_end = MACH_O_64_HEADER_SIZE
        .checked_add(command_span)
        .ok_or_else(|| "Mach-O load-command span overflows address space".to_owned())?;
    if command_end > bytes.len() {
        return Err(format!(
            "Mach-O load-command span ends at {command_end}, beyond image size {}",
            bytes.len()
        ));
    }
    let mut cursor = MACH_O_64_HEADER_SIZE;
    let mut segment_command_present = false;
    let mut entry_command_present = false;
    for index in 0..command_count {
        if cursor.checked_add(8).is_none_or(|end| end > command_end) {
            return Err(format!("Mach-O load command {index} header is truncated"));
        }
        let command = read_u32_le(bytes, cursor)?;
        let command_size = read_u32_le(bytes, cursor + 4)? as usize;
        if command_size < 8 || command_size % 4 != 0 {
            return Err(format!(
                "Mach-O load command {index} has invalid size {command_size}"
            ));
        }
        match command {
            MACH_O_LOAD_COMMAND_SEGMENT_64 => {
                if command_size < 72 {
                    return Err(format!(
                        "Mach-O LC_SEGMENT_64 command {index} is shorter than 72 bytes"
                    ));
                }
                segment_command_present = true;
            }
            MACH_O_LOAD_COMMAND_MAIN => {
                if command_size < 24 {
                    return Err(format!(
                        "Mach-O LC_MAIN command {index} is shorter than 24 bytes"
                    ));
                }
                let entry_offset = read_u64_le(bytes, cursor + 8)?;
                if entry_offset >= bytes.len() as u64 {
                    return Err(format!(
                        "Mach-O LC_MAIN entry offset {entry_offset} exceeds image size {}",
                        bytes.len()
                    ));
                }
                entry_command_present = true;
            }
            MACH_O_LOAD_COMMAND_UNIXTHREAD => {
                if command_size < 16 {
                    return Err(format!(
                        "Mach-O LC_UNIXTHREAD command {index} is shorter than 16 bytes"
                    ));
                }
                entry_command_present = true;
            }
            _ => {}
        }
        cursor = cursor
            .checked_add(command_size)
            .filter(|end| *end <= command_end)
            .ok_or_else(|| format!("Mach-O load command {index} exceeds declared span"))?;
    }
    if cursor != command_end {
        return Err(format!(
            "Mach-O load-command count consumes {} bytes, declared span is {command_span}",
            cursor.saturating_sub(MACH_O_64_HEADER_SIZE)
        ));
    }
    if !segment_command_present {
        return Err("Mach-O executable has no LC_SEGMENT_64 command".to_owned());
    }
    if !entry_command_present {
        return Err("Mach-O executable has neither LC_MAIN nor LC_UNIXTHREAD".to_owned());
    }
    Ok(())
}

fn validate_fat_macho_arm64_executable(
    bytes: &[u8],
    uses_64_bit_offsets: bool,
) -> Result<(), String> {
    if bytes.len() < 8 {
        return Err("universal Mach-O header is truncated".to_owned());
    }
    let architecture_count = read_u32_be(bytes, 4)? as usize;
    let entry_size = if uses_64_bit_offsets {
        32usize
    } else {
        20usize
    };
    let table_size = architecture_count
        .checked_mul(entry_size)
        .and_then(|size| 8usize.checked_add(size))
        .ok_or_else(|| "universal Mach-O architecture table overflows address space".to_owned())?;
    if table_size > bytes.len() {
        return Err(format!(
            "universal Mach-O architecture table ends at {table_size}, beyond image size {}",
            bytes.len()
        ));
    }

    for index in 0..architecture_count {
        let entry_offset = 8 + index * entry_size;
        if read_u32_be(bytes, entry_offset)? != MACH_O_CPU_TYPE_ARM64 {
            continue;
        }
        let (slice_offset, slice_size) = if uses_64_bit_offsets {
            (
                usize::try_from(read_u64_be(bytes, entry_offset + 8)?).map_err(|_| {
                    "universal Mach-O arm64 offset exceeds address space".to_owned()
                })?,
                usize::try_from(read_u64_be(bytes, entry_offset + 16)?)
                    .map_err(|_| "universal Mach-O arm64 size exceeds address space".to_owned())?,
            )
        } else {
            (
                read_u32_be(bytes, entry_offset + 8)? as usize,
                read_u32_be(bytes, entry_offset + 12)? as usize,
            )
        };
        if slice_offset < table_size {
            return Err(format!(
                "universal Mach-O arm64 slice {index} overlaps its architecture table"
            ));
        }
        let alignment_exponent = read_u32_be(
            bytes,
            entry_offset + if uses_64_bit_offsets { 24 } else { 16 },
        )?;
        let alignment = 1usize
            .checked_shl(alignment_exponent)
            .ok_or_else(|| format!("universal Mach-O arm64 slice {index} alignment overflows"))?;
        if slice_offset % alignment != 0 {
            return Err(format!(
                "universal Mach-O arm64 slice {index} offset {slice_offset} violates alignment {alignment}"
            ));
        }
        let slice_end = slice_offset
            .checked_add(slice_size)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| format!("universal Mach-O arm64 slice {index} exceeds image bounds"))?;
        return validate_thin_macho_arm64_executable(&bytes[slice_offset..slice_end]);
    }
    Err("universal Mach-O contains no arm64 executable slice".to_owned())
}

fn atomic_write_executable(output_path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = output_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create final output directory `{}`: {error}",
            parent.display()
        )
    })?;
    let temp_path = temporary_output_path(output_path);
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(|error| {
                format!(
                    "failed to create temporary final output `{}`: {error}",
                    temp_path.display()
                )
            })?;
        file.write_all(bytes).map_err(|error| {
            format!(
                "failed to write temporary final output `{}`: {error}",
                temp_path.display()
            )
        })?;
        file.sync_all().map_err(|error| {
            format!(
                "failed to sync temporary final output `{}`: {error}",
                temp_path.display()
            )
        })?;
        set_executable_permissions(&temp_path)?;
        file.sync_all().map_err(|error| {
            format!(
                "failed to sync executable permissions for `{}`: {error}",
                temp_path.display()
            )
        })?;
        drop(file);
        fs::rename(&temp_path, output_path).map_err(|error| {
            format!(
                "failed to atomically install final output `{}`: {error}",
                output_path.display()
            )
        })?;
        sync_parent_directory(parent)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

fn temporary_output_path(output_path: &Path) -> PathBuf {
    let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let name = output_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("nuis-output");
    output_path.with_file_name(format!(
        ".{name}.nsld-{}-{sequence}.tmp",
        std::process::id()
    ))
}

#[cfg(unix)]
fn set_executable_permissions(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .map_err(|error| format!("failed to inspect `{}`: {error}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .map_err(|error| format!("failed to make `{}` executable: {error}", path.display()))
}

#[cfg(not(unix))]
fn set_executable_permissions(_path: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(unix)]
fn sync_parent_directory(path: &Path) -> Result<(), String> {
    fs::File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            format!(
                "failed to sync output directory `{}`: {error}",
                path.display()
            )
        })
}

#[cfg(not(unix))]
fn sync_parent_directory(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn canonical_arch(machine_arch: &str) -> Option<&'static str> {
    nuis_runtime::canonical_machine_arch(machine_arch)
}

fn canonical_os(machine_os: &str) -> &str {
    match machine_os.trim().to_ascii_lowercase().as_str() {
        "darwin" | "macos" | "apple-darwin" => "macos",
        "linux" | "linux-gnu" | "gnu-linux" => "linux",
        "win32" | "win64" | "windows" => "windows",
        _ => "unknown",
    }
}

fn canonical_object_format(object_format: &str) -> &str {
    match object_format.trim().to_ascii_lowercase().as_str() {
        "mach-o" | "macho" => "mach-o",
        "elf" => "elf",
        "coff" | "pe" | "pe-coff" | "pe/coff" => "pe-coff",
        _ => "unknown",
    }
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let raw: [u8; 4] = bytes
        .get(offset..offset.saturating_add(4))
        .ok_or_else(|| format!("Mach-O u32 at offset {offset} is truncated"))?
        .try_into()
        .map_err(|_| format!("Mach-O u32 at offset {offset} is malformed"))?;
    Ok(u32::from_le_bytes(raw))
}

fn read_u32_be(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let raw: [u8; 4] = bytes
        .get(offset..offset.saturating_add(4))
        .ok_or_else(|| format!("Mach-O big-endian u32 at offset {offset} is truncated"))?
        .try_into()
        .map_err(|_| format!("Mach-O big-endian u32 at offset {offset} is malformed"))?;
    Ok(u32::from_be_bytes(raw))
}

fn read_u64_le(bytes: &[u8], offset: usize) -> Result<u64, String> {
    let raw: [u8; 8] = bytes
        .get(offset..offset.saturating_add(8))
        .ok_or_else(|| format!("Mach-O u64 at offset {offset} is truncated"))?
        .try_into()
        .map_err(|_| format!("Mach-O u64 at offset {offset} is malformed"))?;
    Ok(u64::from_le_bytes(raw))
}

fn read_u64_be(bytes: &[u8], offset: usize) -> Result<u64, String> {
    let raw: [u8; 8] = bytes
        .get(offset..offset.saturating_add(8))
        .ok_or_else(|| format!("Mach-O big-endian u64 at offset {offset} is truncated"))?
        .try_into()
        .map_err(|_| format!("Mach-O big-endian u64 at offset {offset} is malformed"))?;
    Ok(u64::from_be_bytes(raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_well_formed_minimal_arm64_executable_header() {
        let bytes = minimal_arm64_executable();

        assert_eq!(validate_macho_arm64_executable(&bytes), Ok(()));
    }

    #[test]
    fn rejects_object_files_and_truncated_load_commands() {
        let mut object = vec![0u8; 32];
        object[..4].copy_from_slice(&MACH_O_64_LE_MAGIC);
        object[4..8].copy_from_slice(&MACH_O_CPU_TYPE_ARM64.to_le_bytes());
        object[12..16].copy_from_slice(&1u32.to_le_bytes());
        assert!(validate_macho_arm64_executable(&object)
            .unwrap_err()
            .contains("expected MH_EXECUTE"));

        object[12..16].copy_from_slice(&MACH_O_FILE_TYPE_EXECUTE.to_le_bytes());
        object[16..20].copy_from_slice(&1u32.to_le_bytes());
        object[20..24].copy_from_slice(&8u32.to_le_bytes());
        assert!(validate_macho_arm64_executable(&object)
            .unwrap_err()
            .contains("beyond image size"));
    }

    #[test]
    fn rejects_executables_without_an_entry_command() {
        let mut bytes = minimal_arm64_executable();
        bytes[104..108].copy_from_slice(&0u32.to_le_bytes());

        assert!(validate_macho_arm64_executable(&bytes)
            .unwrap_err()
            .contains("neither LC_MAIN nor LC_UNIXTHREAD"));
    }

    #[test]
    fn accepts_a_universal_image_with_an_arm64_executable_slice() {
        let thin = minimal_arm64_executable();

        let mut universal = vec![0u8; 28];
        universal[..4].copy_from_slice(&MACH_O_FAT_BE_MAGIC);
        universal[4..8].copy_from_slice(&1u32.to_be_bytes());
        universal[8..12].copy_from_slice(&MACH_O_CPU_TYPE_ARM64.to_be_bytes());
        universal[16..20].copy_from_slice(&28u32.to_be_bytes());
        universal[20..24].copy_from_slice(&(thin.len() as u32).to_be_bytes());
        universal.extend_from_slice(&thin);

        assert_eq!(validate_macho_arm64_executable(&universal), Ok(()));
    }

    fn minimal_arm64_executable() -> Vec<u8> {
        let mut bytes = vec![0u8; 128];
        bytes[..4].copy_from_slice(&MACH_O_64_LE_MAGIC);
        bytes[4..8].copy_from_slice(&MACH_O_CPU_TYPE_ARM64.to_le_bytes());
        bytes[12..16].copy_from_slice(&MACH_O_FILE_TYPE_EXECUTE.to_le_bytes());
        bytes[16..20].copy_from_slice(&2u32.to_le_bytes());
        bytes[20..24].copy_from_slice(&96u32.to_le_bytes());
        bytes[32..36].copy_from_slice(&MACH_O_LOAD_COMMAND_SEGMENT_64.to_le_bytes());
        bytes[36..40].copy_from_slice(&72u32.to_le_bytes());
        bytes[104..108].copy_from_slice(&MACH_O_LOAD_COMMAND_MAIN.to_le_bytes());
        bytes[108..112].copy_from_slice(&24u32.to_le_bytes());
        bytes[112..120].copy_from_slice(&127u64.to_le_bytes());
        bytes
    }
}
