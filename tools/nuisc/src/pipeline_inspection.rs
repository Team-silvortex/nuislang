use super::{
    AstModule, CompilePipelineReport, NirModule, PipelineArtifacts, ResolvedCompileInput,
    VerifiedYirArtifacts, YirModule,
};

/// Inspection selects a checkpoint before compilation, never after a codegen failure.
pub enum InspectedPipeline {
    VerifiedYir(VerifiedYirArtifacts),
    Native(PipelineArtifacts),
}

#[derive(Clone, Copy)]
pub struct PipelineArtifactView<'a> {
    pub ast: &'a AstModule,
    pub nir: &'a NirModule,
    pub yir: &'a YirModule,
    pub loaded_nustar: &'a [String],
    pub llvm_ir: Option<&'a str>,
}

impl PipelineArtifactView<'_> {
    pub fn checkpoint_name(&self) -> &'static str {
        if self.llvm_ir.is_some() {
            "llvm-ir"
        } else {
            "verified-yir"
        }
    }

    pub fn llvm_status(&self) -> &'static str {
        if self.llvm_ir.is_some() {
            "emitted"
        } else {
            "not_requested"
        }
    }
}

impl PipelineArtifacts {
    pub fn view(&self) -> PipelineArtifactView<'_> {
        PipelineArtifactView {
            ast: &self.ast,
            nir: &self.nir,
            yir: &self.yir,
            loaded_nustar: &self.loaded_nustar,
            llvm_ir: Some(&self.llvm_ir),
        }
    }
}

impl InspectedPipeline {
    pub fn view(&self) -> PipelineArtifactView<'_> {
        match self {
            Self::Native(artifacts) => artifacts.view(),
            Self::VerifiedYir(artifacts) => PipelineArtifactView {
                ast: &artifacts.ast,
                nir: &artifacts.nir,
                yir: &artifacts.yir,
                loaded_nustar: &artifacts.loaded_nustar,
                llvm_ir: None,
            },
        }
    }

    pub fn report(&self, resolved: &ResolvedCompileInput) -> CompilePipelineReport {
        super::pipeline_report::compile_pipeline_view_report(resolved, self.view())
    }
}

impl ResolvedCompileInput {
    pub fn compile_for_inspection(&self) -> Result<InspectedPipeline, String> {
        match self.requested_packaging_mode(None)? {
            Some("headless-aot-bundle") => self
                .compile_to_verified_yir(&Default::default())
                .map(InspectedPipeline::VerifiedYir),
            Some(mode) if crate::aot_native_session::registration_id(mode)?.is_some() => {
                let checkpoint = self.compile_to_verified_yir(&Default::default())?;
                let id = crate::aot_native_session::registration_id(mode)?.unwrap();
                let llvm_ir =
                    yir_lower_llvm::native_session::emit_registered(&checkpoint.yir, id)?.llvm_ir;
                Ok(InspectedPipeline::Native(PipelineArtifacts {
                    ast: checkpoint.ast,
                    nir: checkpoint.nir,
                    yir: checkpoint.yir,
                    loaded_nustar: checkpoint.loaded_nustar,
                    llvm_ir,
                }))
            }
            _ => self.compile().map(InspectedPipeline::Native),
        }
    }
}
