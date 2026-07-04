use super::{object_macho_image::encode_mach_o_arm64_image, reports::NsldObjectFileLayoutReport};
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
    let Some(entry) = object_image_backend_entry(&file_layout.backend_kind) else {
        return NsldObjectImageEncodeResult {
            image: None,
            blockers: vec![format!(
                "object-image-backend:{}:unsupported",
                file_layout.backend_kind
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
    let blockers = image
        .is_none()
        .then(|| format!("object-image-backend:{}:encode-failed", entry.backend_kind))
        .into_iter()
        .collect();

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

#[cfg(test)]
mod tests {
    use super::{object_image_backend_family, object_image_backend_status};

    #[test]
    fn image_backend_registry_keeps_macho_as_one_registered_backend() {
        assert_eq!(object_image_backend_status("mach-o-arm64"), "ready");
        assert_eq!(object_image_backend_family("mach-o-arm64"), "mach-o");
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
    }
}
