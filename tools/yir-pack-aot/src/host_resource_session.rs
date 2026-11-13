use std::{fs, path::Path, process::Command};
use yir_core::{
    native_resource_session::{registered, MAX_BINDING_SOURCE_BYTES},
    YirModule,
};

fn bytes(name: &str, value: &str) -> String {
    let bytes = value
        .bytes()
        .map(|byte| format!("\\{byte:02X}"))
        .collect::<String>();
    format!(
        "@{name} = private constant [{} x i8] c\"{bytes}\"\n",
        value.len()
    )
}

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
        return Err("native resource packaging supports 64-bit macOS/Linux hosts".to_owned());
    }
    if source.len() as u64 > MAX_BINDING_SOURCE_BYTES {
        return Err("resource YIR exceeds static descriptor bound".to_owned());
    }
    let registration = registered(module, id)?;
    let contract = &registration.contract;
    let (version, entry, frame_contract) = if registration.result_type.is_some() {
        (
            3,
            "nuis_native_checked_resource_script_main",
            yir_core::native_resource_session::MIXED_FRAME_EXPORT_CONTRACT,
        )
    } else if registration.state_type.is_some() {
        (
            2,
            "nuis_native_mixed_resource_script_main",
            yir_core::native_resource_session::MIXED_FRAME_EXPORT_CONTRACT,
        )
    } else {
        (
            1,
            "nuis_native_resource_script_main",
            yir_core::native_resource_session::FRAME_EXPORT_CONTRACT,
        )
    };
    let bridge = yir_lower_llvm::native_resource_session::emit_registered(module, registration)?;
    let declaration = registration.canonical();
    let exports = bridge.symbols.join("\n");
    let mut callbacks = bridge.llvm_ir;
    callbacks.push_str(&bytes("resource_compiled_yir", source));
    callbacks.push_str(&bytes("resource_registration", &declaration));
    callbacks.push_str(&bytes("resource_exports", &exports));
    // Mirrors the selected typed descriptor; callbacks resolve in this static link.
    callbacks.push_str(&format!(
        "@resource_binding = private constant {{ i64, ptr, i64, ptr, i64, ptr, i64, ptr, ptr, ptr }} {{ i64 {version}, ptr @resource_compiled_yir, i64 {}, ptr @resource_registration, i64 {}, ptr @resource_exports, i64 {}, ptr @{}, ptr @{}, ptr @{} }}\n\ndefine ptr @nuis_resource_binding_v1() {{\nentry:\n  ret ptr @resource_binding\n}}\n",
        source.len(), declaration.len(), exports.len(), bridge.symbols[0], bridge.symbols[1], bridge.symbols[2]
    ));
    let host = format!("{}\ndeclare ptr @nuis_resource_binding_v1()\ndeclare i32 @{entry}(ptr, i64, ptr, i32, ptr)\n\ndefine i32 @main(i32 %argc, ptr %argv) {{\nentry:\n  %binding = call ptr @nuis_resource_binding_v1()\n  %status = call i32 @{entry}(ptr @resource_expected_yir, i64 {}, ptr %binding, i32 %argc, ptr %argv)\n  ret i32 %status\n}}\n", bytes("resource_expected_yir", source), source.len());
    let runtime = super::host_runtime_library::ensure_runtime_host_staticlib_built()?;
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("resource");
    let callbacks_path = output.join(format!("{stem}_callbacks.ll"));
    let host_path = output.join(format!("{stem}_resource_host.ll"));
    let shim_path = output.join(format!("{stem}_resource_runtime.c"));
    let binary = output.join(stem);
    fs::write(&callbacks_path, callbacks).map_err(|e| e.to_string())?;
    fs::write(&host_path, host).map_err(|e| e.to_string())?;
    fs::write(
        &shim_path,
        super::host_runtime_library::native_blob_runtime_source(),
    )
    .map_err(|e| e.to_string())?;
    let mut command = Command::new("clang");
    command
        .args([&callbacks_path, &host_path, &shim_path, &runtime])
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
        .map_err(|e| format!("resource session linker: {e}"))?;
    if !result.status.success() {
        return Err(format!(
            "resource session link failed:\n{}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    let manifest = format!(
        "module={}\ncpu_host_binary_mode=native_resource_session\nruntime_bootstrap_mode=static_native_resource_session\nnative_resource_contract={contract}\nnative_resource_identity=exact-yir-graph\nnative_resource_registration={declaration}\nnative_resource_open_export={}\nnative_resource_event_export={}\nnative_resource_close_export={}\nllvm_ir={}\ncpu_host_source={}\nruntime_host_staticlib={}\ncpu_host_binary={}\nsingle_binary=true\n",
        input.display(), bridge.symbols[0], bridge.symbols[1], bridge.symbols[2], callbacks_path.display(), host_path.display(), runtime.display(), binary.display()
    );
    fs::write(
        output.join("bundle.txt"),
        format!(
            "{manifest}native_resource_frame_contract={}\n",
            frame_contract
        ),
    )
    .map_err(|e| e.to_string())
}
