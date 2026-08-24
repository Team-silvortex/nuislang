use super::report::ElfAmd64ShellNeededLibraryPlan;
use crate::{
    final_executable_elf_dynamic_plan::{
        validate_elf_amd64_dynamic_dependency_plan, ElfAmd64DynamicDependencyPlanReport,
    },
    final_executable_elf_layout::ELF_AMD64_IMAGE_BASE,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
};

pub(crate) const ELF64_VERSION_SYMBOL_ENTRY_SIZE: usize = 2;
pub(crate) const ELF64_VERSION_NEED_HEADER_SIZE: usize = 16;
pub(crate) const ELF64_VERSION_NEED_AUX_SIZE: usize = 16;
pub(crate) const ELF_VERSION_NEED_CURRENT: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64ShellVersionSymbolPlan {
    pub(crate) version_symbol_id: String,
    pub(crate) target_symbol: String,
    pub(crate) dynamic_symbol_index: usize,
    pub(crate) version_index: u16,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64ShellVersionAuxPlan {
    pub(crate) version_aux_id: String,
    pub(crate) symbol_version_identity: String,
    pub(crate) symbol_version_name: String,
    pub(crate) dynamic_string_offset: usize,
    pub(crate) version_index: u16,
    pub(crate) version_hash: u32,
    pub(crate) record_offset: usize,
    pub(crate) next_offset: usize,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64ShellVersionNeedPlan {
    pub(crate) version_need_id: String,
    pub(crate) dependency_audit_hash: String,
    pub(crate) dependency_identity: String,
    pub(crate) needed_name: String,
    pub(crate) needed_name_dynamic_string_offset: usize,
    pub(crate) record_offset: usize,
    pub(crate) auxiliary_offset: usize,
    pub(crate) next_offset: usize,
    pub(crate) auxiliaries: Vec<ElfAmd64ShellVersionAuxPlan>,
    pub(crate) audit_hash: String,
}

pub(super) struct ElfAmd64ShellVersionMetadataLayout {
    pub(super) dynamic_string_bytes: usize,
    pub(super) version_symbol_file_offset: usize,
    pub(super) version_symbol_virtual_address: u64,
    pub(super) version_symbol_bytes: usize,
    pub(super) version_need_file_offset: usize,
    pub(super) version_need_virtual_address: u64,
    pub(super) version_need_bytes: usize,
    pub(super) metadata_end: usize,
    pub(super) version_symbols: Vec<ElfAmd64ShellVersionSymbolPlan>,
    pub(super) version_needs: Vec<ElfAmd64ShellVersionNeedPlan>,
}

pub(super) fn build_elf_amd64_shell_version_metadata_layout(
    dependency_plan: &ElfAmd64DynamicDependencyPlanReport,
    needed_libraries: &[ElfAmd64ShellNeededLibraryPlan],
    dynamic_symbol_entry_count: usize,
    dynamic_string_file_offset: usize,
    initial_dynamic_string_bytes: usize,
) -> Result<ElfAmd64ShellVersionMetadataLayout, String> {
    validate_elf_amd64_dynamic_dependency_plan(dependency_plan)?;
    if !dependency_plan.plan_ready
        || dependency_plan.bindings.is_empty()
        || dynamic_symbol_entry_count != dependency_plan.bindings.len() + 1
        || needed_libraries.len() != dependency_plan.dependencies.len()
    {
        return Err("ELF shell version metadata rejects dependency coverage drift".to_owned());
    }
    let needed_by_dependency = needed_libraries
        .iter()
        .map(|needed| (needed.dependency_audit_hash.as_str(), needed))
        .collect::<BTreeMap<_, _>>();
    let mut string_cursor = initial_dynamic_string_bytes;
    let mut version_string_offsets = BTreeMap::<String, usize>::new();
    let mut version_symbols = Vec::with_capacity(dependency_plan.bindings.len());
    let mut bindings_by_dependency = BTreeMap::<&str, Vec<_>>::new();
    for binding in &dependency_plan.bindings {
        if binding.dynamic_symbol_index == 0
            || binding.symbol_version_index < 2
            || binding.symbol_version_name.is_empty()
        {
            return Err("ELF shell version metadata rejects a version binding".to_owned());
        }
        bindings_by_dependency
            .entry(binding.dependency_audit_hash.as_str())
            .or_default()
            .push(binding);
        let mut symbol = ElfAmd64ShellVersionSymbolPlan {
            version_symbol_id: format!(
                "elf-amd64-shell-version-symbol-{:04}",
                version_symbols.len()
            ),
            target_symbol: binding.target_symbol.clone(),
            dynamic_symbol_index: binding.dynamic_symbol_index,
            version_index: binding.symbol_version_index,
            audit_hash: String::new(),
        };
        symbol.audit_hash = version_symbol_audit_hash(&dependency_plan.plan_hash, &symbol);
        version_symbols.push(symbol);
    }
    version_symbols.sort_by_key(|symbol| symbol.dynamic_symbol_index);
    if version_symbols
        .iter()
        .enumerate()
        .any(|(index, symbol)| symbol.dynamic_symbol_index != index + 1)
    {
        return Err("ELF shell version-symbol indexes are not contiguous".to_owned());
    }

    let mut version_needs = Vec::with_capacity(dependency_plan.dependencies.len());
    let mut need_cursor = 0usize;
    for dependency in &dependency_plan.dependencies {
        let needed = needed_by_dependency
            .get(dependency.audit_hash.as_str())
            .ok_or_else(|| "ELF shell version metadata lost a needed library".to_owned())?;
        let bindings = bindings_by_dependency
            .remove(dependency.audit_hash.as_str())
            .ok_or_else(|| "ELF shell version metadata lost dependency bindings".to_owned())?;
        let mut unique_versions = BTreeMap::new();
        for binding in bindings {
            let version = (
                binding.symbol_version_identity.as_str(),
                binding.symbol_version_name.as_str(),
                binding.symbol_version_hash,
            );
            if unique_versions
                .insert(binding.symbol_version_index, version)
                .is_some_and(|previous| previous != version)
            {
                return Err("ELF shell version index maps to conflicting names".to_owned());
            }
        }
        let record_offset = need_cursor;
        let auxiliary_offset = ELF64_VERSION_NEED_HEADER_SIZE;
        let mut auxiliaries = Vec::with_capacity(unique_versions.len());
        for (ordinal, (version_index, (identity, name, version_hash))) in
            unique_versions.into_iter().enumerate()
        {
            let dynamic_string_offset = match version_string_offsets.get(name).copied() {
                Some(offset) => offset,
                None => {
                    let offset = string_cursor;
                    string_cursor = checked_add(
                        string_cursor,
                        checked_add(name.len(), 1, "version name")?,
                        "dynamic string table",
                    )?;
                    version_string_offsets.insert(name.to_owned(), offset);
                    offset
                }
            };
            let aux_record_offset = checked_add(
                record_offset,
                checked_add(
                    ELF64_VERSION_NEED_HEADER_SIZE,
                    checked_mul(ordinal, ELF64_VERSION_NEED_AUX_SIZE, "version auxiliary")?,
                    "version auxiliary",
                )?,
                "version auxiliary",
            )?;
            let mut auxiliary = ElfAmd64ShellVersionAuxPlan {
                version_aux_id: format!(
                    "elf-amd64-shell-version-aux-{:04}-{ordinal:04}",
                    version_needs.len()
                ),
                symbol_version_identity: identity.to_owned(),
                symbol_version_name: name.to_owned(),
                dynamic_string_offset,
                version_index,
                version_hash,
                record_offset: aux_record_offset,
                next_offset: usize::from(
                    ordinal + 1
                        < unique_versions_len(&dependency_plan.bindings, &dependency.audit_hash),
                ) * ELF64_VERSION_NEED_AUX_SIZE,
                audit_hash: String::new(),
            };
            auxiliary.audit_hash = version_aux_audit_hash(&dependency_plan.plan_hash, &auxiliary);
            auxiliaries.push(auxiliary);
        }
        let record_bytes = checked_add(
            ELF64_VERSION_NEED_HEADER_SIZE,
            checked_mul(
                auxiliaries.len(),
                ELF64_VERSION_NEED_AUX_SIZE,
                "version need auxiliaries",
            )?,
            "version need record",
        )?;
        need_cursor = checked_add(need_cursor, record_bytes, "version need table")?;
        let mut need = ElfAmd64ShellVersionNeedPlan {
            version_need_id: format!("elf-amd64-shell-version-need-{:04}", version_needs.len()),
            dependency_audit_hash: dependency.audit_hash.clone(),
            dependency_identity: dependency.dependency_identity.clone(),
            needed_name: needed.needed_name.clone(),
            needed_name_dynamic_string_offset: needed.dynamic_string_offset,
            record_offset,
            auxiliary_offset,
            next_offset: 0,
            auxiliaries,
            audit_hash: String::new(),
        };
        need.audit_hash = version_need_audit_hash(&dependency_plan.plan_hash, &need);
        version_needs.push(need);
    }
    if !bindings_by_dependency.is_empty() {
        return Err("ELF shell version metadata has unowned bindings".to_owned());
    }
    for index in 0..version_needs.len().saturating_sub(1) {
        version_needs[index].next_offset = version_needs[index + 1]
            .record_offset
            .checked_sub(version_needs[index].record_offset)
            .ok_or_else(|| "ELF shell version-need order underflows".to_owned())?;
        version_needs[index].audit_hash =
            version_need_audit_hash(&dependency_plan.plan_hash, &version_needs[index]);
    }

    let dynamic_string_end = checked_add(
        dynamic_string_file_offset,
        string_cursor,
        "dynamic string table",
    )?;
    let version_symbol_file_offset = align_up(dynamic_string_end, 2)?;
    let version_symbol_bytes = checked_mul(
        dynamic_symbol_entry_count,
        ELF64_VERSION_SYMBOL_ENTRY_SIZE,
        "version-symbol table",
    )?;
    let version_need_file_offset = align_up(
        checked_add(
            version_symbol_file_offset,
            version_symbol_bytes,
            "version-symbol table",
        )?,
        8,
    )?;
    let metadata_end = checked_add(version_need_file_offset, need_cursor, "version-need table")?;
    Ok(ElfAmd64ShellVersionMetadataLayout {
        dynamic_string_bytes: string_cursor,
        version_symbol_file_offset,
        version_symbol_virtual_address: virtual_address(version_symbol_file_offset)?,
        version_symbol_bytes,
        version_need_file_offset,
        version_need_virtual_address: virtual_address(version_need_file_offset)?,
        version_need_bytes: need_cursor,
        metadata_end,
        version_symbols,
        version_needs,
    })
}

pub(super) fn append_version_names(
    dynamic_strings: &mut Vec<u8>,
    version_needs: &[ElfAmd64ShellVersionNeedPlan],
) -> Result<(), String> {
    let mut names = BTreeMap::new();
    for auxiliary in version_needs
        .iter()
        .flat_map(|need| need.auxiliaries.iter())
    {
        if names
            .insert(
                auxiliary.dynamic_string_offset,
                auxiliary.symbol_version_name.as_str(),
            )
            .is_some_and(|previous| previous != auxiliary.symbol_version_name)
        {
            return Err("ELF shell version string offset is ambiguous".to_owned());
        }
    }
    for (offset, name) in names {
        if offset != dynamic_strings.len() || name.is_empty() || name.as_bytes().contains(&0) {
            return Err("ELF shell version string slot is invalid".to_owned());
        }
        dynamic_strings.extend_from_slice(name.as_bytes());
        dynamic_strings.push(0);
    }
    Ok(())
}

pub(super) fn encode_version_symbol_table(
    width: usize,
    version_symbols: &[ElfAmd64ShellVersionSymbolPlan],
) -> Result<Vec<u8>, String> {
    if width < ELF64_VERSION_SYMBOL_ENTRY_SIZE
        || !width.is_multiple_of(ELF64_VERSION_SYMBOL_ENTRY_SIZE)
        || width / ELF64_VERSION_SYMBOL_ENTRY_SIZE != version_symbols.len() + 1
    {
        return Err("ELF shell version-symbol table width drift".to_owned());
    }
    let mut bytes = vec![0; width];
    let mut seen = BTreeSet::new();
    for symbol in version_symbols {
        if symbol.dynamic_symbol_index == 0
            || symbol.version_index < 2
            || !seen.insert(symbol.dynamic_symbol_index)
        {
            return Err("ELF shell version-symbol encoding rejects its plan".to_owned());
        }
        let offset = checked_mul(
            symbol.dynamic_symbol_index,
            ELF64_VERSION_SYMBOL_ENTRY_SIZE,
            "version-symbol entry",
        )?;
        write_u16(&mut bytes, offset, symbol.version_index)?;
    }
    Ok(bytes)
}

pub(super) fn encode_version_need_table(
    width: usize,
    version_needs: &[ElfAmd64ShellVersionNeedPlan],
) -> Result<Vec<u8>, String> {
    let mut bytes = vec![0; width];
    for need in version_needs {
        write_u16(&mut bytes, need.record_offset, ELF_VERSION_NEED_CURRENT)?;
        write_u16(
            &mut bytes,
            need.record_offset + 2,
            u16::try_from(need.auxiliaries.len())
                .map_err(|_| "ELF shell version auxiliary count exceeds u16".to_owned())?,
        )?;
        write_u32(
            &mut bytes,
            need.record_offset + 4,
            u32::try_from(need.needed_name_dynamic_string_offset)
                .map_err(|_| "ELF shell needed name offset exceeds u32".to_owned())?,
        )?;
        write_u32(
            &mut bytes,
            need.record_offset + 8,
            u32::try_from(need.auxiliary_offset)
                .map_err(|_| "ELF shell version auxiliary offset exceeds u32".to_owned())?,
        )?;
        write_u32(
            &mut bytes,
            need.record_offset + 12,
            u32::try_from(need.next_offset)
                .map_err(|_| "ELF shell version need offset exceeds u32".to_owned())?,
        )?;
        for auxiliary in &need.auxiliaries {
            write_u32(&mut bytes, auxiliary.record_offset, auxiliary.version_hash)?;
            write_u16(&mut bytes, auxiliary.record_offset + 4, 0)?;
            write_u16(
                &mut bytes,
                auxiliary.record_offset + 6,
                auxiliary.version_index,
            )?;
            write_u32(
                &mut bytes,
                auxiliary.record_offset + 8,
                u32::try_from(auxiliary.dynamic_string_offset)
                    .map_err(|_| "ELF shell version name offset exceeds u32".to_owned())?,
            )?;
            write_u32(
                &mut bytes,
                auxiliary.record_offset + 12,
                u32::try_from(auxiliary.next_offset)
                    .map_err(|_| "ELF shell next version offset exceeds u32".to_owned())?,
            )?;
        }
    }
    Ok(bytes)
}

pub(super) fn append_version_plan_canonical(
    out: &mut String,
    version_symbols: &[ElfAmd64ShellVersionSymbolPlan],
    version_needs: &[ElfAmd64ShellVersionNeedPlan],
) {
    for symbol in version_symbols {
        append_text(out, &symbol.version_symbol_id);
        append_text(out, &symbol.audit_hash);
    }
    for need in version_needs {
        append_text(out, &need.version_need_id);
        append_text(out, &need.audit_hash);
        for auxiliary in &need.auxiliaries {
            append_text(out, &auxiliary.version_aux_id);
            append_text(out, &auxiliary.audit_hash);
        }
    }
}

fn unique_versions_len(
    bindings: &[crate::final_executable_elf_dynamic_plan::ElfAmd64DynamicSymbolPlan],
    dependency_audit_hash: &str,
) -> usize {
    bindings
        .iter()
        .filter(|binding| binding.dependency_audit_hash == dependency_audit_hash)
        .map(|binding| binding.symbol_version_index)
        .collect::<BTreeSet<_>>()
        .len()
}

fn version_symbol_audit_hash(plan_hash: &str, symbol: &ElfAmd64ShellVersionSymbolPlan) -> String {
    let mut out = String::new();
    append_text(&mut out, plan_hash);
    append_text(&mut out, &symbol.version_symbol_id);
    append_text(&mut out, &symbol.target_symbol);
    writeln!(
        out,
        "symbol={}|{}",
        symbol.dynamic_symbol_index, symbol.version_index
    )
    .unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

fn version_aux_audit_hash(plan_hash: &str, aux: &ElfAmd64ShellVersionAuxPlan) -> String {
    let mut out = String::new();
    for value in [
        plan_hash,
        &aux.version_aux_id,
        &aux.symbol_version_identity,
        &aux.symbol_version_name,
    ] {
        append_text(&mut out, value);
    }
    writeln!(
        out,
        "aux={}|{}|{}|{}|{}",
        aux.dynamic_string_offset,
        aux.version_index,
        aux.version_hash,
        aux.record_offset,
        aux.next_offset
    )
    .unwrap();
    crate::fnv1a64_hex(out.as_bytes())
}

fn version_need_audit_hash(plan_hash: &str, need: &ElfAmd64ShellVersionNeedPlan) -> String {
    let mut out = String::new();
    for value in [
        plan_hash,
        &need.version_need_id,
        &need.dependency_audit_hash,
        &need.dependency_identity,
        &need.needed_name,
    ] {
        append_text(&mut out, value);
    }
    writeln!(
        out,
        "need={}|{}|{}|{}|{}",
        need.needed_name_dynamic_string_offset,
        need.record_offset,
        need.auxiliary_offset,
        need.next_offset,
        need.auxiliaries.len()
    )
    .unwrap();
    for auxiliary in &need.auxiliaries {
        append_text(&mut out, &auxiliary.audit_hash);
    }
    crate::fnv1a64_hex(out.as_bytes())
}

fn align_up(value: usize, alignment: usize) -> Result<usize, String> {
    value
        .checked_add(alignment - 1)
        .map(|value| value & !(alignment - 1))
        .ok_or_else(|| "ELF shell version alignment overflows".to_owned())
}

fn checked_add(lhs: usize, rhs: usize, label: &str) -> Result<usize, String> {
    lhs.checked_add(rhs)
        .ok_or_else(|| format!("ELF shell {label} span overflows"))
}

fn checked_mul(lhs: usize, rhs: usize, label: &str) -> Result<usize, String> {
    lhs.checked_mul(rhs)
        .ok_or_else(|| format!("ELF shell {label} size overflows"))
}

fn virtual_address(offset: usize) -> Result<u64, String> {
    ELF_AMD64_IMAGE_BASE
        .checked_add(
            u64::try_from(offset).map_err(|_| "ELF shell version offset exceeds u64".to_owned())?,
        )
        .ok_or_else(|| "ELF shell version address overflows".to_owned())
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) -> Result<(), String> {
    let end = checked_add(offset, 2, "u16 write")?;
    bytes
        .get_mut(offset..end)
        .ok_or_else(|| "ELF shell version u16 write exceeds table".to_owned())?
        .copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) -> Result<(), String> {
    let end = checked_add(offset, 4, "u32 write")?;
    bytes
        .get_mut(offset..end)
        .ok_or_else(|| "ELF shell version u32 write exceeds table".to_owned())?
        .copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}
