use nuis_semantics::model::AstVisibility;

use crate::{json_field, json_string_array_field};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PublicSurfaceModuleRecord {
    pub(crate) module: String,
    pub(crate) externs: Vec<String>,
    pub(crate) extern_interfaces: Vec<String>,
    pub(crate) consts: Vec<String>,
    pub(crate) type_aliases: Vec<String>,
    pub(crate) functions: Vec<String>,
    pub(crate) structs: Vec<String>,
    pub(crate) traits: Vec<String>,
}

impl PublicSurfaceModuleRecord {
    fn is_empty(&self) -> bool {
        self.externs.is_empty()
            && self.extern_interfaces.is_empty()
            && self.consts.is_empty()
            && self.type_aliases.is_empty()
            && self.functions.is_empty()
            && self.structs.is_empty()
            && self.traits.is_empty()
    }
}

pub(crate) fn public_surface_records(
    project: &nuisc::project::LoadedProject,
) -> Vec<PublicSurfaceModuleRecord> {
    project
        .modules
        .iter()
        .filter_map(|module| {
            let externs = module
                .ast
                .externs
                .iter()
                .filter(|function| matches!(function.visibility, AstVisibility::Public))
                .map(|function| function.name.clone())
                .collect::<Vec<_>>();
            let extern_interfaces = module
                .ast
                .extern_interfaces
                .iter()
                .filter(|interface| matches!(interface.visibility, AstVisibility::Public))
                .map(|interface| interface.name.clone())
                .collect::<Vec<_>>();
            let consts = module
                .ast
                .consts
                .iter()
                .filter(|constant| matches!(constant.visibility, AstVisibility::Public))
                .map(|constant| constant.name.clone())
                .collect::<Vec<_>>();
            let type_aliases = module
                .ast
                .type_aliases
                .iter()
                .filter(|alias| matches!(alias.visibility, AstVisibility::Public))
                .map(|alias| alias.name.clone())
                .collect::<Vec<_>>();
            let functions = module
                .ast
                .functions
                .iter()
                .filter(|function| matches!(function.visibility, AstVisibility::Public))
                .map(|function| function.name.clone())
                .collect::<Vec<_>>();
            let structs = module
                .ast
                .structs
                .iter()
                .filter(|definition| matches!(definition.visibility, AstVisibility::Public))
                .map(|definition| {
                    let public_fields = definition
                        .fields
                        .iter()
                        .filter(|field| matches!(field.visibility, AstVisibility::Public))
                        .count();
                    let hidden_fields = definition.fields.len().saturating_sub(public_fields);
                    if hidden_fields == 0 {
                        format!("{}(fields={public_fields})", definition.name)
                    } else {
                        format!(
                            "{}(fields={public_fields}, hidden={hidden_fields})",
                            definition.name
                        )
                    }
                })
                .collect::<Vec<_>>();
            let traits = module
                .ast
                .traits
                .iter()
                .filter(|definition| matches!(definition.visibility, AstVisibility::Public))
                .map(|definition| definition.name.clone())
                .collect::<Vec<_>>();
            let record = PublicSurfaceModuleRecord {
                module: format!("{}::{}", module.ast.domain, module.ast.unit),
                externs,
                extern_interfaces,
                consts,
                type_aliases,
                functions,
                structs,
                traits,
            };
            if record.is_empty() {
                None
            } else {
                Some(record)
            }
        })
        .collect()
}

pub(crate) fn describe_public_surface(records: &[PublicSurfaceModuleRecord]) -> String {
    let extern_count = records
        .iter()
        .map(|record| record.externs.len())
        .sum::<usize>();
    let extern_interface_count = records
        .iter()
        .map(|record| record.extern_interfaces.len())
        .sum::<usize>();
    let const_count = records
        .iter()
        .map(|record| record.consts.len())
        .sum::<usize>();
    let function_count = records
        .iter()
        .map(|record| record.functions.len())
        .sum::<usize>();
    let alias_count = records
        .iter()
        .map(|record| record.type_aliases.len())
        .sum::<usize>();
    let struct_count = records
        .iter()
        .map(|record| record.structs.len())
        .sum::<usize>();
    let trait_count = records
        .iter()
        .map(|record| record.traits.len())
        .sum::<usize>();
    let module_count = records.len();
    if module_count == 0 {
        return "<none>".to_owned();
    }
    format!(
        "modules={module_count} extern={extern_count} interface={extern_interface_count} const={const_count} type={alias_count} fn={function_count} struct={struct_count} trait={trait_count}"
    )
}

pub(crate) fn describe_public_surface_modules(records: &[PublicSurfaceModuleRecord]) -> String {
    if records.is_empty() {
        return "<none>".to_owned();
    }
    records
        .iter()
        .map(|record| {
            let mut segments = Vec::new();
            if !record.externs.is_empty() {
                segments.push(format!("extern={}", record.externs.join(", ")));
            }
            if !record.extern_interfaces.is_empty() {
                segments.push(format!("interface={}", record.extern_interfaces.join(", ")));
            }
            if !record.consts.is_empty() {
                segments.push(format!("const={}", record.consts.join(", ")));
            }
            if !record.type_aliases.is_empty() {
                segments.push(format!("type={}", record.type_aliases.join(", ")));
            }
            if !record.functions.is_empty() {
                segments.push(format!("fn={}", record.functions.join(", ")));
            }
            if !record.structs.is_empty() {
                segments.push(format!("struct={}", record.structs.join(", ")));
            }
            if !record.traits.is_empty() {
                segments.push(format!("trait={}", record.traits.join(", ")));
            }
            format!("{} [{}]", record.module, segments.join(" | "))
        })
        .collect::<Vec<_>>()
        .join("; ")
}

pub(crate) fn public_surface_json(records: &[PublicSurfaceModuleRecord]) -> Vec<String> {
    records
        .iter()
        .map(|record| {
            format!(
                "{{{},{},{},{},{},{},{},{}}}",
                json_field("module", &record.module),
                json_string_array_field("externs", &record.externs),
                json_string_array_field("extern_interfaces", &record.extern_interfaces),
                json_string_array_field("consts", &record.consts),
                json_string_array_field("type_aliases", &record.type_aliases),
                json_string_array_field("functions", &record.functions),
                json_string_array_field("structs", &record.structs),
                json_string_array_field("traits", &record.traits),
            )
        })
        .collect()
}
