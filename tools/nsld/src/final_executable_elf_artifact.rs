use crate::{
    final_executable_atomic_output::atomic_write_executable,
    final_executable_elf_loader_probe::{
        probe_elf_amd64_private_shell_image, ElfAmd64LoaderProbeInput,
    },
    final_executable_elf_object::{build_elf_amd64_host_object_linkage, ElfAmd64HostObjectLinkage},
};
use std::path::Path;

const ELF64_HEADER_SIZE: usize = 64;
const ELF64_PROGRAM_HEADER_SIZE: usize = 56;
const ELF_CLASS_64: u8 = 2;
const ELF_DATA_LITTLE_ENDIAN: u8 = 1;
const ELF_VERSION_CURRENT: u8 = 1;
const ELF_TYPE_EXECUTABLE: u16 = 2;
const ELF_TYPE_SHARED: u16 = 3;
const ELF_MACHINE_X86_64: u16 = 62;
const ELF_PROGRAM_TYPE_LOAD: u32 = 1;
const ELF_PROGRAM_FLAG_EXECUTE: u32 = 1;
const ELF_AMD64_CPU_ABI: &str = "cpu.x86_64.sysv64";
const ELF_AMD64_CALLING_ABI: &str = "sysv64";

pub(crate) fn elf_amd64_artifact_image_validation_issues(
    plan: &nuisc::linker::LinkPlan,
) -> Vec<String> {
    load_and_validate_elf_amd64_artifact_image(plan)
        .err()
        .into_iter()
        .map(|error| format!("compiled-artifact-native-handoff:{error}"))
        .collect()
}

pub(crate) fn materialize_elf_amd64_artifact_image(
    plan: &nuisc::linker::LinkPlan,
    output_path: &Path,
) -> Result<(), String> {
    let image = load_and_validate_elf_amd64_artifact_image(plan)?;
    let expected_name = Path::new(&plan.compiled_artifact.binary_name);
    if output_path.file_name() != expected_name.file_name() {
        return Err(format!(
            "final output file name `{}` does not match compiled artifact binary name `{}`",
            output_path.display(),
            plan.compiled_artifact.binary_name
        ));
    }
    atomic_write_executable(output_path, &image)
}

fn load_and_validate_elf_amd64_artifact_image(
    plan: &nuisc::linker::LinkPlan,
) -> Result<Vec<u8>, String> {
    let (artifact, product) = load_elf_amd64_artifact_private_product(plan)?;
    let probe = probe_elf_amd64_private_shell_image(
        ElfAmd64LoaderProbeInput {
            bytes: &product.private_shell_image,
            validation: &product.shell_image_validation,
            unresolved_external_symbol_count: product.summary.unresolved_external_symbols.len(),
        },
        Path::new("."),
        false,
    )?;
    if probe.attempted || probe.materialized || probe.publication_eligible {
        return Err("ELF default finalizer path crossed the loader-probe boundary".to_owned());
    }
    validate_elf64_amd64_executable(&artifact.binary_blob)?;
    Ok(artifact.binary_blob)
}

fn load_elf_amd64_artifact_private_product(
    plan: &nuisc::linker::LinkPlan,
) -> Result<(nuisc::aot::NuisCompiledArtifact, ElfAmd64HostObjectLinkage), String> {
    validate_plan_target(plan)?;
    let artifact_path = Path::new(&plan.compiled_artifact.path);
    let artifact = nuisc::aot::parse_nuis_compiled_artifact(artifact_path).map_err(|error| {
        format!(
            "failed to parse compiled artifact `{}`: {error}",
            artifact_path.display()
        )
    })?;
    nuisc::aot::validate_nuis_compiled_artifact_layout(artifact_path, &artifact)?;
    if artifact.schema != "nuis-compiled-artifact-v1" {
        return Err(format!(
            "compiled artifact schema `{}` is not supported",
            artifact.schema
        ));
    }
    if artifact.packaging_mode != plan.packaging_mode {
        return Err(format!(
            "packaging mode mismatch: plan={}, artifact={}",
            plan.packaging_mode, artifact.packaging_mode
        ));
    }
    if canonical_arch(&artifact.cpu_target_machine_arch) != Some("x86_64")
        || canonical_os(&artifact.cpu_target_machine_os) != "linux"
        || canonical_object_format(&artifact.cpu_target_object_format) != "elf"
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
    if artifact.cpu_target_abi != plan.cpu_target.abi {
        return Err(format!(
            "CPU ABI mismatch: plan={}, artifact={}",
            plan.cpu_target.abi, artifact.cpu_target_abi
        ));
    }
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
    let product = build_elf_amd64_host_object_linkage(&artifact, plan)?;
    Ok((artifact, product))
}

