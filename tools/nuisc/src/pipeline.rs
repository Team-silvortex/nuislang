use std::{fs, path::Path};

use nuis_semantics::model::{AstModule, NirModule};
use yir_core::YirModule;

pub struct PipelineArtifacts {
    pub ast: AstModule,
    pub nir: NirModule,
    pub yir: YirModule,
    pub llvm_ir: String,
}

pub fn compile_source_path(path: &Path) -> Result<PipelineArtifacts, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    compile_source(&source)
}

pub fn compile_source(source: &str) -> Result<PipelineArtifacts, String> {
    let ast = crate::frontend::parse_nuis_ast(source)?;
    let nir = crate::frontend::lower_ast_to_nir(&ast);
    let yir = crate::lowering::lower_nir_to_yir(&nir)?;
    let llvm_ir = yir_lower_llvm::emit_module(&yir)?;
    Ok(PipelineArtifacts {
        ast,
        nir,
        yir,
        llvm_ir,
    })
}
