use std::collections::BTreeMap;

use nuis_semantics::model::{AstAttribute, AstStructDef, AstStructField, AstTypeRef};

use super::{rendering::render_ast_type_ref, LoadedProject, ProjectModule};

fn has_ast_attribute(attributes: &[AstAttribute], expected: &str) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.name == expected)
}

fn classify_packet_field_kind(ty: &AstTypeRef) -> &'static str {
    if ty.is_ref {
        return "ref";
    }
    if ty.is_optional {
        return "optional";
    }
    if matches!(
        ty.name.as_str(),
        "bool" | "i32" | "i64" | "f32" | "f64" | "String" | "Unit"
    ) && ty.generic_args.is_empty()
    {
        return "scalar";
    }
    if matches!(
        ty.name.as_str(),
        "Window" | "WindowMut" | "Pipe" | "Instance" | "Task"
    ) {
        return "container";
    }
    if ty.name == "Thread" {
        return "thread";
    }
    if matches!(ty.name.as_str(), "Mutex" | "MutexGuard") {
        return "sync-resource";
    }
    if matches!(
        ty.name.as_str(),
        "TaskResult" | "DataResult" | "ShaderResult" | "KernelResult" | "NetworkResult"
    ) && ty.generic_args.len() == 1
    {
        return "result";
    }
    if ty.name == "Marker" {
        return "marker";
    }
    if ty.name == "HandleTable" {
        return "handle-table";
    }
    "nominal"
}

fn classify_packet_field_role(kind: &str) -> &'static str {
    match kind {
        "scalar" | "nominal" | "container" => "payload",
        "marker" | "handle-table" => "control-plane",
        "result" | "thread" => "async-carrier",
        "sync-resource" => "sync-resource",
        "ref" | "optional" => "unsupported-shape",
        _ => "payload",
    }
}

