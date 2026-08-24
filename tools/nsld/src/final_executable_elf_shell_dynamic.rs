use super::report::{needed_library_audit_hash, ElfAmd64ShellNeededLibraryPlan};
use crate::{
    final_executable_elf_dynamic_plan::{
        validate_elf_amd64_dynamic_dependency_plan, ElfAmd64DynamicDependencyPlanReport,
    },
    final_executable_elf_layout::{ELF_AMD64_IMAGE_BASE, ELF_AMD64_PAGE_SIZE},
    final_executable_elf_materialization::application::platform::ElfAmd64PlatformStructurePlanReport,
};
use std::collections::BTreeSet;

const BASE_DYNAMIC_ENTRY_COUNT: usize = 12;
const ELF64_DYNAMIC_ENTRY_SIZE: usize = 16;

pub(super) struct ElfAmd64ShellDynamicLayout {
    pub(super) dependency_plan_hash: Option<String>,
    pub(super) interpreter_identity: Option<String>,
    pub(super) interpreter_path: Option<String>,
    pub(super) interpreter_file_offset: Option<usize>,
    pub(super) interpreter_virtual_address: Option<u64>,
    pub(super) interpreter_bytes: usize,
    pub(super) dynamic_string_source_image_offset: Option<usize>,
    pub(super) dynamic_string_source_bytes: usize,
    pub(super) dynamic_string_file_offset: Option<usize>,
    pub(super) dynamic_string_virtual_address: Option<u64>,
    pub(super) dynamic_string_bytes: usize,
    pub(super) metadata_file_offset: Option<usize>,
    pub(super) metadata_virtual_address: Option<u64>,
    pub(super) metadata_bytes: usize,
    pub(super) dynamic_table_file_offset: Option<usize>,
    pub(super) dynamic_table_virtual_address: Option<u64>,
    pub(super) dynamic_table_bytes: usize,
    pub(super) planned_memory_span_bytes: usize,
    pub(super) needed_libraries: Vec<ElfAmd64ShellNeededLibraryPlan>,
}

impl ElfAmd64ShellDynamicLayout {
    pub(super) fn emits_registered_dependencies(&self) -> bool {
        !self.needed_libraries.is_empty()
    }
}

