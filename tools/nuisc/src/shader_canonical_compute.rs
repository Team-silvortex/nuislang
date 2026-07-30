#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalU32Compute {
    pub(crate) operation: CanonicalU32Operation,
    pub(crate) entry: String,
    pub(crate) local_size: [u32; 3],
    pub(crate) descriptor_set: u32,
    pub(crate) input_binding: u32,
    pub(crate) output_binding: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonicalU32Operation {
    CopyU32,
    AddU32,
    SubU32,
    MulU32,
    XorU32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StorageAccess {
    Read,
    Write,
}

struct StorageBinding<'a> {
    group: u32,
    binding: u32,
    name: &'a str,
    access: StorageAccess,
}

struct U32OperationPattern {
    operation: CanonicalU32Operation,
    name: &'static str,
    body_operator: Option<&'static str>,
}

const U32_OPERATION_PATTERNS: &[U32OperationPattern] = &[
    U32OperationPattern {
        operation: CanonicalU32Operation::CopyU32,
        name: "copy-u32",
        body_operator: None,
    },
    U32OperationPattern {
        operation: CanonicalU32Operation::AddU32,
        name: "add-u32",
        body_operator: Some("+"),
    },
    U32OperationPattern {
        operation: CanonicalU32Operation::SubU32,
        name: "sub-u32",
        body_operator: Some("-"),
    },
    U32OperationPattern {
        operation: CanonicalU32Operation::MulU32,
        name: "mul-u32",
        body_operator: Some("*"),
    },
    U32OperationPattern {
        operation: CanonicalU32Operation::XorU32,
        name: "xor-u32",
        body_operator: Some("^"),
    },
];

pub(crate) fn parse_u32_operation(value: &str) -> Result<CanonicalU32Operation, String> {
    U32_OPERATION_PATTERNS
        .iter()
        .find(|pattern| pattern.name == value)
        .map(|pattern| pattern.operation)
        .ok_or_else(|| format!("unsupported Nuis u32 compute operation `{value}`"))
}

pub(crate) fn parse_canonical_inline_wgsl_u32_compute(
    source: &str,
    expected_entry: &str,
) -> Result<CanonicalU32Compute, String> {
    let summary = crate::shader_source::summarize_inline_wgsl_source(source)?;
    if summary.schema != "nuis-inline-wgsl-summary-v1" {
        return Err(format!(
            "canonical inline WGSL summary schema `{}` is unsupported",
            summary.schema
        ));
    }
    let stage = summary
        .stages
        .iter()
        .find(|stage| stage.stage == "compute" && stage.entry == expected_entry)
        .ok_or_else(|| {
            format!("canonical inline WGSL must contain @compute entry `{expected_entry}`")
        })?;
    let local_size = parse_wgsl_workgroup_size(stage.workgroup_size.as_deref())?;
    let storage_bindings = summary
        .bindings
        .iter()
        .filter(|binding| binding.kind == "storage")
        .map(storage_binding)
        .collect::<Result<Vec<_>, _>>()?;
    let input = storage_bindings
        .iter()
        .find(|binding| binding.access == StorageAccess::Read)
        .ok_or_else(|| {
            "canonical inline WGSL u32 compute requires a read-only storage input".to_owned()
        })?;
    let output = storage_bindings
        .iter()
        .find(|binding| binding.access == StorageAccess::Write)
        .ok_or_else(|| {
            "canonical inline WGSL u32 compute requires a writable storage output".to_owned()
        })?;
    if input.group != output.group {
        return Err(
            "canonical inline WGSL u32 compute requires input/output in one descriptor set"
                .to_owned(),
        );
    }
    Ok(CanonicalU32Compute {
        operation: detect_canonical_body_operation(
            source,
            expected_entry,
            input.name,
            output.name,
        )?,
        entry: expected_entry.to_owned(),
        local_size,
        descriptor_set: input.group,
        input_binding: input.binding,
        output_binding: output.binding,
    })
}

fn storage_binding(
    binding: &crate::shader_source::InlineWgslBindingSummary,
) -> Result<StorageBinding<'_>, String> {
    let address_space = binding.address_space.as_deref().unwrap_or_default();
    if !binding.ty.starts_with("array<u32") {
        return Err(format!(
            "canonical inline WGSL storage binding `{}` must be array<u32>",
            binding.name
        ));
    }
    let access = if address_space.contains("read_write") || address_space == "storage" {
        StorageAccess::Write
    } else if address_space.contains("read") {
        StorageAccess::Read
    } else {
        return Err(format!(
            "canonical inline WGSL storage binding `{}` must declare read or read_write access",
            binding.name
        ));
    };
    Ok(StorageBinding {
        group: binding.group,
        binding: binding.binding,
        name: &binding.name,
        access,
    })
}

fn parse_wgsl_workgroup_size(value: Option<&str>) -> Result<[u32; 3], String> {
    let Some(value) = value else {
        return Err("canonical inline WGSL compute stage must declare @workgroup_size".to_owned());
    };
    let values = value
        .split(',')
        .map(str::trim)
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "canonical inline WGSL workgroup_size must use u32 dimensions".to_owned())?;
    let [x, y, z] = values.as_slice() else {
        return Err(
            "canonical inline WGSL workgroup_size must contain three dimensions".to_owned(),
        );
    };
    if [x, y, z].into_iter().any(|value| *value == 0)
        || u64::from(*x) * u64::from(*y) * u64::from(*z) > 1024
    {
        return Err(
            "canonical inline WGSL workgroup_size exceeds the portable compute limit".to_owned(),
        );
    }
    Ok([*x, *y, *z])
}

fn detect_canonical_body_operation(
    source: &str,
    expected_entry: &str,
    input_name: &str,
    output_name: &str,
) -> Result<CanonicalU32Operation, String> {
    let compact = source
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    let entry_marker = format!("fn{expected_entry}(");
    let copy_with_idx = format!("{output_name}[idx]={input_name}[idx];");
    let copy_with_gid = format!("{output_name}[gid.x]={input_name}[gid.x];");
    if !compact.contains(&entry_marker) {
        return Err(format!(
            "canonical inline WGSL u32 body must contain entry `{expected_entry}`"
        ));
    }
    if compact.contains(&copy_with_idx) || compact.contains(&copy_with_gid) {
        return Ok(CanonicalU32Operation::CopyU32);
    }
    for pattern in U32_OPERATION_PATTERNS
        .iter()
        .filter(|pattern| pattern.body_operator.is_some())
    {
        let operator = pattern.body_operator.expect("filtered binary operation");
        let with_idx = format!("{output_name}[idx]={input_name}[idx]{operator}{input_name}[idx];");
        let with_gid =
            format!("{output_name}[gid.x]={input_name}[gid.x]{operator}{input_name}[gid.x];");
        if compact.contains(&with_idx) || compact.contains(&with_gid) {
            return Ok(pattern.operation);
        }
    }
    Err(
        "canonical inline WGSL u32 body must copy input[idx] or apply a registered u32 operation into output[idx]"
            .to_owned(),
    )
}
