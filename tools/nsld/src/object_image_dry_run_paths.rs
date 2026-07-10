use super::{
    fnv1a64_hex, object_file_layout::nsld_object_file_layout_report,
    object_image_backend::encode_object_image_for_backend,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn encode_object_image_dry_run(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Option<Vec<u8>> {
    let file_layout = nsld_object_file_layout_report(manifest, plan);
    encode_object_image_for_backend(manifest, plan, &file_layout).image
}

pub(crate) fn object_image_dry_run_image_path(plan: &nuisc::linker::LinkPlan) -> PathBuf {
    PathBuf::from(&plan.output_dir).join("nuis.nsld.object-image-dry-run.bin")
}

pub(crate) fn image_file_size_and_hash(
    path: &Path,
) -> Result<(Option<usize>, Option<String>), String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "missing_or_unreadable_object_image_dry_run_bytes `{}`: {error}",
            path.display()
        )
    })?;
    Ok((Some(bytes.len()), Some(fnv1a64_hex(&bytes))))
}