fn describe_packet_field_kind_counts(definition: &AstStructDef) -> String {
    let mut counts = BTreeMap::<&'static str, usize>::new();
    for field in &definition.fields {
        *counts
            .entry(classify_packet_field_kind(&field.ty))
            .or_insert(0) += 1;
    }
    counts
        .into_iter()
        .map(|(kind, count)| format!("{kind}={count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn describe_packet_field_role_counts(definition: &AstStructDef) -> String {
    let mut counts = BTreeMap::<&'static str, usize>::new();
    for field in &definition.fields {
        let kind = classify_packet_field_kind(&field.ty);
        *counts.entry(classify_packet_field_role(kind)).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .map(|(role, count)| format!("{role}={count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn classify_packet_field_slot(field: &AstStructField) -> &'static str {
    let has_payload = has_ast_attribute(&field.attributes, "packet_field");
    let has_control = has_ast_attribute(&field.attributes, "packet_control_field");
    match (has_payload, has_control) {
        (true, false) => "payload",
        (false, true) => "control",
        (true, true) => "invalid",
        (false, false) => "none",
    }
}

fn packet_fixed_scalar_width(ty: &AstTypeRef) -> Option<usize> {
    if ty.is_ref || ty.is_optional || !ty.generic_args.is_empty() {
        return None;
    }
    match ty.name.as_str() {
        "bool" => Some(1),
        "i32" | "f32" => Some(4),
        "i64" | "f64" => Some(8),
        _ => None,
    }
}

fn classify_packet_wire_kind(field: &AstStructField) -> &'static str {
    match classify_packet_field_slot(field) {
        "payload" => {
            if let Some(width) = packet_fixed_scalar_width(&field.ty) {
                return match (field.ty.name.as_str(), width) {
                    ("bool", 1) => "bool",
                    ("i32", 4) => "i32",
                    ("i64", 8) => "i64",
                    ("f32", 4) => "f32",
                    ("f64", 8) => "f64",
                    _ => "fixed-scalar",
                };
            }
            match classify_packet_field_kind(&field.ty) {
                "scalar" => "dynamic-scalar",
                "nominal" => "nominal",
                "container" => "container",
                "ref" => "ref",
                "optional" => "optional",
                other => other,
            }
        }
        "control" => "control-plane",
        "invalid" => "invalid",
        "none" => "none",
        _ => "none",
    }
}

fn packet_payload_layout(
    definition: &AstStructDef,
) -> Option<Vec<(usize, &str, &'static str, usize)>> {
    let mut offset = 0usize;
    let mut layout = Vec::new();
    for field in &definition.fields {
        if !has_ast_attribute(&field.attributes, "packet_field") {
            continue;
        }
        let width = packet_fixed_scalar_width(&field.ty)?;
        let wire_kind = classify_packet_wire_kind(field);
        layout.push((offset, field.name.as_str(), wire_kind, width));
        offset += width;
    }
    Some(layout)
}

fn describe_packet_payload_layout(definition: &AstStructDef) -> (String, String, &'static str) {
    let has_control = definition
        .fields
        .iter()
        .any(|field| has_ast_attribute(&field.attributes, "packet_control_field"));
    let Some(layout) = packet_payload_layout(definition) else {
        return (
            "dynamic".to_owned(),
            "intrinsic-only".to_owned(),
            if has_control {
                "dynamic-payload+control"
            } else {
                "dynamic-payload"
            },
        );
    };
    let payload_bytes = layout
        .last()
        .map(|(offset, _, _, width)| offset + width)
        .unwrap_or(0);
    let layout_description = layout
        .into_iter()
        .map(|(offset, name, wire_kind, width)| format!("{name}:{wire_kind}@{offset}+{width}"))
        .collect::<Vec<_>>()
        .join(", ");
    (
        payload_bytes.to_string(),
        layout_description,
        if has_control {
            "fixed-payload+control"
        } else {
            "fixed-payload"
        },
    )
}

fn describe_packet_shape(definition: &AstStructDef) -> &'static str {
    let mut has_scalar = false;
    let mut has_non_scalar = false;
    let mut has_carrier = false;
    for field in &definition.fields {
        match classify_packet_field_kind(&field.ty) {
            "scalar" => has_scalar = true,
            "container" | "result" => {
                has_non_scalar = true;
                has_carrier = true;
            }
            _ => has_non_scalar = true,
        }
    }
    if has_carrier {
        "carrier-mixed"
    } else if has_scalar && !has_non_scalar {
        "scalar-only"
    } else if !has_scalar && has_non_scalar {
        "structured-only"
    } else {
        "mixed"
    }
}

pub(super) fn validate_project_packet_contracts(module: &ProjectModule) -> Result<(), String> {
    for definition in &module.ast.structs {
        let is_packet = definition
            .attributes
            .iter()
            .any(|attribute| attribute.name == "packet");
        if !is_packet {
            continue;
        }
        if definition.fields.is_empty() {
            return Err(format!(
                "project mod `mod {} {}` packet struct `{}` requires at least one field",
                module.ast.domain, module.ast.unit, definition.name
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
                "project mod `mod {} {}` packet struct `{}` requires at least one `@packet_field`",
                module.ast.domain, module.ast.unit, definition.name
            ));
        }
        for field in &definition.fields {
            let field_kind = classify_packet_field_kind(&field.ty);
            let field_role = classify_packet_field_role(field_kind);
            let is_packet_field = field
                .attributes
                .iter()
                .any(|attribute| attribute.name == "packet_field");
            let is_packet_control_field = field
                .attributes
                .iter()
                .any(|attribute| attribute.name == "packet_control_field");
            if is_packet_field && field_role != "payload" {
                return Err(format!(
                    "project mod `mod {} {}` packet struct `{}.{}` annotation `@packet_field` currently only supports payload-role fields (kind={}, role={})",
                    module.ast.domain, module.ast.unit, definition.name, field.name, field_kind, field_role
                ));
            }
            if is_packet_control_field && field_role != "control-plane" {
                return Err(format!(
                    "project mod `mod {} {}` packet struct `{}.{}` annotation `@packet_control_field` currently only supports control-plane-role fields (kind={}, role={})",
                    module.ast.domain, module.ast.unit, definition.name, field.name, field_kind, field_role
                ));
            }
            if is_packet_field && is_packet_control_field {
                return Err(format!(
                    "project mod `mod {} {}` packet struct `{}.{}` cannot use both `@packet_field` and `@packet_control_field`",
                    module.ast.domain, module.ast.unit, definition.name, field.name
                ));
            }
            if field.ty.is_ref {
                return Err(format!(
                    "project mod `mod {} {}` packet struct `{}.{}` is not packet-safe yet: `ref` fields are currently rejected (kind={}, role={})",
                    module.ast.domain, module.ast.unit, definition.name, field.name, field_kind, field_role
                ));
            }
            if field.ty.is_optional {
                return Err(format!(
                    "project mod `mod {} {}` packet struct `{}.{}` is not packet-safe yet: optional fields are currently rejected (kind={}, role={})",
                    module.ast.domain, module.ast.unit, definition.name, field.name, field_kind, field_role
                ));
            }
            match field.ty.name.as_str() {
                "Task" | "Thread" => {
                    return Err(format!(
                        "project mod `mod {} {}` packet struct `{}.{}` is not packet-safe yet: async/concurrency carrier fields like `{}` are currently rejected (kind={}, role={})",
                        module.ast.domain, module.ast.unit, definition.name, field.name, field.ty.name, field_kind, field_role
                    ));
                }
                "TaskResult" | "DataResult" | "ShaderResult" | "KernelResult" | "NetworkResult" => {
                    return Err(format!(
                        "project mod `mod {} {}` packet struct `{}.{}` is not packet-safe yet: result-carrier fields like `{}` are currently rejected (kind={}, role={})",
                        module.ast.domain, module.ast.unit, definition.name, field.name, field.ty.name, field_kind, field_role
                    ));
                }
                "Mutex" | "MutexGuard" => {
                    return Err(format!(
                        "project mod `mod {} {}` packet struct `{}.{}` is not packet-safe yet: synchronization-resource fields like `{}` are currently rejected (kind={}, role={})",
                        module.ast.domain, module.ast.unit, definition.name, field.name, field.ty.name, field_kind, field_role
                    ));
                }
                "Marker" => {
                    if is_packet_control_field {
                        continue;
                    }
                    return Err(format!(
                        "project mod `mod {} {}` packet struct `{}.{}` is not packet-safe yet: `Marker<...>` fields are currently rejected (kind={}, role={})",
                        module.ast.domain, module.ast.unit, definition.name, field.name, field_kind, field_role
                    ));
                }
                "HandleTable" => {
                    if is_packet_control_field {
                        continue;
                    }
                    return Err(format!(
                        "project mod `mod {} {}` packet struct `{}.{}` is not packet-safe yet: `HandleTable<...>` fields are currently rejected (kind={}, role={})",
                        module.ast.domain, module.ast.unit, definition.name, field.name, field_kind, field_role
                    ));
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub fn render_project_packet_index(project: &LoadedProject) -> String {
    let mut out = String::new();
    for project_module in &project.modules {
        let relative = project_module
            .path
            .strip_prefix(&project.root)
            .unwrap_or(project_module.path.as_path())
            .display()
            .to_string();
        for definition in &project_module.ast.structs {
            if !has_ast_attribute(&definition.attributes, "packet") {
                continue;
            }
            let packet_fields = definition
                .fields
                .iter()
                .filter(|field| has_ast_attribute(&field.attributes, "packet_field"))
                .count();
            let packet_control_fields = definition
                .fields
                .iter()
                .filter(|field| has_ast_attribute(&field.attributes, "packet_control_field"))
                .count();
            let field_kinds = describe_packet_field_kind_counts(definition);
            let field_roles = describe_packet_field_role_counts(definition);
            let (payload_bytes, payload_layout, encode_shape) =
                describe_packet_payload_layout(definition);
            out.push_str(&format!(
                "{}\t{}.{}.{}\tfields={}\tpacket_fields={}\tpacket_control_fields={}\tpacket_shape={}\tfield_kinds={}\tfield_roles={}\tpacket_encode_shape={}\tpayload_bytes={}\tpayload_layout={}\n",
                relative,
                project_module.ast.domain,
                project_module.ast.unit,
                definition.name,
                definition.fields.len(),
                packet_fields,
                packet_control_fields,
                describe_packet_shape(definition),
                field_kinds,
                field_roles,
                encode_shape,
                payload_bytes,
                payload_layout
            ));
            for (index, field) in definition.fields.iter().enumerate() {
                let kind = classify_packet_field_kind(&field.ty);
                let fixed_width = packet_fixed_scalar_width(&field.ty)
                    .map(|width| width.to_string())
                    .unwrap_or_else(|| "dynamic".to_owned());
                out.push_str(&format!(
                    "\tindex={}\t{}\t{}\tkind={}\trole={}\tpacket_slot={}\twire_kind={}\tfixed_width={}\tpacket_field={}\tpacket_control_field={}\n",
                    index,
                    field.name,
                    render_ast_type_ref(&field.ty),
                    kind,
                    classify_packet_field_role(kind),
                    classify_packet_field_slot(field),
                    classify_packet_wire_kind(field),
                    fixed_width,
                    has_ast_attribute(&field.attributes, "packet_field"),
                    has_ast_attribute(&field.attributes, "packet_control_field")
                ));
            }
        }
    }
    out
}