fn validate_plan_target(plan: &nuisc::linker::LinkPlan) -> Result<(), String> {
    if plan.packaging_mode != "native-cpu-llvm" {
        return Err(format!(
            "unsupported packaging mode `{}`; expected `native-cpu-llvm`",
            plan.packaging_mode
        ));
    }
    if canonical_arch(&plan.cpu_target.machine_arch) != Some("x86_64") {
        return Err(format!(
            "unsupported target architecture `{}`; expected `x86_64`",
            plan.cpu_target.machine_arch
        ));
    }
    if canonical_os(&plan.cpu_target.machine_os) != "linux" {
        return Err(format!(
            "unsupported target OS `{}`; expected `linux`",
            plan.cpu_target.machine_os
        ));
    }
    if canonical_object_format(&plan.cpu_target.object_format) != "elf" {
        return Err(format!(
            "unsupported object format `{}`; expected `elf`",
            plan.cpu_target.object_format
        ));
    }
    if plan.cpu_target.abi != ELF_AMD64_CPU_ABI {
        return Err(format!(
            "unsupported CPU ABI `{}`; expected `{ELF_AMD64_CPU_ABI}`",
            plan.cpu_target.abi
        ));
    }
    if plan.cpu_target.calling_abi != ELF_AMD64_CALLING_ABI {
        return Err(format!(
            "unsupported calling ABI `{}`; expected `{ELF_AMD64_CALLING_ABI}`",
            plan.cpu_target.calling_abi
        ));
    }
    Ok(())
}

fn validate_elf64_amd64_executable(bytes: &[u8]) -> Result<(), String> {
    validate_elf64_header(bytes, "executable")?;
    let file_type = read_u16(bytes, 16, "ELF executable type")?;
    if !matches!(file_type, ELF_TYPE_EXECUTABLE | ELF_TYPE_SHARED) {
        return Err(format!(
            "ELF image type is {file_type}; expected ET_EXEC or ET_DYN"
        ));
    }
    let entry = read_u64(bytes, 24, "ELF executable entry")?;
    if entry == 0 {
        return Err("ELF executable entry is zero".to_owned());
    }
    let program_offset = checked_usize(
        read_u64(bytes, 32, "ELF program-header offset")?,
        "ELF program-header offset",
    )?;
    let program_entry_size = read_u16(bytes, 54, "ELF program-header entry size")? as usize;
    let program_count = read_u16(bytes, 56, "ELF program-header count")? as usize;
    if program_entry_size != ELF64_PROGRAM_HEADER_SIZE || program_count == 0 {
        return Err(format!(
            "ELF executable program-header shape is invalid: entry_size={program_entry_size} count={program_count}"
        ));
    }
    let table_end = checked_table_end(
        program_offset,
        program_entry_size,
        program_count,
        bytes.len(),
        "ELF program-header table",
    )?;
    if program_offset < ELF64_HEADER_SIZE || table_end > bytes.len() {
        return Err("ELF program-header table overlaps or exceeds the file header".to_owned());
    }
    let mut load_count = 0usize;
    let mut executable_entry = false;
    for index in 0..program_count {
        let offset = program_offset + index * program_entry_size;
        if read_u32(bytes, offset, "ELF program type")? != ELF_PROGRAM_TYPE_LOAD {
            continue;
        }
        load_count += 1;
        let flags = read_u32(bytes, offset + 4, "ELF program flags")?;
        let file_offset = checked_usize(
            read_u64(bytes, offset + 8, "ELF segment file offset")?,
            "ELF segment file offset",
        )?;
        let virtual_address = read_u64(bytes, offset + 16, "ELF segment virtual address")?;
        let file_size = checked_usize(
            read_u64(bytes, offset + 32, "ELF segment file size")?,
            "ELF segment file size",
        )?;
        let memory_size = read_u64(bytes, offset + 40, "ELF segment memory size")?;
        let alignment = read_u64(bytes, offset + 48, "ELF segment alignment")?;
        if memory_size < file_size as u64 {
            return Err(format!(
                "ELF load segment {index} memory size {memory_size} is smaller than file size {file_size}"
            ));
        }
        file_offset
            .checked_add(file_size)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| format!("ELF load segment {index} exceeds image bounds"))?;
        if alignment > 1 {
            if !alignment.is_power_of_two() {
                return Err(format!(
                    "ELF load segment {index} alignment {alignment} is not a power of two"
                ));
            }
            if virtual_address % alignment != file_offset as u64 % alignment {
                return Err(format!(
                    "ELF load segment {index} file and virtual addresses violate alignment {alignment}"
                ));
            }
        }
        let file_backed_end = virtual_address
            .checked_add(file_size as u64)
            .ok_or_else(|| format!("ELF load segment {index} virtual range overflows"))?;
        if flags & ELF_PROGRAM_FLAG_EXECUTE != 0
            && entry >= virtual_address
            && entry < file_backed_end
        {
            executable_entry = true;
        }
    }
    if load_count == 0 {
        return Err("ELF executable has no PT_LOAD segment".to_owned());
    }
    if !executable_entry {
        return Err("ELF entry is not inside a file-backed executable PT_LOAD segment".to_owned());
    }
    Ok(())
}

