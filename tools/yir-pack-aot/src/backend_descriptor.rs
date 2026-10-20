pub(super) fn render_shader_descriptor_lines(
    variant: &yir_lower_contract::ShaderBackendVariant,
    prefix: &str,
) -> String {
    render_backend_descriptor_lines(BackendDescriptorLineSpec {
        prefix,
        backend: variant.backend,
        backend_family: variant.backend_family,
        target_os: variant.target_os,
        target_device: variant.target_device,
        ir_format: variant.ir_format,
        dispatch_abi: variant.dispatch_abi,
        kind: variant.kind,
        priority: variant.priority,
        status: variant.status,
        verification: variant.verification,
        entry: &variant.entry,
        artifact: &variant.artifact,
    })
}

pub(super) fn render_kernel_descriptor_lines(
    variant: &yir_lower_contract::KernelBackendVariant,
    prefix: &str,
) -> String {
    render_backend_descriptor_lines(BackendDescriptorLineSpec {
        prefix,
        backend: variant.backend,
        backend_family: variant.backend_family,
        target_os: variant.target_os,
        target_device: variant.target_device,
        ir_format: variant.ir_format,
        dispatch_abi: variant.dispatch_abi,
        kind: variant.kind,
        priority: variant.priority,
        status: variant.status,
        verification: variant.verification,
        entry: &variant.entry,
        artifact: &variant.artifact,
    })
}

pub(super) struct BackendDescriptorLineSpec<'a> {
    pub(super) prefix: &'a str,
    pub(super) backend: &'a str,
    pub(super) backend_family: &'a str,
    pub(super) target_os: &'a str,
    pub(super) target_device: &'a str,
    pub(super) ir_format: &'a str,
    pub(super) dispatch_abi: &'a str,
    pub(super) kind: &'a str,
    pub(super) priority: usize,
    pub(super) status: &'a str,
    pub(super) verification: &'a str,
    pub(super) entry: &'a str,
    pub(super) artifact: &'a str,
}

fn render_backend_descriptor_lines(spec: BackendDescriptorLineSpec<'_>) -> String {
    let prefix = spec.prefix;
    [
        format!("{prefix}backend={}", spec.backend),
        format!("{prefix}backend_family={}", spec.backend_family),
        format!("{prefix}target_os={}", spec.target_os),
        format!("{prefix}target_device={}", spec.target_device),
        format!("{prefix}ir_format={}", spec.ir_format),
        format!("{prefix}dispatch_abi={}", spec.dispatch_abi),
        format!("{prefix}kind={}", spec.kind),
        format!("{prefix}priority={}", spec.priority),
        format!("{prefix}status={}", spec.status),
        format!("{prefix}verification={}", spec.verification),
        format!("{prefix}entry={}", spec.entry),
        format!("{prefix}artifact={}", spec.artifact),
        prefix.to_string(),
    ]
    .join("\n")
}

pub(super) fn render_summary_block(summary: &str, prefix: &str) -> String {
    let mut lines = vec![format!("{prefix}contract:")];
    lines.extend(summary.lines().map(|line| format!("{prefix}{line}")));
    lines.push(String::new());
    lines.join("\n")
}

pub(super) fn render_wgsl_comment_block(
    wgsl_entry: Option<&str>,
    wgsl_source: Option<&str>,
    prefix: &str,
) -> String {
    let Some(source) = wgsl_source else {
        return format!("{prefix}original_wgsl=absent");
    };
    let mut lines = Vec::new();
    if let Some(entry) = wgsl_entry {
        lines.push(format!("{prefix}original_wgsl_entry={entry}"));
    } else {
        lines.push(format!("{prefix}original_wgsl_entry=unknown"));
    }
    lines.push(format!("{prefix}original_wgsl_begin"));
    for line in source.lines() {
        lines.push(format!("{prefix}{line}"));
    }
    lines.push(format!("{prefix}original_wgsl_end"));
    lines.join("\n")
}
