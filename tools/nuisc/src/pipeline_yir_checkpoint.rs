use super::*;

/// A semantic checkpoint, not an LLVM result or a failed-codegen fallback.
/// Construction stays inside the pipeline; consumers receive read-only YIR.
pub struct VerifiedYirArtifacts {
    pub(crate) ast: AstModule,
    pub(crate) nir: NirModule,
    pub(crate) yir: YirModule,
    pub(crate) loaded_nustar: Vec<String>,
}

impl VerifiedYirArtifacts {
    pub fn yir(&self) -> &YirModule {
        &self.yir
    }

    pub(crate) fn verify(&self) -> Result<(), String> {
        pipeline_ffi_owned_buffer::validate_owned_return_buffer_yir(&self.yir)?;
        pipeline_ffi_owned_object::validate_owned_return_object_yir(&self.yir)?;
        pipeline_ffi_owned_utf8::validate_owned_return_utf8_yir(&self.yir)?;
        crate::nustar_codegen_registry::verify_module_with_loaded_nustar(
            &self.yir,
            &self.loaded_nustar,
        )
    }

    pub fn emit_llvm(self) -> Result<PipelineArtifacts, String> {
        let llvm_ir = crate::nustar_codegen_registry::emit_module_with_loaded_nustar(
            &self.yir,
            &self.loaded_nustar,
        )?;
        Ok(PipelineArtifacts {
            ast: self.ast,
            nir: self.nir,
            yir: self.yir,
            llvm_ir,
            loaded_nustar: self.loaded_nustar,
        })
    }
}
