use nuis_runtime::{NUIS_LIFECYCLE_ENTRY_DISPATCH_ABI_V2, NUIS_NATIVE_ENTRY_SECTION_KIND};
use std::{fs, ops::Range, path::PathBuf};

pub(crate) const NATIVE_ENTRY_ASSET_FILE: &str = "nuis.nsld.native-entry.bin";
pub(crate) const NATIVE_ENTRY_RELOCATION_SLOT_BYTES: usize = 8;

pub(crate) fn native_entry_asset_path(plan: &nuisc::linker::LinkPlan) -> PathBuf {
    PathBuf::from(&plan.output_dir).join(NATIVE_ENTRY_ASSET_FILE)
}

pub(crate) fn emit_native_entry_asset(plan: &nuisc::linker::LinkPlan) -> Result<PathBuf, String> {
    let code = native_entry_code(&plan.cpu_target.machine_arch, &plan.cpu_target.calling_abi)?;
    let mut bytes = vec![0; NATIVE_ENTRY_RELOCATION_SLOT_BYTES];
    bytes.extend_from_slice(code);
    let path = native_entry_asset_path(plan);
    fs::write(&path, bytes).map_err(|error| {
        format!(
            "failed to write Nsld native lifecycle entry `{}`: {error}",
            path.display()
        )
    })?;
    Ok(path)
}

pub(crate) fn native_entry_code_range(section_size_bytes: usize) -> Option<Range<usize>> {
    (section_size_bytes > NATIVE_ENTRY_RELOCATION_SLOT_BYTES)
        .then_some(NATIVE_ENTRY_RELOCATION_SLOT_BYTES..section_size_bytes)
}

pub(crate) fn native_entry_abi_contract() -> &'static str {
    NUIS_LIFECYCLE_ENTRY_DISPATCH_ABI_V2
}

pub(crate) fn native_entry_section_kind() -> &'static str {
    NUIS_NATIVE_ENTRY_SECTION_KIND
}

pub(crate) fn native_entry_machine_arch(machine_arch: &str) -> Result<&'static str, String> {
    nuis_runtime::canonical_machine_arch(machine_arch).ok_or_else(|| {
        format!(
            "unsupported Nsld native lifecycle-entry architecture `{machine_arch}` for {} / {}",
            NUIS_NATIVE_ENTRY_SECTION_KIND, NUIS_LIFECYCLE_ENTRY_DISPATCH_ABI_V2
        )
    })
}

