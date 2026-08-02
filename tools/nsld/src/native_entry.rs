use nuis_runtime::{NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1, NUIS_NATIVE_ENTRY_SECTION_KIND};
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
    NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1
}

pub(crate) fn native_entry_section_kind() -> &'static str {
    NUIS_NATIVE_ENTRY_SECTION_KIND
}

pub(crate) fn native_entry_machine_arch(machine_arch: &str) -> Result<&'static str, String> {
    nuis_runtime::canonical_machine_arch(machine_arch).ok_or_else(|| {
        format!(
            "unsupported Nsld native lifecycle-entry architecture `{machine_arch}` for {} / {}",
            NUIS_NATIVE_ENTRY_SECTION_KIND, NUIS_LIFECYCLE_ENTRY_CONTEXT_ABI_V1
        )
    })
}

fn native_entry_code(machine_arch: &str, calling_abi: &str) -> Result<&'static [u8], String> {
    match (native_entry_machine_arch(machine_arch)?, calling_abi) {
        // ldr w1,[x0,#8]; cmp w1,#1; cset w0,ne; ret
        (nuis_runtime::NUIS_MACHINE_ARCH_AARCH64, "aapcs64" | "aapcs64-darwin") => Ok(&[
            0x01, 0x08, 0x40, 0xb9, 0x3f, 0x04, 0x00, 0x71, 0xe0, 0x07, 0x9f, 0x1a, 0xc0, 0x03,
            0x5f, 0xd6,
        ]),
        // cmp dword ptr [rdi+8],1; setne al; movzx eax,al; ret
        (nuis_runtime::NUIS_MACHINE_ARCH_X86_64, "sysv64") => Ok(&[
            0x83, 0x7f, 0x08, 0x01, 0x0f, 0x95, 0xc0, 0x0f, 0xb6, 0xc0, 0xc3,
        ]),
        // cmp dword ptr [rcx+8],1; setne al; movzx eax,al; ret
        (nuis_runtime::NUIS_MACHINE_ARCH_X86_64, "win64") => Ok(&[
            0x83, 0x79, 0x08, 0x01, 0x0f, 0x95, 0xc0, 0x0f, 0xb6, 0xc0, 0xc3,
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
                0x01, 0x08, 0x40, 0xb9, 0x3f, 0x04, 0x00, 0x71, 0xe0, 0x07, 0x9f, 0x1a, 0xc0, 0x03,
                0x5f, 0xd6
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
            &[0x83, 0x7f, 0x08, 0x01, 0x0f, 0x95, 0xc0, 0x0f, 0xb6, 0xc0, 0xc3]
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
            &[0x83, 0x79, 0x08, 0x01, 0x0f, 0x95, 0xc0, 0x0f, 0xb6, 0xc0, 0xc3]
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
