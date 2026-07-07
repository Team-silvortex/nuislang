use super::{
    object_macho_image::encode_mach_o_arm64_image,
    object_macho_relocations::{
        mach_o_arm64_relocation_lowering_rule_count, mach_o_arm64_relocation_lowering_rules,
        mach_o_arm64_relocation_records, mach_o_arm64_relocation_resolution_issues,
    },
    reports::{
        NsldObjectFileLayoutReport, NsldObjectImageBackendCapabilityDiagnostic,
        NsldObjectImageRelocationRecordDiagnostic, NsldRelocationLoweringRuleDiagnostic,
    },
};
use std::path::Path;

type ObjectImageEncoder =
    fn(&Path, &nuisc::linker::LinkPlan, &NsldObjectFileLayoutReport) -> Option<Vec<u8>>;

#[derive(Clone, Copy)]
struct ObjectImageBackendEntry {
    backend_kind: &'static str,
    object_family: &'static str,
    status: &'static str,
    encoder: Option<ObjectImageEncoder>,
}

pub(crate) struct NsldObjectImageEncodeResult {
    pub(crate) image: Option<Vec<u8>>,
    pub(crate) blockers: Vec<String>,
}

pub(crate) fn encode_object_image_for_backend(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
    file_layout: &NsldObjectFileLayoutReport,
) -> NsldObjectImageEncodeResult {
    let Some(entry) = object_image_backend_entry(&file_layout.writer_backend_kind) else {
        return NsldObjectImageEncodeResult {
            image: None,
            blockers: vec![format!(
                "object-image-backend:{}:unsupported",
                file_layout.writer_backend_kind
            )],
        };
    };
    let Some(encoder) = entry.encoder else {
        return NsldObjectImageEncodeResult {
            image: None,
            blockers: vec![format!(
                "object-image-backend:{}:{}",
                entry.backend_kind, entry.status
            )],
        };
    };
    let image = encoder(manifest, plan, file_layout);
    let mut blockers = image
        .is_none()
        .then(|| format!("object-image-backend:{}:encode-failed", entry.backend_kind))
        .into_iter()
        .collect::<Vec<_>>();
    blockers.extend(object_image_backend_resolution_issues(
        entry.backend_kind,
        manifest,
        plan,
        file_layout,
    ));

    NsldObjectImageEncodeResult { image, blockers }
}

pub(crate) fn object_image_backend_status(backend_kind: &str) -> &'static str {
    object_image_backend_entry(backend_kind)
        .map(|entry| entry.status)
        .unwrap_or("unsupported")
}

pub(crate) fn object_image_backend_family(backend_kind: &str) -> &'static str {
    object_image_backend_entry(backend_kind)
        .map(|entry| entry.object_family)
        .unwrap_or("unknown")
}

pub(crate) fn object_image_backend_capabilities(
    backend_kind: &str,
) -> Vec<NsldObjectImageBackendCapabilityDiagnostic> {
    match backend_kind {
        "mach-o-arm64" => vec![
            backend_capability("file-layout-consumer", "ready"),
            backend_capability("relocation-lowering", "ready"),
            backend_capability("object-image-encoder", "ready"),
        ],
        "elf-aarch64" | "elf-amd64" => vec![
            backend_capability("file-layout-consumer", "ready"),
            backend_capability("relocation-lowering", "not-implemented"),
            backend_capability("object-image-encoder", "not-implemented"),
        ],
        "coff-amd64" => vec![
            backend_capability("file-layout-consumer", "ready"),
            backend_capability("relocation-lowering", "not-implemented"),
            backend_capability("object-image-encoder", "not-implemented"),
        ],
        _ => vec![backend_capability("backend-selection", "unsupported")],
    }
}

pub(crate) fn object_image_backend_relocation_lowering_rule_count(backend_kind: &str) -> usize {
    match backend_kind {
        "mach-o-arm64" => mach_o_arm64_relocation_lowering_rule_count(),
        _ => 0,
    }
}

fn backend_capability(
    capability_id: &str,
    status: &str,
) -> NsldObjectImageBackendCapabilityDiagnostic {
    NsldObjectImageBackendCapabilityDiagnostic {
        capability_id: capability_id.to_owned(),
        status: status.to_owned(),
        required: true,
    }
}

