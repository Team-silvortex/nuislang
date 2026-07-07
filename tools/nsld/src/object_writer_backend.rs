#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectWriterBackend {
    pub(crate) target_id: String,
    pub(crate) backend_kind: String,
    pub(crate) status: String,
    pub(crate) writer_stages: Vec<NsldObjectWriterStage>,
    pub(crate) unsupported_features: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectWriterStage {
    pub(crate) stage_id: String,
    pub(crate) status: String,
    pub(crate) required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldObjectWriterBackendReadiness {
    pub(crate) target_id: String,
    pub(crate) backend_kind: String,
    pub(crate) status: String,
    pub(crate) unsupported_features: Vec<String>,
    pub(crate) blockers: Vec<String>,
    pub(crate) can_emit_object: bool,
}

pub(crate) fn object_writer_backend(
    machine_arch: &str,
    machine_os: &str,
    object_format: &str,
) -> NsldObjectWriterBackend {
    let target_id = object_writer_target_id(machine_arch, machine_os, object_format);
    let backend_kind = object_writer_backend_kind(machine_arch, machine_os, object_format);
    NsldObjectWriterBackend {
        target_id,
        backend_kind: backend_kind.to_owned(),
        status: object_writer_backend_status(backend_kind).to_owned(),
        writer_stages: object_writer_stages(backend_kind),
        unsupported_features: object_writer_unsupported_features(backend_kind),
    }
}

pub(crate) fn object_writer_blockers(
    backend: &NsldObjectWriterBackend,
    upstream_blockers: &[String],
) -> Vec<String> {
    let mut blockers = upstream_blockers.to_vec();
    blockers.extend(
        backend
            .unsupported_features
            .iter()
            .map(|feature| format!("{feature}:not-implemented")),
    );
    blockers.extend(object_writer_stage_blockers(backend));
    blockers
}

pub(crate) fn object_writer_stage_blockers(backend: &NsldObjectWriterBackend) -> Vec<String> {
    backend
        .writer_stages
        .iter()
        .filter(|stage| stage.required && stage.status != "ready")
        .map(|stage| format!("object-writer-stage:{}:{}", stage.stage_id, stage.status))
        .collect()
}

pub(crate) fn object_writer_backend_readiness(
    backend: &NsldObjectWriterBackend,
    object_plan_ready: bool,
    blockers: &[String],
) -> NsldObjectWriterBackendReadiness {
    NsldObjectWriterBackendReadiness {
        target_id: backend.target_id.clone(),
        backend_kind: backend.backend_kind.clone(),
        status: backend.status.clone(),
        unsupported_features: backend.unsupported_features.clone(),
        blockers: blockers.to_vec(),
        can_emit_object: object_plan_ready
            && backend.unsupported_features.is_empty()
            && object_writer_required_stages_ready(backend)
            && blockers.is_empty(),
    }
}

pub(crate) fn object_format_family(object_format: &str) -> &'static str {
    match canonical_object_format(object_format) {
        "mach-o" => "mach-o",
        "elf" => "elf",
        "coff" => "coff",
        _ => "unknown-object-family",
    }
}

fn object_writer_target_id(machine_arch: &str, machine_os: &str, object_format: &str) -> String {
    format!("{machine_arch}-{machine_os}-{object_format}")
}

fn object_writer_backend_kind(
    machine_arch: &str,
    machine_os: &str,
    object_format: &str,
) -> &'static str {
    match (
        canonical_machine_arch(machine_arch),
        canonical_machine_os(machine_os),
        canonical_object_format(object_format),
    ) {
        ("arm64", "macos", "mach-o") => "mach-o-arm64",
        ("aarch64", "linux", "elf") => "elf-aarch64",
        ("x86_64", "linux", "elf") => "elf-amd64",
        ("x86_64", "windows", "coff") => "coff-amd64",
        _ => "unknown-object-writer",
    }
}

fn canonical_machine_arch(machine_arch: &str) -> &str {
    match machine_arch {
        "amd64" => "x86_64",
        "arm64" => "arm64",
        "aarch64" => "aarch64",
        other => other,
    }
}

fn canonical_machine_os(machine_os: &str) -> &str {
    match machine_os {
        "darwin" => "macos",
        other => other,
    }
}

fn canonical_object_format(object_format: &str) -> &str {
    match object_format {
        "pe" | "pe-coff" | "pe/coff" => "coff",
        other => other,
    }
}

fn object_writer_backend_status(backend_kind: &str) -> &'static str {
    match backend_kind {
        "mach-o-arm64" => "ready",
        "unknown-object-writer" => "unsupported-target",
        _ => "recognized-blocked",
    }
}

fn object_writer_stages(backend_kind: &str) -> Vec<NsldObjectWriterStage> {
    object_writer_stage_ids(backend_kind)
        .into_iter()
        .map(|stage_id| NsldObjectWriterStage {
            status: object_writer_stage_status(backend_kind, &stage_id).to_owned(),
            stage_id,
            required: true,
        })
        .collect()
}

