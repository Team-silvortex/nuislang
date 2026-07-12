use std::fs;

pub(crate) use super::container_hashes::*;
pub(crate) use super::container_model::*;
pub(crate) use super::container_render::{render_container_plan_toml, render_container_toml};

use super::reports::NsldAssembleSectionDiagnostic;

pub(crate) fn payload_size(sections: &[NsldContainerSectionEntry]) -> usize {
    sections
        .iter()
        .map(|section| section.size_bytes)
        .fold(0usize, usize::saturating_add)
}

pub(crate) fn layout_hash(
    container_magic: &str,
    container_version: usize,
    section_count: usize,
    section_table_hash: &str,
    output_path: &str,
    hash_bytes: fn(&[u8]) -> String,
) -> String {
    let material = format!(
        "{container_magic}\t{container_version}\t{section_count}\t{section_table_hash}\t{output_path}\n"
    );
    hash_bytes(material.as_bytes())
}

pub(crate) fn section_entries(
    sections: &[NsldAssembleSectionDiagnostic],
    hash_bytes: fn(&[u8]) -> String,
) -> Vec<NsldContainerSectionEntry> {
    let mut offset = 0usize;
    sections
        .iter()
        .map(|section| {
            let size_bytes = fs::metadata(&section.source_path)
                .map(|metadata| metadata.len() as usize)
                .unwrap_or(0);
            let payload_hash = fs::read(&section.source_path)
                .map(|bytes| hash_bytes(&bytes))
                .unwrap_or_else(|_| "missing".to_owned());
            let entry = NsldContainerSectionEntry {
                order_index: section.order_index,
                section_id: section.section_id.clone(),
                section_kind: section.section_kind.clone(),
                source_path: section.source_path.clone(),
                source_hash: section.source_hash.clone(),
                payload_hash,
                required: section.required,
                offset,
                size_bytes,
            };
            offset = offset.saturating_add(size_bytes);
            entry
        })
        .collect()
}

pub(crate) fn payload_bytes(sections: &[NsldContainerSectionEntry]) -> Vec<u8> {
    let mut payload = Vec::new();
    for section in sections {
        if let Ok(bytes) = fs::read(&section.source_path) {
            payload.extend_from_slice(&bytes);
        }
    }
    payload
}

pub(crate) fn payload_hash(
    sections: &[NsldContainerSectionEntry],
    hash_bytes: fn(&[u8]) -> String,
) -> String {
    hash_bytes(&payload_bytes(sections))
}

pub(crate) fn external_imports(plan: &nuisc::linker::LinkPlan) -> Vec<NsldContainerExternalImport> {
    let mut imports = Vec::new();
    let mut push_import = |import_kind: &str, import_name: String, provider: &str| {
        let index = imports.len();
        imports.push(NsldContainerExternalImport {
            import_id: format!("imp{index:04}.{import_kind}"),
            import_kind: import_kind.to_owned(),
            import_name,
            provider: provider.to_owned(),
            required: true,
        });
    };

    if matches!(
        plan.final_stage.link_mode.as_str(),
        "host-toolchain-finalize" | "bundle-packaging"
    ) {
        push_import(
            "final-stage-driver",
            plan.final_stage.driver.clone(),
            "host-toolchain",
        );
    }
    if !plan.cpu_target.clang_target.is_empty() {
        push_import(
            "clang-target",
            plan.cpu_target.clang_target.clone(),
            "host-toolchain",
        );
    }
    if plan.final_stage.link_mode == "bundle-packaging" {
        push_import(
            "host-launcher-wrapper",
            "host-launcher-wrapper".to_owned(),
            "host-toolchain",
        );
    }
    if !plan.hetero_calculate.c_world_policy.is_empty()
        && plan.hetero_calculate.c_world_policy != "none"
    {
        push_import(
            "c-world-policy",
            plan.hetero_calculate.c_world_policy.clone(),
            "c-world-wrapper",
        );
    }

    imports
}

