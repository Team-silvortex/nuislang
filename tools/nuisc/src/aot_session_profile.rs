//! Packaging dispatch only. Scalar layouts and resource ownership remain separate.
use crate::{aot_native_session as scalar, aot_resource_session as resource};
use yir_core::YirModule;

pub(crate) fn has_bound_inputs(mode: &str) -> bool {
    mode == "headless-aot-bundle"
        || mode.starts_with(scalar::PACKAGING_PREFIX)
        || mode.starts_with(scalar::POLICY_PACKAGING_PREFIX)
        || mode.starts_with(resource::PACKAGING_PREFIX)
}

pub(crate) fn is_native(mode: &str) -> Result<bool, String> {
    Ok(scalar::registration_id(mode)?.is_some() || resource::registration_id(mode)?.is_some())
}

pub(crate) fn packaging_llvm(module: &YirModule, mode: &str) -> Result<String, String> {
    if resource::registration_id(mode)?.is_some() {
        Ok(resource::emit_for_packaging_mode(module, mode)?.llvm_ir)
    } else {
        Ok(scalar::emit_for_packaging_mode(module, mode)?.llvm_ir)
    }
}

pub(crate) fn verify_checkpoint(
    module: &YirModule,
    mode: &str,
    llvm: &str,
    bundle: &str,
) -> Result<(), String> {
    if resource::registration_id(mode)?.is_some() {
        resource::verify_checkpoint(module, mode, llvm, bundle)
    } else {
        scalar::verify_checkpoint_for_packaging_mode(module, mode, llvm, bundle).map(|_| ())
    }
}
