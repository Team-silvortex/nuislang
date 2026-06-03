use std::collections::BTreeSet;

use nuis_semantics::model::{AstStructDef, AstStructField, AstTypeRef};

pub(crate) fn validate_struct_annotations(definition: &AstStructDef) -> Result<(), String> {
    let mut seen_struct_annotations = BTreeSet::new();
    let mut is_packet_struct = false;
    for attribute in &definition.attributes {
        match attribute.name.as_str() {
            "packet" => {
                validate_zero_arg_struct_annotation(definition, "packet", &attribute.args)?;
                is_packet_struct = true;
            }
            other => {
                return Err(format!(
                    "struct `{}` uses unknown annotation `@{other}`",
                    definition.name
                ));
            }
        }
        if !seen_struct_annotations.insert(attribute.name.as_str()) {
            return Err(format!(
                "struct `{}` repeats annotation `@{}`",
                definition.name, attribute.name
            ));
        }
    }

    for field in &definition.fields {
        let mut seen_field_annotations = BTreeSet::new();
        for attribute in &field.attributes {
            match attribute.name.as_str() {
                "packet_field" => {
                    validate_zero_arg_struct_field_annotation(
                        definition,
                        field,
                        "packet_field",
                        &attribute.args,
                    )?;
                    if !is_packet_struct {
                        return Err(format!(
                            "struct `{}` field `{}` annotation `@packet_field` requires parent struct `{}` to also declare `@packet`",
                            definition.name, field.name, definition.name
                        ));
                    }
                }
                "packet_control_field" => {
                    validate_zero_arg_struct_field_annotation(
                        definition,
                        field,
                        "packet_control_field",
                        &attribute.args,
                    )?;
                    if !is_packet_struct {
                        return Err(format!(
                            "struct `{}` field `{}` annotation `@packet_control_field` requires parent struct `{}` to also declare `@packet`",
                            definition.name, field.name, definition.name
                        ));
                    }
                }
                other => {
                    return Err(format!(
                        "struct `{}` field `{}` uses unknown annotation `@{other}`",
                        definition.name, field.name
                    ));
                }
            }
            if !seen_field_annotations.insert(attribute.name.as_str()) {
                return Err(format!(
                    "struct `{}` field `{}` repeats annotation `@{}`",
                    definition.name, field.name, attribute.name
                ));
            }
        }
    }

    if is_packet_struct {
        validate_packet_struct_contract(definition)?;
    }
    Ok(())
}

fn validate_packet_struct_contract(definition: &AstStructDef) -> Result<(), String> {
    if definition.fields.is_empty() {
        return Err(format!(
            "struct `{}` annotation `@packet` requires at least one field",
            definition.name
        ));
    }
    let packet_field_count = definition
        .fields
        .iter()
        .filter(|field| {
            field
                .attributes
                .iter()
                .any(|attribute| attribute.name == "packet_field")
        })
        .count();
    if packet_field_count == 0 {
        return Err(format!(
            "struct `{}` annotation `@packet` requires at least one `@packet_field`",
            definition.name
        ));
    }

    for field in &definition.fields {
        let is_packet_field = field
            .attributes
            .iter()
            .any(|attribute| attribute.name == "packet_field");
        let is_packet_control_field = field
            .attributes
            .iter()
            .any(|attribute| attribute.name == "packet_control_field");
        let field_role = packet_field_contract_role(&field.ty);
        if is_packet_field && is_packet_control_field {
            return Err(format!(
                "struct `{}` field `{}` cannot use both `@packet_field` and `@packet_control_field`",
                definition.name, field.name
            ));
        }
        if is_packet_field && field_role != "payload" {
            return Err(format!(
                "struct `{}` field `{}` annotation `@packet_field` currently only supports payload-role fields (role={})",
                definition.name, field.name, field_role
            ));
        }
        if is_packet_control_field && field_role != "control-plane" {
            return Err(format!(
                "struct `{}` field `{}` annotation `@packet_control_field` currently only supports control-plane-role fields (role={})",
                definition.name, field.name, field_role
            ));
        }
        if field.ty.is_ref {
            return Err(format!(
                "struct `{}` field `{}` is not packet-safe yet: `@packet` currently rejects `ref` fields (role={})",
                definition.name, field.name, field_role
            ));
        }
        if field.ty.is_optional {
            return Err(format!(
                "struct `{}` field `{}` is not packet-safe yet: `@packet` currently rejects optional fields (role={})",
                definition.name, field.name, field_role
            ));
        }
        match field.ty.name.as_str() {
            "Task" => {
                return Err(format!(
                    "struct `{}` field `{}` is not packet-safe yet: `@packet` currently rejects `Task<...>` fields (role={})",
                    definition.name, field.name, field_role
                ));
            }
            "TaskResult" | "DataResult" | "ShaderResult" | "KernelResult" | "NetworkResult" => {
                return Err(format!(
                    "struct `{}` field `{}` is not packet-safe yet: `@packet` currently rejects result-carrier fields like `{}` (role={})",
                    definition.name, field.name, field.ty.name, field_role
                ));
            }
            "Marker" => {
                if is_packet_control_field {
                    continue;
                }
                return Err(format!(
                    "struct `{}` field `{}` is not packet-safe yet: `@packet` currently rejects `Marker<...>` fields (role={})",
                    definition.name, field.name, field_role
                ));
            }
            "HandleTable" => {
                if is_packet_control_field {
                    continue;
                }
                return Err(format!(
                    "struct `{}` field `{}` is not packet-safe yet: `@packet` currently rejects `HandleTable<...>` fields (role={})",
                    definition.name, field.name, field_role
                ));
            }
            _ => {}
        }
    }

    Ok(())
}

fn packet_field_contract_role(ty: &AstTypeRef) -> &'static str {
    match ty.name.as_str() {
        "Marker" | "HandleTable" => "control-plane",
        "TaskResult" | "DataResult" | "ShaderResult" | "KernelResult" | "NetworkResult"
        | "Task" => "async-carrier",
        _ if ty.is_ref || ty.is_optional => "unsupported-shape",
        _ => "payload",
    }
}

fn validate_zero_arg_struct_annotation(
    definition: &AstStructDef,
    annotation: &str,
    args: &[nuis_semantics::model::AstAttributeArg],
) -> Result<(), String> {
    if args.is_empty() {
        return Ok(());
    }
    Err(format!(
        "struct `{}` annotation `@{annotation}` does not take arguments",
        definition.name
    ))
}

fn validate_zero_arg_struct_field_annotation(
    definition: &AstStructDef,
    field: &AstStructField,
    annotation: &str,
    args: &[nuis_semantics::model::AstAttributeArg],
) -> Result<(), String> {
    if args.is_empty() {
        return Ok(());
    }
    Err(format!(
        "struct `{}` field `{}` annotation `@{annotation}` does not take arguments",
        definition.name, field.name
    ))
}