fn object_writer_stage_ids(backend_kind: &str) -> Vec<String> {
    match backend_kind {
        "mach-o-arm64" => vec![
            "macho-header".to_owned(),
            "macho-section-table".to_owned(),
            "macho-symbol-table".to_owned(),
            "macho-relocation-table".to_owned(),
            "macho-byte-emission".to_owned(),
        ],
        "elf-aarch64" | "elf-amd64" => vec![
            "elf-header".to_owned(),
            "elf-section-table".to_owned(),
            "elf-symbol-table".to_owned(),
            "elf-relocation-table".to_owned(),
            "elf-byte-emission".to_owned(),
        ],
        "coff-amd64" => vec![
            "coff-header".to_owned(),
            "coff-section-table".to_owned(),
            "coff-symbol-table".to_owned(),
            "coff-relocation-table".to_owned(),
            "coff-byte-emission".to_owned(),
        ],
        _ => vec!["target-selection".to_owned()],
    }
}

fn object_writer_unsupported_features(backend_kind: &str) -> Vec<String> {
    match backend_kind {
        "mach-o-arm64" => Vec::new(),
        "unknown-object-writer" => vec!["object-writer-target".to_owned()],
        _ => vec![
            "object-byte-emitter".to_owned(),
            "native-relocation-applier".to_owned(),
        ],
    }
}

fn object_writer_stage_status(backend_kind: &str, stage_id: &str) -> &'static str {
    match (backend_kind, stage_id) {
        (
            "mach-o-arm64",
            "macho-header"
            | "macho-section-table"
            | "macho-symbol-table"
            | "macho-relocation-table"
            | "macho-byte-emission",
        ) => "ready",
        _ => "not-implemented",
    }
}

fn object_writer_required_stages_ready(backend: &NsldObjectWriterBackend) -> bool {
    backend
        .writer_stages
        .iter()
        .filter(|stage| stage.required)
        .all(|stage| stage.status == "ready")
}

#[cfg(test)]
mod tests {
    use super::{
        object_writer_backend, object_writer_backend_readiness, object_writer_stage_blockers,
    };

    #[test]
    fn recognizes_mach_o_arm64_backend_target() {
        let backend = object_writer_backend("arm64", "macos", "mach-o");

        assert_eq!(backend.target_id, "arm64-macos-mach-o");
        assert_eq!(backend.backend_kind, "mach-o-arm64");
        assert_eq!(backend.status, "ready");
        assert_eq!(
            backend
                .writer_stages
                .iter()
                .map(|stage| stage.stage_id.clone())
                .collect::<Vec<_>>(),
            vec![
                "macho-header".to_owned(),
                "macho-section-table".to_owned(),
                "macho-symbol-table".to_owned(),
                "macho-relocation-table".to_owned(),
                "macho-byte-emission".to_owned()
            ]
        );
        assert_eq!(backend.writer_stages[0].status, "ready");
        assert!(backend
            .writer_stages
            .iter()
            .all(|stage| stage.required && stage.status == "ready"));
        assert!(backend.unsupported_features.is_empty());
    }

    #[test]
    fn mach_o_backend_blockers_forward_upstream_blockers_only() {
        let backend = object_writer_backend("arm64", "macos", "mach-o");
        let blockers =
            super::object_writer_blockers(&backend, &["section-manifest:blocked".to_owned()]);

        assert_eq!(blockers, vec!["section-manifest:blocked".to_owned()]);
    }

    #[test]
    fn stage_blockers_only_include_required_non_ready_stages() {
        let mut backend = object_writer_backend("arm64", "macos", "mach-o");
        backend.writer_stages[0].status = "planned".to_owned();
        backend.writer_stages[1].required = false;
        let blockers = object_writer_stage_blockers(&backend);

        assert!(!blockers.contains(&"object-writer-stage:macho-header:ready".to_owned()));
        assert!(!blockers
            .contains(&"object-writer-stage:macho-section-table:not-implemented".to_owned()));
        assert!(blockers.contains(&"object-writer-stage:macho-header:planned".to_owned()));
    }

    #[test]
    fn readiness_requires_all_required_stages_to_be_ready() {
        let mut backend = object_writer_backend("arm64", "macos", "mach-o");
        let readiness = object_writer_backend_readiness(&backend, true, &[]);

        assert!(readiness.can_emit_object);

        backend.writer_stages[0].status = "planned".to_owned();
        let readiness = object_writer_backend_readiness(&backend, true, &[]);

        assert!(!readiness.can_emit_object);
    }

    #[test]
    fn reports_unknown_backend_targets_explicitly() {
        let backend = object_writer_backend("riscv64", "plan9", "weird-object");

        assert_eq!(backend.target_id, "riscv64-plan9-weird-object");
        assert_eq!(backend.backend_kind, "unknown-object-writer");
        assert_eq!(backend.status, "unsupported-target");
        assert_eq!(backend.writer_stages[0].stage_id, "target-selection");
        assert_eq!(backend.writer_stages[0].status, "not-implemented");
        assert_eq!(
            backend.unsupported_features,
            vec!["object-writer-target".to_owned()]
        );
    }

    #[test]
    fn recognizes_common_arch_and_coff_aliases() {
        let backend = object_writer_backend("amd64", "windows", "pe/coff");

        assert_eq!(backend.target_id, "amd64-windows-pe/coff");
        assert_eq!(backend.backend_kind, "coff-amd64");
        assert_eq!(backend.status, "recognized-blocked");
        assert_eq!(
            super::object_format_family("pe-coff"),
            "coff",
            "PE/COFF aliases should normalize to the COFF object family"
        );
        assert!(backend
            .unsupported_features
            .contains(&"object-byte-emitter".to_owned()));
    }
}
