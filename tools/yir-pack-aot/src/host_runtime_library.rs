use std::{path::PathBuf, process::Command};

pub(super) fn native_blob_runtime_source() -> String {
    // Shared allocation ABI, independent of scalar/resource session admission.
    let mut source =
        "#include <stdint.h>\n#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n"
            .to_owned();
    source.push_str(super::host_text_runtime::host_text_runtime_source());
    nuis_runtime::append_c_shim_owned_blob_runtime(&mut source);
    source
}

pub(super) fn ensure_runtime_host_staticlib_built() -> Result<PathBuf, String> {
    let status = Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("yir-runtime-host")
        .status()
        .map_err(|error| format!("failed to invoke cargo build for yir-runtime-host: {error}"))?;
    if !status.success() {
        return Err("cargo build -p yir-runtime-host failed".to_owned());
    }

    let sibling_path = std::env::current_exe().ok().and_then(|path| {
        path.parent()
            .map(|parent| parent.join("libyir_runtime_host.a"))
    });
    if let Some(path) = sibling_path.filter(|path| path.exists()) {
        return Ok(path);
    }

    let debug_path = PathBuf::from("target/debug/libyir_runtime_host.a");
    if debug_path.exists() {
        return Ok(debug_path);
    }

    Err(format!(
        "expected built runtime host staticlib at `{}`",
        debug_path.display()
    ))
}
