use super::{
    final_executable_image::{
        parse_final_executable_image_header, FINAL_EXECUTABLE_IMAGE_HEADER_SIZE,
        FINAL_EXECUTABLE_IMAGE_MAGIC_TEXT, FINAL_EXECUTABLE_IMAGE_VERSION,
    },
    fnv1a64_hex,
    reports::NsldFinalExecutableEmitReport,
};
use std::fs;

pub(crate) fn populate_final_output_emit_summary(report: &mut NsldFinalExecutableEmitReport) {
    report.final_output_checked = true;
    let Ok(bytes) = fs::read(&report.output_path) else {
        report.final_output_present = false;
        report.final_output_size_bytes = None;
        report.final_output_hash = None;
        report.final_output_image_header_valid = Some(false);
        report.final_output_runnable_candidate = Some(false);
        return;
    };
    let output_hash = fnv1a64_hex(&bytes);
    let header = parse_final_executable_image_header(&bytes);
    let header_valid = header.as_ref().is_some_and(|header| {
        let payload_end = header.payload_offset.saturating_add(header.payload_span);
        header.magic == FINAL_EXECUTABLE_IMAGE_MAGIC_TEXT
            && header.version == FINAL_EXECUTABLE_IMAGE_VERSION
            && header.header_size == FINAL_EXECUTABLE_IMAGE_HEADER_SIZE
            && header.payload_offset == FINAL_EXECUTABLE_IMAGE_HEADER_SIZE
            && payload_end <= bytes.len()
    });
    report.final_output_present = true;
    report.final_output_size_bytes = Some(bytes.len());
    report.final_output_hash = Some(output_hash.clone());
    report.final_output_image_header_valid = Some(header_valid);
    report.final_output_runnable_candidate = Some(
        report.emitted
            && header_valid
            && report.image_dry_run_hash.as_deref() == Some(output_hash.as_str())
            && report.image_dry_run_size_bytes == Some(bytes.len()),
    );
}