pub(crate) fn object_image_backend_relocation_lowering_rules(
    backend_kind: &str,
) -> Vec<NsldRelocationLoweringRuleDiagnostic> {
    match backend_kind {
        "mach-o-arm64" => mach_o_arm64_relocation_lowering_rules(),
        _ => Vec::new(),
    }
}

pub(crate) fn object_image_backend_relocation_records(
    backend_kind: &str,
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
    file_layout: &NsldObjectFileLayoutReport,
) -> Vec<NsldObjectImageRelocationRecordDiagnostic> {
    match backend_kind {
        "mach-o-arm64" => mach_o_arm64_relocation_records(manifest, plan, file_layout),
        _ => Vec::new(),
    }
}

fn object_image_backend_entry(backend_kind: &str) -> Option<ObjectImageBackendEntry> {
    object_image_backend_entries()
        .into_iter()
        .find(|entry| entry.backend_kind == backend_kind)
}

fn object_image_backend_entries() -> Vec<ObjectImageBackendEntry> {
    vec![
        ObjectImageBackendEntry {
            backend_kind: "mach-o-arm64",
            object_family: "mach-o",
            status: "ready",
            encoder: Some(encode_mach_o_arm64_image),
        },
        ObjectImageBackendEntry {
            backend_kind: "elf-aarch64",
            object_family: "elf",
            status: "not-implemented",
            encoder: None,
        },
        ObjectImageBackendEntry {
            backend_kind: "elf-amd64",
            object_family: "elf",
            status: "not-implemented",
            encoder: None,
        },
        ObjectImageBackendEntry {
            backend_kind: "coff-amd64",
            object_family: "coff",
            status: "not-implemented",
            encoder: None,
        },
    ]
}

fn object_image_backend_resolution_issues(
    backend_kind: &str,
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
    file_layout: &NsldObjectFileLayoutReport,
) -> Vec<String> {
    match backend_kind {
        "mach-o-arm64" => mach_o_arm64_relocation_resolution_issues(manifest, plan, file_layout),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        encode_object_image_for_backend, object_image_backend_capabilities,
        object_image_backend_family, object_image_backend_relocation_lowering_rule_count,
        object_image_backend_relocation_lowering_rules, object_image_backend_status,
    };
    use crate::{
        main_test_support::empty_link_plan, object_file_layout::nsld_object_file_layout_report,
    };
    use std::path::Path;

    #[test]
    fn image_backend_registry_keeps_macho_as_one_registered_backend() {
        assert_eq!(object_image_backend_status("mach-o-arm64"), "ready");
        assert_eq!(object_image_backend_family("mach-o-arm64"), "mach-o");
        assert_eq!(
            object_image_backend_relocation_lowering_rule_count("mach-o-arm64"),
            4
        );
        assert_eq!(
            object_image_backend_relocation_lowering_rules("mach-o-arm64")[0].source_seed_kind,
            "bootstrap-entry-seed"
        );
    }

    #[test]
    fn image_backend_registry_reserves_elf_and_coff_slots() {
        assert_eq!(
            object_image_backend_status("elf-aarch64"),
            "not-implemented"
        );
        assert_eq!(object_image_backend_status("elf-amd64"), "not-implemented");
        assert_eq!(object_image_backend_status("coff-amd64"), "not-implemented");
        assert_eq!(object_image_backend_family("elf-amd64"), "elf");
        assert_eq!(object_image_backend_family("coff-amd64"), "coff");
        assert!(object_image_backend_capabilities("elf-amd64")
            .iter()
            .any(
                |capability| capability.capability_id == "object-image-encoder"
                    && capability.status == "not-implemented"
            ));
    }

    #[test]
    fn mach_o_backend_reports_unresolved_relocation_symbols() {
        let plan = empty_link_plan();
        let manifest = Path::new("manifest.toml");
        let mut file_layout = nsld_object_file_layout_report(manifest, &plan);
        file_layout
            .records
            .retain(|record| record.record_id != "section.sec0000.compiled-artifact");

        let result = encode_object_image_for_backend(manifest, &plan, &file_layout);

        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.contains("unresolved-section-symbol")));
    }
}
