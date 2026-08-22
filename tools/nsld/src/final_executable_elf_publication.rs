use crate::{
    final_executable_elf_artifact::elf_amd64_artifact_private_product,
    final_executable_finalizer_registry::ExecutableFinalizerPrivateImagePublicationContext,
    final_executable_private_image_publication::{
        publish_verified_private_image, PrivateImageAdmissionEvidence, PrivateImagePublicationInput,
    },
    final_executable_registered_loader_probe_admission::verify_registered_loader_probe_admission_receipt,
    reports::NsldPrivateImagePublicationReport,
};

pub(crate) const ELF_AMD64_PRIVATE_IMAGE_PUBLICATION_CAPABILITY: &str =
    "nsld.finalizer.elf.amd64.private-image-publication-v1";

pub(crate) fn publish_elf_amd64_private_image(
    context: &ExecutableFinalizerPrivateImagePublicationContext<'_>,
) -> Result<NsldPrivateImagePublicationReport, String> {
    let admission = verify_registered_loader_probe_admission_receipt(context.plan);
    let product = elf_amd64_artifact_private_product(context.plan)?;
    let source_image_hash = product.shell_image_serialization.shell_image_hash.clone();
    let validation_evidence_hash = &product.shell_image_validation.validation_ledger_hash;
    let mut provider_issues = Vec::new();

    if context.capability_id != ELF_AMD64_PRIVATE_IMAGE_PUBLICATION_CAPABILITY {
        provider_issues.push("private-image-publication-capability-identity-mismatch".to_owned());
    }
    if context.target_key != "x86_64-linux-elf" {
        provider_issues.push("private-image-publication-target-identity-mismatch".to_owned());
    }
    if admission.provider_id.as_deref() != Some(context.provider_id)
        || admission.target_key.as_deref() != Some(context.target_key)
    {
        provider_issues.push("private-image-publication-admission-provider-drift".to_owned());
    }
    if admission.current_image_identity_hash.as_deref() != Some(source_image_hash.as_str()) {
        provider_issues.push("private-image-publication-source-image-drift".to_owned());
    }
    if admission.current_validation_evidence_hash.as_deref()
        != Some(validation_evidence_hash.as_str())
    {
        provider_issues.push("private-image-publication-validation-evidence-drift".to_owned());
    }

    publish_verified_private_image(PrivateImagePublicationInput {
        context,
        admission: PrivateImageAdmissionEvidence {
            contract: admission.contract,
            status: admission.status,
            receipt_file: admission.receipt_file,
            receipt_present: admission.receipt_present,
            valid: admission.valid,
            receipt_hash_sha256: admission.receipt_hash_sha256.as_deref(),
            verification_ledger_sha256: &admission.verification_ledger_sha256,
            issues: &admission.issues,
        },
        source_image: &product.private_shell_image,
        source_image_hash: &source_image_hash,
        provider_issues,
    })
}