pub(crate) fn compatibility_domains(
    plan: &nuisc::linker::LinkPlan,
    sections: &[NsldContainerSectionEntry],
) -> Vec<NsldContainerCompatibilityDomain> {
    let has_native_object = sections
        .iter()
        .any(|section| section.section_kind == "native-object-output");
    let has_c_world_policy = !plan.hetero_calculate.c_world_policy.is_empty()
        && plan.hetero_calculate.c_world_policy != "none";
    let needs_host_finalize = matches!(
        plan.final_stage.link_mode.as_str(),
        "host-toolchain-finalize" | "bundle-packaging"
    );
    if !(has_native_object || has_c_world_policy || needs_host_finalize) {
        return Vec::new();
    }

    vec![NsldContainerCompatibilityDomain {
        domain_id: "compat0000.cffi-von-neumann".to_owned(),
        domain_kind: "cffi-host-compat".to_owned(),
        paradigm: "classic-von-neumann-host".to_owned(),
        lifecycle_hook: "on_cffi_native_object".to_owned(),
        abi_family: if plan.cpu_target.object_format.is_empty() {
            "host-native".to_owned()
        } else {
            plan.cpu_target.object_format.clone()
        },
        wrapper_policy: if has_c_world_policy {
            plan.hetero_calculate.c_world_policy.clone()
        } else {
            "host-toolchain-wrapper".to_owned()
        },
        required: has_native_object || needs_host_finalize,
    }]
}

pub(crate) fn loader_blockers(
    external_imports: &[NsldContainerExternalImport],
    container_blockers: &[String],
) -> Vec<String> {
    let mut blockers = container_blockers.to_vec();
    blockers.extend(
        external_imports
            .iter()
            .filter(|external_import| external_import.required)
            .map(|external_import| {
                format!(
                    "external-import:{}:{}",
                    external_import.import_kind, external_import.import_name
                )
            }),
    );
    blockers
}

pub(crate) fn loader_symbols(
    loader_entry_kind: &str,
    loader_entry_symbol: &str,
    loader_entry_section_id: &str,
    sections: &[NsldContainerSectionEntry],
) -> Vec<NsldContainerLoaderSymbol> {
    sections
        .iter()
        .find(|section| section.section_id == loader_entry_section_id)
        .map(|section| {
            vec![NsldContainerLoaderSymbol {
                symbol_id: "sym0000.loader-entry".to_owned(),
                symbol_kind: loader_entry_kind.to_owned(),
                symbol_name: loader_entry_symbol.to_owned(),
                lifecycle_hook: "on_lifecycle_bootstrap".to_owned(),
                section_id: section.section_id.clone(),
                offset: section.offset,
                size_bytes: section.size_bytes,
                payload_hash: section.payload_hash.clone(),
            }]
        })
        .unwrap_or_default()
}

pub(crate) fn hetero_loader_symbols(
    nodes: &[nuisc::linker::LinkPlanHeteroNode],
    sections: &[NsldContainerSectionEntry],
    start_index: usize,
) -> Vec<NsldContainerLoaderSymbol> {
    let mut symbols = Vec::new();
    for (node_index, node) in nodes.iter().enumerate() {
        if let Some(section) = sections
            .iter()
            .find(|section| section.source_path == node.link_input)
            .or_else(|| {
                sections
                    .iter()
                    .filter(|section| is_lowering_sidecar_section(&section.section_kind))
                    .nth(node_index)
            })
        {
            let index = start_index + symbols.len();
            symbols.push(NsldContainerLoaderSymbol {
                symbol_id: format!(
                    "sym{index:04}.hetero-node.{}.{}",
                    node.domain_family, node.package_id
                ),
                symbol_kind: "hetero-node-dispatch".to_owned(),
                symbol_name: node.timestamp.clone(),
                lifecycle_hook: node.lifecycle_hook.clone(),
                section_id: section.section_id.clone(),
                offset: section.offset,
                size_bytes: section.size_bytes,
                payload_hash: section.payload_hash.clone(),
            });
        }
    }
    symbols
}