pub(super) fn build_elf_amd64_shell_dynamic_layout(
    platform: &ElfAmd64PlatformStructurePlanReport,
    dependency_plan: Option<&ElfAmd64DynamicDependencyPlanReport>,
) -> Result<ElfAmd64ShellDynamicLayout, String> {
    if let Some(plan) = dependency_plan {
        validate_elf_amd64_dynamic_dependency_plan(plan)?;
        if plan.platform_structure_plan_hash != platform.plan_hash
            || plan.unresolved_symbol_count != platform.target_count
        {
            return Err("ELF shell dynamic plan rejects platform lineage drift".to_owned());
        }
    }
    let dynamic_enabled = platform.target_count > 0;
    let emission_ready =
        dependency_plan.is_some_and(|plan| plan.plan_ready && !plan.dependencies.is_empty());
    let dependency_plan_hash = dependency_plan.map(|plan| plan.plan_hash.clone());

    let mut interpreter_identity = None;
    let mut interpreter_path = None;
    let mut interpreter_file_offset = None;
    let mut interpreter_virtual_address = None;
    let mut interpreter_bytes = 0usize;
    let mut dynamic_string_source_image_offset = None;
    let mut dynamic_string_source_bytes = 0usize;
    let mut dynamic_string_file_offset = None;
    let mut dynamic_string_virtual_address = None;
    let mut dynamic_string_bytes = 0usize;
    let mut metadata_file_offset = None;
    let mut metadata_virtual_address = None;
    let mut metadata_bytes = 0usize;
    let mut needed_libraries = Vec::new();

    let dynamic_base = if emission_ready {
        let plan = dependency_plan.unwrap();
        let identities = plan
            .dependencies
            .iter()
            .map(|dependency| {
                (
                    dependency.interpreter_identity.as_str(),
                    dependency.interpreter_path.as_str(),
                )
            })
            .collect::<BTreeSet<_>>();
        let selected_interpreters = identities.into_iter().collect::<Vec<_>>();
        let [(identity, path)] = selected_interpreters.as_slice() else {
            return Err("ELF shell dependency plan does not select one interpreter".to_owned());
        };
        let metadata_offset = align_up(platform.planned_memory_span_bytes, ELF_AMD64_PAGE_SIZE)?;
        let interp_bytes = checked_add(path.len(), 1, "interpreter payload")?;
        let dynstr_offset = checked_add(metadata_offset, interp_bytes, "interpreter payload")?;
        let mut string_cursor = platform.dynamic_string_region_bytes;
        for (index, dependency) in plan.dependencies.iter().enumerate() {
            let mut needed = ElfAmd64ShellNeededLibraryPlan {
                needed_id: format!("elf-amd64-shell-needed-library-{index:04}"),
                dependency_audit_hash: dependency.audit_hash.clone(),
                dependency_identity: dependency.dependency_identity.clone(),
                needed_name: dependency.needed_name.clone(),
                dynamic_string_offset: string_cursor,
                audit_hash: String::new(),
            };
            needed.audit_hash = needed_library_audit_hash(&plan.plan_hash, &needed);
            string_cursor = checked_add(
                string_cursor,
                checked_add(dependency.needed_name.len(), 1, "needed library name")?,
                "dynamic string table",
            )?;
            needed_libraries.push(needed);
        }
        let metadata_size = checked_add(interp_bytes, string_cursor, "dynamic metadata")?;
        interpreter_identity = Some((*identity).to_owned());
        interpreter_path = Some((*path).to_owned());
        interpreter_file_offset = Some(metadata_offset);
        interpreter_virtual_address = Some(virtual_address(metadata_offset)?);
        interpreter_bytes = interp_bytes;
        dynamic_string_source_image_offset = Some(platform.dynamic_string_region_image_offset);
        dynamic_string_source_bytes = platform.dynamic_string_region_bytes;
        dynamic_string_file_offset = Some(dynstr_offset);
        dynamic_string_virtual_address = Some(virtual_address(dynstr_offset)?);
        dynamic_string_bytes = string_cursor;
        metadata_file_offset = Some(metadata_offset);
        metadata_virtual_address = Some(virtual_address(metadata_offset)?);
        metadata_bytes = metadata_size;
        checked_add(metadata_offset, metadata_size, "dynamic metadata")?
    } else {
        platform.planned_memory_span_bytes
    };

    let dynamic_entry_count = if dynamic_enabled {
        checked_add(
            BASE_DYNAMIC_ENTRY_COUNT,
            needed_libraries.len() + usize::from(!needed_libraries.is_empty()),
            "dynamic entry count",
        )?
    } else {
        0
    };
    let dynamic_table_bytes = dynamic_entry_count
        .checked_mul(ELF64_DYNAMIC_ENTRY_SIZE)
        .ok_or_else(|| "ELF shell dynamic table size overflows".to_owned())?;
    let dynamic_table_file_offset = dynamic_enabled
        .then(|| align_up(dynamic_base, ELF_AMD64_PAGE_SIZE))
        .transpose()?;
    let dynamic_table_virtual_address =
        dynamic_table_file_offset.map(virtual_address).transpose()?;
    let planned_memory_span_bytes = match dynamic_table_file_offset {
        Some(offset) => checked_add(offset, dynamic_table_bytes, "dynamic table")?,
        None => platform.planned_memory_span_bytes,
    };
    Ok(ElfAmd64ShellDynamicLayout {
        dependency_plan_hash,
        interpreter_identity,
        interpreter_path,
        interpreter_file_offset,
        interpreter_virtual_address,
        interpreter_bytes,
        dynamic_string_source_image_offset,
        dynamic_string_source_bytes,
        dynamic_string_file_offset,
        dynamic_string_virtual_address,
        dynamic_string_bytes,
        metadata_file_offset,
        metadata_virtual_address,
        metadata_bytes,
        dynamic_table_file_offset,
        dynamic_table_virtual_address,
        dynamic_table_bytes,
        planned_memory_span_bytes,
        needed_libraries,
    })
}

fn align_up(value: usize, alignment: usize) -> Result<usize, String> {
    value
        .checked_add(alignment - 1)
        .map(|value| value & !(alignment - 1))
        .ok_or_else(|| "ELF shell dynamic alignment overflows".to_owned())
}

fn checked_add(lhs: usize, rhs: usize, label: &str) -> Result<usize, String> {
    lhs.checked_add(rhs)
        .ok_or_else(|| format!("ELF shell {label} span overflows"))
}

fn virtual_address(offset: usize) -> Result<u64, String> {
    ELF_AMD64_IMAGE_BASE
        .checked_add(
            u64::try_from(offset)
                .map_err(|_| "ELF shell offset exceeds u64 address space".to_owned())?,
        )
        .ok_or_else(|| "ELF shell virtual address overflows".to_owned())
}