fn validate_elf64_header(bytes: &[u8], kind: &str) -> Result<(), String> {
    if bytes.len() < ELF64_HEADER_SIZE {
        return Err(format!(
            "ELF {kind} is truncated: expected at least {ELF64_HEADER_SIZE} bytes, found {}",
            bytes.len()
        ));
    }
    if bytes.get(..4) != Some(b"\x7fELF") {
        return Err(format!("ELF {kind} magic is invalid"));
    }
    if bytes[4] != ELF_CLASS_64
        || bytes[5] != ELF_DATA_LITTLE_ENDIAN
        || bytes[6] != ELF_VERSION_CURRENT
    {
        return Err(format!(
            "ELF {kind} ident is unsupported: class={} data={} version={}",
            bytes[4], bytes[5], bytes[6]
        ));
    }
    let machine = read_u16(bytes, 18, "ELF machine")?;
    if machine != ELF_MACHINE_X86_64 {
        return Err(format!(
            "ELF {kind} machine is {machine}; expected x86_64 ({ELF_MACHINE_X86_64})"
        ));
    }
    if read_u32(bytes, 20, "ELF version")? != 1 {
        return Err(format!("ELF {kind} version is not EV_CURRENT"));
    }
    let header_size = read_u16(bytes, 52, "ELF header size")? as usize;
    if header_size != ELF64_HEADER_SIZE {
        return Err(format!(
            "ELF {kind} header size is {header_size}; expected {ELF64_HEADER_SIZE}"
        ));
    }
    Ok(())
}

fn checked_table_end(
    offset: usize,
    entry_size: usize,
    count: usize,
    image_size: usize,
    label: &str,
) -> Result<usize, String> {
    offset
        .checked_add(
            entry_size
                .checked_mul(count)
                .ok_or_else(|| format!("{label} size overflows"))?,
        )
        .filter(|end| *end <= image_size)
        .ok_or_else(|| format!("{label} exceeds image bounds"))
}

fn checked_usize(value: u64, label: &str) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| format!("{label} exceeds host address space"))
}

fn canonical_arch(machine_arch: &str) -> Option<&'static str> {
    nuis_runtime::canonical_machine_arch(machine_arch)
}

fn canonical_os(machine_os: &str) -> &str {
    match machine_os.trim().to_ascii_lowercase().as_str() {
        "linux" | "linux-gnu" | "gnu-linux" => "linux",
        _ => "unknown",
    }
}

fn canonical_object_format(object_format: &str) -> &str {
    match object_format.trim().to_ascii_lowercase().as_str() {
        "elf" => "elf",
        _ => "unknown",
    }
}

fn read_u16(bytes: &[u8], offset: usize, label: &str) -> Result<u16, String> {
    let raw: [u8; 2] = bytes
        .get(offset..offset.saturating_add(2))
        .ok_or_else(|| format!("{label} at offset {offset} is truncated"))?
        .try_into()
        .map_err(|_| format!("{label} at offset {offset} is malformed"))?;
    Ok(u16::from_le_bytes(raw))
}

fn read_u32(bytes: &[u8], offset: usize, label: &str) -> Result<u32, String> {
    let raw: [u8; 4] = bytes
        .get(offset..offset.saturating_add(4))
        .ok_or_else(|| format!("{label} at offset {offset} is truncated"))?
        .try_into()
        .map_err(|_| format!("{label} at offset {offset} is malformed"))?;
    Ok(u32::from_le_bytes(raw))
}

fn read_u64(bytes: &[u8], offset: usize, label: &str) -> Result<u64, String> {
    let raw: [u8; 8] = bytes
        .get(offset..offset.saturating_add(8))
        .ok_or_else(|| format!("{label} at offset {offset} is truncated"))?
        .try_into()
        .map_err(|_| format!("{label} at offset {offset} is malformed"))?;
    Ok(u64::from_le_bytes(raw))
}

#[cfg(test)]
#[path = "final_executable_elf_artifact_tests.rs"]
mod tests;