fn is_lowering_sidecar_section(section_kind: &str) -> bool {
    matches!(
        section_kind,
        "lowering-sidecar-input"
            | "shader-lowering-sidecar-input"
            | "kernel-lowering-sidecar-input"
    )
}

pub(crate) fn native_object_loader_symbols(
    sections: &[NsldContainerSectionEntry],
    start_index: usize,
) -> Vec<NsldContainerLoaderSymbol> {
    sections
        .iter()
        .filter(|section| section.section_kind == "native-object-output")
        .enumerate()
        .map(|(native_index, section)| {
            let index = start_index + native_index;
            NsldContainerLoaderSymbol {
                symbol_id: format!("sym{index:04}.native-object-output"),
                symbol_kind: "native-object-output".to_owned(),
                symbol_name: "__nuis_native_object".to_owned(),
                lifecycle_hook: "on_cffi_native_object".to_owned(),
                section_id: section.section_id.clone(),
                offset: section.offset,
                size_bytes: section.size_bytes,
                payload_hash: section.payload_hash.clone(),
            }
        })
        .collect()
}

pub(crate) fn relocations(
    loader_symbols: &[NsldContainerLoaderSymbol],
) -> Vec<NsldContainerRelocationEntry> {
    loader_symbols
        .iter()
        .enumerate()
        .map(|(index, symbol)| NsldContainerRelocationEntry {
            relocation_id: format!("rel{index:04}.{}", relocation_id_suffix(symbol)),
            relocation_kind: relocation_kind_for_symbol(symbol).to_owned(),
            source_section_id: symbol.section_id.clone(),
            source_offset: symbol.offset,
            target_symbol_id: symbol.symbol_id.clone(),
            addend: 0,
        })
        .collect()
}

fn relocation_id_suffix(symbol: &NsldContainerLoaderSymbol) -> &'static str {
    match symbol.symbol_kind.as_str() {
        "hetero-node-dispatch" => "hetero-node",
        "native-object-output" => "native-object",
        _ => "lifecycle-entry",
    }
}