fn native_entry_code(machine_arch: &str, calling_abi: &str) -> Result<&'static [u8], String> {
    match (native_entry_machine_arch(machine_arch)?, calling_abi) {
        // Preserve LR, derive table/request/response from the context prefix,
        // call table.handler(context, table, request, response), and return status.
        (nuis_runtime::NUIS_MACHINE_ARCH_AARCH64, "aapcs64" | "aapcs64-darwin") => Ok(&[
            0xfd, 0x7b, 0xbf, 0xa9, 0xfd, 0x03, 0x00, 0x91, 0x01, 0x00, 0x01, 0x91, 0x02, 0xc0,
            0x01, 0x91, 0x03, 0x60, 0x02, 0x91, 0x24, 0x14, 0x40, 0xf9, 0x80, 0x00, 0x3f, 0xd6,
            0xfd, 0x7b, 0xc1, 0xa8, 0xc0, 0x03, 0x5f, 0xd6,
        ]),
        // SysV: rdi=context, rsi=table, rdx=request, rcx=response.
        (nuis_runtime::NUIS_MACHINE_ARCH_X86_64, "sysv64") => Ok(&[
            0x48, 0x83, 0xec, 0x08, 0x48, 0x8d, 0x77, 0x40, 0x48, 0x8d, 0x57, 0x70, 0x48, 0x8d,
            0x8f, 0x98, 0x00, 0x00, 0x00, 0x48, 0x8b, 0x46, 0x28, 0xff, 0xd0, 0x48, 0x83, 0xc4,
            0x08, 0xc3,
        ]),
        // Win64: rcx=context, rdx=table, r8=request, r9=response.
        (nuis_runtime::NUIS_MACHINE_ARCH_X86_64, "win64") => Ok(&[
            0x48, 0x83, 0xec, 0x28, 0x48, 0x8d, 0x51, 0x40, 0x4c, 0x8d, 0x41, 0x70, 0x4c, 0x8d,
            0x89, 0x98, 0x00, 0x00, 0x00, 0x48, 0x8b, 0x42, 0x28, 0xff, 0xd0, 0x48, 0x83, 0xc4,
            0x28, 0xc3,
        ]),
        (arch, abi) => Err(format!(
            "unsupported Nsld native lifecycle-entry calling ABI `{abi}` for `{arch}`"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::main_test_support::empty_link_plan;

    #[test]
    fn emits_deterministic_aarch64_entry_after_relocation_slot() {
        let dir =
            std::env::temp_dir().join(format!("nsld-native-entry-aarch64-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();

        let path = emit_native_entry_asset(&plan).unwrap();
        let bytes = fs::read(path).unwrap();
        fs::remove_dir_all(dir).unwrap();

        assert_eq!(&bytes[..NATIVE_ENTRY_RELOCATION_SLOT_BYTES], &[0; 8]);
        assert_eq!(
            &bytes[NATIVE_ENTRY_RELOCATION_SLOT_BYTES..],
            &[
                0xfd, 0x7b, 0xbf, 0xa9, 0xfd, 0x03, 0x00, 0x91, 0x01, 0x00, 0x01, 0x91, 0x02, 0xc0,
                0x01, 0x91, 0x03, 0x60, 0x02, 0x91, 0x24, 0x14, 0x40, 0xf9, 0x80, 0x00, 0x3f, 0xd6,
                0xfd, 0x7b, 0xc1, 0xa8, 0xc0, 0x03, 0x5f, 0xd6
            ]
        );
    }

    #[test]
    fn emits_x86_64_entry_without_host_architecture_coupling() {
        let dir =
            std::env::temp_dir().join(format!("nsld-native-entry-x86-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.cpu_target.machine_arch = "x86_64".to_owned();
        plan.cpu_target.calling_abi = "sysv64".to_owned();

        let path = emit_native_entry_asset(&plan).unwrap();
        let bytes = fs::read(path).unwrap();
        fs::remove_dir_all(dir).unwrap();

        assert_eq!(&bytes[..NATIVE_ENTRY_RELOCATION_SLOT_BYTES], &[0; 8]);
        assert_eq!(
            &bytes[NATIVE_ENTRY_RELOCATION_SLOT_BYTES..],
            &[
                0x48, 0x83, 0xec, 0x08, 0x48, 0x8d, 0x77, 0x40, 0x48, 0x8d, 0x57, 0x70, 0x48, 0x8d,
                0x8f, 0x98, 0x00, 0x00, 0x00, 0x48, 0x8b, 0x46, 0x28, 0xff, 0xd0, 0x48, 0x83, 0xc4,
                0x08, 0xc3
            ]
        );
    }

    #[test]
    fn emits_x86_64_win64_context_probe() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-native-entry-x86-win64-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.cpu_target.machine_arch = "amd64".to_owned();
        plan.cpu_target.calling_abi = "win64".to_owned();

        let path = emit_native_entry_asset(&plan).unwrap();
        let bytes = fs::read(path).unwrap();
        fs::remove_dir_all(dir).unwrap();

        assert_eq!(
            &bytes[NATIVE_ENTRY_RELOCATION_SLOT_BYTES..],
            &[
                0x48, 0x83, 0xec, 0x28, 0x48, 0x8d, 0x51, 0x40, 0x4c, 0x8d, 0x41, 0x70, 0x4c, 0x8d,
                0x89, 0x98, 0x00, 0x00, 0x00, 0x48, 0x8b, 0x42, 0x28, 0xff, 0xd0, 0x48, 0x83, 0xc4,
                0x28, 0xc3
            ]
        );
    }

    #[test]
    fn rejects_unregistered_native_entry_architecture() {
        let mut plan = empty_link_plan();
        plan.cpu_target.machine_arch = "mystery-cpu".to_owned();
        let error = emit_native_entry_asset(&plan).unwrap_err();
        assert!(error.contains("unsupported Nsld native lifecycle-entry architecture"));
    }

    #[test]
    fn rejects_unregistered_calling_abi() {
        let mut plan = empty_link_plan();
        plan.cpu_target.calling_abi = "mystery-aapcs64".to_owned();
        let error = emit_native_entry_asset(&plan).unwrap_err();
        assert!(error.contains("unsupported Nsld native lifecycle-entry calling ABI"));
    }

    #[test]
    fn normalizes_machine_architecture_aliases_for_container_contracts() {
        assert_eq!(native_entry_machine_arch("arm64").unwrap(), "aarch64");
        assert_eq!(native_entry_machine_arch("amd64").unwrap(), "x86_64");
    }
}
