use super::host_runtime_library::ensure_runtime_host_staticlib_built;
use std::{fs, path::Path, process::Command};
use yir_core::YirModule;
use yir_lower_llvm::native_session::{emit_registered, NativeSessionBridge};

mod llvm;

/// An opt-in scalar session artifact, separate from window/provider packaging.
pub(super) fn build(
    module: &YirModule,
    source: &str,
    input: &Path,
    output: &Path,
    id: &str,
) -> Result<(), String> {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return Err(
            "native scalar session packaging currently supports 64-bit macOS/Linux hosts"
                .to_owned(),
        );
    }
    if source.len() as u64 > yir_core::native_scalar_session::MAX_BINDING_SOURCE_BYTES {
        return Err(
            "native scalar session YIR exceeds the static host descriptor bound".to_owned(),
        );
    }
    let bridge = emit_registered(module, id)?;
    let runtime = ensure_runtime_host_staticlib_built()?;
    fs::create_dir_all(output)
        .map_err(|error| format!("failed to create native session output: {error}"))?;
    let stem = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("session");
    let callbacks = output.join(format!("{stem}_callbacks.ll"));
    let host = output.join(format!("{stem}_native_host.ll"));
    let shim = output.join(format!("{stem}_scalar_runtime.c"));
    let binary = output.join(stem);
    fs::write(&callbacks, llvm::callbacks(&bridge, source)).map_err(|e| e.to_string())?;
    fs::write(&host, llvm::entry(&bridge, source)).map_err(|e| e.to_string())?;
    fs::write(&shim, runtime_source()).map_err(|e| e.to_string())?;
    let mut command = Command::new("clang");
    command
        .args([&callbacks, &host, &shim, &runtime])
        .arg("-O2");
    if cfg!(target_os = "linux") {
        command.args(["-ldl", "-lpthread", "-lm", "-lrt", "-lutil"]);
    } else {
        command.arg("-liconv");
    }
    let result = command
        .arg("-o")
        .arg(&binary)
        .output()
        .map_err(|e| format!("failed to invoke native session linker: {e}"))?;
    if !result.status.success() {
        return Err(format!(
            "native scalar session link failed:\n{}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    let mut manifest = format!(
        "module={}\ncpu_host_binary_mode=native_scalar_session\nruntime_bootstrap_mode=static_native_session\napplication_session_id={id}\nnative_session_contract={}\nnative_session_identity=exact-yir-graph\nllvm_ir={}\ncpu_host_source={}\nruntime_host_staticlib={}\ncpu_host_binary={}\nsingle_binary=true\n",
        input.display(), yir_core::native_scalar_session::CONTRACT, callbacks.display(), host.display(), runtime.display(), binary.display()
    );
    manifest.push_str(&format!(
        "native_session_layout={}\n",
        bridge.state_layout.source()
    ));
    fs::write(output.join("bundle.txt"), manifest).map_err(|e| e.to_string())
}

fn runtime_source() -> String {
    // Reuse the existing allocation ABI; this is not a new C application shim.
    let mut source =
        "#include <stdint.h>\n#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n"
            .to_owned();
    source.push_str(super::host_text_runtime::host_text_runtime_source());
    nuis_runtime::append_c_shim_owned_blob_runtime(&mut source);
    source
}
