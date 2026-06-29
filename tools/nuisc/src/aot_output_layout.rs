use std::path::{Path, PathBuf};

pub(crate) struct OutputLayout {
    pub(crate) ast_path: PathBuf,
    pub(crate) nir_path: PathBuf,
    pub(crate) yir_path: PathBuf,
    pub(crate) llvm_ir_path: PathBuf,
    pub(crate) shim_path: PathBuf,
    pub(crate) binary_stub_path: PathBuf,
}

pub(crate) fn output_layout(input: &Path, output_dir: &Path) -> OutputLayout {
    let stem = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("nuis_module");
    OutputLayout {
        ast_path: output_dir.join(format!("{stem}.ast.txt")),
        nir_path: output_dir.join(format!("{stem}.nir.txt")),
        yir_path: output_dir.join(format!("{stem}.yir")),
        llvm_ir_path: output_dir.join(format!("{stem}.ll")),
        shim_path: output_dir.join(format!("{stem}_shim.c")),
        binary_stub_path: output_dir.join(stem),
    }
}
