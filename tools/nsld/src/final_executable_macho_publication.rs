use crate::{
    final_executable_finalizer_registry::ExecutableFinalizerPrivateImagePublicationContext,
    final_executable_macho_admission::verify_macho_arm64_publication_admission_receipt,
    final_executable_macho_artifact::macho_artifact_private_shell_product,
    final_executable_private_image_publication::{
        publish_verified_private_image, PrivateImageAdmissionEvidence, PrivateImagePublicationInput,
    },
    reports::NsldPrivateImagePublicationReport,
};

pub(crate) const MACHO_ARM64_PRIVATE_IMAGE_PUBLICATION_CAPABILITY: &str =
    "nsld.finalizer.mach-o.arm64.private-image-publication-v1";

pub(crate) fn publish_macho_arm64_private_image(
    context: &ExecutableFinalizerPrivateImagePublicationContext<'_>,
) -> Result<NsldPrivateImagePublicationReport, String> {
    let product = macho_artifact_private_shell_product(context.plan)?;
    let admission = verify_macho_arm64_publication_admission_receipt(context.plan, &product);
    let source_image_hash = product
        .summary
        .shell_image_serialization
        .shell_image_hash
        .clone();
    let mut provider_issues = Vec::new();

    if context.capability_id != MACHO_ARM64_PRIVATE_IMAGE_PUBLICATION_CAPABILITY {
        provider_issues.push("private-image-publication-capability-identity-mismatch".to_owned());
    }
    if context.target_key != "aarch64-macos-mach-o" {
        provider_issues.push("private-image-publication-target-identity-mismatch".to_owned());
    }

    publish_verified_private_image(PrivateImagePublicationInput {
        context,
        admission: PrivateImageAdmissionEvidence {
            contract: &admission.contract,
            status: &admission.status,
            receipt_file: &admission.receipt_file,
            receipt_present: admission.receipt_present,
            valid: admission.valid,
            receipt_hash_sha256: admission.receipt_hash_sha256.as_deref(),
            verification_ledger_sha256: &admission.verification_ledger_sha256,
            issues: &admission.issues,
        },
        source_image: &product.bytes,
        source_image_hash: &source_image_hash,
        provider_issues,
    })
}