fn relocation_kind_for_symbol(symbol: &NsldContainerLoaderSymbol) -> &'static str {
    match symbol.symbol_kind.as_str() {
        "hetero-node-dispatch" => "hetero-node-binding",
        "native-object-output" => "native-object-binding",
        _ => "lifecycle-entry-binding",
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn file_hash(
    container_plan: &NsldContainerPlanReport,
    sections: &[NsldContainerSectionEntry],
    loader_entry_kind: &str,
    loader_entry_symbol: &str,
    loader_entry_section_id: &str,
    loader_symbols: &[NsldContainerLoaderSymbol],
    relocations: &[NsldContainerRelocationEntry],
    compatibility_domains: &[NsldContainerCompatibilityDomain],
    external_imports: &[NsldContainerExternalImport],
    loader_readiness: &str,
    loader_blockers: &[String],
    payload_size_bytes: usize,
    payload_hash: &str,
    hash_bytes: fn(&[u8]) -> String,
) -> String {
    let mut material = String::new();
    material.push_str(&container_plan.container_magic);
    material.push('\t');
    material.push_str(&container_plan.container_version.to_string());
    material.push('\t');
    material.push_str(&container_plan.container_layout_hash);
    material.push('\t');
    material.push_str(loader_readiness);
    material.push('\t');
    material.push_str(loader_entry_kind);
    material.push('\t');
    material.push_str(loader_entry_symbol);
    material.push('\t');
    material.push_str(loader_entry_section_id);
    material.push('\t');
    material.push_str(&payload_size_bytes.to_string());
    material.push('\t');
    material.push_str(payload_hash);
    material.push('\n');
    for section in sections {
        material.push_str(&section.order_index.to_string());
        material.push('\t');
        material.push_str(&section.section_id);
        material.push('\t');
        material.push_str(&section.section_kind);
        material.push('\t');
        material.push_str(&section.source_hash);
        material.push('\t');
        material.push_str(&section.payload_hash);
        material.push('\t');
        material.push_str(&section.source_path);
        material.push('\t');
        material.push_str(&section.offset.to_string());
        material.push('\t');
        material.push_str(&section.size_bytes.to_string());
        material.push('\n');
    }
    for symbol in loader_symbols {
        material.push_str("loader_symbol\t");
        material.push_str(&symbol.symbol_id);
        material.push('\t');
        material.push_str(&symbol.symbol_kind);
        material.push('\t');
        material.push_str(&symbol.symbol_name);
        material.push('\t');
        material.push_str(&symbol.lifecycle_hook);
        material.push('\t');
        material.push_str(&symbol.section_id);
        material.push('\t');
        material.push_str(&symbol.offset.to_string());
        material.push('\t');
        material.push_str(&symbol.size_bytes.to_string());
        material.push('\t');
        material.push_str(&symbol.payload_hash);
        material.push('\n');
    }
    for relocation in relocations {
        material.push_str("relocation\t");
        material.push_str(&relocation.relocation_id);
        material.push('\t');
        material.push_str(&relocation.relocation_kind);
        material.push('\t');
        material.push_str(&relocation.source_section_id);
        material.push('\t');
        material.push_str(&relocation.source_offset.to_string());
        material.push('\t');
        material.push_str(&relocation.target_symbol_id);
        material.push('\t');
        material.push_str(&relocation.addend.to_string());
        material.push('\n');
    }
    for domain in compatibility_domains {
        material.push_str("compatibility_domain\t");
        material.push_str(&domain.domain_id);
        material.push('\t');
        material.push_str(&domain.domain_kind);
        material.push('\t');
        material.push_str(&domain.paradigm);
        material.push('\t');
        material.push_str(&domain.lifecycle_hook);
        material.push('\t');
        material.push_str(&domain.abi_family);
        material.push('\t');
        material.push_str(&domain.wrapper_policy);
        material.push('\t');
        material.push_str(if domain.required {
            "required"
        } else {
            "optional"
        });
        material.push('\n');
    }
    for external_import in external_imports {
        material.push_str("external_import\t");
        material.push_str(&external_import.import_id);
        material.push('\t');
        material.push_str(&external_import.import_kind);
        material.push('\t');
        material.push_str(&external_import.import_name);
        material.push('\t');
        material.push_str(&external_import.provider);
        material.push('\t');
        material.push_str(if external_import.required {
            "required"
        } else {
            "optional"
        });
        material.push('\n');
    }
    for blocker in loader_blockers {
        material.push_str("loader_blocker\t");
        material.push_str(blocker);
        material.push('\n');
    }
    for blocker in &container_plan.blockers {
        material.push_str("blocker\t");
        material.push_str(blocker);
        material.push('\n');
    }
    hash_bytes(material.as_bytes())
}

pub(crate) fn payload_range_issues(
    report: &NsldContainerReport,
    payload: &[u8],
    hash_bytes: fn(&[u8]) -> String,
) -> Vec<String> {
    let mut issues = Vec::new();
    for section in &report.sections {
        let end = section.offset.saturating_add(section.size_bytes);
        if end > payload.len() {
            issues.push(format!(
                "section_range_out_of_bounds: {} offset={} size={} payload_size={}",
                section.section_id,
                section.offset,
                section.size_bytes,
                payload.len()
            ));
            continue;
        }
        let actual_hash = hash_bytes(&payload[section.offset..end]);
        if actual_hash != section.payload_hash {
            issues.push(format!(
                "section_payload_hash mismatch: {} expected {}, found {}",
                section.section_id, section.payload_hash, actual_hash
            ));
        }
    }
    issues
}
