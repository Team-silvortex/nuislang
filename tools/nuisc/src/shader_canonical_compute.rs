#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalU32Compute {
    pub(crate) operation: CanonicalU32Operation,
    pub(crate) entry: String,
    pub(crate) local_size: [u32; 3],
    pub(crate) descriptor_set: u32,
    pub(crate) input_binding: u32,
    pub(crate) aux_input_binding: Option<u32>,
    pub(crate) output_binding: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonicalU32Operation {
    CopyU32,
    AddU32,
    SubU32,
    MulU32,
    XorU32,
    AddPairU32,
}

impl CanonicalU32Operation {
    pub(crate) fn input_count(self) -> usize {
        match self {
            Self::AddPairU32 => 2,
            Self::CopyU32 | Self::AddU32 | Self::SubU32 | Self::MulU32 | Self::XorU32 => 1,
        }
    }
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
    input_count: usize,
}

const U32_OPERATION_PATTERNS: &[U32OperationPattern] = &[
    U32OperationPattern {
        operation: CanonicalU32Operation::CopyU32,
        name: "copy-u32",
        body_operator: None,
        input_count: 1,
    },
    U32OperationPattern {
        operation: CanonicalU32Operation::AddU32,
        name: "add-u32",
        body_operator: Some("+"),
        input_count: 1,
    },
    U32OperationPattern {
        operation: CanonicalU32Operation::SubU32,
        name: "sub-u32",
        body_operator: Some("-"),
        input_count: 1,
    },
    U32OperationPattern {
        operation: CanonicalU32Operation::MulU32,
        name: "mul-u32",
        body_operator: Some("*"),
        input_count: 1,
    },
    U32OperationPattern {
        operation: CanonicalU32Operation::XorU32,
        name: "xor-u32",
        body_operator: Some("^"),
        input_count: 1,
    },
    U32OperationPattern {
        operation: CanonicalU32Operation::AddPairU32,
        name: "add-pair-u32",
        body_operator: Some("+"),
        input_count: 2,
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
    let mut inputs = storage_bindings
        .iter()
        .filter(|binding| binding.access == StorageAccess::Read)
        .collect::<Vec<_>>();
    inputs.sort_by_key(|binding| binding.binding);
    if !(1..=2).contains(&inputs.len()) {
        return Err(
            "canonical inline WGSL u32 compute requires one or two read-only storage inputs"
                .to_owned(),
        );
    }
    let outputs = storage_bindings
        .iter()
        .filter(|binding| binding.access == StorageAccess::Write)
        .collect::<Vec<_>>();
    let [output] = outputs.as_slice() else {
        return Err(
            "canonical inline WGSL u32 compute requires exactly one writable storage output"
                .to_owned(),
        );
    };
    if inputs.iter().any(|input| input.group != output.group) {
        return Err(
            "canonical inline WGSL u32 compute requires input/output in one descriptor set"
                .to_owned(),
        );
    }
    if inputs.iter().any(|input| input.binding == output.binding)
        || inputs
            .windows(2)
            .any(|pair| pair[0].binding == pair[1].binding)
    {
        return Err("canonical inline WGSL u32 compute bindings must be distinct".to_owned());
    }
    let input_names = inputs
        .iter()
        .map(|binding| binding.name)
        .collect::<Vec<_>>();
    let operation =
        detect_canonical_body_operation(source, expected_entry, &input_names, output.name)?;
    if operation.input_count() != inputs.len() {
        return Err(format!(
            "canonical inline WGSL u32 operation expects {} input binding(s), found {}",
            operation.input_count(),
            inputs.len()
        ));
    }
    Ok(CanonicalU32Compute {
        operation,
        entry: expected_entry.to_owned(),
        local_size,
        descriptor_set: output.group,
        input_binding: inputs[0].binding,
        aux_input_binding: inputs.get(1).map(|input| input.binding),
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
    input_names: &[&str],
    output_name: &str,
) -> Result<CanonicalU32Operation, String> {
    let compact = source
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    let entry_marker = format!("fn{expected_entry}(");
    if !compact.contains(&entry_marker) {
        return Err(format!(
            "canonical inline WGSL u32 body must contain entry `{expected_entry}`"
        ));
    }
    let [input_name] = input_names else {
        return detect_two_input_body_operation(&compact, input_names, output_name);
    };
    let copy_with_idx = format!("{output_name}[idx]={input_name}[idx];");
    let copy_with_gid = format!("{output_name}[gid.x]={input_name}[gid.x];");
    if compact.contains(&copy_with_idx) || compact.contains(&copy_with_gid) {
        return Ok(CanonicalU32Operation::CopyU32);
    }
    for pattern in U32_OPERATION_PATTERNS
        .iter()
        .filter(|pattern| pattern.body_operator.is_some() && pattern.input_count == 1)
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
        "canonical inline WGSL u32 body must copy input[idx], apply a registered unary u32 operation, or add two registered input buffers"
            .to_owned(),
    )
}

fn detect_two_input_body_operation(
    compact: &str,
    input_names: &[&str],
    output_name: &str,
) -> Result<CanonicalU32Operation, String> {
    let [left_name, right_name] = input_names else {
        return Err(
            "canonical inline WGSL u32 compute supports at most two read-only storage inputs"
                .to_owned(),
        );
    };
    for index in ["idx", "gid.x"] {
        let left_right =
            format!("{output_name}[{index}]={left_name}[{index}]+{right_name}[{index}];");
        let right_left =
            format!("{output_name}[{index}]={right_name}[{index}]+{left_name}[{index}];");
        if compact.contains(&left_right) || compact.contains(&right_left) {
            return Ok(CanonicalU32Operation::AddPairU32);
        }
    }
    Err(
        "canonical inline WGSL two-input u32 body must add both input buffers into output[idx]"
            .to_owned(),
    )
}
